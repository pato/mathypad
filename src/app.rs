//! Application state and core logic

use crate::{
    Mode,
    expression::{evaluate_with_variables, update_line_references_in_text},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// Animation state for a result line
#[derive(Clone, Debug)]
pub struct ResultAnimation {
    pub start_time: Instant,
    pub duration_ms: u64,
    pub animation_type: AnimationType,
}

#[derive(Clone, Debug)]
pub enum AnimationType {
    FadeIn,
}

impl ResultAnimation {
    pub fn new_fade_in() -> Self {
        Self {
            start_time: Instant::now(),
            duration_ms: 250, // 250ms fade-in
            animation_type: AnimationType::FadeIn,
        }
    }

    /// Get the animation progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_millis() as f32;
        let progress = elapsed / self.duration_ms as f32;
        progress.min(1.0)
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    /// Get the opacity for fade-in animation (0.0 to 1.0)
    pub fn opacity(&self) -> f32 {
        match self.animation_type {
            AnimationType::FadeIn => {
                let progress = self.progress();
                // Smooth ease-out animation
                1.0 - (1.0 - progress).powi(3)
            }
        }
    }
}

/// Main application state for the mathematical notepad
pub struct App {
    pub text_lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub results: Vec<Option<String>>,
    pub variables: HashMap<String, String>, // variable_name -> value_string
    pub mode: Mode,
    pub result_animations: Vec<Option<ResultAnimation>>, // Animation state for each result
    pub file_path: Option<PathBuf>,                      // Path to the currently opened file
    pub has_unsaved_changes: bool,                       // Track if there are unsaved changes
    pub show_unsaved_dialog: bool,                       // Show the unsaved changes dialog
    pub show_save_as_dialog: bool,                       // Show the save as dialog
    pub save_as_input: String,                           // Current input for save as filename
    pub save_as_and_quit: bool, // Whether to quit after saving in save as dialog
}

impl Default for App {
    fn default() -> App {
        App {
            text_lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            results: vec![None],
            variables: HashMap::new(),
            mode: Mode::Insert,            // Start in insert mode
            result_animations: vec![None], // Start with no animations
            file_path: None,               // No file loaded initially
            has_unsaved_changes: false,    // Start with no changes
            show_unsaved_dialog: false,    // Start without showing dialog
            show_save_as_dialog: false,    // Start without showing save as dialog
            save_as_input: String::new(),  // Start with empty filename input
            save_as_and_quit: false,       // Start without quit flag
        }
    }
}

impl App {
    #[cfg(test)]
    pub fn test_scenario_line_splitting(&mut self) -> (String, String) {
        // Set up the scenario: "5" on line 1, "line1 + 1" on line 2
        self.text_lines = vec!["5".to_string(), "line1 + 1".to_string()];
        self.results = vec![None, None];
        self.cursor_line = 0;
        self.cursor_col = 0; // Position cursor OVER the "5" (at beginning)

        // Record the original state
        let before = format!(
            "Line 1: '{}', Line 2: '{}'",
            self.text_lines[0], self.text_lines[1]
        );

        // Hit enter when cursor is over the 5 (actually after it)
        self.new_line();

        // Record the new state
        let after = format!(
            "Line 1: '{}', Line 2: '{}', Line 3: '{}'",
            self.text_lines.first().unwrap_or(&"".to_string()),
            self.text_lines.get(1).unwrap_or(&"".to_string()),
            self.text_lines.get(2).unwrap_or(&"".to_string())
        );

        (before, after)
    }
    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        if self.cursor_line < self.text_lines.len() {
            self.text_lines[self.cursor_line].insert(self.cursor_col, c);
            self.cursor_col += 1;
            self.update_result(self.cursor_line);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete the character before the cursor
    pub fn delete_char(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            if self.cursor_col > 0 {
                // Delete character within the current line
                self.text_lines[self.cursor_line].remove(self.cursor_col - 1);
                self.cursor_col -= 1;
                self.update_result(self.cursor_line);
                self.has_unsaved_changes = true;
            } else if self.cursor_line > 0 {
                // Cursor is at beginning of line - merge with previous line
                let current_line = self.text_lines.remove(self.cursor_line);
                self.results.remove(self.cursor_line);

                // Remove corresponding animation if it exists
                if self.cursor_line < self.result_animations.len() {
                    self.result_animations.remove(self.cursor_line);
                }

                // Check if the previous line is empty - if so, we conceptually want to delete
                // the previous line rather than the current line
                let prev_line_empty = self.text_lines[self.cursor_line - 1].is_empty();

                if prev_line_empty && !current_line.is_empty() {
                    // Previous line is empty, current line has content
                    // Delete the previous line (conceptually what the user wants)
                    self.text_lines[self.cursor_line - 1] = current_line;
                    self.update_line_references_for_deletion(self.cursor_line - 1);
                    self.cursor_line -= 1;
                    self.cursor_col = 0;
                } else {
                    // Normal case: merge current line into previous line
                    self.update_line_references_for_deletion(self.cursor_line);
                    self.cursor_line -= 1;
                    self.cursor_col = self.text_lines[self.cursor_line].len();
                    self.text_lines[self.cursor_line].push_str(&current_line);
                }

                // Re-evaluate all lines after deletion to ensure line references resolve correctly
                self.update_result(self.cursor_line);
                for i in (self.cursor_line + 1)..self.text_lines.len() {
                    self.update_result(i);
                }
                self.has_unsaved_changes = true;
            }
        }
    }

    /// Delete the word before the cursor (Ctrl+W behavior)
    pub fn delete_word(&mut self) {
        if self.cursor_line < self.text_lines.len() && self.cursor_col > 0 {
            let line = &self.text_lines[self.cursor_line];
            let mut new_col = self.cursor_col;

            // Skip trailing whitespace
            while new_col > 0 && line.chars().nth(new_col - 1).unwrap_or(' ').is_whitespace() {
                new_col -= 1;
            }

            // Delete word characters (alphanumeric and underscore)
            while new_col > 0 {
                let ch = line.chars().nth(new_col - 1).unwrap_or(' ');
                if ch.is_alphanumeric() || ch == '_' {
                    new_col -= 1;
                } else {
                    break;
                }
            }

            // If no word was found, delete non-word characters until whitespace
            if new_col == self.cursor_col {
                while new_col > 0 {
                    let ch = line.chars().nth(new_col - 1).unwrap_or(' ');
                    if ch.is_whitespace() || ch.is_alphanumeric() || ch == '_' {
                        break;
                    }
                    new_col -= 1;
                }
            }

            // Delete the characters from new_col to cursor_col
            if new_col < self.cursor_col {
                self.text_lines[self.cursor_line].drain(new_col..self.cursor_col);
                self.cursor_col = new_col;
                self.update_result(self.cursor_line);
                self.has_unsaved_changes = true;
            }
        }
    }

    /// Insert a new line at the cursor position
    pub fn new_line(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            let current_line = self.text_lines[self.cursor_line].clone();
            let (left, right) = current_line.split_at(self.cursor_col);
            self.text_lines[self.cursor_line] = left.to_string();
            self.text_lines
                .insert(self.cursor_line + 1, right.to_string());
            self.results.insert(self.cursor_line + 1, None);

            // Insert corresponding empty animation slot
            if self.cursor_line + 1 < self.result_animations.len() {
                self.result_animations.insert(self.cursor_line + 1, None);
            } else {
                // Ensure animations vector is large enough
                while self.result_animations.len() <= self.cursor_line + 1 {
                    self.result_animations.push(None);
                }
            }

            // Handle line reference updates for insertion
            let insertion_point = self.cursor_line + 1; // 0-based index of newly inserted line

            // Handle line reference updates for line splitting
            // When splitting, we need to consider where content ends up
            let left_empty = left.trim().is_empty();
            let right_empty = right.trim().is_empty();

            if left_empty && !right_empty {
                // Content moved from cursor_line to insertion_point
                // Use combined update that handles both content move and position shifts
                self.update_line_references_for_line_split_with_content_move(
                    self.cursor_line,
                    insertion_point,
                );
            } else {
                // Standard insertion: just shift references
                self.update_line_references_for_standard_insertion(insertion_point);
            }

            self.cursor_line += 1;
            self.cursor_col = 0;

            // Make sure to evaluate all lines in the correct order
            // First evaluate the lines that were directly affected by the split
            self.update_result(self.cursor_line - 1); // Line 0
            self.update_result(self.cursor_line); // Line 1

            // Then re-evaluate any lines that had their references updated
            // This ensures line references can resolve correctly
            for i in (self.cursor_line + 1)..self.text_lines.len() {
                self.update_result(i);
            }
            self.has_unsaved_changes = true;
        }
    }

    /// Move cursor up one line
    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.text_lines[self.cursor_line].len());
        }
    }

    /// Move cursor down one line
    pub fn move_cursor_down(&mut self) {
        if self.cursor_line + 1 < self.text_lines.len() {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.text_lines[self.cursor_line].len());
        }
    }

    /// Move cursor left one character
    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    /// Move cursor right one character
    pub fn move_cursor_right(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            self.cursor_col = (self.cursor_col + 1).min(self.text_lines[self.cursor_line].len());
        }
    }

    /// Update the calculation result for a given line
    pub fn update_result(&mut self, line_index: usize) {
        if line_index < self.text_lines.len() && line_index < self.results.len() {
            let line = &self.text_lines[line_index];
            let (result, variable_assignment) =
                evaluate_with_variables(line, &self.variables, &self.results, line_index);

            // Check if result has changed to trigger animation
            let result_changed = self.results[line_index] != result;

            // If this is a variable assignment, store it and re-evaluate dependent lines
            if let Some((var_name, var_value)) = variable_assignment {
                // Check if this variable assignment actually changed the value
                let variable_changed = self.variables.get(&var_name) != Some(&var_value);

                self.variables.insert(var_name.clone(), var_value);

                // If the variable changed, re-evaluate all lines that might use this variable
                if variable_changed {
                    self.results[line_index] = result.clone();
                    if result_changed && result.is_some() {
                        self.start_result_animation(line_index);
                    }
                    self.re_evaluate_dependent_lines(&var_name, line_index);
                    return;
                }
            }

            self.results[line_index] = result.clone();

            // Start fade-in animation if result changed and is not None
            if result_changed && result.is_some() {
                self.start_result_animation(line_index);
            }
        } else {
            // This should never happen in normal operation, but let's be defensive
            eprintln!(
                "Warning: Attempted to update result for invalid line index {}",
                line_index
            );
        }
    }

    /// Re-evaluate all lines that might depend on the given variable
    fn re_evaluate_dependent_lines(&mut self, changed_variable: &str, assignment_line: usize) {
        // Re-evaluate all lines after the assignment line that might use this variable
        for line_idx in (assignment_line + 1)..self.text_lines.len() {
            if line_idx < self.results.len() {
                let line = &self.text_lines[line_idx];

                // Check if this line contains the variable name
                // This is a simple heuristic - we could make it more sophisticated
                if line.contains(changed_variable) {
                    let (result, nested_assignment) =
                        evaluate_with_variables(line, &self.variables, &self.results, line_idx);

                    // Handle nested variable assignments (variables that depend on other variables)
                    if let Some((nested_var_name, nested_var_value)) = nested_assignment {
                        let nested_changed =
                            self.variables.get(&nested_var_name) != Some(&nested_var_value);
                        self.variables
                            .insert(nested_var_name.clone(), nested_var_value);

                        // If this nested assignment changed, recursively update its dependents
                        if nested_changed {
                            self.results[line_idx] = result;
                            self.re_evaluate_dependent_lines(&nested_var_name, line_idx);
                        }
                    } else {
                        self.results[line_idx] = result;
                    }
                }
            }
        }
    }

    /// Update line references in all lines when a line is deleted
    /// All references > deleted_line need to be decremented by 1
    /// References to the deleted line become invalid
    fn update_line_references_for_deletion(&mut self, deleted_line: usize) {
        for i in 0..self.text_lines.len() {
            let updated_text =
                update_line_references_in_text(&self.text_lines[i], deleted_line, -1);
            if updated_text != self.text_lines[i] {
                self.text_lines[i] = updated_text;
                // Re-evaluate this line since its content changed
                self.update_result(i);
            }
        }
    }

    /// Update line references in all lines when a new line is inserted
    /// All references >= insertion_point need to be incremented by 1
    fn update_line_references_for_standard_insertion(&mut self, insertion_point: usize) {
        for i in 0..self.text_lines.len() {
            let updated_text =
                update_line_references_in_text(&self.text_lines[i], insertion_point, 1);
            if updated_text != self.text_lines[i] {
                self.text_lines[i] = updated_text;
                // Re-evaluate this line since its content changed
                self.update_result(i);
            }
        }
    }

    /// Combined update for line splitting with content movement
    /// Handles both content following and position-based shifts in one pass
    fn update_line_references_for_line_split_with_content_move(
        &mut self,
        content_from: usize,
        insertion_point: usize,
    ) {
        use crate::expression::extract_line_references;

        for i in 0..self.text_lines.len() {
            let references = extract_line_references(&self.text_lines[i]);
            let mut updated_text = self.text_lines[i].clone();

            // Process references in reverse order to maintain correct string positions
            for (start_pos, end_pos, line_num) in references.into_iter().rev() {
                let new_line_num = if line_num == content_from {
                    // Content moved from content_from to insertion_point
                    insertion_point
                } else if line_num >= insertion_point {
                    // Standard position shift for lines >= insertion_point
                    line_num + 1
                } else {
                    // Lines before insertion_point stay unchanged
                    line_num
                };

                if new_line_num != line_num {
                    let new_ref = format!("line{}", new_line_num + 1); // +1 for 1-based display
                    updated_text.replace_range(start_pos..end_pos, &new_ref);
                }
            }

            if updated_text != self.text_lines[i] {
                self.text_lines[i] = updated_text;
                // Re-evaluate this line since its content changed
                self.update_result(i);
            }
        }
    }

    /// Recalculate all lines in the notebook
    pub fn recalculate_all(&mut self) {
        // Clear variables to ensure fresh calculation
        self.variables.clear();

        // Recalculate each line in order
        for i in 0..self.text_lines.len() {
            self.update_result(i);
        }
    }

    /// Start a fade-in animation for a result
    fn start_result_animation(&mut self, line_index: usize) {
        // Ensure the animations vector is large enough
        while self.result_animations.len() <= line_index {
            self.result_animations.push(None);
        }

        // Only start animation if there's actually a result
        if line_index < self.results.len() && self.results[line_index].is_some() {
            self.result_animations[line_index] = Some(ResultAnimation::new_fade_in());
        }
    }

    /// Update all animations and remove completed ones
    pub fn update_animations(&mut self) {
        for animation in &mut self.result_animations {
            if let Some(anim) = animation {
                if anim.is_complete() {
                    *animation = None;
                }
            }
        }
    }

    /// Get the animation for a specific line
    pub fn get_result_animation(&self, line_index: usize) -> Option<&ResultAnimation> {
        self.result_animations.get(line_index)?.as_ref()
    }

    /// Save the current content to the file
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.file_path {
            use std::fs;
            let content = self.text_lines.join("\n");
            fs::write(path, content)?;
            self.has_unsaved_changes = false;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path set",
            ))
        }
    }

    /// Save the current content to a new file
    pub fn save_as(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        use std::fs;
        let content = self.text_lines.join("\n");
        fs::write(&path, content)?;
        self.file_path = Some(path);
        self.has_unsaved_changes = false;
        Ok(())
    }

    /// Set the file path (used when loading a file)
    pub fn set_file_path(&mut self, path: Option<PathBuf>) {
        self.file_path = path;
        self.has_unsaved_changes = false;
    }

    /// Show the save as dialog
    pub fn show_save_as_dialog(&mut self, quit_after_save: bool) {
        self.show_save_as_dialog = true;
        self.save_as_and_quit = quit_after_save;
        self.save_as_input = ".pad".to_string();
    }

    /// Try to save with the current save-as filename
    pub fn save_as_from_dialog(&mut self) -> Result<bool, std::io::Error> {
        if !self.save_as_input.trim().is_empty() {
            let path = PathBuf::from(self.save_as_input.trim());
            self.save_as(path)?;
            self.show_save_as_dialog = false;

            let should_quit = self.save_as_and_quit;
            self.save_as_and_quit = false;
            Ok(should_quit)
        } else {
            // Empty filename, don't save
            Ok(false)
        }
    }
}

