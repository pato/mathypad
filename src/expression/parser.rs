//! Expression parsing and tokenization functions

use super::tokens::Token;
use super::chumsky_parser::parse_expression_chumsky;
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

/// Tokenize a mathematical expression with unit support
pub fn tokenize_with_units(expr: &str) -> Option<Vec<Token>> {
    // First try the chumsky parser
    if let Ok(tokens) = parse_expression_chumsky(expr) {
        return Some(tokens);
    }
    
    // Fall back to the original hand-written parser
    tokenize_with_units_fallback(expr)
}

/// Fallback tokenizer using the original hand-written parser
fn tokenize_with_units_fallback(expr: &str) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
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
                // Parse number (with potential commas)
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',')
                {
                    i += 1;
                }

                let number_str: String = chars[start..i].iter().collect();
                let cleaned_number = number_str.replace(",", "");
                let num = cleaned_number.parse::<f64>().ok()?;

                // Skip whitespace
                while i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }

                // Look for unit
                if i < chars.len() && chars[i].is_ascii_alphabetic() {
                    let unit_start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '/') {
                        i += 1;
                    }

                    let unit_str: String = chars[unit_start..i].iter().collect();
                    if let Some(unit) = parse_unit(&unit_str) {
                        tokens.push(Token::NumberWithUnit(num, unit));
                    } else {
                        tokens.push(Token::Number(num));
                        // Put back the unit characters - they might be part of something else
                        i = unit_start;
                    }
                } else {
                    tokens.push(Token::Number(num));
                }
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' => {
                tokens.push(Token::Multiply);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Divide);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LeftParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RightParen);
                i += 1;
            }
            't' | 'T' => {
                // Check for "to" keyword
                if i + 1 < chars.len() && chars[i + 1].to_lowercase().next() == Some('o') {
                    // Check if it's actually "to" and not part of a longer word like "tb"
                    if i + 2 >= chars.len() || !chars[i + 2].is_ascii_alphabetic() {
                        // Skip whitespace after "to"
                        i += 2;
                        while i < chars.len() && chars[i] == ' ' {
                            i += 1;
                        }
                        tokens.push(Token::To);
                    } else {
                        // Fall through to general alphabetic handling
                        let unit_start = i;
                        while i < chars.len()
                            && (chars[i].is_ascii_alphabetic()
                                || chars[i].is_ascii_digit()
                                || chars[i] == '/')
                        {
                            i += 1;
                        }

                        let word: String = chars[unit_start..i].iter().collect();
                        if word.to_lowercase() == "to" {
                            tokens.push(Token::To);
                        } else if word.to_lowercase() == "in" {
                            tokens.push(Token::In);
                        } else if let Some(line_num) = parse_line_reference(&word) {
                            tokens.push(Token::LineReference(line_num));
                        } else if let Some(unit) = parse_unit(&word) {
                            tokens.push(Token::NumberWithUnit(1.0, unit));
                        } else {
                            return None;
                        }
                    }
                } else {
                    // Fall through to general alphabetic handling
                    let unit_start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphabetic()
                            || chars[i].is_ascii_digit()
                            || chars[i] == '/')
                    {
                        i += 1;
                    }

                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" {
                        tokens.push(Token::To);
                    } else if word.to_lowercase() == "in" {
                        tokens.push(Token::In);
                    } else if let Some(line_num) = parse_line_reference(&word) {
                        tokens.push(Token::LineReference(line_num));
                    } else if let Some(unit) = parse_unit(&word) {
                        tokens.push(Token::NumberWithUnit(1.0, unit));
                    } else {
                        return None;
                    }
                }
            }
            'i' | 'I' => {
                // Check for "in" keyword
                if i + 1 < chars.len() && chars[i + 1].to_lowercase().next() == Some('n') {
                    // Check if it's actually "in" and not part of a longer word
                    if i + 2 >= chars.len() || !chars[i + 2].is_ascii_alphabetic() {
                        // Skip whitespace after "in"
                        i += 2;
                        while i < chars.len() && chars[i] == ' ' {
                            i += 1;
                        }
                        tokens.push(Token::In);
                    } else {
                        // Fall through to general alphabetic handling
                        let unit_start = i;
                        while i < chars.len()
                            && (chars[i].is_ascii_alphabetic()
                                || chars[i].is_ascii_digit()
                                || chars[i] == '/')
                        {
                            i += 1;
                        }

                        let word: String = chars[unit_start..i].iter().collect();
                        if word.to_lowercase() == "to" {
                            tokens.push(Token::To);
                        } else if word.to_lowercase() == "in" {
                            tokens.push(Token::In);
                        } else if let Some(line_num) = parse_line_reference(&word) {
                            tokens.push(Token::LineReference(line_num));
                        } else if let Some(unit) = parse_unit(&word) {
                            tokens.push(Token::NumberWithUnit(1.0, unit));
                        } else {
                            return None;
                        }
                    }
                } else {
                    // Fall through to general alphabetic handling
                    let unit_start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphabetic()
                            || chars[i].is_ascii_digit()
                            || chars[i] == '/')
                    {
                        i += 1;
                    }

                    let word: String = chars[unit_start..i].iter().collect();
                    if word.to_lowercase() == "to" {
                        tokens.push(Token::To);
                    } else if word.to_lowercase() == "in" {
                        tokens.push(Token::In);
                    } else if let Some(line_num) = parse_line_reference(&word) {
                        tokens.push(Token::LineReference(line_num));
                    } else if let Some(unit) = parse_unit(&word) {
                        tokens.push(Token::NumberWithUnit(1.0, unit));
                    } else {
                        return None;
                    }
                }
            }
            _ => {
                if ch.is_ascii_alphabetic() {
                    // Could be a unit, keyword, or line reference
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
                    if word.to_lowercase() == "to" {
                        tokens.push(Token::To);
                    } else if word.to_lowercase() == "in" {
                        tokens.push(Token::In);
                    } else if let Some(line_num) = parse_line_reference(&word) {
                        tokens.push(Token::LineReference(line_num));
                    } else if let Some(unit) = parse_unit(&word) {
                        // Standalone unit for conversion target
                        tokens.push(Token::NumberWithUnit(1.0, unit));
                    } else {
                        return None; // Unexpected word
                    }
                } else {
                    return None; // Unexpected character
                }
            }
        }
    }

    Some(tokens)
}

