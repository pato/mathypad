//! Unit value representation and operations

use super::types::{Unit, UnitType};
use crate::{FLOAT_EPSILON, MAX_INTEGER_FOR_FORMATTING};

/// Represents a numeric value with an optional unit
#[derive(Debug, Clone)]
pub struct UnitValue {
    pub value: f64,
    pub unit: Option<Unit>,
}

impl UnitValue {
    /// Create a new UnitValue
    pub fn new(value: f64, unit: Option<Unit>) -> Self {
        UnitValue { value, unit }
    }

    /// Convert this value to a different unit of the same type
    pub fn to_unit(&self, target_unit: &Unit) -> Option<UnitValue> {
        match &self.unit {
            Some(current_unit) => {
                // Special handling for rate unit conversions first
                if let (
                    Unit::RateUnit(curr_num, curr_denom),
                    Unit::RateUnit(targ_num, targ_denom),
                ) = (current_unit, target_unit)
                {
                    // Check if numerators are compatible (same currency/unit type)
                    if curr_num.unit_type() == targ_num.unit_type() && curr_num == targ_num {
                        // Convert the denominator to match the target
                        let curr_denom_base = curr_denom.to_base_value(1.0);
                        let targ_denom_base = targ_denom.to_base_value(1.0);

                        // Rate conversion: if we have $/month and want $/year,
                        // we need to multiply by how many current periods fit in target period
                        // For $/month to $/year: $5/month * 12 months/year = $60/year
                        let conversion_factor = targ_denom_base / curr_denom_base;
                        let converted_value = self.value * conversion_factor;

                        return Some(UnitValue::new(converted_value, Some(target_unit.clone())));
                    } else if curr_num.unit_type() == UnitType::Currency
                        && targ_num.unit_type() == UnitType::Currency
                    {
                        // Different currencies - not convertible
                        return None;
                    }
                }

                // Check if units are the same type or compatible data rates
                if current_unit.unit_type() == target_unit.unit_type()
                    || self.can_convert_between_data_rates(current_unit, target_unit)
                {
                    let base_value = current_unit.to_base_value(self.value);
                    let converted_value = target_unit.clone().from_base_value(base_value);
                    Some(UnitValue::new(converted_value, Some(target_unit.clone())))
                }
                // Check for special conversions between bits and bytes
                else if self.can_convert_between_bits_bytes(current_unit, target_unit) {
                    self.convert_bits_bytes(current_unit, target_unit)
                } else {
                    None // Can't convert between different unit types
                }
            }
            None => {
                // Handle conversion from dimensionless value to percentage
                use super::types::UnitType;
                if target_unit.unit_type() == UnitType::Percentage {
                    let converted_value = target_unit.clone().from_base_value(self.value);
                    Some(UnitValue::new(converted_value, Some(target_unit.clone())))
                } else {
                    None // No unit to convert from
                }
            }
        }
    }

    /// Check if conversion between data rates with different time units is possible
    fn can_convert_between_data_rates(&self, current: &Unit, target: &Unit) -> bool {
        use super::types::UnitType;
        matches!(
            (current.unit_type(), target.unit_type()),
            (UnitType::DataRate { .. }, UnitType::DataRate { .. })
        )
    }

    /// Check if conversion between bits and bytes is possible
    fn can_convert_between_bits_bytes(&self, current: &Unit, target: &Unit) -> bool {
        use super::types::UnitType;
        matches!(
            (current.unit_type(), target.unit_type()),
            (UnitType::Bit, UnitType::Data)
                | (UnitType::Data, UnitType::Bit)
                | (UnitType::BitRate, UnitType::DataRate { .. })
                | (UnitType::DataRate { .. }, UnitType::BitRate)
        )
    }

