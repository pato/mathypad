//! User interface components and rendering
//!
//! This module handles all TUI rendering and event handling functionality.

mod events;
mod render;

#[cfg(test)]
mod tests;

pub use events::{handle_command_mode, run_interactive_mode, run_interactive_mode_with_file};
pub use render::{parse_colors, render_results_panel, render_text_area, ui};
