//! Vim-like editing modes for the application.

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Insert mode - normal text editing
    Insert,
    /// Normal mode - vim-like navigation
    Normal,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Insert
    }
}