#[cfg(test)]
mod app_tests {
    use super::*;

    #[test]
    fn test_line_splitting_with_line_references() {
        let mut app = App::default();
        let (before, after) = app.test_scenario_line_splitting();

        println!("Before: {}", before);
        println!("After: {}", after);

        // Expected behavior analysis:
        // Original: Line 1: "5", Line 2: "line1 + 1"
        // After hitting enter at position 1 in "5":
        // Line 1: "5", Line 2: "", Line 3: "line1 + 1" (shifted down from line 2)
        //
        // The question is: should "line1" in "line1 + 1" stay as "line1" or become "line2"?
        // User's expectation: it should become "line2" because the line that was originally
        // at position 2 is now at position 3, so references to lines >= 2 should be incremented.

        // Debug the actual state
        println!("App state after split:");
        for (i, line) in app.text_lines.iter().enumerate() {
            println!("  Line {}: '{}'", i + 1, line);
        }

        // The real issue: we're inserting at position 1, so lines at position 1 and after
        // should get their references incremented. "line1 + 1" was originally at position 1,
        // is now at position 2, and "line1" in it should become "line2" because ALL lines
        // shift down after insertion point.
        assert!(
            app.text_lines[2].contains("line2"),
            "Expected 'line1' to be updated to 'line2' but got: '{}'",
            app.text_lines[2]
        );
    }

