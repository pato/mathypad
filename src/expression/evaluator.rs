//! Expression evaluation functions with unit-aware arithmetic

use super::parser::tokenize_with_units;
use super::tokens::{SemanticToken, Token};
use crate::FLOAT_EPSILON;
use crate::units::{Unit, UnitType, UnitValue, parse_unit};
use std::collections::HashMap;

/// Convert raw tokens into semantic tokens for evaluation
/// This groups related tokens (like Number + Unit) into meaningful evaluation units
pub fn analyze_semantics(tokens: &[Token]) -> Vec<SemanticToken> {
    let mut semantic_tokens = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match (&tokens[i], tokens.get(i + 1), tokens.get(i + 2)) {
            // "Number % of" sequence -> Value(Number%) + PercentOf operation
            (Token::Number(value), Some(Token::Unit(Unit::Percent)), Some(Token::Of)) => {
                let unit_value = UnitValue::new(*value, Some(Unit::Percent));
                semantic_tokens.push(SemanticToken::Value(unit_value));
                semantic_tokens.push(SemanticToken::PercentOf);
                i += 3; // Skip all three tokens
            }

            // Number + Unit -> Value with unit
            (Token::Number(value), Some(Token::Unit(unit)), _) => {
                let unit_value = UnitValue::new(*value, Some(unit.clone()));
                semantic_tokens.push(SemanticToken::Value(unit_value));
                i += 2; // Skip both tokens
            }

            // Standalone Number -> Value without unit
            (Token::Number(value), _, _) => {
                let unit_value = UnitValue::new(*value, None);
                semantic_tokens.push(SemanticToken::Value(unit_value));
                i += 1;
            }

            // "to Unit" -> ConvertTo operation
            (Token::To, Some(Token::Unit(unit)), _) => {
                semantic_tokens.push(SemanticToken::ConvertTo(unit.clone()));
                i += 2; // Skip both tokens
            }

            // "in Unit" -> ConvertIn operation
            (Token::In, Some(Token::Unit(unit)), _) => {
                semantic_tokens.push(SemanticToken::ConvertIn(unit.clone()));
                i += 2; // Skip both tokens
            }

            // Standalone Unit after conversion keyword -> handle as conversion target
            (Token::Unit(unit), _, _) => {
                // Check if previous token suggests this is a conversion target
                let is_conversion_target = semantic_tokens
                    .last()
                    .map(|t| matches!(t, SemanticToken::Value(_)))
                    .unwrap_or(false);

                if is_conversion_target {
                    // This is likely "5 GiB MiB" which should be "5 GiB to MiB"
                    // Treat the unit as a conversion target
                    semantic_tokens.push(SemanticToken::ConvertTo(unit.clone()));
                } else {
                    // Standalone unit becomes a value with coefficient 1.0
                    let unit_value = UnitValue::new(1.0, Some(unit.clone()));
                    semantic_tokens.push(SemanticToken::Value(unit_value));
                }
                i += 1;
            }

            // Legacy NumberWithUnit -> Value (for compatibility during transition)
            (Token::NumberWithUnit(value, unit), _, _) => {
                let unit_value = UnitValue::new(*value, Some(unit.clone()));
                semantic_tokens.push(SemanticToken::Value(unit_value));
                i += 1;
            }

            // Mathematical operators
            (Token::Plus, _, _) => {
                semantic_tokens.push(SemanticToken::Add);
                i += 1;
            }
            (Token::Minus, _, _) => {
                semantic_tokens.push(SemanticToken::Subtract);
                i += 1;
            }
            (Token::Multiply, _, _) => {
                semantic_tokens.push(SemanticToken::Multiply);
                i += 1;
            }
            (Token::Divide, _, _) => {
                semantic_tokens.push(SemanticToken::Divide);
                i += 1;
            }

            // Grouping
            (Token::LeftParen, _, _) => {
                semantic_tokens.push(SemanticToken::LeftParen);
                i += 1;
            }
            (Token::RightParen, _, _) => {
                semantic_tokens.push(SemanticToken::RightParen);
                i += 1;
            }

            // References and variables
            (Token::LineReference(line), _, _) => {
                semantic_tokens.push(SemanticToken::LineReference(*line));
                i += 1;
            }
            (Token::Variable(name), _, _) => {
                semantic_tokens.push(SemanticToken::Variable(name.clone()));
                i += 1;
            }

            // Assignment
            (Token::Assign, _, _) => {
                semantic_tokens.push(SemanticToken::Assign);
                i += 1;
            }

            // Skip conversion keywords when they're handled above
            (Token::To, _, _) | (Token::In, _, _) | (Token::Of, _, _) => {
                // These should have been handled in combination above
                // If we reach here, they're standalone (which might be an error)
                i += 1; // Skip for now
            }
        }
    }

    semantic_tokens
}

