//! Application state and core logic

use crate::Mode;
use mathypad_core::core::MathypadCore;
use mathypad_core::expression::update_line_references_in_text;
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
    /// Core calculation and text state (shared with web UI)
    pub core: MathypadCore,
    pub scroll_offset: usize,
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
    pub command_line: String,            // Current command line input (starts with ':')
    pub command_cursor: usize,           // Cursor position in command line
}

impl Default for App {
    fn default() -> App {
        App {
            core: MathypadCore::new(),
            scroll_offset: 0,
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
            command_line: String::new(),       // Start with empty command line
            command_cursor: 0,                 // Start cursor at beginning of command line
        }
    }
}

impl App {
    #[cfg(test)]
    pub fn test_scenario_line_splitting(&mut self) -> (String, String) {
        // Set up the scenario: "5" on line 1, "line1 + 1" on line 2
        self.core.text_lines = vec!["5".to_string(), "line1 + 1".to_string()];
        self.core.results = vec![None, None];
        self.core.cursor_line = 0;
        self.core.cursor_col = 0; // Position cursor OVER the "5" (at beginning)

        // Record the original state
        let before = format!(
            "Line 1: '{}', Line 2: '{}'",
            self.core.text_lines[0], self.core.text_lines[1]
        );

        // Hit enter when cursor is over the 5 (actually after it)
        self.new_line();

        // Record the new state
        let after = format!(
            "Line 1: '{}', Line 2: '{}', Line 3: '{}'",
            self.core.text_lines.first().unwrap_or(&"".to_string()),
            self.core.text_lines.get(1).unwrap_or(&"".to_string()),
            self.core.text_lines.get(2).unwrap_or(&"".to_string())
        );

        (before, after)
    }
    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        self.core.insert_char(c);
        self.has_unsaved_changes = true;
    }

    /// Delete the character before the cursor
    pub fn delete_char(&mut self) {
        if self.core.cursor_line < self.core.text_lines.len() {
            if self.core.cursor_col > 0 {
                // Delete character within the current line
                let line = &mut self.core.text_lines[self.core.cursor_line];

                // Find the byte index of the character to delete
                let char_indices: Vec<_> = line.char_indices().collect();
                if self.core.cursor_col > 0 && self.core.cursor_col <= char_indices.len() {
                    // We want to delete the character before the cursor (at cursor_col - 1)
                    let char_to_delete_idx = self.core.cursor_col - 1;

                    // Get the byte range of this character
                    let start_byte = char_indices[char_to_delete_idx].0;
                    let end_byte = if char_to_delete_idx + 1 < char_indices.len() {
                        char_indices[char_to_delete_idx + 1].0
                    } else {
                        line.len()
                    };

                    // Remove the character using drain
                    line.drain(start_byte..end_byte);
                    self.core.cursor_col -= 1;
                    self.update_result(self.core.cursor_line);
                    self.has_unsaved_changes = true;
                }
            } else if self.core.cursor_line > 0 {
                // Cursor is at beginning of line - merge with previous line
                let current_line = self.core.text_lines.remove(self.core.cursor_line);
                self.core.results.remove(self.core.cursor_line);

                // Remove corresponding animation if it exists
                if self.core.cursor_line < self.result_animations.len() {
                    self.result_animations.remove(self.core.cursor_line);
                }

                // Remove corresponding copy flash animation if it exists
                if self.core.cursor_line < self.copy_flash_animations.len() {
                    self.copy_flash_animations.remove(self.core.cursor_line);
                }
                if self.core.cursor_line < self.copy_flash_is_result.len() {
                    self.copy_flash_is_result.remove(self.core.cursor_line);
                }

                // Check if the previous line is empty - if so, we conceptually want to delete
                // the previous line rather than the current line
                let prev_line_empty = self.core.text_lines[self.core.cursor_line - 1].is_empty();

                if prev_line_empty && !current_line.is_empty() {
                    // Previous line is empty, current line has content
                    // Delete the previous line (conceptually what the user wants)
                    self.core.text_lines[self.core.cursor_line - 1] = current_line;
                    self.update_line_references_for_deletion(self.core.cursor_line - 1);
                    self.core.cursor_line -= 1;
                    self.core.cursor_col = 0;
                } else {
                    // Normal case: merge current line into previous line
                    self.update_line_references_for_deletion(self.core.cursor_line);
                    self.core.cursor_line -= 1;
                    self.core.cursor_col =
                        self.core.text_lines[self.core.cursor_line].chars().count();
                    self.core.text_lines[self.core.cursor_line].push_str(&current_line);
                }

                // Re-evaluate all lines after deletion to ensure line references resolve correctly
                self.update_result(self.core.cursor_line);
                for i in (self.core.cursor_line + 1)..self.core.text_lines.len() {
                    self.update_result(i);
                }
                self.has_unsaved_changes = true;
            }
        }
    }

    /// Delete the word before the cursor (Ctrl+W behavior)
    pub fn delete_word(&mut self) {
        if self.core.cursor_line < self.core.text_lines.len() && self.core.cursor_col > 0 {
            let line = &self.core.text_lines[self.core.cursor_line];
            let mut new_col = self.core.cursor_col;

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
            if new_col == self.core.cursor_col {
                while new_col > 0 {
                    let ch = line.chars().nth(new_col - 1).unwrap_or(' ');
                    if ch.is_whitespace() || ch.is_alphanumeric() || ch == '_' {
                        break;
                    }
                    new_col -= 1;
                }
            }

            // Delete the characters from new_col to cursor_col
            if new_col < self.core.cursor_col {
                let line = &self.core.text_lines[self.core.cursor_line];
                let char_indices: Vec<_> = line.char_indices().collect();

                if !char_indices.is_empty() {
                    let start_byte = if new_col < char_indices.len() {
                        char_indices[new_col].0
                    } else {
                        line.len()
                    };

                    let end_byte = if self.core.cursor_col < char_indices.len() {
                        char_indices[self.core.cursor_col].0
                    } else {
                        line.len()
                    };

                    if start_byte < end_byte {
                        self.core.text_lines[self.core.cursor_line].drain(start_byte..end_byte);
                    }
                }

                self.core.cursor_col = new_col;
                self.update_result(self.core.cursor_line);
                self.has_unsaved_changes = true;
            }
        }
    }

    /// Insert a new line at the cursor position
    pub fn new_line(&mut self) {
        if self.core.cursor_line < self.core.text_lines.len() {
            let current_line = self.core.text_lines[self.core.cursor_line].clone();
            let char_count = current_line.chars().count();
            let safe_cursor_col = self.core.cursor_col.min(char_count);

            // Split the line at character boundary, not byte boundary
            let (left, right) = if safe_cursor_col == 0 {
                ("".to_string(), current_line)
            } else if safe_cursor_col >= char_count {
                (current_line, "".to_string())
            } else {
                let split_byte_idx = current_line
                    .char_indices()
                    .nth(safe_cursor_col)
                    .map(|(i, _)| i)
                    .unwrap_or(current_line.len());
                let left = current_line[..split_byte_idx].to_string();
                let right = current_line[split_byte_idx..].to_string();
                (left, right)
            };

            // Check emptiness before moving the strings
            let left_empty = left.trim().is_empty();
            let right_empty = right.trim().is_empty();

            self.core.text_lines[self.core.cursor_line] = left;
            self.core
                .text_lines
                .insert(self.core.cursor_line + 1, right);
            self.core.results.insert(self.core.cursor_line + 1, None);

            // Insert corresponding empty animation slot
            if self.core.cursor_line + 1 < self.result_animations.len() {
                self.result_animations
                    .insert(self.core.cursor_line + 1, None);
            } else {
                // Ensure animations vector is large enough
                while self.result_animations.len() <= self.core.cursor_line + 1 {
                    self.result_animations.push(None);
                }
            }

            // Also ensure copy flash animations vector is large enough
            if self.core.cursor_line + 1 < self.copy_flash_animations.len() {
                self.copy_flash_animations
                    .insert(self.core.cursor_line + 1, None);
                self.copy_flash_is_result
                    .insert(self.core.cursor_line + 1, false);
            } else {
                while self.copy_flash_animations.len() <= self.core.cursor_line + 1 {
                    self.copy_flash_animations.push(None);
                    self.copy_flash_is_result.push(false);
                }
            }

            // Handle line reference updates for insertion
            let insertion_point = self.core.cursor_line + 1; // 0-based index of newly inserted line

            if left_empty && !right_empty {
                // Content moved from cursor_line to insertion_point
                // Use combined update that handles both content move and position shifts
                self.update_line_references_for_line_split_with_content_move(
                    self.core.cursor_line,
                    insertion_point,
                );
            } else {
                // Standard insertion: just shift references
                self.update_line_references_for_standard_insertion(insertion_point);
            }

            self.core.cursor_line += 1;
            self.core.cursor_col = 0;

            // Make sure to evaluate all lines in the correct order
            // First evaluate the lines that were directly affected by the split
            self.update_result(self.core.cursor_line - 1); // Line 0
            self.update_result(self.core.cursor_line); // Line 1

            // Then re-evaluate any lines that had their references updated
            // This ensures line references can resolve correctly
            for i in (self.core.cursor_line + 1)..self.core.text_lines.len() {
                self.update_result(i);
            }
            self.has_unsaved_changes = true;
        }
    }

    /// Move cursor up one line
    pub fn move_cursor_up(&mut self) {
        if self.core.cursor_line > 0 {
            self.core.cursor_line -= 1;
            self.core.cursor_col = self
                .core
                .cursor_col
                .min(self.core.text_lines[self.core.cursor_line].len());
        }
    }

    /// Move cursor down one line
    pub fn move_cursor_down(&mut self) {
        if self.core.cursor_line + 1 < self.core.text_lines.len() {
            self.core.cursor_line += 1;
            self.core.cursor_col = self
                .core
                .cursor_col
                .min(self.core.text_lines[self.core.cursor_line].len());
        }
    }

    /// Move cursor left one character
    pub fn move_cursor_left(&mut self) {
        if self.core.cursor_col > 0 {
            self.core.cursor_col -= 1;
        }
    }

    /// Move cursor right one character
    pub fn move_cursor_right(&mut self) {
        if self.core.cursor_line < self.core.text_lines.len() {
            self.core.cursor_col =
                (self.core.cursor_col + 1).min(self.core.text_lines[self.core.cursor_line].len());
        }
    }

    /// Delete the entire current line (vim 'dd' command)
    pub fn delete_line(&mut self) {
        if self.core.text_lines.len() > 1 {
            // Update line references before deletion
            self.update_line_references_for_deletion(self.core.cursor_line);

            // Remove the line
            self.core.text_lines.remove(self.core.cursor_line);
            self.core.results.remove(self.core.cursor_line);

            // Remove animation states
            if self.core.cursor_line < self.result_animations.len() {
                self.result_animations.remove(self.core.cursor_line);
            }
            if self.core.cursor_line < self.copy_flash_animations.len() {
                self.copy_flash_animations.remove(self.core.cursor_line);
                self.copy_flash_is_result.remove(self.core.cursor_line);
            }

            // Adjust cursor position
            if self.core.cursor_line >= self.core.text_lines.len() && self.core.cursor_line > 0 {
                self.core.cursor_line -= 1;
            }
            self.core.cursor_col = 0;

            // Re-evaluate all lines after deletion
            for i in self.core.cursor_line..self.core.text_lines.len() {
                self.update_result(i);
            }

            self.has_unsaved_changes = true;
        } else if self.core.text_lines.len() == 1 {
            // If only one line, just clear it instead of deleting
            self.core.text_lines[0].clear();
            self.core.results[0] = None;
            self.core.cursor_col = 0;
            self.update_result(0);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete character at cursor position (vim 'x' command)
    pub fn delete_char_at_cursor(&mut self) {
        if self.core.cursor_line < self.core.text_lines.len() {
            let line = &self.core.text_lines[self.core.cursor_line];
            let char_count = line.chars().count();

            if self.core.cursor_col < char_count {
                let char_indices: Vec<_> = line.char_indices().collect();
                if self.core.cursor_col < char_indices.len() {
                    let byte_start = char_indices[self.core.cursor_col].0;
                    let byte_end = if self.core.cursor_col + 1 < char_indices.len() {
                        char_indices[self.core.cursor_col + 1].0
                    } else {
                        line.len()
                    };

                    self.core.text_lines[self.core.cursor_line].drain(byte_start..byte_end);

                    // Adjust cursor if at end of line after deletion
                    let new_char_count =
                        self.core.text_lines[self.core.cursor_line].chars().count();
                    if self.core.cursor_col >= new_char_count && self.core.cursor_col > 0 {
                        self.core.cursor_col = new_char_count;
                    }
                    self.update_result(self.core.cursor_line);
                    self.has_unsaved_changes = true;
                }
            }
        }
    }

    /// Move cursor forward by one word (vim 'w' command)
    /// A word is a sequence of alphanumeric characters or underscores
    pub fn move_word_forward(&mut self) {
        if self.core.cursor_line >= self.core.text_lines.len() {
            return;
        }

        let line = &self.core.text_lines[self.core.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let mut new_col = self.core.cursor_col;

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
        if new_col >= chars.len() && self.core.cursor_line + 1 < self.core.text_lines.len() {
            self.core.cursor_line += 1;
            self.core.cursor_col = 0;
        } else {
            self.core.cursor_col = new_col;
        }
    }

    /// Move cursor backward by one word (vim 'b' command)
    pub fn move_word_backward(&mut self) {
        if self.core.cursor_line >= self.core.text_lines.len() {
            return;
        }

        let line = &self.core.text_lines[self.core.cursor_line];
        let chars: Vec<char> = line.chars().collect();

        if self.core.cursor_col == 0 {
            // If at start of line, move to end of previous line
            if self.core.cursor_line > 0 {
                self.core.cursor_line -= 1;
                self.core.cursor_col = self.core.text_lines[self.core.cursor_line].chars().count();
            }
            return;
        }

        let mut new_col = self.core.cursor_col;
        new_col = new_col.saturating_sub(1);

        // Skip non-word characters backwards
        while new_col > 0 && !(chars[new_col].is_alphanumeric() || chars[new_col] == '_') {
            new_col -= 1;
        }

        // Skip word characters backwards to find start of word
        while new_col > 0 && (chars[new_col - 1].is_alphanumeric() || chars[new_col - 1] == '_') {
            new_col -= 1;
        }

        self.core.cursor_col = new_col;
    }

    /// Move cursor forward by one WORD (vim 'W' command)
    /// A WORD is a sequence of non-whitespace characters
    pub fn move_word_forward_big(&mut self) {
        if self.core.cursor_line >= self.core.text_lines.len() {
            return;
        }

        let line = &self.core.text_lines[self.core.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let mut new_col = self.core.cursor_col;

        // Skip current WORD if we're in one
        while new_col < chars.len() && !chars[new_col].is_whitespace() {
            new_col += 1;
        }

        // Skip whitespace
        while new_col < chars.len() && chars[new_col].is_whitespace() {
            new_col += 1;
        }

        // If we've reached the end of the line, move to the next line
        if new_col >= chars.len() && self.core.cursor_line + 1 < self.core.text_lines.len() {
            self.core.cursor_line += 1;
            self.core.cursor_col = 0;
        } else {
            self.core.cursor_col = new_col;
        }
    }

    /// Move cursor backward by one WORD (vim 'B' command)
    pub fn move_word_backward_big(&mut self) {
        if self.core.cursor_line >= self.core.text_lines.len() {
            return;
        }

        let line = &self.core.text_lines[self.core.cursor_line];
        let chars: Vec<char> = line.chars().collect();

        if self.core.cursor_col == 0 {
            // If at start of line, move to end of previous line
            if self.core.cursor_line > 0 {
                self.core.cursor_line -= 1;
                self.core.cursor_col = self.core.text_lines[self.core.cursor_line].chars().count();
            }
            return;
        }

        let mut new_col = self.core.cursor_col;
        new_col = new_col.saturating_sub(1);

        // Skip whitespace backwards
        while new_col > 0 && chars[new_col].is_whitespace() {
            new_col -= 1;
        }

        // Skip non-whitespace backwards to find start of WORD
        while new_col > 0 && !chars[new_col - 1].is_whitespace() {
            new_col -= 1;
        }

        self.core.cursor_col = new_col;
    }

    /// Delete from cursor to the end of the current word (vim 'dw' command)
    pub fn delete_word_forward(&mut self) {
        if self.core.cursor_line >= self.core.text_lines.len() {
            return;
        }

        let line = &self.core.text_lines[self.core.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let start_col = self.core.cursor_col;
        let mut end_col = self.core.cursor_col;

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
            let line = &self.core.text_lines[self.core.cursor_line];
            let char_indices: Vec<_> = line.char_indices().collect();

            if !char_indices.is_empty() {
                let start_byte = if start_col < char_indices.len() {
                    char_indices[start_col].0
                } else {
                    line.len()
                };

                let end_byte = if end_col < char_indices.len() {
                    char_indices[end_col].0
                } else {
                    line.len()
                };

                if start_byte < end_byte {
                    self.core.text_lines[self.core.cursor_line].drain(start_byte..end_byte);
                }
            }

            self.update_result(self.core.cursor_line);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete from cursor to the beginning of the previous word (vim 'db' command)
    pub fn delete_word_backward(&mut self) {
        if self.core.cursor_line >= self.core.text_lines.len() || self.core.cursor_col == 0 {
            return;
        }

        let line = &self.core.text_lines[self.core.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let end_col = self.core.cursor_col;
        let mut start_col = self.core.cursor_col;

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
            let line = &self.core.text_lines[self.core.cursor_line];
            let char_indices: Vec<_> = line.char_indices().collect();

            if !char_indices.is_empty() {
                let start_byte = if start_col < char_indices.len() {
                    char_indices[start_col].0
                } else {
                    line.len()
                };

                let end_byte = if end_col < char_indices.len() {
                    char_indices[end_col].0
                } else {
                    line.len()
                };

                if start_byte < end_byte {
                    self.core.text_lines[self.core.cursor_line].drain(start_byte..end_byte);
                }
            }

            self.core.cursor_col = start_col;
            self.update_result(self.core.cursor_line);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete from cursor to the end of the current WORD (vim 'dW' command)
    pub fn delete_word_forward_big(&mut self) {
        if self.core.cursor_line >= self.core.text_lines.len() {
            return;
        }

        let line = &self.core.text_lines[self.core.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let start_col = self.core.cursor_col;
        let mut end_col = self.core.cursor_col;

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
            let line = &self.core.text_lines[self.core.cursor_line];
            let char_indices: Vec<_> = line.char_indices().collect();

            if !char_indices.is_empty() {
                let start_byte = if start_col < char_indices.len() {
                    char_indices[start_col].0
                } else {
                    line.len()
                };

                let end_byte = if end_col < char_indices.len() {
                    char_indices[end_col].0
                } else {
                    line.len()
                };

                if start_byte < end_byte {
                    self.core.text_lines[self.core.cursor_line].drain(start_byte..end_byte);
                }
            }

            self.update_result(self.core.cursor_line);
            self.has_unsaved_changes = true;
        }
    }

    /// Delete from cursor to the beginning of the previous WORD (vim 'dB' command)
    pub fn delete_word_backward_big(&mut self) {
        if self.core.cursor_line >= self.core.text_lines.len() || self.core.cursor_col == 0 {
            return;
        }

        let line = &self.core.text_lines[self.core.cursor_line];
        let chars: Vec<char> = line.chars().collect();
        let end_col = self.core.cursor_col;
        let mut start_col = self.core.cursor_col;

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
            let line = &self.core.text_lines[self.core.cursor_line];
            let char_indices: Vec<_> = line.char_indices().collect();

            if !char_indices.is_empty() {
                let start_byte = if start_col < char_indices.len() {
                    char_indices[start_col].0
                } else {
                    line.len()
                };

                let end_byte = if end_col < char_indices.len() {
                    char_indices[end_col].0
                } else {
                    line.len()
                };

                if start_byte < end_byte {
                    self.core.text_lines[self.core.cursor_line].drain(start_byte..end_byte);
                }
            }

            self.core.cursor_col = start_col;
            self.update_result(self.core.cursor_line);
            self.has_unsaved_changes = true;
        }
    }

    /// Update the calculation result for a given line
    pub fn update_result(&mut self, line_index: usize) {
        self.core.update_result(line_index);

        // Check if we need to start animation for the updated result
        if line_index < self.core.results.len() && self.core.results[line_index].is_some() {
            self.start_result_animation(line_index);
        }
    }

    /// Update line references in all lines when a line is deleted
    /// All references > deleted_line need to be decremented by 1
    /// References to the deleted line become invalid
    fn update_line_references_for_deletion(&mut self, deleted_line: usize) {
        for i in 0..self.core.text_lines.len() {
            let updated_text =
                update_line_references_in_text(&self.core.text_lines[i], deleted_line, -1);
            if updated_text != self.core.text_lines[i] {
                self.core.text_lines[i] = updated_text;
                // Re-evaluate this line since its content changed
                self.update_result(i);
            }
        }
    }

    /// Update line references in all lines when a new line is inserted
    /// All references >= insertion_point need to be incremented by 1
    fn update_line_references_for_standard_insertion(&mut self, insertion_point: usize) {
        for i in 0..self.core.text_lines.len() {
            let updated_text =
                update_line_references_in_text(&self.core.text_lines[i], insertion_point, 1);
            if updated_text != self.core.text_lines[i] {
                self.core.text_lines[i] = updated_text;
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

        for i in 0..self.core.text_lines.len() {
            let references = extract_line_references(&self.core.text_lines[i]);
            let mut updated_text = self.core.text_lines[i].clone();

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

            if updated_text != self.core.text_lines[i] {
                self.core.text_lines[i] = updated_text;
                // Re-evaluate this line since its content changed
                self.update_result(i);
            }
        }
    }

    /// Recalculate all lines in the notebook
    pub fn recalculate_all(&mut self) {
        // Clear variables to ensure fresh calculation
        self.core.variables.clear();

        // Recalculate each line in order
        for i in 0..self.core.text_lines.len() {
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
        if line_index < self.core.results.len() && self.core.results[line_index].is_some() {
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
            let content = self.core.text_lines.join("\n");
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
        let content = self.core.text_lines.join("\n");
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
        // Copy to clipboard using arboard (only available on non-WASM platforms)
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut clipboard = arboard::Clipboard::new()
                .map_err(|e| format!("Failed to access clipboard: {}", e))?;
            clipboard
                .set_text(text)
                .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
        }

        #[cfg(target_arch = "wasm32")]
        {
            // On WASM, we can't use arboard but we still want to show the animation
            // Web clipboard access would need to be implemented using web-sys if needed
            let _ = text; // Suppress unused variable warning
        }

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
        for (i, line) in app.core.text_lines.iter().enumerate() {
            println!("  Line {}: '{}'", i + 1, line);
        }

        // The real issue: we're inserting at position 1, so lines at position 1 and after
        // should get their references incremented. "line1 + 1" was originally at position 1,
        // is now at position 2, and "line1" in it should become "line2" because ALL lines
        // shift down after insertion point.
        assert!(
            app.core.text_lines[2].contains("line2"),
            "Expected 'line1' to be updated to 'line2' but got: '{}'",
            app.core.text_lines[2]
        );
    }

    #[test]
    fn test_line_splitting_at_beginning() {
        let mut app = App::default();
        app.core.text_lines = vec!["5".to_string(), "line1 + 1".to_string()];
        app.core.results = vec![None, None];
        app.core.cursor_line = 0;
        app.core.cursor_col = 0; // Position cursor at beginning of "5"

        app.new_line();

        // When hitting enter at beginning:
        // Line 1: "" (empty), Line 2: "5" (content moved down), Line 3: "line2 + 1"
        assert_eq!(app.core.text_lines[0], "");
        assert_eq!(app.core.text_lines[1], "5");
        assert!(
            app.core.text_lines[2].contains("line2"),
            "Expected 'line1' to be updated to 'line2' but got: '{}'",
            app.core.text_lines[2]
        );
    }

    #[test]
    fn test_user_reported_scenario() {
        let mut app = App::default();
        app.core.text_lines = vec!["5".to_string(), "line1 + 1".to_string()];
        app.core.results = vec![None, None];

        // Simulate exactly what the user described:
        // "i have the following notebook: 5, line1 + 1"
        app.update_result(0); // This should make line 1 result in "5"
        app.update_result(1); // This should make line 2 result in "6" (5 + 1)

        // Verify initial results work correctly
        assert_eq!(app.core.results[0], Some("5".to_string()));
        assert_eq!(app.core.results[1], Some("6".to_string()));

        // Now simulate "hit enter when the cursor is over the 5"
        app.core.cursor_line = 0;
        app.core.cursor_col = 0; // Position cursor at the beginning of "5"
        app.new_line();

        // Verify the results:
        // Line 1: "" (empty)
        // Line 2: "5"
        // Line 3: "line2 + 1" (updated reference)
        assert_eq!(app.core.text_lines[0], "");
        assert_eq!(app.core.text_lines[1], "5");
        assert!(
            app.core.text_lines[2].contains("line2"),
            "Expected line reference to be updated to 'line2', got: '{}'",
            app.core.text_lines[2]
        );

        // The key test: verify that the expression evaluates correctly automatically
        // line2 should now refer to "5" on line 2, so "line2 + 1" should be 6
        assert_eq!(
            app.core.results[2],
            Some("6".to_string()),
            "Expected 'line2 + 1' to evaluate to 6 automatically, got: {:?}",
            app.core.results[2]
        );
    }

    #[test]
    fn test_deletion_with_line_references() {
        let mut app = App::default();
        app.core.text_lines = vec!["".to_string(), "5".to_string(), "line2 + 1".to_string()];
        app.core.results = vec![None, None, None];

        // Set up: after line splitting we have ["", "5", "line2 + 1"]
        app.update_result(0);
        app.update_result(1);
        app.update_result(2);

        // Verify initial state works
        assert_eq!(app.core.results[1], Some("5".to_string()));
        assert_eq!(app.core.results[2], Some("6".to_string()));

        // Now delete the empty first line by positioning cursor at beginning of line 2 and hitting backspace
        app.core.cursor_line = 1;
        app.core.cursor_col = 0;
        app.delete_char(); // This should merge lines and update references

        // Expected result: ["5", "line1 + 1"] (reference should go back to line1)
        assert_eq!(app.core.text_lines.len(), 2);
        assert_eq!(app.core.text_lines[0], "5");
        assert!(
            app.core.text_lines[1].contains("line1"),
            "Expected 'line2' to be updated back to 'line1', got: '{}'",
            app.core.text_lines[1]
        );

        // The critical test: expression should still evaluate correctly
        assert_eq!(
            app.core.results[1],
            Some("6".to_string()),
            "Expected 'line1 + 1' to evaluate to 6 after deletion, got: {:?}",
            app.core.results[1]
        );
    }

    #[test]
    fn test_full_user_workflow_add_then_remove_lines() {
        let mut app = App::default();
        app.core.text_lines = vec!["5".to_string(), "line1 + 1".to_string()];
        app.core.results = vec![None, None];

        // Start with the user's original notebook
        app.update_result(0);
        app.update_result(1);

        // Verify initial state: 5, line1 + 1 = 6
        assert_eq!(app.core.results[0], Some("5".to_string()));
        assert_eq!(app.core.results[1], Some("6".to_string()));

        // Step 1: Hit enter at beginning of "5" (add lines)
        app.core.cursor_line = 0;
        app.core.cursor_col = 0;
        app.new_line();

        // Should have: ["", "5", "line2 + 1"] with line2 + 1 = 6
        assert_eq!(app.core.text_lines.len(), 3);
        assert_eq!(app.core.text_lines[0], "");
        assert_eq!(app.core.text_lines[1], "5");
        assert!(app.core.text_lines[2].contains("line2"));
        assert_eq!(app.core.results[2], Some("6".to_string()));

        // Step 2: Remove the empty line (delete lines)
        app.core.cursor_line = 1;
        app.core.cursor_col = 0;
        app.delete_char();

        // Should be back to: ["5", "line1 + 1"] with line1 + 1 = 6
        assert_eq!(app.core.text_lines.len(), 2);
        assert_eq!(app.core.text_lines[0], "5");
        assert!(app.core.text_lines[1].contains("line1"));
        assert_eq!(
            app.core.results[1],
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
        let mut app = App::default();
        app.core.text_lines = vec!["test".to_string(), "test2".to_string()];
        app.copy_flash_animations = vec![None, None];

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
        let mut app = App::default();
        app.core.text_lines = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        app.core.results = vec![None, None, None];
        app.result_animations = vec![None, None, None];
        app.copy_flash_animations = vec![None, None, None];
        app.copy_flash_is_result = vec![false, false, false];
        app.core.cursor_line = 1;

        // Delete middle line
        app.delete_line();
        assert_eq!(app.core.text_lines, vec!["first", "third"]);
        assert_eq!(app.core.cursor_line, 1);
        assert_eq!(app.core.cursor_col, 0);

        // Delete last line
        app.delete_line();
        assert_eq!(app.core.text_lines, vec!["first"]);
        assert_eq!(app.core.cursor_line, 0);

        // Try to delete only line - should just clear it
        app.delete_line();
        assert_eq!(app.core.text_lines, vec![""]);
        assert_eq!(app.core.cursor_line, 0);
    }

    #[test]
    fn test_delete_char_at_cursor() {
        let mut app = App::default();
        app.core.text_lines = vec!["hello world".to_string()];
        app.core.results = vec![None];
        app.core.cursor_line = 0;
        app.core.cursor_col = 6; // at 'w'

        app.delete_char_at_cursor();
        assert_eq!(app.core.text_lines[0], "hello orld");
        assert_eq!(app.core.cursor_col, 6);

        // Delete at end of line should do nothing
        app.core.cursor_col = app.core.text_lines[0].len();
        app.delete_char_at_cursor();
        assert_eq!(app.core.text_lines[0], "hello orld");
    }

    #[test]
    fn test_word_movement_forward() {
        let mut app = App::default();
        app.core.text_lines = vec!["hello world test_var 123".to_string()];
        app.core.cursor_line = 0;
        app.core.cursor_col = 0;

        // Move from start to 'world'
        app.move_word_forward();
        assert_eq!(app.core.cursor_col, 6);

        // Move to 'test_var'
        app.move_word_forward();
        assert_eq!(app.core.cursor_col, 12);

        // Move to '123'
        app.move_word_forward();
        assert_eq!(app.core.cursor_col, 21);

        // Try to move past end - should go to end of line
        app.move_word_forward();
        assert_eq!(app.core.cursor_col, 24); // Should be at end of line
    }

    #[test]
    fn test_word_movement_backward() {
        let mut app = App::default();
        app.core.text_lines = vec!["hello world test_var".to_string()];
        app.core.cursor_line = 0;
        app.core.cursor_col = 20; // at end

        // Move to start of 'test_var'
        app.move_word_backward();
        assert_eq!(app.core.cursor_col, 12);

        // Move to start of 'world'
        app.move_word_backward();
        assert_eq!(app.core.cursor_col, 6);

        // Move to start of 'hello'
        app.move_word_backward();
        assert_eq!(app.core.cursor_col, 0);
    }

    #[test]
    fn test_word_movement_big() {
        let mut app = App::default();
        app.core.text_lines = vec!["hello-world test::func()".to_string()];
        app.core.cursor_line = 0;
        app.core.cursor_col = 0;

        // 'hello-world' is one WORD
        app.move_word_forward_big();
        assert_eq!(app.core.cursor_col, 12);

        // Move backward
        app.core.cursor_col = 24;
        app.move_word_backward_big();
        assert_eq!(app.core.cursor_col, 12);

        app.move_word_backward_big();
        assert_eq!(app.core.cursor_col, 0);
    }

    #[test]
    fn test_delete_word_forward() {
        let mut app = App::default();
        app.core.text_lines = vec!["hello world test".to_string()];
        app.core.results = vec![None];
        app.core.cursor_line = 0;
        app.core.cursor_col = 0;

        // Delete 'hello ' (word + trailing space)
        app.delete_word_forward();
        assert_eq!(app.core.text_lines[0], "world test");
        assert_eq!(app.core.cursor_col, 0);

        // Delete from middle of word
        app.core.cursor_col = 2; // in 'world'
        app.delete_word_forward();
        assert_eq!(app.core.text_lines[0], "wotest");
    }

    #[test]
    fn test_delete_word_backward() {
        let mut app = App::default();
        app.core.text_lines = vec!["hello world test".to_string()];
        app.core.results = vec![None];
        app.core.cursor_line = 0;
        app.core.cursor_col = 11; // at space after 'world'

        // Delete 'world'
        app.delete_word_backward();
        assert_eq!(app.core.text_lines[0], "hello  test");
        assert_eq!(app.core.cursor_col, 6);
    }

    #[test]
    fn test_delete_word_forward_big() {
        let mut app = App::default();
        app.core.text_lines = vec!["hello-world test::func()".to_string()];
        app.core.results = vec![None];
        app.core.cursor_line = 0;
        app.core.cursor_col = 0;

        // Delete 'hello-world ' (WORD + trailing space)
        app.delete_word_forward_big();
        assert_eq!(app.core.text_lines[0], "test::func()");
        assert_eq!(app.core.cursor_col, 0);
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

    #[test]
    fn test_command_mode_initialization() {
        let mut app = App::default();

        // Initially not in command mode
        assert_eq!(app.mode, Mode::Insert);
        assert_eq!(app.command_line, "");
        assert_eq!(app.command_cursor, 0);

        // Enter command mode
        app.mode = Mode::Command;
        app.command_line = ":".to_string();
        app.command_cursor = 1;

        assert_eq!(app.mode, Mode::Command);
        assert_eq!(app.command_line, ":");
        assert_eq!(app.command_cursor, 1);
    }

    #[test]
    fn test_command_line_editing() {
        let mut app = App {
            mode: Mode::Command,
            command_line: ":w".to_string(),
            command_cursor: 2,
            ..Default::default()
        };

        // Test cursor movement
        assert_eq!(app.command_cursor, 2);

        // Test that command line can be modified
        app.command_line = ":wq".to_string();
        app.command_cursor = 3;

        assert_eq!(app.command_line, ":wq");
        assert_eq!(app.command_cursor, 3);
    }

    #[test]
    fn test_command_mode_toggle() {
        let mut app = App {
            mode: Mode::Normal,
            ..Default::default()
        };

        // Start in normal mode
        assert_eq!(app.mode, Mode::Normal);

        // Switch to command mode
        app.mode = Mode::Command;
        assert_eq!(app.mode, Mode::Command);

        // Switch back to normal mode
        app.mode = Mode::Normal;
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_delete_char_utf8() {
        let mut app = App::default();

        // Test deleting multi-byte characters
        app.insert_char('€');
        assert_eq!(app.core.cursor_col, 1);
        app.insert_char(' ');
        assert_eq!(app.core.cursor_col, 2);
        app.insert_char('5');
        assert_eq!(app.core.cursor_col, 3);
        assert_eq!(app.core.text_lines[0], "€ 5");

        // Delete '5'
        app.delete_char();
        assert_eq!(app.core.text_lines[0], "€ ");
        assert_eq!(app.core.cursor_col, 2);

        // Delete space
        app.delete_char();
        assert_eq!(app.core.text_lines[0], "€");
        assert_eq!(app.core.cursor_col, 1);

        // Delete euro sign
        app.delete_char();
        assert_eq!(app.core.text_lines[0], "");
        assert_eq!(app.core.cursor_col, 0);

        // Test with emoji
        app.insert_char('🚀');
        app.insert_char('€');
        app.insert_char('¥');
        assert_eq!(app.core.text_lines[0], "🚀€¥");
        assert_eq!(app.core.cursor_col, 3);

        // Delete yen
        app.delete_char();
        assert_eq!(app.core.text_lines[0], "🚀€");
        assert_eq!(app.core.cursor_col, 2);

        // Delete euro
        app.delete_char();
        assert_eq!(app.core.text_lines[0], "🚀");
        assert_eq!(app.core.cursor_col, 1);
    }
}
