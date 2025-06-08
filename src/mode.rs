//! Vim-like editing modes for the application.

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Mode {
    /// Insert mode - normal text editing
    #[default]
    Insert,
    /// Normal mode - vim-like navigation
    Normal,
}