    #[test]
    fn test_line_splitting_at_beginning() {
        let mut app = App::default();
        // Set up: "5" on line 1, "line1 + 1" on line 2
        app.text_lines = vec!["5".to_string(), "line1 + 1".to_string()];
        app.results = vec![None, None];
        app.cursor_line = 0;
        app.cursor_col = 0; // Position cursor at beginning of "5"

        app.new_line();

        // When hitting enter at beginning:
        // Line 1: "" (empty), Line 2: "5" (content moved down), Line 3: "line2 + 1"
        assert_eq!(app.text_lines[0], "");
        assert_eq!(app.text_lines[1], "5");
        assert!(
            app.text_lines[2].contains("line2"),
            "Expected 'line1' to be updated to 'line2' but got: '{}'",
            app.text_lines[2]
        );
    }

    #[test]
    fn test_user_reported_scenario() {
        let mut app = App::default();

        // Simulate exactly what the user described:
        // "i have the following notebook: 5, line1 + 1"
        app.text_lines = vec!["5".to_string(), "line1 + 1".to_string()];
        app.results = vec![None, None];
        app.update_result(0); // This should make line 1 result in "5"
        app.update_result(1); // This should make line 2 result in "6" (5 + 1)

        // Verify initial results work correctly
        assert_eq!(app.results[0], Some("5".to_string()));
        assert_eq!(app.results[1], Some("6".to_string()));

        // Now simulate "hit enter when the cursor is over the 5"
        app.cursor_line = 0;
        app.cursor_col = 0; // Position cursor at the beginning of "5"
        app.new_line();

        // Verify the results:
        // Line 1: "" (empty)
        // Line 2: "5"
        // Line 3: "line2 + 1" (updated reference)
        assert_eq!(app.text_lines[0], "");
        assert_eq!(app.text_lines[1], "5");
        assert!(
            app.text_lines[2].contains("line2"),
            "Expected line reference to be updated to 'line2', got: '{}'",
            app.text_lines[2]
        );

        // The key test: verify that the expression evaluates correctly automatically
        // line2 should now refer to "5" on line 2, so "line2 + 1" should be 6
        assert_eq!(
            app.results[2],
            Some("6".to_string()),
            "Expected 'line2 + 1' to evaluate to 6 automatically, got: {:?}",
            app.results[2]
        );
    }

