//! Unit system for mathypad
//!
//! This module handles all unit-related functionality including:
//! - Unit definitions and conversions
//! - Unit value representation
//! - Unit parsing

mod parser;
mod types;
mod value;

#[cfg(test)]
mod tests;

pub use parser::parse_unit;
pub use types::{Unit, UnitConversionError, UnitType};
pub use value::UnitValue;