/// Evaluate semantic tokens (cleaner than raw token evaluation)
pub fn evaluate_semantic_tokens(
    semantic_tokens: &[SemanticToken],
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    if semantic_tokens.is_empty() {
        return None;
    }

    // Handle simple conversion expressions like "Value ConvertTo(Unit)"
    if semantic_tokens.len() == 2 {
        if let (SemanticToken::Value(value), SemanticToken::ConvertTo(to_unit)) =
            (&semantic_tokens[0], &semantic_tokens[1])
        {
            return value.to_unit(to_unit);
        }
    }

    // Handle percentage operations like "Value(%) PercentOf Value"
    if semantic_tokens.len() == 3 {
        if let (
            SemanticToken::Value(percentage_val),
            SemanticToken::PercentOf,
            SemanticToken::Value(base_val),
        ) = (
            &semantic_tokens[0],
            &semantic_tokens[1],
            &semantic_tokens[2],
        ) {
            // Extract percentage value (should have Percent unit)
            if let Some(Unit::Percent) = percentage_val.unit {
                let percentage = percentage_val.value / 100.0; // Convert % to decimal
                let result_value = base_val.value * percentage;

                // Result has same unit as base value
                return Some(UnitValue::new(result_value, base_val.unit.clone()));
            }
        }
    }

    // Handle simple arithmetic operations
    if semantic_tokens.len() == 3 {
        if let (SemanticToken::Value(left), op, SemanticToken::Value(right)) = (
            &semantic_tokens[0],
            &semantic_tokens[1],
            &semantic_tokens[2],
        ) {
            return match op {
                SemanticToken::Add => {
                    // Addition: units must be compatible
                    match (&left.unit, &right.unit) {
                        (Some(unit_a), Some(unit_b)) => {
                            if unit_a.unit_type() == unit_b.unit_type() {
                                let base_a = unit_a.to_base_value(left.value);
                                let base_b = unit_b.to_base_value(right.value);
                                let result_base = base_a + base_b;

                                // Choose the smaller unit (larger value) for the result
                                let result_unit =
                                    if unit_a.to_base_value(1.0) < unit_b.to_base_value(1.0) {
                                        unit_a
                                    } else {
                                        unit_b
                                    };
                                let result_value = result_unit.clone().from_base_value(result_base);
                                Some(UnitValue::new(result_value, Some(result_unit.clone())))
                            } else {
                                None // Can't add different unit types
                            }
                        }
                        (None, None) => Some(UnitValue::new(left.value + right.value, None)),
                        _ => None, // Can't add number with unit and number without unit
                    }
                }
                SemanticToken::Subtract => {
                    // Subtraction: units must be compatible
                    match (&left.unit, &right.unit) {
                        (Some(unit_a), Some(unit_b)) => {
                            if unit_a.unit_type() == unit_b.unit_type() {
                                let base_a = unit_a.to_base_value(left.value);
                                let base_b = unit_b.to_base_value(right.value);
                                let result_base = base_a - base_b;

                                // Choose the smaller unit (larger value) for the result
                                let result_unit =
                                    if unit_a.to_base_value(1.0) < unit_b.to_base_value(1.0) {
                                        unit_a
                                    } else {
                                        unit_b
                                    };
                                let result_value = result_unit.clone().from_base_value(result_base);
                                Some(UnitValue::new(result_value, Some(result_unit.clone())))
                            } else {
                                None
                            }
                        }
                        (None, None) => Some(UnitValue::new(left.value - right.value, None)),
                        _ => None,
                    }
                }
                SemanticToken::Multiply => {
                    // Multiplication: special cases for units
                    use crate::units::UnitType;
                    match (&left.unit, &right.unit) {
                        // Time * Rate = Data (convert time to seconds first)
                        (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                            if time_unit.unit_type() == UnitType::Time
                                && rate_unit.unit_type() == UnitType::DataRate =>
                        {
                            // Determine which value is time and which is rate
                            let (time_value, time_u, rate_value, rate_u) =
                                if time_unit.unit_type() == UnitType::Time {
                                    (left.value, time_unit, right.value, rate_unit)
                                } else {
                                    (right.value, time_unit, left.value, rate_unit)
                                };

                            // Convert time to seconds
                            let time_in_seconds = time_u.to_base_value(time_value);

                            // Rate * time = data
                            let data_unit = match rate_u.to_data_unit() {
                                Ok(unit) => unit,
                                Err(_) => return None,
                            };
                            Some(UnitValue::new(
                                rate_value * time_in_seconds,
                                Some(data_unit),
                            ))
                        }
                        // Time * BitRate = Bits (convert time to seconds first)
                        (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                            if time_unit.unit_type() == UnitType::Time
                                && rate_unit.unit_type() == UnitType::BitRate =>
                        {
                            // Determine which value is time and which is rate
                            let (time_value, time_u, rate_value, rate_u) =
                                if time_unit.unit_type() == UnitType::Time {
                                    (left.value, time_unit, right.value, rate_unit)
                                } else {
                                    (right.value, time_unit, left.value, rate_unit)
                                };

                            // Convert time to seconds
                            let time_in_seconds = time_u.to_base_value(time_value);

                            // BitRate * time = bits
                            let bit_unit = match rate_u.to_data_unit() {
                                Ok(unit) => unit,
                                Err(_) => return None,
                            };
                            Some(UnitValue::new(rate_value * time_in_seconds, Some(bit_unit)))
                        }
                        // Time * RequestRate = Requests (convert time to seconds first)
                        (Some(time_unit), Some(rate_unit)) | (Some(rate_unit), Some(time_unit))
                            if time_unit.unit_type() == UnitType::Time
                                && rate_unit.unit_type() == UnitType::RequestRate =>
                        {
                            // Determine which value is time and which is rate
                            let (time_value, time_u, rate_value, rate_u) =
                                if time_unit.unit_type() == UnitType::Time {
                                    (left.value, time_unit, right.value, rate_unit)
                                } else {
                                    (right.value, time_unit, left.value, rate_unit)
                                };

                            // Convert time to seconds
                            let time_in_seconds = time_u.to_base_value(time_value);

                            // RequestRate * time = requests
                            let request_unit = match rate_u.to_request_unit() {
                                Ok(unit) => unit,
                                Err(_) => return None,
                            };
                            Some(UnitValue::new(
                                rate_value * time_in_seconds,
                                Some(request_unit),
                            ))
                        }
                        // Data * Time = Data (total transferred) - for specific data units
                        (Some(data_unit), Some(time_unit)) | (Some(time_unit), Some(data_unit))
                            if data_unit.unit_type() == UnitType::Data
                                && time_unit.unit_type() == UnitType::Time =>
                        {
                            Some(UnitValue::new(
                                left.value * right.value,
                                Some(data_unit.clone()),
                            ))
                        }
                        (Some(rate_unit), Some(Unit::Second))
                        | (Some(Unit::Second), Some(rate_unit))
                            if rate_unit.unit_type() == UnitType::DataRate =>
                        {
                            let data_unit = match rate_unit.to_data_unit() {
                                Ok(unit) => unit,
                                Err(_) => return None,
                            };
                            Some(UnitValue::new(left.value * right.value, Some(data_unit)))
                        }
                        (Some(unit), None) | (None, Some(unit)) => {
                            // Number * unit = unit
                            Some(UnitValue::new(left.value * right.value, Some(unit.clone())))
                        }
                        (None, None) => Some(UnitValue::new(left.value * right.value, None)),
                        _ => None, // Unsupported unit combination
                    }
                }
                SemanticToken::Divide => {
                    use crate::units::UnitType;
                    match (&left.unit, &right.unit) {
                        (Some(data_unit), Some(time_unit))
                            if data_unit.unit_type() == UnitType::Data
                                && time_unit.unit_type() == UnitType::Time =>
                        {
                            // Data / time = rate
                            // Convert time to seconds first
                            let time_in_seconds = time_unit.to_base_value(right.value);
                            let rate_unit = match data_unit.to_rate_unit() {
                                Ok(unit) => unit,
                                Err(_) => return None,
                            };
                            Some(UnitValue::new(
                                left.value / time_in_seconds,
                                Some(rate_unit),
                            ))
                        }
                        (Some(request_unit), Some(time_unit))
                            if request_unit.unit_type() == UnitType::Request
                                && time_unit.unit_type() == UnitType::Time =>
                        {
                            // Requests / time = request rate
                            // Convert time to seconds first
                            let time_in_seconds = time_unit.to_base_value(right.value);
                            let rate_unit = match request_unit.to_rate_unit() {
                                Ok(unit) => unit,
                                Err(_) => return None,
                            };
                            Some(UnitValue::new(
                                left.value / time_in_seconds,
                                Some(rate_unit),
                            ))
                        }
                        // Data / DataRate = Time
                        (Some(data_unit), Some(rate_unit))
                            if data_unit.unit_type() == UnitType::Data
                                && rate_unit.unit_type() == UnitType::DataRate =>
                        {
                            // Convert data to bytes and rate to bytes per second
                            let data_in_bytes = data_unit.to_base_value(left.value);
                            let rate_in_bytes_per_sec = rate_unit.to_base_value(right.value);
                            if rate_in_bytes_per_sec.abs() < crate::FLOAT_EPSILON {
                                return None;
                            }
                            let time_in_seconds = data_in_bytes / rate_in_bytes_per_sec;
                            Some(UnitValue::new(time_in_seconds, Some(Unit::Second)))
                        }
                        // Data / BitRate = Time (need to convert between bits and bytes)
                        (Some(data_unit), Some(rate_unit))
                            if data_unit.unit_type() == UnitType::Data
                                && rate_unit.unit_type() == UnitType::BitRate =>
                        {
                            // Convert data to bytes and rate to bits per second
                            let data_in_bytes = data_unit.to_base_value(left.value);
                            let rate_in_bits_per_sec = rate_unit.to_base_value(right.value);
                            if rate_in_bits_per_sec.abs() < crate::FLOAT_EPSILON {
                                return None;
                            }
                            // Convert bytes to bits (1 byte = 8 bits)
                            let data_in_bits = data_in_bytes * 8.0;
                            let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
                            Some(UnitValue::new(time_in_seconds, Some(Unit::Second)))
                        }
                        // Bit / DataRate = Time (need to convert between bits and bytes)
                        (Some(data_unit), Some(rate_unit))
                            if data_unit.unit_type() == UnitType::Bit
                                && rate_unit.unit_type() == UnitType::DataRate =>
                        {
                            // Convert data to bits and rate to bytes per second
                            let data_in_bits = data_unit.to_base_value(left.value);
                            let rate_in_bytes_per_sec = rate_unit.to_base_value(right.value);
                            if rate_in_bytes_per_sec.abs() < crate::FLOAT_EPSILON {
                                return None;
                            }
                            // Convert bytes to bits (1 byte = 8 bits)
                            let rate_in_bits_per_sec = rate_in_bytes_per_sec * 8.0;
                            let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
                            Some(UnitValue::new(time_in_seconds, Some(Unit::Second)))
                        }
                        // Bit / BitRate = Time
                        (Some(data_unit), Some(rate_unit))
                            if data_unit.unit_type() == UnitType::Bit
                                && rate_unit.unit_type() == UnitType::BitRate =>
                        {
                            // Convert data to bits and rate to bits per second
                            let data_in_bits = data_unit.to_base_value(left.value);
                            let rate_in_bits_per_sec = rate_unit.to_base_value(right.value);
                            if rate_in_bits_per_sec.abs() < crate::FLOAT_EPSILON {
                                return None;
                            }
                            let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
                            Some(UnitValue::new(time_in_seconds, Some(Unit::Second)))
                        }
                        (Some(rate_unit), Some(time_unit))
                            if rate_unit.unit_type() == UnitType::RequestRate
                                && time_unit.unit_type() == UnitType::Time =>
                        {
                            // RequestRate / time = RequestRate (rate per unit time)
                            // This is a more complex case - dividing a rate by time
                            // For now, we'll treat this as invalid
                            None
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
                                let mut base_a = unit_a.to_base_value(left.value);
                                let mut base_b = unit_b.to_base_value(right.value);

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

                                if base_b.abs() < crate::FLOAT_EPSILON {
                                    return None;
                                }
                                let ratio = base_a / base_b;
                                Some(UnitValue::new(ratio, None)) // No unit = dimensionless
                            } else {
                                None // Incompatible unit types
                            }
                        }
                        (Some(unit), None) => {
                            // unit / number = unit
                            if right.value.abs() < crate::FLOAT_EPSILON {
                                return None;
                            }
                            Some(UnitValue::new(left.value / right.value, Some(unit.clone())))
                        }
                        (None, None) => {
                            if right.value.abs() < crate::FLOAT_EPSILON {
                                return None;
                            }
                            Some(UnitValue::new(left.value / right.value, None))
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
        }
    }

    // Handle complex expressions with full evaluation
    evaluate_complex_semantic_expression(semantic_tokens, previous_results, current_line)
}

/// Find variable assignment pattern in semantic token stream
fn find_variable_assignment_in_semantic_tokens(
    semantic_tokens: &[SemanticToken],
    variables: &HashMap<String, String>,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<(String, String)> {
    // Look for pattern: Variable Assign Expression
    if semantic_tokens.len() >= 3 {
        if let (SemanticToken::Variable(var_name), SemanticToken::Assign) = 
            (&semantic_tokens[0], &semantic_tokens[1]) 
        {
            // Extract the right-hand side (everything after =)
            let rhs_tokens = &semantic_tokens[2..];

            // Evaluate the right-hand side using semantic evaluation
            if let Some(value) = evaluate_complex_semantic_expression_with_variables(
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

/// Evaluate complex semantic expressions with variable support
fn evaluate_complex_semantic_expression_with_variables(
    semantic_tokens: &[SemanticToken],
    variables: &HashMap<String, String>,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    if semantic_tokens.is_empty() {
        return None;
    }

    // Convert any line references or variables to values first
    let mut resolved_tokens = Vec::new();
    for token in semantic_tokens {
        match token {
            SemanticToken::LineReference(line_index) => {
                if let Some(line_result) = resolve_line_reference(*line_index, previous_results, current_line) {
                    resolved_tokens.push(SemanticToken::Value(line_result));
                } else {
                    return None; // Invalid or circular reference
                }
            }
            SemanticToken::Variable(var_name) => {
                if let Some(var_result) = resolve_variable(var_name, variables) {
                    resolved_tokens.push(SemanticToken::Value(var_result));
                } else {
                    return None; // Undefined variable
                }
            }
            _ => resolved_tokens.push(token.clone()),
        }
    }

    // Check for conversion operations at the end (like "expression to unit")
    let mut target_unit_for_conversion = None;
    let mut evaluation_tokens = &resolved_tokens[..];
    
    // Look for ConvertTo or ConvertIn at the end
    if let Some(SemanticToken::ConvertTo(unit)) | Some(SemanticToken::ConvertIn(unit)) = resolved_tokens.last() {
        target_unit_for_conversion = Some(unit.clone());
        evaluation_tokens = &resolved_tokens[..resolved_tokens.len()-1];
    }

    // Evaluate the main expression using operator precedence
    let result = evaluate_semantic_expression_with_precedence(evaluation_tokens)?;

    // Apply conversion if needed
    if let Some(target_unit) = target_unit_for_conversion {
        result.to_unit(&target_unit)
    } else {
        Some(result)
    }
}

/// Evaluate complex semantic expressions using a shunting-yard-like algorithm
fn evaluate_complex_semantic_expression(
    semantic_tokens: &[SemanticToken],
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<UnitValue> {
    if semantic_tokens.is_empty() {
        return None;
    }

    // Convert any line references or variables to values first
    let mut resolved_tokens = Vec::new();
    for token in semantic_tokens {
        match token {
            SemanticToken::LineReference(line_index) => {
                if let Some(line_result) =
                    resolve_line_reference(*line_index, previous_results, current_line)
                {
                    resolved_tokens.push(SemanticToken::Value(line_result));
                } else {
                    return None; // Invalid or circular reference
                }
            }
            // TODO: Add variable resolution when we have access to variables
            _ => resolved_tokens.push(token.clone()),
        }
    }

    // Check for conversion operations at the end (like "expression to unit")
    let mut target_unit_for_conversion = None;
    let mut evaluation_tokens = &resolved_tokens[..];

    // Look for ConvertTo or ConvertIn at the end
    if let Some(SemanticToken::ConvertTo(unit)) | Some(SemanticToken::ConvertIn(unit)) =
        resolved_tokens.last()
    {
        target_unit_for_conversion = Some(unit.clone());
        evaluation_tokens = &resolved_tokens[..resolved_tokens.len() - 1];
    }

    // Evaluate the main expression using operator precedence
    let result = evaluate_semantic_expression_with_precedence(evaluation_tokens)?;

    // Apply conversion if needed
    if let Some(target_unit) = target_unit_for_conversion {
        result.to_unit(&target_unit)
    } else {
        Some(result)
    }
}

/// Evaluate semantic expression with operator precedence (shunting yard algorithm)
fn evaluate_semantic_expression_with_precedence(tokens: &[SemanticToken]) -> Option<UnitValue> {
    if tokens.is_empty() {
        return None;
    }

    // Single value case
    if tokens.len() == 1 {
        if let SemanticToken::Value(value) = &tokens[0] {
            return Some(value.clone());
        }
        return None;
    }

    let mut operator_stack = Vec::new();
    let mut value_stack = Vec::new();

    for token in tokens {
        match token {
            SemanticToken::Value(value) => {
                value_stack.push(value.clone());
            }
            SemanticToken::Add
            | SemanticToken::Subtract
            | SemanticToken::Multiply
            | SemanticToken::Divide => {
                while let Some(top_op) = operator_stack.last() {
                    if semantic_precedence(token) <= semantic_precedence(top_op) {
                        let op = operator_stack.pop().unwrap();
                        if !apply_semantic_operator(&mut value_stack, &op) {
                            return None;
                        }
                    } else {
                        break;
                    }
                }
                operator_stack.push(token.clone());
            }
            SemanticToken::LeftParen => {
                operator_stack.push(token.clone());
            }
            SemanticToken::RightParen => {
                while let Some(op) = operator_stack.pop() {
                    if matches!(op, SemanticToken::LeftParen) {
                        break;
                    }
                    if !apply_semantic_operator(&mut value_stack, &op) {
                        return None;
                    }
                }
            }
            SemanticToken::PercentOf => {
                // Handle percentage operations specially
                if value_stack.len() >= 2 {
                    let base_val = value_stack.pop().unwrap();
                    let percentage_val = value_stack.pop().unwrap();

                    if let Some(Unit::Percent) = percentage_val.unit {
                        let percentage = percentage_val.value / 100.0;
                        let result_value = base_val.value * percentage;
                        value_stack.push(UnitValue::new(result_value, base_val.unit));
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ => {
                // Skip other tokens (ConvertTo, ConvertIn handled elsewhere)
            }
        }
    }

    // Apply remaining operators
    while let Some(op) = operator_stack.pop() {
        if !apply_semantic_operator(&mut value_stack, &op) {
            return None;
        }
    }

    if value_stack.len() == 1 {
        Some(value_stack.pop().unwrap())
    } else {
        None
    }
}

/// Get operator precedence for semantic tokens
fn semantic_precedence(token: &SemanticToken) -> i32 {
    match token {
        SemanticToken::Add | SemanticToken::Subtract => 1,
        SemanticToken::Multiply | SemanticToken::Divide => 2,
        _ => 0,
    }
}

/// Apply a semantic operator to two values
fn apply_semantic_operator(stack: &mut Vec<UnitValue>, op: &SemanticToken) -> bool {
    if stack.len() < 2 {
        return false;
    }

    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();

    let result = match op {
        SemanticToken::Add => {
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
        SemanticToken::Subtract => {
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
        SemanticToken::Multiply => {
            // Use the same complex multiplication logic from the simple case
            match multiply_unit_values(&a, &b) {
                Some(result) => result,
                None => return false,
            }
        }
        SemanticToken::Divide => {
            // Use the same complex division logic from the simple case
            match divide_unit_values(&a, &b) {
                Some(result) => result,
                None => return false,
            }
        }
        _ => return false,
    };

    stack.push(result);
    true
}

/// Extract multiplication logic into a separate function for reuse
fn multiply_unit_values(a: &UnitValue, b: &UnitValue) -> Option<UnitValue> {
    use crate::units::UnitType;
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
                Err(_) => return None,
            };
            Some(UnitValue::new(
                rate_value * time_in_seconds,
                Some(data_unit),
            ))
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
                Err(_) => return None,
            };
            Some(UnitValue::new(rate_value * time_in_seconds, Some(bit_unit)))
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
                Err(_) => return None,
            };
            Some(UnitValue::new(
                rate_value * time_in_seconds,
                Some(request_unit),
            ))
        }
        // Data * Time = Data (total transferred) - for specific data units
        (Some(data_unit), Some(time_unit)) | (Some(time_unit), Some(data_unit))
            if data_unit.unit_type() == UnitType::Data
                && time_unit.unit_type() == UnitType::Time =>
        {
            Some(UnitValue::new(a.value * b.value, Some(data_unit.clone())))
        }
        (Some(rate_unit), Some(Unit::Second)) | (Some(Unit::Second), Some(rate_unit))
            if rate_unit.unit_type() == UnitType::DataRate =>
        {
            let data_unit = match rate_unit.to_data_unit() {
                Ok(unit) => unit,
                Err(_) => return None,
            };
            Some(UnitValue::new(a.value * b.value, Some(data_unit)))
        }
        (Some(unit), None) | (None, Some(unit)) => {
            // Number * unit = unit
            Some(UnitValue::new(a.value * b.value, Some(unit.clone())))
        }
        (None, None) => Some(UnitValue::new(a.value * b.value, None)),
        _ => None, // Unsupported unit combination
    }
}

/// Extract division logic into a separate function for reuse
fn divide_unit_values(a: &UnitValue, b: &UnitValue) -> Option<UnitValue> {
    use crate::units::UnitType;
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
                Err(_) => return None,
            };
            Some(UnitValue::new(a.value / time_in_seconds, Some(rate_unit)))
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
                Err(_) => return None,
            };
            Some(UnitValue::new(a.value / time_in_seconds, Some(rate_unit)))
        }
        // Data / DataRate = Time
        (Some(data_unit), Some(rate_unit))
            if data_unit.unit_type() == UnitType::Data
                && rate_unit.unit_type() == UnitType::DataRate =>
        {
            // Convert data to bytes and rate to bytes per second
            let data_in_bytes = data_unit.to_base_value(a.value);
            let rate_in_bytes_per_sec = rate_unit.to_base_value(b.value);
            if rate_in_bytes_per_sec.abs() < crate::FLOAT_EPSILON {
                return None;
            }
            let time_in_seconds = data_in_bytes / rate_in_bytes_per_sec;
            Some(UnitValue::new(time_in_seconds, Some(Unit::Second)))
        }
        // Data / BitRate = Time (need to convert between bits and bytes)
        (Some(data_unit), Some(rate_unit))
            if data_unit.unit_type() == UnitType::Data
                && rate_unit.unit_type() == UnitType::BitRate =>
        {
            // Convert data to bytes and rate to bits per second
            let data_in_bytes = data_unit.to_base_value(a.value);
            let rate_in_bits_per_sec = rate_unit.to_base_value(b.value);
            if rate_in_bits_per_sec.abs() < crate::FLOAT_EPSILON {
                return None;
            }
            // Convert bytes to bits (1 byte = 8 bits)
            let data_in_bits = data_in_bytes * 8.0;
            let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
            Some(UnitValue::new(time_in_seconds, Some(Unit::Second)))
        }
        // Bit / DataRate = Time (need to convert between bits and bytes)
        (Some(data_unit), Some(rate_unit))
            if data_unit.unit_type() == UnitType::Bit
                && rate_unit.unit_type() == UnitType::DataRate =>
        {
            // Convert data to bits and rate to bytes per second
            let data_in_bits = data_unit.to_base_value(a.value);
            let rate_in_bytes_per_sec = rate_unit.to_base_value(b.value);
            if rate_in_bytes_per_sec.abs() < crate::FLOAT_EPSILON {
                return None;
            }
            // Convert bytes to bits (1 byte = 8 bits)
            let rate_in_bits_per_sec = rate_in_bytes_per_sec * 8.0;
            let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
            Some(UnitValue::new(time_in_seconds, Some(Unit::Second)))
        }
        // Bit / BitRate = Time
        (Some(data_unit), Some(rate_unit))
            if data_unit.unit_type() == UnitType::Bit
                && rate_unit.unit_type() == UnitType::BitRate =>
        {
            // Convert data to bits and rate to bits per second
            let data_in_bits = data_unit.to_base_value(a.value);
            let rate_in_bits_per_sec = rate_unit.to_base_value(b.value);
            if rate_in_bits_per_sec.abs() < crate::FLOAT_EPSILON {
                return None;
            }
            let time_in_seconds = data_in_bits / rate_in_bits_per_sec;
            Some(UnitValue::new(time_in_seconds, Some(Unit::Second)))
        }
        (Some(rate_unit), Some(time_unit))
            if rate_unit.unit_type() == UnitType::RequestRate
                && time_unit.unit_type() == UnitType::Time =>
        {
            // RequestRate / time = RequestRate (rate per unit time)
            // This is a more complex case - dividing a rate by time
            // For now, we'll treat this as invalid
            None
        }
        // Compatible units divided = dimensionless ratio
        (Some(unit_a), Some(unit_b)) => {
            // Check if units are compatible (same unit type or bit/data conversion)
            let compatible = unit_a.unit_type() == unit_b.unit_type()
                || (unit_a.unit_type() == UnitType::Bit && unit_b.unit_type() == UnitType::Data)
                || (unit_a.unit_type() == UnitType::Data && unit_b.unit_type() == UnitType::Bit);

            if compatible {
                // Convert both to base values and divide to get dimensionless ratio
                let mut base_a = unit_a.to_base_value(a.value);
                let mut base_b = unit_b.to_base_value(b.value);

                // Handle bit/byte conversions: normalize to same base (bits)
                if unit_a.unit_type() == UnitType::Data && unit_b.unit_type() == UnitType::Bit {
                    base_a *= 8.0; // Convert bytes to bits
                } else if unit_a.unit_type() == UnitType::Bit
                    && unit_b.unit_type() == UnitType::Data
                {
                    base_b *= 8.0; // Convert bytes to bits
                }

                if base_b.abs() < crate::FLOAT_EPSILON {
                    return None;
                }
                let ratio = base_a / base_b;
                Some(UnitValue::new(ratio, None)) // No unit = dimensionless
            } else {
                None // Incompatible unit types
            }
        }
        (Some(unit), None) => {
            // unit / number = unit
            if b.value.abs() < crate::FLOAT_EPSILON {
                return None;
            }
            Some(UnitValue::new(a.value / b.value, Some(unit.clone())))
        }
        (None, None) => {
            if b.value.abs() < crate::FLOAT_EPSILON {
                return None;
            }
            Some(UnitValue::new(a.value / b.value, None))
        }
        _ => None,
    }
}

/// Preprocess tokens to convert Number + Unit pairs to NumberWithUnit tokens
/// This maintains compatibility with existing evaluation logic while supporting the new tokenization
pub fn preprocess_tokens_for_evaluation(tokens: &[Token]) -> Vec<Token> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match (&tokens[i], tokens.get(i + 1)) {
            // Convert Number + Unit pairs to NumberWithUnit
            (Token::Number(value), Some(Token::Unit(unit))) => {
                result.push(Token::NumberWithUnit(*value, unit.clone()));
                i += 2; // Skip both tokens
            }
            // Handle Unit tokens that come after conversion keywords (to/in)
            (Token::Unit(unit), _) => {
                // Check if this Unit follows a "to" or "in" keyword
                let after_conversion_keyword = result
                    .last()
                    .map(|t| matches!(t, Token::To | Token::In))
                    .unwrap_or(false);

                if after_conversion_keyword {
                    // For conversion contexts, preserve the unit as NumberWithUnit(1.0, unit)
                    // This allows the conversion logic to extract the target unit
                    result.push(Token::NumberWithUnit(1.0, unit.clone()));
                } else {
                    // For other contexts (like standalone units), also treat as NumberWithUnit(1.0, unit)
                    result.push(Token::NumberWithUnit(1.0, unit.clone()));
                }
                i += 1;
            }
            // Handle other tokens normally
            _ => {
                result.push(tokens[i].clone());
                i += 1;
            }
        }
    }

    result
}

/// Semantic-based evaluation function (new approach)
pub fn evaluate_expression_with_context_semantic(
    text: &str,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<String> {
    // New semantic approach: tokenize -> analyze semantics -> evaluate
    if let Some(tokens) = super::parser::tokenize_with_units(text) {
        let semantic_tokens = analyze_semantics(&tokens);
        if let Some(result) =
            evaluate_semantic_tokens(&semantic_tokens, previous_results, current_line)
        {
            return Some(result.format());
        }
    }
    None
}

/// Main evaluation function that handles context for line references (legacy approach)
pub fn evaluate_expression_with_context(
    text: &str,
    previous_results: &[Option<String>],
    current_line: usize,
) -> Option<String> {
    // New approach: tokenize everything then find mathematical patterns
    if let Some(tokens) = super::parser::tokenize_with_units(text) {
        // Apply preprocessing to convert Number + Unit pairs to NumberWithUnit
        let preprocessed_tokens = preprocess_tokens_for_evaluation(&tokens);

        // Try to find and evaluate mathematical patterns in the token stream
        if let Some(result) = evaluate_tokens_stream_with_context(
            &preprocessed_tokens,
            previous_results,
            current_line,
        ) {
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
                // Preprocess tokens to convert Number + Unit pairs to NumberWithUnit tokens
                let preprocessed_tokens = preprocess_tokens_for_evaluation(subseq);

                // Try to evaluate this subsequence
                if let Some(result) = evaluate_tokens_with_units_and_context(
                    &preprocessed_tokens,
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
                | Token::Unit(_)
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
                | Token::Unit(_)
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
                    | Token::Unit(_)
                    | Token::LineReference(_)
                    | Token::Variable(_)
            )
        };
        let is_unit_or_var = |t: &Token| {
            matches!(
                t,
                Token::NumberWithUnit(_, _) | Token::Unit(_) | Token::Variable(_)
            )
        };

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

    // Pattern 2.5: Percentage operations (value unit of value) - 4 tokens
    if tokens.len() == 4 {
        let is_value_or_var = |t: &Token| {
            matches!(
                t,
                Token::Number(_)
                    | Token::NumberWithUnit(_, _)
                    | Token::Unit(_)
                    | Token::LineReference(_)
                    | Token::Variable(_)
            )
        };

        // Check for "Number Unit Of Value" pattern (e.g., "10 % of 50")
        if is_value_or_var(&tokens[0])
            && matches!(tokens[1], Token::Unit(_))
            && matches!(tokens[2], Token::Of)
            && is_value_or_var(&tokens[3])
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
                    | Token::Unit(_)
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

/// Semantic-based evaluation function with variable support (new approach)
pub fn evaluate_with_variables_semantic(
    text: &str,
    variables: &HashMap<String, String>,
    previous_results: &[Option<String>],
    current_line: usize,
) -> (Option<String>, Option<(String, String)>) {
    // Return (result, optional_variable_assignment)

    // New semantic approach: tokenize -> analyze semantics -> evaluate
    if let Some(tokens) = super::parser::tokenize_with_units(text) {
        let semantic_tokens = analyze_semantics(&tokens);

        // First check for variable assignments
        if let Some(assignment) = find_variable_assignment_in_semantic_tokens(
            &semantic_tokens,
            variables,
            previous_results,
            current_line,
        ) {
            return (Some(assignment.1.clone()), Some(assignment));
        }

        // Then look for mathematical expressions
        if let Some(result) = evaluate_complex_semantic_expression_with_variables(
            &semantic_tokens,
            variables,
            previous_results,
            current_line,
        ) {
            return (Some(result.format()), None);
        }
    }

    (None, None)
}

/// Enhanced evaluation function that handles both expressions and variable assignments (legacy approach)
pub fn evaluate_with_variables(
    text: &str,
    variables: &HashMap<String, String>,
    previous_results: &[Option<String>],
    current_line: usize,
) -> (Option<String>, Option<(String, String)>) {
    // Return (result, optional_variable_assignment)

    // New approach: tokenize everything then find patterns
    if let Some(tokens) = super::parser::tokenize_with_units(text) {
        // Apply preprocessing to convert Number + Unit pairs to NumberWithUnit
        let preprocessed_tokens = preprocess_tokens_for_evaluation(&tokens);

        // First check for variable assignments
        if let Some(assignment) = find_variable_assignment_in_tokens(
            &preprocessed_tokens,
            variables,
            previous_results,
            current_line,
        ) {
            return (Some(assignment.1.clone()), Some(assignment));
        }

        // Then look for mathematical expressions
        if let Some(result) = evaluate_tokens_stream_with_variables(
            &preprocessed_tokens,
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

            // Preprocess tokens to convert Number + Unit pairs to NumberWithUnit tokens
            let preprocessed_tokens = preprocess_tokens_for_evaluation(rhs_tokens);

            // Evaluate the right-hand side
            if let Some(value) = evaluate_tokens_with_units_and_context_and_variables(
                &preprocessed_tokens,
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
                // Preprocess tokens to convert Number + Unit pairs to NumberWithUnit tokens
                let preprocessed_tokens = preprocess_tokens_for_evaluation(subseq);

                // Try to evaluate this subsequence
                if let Some(result) = evaluate_tokens_with_units_and_context_and_variables(
                    &preprocessed_tokens,
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
            | Token::Unit(_)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_analysis() {
        use crate::units::Unit;

        // Test Number + Unit -> Value
        let tokens = vec![Token::Number(5.0), Token::Unit(Unit::GiB)];
        let semantic = analyze_semantics(&tokens);
        assert_eq!(semantic.len(), 1);
        if let SemanticToken::Value(val) = &semantic[0] {
            assert_eq!(val.value, 5.0);
            assert_eq!(val.unit, Some(Unit::GiB));
        } else {
            panic!("Expected Value token");
        }

        // Test conversion: "5 GiB to MiB"
        let tokens = vec![
            Token::Number(5.0),
            Token::Unit(Unit::GiB),
            Token::To,
            Token::Unit(Unit::MiB),
        ];
        let semantic = analyze_semantics(&tokens);
        assert_eq!(semantic.len(), 2);
        assert!(matches!(semantic[0], SemanticToken::Value(_)));
        assert!(matches!(semantic[1], SemanticToken::ConvertTo(Unit::MiB)));

        // Test percentage: "10% of 50"
        let tokens = vec![
            Token::Number(10.0),
            Token::Unit(Unit::Percent),
            Token::Of,
            Token::Number(50.0),
        ];
        let semantic = analyze_semantics(&tokens);
        assert_eq!(semantic.len(), 3);
        assert!(matches!(semantic[0], SemanticToken::Value(_)));
        assert!(matches!(semantic[1], SemanticToken::PercentOf));
        assert!(matches!(semantic[2], SemanticToken::Value(_)));

        // Test arithmetic: "5 GiB + 512 MiB"
        let tokens = vec![
            Token::Number(5.0),
            Token::Unit(Unit::GiB),
            Token::Plus,
            Token::Number(512.0),
            Token::Unit(Unit::MiB),
        ];
        let semantic = analyze_semantics(&tokens);
        assert_eq!(semantic.len(), 3);
        assert!(matches!(semantic[0], SemanticToken::Value(_)));
        assert!(matches!(semantic[1], SemanticToken::Add));
        assert!(matches!(semantic[2], SemanticToken::Value(_)));
    }

    #[test]
    fn test_semantic_evaluation() {
        use crate::units::Unit;

        // Test simple conversion: "5 GiB to MiB"
        let tokens = vec![
            Token::Number(5.0),
            Token::Unit(Unit::GiB),
            Token::To,
            Token::Unit(Unit::MiB),
        ];
        let semantic = analyze_semantics(&tokens);
        let result = evaluate_semantic_tokens(&semantic, &[], 0);
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert_eq!(unit_val.value, 5120.0); // 5 * 1024
        assert_eq!(unit_val.unit, Some(Unit::MiB));

        // Test percentage: "10% of 50"
        let tokens = vec![
            Token::Number(10.0),
            Token::Unit(Unit::Percent),
            Token::Of,
            Token::Number(50.0),
        ];
        let semantic = analyze_semantics(&tokens);
        let result = evaluate_semantic_tokens(&semantic, &[], 0);
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert_eq!(unit_val.value, 5.0); // 10% of 50 = 5
        assert_eq!(unit_val.unit, None);

        // Test arithmetic: "5 GiB + 512 MiB"
        let tokens = vec![
            Token::Number(5.0),
            Token::Unit(Unit::GiB),
            Token::Plus,
            Token::Number(512.0),
            Token::Unit(Unit::MiB),
        ];
        let semantic = analyze_semantics(&tokens);
        let result = evaluate_semantic_tokens(&semantic, &[], 0);
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert_eq!(unit_val.value, 5632.0); // 5*1024 + 512 = 5632
        assert_eq!(unit_val.unit, Some(Unit::MiB));

        // Test multiplication: "2 * 512 MiB"
        let tokens = vec![
            Token::Number(2.0),
            Token::Multiply,
            Token::Number(512.0),
            Token::Unit(Unit::MiB),
        ];
        let semantic = analyze_semantics(&tokens);
        let result = evaluate_semantic_tokens(&semantic, &[], 0);
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert_eq!(unit_val.value, 1024.0); // 2 * 512 = 1024
        assert_eq!(unit_val.unit, Some(Unit::MiB));

        // Test division: "1024 MiB / 2"
        let tokens = vec![
            Token::Number(1024.0),
            Token::Unit(Unit::MiB),
            Token::Divide,
            Token::Number(2.0),
        ];
        let semantic = analyze_semantics(&tokens);
        let result = evaluate_semantic_tokens(&semantic, &[], 0);
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert_eq!(unit_val.value, 512.0); // 1024 / 2 = 512
        assert_eq!(unit_val.unit, Some(Unit::MiB));
    }

    #[test]
    fn test_complex_semantic_evaluation() {
        use crate::units::Unit;

        // Test complex expression: "(5 GiB + 512 MiB) * 2"
        let tokens = vec![
            Token::LeftParen,
            Token::Number(5.0),
            Token::Unit(Unit::GiB),
            Token::Plus,
            Token::Number(512.0),
            Token::Unit(Unit::MiB),
            Token::RightParen,
            Token::Multiply,
            Token::Number(2.0),
        ];
        let semantic = analyze_semantics(&tokens);
        let result = evaluate_semantic_tokens(&semantic, &[], 0);
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert_eq!(unit_val.value, 11264.0); // (5*1024 + 512) * 2 = 5632 * 2 = 11264
        assert_eq!(unit_val.unit, Some(Unit::MiB));

        // Test conversion with complex expression: "(1 GiB + 512 MiB) to KiB"
        let tokens = vec![
            Token::LeftParen,
            Token::Number(1.0),
            Token::Unit(Unit::GiB),
            Token::Plus,
            Token::Number(512.0),
            Token::Unit(Unit::MiB),
            Token::RightParen,
            Token::To,
            Token::Unit(Unit::KiB),
        ];
        let semantic = analyze_semantics(&tokens);
        let result = evaluate_semantic_tokens(&semantic, &[], 0);
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert_eq!(unit_val.value, 1572864.0); // (1*1024 + 512) * 1024 = 1536 * 1024 = 1572864
        assert_eq!(unit_val.unit, Some(Unit::KiB));

        // Test operator precedence: "2 + 3 * 4" (should be 14, not 20)
        let tokens = vec![
            Token::Number(2.0),
            Token::Plus,
            Token::Number(3.0),
            Token::Multiply,
            Token::Number(4.0),
        ];
        let semantic = analyze_semantics(&tokens);
        let result = evaluate_semantic_tokens(&semantic, &[], 0);
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert_eq!(unit_val.value, 14.0); // 2 + (3 * 4) = 2 + 12 = 14
        assert_eq!(unit_val.unit, None);
    }

    #[test]
    fn test_semantic_vs_legacy_evaluation() {
        // Test that semantic approach produces same results as legacy approach
        let test_cases = vec![
            "5 GiB + 512 MiB",
            "1 GiB to MiB",
            "10% of 50",
            "2 * 512 MiB",
            "1024 MiB / 2",
            "(2 GiB + 1024 MiB) * 2",
            "10 + 20",
            "2 + 3 * 4",
        ];

        for test_case in test_cases {
            let legacy_result = evaluate_expression_with_context(test_case, &[], 0);
            let semantic_result = evaluate_expression_with_context_semantic(test_case, &[], 0);

            println!(
                "Testing '{}': legacy={:?}, semantic={:?}",
                test_case, legacy_result, semantic_result
            );

            // Both should either succeed or fail together
            match (legacy_result, semantic_result) {
                (Some(legacy), Some(semantic)) => {
                    // Results should be the same (allowing for minor formatting differences)
                    assert_eq!(
                        legacy, semantic,
                        "Results differ for '{}': legacy='{}', semantic='{}'",
                        test_case, legacy, semantic
                    );
                }
                (None, None) => {
                    // Both failed, which is fine
                }
                (legacy, semantic) => {
                    // One succeeded and one failed - this shouldn't happen for these test cases
                    panic!(
                        "Evaluation mismatch for '{}': legacy={:?}, semantic={:?}",
                        test_case, legacy, semantic
                    );
                }
            }
        }
    }

    #[test]
    fn test_semantic_evaluation_coverage() {
        // Test expressions that might benefit from semantic approach
        let test_cases = vec![
            // Complex nested expressions
            "((1 GiB + 512 MiB) * 2 + 1024 MiB) to KiB",
            // Multiple conversions
            "5 GiB + 512 MiB + 256 KiB",
            // Mixed arithmetic with precedence
            "1 GiB + 2 * 512 MiB - 256 MiB",
        ];

        for test_case in test_cases {
            let semantic_result = evaluate_expression_with_context_semantic(test_case, &[], 0);
            println!(
                "Semantic evaluation of '{}': {:?}",
                test_case, semantic_result
            );

            // These should be evaluated successfully by the semantic approach
            assert!(
                semantic_result.is_some(),
                "Semantic evaluation failed for: '{}'",
                test_case
            );
        }
    }

    #[test]
    fn test_semantic_variable_evaluation() {
        use std::collections::HashMap;
        
        // Set up variables for testing
        let mut variables = HashMap::new();
        variables.insert("ram".to_string(), "16 GiB".to_string());
        variables.insert("speed".to_string(), "100 Mbps".to_string());
        
        // Test variable assignment: "servers = 10"
        let (result, assignment) = evaluate_with_variables_semantic(
            "servers = 10", &variables, &[], 0
        );
        assert!(result.is_some());
        assert!(assignment.is_some());
        let (var_name, var_value) = assignment.unwrap();
        assert_eq!(var_name, "servers");
        assert_eq!(var_value, "10");
        
        // Test variable usage: "ram * 2"
        let (result, assignment) = evaluate_with_variables_semantic(
            "ram * 2", &variables, &[], 0
        );
        assert!(result.is_some());
        assert!(assignment.is_none());
        assert_eq!(result.unwrap(), "32 GiB");
        
        // Test complex variable expression: "ram + 8 GiB"
        let (result, _) = evaluate_with_variables_semantic(
            "ram + 8 GiB", &variables, &[], 0
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "24 GiB");
        
        // Test compatibility with legacy approach
        let legacy_result = evaluate_with_variables("ram * 2", &variables, &[], 0);
        let semantic_result = evaluate_with_variables_semantic("ram * 2", &variables, &[], 0);
        assert_eq!(legacy_result.0, semantic_result.0);
        assert_eq!(legacy_result.1, semantic_result.1);
    }

    #[test]
    fn test_preprocess_tokens_for_evaluation() {
        use crate::units::Unit;

        // Test Number + Unit pairs are converted to NumberWithUnit
        let tokens = vec![
            Token::Number(5.0),
            Token::Unit(Unit::GiB),
            Token::Plus,
            Token::Number(10.0),
            Token::Unit(Unit::MiB),
        ];
        let processed = preprocess_tokens_for_evaluation(&tokens);
        assert_eq!(processed.len(), 3);
        assert!(matches!(
            processed[0],
            Token::NumberWithUnit(5.0, Unit::GiB)
        ));
        assert!(matches!(processed[1], Token::Plus));
        assert!(matches!(
            processed[2],
            Token::NumberWithUnit(10.0, Unit::MiB)
        ));

        // Test standalone Unit tokens become NumberWithUnit(1.0, unit)
        let tokens = vec![Token::Number(1.0), Token::To, Token::Unit(Unit::KiB)];
        let processed = preprocess_tokens_for_evaluation(&tokens);
        assert_eq!(processed.len(), 3);
        assert!(matches!(processed[0], Token::Number(1.0)));
        assert!(matches!(processed[1], Token::To));
        assert!(matches!(
            processed[2],
            Token::NumberWithUnit(1.0, Unit::KiB)
        ));

        // Test mixed tokens pass through correctly
        let tokens = vec![
            Token::Number(42.0),
            Token::Plus,
            Token::Variable("x".to_string()),
            Token::Multiply,
            Token::Number(2.0),
            Token::Unit(Unit::GiB),
        ];
        let processed = preprocess_tokens_for_evaluation(&tokens);
        assert_eq!(processed.len(), 5);
        assert!(matches!(processed[0], Token::Number(42.0)));
        assert!(matches!(processed[1], Token::Plus));
        assert!(matches!(processed[2], Token::Variable(_)));
        assert!(matches!(processed[3], Token::Multiply));
        assert!(matches!(
            processed[4],
            Token::NumberWithUnit(2.0, Unit::GiB)
        ));
    }

    #[test]
    fn test_evaluation_with_separate_tokens() {
        // Test that evaluation works properly with the new separate Number + Unit tokens

        // Test simple arithmetic with units using the new tokenization
        let result = evaluate_expression_with_context("5 GiB + 512 MiB", &[], 0);
        assert!(result.is_some());
        let result_str = result.unwrap();
        // Should be 5,632 MiB (5*1024 + 512 = 5632)
        assert!(result_str.contains("5,632") && result_str.contains("MiB"));

        // Test conversion using new tokenization
        // First, let's see what tokens are being generated
        let tokens = tokenize_with_units("1 GiB to MiB").unwrap();
        println!("DEBUG: Tokens for '1 GiB to MiB': {:?}", tokens);
        let preprocessed = preprocess_tokens_for_evaluation(&tokens);
        println!("DEBUG: Preprocessed tokens: {:?}", preprocessed);

        let result = evaluate_expression_with_context("1 GiB to MiB", &[], 0);
        println!("DEBUG: Result for '1 GiB to MiB': {:?}", result);
        // Let's also test a working example from the old tests
        let result_old = evaluate_expression_with_context("1024 MiB", &[], 0);
        println!("DEBUG: Result for '1024 MiB': {:?}", result_old);

        assert!(result.is_some());
        let result_str = result.unwrap();
        // The actual result might not have comma formatting, let's check both
        assert!(
            (result_str.contains("1024") || result_str.contains("1,024"))
                && result_str.contains("MiB")
        );

        // Test standalone numbers still work
        let result = evaluate_expression_with_context("10 + 20", &[], 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "30");

        // Test complex expressions
        let result = evaluate_expression_with_context("(2 GiB + 1024 MiB) * 2", &[], 0);
        assert!(result.is_some());
        let result_str = result.unwrap();
        // Should be 6,144 MiB (mathematically equivalent to 6 GiB)
        assert!(result_str.contains("6,144") && result_str.contains("MiB"));
    }
}