/// Find mathematical expressions in text
pub fn find_math_expression(text: &str) -> Vec<String> {
    let mut expressions = Vec::new();
    let chars: Vec<char> = text.chars().collect();

    // Start looking for expressions from the beginning
    let mut i = 0;
    while i < chars.len() {
        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        if i >= chars.len() {
            break;
        }

        // Check if we're at the start of a potential math expression
        if chars[i].is_ascii_digit() || chars[i] == '(' || chars[i] == '-' {
            let start = i;
            let expression = extract_math_portion(&text[start..]);

            if !expression.trim().is_empty() && is_valid_math_expression(&expression) {
                expressions.push(expression.trim().to_string());
            }

            // Move past this expression
            i = start + expression.len();
        } else if chars[i].is_ascii_alphabetic() {
            // Check if this might be a line reference
            let word_start = i;
            while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i].is_ascii_digit()) {
                i += 1;
            }

            let word: String = chars[word_start..i].iter().collect();
            if parse_line_reference(&word).is_some() {
                // This is a line reference, look for math after it
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }

                if i < chars.len()
                    && (chars[i] == '+' || chars[i] == '-' || chars[i] == '*' || chars[i] == '/')
                {
                    // There's an operator after the line reference
                    let start = word_start;
                    let expression = extract_math_portion(&text[start..]);

                    if !expression.trim().is_empty() && is_valid_math_expression(&expression) {
                        expressions.push(expression.trim().to_string());
                    }

                    i = start + expression.len();
                } else {
                    // Just a standalone line reference
                    expressions.push(word);
                }
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    // Filter out sub-expressions
    let mut filtered_expressions = Vec::new();
    for expr in &expressions {
        if !expr.trim().is_empty() {
            let mut is_subexpression = false;
            for other_expr in &expressions {
                if other_expr != expr && other_expr.len() > expr.len() && other_expr.contains(expr)
                {
                    is_subexpression = true;
                    break;
                }
            }
            if !is_subexpression {
                filtered_expressions.push(expr.clone());
            }
        }
    }

    filtered_expressions
}

/// Extract the mathematical portion from text
fn extract_math_portion(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut math_end = 0;
    let mut found_digit = false;
    let mut i = 0;
    let mut last_was_operator = false;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '0'..='9' | '.' | ',' => {
                found_digit = true;
                math_end = i + 1;
                last_was_operator = false;
            }
            '+' | '-' | '*' | '/' => {
                if found_digit {
                    last_was_operator = true;
                    // Don't update math_end yet - wait for next operand
                }
            }
            '(' => {
                math_end = i + 1;
                last_was_operator = false;
            }
            ')' => {
                math_end = i + 1;
                last_was_operator = false;
                // Don't break here - continue to see if there are more operators
            }
            ' ' => {
                // Continue, space is okay
                // Don't change last_was_operator
            }
            _ => {
                if ch.is_ascii_alphabetic() {
                    // Check if this starts a complete known unit (with word boundaries)
                    let remaining = &text[i..];
                    let mut word_end = i;
                    for (j, word_char) in remaining.chars().enumerate() {
                        if word_char.is_ascii_alphabetic() || word_char == '/' {
                            word_end = i + j + 1;
                        } else {
                            break;
                        }
                    }
                    let potential_word = &text[i..word_end];

                    // Only treat as unit if it's a complete unit word, "to", "in", or line reference
                    if (parse_unit(potential_word).is_some()
                        || potential_word.to_lowercase() == "to"
                        || potential_word.to_lowercase() == "in"
                        || parse_line_reference(potential_word).is_some())
                        && (word_end >= text.len()
                            || !text.chars().nth(word_end).unwrap().is_ascii_alphabetic())
                    {
                        math_end = word_end;
                        last_was_operator = false;
                        // Skip to the end of this word
                        i = word_end;
                        continue;
                    } else {
                        // Unknown or partial word, end of math expression here
                        break;
                    }
                } else {
                    // Other character, end of math expression
                    break;
                }
            }
        }

        // If we just processed an operator, update math_end only if we find more content
        if last_was_operator && i + 1 < chars.len() {
            // Look ahead for more content
            let mut j = i + 1;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            if j < chars.len()
                && (chars[j].is_ascii_digit() || chars[j] == '(' || chars[j].is_ascii_alphabetic())
            {
                math_end = i + 1;
            }
        }

        i += 1;
    }

    chars[..math_end]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
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
