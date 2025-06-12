//! Application state and core logic

use crate::{Mode, expression::evaluate_with_variables};
use std::collections::HashMap;

/// Main application state for the mathematical notepad
pub struct App {
    pub text_lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub results: Vec<Option<String>>,
    pub variables: HashMap<String, String>, // variable_name -> value_string
    pub mode: Mode,
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
            mode: Mode::Insert, // Start in insert mode
        }
    }
}

impl App {
    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        if self.cursor_line < self.text_lines.len() {
            self.text_lines[self.cursor_line].insert(self.cursor_col, c);
            self.cursor_col += 1;
            self.update_result(self.cursor_line);
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
            } else if self.cursor_line > 0 {
                // Cursor is at beginning of line - merge with previous line
                let current_line = self.text_lines.remove(self.cursor_line);
                self.results.remove(self.cursor_line);
                self.cursor_line -= 1;
                self.cursor_col = self.text_lines[self.cursor_line].len();
                self.text_lines[self.cursor_line].push_str(&current_line);
                self.update_result(self.cursor_line);
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
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.update_result(self.cursor_line - 1);
            self.update_result(self.cursor_line);
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

            // If this is a variable assignment, store it and re-evaluate dependent lines
            if let Some((var_name, var_value)) = variable_assignment {
                // Check if this variable assignment actually changed the value
                let variable_changed = self.variables.get(&var_name) != Some(&var_value);
                
                self.variables.insert(var_name.clone(), var_value);
                
                // If the variable changed, re-evaluate all lines that might use this variable
                if variable_changed {
                    self.results[line_index] = result;
                    self.re_evaluate_dependent_lines(&var_name, line_index);
                    return;
                }
            }

            self.results[line_index] = result;
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
                        let nested_changed = self.variables.get(&nested_var_name) != Some(&nested_var_value);
                        self.variables.insert(nested_var_name.clone(), nested_var_value);
                        
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
}
