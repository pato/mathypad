//! GUI module - egui-based interface
//!
//! This module provides the graphical user interface using egui framework.
//! Only available when the 'gui' feature is enabled.

#[cfg(feature = "gui")]
mod app;

#[cfg(feature = "gui")]
pub use app::MathypadGuiApp;

#[cfg(feature = "gui")]
pub mod wasm;

// Re-export egui types for convenience when gui feature is enabled
#[cfg(feature = "gui")]
pub use eframe;
#[cfg(feature = "gui")]
pub use egui;
