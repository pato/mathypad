//! Mathypad Core - Shared calculation and parsing logic
//! 
//! This crate contains the core mathematical evaluation, unit conversion, 
//! and expression parsing logic that is shared between the TUI and web UI versions of Mathypad.

pub mod expression;
pub mod units;
pub mod core;

// Constants used throughout the application
pub const MAX_INTEGER_FOR_FORMATTING: f64 = 1e15;
pub const FLOAT_EPSILON: f64 = f64::EPSILON;

// Re-export commonly used types for convenience
pub use expression::{
    evaluator::{evaluate_expression_with_context, evaluate_with_variables},
    parser::*,
};
pub use units::{Unit, UnitType, UnitValue, parse_unit};

/// Test helpers for expression evaluation - shared across implementations
#[cfg(test)]
pub mod test_helpers {
    use crate::expression::evaluator::evaluate_expression_with_context;
    use crate::units::UnitValue;

    pub fn evaluate_test_expression(expr: &str) -> Option<String> {
        evaluate_expression_with_context(expr, &[], 0)
    }

    pub fn evaluate_with_unit_info(expr: &str) -> (Option<String>, Option<UnitValue>) {
        if let Some(result_str) = evaluate_expression_with_context(expr, &[], 0) {
            // Try to parse the result as a UnitValue for unit information
            let unit_value = crate::expression::evaluator::parse_result_string(&result_str);
            (Some(result_str), unit_value)
        } else {
            (None, None)
        }
    }
}