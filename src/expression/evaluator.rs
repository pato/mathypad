//! Expression evaluation functions with unit-aware arithmetic

use super::parser::tokenize_with_units;
use super::tokens::Token;
use crate::FLOAT_EPSILON;
use crate::units::{Unit, UnitType, UnitValue, parse_unit};
use crate::rate_unit;
use std::collections::HashMap;

/// Main evaluation function that handles context for line references
pub fn evaluate_expression_with_context(
    text: &str,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<String> {
    // New approach: tokenize everything then find mathematical patterns
    if let Some(tokens) = super::parser::tokenize_with_units(text) {
        // Try to find and evaluate mathematical patterns in the token stream
        if let Some(result) =
            evaluate_tokens_stream_with_context(&tokens, previous_results, current_line)
        {
            return Some(result.format());
        }
    }

    None
}

/// Find and evaluate mathematical patterns in a token stream
pub fn evaluate_tokens_stream_with_context(
    tokens: &[Token],
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    if tokens.is_empty() {
        return None;
    }

    // Look for the longest valid mathematical subsequence
    // Try different starting positions and lengths
    for start in 0..tokens.len() {
        for end in (start + 1..=tokens.len()).rev() {
            // Try longest first
            let subseq = &tokens[start..end];
            if is_valid_mathematical_sequence(subseq) {
                // Try to evaluate this subsequence
                if let Some(result) =
                    evaluate_tokens_with_units_and_context(subseq, previous_results, current_line)
                {
                    return Some(result);
                }
                // If this subsequence failed to evaluate and it spans the entire input with operators,
                // don't try shorter subsequences (this prevents "5 / 0" from evaluating as "5")
                if has_mathematical_operators(subseq) && start == 0 && end == tokens.len() {
                    return None; // Fail entirely for the full expression
                }
            }
        }
    }

    None
}

/// Check if a token sequence contains mathematical operators
fn has_mathematical_operators(tokens: &[Token]) -> bool {
    tokens.iter().any(|t| {
        matches!(
            t,
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide
        )
    })
}

/// Check if a token sequence forms a valid mathematical expression
fn is_valid_mathematical_sequence(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    // Must have at least one number, unit, line reference, or variable
    let has_value = tokens.iter().any(|t| {
        matches!(
            t,
            Token::Number(_)
                | Token::NumberWithUnit(_, _)
                | Token::LineReference(_)
                | Token::Variable(_)
        )
    });

    if !has_value {
        return false;
    }

    // Simple validation: check for basic mathematical patterns
    // More sophisticated validation can be added as needed

    // Pattern 1: Single value (number, unit, variable, line ref)
    if tokens.len() == 1 {
        return matches!(
            tokens[0],
            Token::Number(_)
                | Token::NumberWithUnit(_, _)
                | Token::LineReference(_)
                | Token::Variable(_)
        );
    }

    // Pattern 2: Value + unit conversion (e.g., "5 GiB to TB", "storage to TB")
    if tokens.len() == 3 {
        let is_value_or_var = |t: &Token| {
            matches!(
                t,
                Token::Number(_)
                    | Token::NumberWithUnit(_, _)
                    | Token::LineReference(_)
                    | Token::Variable(_)
            )
        };
        let is_unit_or_var =
            |t: &Token| matches!(t, Token::NumberWithUnit(_, _) | Token::Variable(_));

        if is_value_or_var(&tokens[0])
            && matches!(tokens[1], Token::To | Token::In)
            && is_unit_or_var(&tokens[2])
        {
            return true;
        }

        // Pattern: Percentage of value (e.g., "10% of 50")
        if matches!(tokens[0], Token::NumberWithUnit(_, Unit::Percent))
            && matches!(tokens[1], Token::Of)
            && is_value_or_var(&tokens[2])
        {
            return true;
        }
    }

    // Pattern 3: Binary operations (value op value)
    if tokens.len() == 3 {
        let is_value = |t: &Token| {
            matches!(
                t,
                Token::Number(_)
                    | Token::NumberWithUnit(_, _)
                    | Token::LineReference(_)
                    | Token::Variable(_)
            )
        };
        let is_op = |t: &Token| {
            matches!(
                t,
                Token::Plus | Token::Minus | Token::Multiply | Token::Divide
            )
        };

        if is_value(&tokens[0]) && is_op(&tokens[1]) && is_value(&tokens[2]) {
            return true;
        }
    }

    // Pattern 4: More complex expressions with parentheses, multiple operations
    // For now, if we have values and operators, assume it could be valid
    // The actual evaluation will determine if it's truly valid
    let has_operator = tokens.iter().any(|t| {
        matches!(
            t,
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide
        )
    });

    has_value && (tokens.len() == 1 || has_operator)
}

/// Enhanced evaluation function that handles both expressions and variable assignments
pub fn evaluate_with_variables(
    text: &str,
    variables: &HashMap<String, String>,
    previous_results: &[Option<String>],
    current_line: usize,
) -> (Option<String>, Option<(String, String)>) {
    // Return (result, optional_variable_assignment)

    // New approach: tokenize everything then find patterns
    if let Some(tokens) = super::parser::tokenize_with_units(text) {
        // First check for variable assignments
        if let Some(assignment) =
            find_variable_assignment_in_tokens(&tokens, variables, previous_results, current_line)
        {
            return (Some(assignment.1.clone()), Some(assignment));
        }

        // Then look for mathematical expressions
        if let Some(result) = evaluate_tokens_stream_with_variables(
            &tokens,
            variables,
            previous_results,
            current_line,
        ) {
            return (Some(result.format()), None);
        }
    }

    (None, None)
}

/// Find variable assignment pattern in token stream
fn find_variable_assignment_in_tokens(
    tokens: &[Token],
    variables: &HashMap<String, String>,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<(String, String)> {
    // Look for pattern: Variable Assign Expression
    if tokens.len() >= 3 {
        if let (Token::Variable(var_name), Token::Assign) = (&tokens[0], &tokens[1]) {
            // Extract the right-hand side (everything after =)
            let rhs_tokens = &tokens[2..];

            // Evaluate the right-hand side
            if let Some(value) = evaluate_tokens_with_units_and_context_and_variables(
                rhs_tokens,
                variables,
                previous_results,
                current_line,
            ) {
                return Some((var_name.clone(), value.format()));
            }
        }
    }

    None
}

/// Find and evaluate mathematical patterns in a token stream with variable support
fn evaluate_tokens_stream_with_variables(
    tokens: &[Token],
    variables: &HashMap<String, String>,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    if tokens.is_empty() {
        return None;
    }

    // First check if we have undefined variables in what looks like a mathematical context
    if has_undefined_variables_in_math_context(tokens, variables) {
        return None; // Fail entirely if undefined variables are in mathematical expressions
    }

    // Look for the longest valid mathematical subsequence
    // Try different starting positions and lengths
    for start in 0..tokens.len() {
        for end in (start + 1..=tokens.len()).rev() {
            // Try longest first
            let subseq = &tokens[start..end];
            if is_valid_mathematical_sequence(subseq) && all_variables_defined(subseq, variables) {
                // Try to evaluate this subsequence
                if let Some(result) = evaluate_tokens_with_units_and_context_and_variables(
                    subseq,
                    variables,
                    previous_results,
                    current_line,
                ) {
                    return Some(result);
                }
                // If this subsequence failed to evaluate and it spans the entire input with operators,
                // don't try shorter subsequences (this prevents "5 / 0" from evaluating as "5")
                if has_mathematical_operators(subseq) && start == 0 && end == tokens.len() {
                    return None; // Fail entirely for the full expression
                }
            }
        }
    }

    None
}

/// Check if there are undefined variables in what appears to be a mathematical context
fn has_undefined_variables_in_math_context(
    tokens: &[Token],
    variables: &HashMap<String, String>,
) -> bool {
    // Look for undefined variables that are adjacent to mathematical operators or values
    for i in 0..tokens.len() {
        if let Token::Variable(var_name) = &tokens[i] {
            if !variables.contains_key(var_name) {
                // Check if this undefined variable is in a mathematical context
                let has_math_neighbor = (i > 0 && is_math_token(&tokens[i - 1]))
                    || (i + 1 < tokens.len() && is_math_token(&tokens[i + 1]));

                if has_math_neighbor {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a token is mathematical (operator, number, unit, etc.)
fn is_math_token(token: &Token) -> bool {
    matches!(
        token,
        Token::Number(_)
            | Token::NumberWithUnit(_, _)
            | Token::LineReference(_)
            | Token::Plus
            | Token::Minus
            | Token::Multiply
            | Token::Divide
            | Token::LeftParen
            | Token::RightParen
            | Token::To
            | Token::In
    )
}

/// Check if all variables in a token sequence are defined
fn all_variables_defined(tokens: &[Token], variables: &HashMap<String, String>) -> bool {
    for token in tokens {
        if let Token::Variable(var_name) = token {
            if !variables.contains_key(var_name) {
                return false;
            }
        }
    }
    true
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

        // Handle percentage of value expressions like "10% of 50"
        if let (Token::NumberWithUnit(percentage, Unit::Percent), Token::Of, value_token) =
            (&tokens[0], &tokens[1], &tokens[2])
        {
            // Resolve the value token (could be number, unit, variable, or line reference)
            let base_value = match value_token {
                Token::Number(n) => UnitValue::new(*n, None),
                Token::NumberWithUnit(n, unit) => UnitValue::new(*n, Some(unit.clone())),
                Token::LineReference(line_index) => {
                    resolve_line_reference(*line_index, previous_results, current_line)?
                }
                _ => return None, // Variables would need additional handling
            };

            // Calculate percentage: convert percentage to decimal first, then multiply
            let percentage_decimal = Unit::Percent.to_base_value(*percentage);
            return Some(UnitValue::new(
                percentage_decimal * base_value.value,
                base_value.unit,
            ));
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

/// Variable-aware version of evaluate_tokens_with_units_and_context
fn evaluate_tokens_with_units_and_context_and_variables(
    tokens: &[Token],
    variables: &HashMap<String, String>,
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

        // Handle percentage of value expressions like "10% of 50"
        if let (Token::NumberWithUnit(percentage, Unit::Percent), Token::Of, value_token) =
            (&tokens[0], &tokens[1], &tokens[2])
        {
            // Resolve the value token (could be number, unit, variable, or line reference)
            let base_value = match value_token {
                Token::Number(n) => UnitValue::new(*n, None),
                Token::NumberWithUnit(n, unit) => UnitValue::new(*n, Some(unit.clone())),
                Token::LineReference(line_index) => {
                    resolve_line_reference(*line_index, previous_results, current_line)?
                }
                Token::Variable(var_name) => resolve_variable(var_name, variables)?,
                _ => return None,
            };

            // Calculate percentage: convert percentage to decimal first, then multiply
            let percentage_decimal = Unit::Percent.to_base_value(*percentage);
            return Some(UnitValue::new(
                percentage_decimal * base_value.value,
                base_value.unit,
            ));
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
            Token::Variable(var_name) => {
                // Resolve variable to its value
                if let Some(var_result) = resolve_variable(var_name, variables) {
                    value_stack.push(var_result);
                } else {
                    return None; // Undefined variable
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

/// Resolve a variable to its UnitValue
fn resolve_variable(var_name: &str, variables: &HashMap<String, String>) -> Option<UnitValue> {
    if let Some(var_value_str) = variables.get(var_name) {
        // Parse the variable value back into a UnitValue
        parse_result_string(var_value_str)
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
                    if unit_a.is_compatible_for_addition(unit_b) {
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
                    if unit_a.is_compatible_for_addition(unit_b) {
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
                        && (matches!(rate_unit.unit_type(), UnitType::DataRate(_))) =>
                {
                    // Determine which value is time and which is rate
                    let (time_value, time_u, rate_value, rate_u) =
                        if time_unit.unit_type() == UnitType::Time {
                            (a.value, time_unit, b.value, rate_unit)
                        } else {
                            (b.value, time_unit, a.value, rate_unit)
                        };

                    let time_divider = match rate_unit.unit_type() {
                        UnitType::DataRate(seconds) => seconds,
                        _ => 1.0,
                    };

                    // Convert times to seconds
                    let time_in_seconds = time_u.to_base_value(time_value) / time_divider;

                    // Rate * time = data
                    let data_unit = match rate_u.to_data_unit() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    UnitValue::new(rate_value * time_in_seconds, Some(data_unit))
                }
                // Time * BitRate = Bits
                (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                    if time_unit.unit_type() == UnitType::Time
                        && rate_unit.unit_type() == UnitType::BitRate =>
                {
                    // Check if this is a generic rate unit
                    if let Unit::RateUnit(rate_data, rate_time) = rate_unit {
                        // For generic rates, handle the time conversion properly
                        let (time_value, rate_value) = if time_unit.unit_type() == UnitType::Time {
                            (a.value, b.value)
                        } else {
                            (b.value, a.value)
                        };

                        // Convert time units to match
                        let time_in_rate_units = if time_unit == rate_time.as_ref() {
                            time_value
                        } else {
                            // Convert time to the rate's time unit
                            let time_in_seconds = time_unit.to_base_value(time_value);
                            rate_time.clone().from_base_value(time_in_seconds)
                        };

                        UnitValue::new(
                            rate_value * time_in_rate_units,
                            Some(rate_data.as_ref().clone()),
                        )
                    } else {
                        // Standard bit rate handling (per second)
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
                    if matches!(rate_unit.unit_type(), UnitType::DataRate(_)) =>
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
                    // Check if time unit is seconds - if so, create traditional per-second rate
                    if time_unit == &Unit::Second {
                        // Data / seconds = traditional rate (for backwards compatibility)
                        let rate_unit = match data_unit.to_rate_unit() {
                            Ok(unit) => unit,
                            Err(_) => return false,
                        };
                        UnitValue::new(a.value / b.value, Some(rate_unit))
                    } else {
                        // Data / other time unit = generic rate
                        let rate_unit = Unit::RateUnit(
                            Box::new(data_unit.clone()),
                            Box::new(time_unit.clone()),
                        );
                        UnitValue::new(a.value / b.value, Some(rate_unit))
                    }
                }
                (Some(bit_unit), Some(time_unit))
                    if bit_unit.unit_type() == UnitType::Bit
                        && time_unit.unit_type() == UnitType::Time =>
                {
                    // Check if time unit is seconds - if so, create traditional per-second bit rate
                    if time_unit == &Unit::Second {
                        // Bit / seconds = traditional bit rate (for backwards compatibility)
                        let rate_unit = match bit_unit.to_rate_unit() {
                            Ok(unit) => unit,
                            Err(_) => return false,
                        };
                        UnitValue::new(a.value / b.value, Some(rate_unit))
                    } else {
                        // Bit / other time unit = generic bit rate
                        let rate_unit =
                            rate_unit!(bit_unit.clone(), time_unit.clone());
                        UnitValue::new(a.value / b.value, Some(rate_unit))
                    }
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
                        && matches!(rate_unit.unit_type(), UnitType::DataRate(_)) =>
                {
                    // Check if this is a generic rate unit
                    if let Unit::RateUnit(rate_data, rate_time) = rate_unit {
                        // For generic rates, we need to match the data units and return the time unit
                        if data_unit.unit_type() == rate_data.unit_type() {
                            // Convert both to base units
                            let data_base = data_unit.to_base_value(a.value);
                            let rate_data_base = rate_data.to_base_value(b.value);
                            if rate_data_base.abs() < FLOAT_EPSILON {
                                return false;
                            }
                            let time_value = data_base / rate_data_base;
                            UnitValue::new(time_value, Some(rate_time.as_ref().clone()))
                        } else {
                            return false;
                        }
                    } else {
                        // Standard per-second rate handling
                        let data_in_bytes = data_unit.to_base_value(a.value);
                        let rate_in_bytes_per_sec = rate_unit.to_base_value(b.value);
                        if rate_in_bytes_per_sec.abs() < FLOAT_EPSILON {
                            return false;
                        }
                        let time_in_seconds = data_in_bytes / rate_in_bytes_per_sec;
                        UnitValue::new(time_in_seconds, Some(Unit::Second))
                    }
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
                        && matches!(rate_unit.unit_type(), UnitType::DataRate(_)) =>
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
                // Compatible units divided = dimensionless ratio
                (Some(unit_a), Some(unit_b)) => {
                    // Check if units are compatible (same unit type or bit/data conversion)
                    let compatible = unit_a.unit_type() == unit_b.unit_type()
                        || (unit_a.unit_type() == UnitType::Bit
                            && unit_b.unit_type() == UnitType::Data)
                        || (unit_a.unit_type() == UnitType::Data
                            && unit_b.unit_type() == UnitType::Bit);

                    if compatible {
                        // Convert both to base values and divide to get dimensionless ratio
                        let mut base_a = unit_a.to_base_value(a.value);
                        let mut base_b = unit_b.to_base_value(b.value);

                        // Handle bit/byte conversions: normalize to same base (bits)
                        if unit_a.unit_type() == UnitType::Data
                            && unit_b.unit_type() == UnitType::Bit
                        {
                            base_a *= 8.0; // Convert bytes to bits
                        } else if unit_a.unit_type() == UnitType::Bit
                            && unit_b.unit_type() == UnitType::Data
                        {
                            base_b *= 8.0; // Convert bytes to bits
                        }

                        if base_b.abs() < FLOAT_EPSILON {
                            return false;
                        }
                        let ratio = base_a / base_b;
                        UnitValue::new(ratio, None) // No unit = dimensionless
                    } else {
                        return false; // Incompatible unit types
                    }
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
