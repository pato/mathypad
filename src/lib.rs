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
pub mod expression;
pub mod mode;
pub mod ui;
pub mod units;

#[cfg(test)]
mod integration_tests;

// Re-export commonly used types for convenience
pub use app::App;
pub use cli::run_one_shot_mode;
pub use expression::evaluate_expression_with_context;
pub use mode::Mode;
pub use ui::run_interactive_mode;
pub use units::{Unit, UnitType, UnitValue};

// Constants used throughout the application
pub const TICK_RATE_MS: u64 = 250;
pub const MAX_INTEGER_FOR_FORMATTING: f64 = 1e15;
pub const FLOAT_EPSILON: f64 = f64::EPSILON;

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    // Helper function to evaluate expressions for testing
    pub fn evaluate_test_expression(input: &str) -> Option<String> {
        evaluate_expression_with_context(input, &[], 0)
    }

    // Helper function to get unit conversion results for testing
    pub fn evaluate_with_unit_info(input: &str) -> Option<UnitValue> {
        // Use the new token-based approach
        use crate::expression::{evaluate_tokens_stream_with_context, tokenize_with_units};

        if let Some(tokens) = tokenize_with_units(input) {
            evaluate_tokens_stream_with_context(&tokens, &[], 0)
        } else {
            None
        }
    }
}
