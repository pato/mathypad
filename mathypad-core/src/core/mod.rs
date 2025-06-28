//! Core abstractions for shared application state and logic

pub mod file_ops;
pub mod highlighting;
pub mod state;

pub use file_ops::{FileOperations, deserialize_lines, serialize_lines};
pub use highlighting::{HighlightType, HighlightedSpan, highlight_expression};
pub use state::MathypadCore;
