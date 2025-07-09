//! # Mathypad
//!
//! A smart calculator that understands units and makes complex calculations simple.
//!
//! This library provides the core functionality for mathypad, including:
//! - Unit-aware mathematical expression evaluation
//! - Comprehensive unit conversion system
//! - TUI application framework
//! - CLI interface utilities

pub mod cli;
pub mod version;

// TUI-related modules (not available on WASM)
#[cfg(not(target_arch = "wasm32"))]
pub mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod mode;
#[cfg(not(target_arch = "wasm32"))]
pub mod ui;

// GUI module (only available with 'gui' feature)
#[cfg(feature = "gui")]
pub mod gui;

// Re-export core functionality
pub use mathypad_core::{expression, units};

#[cfg(test)]
mod integration_tests;

// Re-export commonly used types for convenience
pub use cli::run_one_shot_mode;
pub use mathypad_core::expression::evaluator::evaluate_expression_with_context;
pub use mathypad_core::{Unit, UnitType, UnitValue};

// TUI-related re-exports (not available on WASM)
#[cfg(not(target_arch = "wasm32"))]
pub use app::App;
#[cfg(not(target_arch = "wasm32"))]
pub use mode::Mode;
#[cfg(not(target_arch = "wasm32"))]
pub use ui::{run_interactive_mode, run_interactive_mode_with_file};

// TUI constants (not needed on WASM)
#[cfg(not(target_arch = "wasm32"))]
pub const TICK_RATE_MS: u64 = 16; // ~60 FPS for smooth animations

// Re-export constants from core
pub use mathypad_core::{FLOAT_EPSILON, MAX_INTEGER_FOR_FORMATTING};

// Re-export test helpers from core to avoid duplication
#[cfg(test)]
pub use mathypad_core::test_helpers;

// WASM entry point (only for wasm32 target with gui feature)
#[cfg(all(feature = "gui", target_arch = "wasm32"))]
pub use gui::wasm::main;
