//! # Mathypad
//!
//! A smart calculator that understands units and makes complex calculations simple.
//!
//! This library provides the core functionality for mathypad, including:
//! - Unit-aware mathematical expression evaluation
//! - Comprehensive unit conversion system
//! - TUI application framework
//! - CLI interface utilities

pub mod app;
pub mod cli;
pub mod mode;
pub mod ui;
pub mod version;

// Re-export core functionality
pub use mathypad_core::{expression, units};

#[cfg(test)]
mod integration_tests;

// Re-export commonly used types for convenience
pub use app::App;
pub use cli::run_one_shot_mode;
pub use mathypad_core::expression::evaluator::evaluate_expression_with_context;
pub use mathypad_core::{Unit, UnitType, UnitValue};
pub use mode::Mode;
pub use ui::{run_interactive_mode, run_interactive_mode_with_file};

// Constants used throughout the application
pub const TICK_RATE_MS: u64 = 16; // ~60 FPS for smooth animations

// Re-export constants from core
pub use mathypad_core::{FLOAT_EPSILON, MAX_INTEGER_FOR_FORMATTING};

// Re-export test helpers from core to avoid duplication
#[cfg(test)]
pub use mathypad_core::test_helpers;
