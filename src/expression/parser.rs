//! Expression parsing and tokenization functions

use super::chumsky_parser::parse_expression_chumsky;
use super::tokens::Token;
use crate::units::parse_unit;

/// Parse a line reference string like "line1", "line2" etc.
pub fn parse_line_reference(text: &str) -> Option<usize> {
    let text_lower = text.to_lowercase();
    if let Some(number_part) = text_lower.strip_prefix("line") {
        if let Ok(line_num) = number_part.parse::<usize>() {
            if line_num > 0 {
                return Some(line_num - 1); // Convert to 0-based indexing
            }
        }
    }
    None
}

/// Extract all line references from a text string
/// Returns a vector of (start_pos, end_pos, line_number) tuples for each "lineN" found
pub fn extract_line_references(text: &str) -> Vec<(usize, usize, usize)> {
    let mut references = Vec::new();
    let text_lower = text.to_lowercase();
    let mut search_start = 0;

    while let Some(line_pos) = text_lower[search_start..].find("line") {
        let absolute_pos = search_start + line_pos;
        
        // Check if "line" is at a word boundary (not preceded by alphanumeric)
        let is_word_start = absolute_pos == 0 || 
            !text_lower.chars().nth(absolute_pos - 1).unwrap_or(' ').is_ascii_alphanumeric();
        
        if is_word_start {
            let remaining = &text_lower[absolute_pos + 4..]; // Skip "line"
            
            // Find the number part in the lowercase version
            let mut num_end = 0;
            for ch in remaining.chars() {
                if ch.is_ascii_digit() {
                    num_end += 1;
                } else {
                    break;
                }
            }
            
            if num_end > 0 {
                // Check if "lineN" is at a word boundary (not followed by alphanumeric)
                let is_word_end = absolute_pos + 4 + num_end >= text_lower.len() ||
                    !text_lower.chars().nth(absolute_pos + 4 + num_end).unwrap_or(' ').is_ascii_alphanumeric();
                
                if is_word_end {
                    // Parse the number from the original text (not lowercase) to preserve digits
                    let original_remaining = &text[absolute_pos + 4..];
                    if let Ok(line_num) = original_remaining[..num_end].parse::<usize>() {
                        if line_num > 0 {
                            let start_pos = absolute_pos;
                            let end_pos = absolute_pos + 4 + num_end; // "line" + digits
                            references.push((start_pos, end_pos, line_num - 1)); // Convert to 0-based
                        }
                    }
                }
            }
        }
        
        search_start = absolute_pos + 4; // Move past "line"
    }
    
    references
}

/// Update line references in text by applying an offset to references >= threshold
/// If offset is positive: increment references >= threshold
/// If offset is negative: decrement references > threshold, mark deleted line refs as invalid
pub fn update_line_references_in_text(text: &str, threshold: usize, offset: i32) -> String {
    let references = extract_line_references(text);
    
    if references.is_empty() {
        return text.to_string();
    }
    
    let mut result = text.to_string();
    
    // Process references in reverse order to maintain correct string positions
    for (start_pos, end_pos, line_num) in references.into_iter().rev() {
        if offset > 0 {
            // Line insertion: increment references >= insertion point
            if line_num >= threshold {
                let new_ref = format!("line{}", line_num + 1 + 1); // +1 for the offset, +1 for 1-based
                result.replace_range(start_pos..end_pos, &new_ref);
            }
        } else {
            // Line deletion: handle references to deleted line and after
            if line_num == threshold {
                // Reference to the deleted line - mark as invalid
                result.replace_range(start_pos..end_pos, "INVALID_REF");
            } else if line_num > threshold {
                // Reference after deleted line - decrement by 1
                let new_ref = format!("line{}", line_num); // line_num already represents the new 1-based line number after shift
                result.replace_range(start_pos..end_pos, &new_ref);
            }
            // References before deleted line stay unchanged
        }
    }
    
    result
}

