//! Expression parsing and evaluation system
//!
//! This module handles mathematical expression parsing, tokenization, and evaluation
//! with unit-aware arithmetic operations.

mod chumsky_parser;
mod evaluator;
mod parser;
mod tokens;

#[cfg(test)]
mod tests;

pub use chumsky_parser::{
    TokenWithSpan, parse_expression_chumsky, parse_expression_for_highlighting,
};
pub use evaluator::{
    evaluate_expression_with_context, evaluate_tokens_stream_with_context,
    evaluate_tokens_with_units_and_context, evaluate_with_variables,
    parse_and_evaluate_with_context, parse_result_string, resolve_line_reference,
};
pub use parser::{
    extract_line_references, is_valid_math_expression, is_valid_mathematical_expression,
    parse_line_reference, tokenize_with_units, update_line_references_in_text,
};
pub use tokens::Token;
