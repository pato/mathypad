//! User interface components and rendering
//! 
//! This module handles all TUI rendering and event handling functionality.

mod events;
mod render;

pub use events::run_interactive_mode;
pub use render::{ui, render_text_area, render_results_panel, parse_colors};