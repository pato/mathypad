//! Token definitions for mathematical expressions

use crate::units::{Unit, UnitValue};

/// Tokens for mathematical expressions with unit support
#[derive(Debug, Clone)]
pub enum Token {
    Number(f64),
    NumberWithUnit(f64, Unit), // Legacy - will be removed after refactoring
    Unit(Unit),                // New - standalone unit token
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
    To,                   // for conversions like "to KiB"
    In,                   // for conversions like "in KiB"
    Of,                   // for percentage operations like "10% of 50"
    LineReference(usize), // for referencing other lines like "line1", "line2"
    Variable(String),     // for variable references like "servers", "ram"
    Assign,               // for assignment operator "="
}

/// Semantic tokens represent meaningful evaluation units after parsing
/// These are what the evaluator actually works with
#[derive(Debug, Clone)]
pub enum SemanticToken {
    /// A value (number with optional unit)
    Value(UnitValue),

    /// Mathematical operators
    Add,
    Subtract,
    Multiply,
    Divide,

    /// Grouping
    LeftParen,
    RightParen,

    /// Conversion operations
    ConvertTo(Unit), // "to MiB" -> target unit for conversion
    ConvertIn(Unit), // "in KiB" -> target unit for conversion

    /// Percentage operations
    PercentOf, // "% of" -> percentage operation

    /// References
    LineReference(usize), // Reference to another line
    Variable(String), // Variable reference

    /// Assignment
    Assign, // Variable assignment
}
