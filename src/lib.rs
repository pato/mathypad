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