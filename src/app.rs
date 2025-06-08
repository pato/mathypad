//! Application state and core logic

use crate::{Mode, evaluate_expression_with_context};

/// Main application state for the mathematical notepad
pub struct App {
    pub text_lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub results: Vec<Option<String>>,
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
        if self.cursor_line < self.text_lines.len() && self.cursor_col > 0 {
            self.text_lines[self.cursor_line].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
            self.update_result(self.cursor_line);
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
            self.results[line_index] =
                evaluate_expression_with_context(line, &self.results, line_index);
        } else {
            // This should never happen in normal operation, but let's be defensive
            eprintln!(
                "Warning: Attempted to update result for invalid line index {}",
                line_index
            );
        }
    }
}