    #[test]
    fn test_deletion_with_line_references() {
        let mut app = App::default();

        // Set up: after line splitting we have ["", "5", "line2 + 1"]
        app.text_lines = vec!["".to_string(), "5".to_string(), "line2 + 1".to_string()];
        app.results = vec![None, None, None];
        app.update_result(0);
        app.update_result(1);
        app.update_result(2);

        // Verify initial state works
        assert_eq!(app.results[1], Some("5".to_string()));
        assert_eq!(app.results[2], Some("6".to_string()));

        // Now delete the empty first line by positioning cursor at beginning of line 2 and hitting backspace
        app.cursor_line = 1;
        app.cursor_col = 0;
        app.delete_char(); // This should merge lines and update references

        // Expected result: ["5", "line1 + 1"] (reference should go back to line1)
        assert_eq!(app.text_lines.len(), 2);
        assert_eq!(app.text_lines[0], "5");
        assert!(
            app.text_lines[1].contains("line1"),
            "Expected 'line2' to be updated back to 'line1', got: '{}'",
            app.text_lines[1]
        );

        // The critical test: expression should still evaluate correctly
        assert_eq!(
            app.results[1],
            Some("6".to_string()),
            "Expected 'line1 + 1' to evaluate to 6 after deletion, got: {:?}",
            app.results[1]
        );
    }

