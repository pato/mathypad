//! Token definitions for mathematical expressions

use crate::units::Unit;

/// Tokens for mathematical expressions with unit support
#[derive(Debug, Clone)]
pub enum Token {
    Number(f64),
    NumberWithUnit(f64, Unit),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    LeftParen,
    RightParen,
    To,                   // for conversions like "to KiB"
    In,                   // for conversions like "in KiB"
    Of,                   // for percentage operations like "10% of 50"
    LineReference(usize), // for referencing other lines like "line1", "line2"
    Variable(String),     // for variable references like "servers", "ram"
    Assign,               // for assignment operator "="
}
