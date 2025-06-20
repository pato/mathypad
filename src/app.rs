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
    CopyFlash,
}

impl ResultAnimation {
    pub fn new_fade_in() -> Self {
        Self {
            start_time: Instant::now(),
            duration_ms: 250, // 250ms fade-in
            animation_type: AnimationType::FadeIn,
        }
    }

    pub fn new_copy_flash() -> Self {
        Self {
            start_time: Instant::now(),
            duration_ms: 150, // 150ms flash animation - faster and snappier
            animation_type: AnimationType::CopyFlash,
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
            AnimationType::CopyFlash => {
                let progress = self.progress();
                // Flash effect: bright at start, fade to normal
                1.0 - progress
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
    pub separator_position: u16, // Position of the separator between text and results (percentage)
    pub is_dragging_separator: bool, // Whether the user is currently dragging the separator
    pub is_hovering_separator: bool, // Whether the mouse is hovering over the separator
    pub copy_flash_animations: Vec<Option<ResultAnimation>>, // Flash animations for copied lines
    pub copy_flash_is_result: Vec<bool>, // Track which panel was flashed (true = results, false = text)
    pub last_click_time: Option<Instant>, // For double-click detection
    pub last_click_position: Option<(u16, u16)>, // Last click position for double-click detection
    pub show_welcome_dialog: bool,       // Show the welcome screen for new versions
    pub welcome_scroll_offset: usize,    // Scroll position for welcome screen changelog
    pub pending_normal_command: Option<char>, // For multi-character vim commands like 'dd'
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
            mode: Mode::Insert,                // Start in insert mode
            result_animations: vec![None],     // Start with no animations
            file_path: None,                   // No file loaded initially
            has_unsaved_changes: false,        // Start with no changes
            show_unsaved_dialog: false,        // Start without showing dialog
            show_save_as_dialog: false,        // Start without showing save as dialog
            save_as_input: String::new(),      // Start with empty filename input
            save_as_and_quit: false,           // Start without quit flag
            separator_position: 80,            // Default to 80% for text, 20% for results
            is_dragging_separator: false,      // Start without dragging
            is_hovering_separator: false,      // Start without hovering
            copy_flash_animations: vec![None], // Start with no copy animations
            copy_flash_is_result: vec![false], // Start with no copy panel tracking
            last_click_time: None,             // No previous clicks
            last_click_position: None,         // No previous click position
            show_welcome_dialog: false,        // Start without showing welcome dialog
            welcome_scroll_offset: 0,          // Start at top of welcome content
            pending_normal_command: None,      // No pending vim command
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

                // Remove corresponding copy flash animation if it exists
                if self.cursor_line < self.copy_flash_animations.len() {
                    self.copy_flash_animations.remove(self.cursor_line);
                }
                if self.cursor_line < self.copy_flash_is_result.len() {
                    self.copy_flash_is_result.remove(self.cursor_line);
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

            // Also ensure copy flash animations vector is large enough
            if self.cursor_line + 1 < self.copy_flash_animations.len() {
                self.copy_flash_animations
                    .insert(self.cursor_line + 1, None);
                self.copy_flash_is_result
                    .insert(self.cursor_line + 1, false);
            } else {
                while self.copy_flash_animations.len() <= self.cursor_line + 1 {
                    self.copy_flash_animations.push(None);
                    self.copy_flash_is_result.push(false);
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

    /// Delete the entire current line (vim 'dd' command)
    pub fn delete_line(&mut self) {
        if self.text_lines.len() > 1 {
            // Update line references before deletion
            self.update_line_references_for_deletion(self.cursor_line);

            // Remove the line
            self.text_lines.remove(self.cursor_line);
            self.results.remove(self.cursor_line);

            // Remove animation states
            if self.cursor_line < self.result_animations.len() {
                self.result_animations.remove(self.cursor_line);
            }
            if self.cursor_line < self.copy_flash_animations.len() {
                self.copy_flash_animations.remove(self.cursor_line);
                self.copy_flash_is_result.remove(self.cursor_line);
            }

            // Adjust cursor position
            if self.cursor_line >= self.text_lines.len() && self.cursor_line > 0 {
                self.cursor_line -= 1;
            }
            self.cursor_col = 0;

            // Re-evaluate all lines after deletion
            for i in self.cursor_line..self.text_lines.len() {
                self.update_result(i);
            }

            self.has_unsaved_changes = true;
        } else if self.text_lines.len() == 1 {
            // If only one line, just clear it instead of deleting
            self.text_lines[0].clear();
            self.results[0] = None;
            self.cursor_col = 0;
            self.update_result(0);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete character at cursor position (vim 'x' command)
    pub fn delete_char_at_cursor(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            let line_len = self.text_lines[self.cursor_line].len();
            if self.cursor_col < line_len {
                self.text_lines[self.cursor_line].remove(self.cursor_col);
                // Adjust cursor if at end of line after deletion
                if self.cursor_col >= self.text_lines[self.cursor_line].len() && self.cursor_col > 0
                {
                    self.cursor_col = self.text_lines[self.cursor_line].len();
                }
                self.update_result(self.cursor_line);
                self.has_unsaved_changes = true;
            }
        }
    }

    /// Move cursor forward by one word (vim 'w' command)
    /// A word is a sequence of alphanumeric characters or underscores
    pub fn move_word_forward(&mut self) {
        if self.cursor_line >= self.text_lines.len() {
            return;
        }

        let line = &self.text_lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let mut new_col = self.cursor_col;

        // Skip current word if we're in one
        while new_col < chars.len() && (chars[new_col].is_alphanumeric() || chars[new_col] == '_') {
            new_col += 1;
        }

        // Skip non-word characters
        while new_col < chars.len() && !(chars[new_col].is_alphanumeric() || chars[new_col] == '_')
        {
            new_col += 1;
        }

        // If we've reached the end of the line, move to the next line
        if new_col >= chars.len() && self.cursor_line + 1 < self.text_lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            self.cursor_col = new_col;
        }
    }

    /// Move cursor backward by one word (vim 'b' command)
    pub fn move_word_backward(&mut self) {
        if self.cursor_line >= self.text_lines.len() {
            return;
        }

        let line = &self.text_lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();

        if self.cursor_col == 0 {
            // If at start of line, move to end of previous line
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = self.text_lines[self.cursor_line].len();
            }
            return;
        }

        let mut new_col = self.cursor_col;
        new_col = new_col.saturating_sub(1);

        // Skip non-word characters backwards
        while new_col > 0 && !(chars[new_col].is_alphanumeric() || chars[new_col] == '_') {
            new_col -= 1;
        }

        // Skip word characters backwards to find start of word
        while new_col > 0 && (chars[new_col - 1].is_alphanumeric() || chars[new_col - 1] == '_') {
            new_col -= 1;
        }

        self.cursor_col = new_col;
    }

    /// Move cursor forward by one WORD (vim 'W' command)
    /// A WORD is a sequence of non-whitespace characters
    pub fn move_word_forward_big(&mut self) {
        if self.cursor_line >= self.text_lines.len() {
            return;
        }

        let line = &self.text_lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let mut new_col = self.cursor_col;

        // Skip current WORD if we're in one
        while new_col < chars.len() && !chars[new_col].is_whitespace() {
            new_col += 1;
        }

        // Skip whitespace
        while new_col < chars.len() && chars[new_col].is_whitespace() {
            new_col += 1;
        }

        // If we've reached the end of the line, move to the next line
        if new_col >= chars.len() && self.cursor_line + 1 < self.text_lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            self.cursor_col = new_col;
        }
    }

    /// Move cursor backward by one WORD (vim 'B' command)
    pub fn move_word_backward_big(&mut self) {
        if self.cursor_line >= self.text_lines.len() {
            return;
        }

        let line = &self.text_lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();

        if self.cursor_col == 0 {
            // If at start of line, move to end of previous line
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = self.text_lines[self.cursor_line].len();
            }
            return;
        }

        let mut new_col = self.cursor_col;
        new_col = new_col.saturating_sub(1);

        // Skip whitespace backwards
        while new_col > 0 && chars[new_col].is_whitespace() {
            new_col -= 1;
        }

        // Skip non-whitespace backwards to find start of WORD
        while new_col > 0 && !chars[new_col - 1].is_whitespace() {
            new_col -= 1;
        }

        self.cursor_col = new_col;
    }

    /// Delete from cursor to the end of the current word (vim 'dw' command)
    pub fn delete_word_forward(&mut self) {
        if self.cursor_line >= self.text_lines.len() {
            return;
        }

        let line = &self.text_lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let start_col = self.cursor_col;
        let mut end_col = self.cursor_col;

        // Skip current word if we're in one
        while end_col < chars.len() && (chars[end_col].is_alphanumeric() || chars[end_col] == '_') {
            end_col += 1;
        }

        // Also skip trailing non-word characters (spaces, punctuation) to next word
        while end_col < chars.len() && !(chars[end_col].is_alphanumeric() || chars[end_col] == '_')
        {
            end_col += 1;
        }

        // Delete the range
        if end_col > start_col {
            self.text_lines[self.cursor_line].drain(start_col..end_col);
            self.update_result(self.cursor_line);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete from cursor to the beginning of the previous word (vim 'db' command)
    pub fn delete_word_backward(&mut self) {
        if self.cursor_line >= self.text_lines.len() || self.cursor_col == 0 {
            return;
        }

        let line = &self.text_lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let end_col = self.cursor_col;
        let mut start_col = self.cursor_col;

        start_col = start_col.saturating_sub(1);

        // Skip non-word characters backwards
        while start_col > 0 && !(chars[start_col].is_alphanumeric() || chars[start_col] == '_') {
            start_col -= 1;
        }

        // Skip word characters backwards to find start of word
        while start_col > 0
            && (chars[start_col - 1].is_alphanumeric() || chars[start_col - 1] == '_')
        {
            start_col -= 1;
        }

        // Delete the range
        if end_col > start_col {
            self.text_lines[self.cursor_line].drain(start_col..end_col);
            self.cursor_col = start_col;
            self.update_result(self.cursor_line);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete from cursor to the end of the current WORD (vim 'dW' command)
    pub fn delete_word_forward_big(&mut self) {
        if self.cursor_line >= self.text_lines.len() {
            return;
        }

        let line = &self.text_lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let start_col = self.cursor_col;
        let mut end_col = self.cursor_col;

        // Skip current WORD if we're in one
        while end_col < chars.len() && !chars[end_col].is_whitespace() {
            end_col += 1;
        }

        // Also skip trailing whitespace to next WORD
        while end_col < chars.len() && chars[end_col].is_whitespace() {
            end_col += 1;
        }

        // Delete the range
        if end_col > start_col {
            self.text_lines[self.cursor_line].drain(start_col..end_col);
            self.update_result(self.cursor_line);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete from cursor to the beginning of the previous WORD (vim 'dB' command)
    pub fn delete_word_backward_big(&mut self) {
        if self.cursor_line >= self.text_lines.len() || self.cursor_col == 0 {
            return;
        }

        let line = &self.text_lines[self.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let end_col = self.cursor_col;
        let mut start_col = self.cursor_col;

        start_col = start_col.saturating_sub(1);

        // Skip whitespace backwards
        while start_col > 0 && chars[start_col].is_whitespace() {
            start_col -= 1;
        }

        // Skip non-whitespace backwards to find start of WORD
        while start_col > 0 && !chars[start_col - 1].is_whitespace() {
            start_col -= 1;
        }

        // Delete the range
        if end_col > start_col {
            self.text_lines[self.cursor_line].drain(start_col..end_col);
            self.cursor_col = start_col;
            self.update_result(self.cursor_line);
            self.has_unsaved_changes = true;
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

        // Update copy flash animations
        for animation in &mut self.copy_flash_animations {
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

    /// Update separator position based on mouse column position
    pub fn update_separator_position(&mut self, mouse_x: u16, terminal_width: u16) {
        // Calculate percentage based on mouse position
        let percentage = ((mouse_x as f32 / terminal_width as f32) * 100.0) as u16;
        // Clamp between 20% and 80% to ensure both panels remain usable
        self.separator_position = percentage.clamp(20, 80);
    }

    /// Check if mouse position is over the separator (within a few columns for easier dragging)
    pub fn is_mouse_over_separator(&self, mouse_x: u16, terminal_width: u16) -> bool {
        let separator_x = (self.separator_position as f32 / 100.0 * terminal_width as f32) as u16;
        // Allow dragging within 2 columns of the separator
        mouse_x.abs_diff(separator_x) <= 2
    }

    /// Start dragging the separator
    pub fn start_dragging_separator(&mut self) {
        self.is_dragging_separator = true;
    }

    /// Stop dragging the separator
    pub fn stop_dragging_separator(&mut self) {
        self.is_dragging_separator = false;
    }

    /// Set hover state for the separator
    pub fn set_separator_hover(&mut self, hovering: bool) {
        self.is_hovering_separator = hovering;
    }

    /// Copy text to clipboard and start flash animation
    pub fn copy_to_clipboard(
        &mut self,
        text: &str,
        line_index: usize,
        is_result: bool,
    ) -> Result<(), String> {
        // Copy to clipboard using arboard
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
        clipboard
            .set_text(text)
            .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

        // Start flash animation for the copied line
        self.start_copy_flash_animation(line_index, is_result);

        Ok(())
    }

    /// Start a copy flash animation for a specific line
    fn start_copy_flash_animation(&mut self, line_index: usize, is_result: bool) {
        // Ensure the copy flash animations vector is large enough
        while self.copy_flash_animations.len() <= line_index {
            self.copy_flash_animations.push(None);
            self.copy_flash_is_result.push(false);
        }

        self.copy_flash_animations[line_index] = Some(ResultAnimation::new_copy_flash());
        // Ensure the tracking vector is large enough too
        while self.copy_flash_is_result.len() <= line_index {
            self.copy_flash_is_result.push(false);
        }
        self.copy_flash_is_result[line_index] = is_result;
    }

    /// Check if a click is a double-click
    pub fn is_double_click(&mut self, mouse_x: u16, mouse_y: u16) -> bool {
        let now = Instant::now();
        let double_click_threshold = std::time::Duration::from_millis(500); // 500ms double-click window
        let position_threshold = 2; // Allow 2 pixels of movement

        if let (Some(last_time), Some((last_x, last_y))) =
            (self.last_click_time, self.last_click_position)
        {
            let time_diff = now.duration_since(last_time);
            let position_diff = ((mouse_x as i32 - last_x as i32).abs()
                + (mouse_y as i32 - last_y as i32).abs()) as u16;

            if time_diff <= double_click_threshold && position_diff <= position_threshold {
                // Reset click tracking after successful double-click
                self.last_click_time = None;
                self.last_click_position = None;
                return true;
            }
        }

        // Update click tracking
        self.last_click_time = Some(now);
        self.last_click_position = Some((mouse_x, mouse_y));
        false
    }

    /// Get the copy flash animation for a specific line
    pub fn get_copy_flash_animation(&self, line_index: usize) -> Option<&ResultAnimation> {
        self.copy_flash_animations.get(line_index)?.as_ref()
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
        let mut app = App {
            text_lines: vec!["5".to_string(), "line1 + 1".to_string()],
            results: vec![None, None],
            cursor_line: 0,
            cursor_col: 0, // Position cursor at beginning of "5"
            ..Default::default()
        };

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
        let mut app = App {
            text_lines: vec!["5".to_string(), "line1 + 1".to_string()],
            results: vec![None, None],
            ..Default::default()
        };

        // Simulate exactly what the user described:
        // "i have the following notebook: 5, line1 + 1"
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
        let mut app = App {
            text_lines: vec!["".to_string(), "5".to_string(), "line2 + 1".to_string()],
            results: vec![None, None, None],
            ..Default::default()
        };

        // Set up: after line splitting we have ["", "5", "line2 + 1"]
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
        let mut app = App {
            text_lines: vec!["5".to_string(), "line1 + 1".to_string()],
            results: vec![None, None],
            ..Default::default()
        };

        // Start with the user's original notebook
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

    #[test]
    fn test_separator_position_updates() {
        let mut app = App::default();

        // Test default position
        assert_eq!(app.separator_position, 80);

        // Test updating separator position
        app.update_separator_position(400, 1000); // 40% position
        assert_eq!(app.separator_position, 40);

        // Test clamping to minimum
        app.update_separator_position(100, 1000); // 10% position - should be clamped to 20%
        assert_eq!(app.separator_position, 20);

        // Test clamping to maximum
        app.update_separator_position(900, 1000); // 90% position - should be clamped to 80%
        assert_eq!(app.separator_position, 80);
    }

    #[test]
    fn test_mouse_over_separator_detection() {
        let app = App::default(); // 80% separator position
        let terminal_width = 1000;
        let separator_x = 800; // 80% of 1000

        // Test exact position
        assert!(app.is_mouse_over_separator(separator_x, terminal_width));

        // Test within range (±2 columns)
        assert!(app.is_mouse_over_separator(separator_x - 2, terminal_width));
        assert!(app.is_mouse_over_separator(separator_x + 2, terminal_width));

        // Test outside range
        assert!(!app.is_mouse_over_separator(separator_x - 3, terminal_width));
        assert!(!app.is_mouse_over_separator(separator_x + 3, terminal_width));
    }

    #[test]
    fn test_separator_dragging_state() {
        let mut app = App::default();

        // Test initial state
        assert!(!app.is_dragging_separator);

        // Test starting drag
        app.start_dragging_separator();
        assert!(app.is_dragging_separator);

        // Test stopping drag
        app.stop_dragging_separator();
        assert!(!app.is_dragging_separator);
    }

    #[test]
    fn test_separator_hover_state() {
        let mut app = App::default();

        // Test initial state
        assert!(!app.is_hovering_separator);

        // Test starting hover
        app.set_separator_hover(true);
        assert!(app.is_hovering_separator);

        // Test stopping hover
        app.set_separator_hover(false);
        assert!(!app.is_hovering_separator);
    }

    #[test]
    fn test_double_click_detection() {
        let mut app = App::default();

        // First click should not be a double-click
        assert!(!app.is_double_click(100, 100));

        // Immediate second click should be a double-click
        assert!(app.is_double_click(100, 100));

        // After double-click, state should be reset
        assert!(!app.is_double_click(100, 100));
    }

    #[test]
    fn test_double_click_position_threshold() {
        let mut app = App::default();

        // First click
        assert!(!app.is_double_click(100, 100));

        // Click within threshold should be double-click
        assert!(app.is_double_click(101, 101));

        // Reset state
        app.last_click_time = None;
        app.last_click_position = None;

        // First click
        assert!(!app.is_double_click(100, 100));

        // Click outside threshold should not be double-click
        assert!(!app.is_double_click(110, 110));
    }

    #[test]
    fn test_copy_flash_animation() {
        let mut app = App {
            text_lines: vec!["test".to_string(), "test2".to_string()],
            copy_flash_animations: vec![None, None],
            ..Default::default()
        };

        // Ensure we have enough lines

        // Test copy to clipboard (we can't actually test clipboard, but we can test the animation)
        app.start_copy_flash_animation(0, false);

        // Should have flash animation
        assert!(app.get_copy_flash_animation(0).is_some());
        assert!(app.get_copy_flash_animation(1).is_none());

        // Animation should be of correct type
        let animation = app.get_copy_flash_animation(0).unwrap();
        assert!(matches!(
            animation.animation_type,
            crate::app::AnimationType::CopyFlash
        ));
    }

    #[test]
    fn test_delete_line() {
        let mut app = App {
            text_lines: vec![
                "first".to_string(),
                "second".to_string(),
                "third".to_string(),
            ],
            results: vec![None, None, None],
            result_animations: vec![None, None, None],
            copy_flash_animations: vec![None, None, None],
            copy_flash_is_result: vec![false, false, false],
            cursor_line: 1,
            ..Default::default()
        };

        // Delete middle line
        app.delete_line();
        assert_eq!(app.text_lines, vec!["first", "third"]);
        assert_eq!(app.cursor_line, 1);
        assert_eq!(app.cursor_col, 0);

        // Delete last line
        app.delete_line();
        assert_eq!(app.text_lines, vec!["first"]);
        assert_eq!(app.cursor_line, 0);

        // Try to delete only line - should just clear it
        app.delete_line();
        assert_eq!(app.text_lines, vec![""]);
        assert_eq!(app.cursor_line, 0);
    }

    #[test]
    fn test_delete_char_at_cursor() {
        let mut app = App {
            text_lines: vec!["hello world".to_string()],
            results: vec![None],
            cursor_line: 0,
            cursor_col: 6, // at 'w'
            ..Default::default()
        };

        app.delete_char_at_cursor();
        assert_eq!(app.text_lines[0], "hello orld");
        assert_eq!(app.cursor_col, 6);

        // Delete at end of line should do nothing
        app.cursor_col = app.text_lines[0].len();
        app.delete_char_at_cursor();
        assert_eq!(app.text_lines[0], "hello orld");
    }

    #[test]
    fn test_word_movement_forward() {
        let mut app = App {
            text_lines: vec!["hello world test_var 123".to_string()],
            cursor_line: 0,
            cursor_col: 0,
            ..Default::default()
        };

        // Move from start to 'world'
        app.move_word_forward();
        assert_eq!(app.cursor_col, 6);

        // Move to 'test_var'
        app.move_word_forward();
        assert_eq!(app.cursor_col, 12);

        // Move to '123'
        app.move_word_forward();
        assert_eq!(app.cursor_col, 21);

        // Try to move past end - should go to end of line
        app.move_word_forward();
        assert_eq!(app.cursor_col, 24); // Should be at end of line
    }

    #[test]
    fn test_word_movement_backward() {
        let mut app = App {
            text_lines: vec!["hello world test_var".to_string()],
            cursor_line: 0,
            cursor_col: 20, // at end
            ..Default::default()
        };

        // Move to start of 'test_var'
        app.move_word_backward();
        assert_eq!(app.cursor_col, 12);

        // Move to start of 'world'
        app.move_word_backward();
        assert_eq!(app.cursor_col, 6);

        // Move to start of 'hello'
        app.move_word_backward();
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn test_word_movement_big() {
        let mut app = App {
            text_lines: vec!["hello-world test::func()".to_string()],
            cursor_line: 0,
            cursor_col: 0,
            ..Default::default()
        };

        // 'hello-world' is one WORD
        app.move_word_forward_big();
        assert_eq!(app.cursor_col, 12);

        // Move backward
        app.cursor_col = 24;
        app.move_word_backward_big();
        assert_eq!(app.cursor_col, 12);

        app.move_word_backward_big();
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn test_delete_word_forward() {
        let mut app = App {
            text_lines: vec!["hello world test".to_string()],
            results: vec![None],
            cursor_line: 0,
            cursor_col: 0,
            ..Default::default()
        };

        // Delete 'hello ' (word + trailing space)
        app.delete_word_forward();
        assert_eq!(app.text_lines[0], "world test");
        assert_eq!(app.cursor_col, 0);

        // Delete from middle of word
        app.cursor_col = 2; // in 'world'
        app.delete_word_forward();
        assert_eq!(app.text_lines[0], "wotest");
    }

    #[test]
    fn test_delete_word_backward() {
        let mut app = App {
            text_lines: vec!["hello world test".to_string()],
            results: vec![None],
            cursor_line: 0,
            cursor_col: 11, // at space after 'world'
            ..Default::default()
        };

        // Delete 'world'
        app.delete_word_backward();
        assert_eq!(app.text_lines[0], "hello  test");
        assert_eq!(app.cursor_col, 6);
    }

    #[test]
    fn test_delete_word_forward_big() {
        let mut app = App {
            text_lines: vec!["hello-world test::func()".to_string()],
            results: vec![None],
            cursor_line: 0,
            cursor_col: 0,
            ..Default::default()
        };

        // Delete 'hello-world ' (WORD + trailing space)
        app.delete_word_forward_big();
        assert_eq!(app.text_lines[0], "test::func()");
        assert_eq!(app.cursor_col, 0);
    }

    #[test]
    fn test_pending_normal_command() {
        let mut app = App {
            pending_normal_command: Some('d'),
            ..Default::default()
        };

        // Should be able to set and read pending command
        assert_eq!(app.pending_normal_command, Some('d'));

        // Clear it
        app.pending_normal_command = None;
        assert_eq!(app.pending_normal_command, None);
    }
}
