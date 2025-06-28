//! Core abstractions for shared application state and logic

pub mod state;
pub mod highlighting;
pub mod file_ops;

pub use state::MathypadCore;
pub use highlighting::{HighlightedSpan, HighlightType, highlight_expression};
pub use file_ops::{FileOperations, serialize_lines, deserialize_lines};