//! Command-line interface functions

use crate::evaluate_expression_with_context;
use crate::expression::parse_line_reference;
use crate::units::parse_unit;
use std::error::Error;

/// Run one-shot evaluation mode (non-interactive)
pub fn run_one_shot_mode(expression: &str) -> Result<(), Box<dyn Error>> {
    // Print the expression with syntax highlighting
    print_formatted_expression(expression);

    // Evaluate the expression (no context for one-shot mode)
    if let Some(result) = evaluate_expression_with_context(expression, &[], 0) {
        println!(" = {}", result);
    } else {
        println!(" = (invalid expression)");
    }

    Ok(())
}

/// Print a mathematical expression with ANSI color formatting
pub fn print_formatted_expression(text: &str) {
    // Use ANSI escape codes to print numbers in light blue and units in green
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();

    while current_pos < chars.len() {
        if chars[current_pos].is_ascii_alphabetic() {
            // Handle potential units, keywords, and line references first
            let start_pos = current_pos;

            while current_pos < chars.len()
                && (chars[current_pos].is_ascii_alphabetic()
                    || chars[current_pos].is_ascii_digit()
                    || chars[current_pos] == '/')
            {
                current_pos += 1;
            }

            let word_text: String = chars[start_pos..current_pos].iter().collect();

            // Check if it's a valid unit, keyword, or line reference
            if parse_line_reference(&word_text).is_some() {
                // Print line reference in magenta (ANSI color code 95)
                print!("\x1b[95m{}\x1b[0m", word_text);
            } else if word_text.to_lowercase() == "to" || word_text.to_lowercase() == "in" {
                // Print keywords in yellow (ANSI color code 93)
                print!("\x1b[93m{}\x1b[0m", word_text);
            } else if parse_unit(&word_text).is_some() {
                // Print units in green (ANSI color code 92)
                print!("\x1b[92m{}\x1b[0m", word_text);
            } else {
                print!("{}", word_text);
            }
        } else if chars[current_pos].is_ascii_digit() || chars[current_pos] == '.' {
            // Handle numbers
            let start_pos = current_pos;
            let mut has_digit = false;
            let mut has_dot = false;

            while current_pos < chars.len() {
                let ch = chars[current_pos];
                if ch.is_ascii_digit() {
                    has_digit = true;
                    current_pos += 1;
                } else if ch == '.' && !has_dot {
                    has_dot = true;
                    current_pos += 1;
                } else if ch == ',' {
                    current_pos += 1;
                } else {
                    break;
                }
            }

            if has_digit {
                let number_text: String = chars[start_pos..current_pos].iter().collect();
                // Print number in light blue (ANSI color code 94)
                print!("\x1b[94m{}\x1b[0m", number_text);
            } else {
                print!("{}", chars[start_pos]);
                current_pos = start_pos + 1;
            }
        } else if "+-*/()".contains(chars[current_pos]) {
            // Print operators in cyan (ANSI color code 96)
            print!("\x1b[96m{}\x1b[0m", chars[current_pos]);
            current_pos += 1;
        } else {
            print!("{}", chars[current_pos]);
            current_pos += 1;
        }
    }
}
