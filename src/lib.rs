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
    use crate::expression::{find_math_expression, parse_and_evaluate_with_context};

    // Helper function to evaluate expressions for testing
    pub fn evaluate_test_expression(input: &str) -> Option<String> {
        evaluate_expression_with_context(input, &[], 0)
    }

    // Helper function to get unit conversion results for testing
    pub fn evaluate_with_unit_info(input: &str) -> Option<UnitValue> {
        let expressions = find_math_expression(input);
        for expr in expressions {
            if let Some(result) = parse_and_evaluate_with_context(&expr, &[], 0) {
                return Some(result);
            }
        }
        None
    }
}