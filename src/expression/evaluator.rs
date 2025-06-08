//! Expression evaluation functions with unit-aware arithmetic

use super::parser::{is_valid_math_expression, tokenize_with_units};
use super::tokens::Token;
use crate::FLOAT_EPSILON;
use crate::units::{Unit, UnitType, UnitValue, parse_unit};

/// Main evaluation function that handles context for line references
pub fn evaluate_expression_with_context(
    text: &str,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<String> {
    // Find the longest valid mathematical expression in the text
    let expressions = find_math_expressions(text);

    for expr in expressions {
        // Try unit-aware parsing first
        if let Some(unit_value) =
            parse_and_evaluate_with_context(&expr, previous_results, current_line)
        {
            return Some(unit_value.format());
        }
        // Then try simple parsing
        if let Some(simple_result) = parse_and_evaluate_simple(&expr) {
            let unit_value = UnitValue::new(simple_result, None);
            return Some(unit_value.format());
        }
    }

    None
}

/// Parse and evaluate with context for line references
pub fn parse_and_evaluate_with_context(
    expr: &str,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    let tokens = tokenize_with_units(expr)?;
    evaluate_tokens_with_units_and_context(&tokens, previous_results, current_line)
}

/// Evaluate tokens with unit-aware arithmetic and context support
pub fn evaluate_tokens_with_units_and_context(
    tokens: &[Token],
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    if tokens.is_empty() {
        return None;
    }

    // Handle simple conversion expressions like "1 GiB to KiB" (only if it's the entire expression)
    if tokens.len() == 3 {
        if let (
            Token::NumberWithUnit(value, from_unit),
            Token::To,
            Token::NumberWithUnit(_, to_unit),
        ) = (&tokens[0], &tokens[1], &tokens[2])
        {
            let unit_value = UnitValue::new(*value, Some(from_unit.clone()));
            return unit_value.to_unit(to_unit);
        }
    }

    // Check if we have an "in" or "to" conversion request at the end
    let mut target_unit_for_conversion = None;
    let mut evaluation_tokens = tokens;

    // Look for "in" or "to" followed by a unit at the end
    for i in 0..tokens.len().saturating_sub(1) {
        if let Token::In | Token::To = &tokens[i] {
            // Look for unit after "in" or "to"
            for j in (i + 1)..tokens.len() {
                if let Token::NumberWithUnit(_, unit) = &tokens[j] {
                    target_unit_for_conversion = Some(unit.clone());
                    evaluation_tokens = &tokens[..i]; // Evaluate everything before "in"/"to"
                    break;
                }
            }
            break;
        }
    }

    // Handle simple arithmetic with units
    let mut operator_stack = Vec::new();
    let mut value_stack = Vec::new();

    for token in evaluation_tokens {
        match token {
            Token::Number(n) => {
                value_stack.push(UnitValue::new(*n, None));
            }
            Token::NumberWithUnit(value, unit) => {
                value_stack.push(UnitValue::new(*value, Some(unit.clone())));
            }
            Token::LineReference(line_index) => {
                // Resolve line reference to its calculated result
                if let Some(line_result) =
                    resolve_line_reference(*line_index, previous_results, current_line)
                {
                    value_stack.push(line_result);
                } else {
                    return None; // Invalid or circular reference
                }
            }
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide => {
                while let Some(top_op) = operator_stack.last() {
                    if precedence_unit(token) <= precedence_unit(top_op) {
                        let op = operator_stack.pop().unwrap();
                        if !apply_operator_with_units(&mut value_stack, &op) {
                            return None;
                        }
                    } else {
                        break;
                    }
                }
                operator_stack.push(token.clone());
            }
            Token::LeftParen => {
                operator_stack.push(token.clone());
            }
            Token::RightParen => {
                while let Some(op) = operator_stack.pop() {
                    if matches!(op, Token::LeftParen) {
                        break;
                    }
                    if !apply_operator_with_units(&mut value_stack, &op) {
                        return None;
                    }
                }
            }
            _ => {}
        }
    }

    while let Some(op) = operator_stack.pop() {
        if !apply_operator_with_units(&mut value_stack, &op) {
            return None;
        }
    }

    if value_stack.len() == 1 {
        let mut result = value_stack.pop().unwrap();

        // If we have a target unit for conversion, convert the result
        if let Some(target_unit) = target_unit_for_conversion {
            if let Some(converted) = result.to_unit(&target_unit) {
                result = converted;
            }
        }

        Some(result)
    } else {
        None
    }
}

/// Resolve a line reference to its calculated result
pub fn resolve_line_reference(
    line_index: usize,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    // Prevent circular references
    if line_index >= current_line {
        return None;
    }

    // Check if the referenced line exists and has a result
    if line_index < previous_results.len() {
        if let Some(result_str) = &previous_results[line_index] {
            // Parse the result string back into a UnitValue
            return parse_result_string(result_str);
        }
    }

    None
}

/// Parse a result string back into a UnitValue
pub fn parse_result_string(result_str: &str) -> Option<UnitValue> {
    // Parse a result string like "14 GiB" or "42" back into a UnitValue
    let parts: Vec<&str> = result_str.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    // Try to parse the first part as a number
    let number_str = parts[0].replace(",", ""); // Remove commas
    if let Ok(value) = number_str.parse::<f64>() {
        if parts.len() == 1 {
            // Just a number
            return Some(UnitValue::new(value, None));
        } else if parts.len() == 2 {
            // Number with unit
            if let Some(unit) = parse_unit(parts[1]) {
                return Some(UnitValue::new(value, Some(unit)));
            }
        }
    }

    None
}

/// Simple mathematical expression parsing without units
fn parse_and_evaluate_simple(expr: &str) -> Option<f64> {
    let expr = expr.replace(" ", "");

    if expr.is_empty() {
        return None;
    }

    let mut tokens = Vec::new();
    let mut current_number = String::new();

    for ch in expr.chars() {
        match ch {
            '0'..='9' | '.' | ',' => {
                current_number.push(ch);
            }
            '+' | '-' | '*' | '/' | '(' | ')' => {
                if !current_number.is_empty() {
                    // Remove commas before parsing
                    let cleaned_number = current_number.replace(",", "");
                    if let Ok(num) = cleaned_number.parse::<f64>() {
                        tokens.push(Token::Number(num));
                    } else {
                        return None;
                    }
                    current_number.clear();
                }
                tokens.push(match ch {
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Multiply,
                    '/' => Token::Divide,
                    '(' => Token::LeftParen,
                    ')' => Token::RightParen,
                    _ => return None,
                });
            }
            _ => return None,
        }
    }

    if !current_number.is_empty() {
        // Remove commas before parsing
        let cleaned_number = current_number.replace(",", "");
        if let Ok(num) = cleaned_number.parse::<f64>() {
            tokens.push(Token::Number(num));
        } else {
            return None;
        }
    }

    evaluate_tokens(&tokens)
}

/// Find mathematical expressions in text (fallback function)
fn find_math_expressions(text: &str) -> Vec<String> {
    let mut expressions = Vec::new();
    let chars: Vec<char> = text.chars().collect();

    // First check if the entire text is a valid math expression
    let trimmed_text = text.trim();
    if !trimmed_text.is_empty() && is_valid_math_expression(trimmed_text) {
        expressions.push(trimmed_text.to_string());
        // If the entire text is valid, don't look for sub-expressions
        return expressions;
    }

    // Early validation - check for obviously invalid expressions
    if has_invalid_expression_structure(trimmed_text) {
        return expressions; // Return empty - invalid expression
    }

    // Then look for sub-expressions only if the entire text is NOT valid
    for start in 0..chars.len() {
        if chars[start].is_ascii_digit() || chars[start] == '(' {
            for end in start + 1..=chars.len() {
                let candidate = chars[start..end].iter().collect::<String>();
                let trimmed_candidate = extract_math_portion(&candidate);

                if !trimmed_candidate.is_empty()
                    && is_valid_math_expression(&trimmed_candidate)
                    && trimmed_candidate != trimmed_text
                {
                    // Don't re-add the full text
                    expressions.push(trimmed_candidate);
                }
            }
        }
    }

    // Sort by complexity (length and operator count) descending
    expressions.sort_by(|a, b| {
        let complexity_a = a.len() + a.chars().filter(|c| "+-*/()".contains(*c)).count() * 2;
        let complexity_b = b.len() + b.chars().filter(|c| "+-*/()".contains(*c)).count() * 2;
        complexity_b.cmp(&complexity_a)
    });

    // Remove duplicates and sub-expressions
    let mut filtered_expressions = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for expr in &expressions {
        if seen.insert(expr.clone()) {
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

/// Check if an expression has invalid structure
fn has_invalid_expression_structure(text: &str) -> bool {
    // Check if the text ends with an operator OR starts with an operator (except minus for negation)
    let text_ends_with_operator = {
        let last_char = text.chars().rev().find(|c| !c.is_whitespace());
        matches!(last_char, Some('+') | Some('-') | Some('*') | Some('/'))
    };

    let text_starts_with_operator = {
        let first_char = text.chars().find(|c| !c.is_whitespace());
        matches!(first_char, Some('*') | Some('/') | Some('+'))
    };

    // Check for unbalanced parentheses
    let has_unbalanced_parens = {
        let mut paren_count = 0;
        for ch in text.chars() {
            match ch {
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        paren_count != 0
    };

    text_ends_with_operator || text_starts_with_operator || has_unbalanced_parens
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
                        || potential_word.to_lowercase() == "in")
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

/// Get operator precedence for unit-aware evaluation
fn precedence_unit(token: &Token) -> i32 {
    match token {
        Token::Plus | Token::Minus => 1,
        Token::Multiply | Token::Divide => 2,
        _ => 0,
    }
}

/// Apply an operator to two unit values
fn apply_operator_with_units(stack: &mut Vec<UnitValue>, op: &Token) -> bool {
    if stack.len() < 2 {
        return false;
    }

    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();

    let result = match op {
        Token::Plus => {
            // Addition: units must be compatible
            match (&a.unit, &b.unit) {
                (Some(unit_a), Some(unit_b)) => {
                    if unit_a.unit_type() == unit_b.unit_type() {
                        let base_a = unit_a.to_base_value(a.value);
                        let base_b = unit_b.to_base_value(b.value);
                        let result_base = base_a + base_b;

                        // Choose the smaller unit (larger value) for the result
                        let result_unit = if unit_a.to_base_value(1.0) < unit_b.to_base_value(1.0) {
                            unit_a
                        } else {
                            unit_b
                        };
                        let result_value = result_unit.clone().from_base_value(result_base);
                        UnitValue::new(result_value, Some(result_unit.clone()))
                    } else {
                        return false;
                    }
                }
                (None, None) => UnitValue::new(a.value + b.value, None),
                _ => return false, // Can't add number with unit and number without unit
            }
        }
        Token::Minus => {
            // Subtraction: units must be compatible
            match (&a.unit, &b.unit) {
                (Some(unit_a), Some(unit_b)) => {
                    if unit_a.unit_type() == unit_b.unit_type() {
                        let base_a = unit_a.to_base_value(a.value);
                        let base_b = unit_b.to_base_value(b.value);
                        let result_base = base_a - base_b;

                        // Choose the smaller unit (larger value) for the result
                        let result_unit = if unit_a.to_base_value(1.0) < unit_b.to_base_value(1.0) {
                            unit_a
                        } else {
                            unit_b
                        };
                        let result_value = result_unit.clone().from_base_value(result_base);
                        UnitValue::new(result_value, Some(result_unit.clone()))
                    } else {
                        return false;
                    }
                }
                (None, None) => UnitValue::new(a.value - b.value, None),
                _ => return false,
            }
        }
        Token::Multiply => {
            // Multiplication: special cases for units
            match (&a.unit, &b.unit) {
                // Time * Rate = Data (convert time to seconds first)
                (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                    if time_unit.unit_type() == UnitType::Time
                        && rate_unit.unit_type() == UnitType::DataRate =>
                {
                    // Determine which value is time and which is rate
                    let (time_value, time_u, rate_value, rate_u) =
                        if time_unit.unit_type() == UnitType::Time {
                            (a.value, time_unit, b.value, rate_unit)
                        } else {
                            (b.value, time_unit, a.value, rate_unit)
                        };

                    // Convert time to seconds
                    let time_in_seconds = time_u.to_base_value(time_value);

                    // Rate * time = data
                    let data_unit = match rate_u.to_data_unit() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    UnitValue::new(rate_value * time_in_seconds, Some(data_unit))
                }
                // Time * BitRate = Bits (convert time to seconds first)
                (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                    if time_unit.unit_type() == UnitType::Time
                        && rate_unit.unit_type() == UnitType::BitRate =>
                {
                    // Determine which value is time and which is rate
                    let (time_value, time_u, rate_value, rate_u) =
                        if time_unit.unit_type() == UnitType::Time {
                            (a.value, time_unit, b.value, rate_unit)
                        } else {
                            (b.value, time_unit, a.value, rate_unit)
                        };

                    // Convert time to seconds
                    let time_in_seconds = time_u.to_base_value(time_value);

                    // BitRate * time = bits
                    let bit_unit = match rate_u.to_data_unit() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    UnitValue::new(rate_value * time_in_seconds, Some(bit_unit))
                }
                // Time * RequestRate = Requests (convert time to seconds first)
                (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                    if time_unit.unit_type() == UnitType::Time
                        && rate_unit.unit_type() == UnitType::RequestRate =>
                {
                    // Determine which value is time and which is rate
                    let (time_value, time_u, rate_value, rate_u) =
                        if time_unit.unit_type() == UnitType::Time {
                            (a.value, time_unit, b.value, rate_unit)
                        } else {
                            (b.value, time_unit, a.value, rate_unit)
                        };

                    // Convert time to seconds
                    let time_in_seconds = time_u.to_base_value(time_value);

                    // RequestRate * time = requests
                    let request_unit = match rate_u.to_request_unit() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    UnitValue::new(rate_value * time_in_seconds, Some(request_unit))
                }
                // Data * Time = Data (total transferred) - for specific data units
                (Some(data_unit), Some(time_unit)) | (Some(time_unit), Some(data_unit))
                    if data_unit.unit_type() == UnitType::Data
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    UnitValue::new(a.value * b.value, Some(data_unit.clone()))
                }
                (Some(rate_unit), Some(Unit::Second)) | (Some(Unit::Second), Some(rate_unit))
                    if rate_unit.unit_type() == UnitType::DataRate =>
                {
                    let data_unit = match rate_unit.to_data_unit() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    UnitValue::new(a.value * b.value, Some(data_unit))
                }
                (Some(unit), None) | (None, Some(unit)) => {
                    // Number * unit = unit
                    UnitValue::new(a.value * b.value, Some(unit.clone()))
                }
                (None, None) => UnitValue::new(a.value * b.value, None),
                _ => return false, // Unsupported unit combination
            }
        }
        Token::Divide => {
            match (&a.unit, &b.unit) {
                (Some(data_unit), Some(time_unit))
                    if data_unit.unit_type() == UnitType::Data
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    // Data / time = rate
                    // Convert time to seconds first
                    let time_in_seconds = time_unit.to_base_value(b.value);
                    let rate_unit = match data_unit.to_rate_unit() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    UnitValue::new(a.value / time_in_seconds, Some(rate_unit))
                }
                (Some(request_unit), Some(time_unit))
                    if request_unit.unit_type() == UnitType::Request
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    // Requests / time = request rate
                    // Convert time to seconds first
                    let time_in_seconds = time_unit.to_base_value(b.value);
                    let rate_unit = match request_unit.to_rate_unit() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    UnitValue::new(a.value / time_in_seconds, Some(rate_unit))
                }
                // Data / DataRate = Time
                (Some(data_unit), Some(rate_unit))
                    if data_unit.unit_type() == UnitType::Data
                        && rate_unit.unit_type() == UnitType::DataRate =>
                {
                    // Convert data to bytes and rate to bytes per second
                    let data_in_bytes = data_unit.to_base_value(a.value);
                    let rate_in_bytes_per_sec = rate_unit.to_base_value(b.value);
                    if rate_in_bytes_per_sec.abs() < FLOAT_EPSILON {
                        return false;
                    }
                    let time_in_seconds = data_in_bytes / rate_in_bytes_per_sec;
                    UnitValue::new(time_in_seconds, Some(Unit::Second))
                }
                // Data / BitRate = Time (need to convert between bits and bytes)
                (Some(data_unit), Some(rate_unit))
                    if data_unit.unit_type() == UnitType::Data
                        && rate_unit.unit_type() == UnitType::BitRate =>
                {
                    // Convert data to bytes and rate to bits per second
                    let data_in_bytes = data_unit.to_base_value(a.value);
                    let rate_in_bits_per_sec = rate_unit.to_base_value(b.value);
                    if rate_in_bits_per_sec.abs() < FLOAT_EPSILON {
                        return false;
                    }
                    // Convert bytes to bits (1 byte = 8 bits)
                    let data_in_bits = data_in_bytes * 8.0;
                    let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
                    UnitValue::new(time_in_seconds, Some(Unit::Second))
                }
                // Bit / DataRate = Time (need to convert between bits and bytes)
                (Some(data_unit), Some(rate_unit))
                    if data_unit.unit_type() == UnitType::Bit
                        && rate_unit.unit_type() == UnitType::DataRate =>
                {
                    // Convert data to bits and rate to bytes per second
                    let data_in_bits = data_unit.to_base_value(a.value);
                    let rate_in_bytes_per_sec = rate_unit.to_base_value(b.value);
                    if rate_in_bytes_per_sec.abs() < FLOAT_EPSILON {
                        return false;
                    }
                    // Convert bytes to bits (1 byte = 8 bits)
                    let rate_in_bits_per_sec = rate_in_bytes_per_sec * 8.0;
                    let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
                    UnitValue::new(time_in_seconds, Some(Unit::Second))
                }
                // Bit / BitRate = Time
                (Some(data_unit), Some(rate_unit))
                    if data_unit.unit_type() == UnitType::Bit
                        && rate_unit.unit_type() == UnitType::BitRate =>
                {
                    // Convert data to bits and rate to bits per second
                    let data_in_bits = data_unit.to_base_value(a.value);
                    let rate_in_bits_per_sec = rate_unit.to_base_value(b.value);
                    if rate_in_bits_per_sec.abs() < FLOAT_EPSILON {
                        return false;
                    }
                    let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
                    UnitValue::new(time_in_seconds, Some(Unit::Second))
                }
                (Some(rate_unit), Some(time_unit))
                    if rate_unit.unit_type() == UnitType::RequestRate
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    // RequestRate / time = RequestRate (rate per unit time)
                    // This is a more complex case - dividing a rate by time
                    // For now, we'll treat this as invalid
                    return false;
                }
                (Some(unit), None) => {
                    // unit / number = unit
                    if b.value.abs() < FLOAT_EPSILON {
                        return false;
                    }
                    UnitValue::new(a.value / b.value, Some(unit.clone()))
                }
                (None, None) => {
                    if b.value.abs() < FLOAT_EPSILON {
                        return false;
                    }
                    UnitValue::new(a.value / b.value, None)
                }
                _ => return false,
            }
        }
        _ => return false,
    };

    stack.push(result);
    true
}

/// Simple token evaluation without units
fn evaluate_tokens(tokens: &[Token]) -> Option<f64> {
    if tokens.is_empty() {
        return None;
    }

    let mut output = Vec::new();
    let mut operators = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) => output.push(*n),
            Token::LeftParen => operators.push(token.clone()),
            Token::RightParen => {
                while let Some(op) = operators.pop() {
                    if matches!(op, Token::LeftParen) {
                        break;
                    }
                    if !apply_operator(&mut output, &op) {
                        return None;
                    }
                }
            }
            op => {
                while let Some(top_op) = operators.last() {
                    if matches!(top_op, Token::LeftParen) || precedence(op) > precedence(top_op) {
                        break;
                    }
                    let op_to_apply = operators.pop().unwrap();
                    if !apply_operator(&mut output, &op_to_apply) {
                        return None;
                    }
                }
                operators.push(op.clone());
            }
        }
    }

    while let Some(op) = operators.pop() {
        if !apply_operator(&mut output, &op) {
            return None;
        }
    }

    if output.len() == 1 {
        Some(output[0])
    } else {
        None
    }
}

/// Get operator precedence for simple evaluation
fn precedence(token: &Token) -> i32 {
    match token {
        Token::Plus | Token::Minus => 1,
        Token::Multiply | Token::Divide => 2,
        _ => 0,
    }
}

/// Apply an operator to two numeric values
fn apply_operator(output: &mut Vec<f64>, op: &Token) -> bool {
    if output.len() < 2 {
        return false;
    }

    let b = output.pop().unwrap();
    let a = output.pop().unwrap();

    let result = match op {
        Token::Plus => a + b,
        Token::Minus => a - b,
        Token::Multiply => a * b,
        Token::Divide => {
            if b.abs() < f64::EPSILON {
                return false;
            }
            a / b
        }
        _ => return false,
    };

    output.push(result);
    true
}
