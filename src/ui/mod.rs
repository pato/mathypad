//! User interface components and rendering
//!
//! This module handles all TUI rendering and event handling functionality.

mod events;
mod render;

pub use events::{run_interactive_mode, run_interactive_mode_with_file};
pub use render::{parse_colors, render_results_panel, render_text_area, ui};
