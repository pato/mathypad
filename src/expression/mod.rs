//! Expression parsing and evaluation system
//!
//! This module handles mathematical expression parsing, tokenization, and evaluation
//! with unit-aware arithmetic operations.

mod evaluator;
mod parser;
mod tokens;
mod chumsky_parser;

#[cfg(test)]
mod tests;

pub use evaluator::{
    evaluate_expression_with_context, evaluate_tokens_with_units_and_context,
    parse_and_evaluate_with_context, parse_result_string, resolve_line_reference,
    evaluate_with_variables, evaluate_tokens_stream_with_context,
};
pub use parser::{
    is_valid_math_expression, parse_line_reference, tokenize_with_units,
    is_valid_mathematical_expression,
};
pub use chumsky_parser::parse_expression_chumsky;
pub use tokens::Token;