/// Tokenize any text into tokens - always succeeds, may include non-mathematical tokens
pub fn tokenize_with_units(expr: &str) -> Option<Vec<Token>> {
    // Use the chumsky parser - now accepts any input
    match parse_expression_chumsky(expr) {
        Ok(tokens) if tokens.is_empty() => None, // Only fail on truly empty input
        Ok(tokens) => Some(tokens),              // Accept any non-empty token sequence
        Err(_) => None,                          // Only fail on parse errors
    }
}

/// Check if a sequence of tokens forms a valid mathematical expression
pub fn is_valid_mathematical_expression(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    // Count different token types
    let mut has_number_or_value = false;
    let mut consecutive_operators = 0;
    let mut consecutive_values = 0;

    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Number(_)
            | Token::NumberWithUnit(_, _)
            | Token::LineReference(_)
            | Token::Variable(_) => {
                has_number_or_value = true;
                consecutive_values += 1;
                consecutive_operators = 0;

                // More than 1 consecutive value without operators is invalid (except for assignments and conversions)
                if consecutive_values > 1 {
                    // Allow if this is part of an assignment (Variable = Expression)
                    if i >= 2
                        && matches!(tokens[i - 1], Token::Assign)
                        && matches!(tokens[i - 2], Token::Variable(_))
                    {
                        consecutive_values = 1; // Reset count after assignment
                    } else {
                        return false;
                    }
                }
            }
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide => {
                consecutive_operators += 1;
                consecutive_values = 0;

                // More than 1 consecutive operator is invalid (except minus for negation)
                if consecutive_operators > 1 && !matches!(token, Token::Minus) {
                    return false;
                }
            }
            Token::LeftParen | Token::RightParen => {
                consecutive_operators = 0;
                consecutive_values = 0;
            }
            Token::To | Token::In | Token::Of => {
                // These are OK for conversions and percentage operations
                consecutive_operators = 0;
                consecutive_values = 0;
            }
            Token::Assign => {
                // Assignment is only valid after a variable
                if i == 0 || !matches!(tokens[i - 1], Token::Variable(_)) {
                    return false;
                }
                consecutive_operators = 0;
                consecutive_values = 0;
            }
        }
    }

    // Must have at least one number/value to be a mathematical expression
    has_number_or_value
}

