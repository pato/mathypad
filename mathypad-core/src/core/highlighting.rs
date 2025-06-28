//! UI-agnostic syntax highlighting for mathematical expressions

use crate::expression::parser::parse_line_reference;
use crate::units::parse_unit;
use std::collections::HashMap;

/// A highlighted text span with semantic type information
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightedSpan {
    pub text: String,
    pub highlight_type: HighlightType,
}

/// Types of syntax highlighting
#[derive(Debug, Clone, PartialEq)]
pub enum HighlightType {
    /// Numeric literals (e.g., "123", "3.14", "1,000")
    Number,
    /// Unit names (e.g., "kg", "miles", "seconds")
    Unit,
    /// Line references (e.g., "line1", "line2")
    LineReference,
    /// Keywords (e.g., "to", "in", "of")
    Keyword,
    /// Mathematical operators (e.g., "+", "-", "*", "/", "^", "=")
    Operator,
    /// Variable names that are defined
    Variable,
    /// Function names (e.g., "sqrt")
    Function,
    /// Normal text (no special highlighting)
    Normal,
}

impl HighlightType {
    /// Get the standard RGB color values for this highlight type
    /// Returns (red, green, blue) as u8 values
    pub fn rgb_color(&self) -> (u8, u8, u8) {
        match self {
            HighlightType::Number => (173, 216, 230),     // Light blue
            HighlightType::Unit => (144, 238, 144),       // Light green
            HighlightType::LineReference => (221, 160, 221), // Plum/magenta
            HighlightType::Keyword => (255, 255, 0),      // Yellow
            HighlightType::Operator => (0, 255, 255),     // Cyan
            HighlightType::Variable => (224, 255, 255),   // Light cyan
            HighlightType::Function => (0, 255, 255),     // Cyan
            HighlightType::Normal => (200, 200, 200),     // Light gray
        }
    }
}

/// Parse text and return highlighted spans for syntax highlighting
pub fn highlight_expression(
    text: &str,
    variables: &HashMap<String, String>,
) -> Vec<HighlightedSpan> {
    let mut spans = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();

    while current_pos < chars.len() {
        if chars[current_pos].is_ascii_alphabetic() {
            // Handle potential units, keywords, and line references first
            let start_pos = current_pos;

            while current_pos < chars.len()
                && (chars[current_pos].is_ascii_alphabetic() || chars[current_pos].is_ascii_digit())
            {
                current_pos += 1;
            }

            let word_text: String = chars[start_pos..current_pos].iter().collect();

            // Check if it's a valid unit, keyword, line reference, function, or variable
            let highlight_type = if parse_line_reference(&word_text).is_some() {
                HighlightType::LineReference
            } else if word_text.to_lowercase() == "to"
                || word_text.to_lowercase() == "in"
                || word_text.to_lowercase() == "of"
            {
                HighlightType::Keyword
            } else if word_text.to_lowercase() == "sqrt" {
                HighlightType::Function
            } else if parse_unit(&word_text).is_some() {
                HighlightType::Unit
            } else if variables.contains_key(&word_text) {
                HighlightType::Variable
            } else {
                HighlightType::Normal
            };

            spans.push(HighlightedSpan {
                text: word_text,
                highlight_type,
            });
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

            let number_text: String = chars[start_pos..current_pos].iter().collect();

            if has_digit {
                spans.push(HighlightedSpan {
                    text: number_text,
                    highlight_type: HighlightType::Number,
                });
            } else {
                spans.push(HighlightedSpan {
                    text: number_text,
                    highlight_type: HighlightType::Normal,
                });
                current_pos = start_pos + 1;
            }
        } else if chars[current_pos] == '%' {
            // Handle percentage symbol as a unit
            spans.push(HighlightedSpan {
                text: "%".to_string(),
                highlight_type: HighlightType::Unit,
            });
            current_pos += 1;
        } else if "$€£¥₹₩".contains(chars[current_pos]) {
            // Handle currency symbols as units
            spans.push(HighlightedSpan {
                text: chars[current_pos].to_string(),
                highlight_type: HighlightType::Unit,
            });
            current_pos += 1;
        } else if "+-*/()=^".contains(chars[current_pos]) {
            // Handle operators (including assignment and exponentiation)
            spans.push(HighlightedSpan {
                text: chars[current_pos].to_string(),
                highlight_type: HighlightType::Operator,
            });
            current_pos += 1;
        } else {
            // Handle other characters
            spans.push(HighlightedSpan {
                text: chars[current_pos].to_string(),
                highlight_type: HighlightType::Normal,
            });
            current_pos += 1;
        }
    }

    spans
}

/// Convenience function to highlight a single line with cursor position
/// Returns the spans and the character index where the cursor should be highlighted
pub fn highlight_expression_with_cursor(
    text: &str,
    cursor_col: usize,
    variables: &HashMap<String, String>,
) -> (Vec<HighlightedSpan>, usize) {
    let spans = highlight_expression(text, variables);
    // The cursor highlighting would be handled by the UI layer
    // This function exists for API compatibility
    (spans, cursor_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_highlighting() {
        let variables = HashMap::new();
        let spans = highlight_expression("123.45", &variables);

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "123.45");
        assert_eq!(spans[0].highlight_type, HighlightType::Number);
    }

    #[test]
    fn test_operator_highlighting() {
        let variables = HashMap::new();
        let spans = highlight_expression("5 + 3", &variables);

        assert_eq!(spans.len(), 5); // "5", " ", "+", " ", "3"
        assert_eq!(spans[0].highlight_type, HighlightType::Number);
        assert_eq!(spans[1].highlight_type, HighlightType::Normal); // space
        assert_eq!(spans[2].highlight_type, HighlightType::Operator);
        assert_eq!(spans[3].highlight_type, HighlightType::Normal); // space
        assert_eq!(spans[4].highlight_type, HighlightType::Number);
    }

    #[test]
    fn test_unit_highlighting() {
        let variables = HashMap::new();
        let spans = highlight_expression("100 kg", &variables);

        assert_eq!(spans.len(), 3); // "100", " ", "kg"
        assert_eq!(spans[0].highlight_type, HighlightType::Number);
        assert_eq!(spans[1].highlight_type, HighlightType::Normal); // space
        assert_eq!(spans[2].highlight_type, HighlightType::Unit);
    }

    #[test]
    fn test_line_reference_highlighting() {
        let variables = HashMap::new();
        let spans = highlight_expression("line1 + 5", &variables);

        assert!(
            spans
                .iter()
                .any(|s| s.highlight_type == HighlightType::LineReference)
        );
        assert!(spans.iter().any(|s| s.text == "line1"));
    }

    #[test]
    fn test_variable_highlighting() {
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), "42".to_string());

        let spans = highlight_expression("x * 2", &variables);

        assert!(
            spans
                .iter()
                .any(|s| s.highlight_type == HighlightType::Variable)
        );
        assert!(spans.iter().any(|s| s.text == "x"));
    }

    #[test]
    fn test_keyword_highlighting() {
        let variables = HashMap::new();
        let spans = highlight_expression("100 kg to lb", &variables);

        assert!(
            spans
                .iter()
                .any(|s| s.highlight_type == HighlightType::Keyword)
        );
        assert!(spans.iter().any(|s| s.text == "to"));
    }

    #[test]
    fn test_function_highlighting() {
        let variables = HashMap::new();
        let spans = highlight_expression("sqrt(16)", &variables);

        assert!(
            spans
                .iter()
                .any(|s| s.highlight_type == HighlightType::Function)
        );
        assert!(spans.iter().any(|s| s.text == "sqrt"));
    }
}
