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
pub use mode::Mode;
pub use ui::{run_interactive_mode, run_interactive_mode_with_file};
pub use mathypad_core::{Unit, UnitType, UnitValue};

// Constants used throughout the application
pub const TICK_RATE_MS: u64 = 16; // ~60 FPS for smooth animations

// Re-export constants from core
pub use mathypad_core::{MAX_INTEGER_FOR_FORMATTING, FLOAT_EPSILON};

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    // Helper function to evaluate expressions for testing
    pub fn evaluate_test_expression(input: &str) -> Option<String> {
        evaluate_expression_with_context(input, &[], 0)
    }

    // Helper function to get unit conversion results for testing
    pub fn evaluate_with_unit_info(input: &str) -> Option<UnitValue> {
        // Use the core evaluation and try to parse as UnitValue
        if let Some(result_str) = evaluate_expression_with_context(input, &[], 0) {
            mathypad_core::expression::evaluator::parse_result_string(&result_str)
        } else {
            None
        }
    }
}
