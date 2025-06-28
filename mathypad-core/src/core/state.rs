//! Core application state shared between TUI and web UI

use crate::expression::{evaluate_with_variables, update_line_references_in_text};
use std::collections::HashMap;

/// Core application state containing text, results, and variables
/// This is UI-agnostic and can be used by both TUI and web implementations
#[derive(Debug, Clone)]
pub struct MathypadCore {
    /// The text content of each line
    pub text_lines: Vec<String>,
    /// Current cursor line position (0-indexed)
    pub cursor_line: usize,
    /// Current cursor column position (0-indexed, in characters)
    pub cursor_col: usize,
    /// Evaluation results for each line (None means no result or error)
    pub results: Vec<Option<String>>,
    /// Variable storage (variable_name -> value_string)
    pub variables: HashMap<String, String>,
}

impl Default for MathypadCore {
    fn default() -> Self {
        Self {
            text_lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            results: vec![None],
            variables: HashMap::new(),
        }
    }
}

impl MathypadCore {
    /// Create a new empty MathypadCore instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a MathypadCore from a list of text lines
    pub fn from_lines(lines: Vec<String>) -> Self {
        let line_count = lines.len().max(1);
        let mut core = Self {
            text_lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
            cursor_line: 0,
            cursor_col: 0,
            results: vec![None; line_count],
            variables: HashMap::new(),
        };
        core.recalculate_all();
        core
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        if self.cursor_line < self.text_lines.len() {
            // Convert cursor position from character index to byte index for insertion
            let line = &self.text_lines[self.cursor_line];
            let char_count = line.chars().count();

            // Ensure cursor position is within bounds
            let safe_cursor_col = self.cursor_col.min(char_count);

            // Find the byte position for character insertion
            let byte_index = if safe_cursor_col == 0 {
                0
            } else if safe_cursor_col >= char_count {
                line.len()
            } else {
                line.char_indices()
                    .nth(safe_cursor_col)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len())
            };

            self.text_lines[self.cursor_line].insert(byte_index, c);
            self.cursor_col += 1;
            self.update_result(self.cursor_line);
        }
    }

    /// Delete the character before the cursor
    pub fn delete_char(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            if self.cursor_col > 0 {
                // Delete character within the current line
                let line = &mut self.text_lines[self.cursor_line];

                // Find the byte index of the character to delete
                let char_indices: Vec<_> = line.char_indices().collect();
                if self.cursor_col > 0 && self.cursor_col <= char_indices.len() {
                    let char_to_delete_idx = self.cursor_col - 1;
                    let start_byte = char_indices[char_to_delete_idx].0;
                    let end_byte = if char_to_delete_idx + 1 < char_indices.len() {
                        char_indices[char_to_delete_idx + 1].0
                    } else {
                        line.len()
                    };
                    line.drain(start_byte..end_byte);
                }

                self.cursor_col -= 1;
                self.update_result(self.cursor_line);
            } else if self.cursor_line > 0 {
                // Delete newline - merge with previous line
                let current_line = self.text_lines.remove(self.cursor_line);
                self.cursor_line -= 1;
                self.cursor_col = self.text_lines[self.cursor_line].chars().count();
                self.text_lines[self.cursor_line].push_str(&current_line);

                // Remove the corresponding result
                self.results.remove(self.cursor_line + 1);

                // Update all affected line references
                self.update_line_references_for_deletion(self.cursor_line + 1);
                self.recalculate_all();
            }
        }
    }

    /// Insert a new line at the current cursor position
    pub fn new_line(&mut self) {
        if self.cursor_line < self.text_lines.len() {
            let line = &self.text_lines[self.cursor_line];
            let char_count = line.chars().count();
            let safe_cursor_col = self.cursor_col.min(char_count);

            // Find the byte position for splitting
            let byte_index = if safe_cursor_col == 0 {
                0
            } else if safe_cursor_col >= char_count {
                line.len()
            } else {
                line.char_indices()
                    .nth(safe_cursor_col)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len())
            };

            // Split the line at the cursor position
            let remaining = self.text_lines[self.cursor_line].split_off(byte_index);

            // Insert the new line
            self.cursor_line += 1;
            self.text_lines.insert(self.cursor_line, remaining);
            self.cursor_col = 0;

            // Insert corresponding result placeholder
            self.results.insert(self.cursor_line, None);

            // Update line references for insertion
            self.update_line_references_for_insertion(self.cursor_line);
            self.recalculate_all();
        }
    }

    /// Update the result for a specific line
    pub fn update_result(&mut self, line_index: usize) {
        if line_index < self.text_lines.len() {
            let line_text = &self.text_lines[line_index];

            // Evaluate the expression with current variables and other line results
            let (result, variable_assignment) =
                evaluate_with_variables(line_text, &self.variables, &self.results, line_index);

            // Handle variable assignment if present
            if let Some((var_name, var_value)) = variable_assignment {
                self.variables.insert(var_name, var_value);
            }

            // Ensure results vector is large enough
            while self.results.len() <= line_index {
                self.results.push(None);
            }

            // Store the result
            self.results[line_index] = result;
        }
    }

    /// Recalculate all results and variables
    pub fn recalculate_all(&mut self) {
        // Clear variables and recalculate from scratch
        self.variables.clear();

        // Ensure results vector matches text lines
        self.results.resize(self.text_lines.len(), None);

        // Evaluate each line in order
        for i in 0..self.text_lines.len() {
            self.update_result(i);
        }
    }

    /// Update line references after a line insertion
    fn update_line_references_for_insertion(&mut self, inserted_at: usize) {
        for (i, line) in self.text_lines.iter_mut().enumerate() {
            if i != inserted_at {
                *line = update_line_references_in_text(line, inserted_at, 1);
            }
        }
    }

    /// Update line references after a line deletion
    fn update_line_references_for_deletion(&mut self, deleted_at: usize) {
        for line in self.text_lines.iter_mut() {
            *line = update_line_references_in_text(line, deleted_at, -1);
        }
    }

    /// Move cursor to a specific position
    pub fn move_cursor_to(&mut self, line: usize, col: usize) {
        self.cursor_line = line.min(self.text_lines.len().saturating_sub(1));
        if self.cursor_line < self.text_lines.len() {
            let max_col = self.text_lines[self.cursor_line].chars().count();
            self.cursor_col = col.min(max_col);
        }
    }

    /// Get the current line content
    pub fn current_line(&self) -> &str {
        if self.cursor_line < self.text_lines.len() {
            &self.text_lines[self.cursor_line]
        } else {
            ""
        }
    }

    /// Get the result for the current line
    pub fn current_result(&self) -> Option<&str> {
        if self.cursor_line < self.results.len() {
            self.results[self.cursor_line].as_deref()
        } else {
            None
        }
    }

    /// Set text content from a string (splitting into lines)
    pub fn set_content(&mut self, content: &str) {
        self.text_lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.results = vec![None; self.text_lines.len()];
        self.variables.clear();
        self.recalculate_all();
    }

    /// Get content as a single string
    pub fn get_content(&self) -> String {
        self.text_lines.join("\n")
    }
}