    /// Convert between bits and bytes (8 bits = 1 byte)
    fn convert_bits_bytes(&self, current: &Unit, target: &Unit) -> Option<UnitValue> {
        use super::types::UnitType;

        match (current.unit_type(), target.unit_type()) {
            // Bit to Byte conversion
            (UnitType::Bit, UnitType::Data) => {
                let bits = current.to_base_value(self.value); // Convert to base bits
                let bytes = bits / 8.0; // 8 bits = 1 byte
                let converted_value = target.clone().from_base_value(bytes);
                Some(UnitValue::new(converted_value, Some(target.clone())))
            }
            // Byte to Bit conversion
            (UnitType::Data, UnitType::Bit) => {
                let bytes = current.to_base_value(self.value); // Convert to base bytes
                let bits = bytes * 8.0; // 1 byte = 8 bits
                let converted_value = target.clone().from_base_value(bits);
                Some(UnitValue::new(converted_value, Some(target.clone())))
            }
            // Bit rate to Byte rate conversion
            (UnitType::BitRate, UnitType::DataRate { .. }) => {
                let bits_per_sec = current.to_base_value(self.value); // Convert to base bits/sec
                let bytes_per_sec = bits_per_sec / 8.0; // 8 bits/sec = 1 byte/sec
                let converted_value = target.clone().from_base_value(bytes_per_sec);
                Some(UnitValue::new(converted_value, Some(target.clone())))
            }
            // Byte rate to Bit rate conversion
            (UnitType::DataRate { .. }, UnitType::BitRate) => {
                let bytes_per_sec = current.to_base_value(self.value); // Convert to base bytes/sec
                let bits_per_sec = bytes_per_sec * 8.0; // 1 byte/sec = 8 bits/sec
                let converted_value = target.clone().from_base_value(bits_per_sec);
                Some(UnitValue::new(converted_value, Some(target.clone())))
            }
            _ => None,
        }
    }

    /// Format the value for display
    pub fn format(&self) -> String {
        let formatted_value =
            if self.value.fract() == 0.0 && self.value.abs() < MAX_INTEGER_FOR_FORMATTING {
                format_number_with_commas(self.value as i64)
            } else {
                format_decimal_with_commas(self.value)
            };

        match &self.unit {
            Some(unit) => format!("{} {}", formatted_value, unit.display_name()),
            None => formatted_value,
        }
    }
}

/// Format a number with comma separators
fn format_number_with_commas(num: i64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().collect();

    let is_negative = chars.first() == Some(&'-');
    let start_idx = if is_negative { 1 } else { 0 };

    if is_negative {
        result.push('-');
    }

    for (i, ch) in chars[start_idx..].iter().enumerate() {
        if i > 0 && (chars.len() - start_idx - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    result
}

/// Format a decimal number with comma separators (for whole part)
fn format_decimal_with_commas(num: f64) -> String {
    if num.abs() < FLOAT_EPSILON {
        return "0".to_string();
    }

    let is_negative = num < 0.0;
    let abs_num = num.abs();

    let formatted = format!("{:.3}", abs_num);

    // Split into whole and decimal parts
    let parts: Vec<&str> = formatted.split('.').collect();
    if parts.len() != 2 {
        return if is_negative {
            format!("-{}", formatted)
        } else {
            formatted
        };
    }

    let whole_part = parts[0];
    let decimal_part = parts[1];

    // Add commas to whole part
    let whole_with_commas = if whole_part == "0" {
        "0".to_string()
    } else {
        let whole_chars: Vec<char> = whole_part.chars().collect();
        let mut result = String::new();

        for (i, ch) in whole_chars.iter().enumerate() {
            if i > 0 && (whole_chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(*ch);
        }
        result
    };

    // Remove trailing zeros from decimal part
    let decimal_trimmed = decimal_part.trim_end_matches('0');

    let formatted_result = if decimal_trimmed.is_empty() {
        whole_with_commas
    } else {
        format!("{}.{}", whole_with_commas, decimal_trimmed)
    };

    if is_negative {
        format!("-{}", formatted_result)
    } else {
        formatted_result
    }
}
