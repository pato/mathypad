//! Expression parsing and evaluation system
//! 
//! This module handles mathematical expression parsing, tokenization, and evaluation
//! with unit-aware arithmetic operations.

mod tokens;
mod parser;
mod evaluator;

pub use tokens::Token;
pub use parser::{
    tokenize_with_units, 
    find_math_expression, 
    is_valid_math_expression,
    parse_line_reference,
};
pub use evaluator::{
    evaluate_expression_with_context,
    parse_and_evaluate_with_context,
    evaluate_tokens_with_units_and_context,
    resolve_line_reference,
    parse_result_string,
};