/// Check if a string represents a valid mathematical expression
pub fn is_valid_math_expression(expr: &str) -> bool {
    let expr = expr.trim();
    if expr.is_empty() {
        return false;
    }

    let mut has_number = false;
    // let mut has_operator = false;
    let mut paren_count = 0;
    let mut prev_was_operator = true; // Start as true to allow leading numbers

    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            ' ' => {
                i += 1;
                continue;
            }
            '0'..='9' => {
                has_number = true;
                prev_was_operator = false;
                // Skip through the whole number (including commas and decimals)
                while i < chars.len()
                    && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',')
                {
                    i += 1;
                }

                // Skip whitespace
                while i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }

                // Check for unit
                if i < chars.len() && chars[i].is_ascii_alphabetic() {
                    let unit_start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '/') {
                        i += 1;
                    }

                    let unit_str: String = chars[unit_start..i].iter().collect();
                    if parse_unit(&unit_str).is_none()
                        && unit_str.to_lowercase() != "to"
                        && unit_str.to_lowercase() != "in"
                        && parse_line_reference(&unit_str).is_none()
                    {
                        // Not a recognized unit or line reference, rewind
                        i = unit_start;
                    }
                }
                continue;
            }
            '.' => {
                if prev_was_operator {
                    return false; // Can't start with decimal point
                }
                i += 1;
            }
            '+' | '-' | '*' | '/' => {
                if prev_was_operator && ch != '-' {
                    return false; // Two operators in a row (except minus for negation)
                }
                prev_was_operator = true;
                i += 1;
            }
            '(' => {
                paren_count += 1;
                prev_was_operator = true;
                i += 1;
            }
            ')' => {
                paren_count -= 1;
                if paren_count < 0 {
                    return false;
                }
                prev_was_operator = false;
                i += 1;
            }
            _ => {
                if ch.is_ascii_alphabetic() {
                    let unit_start = i;
                    // For potential line references, also include digits
                    while i < chars.len()
                        && (chars[i].is_ascii_alphabetic()
                            || chars[i].is_ascii_digit()
                            || chars[i] == '/')
                    {
                        i += 1;
                    }

                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" || word.to_lowercase() == "in" {
                        prev_was_operator = true;
                    } else if parse_line_reference(&word).is_some() {
                        // Valid line reference, acts like a number
                        has_number = true;
                        prev_was_operator = false;
                    } else if parse_unit(&word).is_some() {
                        // Valid unit, continue
                        prev_was_operator = false;
                    } else {
                        // Unknown word - treat as the end of the expression
                        // Check if what we have so far is valid
                        break;
                    }
                } else {
                    // If we encounter any other character, check if what we have so far is valid
                    break;
                }
            }
        }
    }

    // Must have balanced parentheses, at least one number, and if it has operators, must end properly
    paren_count == 0 && has_number && !prev_was_operator
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_line_reference() {
        // Test valid line references
        assert_eq!(parse_line_reference("line1"), Some(0));
        assert_eq!(parse_line_reference("line2"), Some(1));
        assert_eq!(parse_line_reference("line10"), Some(9));
        assert_eq!(parse_line_reference("line999"), Some(998));

        // Test case insensitive
        assert_eq!(parse_line_reference("LINE1"), Some(0));
        assert_eq!(parse_line_reference("Line2"), Some(1));
        assert_eq!(parse_line_reference("LiNe3"), Some(2));

        // Test invalid line references
        assert_eq!(parse_line_reference("line0"), None); // 0 is invalid
        assert_eq!(parse_line_reference("line"), None); // No number
        assert_eq!(parse_line_reference("line-1"), None); // Negative
        assert_eq!(parse_line_reference("linea"), None); // Not a number
        assert_eq!(parse_line_reference("notline1"), None); // Wrong prefix
        assert_eq!(parse_line_reference(""), None); // Empty
        assert_eq!(parse_line_reference("1line"), None); // Wrong order
    }

    #[test]
    fn test_tokenize_with_units_basic() {
        // Test basic numbers
        let tokens = tokenize_with_units("42").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Number(42.0)));

        // Test numbers with units
        let tokens = tokenize_with_units("5 GiB").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(5.0, _)));

        // Test simple arithmetic
        let tokens = tokenize_with_units("2 + 3").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Number(2.0)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Number(3.0)));
    }

    #[test]
    fn test_tokenize_with_units_invalid() {
        // Test that tokenizer now accepts all text (refactored approach)
        let result = tokenize_with_units("invalid text");
        assert!(result.is_some()); // Tokenizer now accepts everything

        // Still fails on clearly malformed expressions
        assert!(tokenize_with_units("1 + 2)").is_none());
        assert!(tokenize_with_units("1 invalidunit").is_some()); // Now parses as [Number, Variable]

        // Note: empty string actually returns Ok([]) in chumsky parser
        // but tokenize_with_units returns None for empty results
        let result = tokenize_with_units("");
        assert!(result.is_none());
    }

    #[test]
    fn test_is_valid_math_expression() {
        // Test valid expressions
        assert!(is_valid_math_expression("42"));
        assert!(is_valid_math_expression("2 + 3"));
        assert!(is_valid_math_expression("(1 + 2) * 3"));
        assert!(is_valid_math_expression("5 GiB + 10 MiB"));
        assert!(is_valid_math_expression("line1 * 2"));
        assert!(is_valid_math_expression("1 TiB to GiB"));
        assert!(is_valid_math_expression("24 MiB * 32 in KiB"));

        // Test invalid expressions
        assert!(!is_valid_math_expression(""));
        assert!(!is_valid_math_expression("invalid text"));
        assert!(!is_valid_math_expression("1 +"));
        assert!(!is_valid_math_expression("+ 2"));
        assert!(!is_valid_math_expression("1 + + 2"));
        assert!(!is_valid_math_expression("(1 + 2"));
        assert!(!is_valid_math_expression("1 + 2)"));

        // Note: "1 invalidunit" is actually considered valid by is_valid_math_expression
        // because it sees "1" as a valid number and stops there
        // The actual parsing will fail later, but this function is for syntax validation

        // Test edge cases
        assert!(is_valid_math_expression("0"));
        assert!(is_valid_math_expression("-5")); // Negative numbers
        assert!(is_valid_math_expression("1.5"));
        assert!(is_valid_math_expression("1,000"));
        assert!(is_valid_math_expression("1,000,000.50"));
    }

    #[test]
    fn test_is_valid_math_expression_units() {
        // Test various unit formats
        assert!(is_valid_math_expression("5GiB")); // No space
        assert!(is_valid_math_expression("5 GiB")); // With space
        assert!(is_valid_math_expression("10.5 MB/s")); // Compound unit
        assert!(is_valid_math_expression("100 QPS")); // QPS unit
        assert!(is_valid_math_expression("1 hour")); // Time unit
        assert!(is_valid_math_expression("8 bit")); // Bit unit

        // Test conversions
        assert!(is_valid_math_expression("1 GiB to MiB"));
        assert!(is_valid_math_expression("24 MiB * 32 in KiB"));
        assert!(is_valid_math_expression("100 QPS to req/min"));

        // Test case variations
        assert!(is_valid_math_expression("1 gib TO mib"));
        assert!(is_valid_math_expression("1 GIB to MIB"));
    }

    #[test]
    fn test_is_valid_math_expression_operators() {
        // Test all operators
        assert!(is_valid_math_expression("1 + 2"));
        assert!(is_valid_math_expression("5 - 3"));
        assert!(is_valid_math_expression("4 * 6"));
        assert!(is_valid_math_expression("8 / 2"));

        // Test operator combinations
        assert!(is_valid_math_expression("1 + 2 - 3"));
        assert!(is_valid_math_expression("2 * 3 + 4"));
        assert!(is_valid_math_expression("10 / 2 - 1"));

        // Test with parentheses
        assert!(is_valid_math_expression("(1 + 2) * 3"));
        assert!(is_valid_math_expression("1 + (2 * 3)"));
        assert!(is_valid_math_expression("((1 + 2) * 3) - 4"));

        // Test invalid operator usage
        assert!(!is_valid_math_expression("1 + * 2"));
        assert!(!is_valid_math_expression("* 1 + 2"));
        assert!(!is_valid_math_expression("1 + 2 *"));
    }

    #[test]
    fn test_is_valid_math_expression_line_references() {
        // Test line references
        assert!(is_valid_math_expression("line1"));
        assert!(is_valid_math_expression("line10"));
        assert!(is_valid_math_expression("line1 + line2"));
        assert!(is_valid_math_expression("line1 * 2"));
        assert!(is_valid_math_expression("(line1 + line2) / 2"));

        // Test line references with units
        assert!(is_valid_math_expression("line1 + 5 GiB"));
        assert!(is_valid_math_expression("line1 to MiB"));
        assert!(is_valid_math_expression("line1 + line2 in KiB"));

        // Test case insensitive line references
        assert!(is_valid_math_expression("LINE1"));
        assert!(is_valid_math_expression("Line2"));
        assert!(is_valid_math_expression("LiNe3 + LiNe4"));
    }

    #[test]
    fn test_whitespace_handling() {
        // Test various whitespace scenarios
        assert!(is_valid_math_expression("  1 + 2  "));
        assert!(is_valid_math_expression("1   +   2"));
        assert!(is_valid_math_expression("1\t+\t2"));
        assert!(is_valid_math_expression("1+2")); // No spaces

        // Test whitespace in units
        assert!(is_valid_math_expression("5   GiB"));
        assert!(is_valid_math_expression("5GiB"));

        // Test whitespace around keywords
        assert!(is_valid_math_expression("1 GiB  to  MiB"));
        assert!(is_valid_math_expression("1 GiB to MiB"));
    }

    #[test]
    fn test_extract_line_references() {
        // Test basic line reference extraction
        assert_eq!(extract_line_references("line1 + 5"), vec![(0, 5, 0)]);
        assert_eq!(extract_line_references("10 + line2"), vec![(5, 10, 1)]);
        assert_eq!(extract_line_references("line1 + line2 * line3"), 
                   vec![(0, 5, 0), (8, 13, 1), (16, 21, 2)]);

        // Test case insensitivity
        assert_eq!(extract_line_references("Line1 + Line2"), vec![(0, 5, 0), (8, 13, 1)]);
        assert_eq!(extract_line_references("LINE1 + line2"), vec![(0, 5, 0), (8, 13, 1)]);

        // Test with complex expressions
        assert_eq!(extract_line_references("(line1 + line2) * 2 to GiB"), 
                   vec![(1, 6, 0), (9, 14, 1)]);

        // Test multi-digit line numbers
        assert_eq!(extract_line_references("line10 + line123"), vec![(0, 6, 9), (9, 16, 122)]);

        // Test no line references
        assert_eq!(extract_line_references("5 + 3 * 2"), vec![]);
        assert_eq!(extract_line_references("hello world"), vec![]);

        // Test edge cases
        assert_eq!(extract_line_references("line0"), vec![]); // line0 is invalid
        assert_eq!(extract_line_references("line"), vec![]); // no number
        assert_eq!(extract_line_references("myline1"), vec![]); // not starting with "line"

        // Test with text around
        assert_eq!(extract_line_references("result: line1 + 2"), vec![(8, 13, 0)]);
    }

    #[test]
    fn test_update_line_references_insertion() {
        // Test insertion at the beginning (all references should be incremented)
        assert_eq!(update_line_references_in_text("line1 + line2", 0, 1), "line2 + line3");
        assert_eq!(update_line_references_in_text("line3 + 5", 0, 1), "line4 + 5");

        // Test insertion in the middle (only references >= insertion point are updated)
        assert_eq!(update_line_references_in_text("line1 + line3", 2, 1), "line1 + line4");
        assert_eq!(update_line_references_in_text("line1 + line2 + line3", 2, 1), "line1 + line2 + line4");

        // Test insertion at the end (no references should be updated)
        assert_eq!(update_line_references_in_text("line1 + line2", 5, 1), "line1 + line2");

        // Test no line references
        assert_eq!(update_line_references_in_text("5 + 3", 1, 1), "5 + 3");

        // Test complex expressions
        assert_eq!(update_line_references_in_text("(line2 + line4) * 2 to GiB", 3, 1), 
                   "(line2 + line5) * 2 to GiB");
    }

    #[test]
    fn test_update_line_references_deletion() {
        // Test deletion at the beginning 
        assert_eq!(update_line_references_in_text("line1 + line2 + line3", 0, -1), 
                   "INVALID_REF + line1 + line2");

        // Test deletion in the middle
        assert_eq!(update_line_references_in_text("line1 + line2 + line3", 1, -1), 
                   "line1 + INVALID_REF + line2");

        // Test deletion at the end
        assert_eq!(update_line_references_in_text("line1 + line2 + line3", 2, -1), 
                   "line1 + line2 + INVALID_REF");

        // Test references before deleted line stay unchanged
        assert_eq!(update_line_references_in_text("line1 + line5", 3, -1), 
                   "line1 + line4");

        // Test no line references
        assert_eq!(update_line_references_in_text("5 + 3", 1, -1), "5 + 3");

        // Test complex scenarios
        assert_eq!(update_line_references_in_text("line1 + line3 + line5", 2, -1), 
                   "line1 + INVALID_REF + line4");

        // Test the user's reported scenario: deleting empty first line
        assert_eq!(update_line_references_in_text("line2 + 1", 0, -1), 
                   "line1 + 1");
    }

    #[test]
    fn test_update_line_references_edge_cases() {
        // Test multiple references to the same line
        assert_eq!(update_line_references_in_text("line2 + line2 * line2", 1, -1), 
                   "INVALID_REF + INVALID_REF * INVALID_REF");

        // Test large line numbers
        assert_eq!(update_line_references_in_text("line100 + line200", 150, 1), 
                   "line100 + line201");

        // Test case preservation in complex text
        assert_eq!(update_line_references_in_text("Result: Line1 + LINE2", 1, -1), 
                   "Result: Line1 + INVALID_REF");

        // Test with mixed content
        assert_eq!(update_line_references_in_text("Memory usage: line3 * 1024 bytes", 2, 1), 
                   "Memory usage: line4 * 1024 bytes");
    }
}