    #[test]
    fn test_full_user_workflow_add_then_remove_lines() {
        let mut app = App::default();

        // Start with the user's original notebook
        app.text_lines = vec!["5".to_string(), "line1 + 1".to_string()];
        app.results = vec![None, None];
        app.update_result(0);
        app.update_result(1);

        // Verify initial state: 5, line1 + 1 = 6
        assert_eq!(app.results[0], Some("5".to_string()));
        assert_eq!(app.results[1], Some("6".to_string()));

        // Step 1: Hit enter at beginning of "5" (add lines)
        app.cursor_line = 0;
        app.cursor_col = 0;
        app.new_line();

        // Should have: ["", "5", "line2 + 1"] with line2 + 1 = 6
        assert_eq!(app.text_lines.len(), 3);
        assert_eq!(app.text_lines[0], "");
        assert_eq!(app.text_lines[1], "5");
        assert!(app.text_lines[2].contains("line2"));
        assert_eq!(app.results[2], Some("6".to_string()));

        // Step 2: Remove the empty line (delete lines)
        app.cursor_line = 1;
        app.cursor_col = 0;
        app.delete_char();

        // Should be back to: ["5", "line1 + 1"] with line1 + 1 = 6
        assert_eq!(app.text_lines.len(), 2);
        assert_eq!(app.text_lines[0], "5");
        assert!(app.text_lines[1].contains("line1"));
        assert_eq!(
            app.results[1],
            Some("6".to_string()),
            "Full workflow failed: expected line1 + 1 = 6 after add/remove cycle"
        );
    }
}
