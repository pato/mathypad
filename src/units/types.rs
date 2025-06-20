//! Unit type definitions and conversions

use std::borrow::Cow;

use crate::units::{BinaryPrefix, Prefix};

/// Error type for unit conversion operations
#[derive(Debug, Clone, PartialEq)]
pub struct UnitConversionError;

impl std::fmt::Display for UnitConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unit conversion not supported")
    }
}

impl std::error::Error for UnitConversionError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    // Time units (base: seconds)
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,

    // Bit units (base 10)
    Bit,
    Kb, // Kilobit
    Mb, // Megabit
    Gb, // Gigabit
    Tb, // Terabit
    Pb, // Petabit
    Eb, // Exabit

    // Bit units (base 2)
    Kib, // Kibibit
    Mib, // Mebibit
    Gib, // Gibibit
    Tib, // Tebibit
    Pib, // Pebibit
    Eib, // Exbibit

    // Data units (base 10)
    Byte,
    KB, // Kilobyte
    MB, // Megabyte
    GB, // Gigabyte
    TB, // Terabyte
    PB, // Petabyte
    EB, // Exabyte

    // Data units (base 2)
    KiB, // Kibibyte
    MiB, // Mebibyte
    GiB, // Gibibyte
    TiB, // Tebibyte
    PiB, // Pebibyte
    EiB, // Exbibyte

    // Request/Query count (base unit: requests)
    Request,
    Query,

    // Percentage unit (base: decimal value 0.0-1.0)
    Percent,

    //  Generic rates
    RateUnit(Box<Unit>, Box<Unit>),
}

/// Macro to simplify creating RateUnit instances
#[macro_export]
macro_rules! rate_unit {
    ($numerator:expr, $denominator:expr) => {
        Unit::RateUnit(Box::new($numerator), Box::new($denominator))
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitType {
    Time,
    Bit,
    Data,
    Request,
    BitRate,
    DataRate(f64),
    RequestRate,
    Percentage,
}

impl Unit {
    /// Convert a value in this unit to the base unit for its type
    pub fn to_base_value(&self, value: f64) -> f64 {
        match self {
            // Time units (convert to seconds)
            Unit::Nanosecond => value / 1_000_000_000.0,
            Unit::Microsecond => value / 1_000_000.0,
            Unit::Millisecond => value / 1_000.0,
            Unit::Second => value,
            Unit::Minute => value * 60.0,
            Unit::Hour => value * 3600.0,
            Unit::Day => value * 86400.0,

            // Bit units base 10 (convert to bits)
            Unit::Bit => value,
            Unit::Kb => value * 1_000.0,
            Unit::Mb => value * 1_000_000.0,
            Unit::Gb => value * 1_000_000_000.0,
            Unit::Tb => value * 1_000_000_000_000.0,
            Unit::Pb => value * 1_000_000_000_000_000.0,
            Unit::Eb => value * 1_000_000_000_000_000_000.0,

            // Bit units base 2 (convert to bits)
            Unit::Kib => value * 1_024.0,
            Unit::Mib => value * 1_048_576.0,
            Unit::Gib => value * 1_073_741_824.0,
            Unit::Tib => value * 1_099_511_627_776.0,
            Unit::Pib => value * 1_125_899_906_842_624.0,
            Unit::Eib => value * 1_152_921_504_606_846_976.0,

            // Data units base 10 (convert to bytes)
            Unit::Byte => value,
            Unit::KB => value * 1_000.0,
            Unit::MB => value * 1_000_000.0,
            Unit::GB => value * 1_000_000_000.0,
            Unit::TB => value * 1_000_000_000_000.0,
            Unit::PB => value * 1_000_000_000_000_000.0,
            Unit::EB => value * 1_000_000_000_000_000_000.0,

            // Data units base 2 (convert to bytes)
            Unit::KiB => value * 1_024.0,
            Unit::MiB => value * 1_048_576.0,
            Unit::GiB => value * 1_073_741_824.0,
            Unit::TiB => value * 1_099_511_627_776.0,
            Unit::PiB => value * 1_125_899_906_842_624.0,
            Unit::EiB => value * 1_152_921_504_606_846_976.0,

            // Request/Query count (base unit: requests/queries)
            Unit::Request => value,
            Unit::Query => value, // Queries and requests are equivalent

            // Percentage unit (convert to decimal 0.0-1.0)
            Unit::Percent => value / 100.0,

            Unit::RateUnit(v1, v2) => {
                // Convert to base units per second: (data_value * data_base) / (time_value * time_base)
                // where time_base is always in seconds
                let data_base = v1.to_base_value(1.0);
                let time_base = v2.to_base_value(1.0);
                (value * data_base) / time_base
            }
        }
    }

    /// Convert a base value to this unit
    #[allow(clippy::wrong_self_convention)]
    pub fn from_base_value(self, base_value: f64) -> f64 {
        match self {
            // Time units (from seconds)
            Unit::Nanosecond => base_value * 1_000_000_000.0,
            Unit::Microsecond => base_value * 1_000_000.0,
            Unit::Millisecond => base_value * 1_000.0,
            Unit::Second => base_value,
            Unit::Minute => base_value / 60.0,
            Unit::Hour => base_value / 3600.0,
            Unit::Day => base_value / 86400.0,

            // Bit units base 10 (from bits)
            Unit::Bit => base_value,
            Unit::Kb => base_value / 1_000.0,
            Unit::Mb => base_value / 1_000_000.0,
            Unit::Gb => base_value / 1_000_000_000.0,
            Unit::Tb => base_value / 1_000_000_000_000.0,
            Unit::Pb => base_value / 1_000_000_000_000_000.0,
            Unit::Eb => base_value / 1_000_000_000_000_000_000.0,

            // Bit units base 2 (from bits)
            Unit::Kib => base_value / 1_024.0,
            Unit::Mib => base_value / 1_048_576.0,
            Unit::Gib => base_value / 1_073_741_824.0,
            Unit::Tib => base_value / 1_099_511_627_776.0,
            Unit::Pib => base_value / 1_125_899_906_842_624.0,
            Unit::Eib => base_value / 1_152_921_504_606_846_976.0,

            // Data units base 10 (from bytes)
            Unit::Byte => base_value,
            Unit::KB => base_value / 1_000.0,
            Unit::MB => base_value / 1_000_000.0,
            Unit::GB => base_value / 1_000_000_000.0,
            Unit::TB => base_value / 1_000_000_000_000.0,
            Unit::PB => base_value / 1_000_000_000_000_000.0,
            Unit::EB => base_value / 1_000_000_000_000_000_000.0,

            // Data units base 2 (from bytes)
            Unit::KiB => base_value / 1_024.0,
            Unit::MiB => base_value / 1_048_576.0,
            Unit::GiB => base_value / 1_073_741_824.0,
            Unit::TiB => base_value / 1_099_511_627_776.0,
            Unit::PiB => base_value / 1_125_899_906_842_624.0,
            Unit::EiB => base_value / 1_152_921_504_606_846_976.0,

            // Request/Query count (from requests/queries)
            Unit::Request => base_value,
            Unit::Query => base_value,

            // Percentage unit (from decimal 0.0-1.0)
            Unit::Percent => base_value * 100.0,

            // Rate unit
            Unit::RateUnit(v1, v2) => {
                // Convert from base units per second to target rate
                // base_value is in (base_data_units / second)
                // We want (target_data_units / target_time_units)
                let data_base = v1.to_base_value(1.0);
                let time_base = v2.to_base_value(1.0);
                (base_value * time_base) / data_base
            }
        }
    }

    /// Get the unit type for this unit
    pub fn unit_type(&self) -> UnitType {
        match self {
            Unit::Nanosecond
            | Unit::Microsecond
            | Unit::Millisecond
            | Unit::Second
            | Unit::Minute
            | Unit::Hour
            | Unit::Day => UnitType::Time,
            Unit::Bit
            | Unit::Kb
            | Unit::Mb
            | Unit::Gb
            | Unit::Tb
            | Unit::Pb
            | Unit::Eb
            | Unit::Kib
            | Unit::Mib
            | Unit::Gib
            | Unit::Tib
            | Unit::Pib
            | Unit::Eib => UnitType::Bit,
            Unit::Byte
            | Unit::KB
            | Unit::MB
            | Unit::GB
            | Unit::TB
            | Unit::PB
            | Unit::EB
            | Unit::KiB
            | Unit::MiB
            | Unit::GiB
            | Unit::TiB
            | Unit::PiB
            | Unit::EiB => UnitType::Data,
            Unit::Request | Unit::Query => UnitType::Request,
            Unit::Percent => UnitType::Percentage,
            Unit::RateUnit(b1, b2) => {
                if b2.unit_type() != UnitType::Time {
                    panic!("We handle only rates")
                }
                match b1.unit_type() {
                    UnitType::Bit => UnitType::BitRate,
                    UnitType::Data => UnitType::DataRate(b2.to_base_value(1.0)),
                    UnitType::Request => UnitType::RequestRate,
                    _ => panic!("Rate unknown"),
                }
            }
        }
    }

    /// Get the display name for this unit
    pub fn display_name(&self) -> Cow<'static, str> {
        match self {
            Unit::Nanosecond => Cow::Borrowed("ns"),
            Unit::Microsecond => Cow::Borrowed("us"),
            Unit::Millisecond => Cow::Borrowed("ms"),
            Unit::Second => Cow::Borrowed("s"),
            Unit::Minute => Cow::Borrowed("min"),
            Unit::Hour => Cow::Borrowed("h"),
            Unit::Day => Cow::Borrowed("day"),
            Unit::Bit => Cow::Borrowed("bit"),
            Unit::Kb => Cow::Borrowed("Kb"),
            Unit::Mb => Cow::Borrowed("Mb"),
            Unit::Gb => Cow::Borrowed("Gb"),
            Unit::Tb => Cow::Borrowed("Tb"),
            Unit::Pb => Cow::Borrowed("Pb"),
            Unit::Eb => Cow::Borrowed("Eb"),
            Unit::Kib => Cow::Borrowed("Kib"),
            Unit::Mib => Cow::Borrowed("Mib"),
            Unit::Gib => Cow::Borrowed("Gib"),
            Unit::Tib => Cow::Borrowed("Tib"),
            Unit::Pib => Cow::Borrowed("Pib"),
            Unit::Eib => Cow::Borrowed("Eib"),
            Unit::Byte => Cow::Borrowed("B"),
            Unit::KB => Cow::Borrowed("KB"),
            Unit::MB => Cow::Borrowed("MB"),
            Unit::GB => Cow::Borrowed("GB"),
            Unit::TB => Cow::Borrowed("TB"),
            Unit::PB => Cow::Borrowed("PB"),
            Unit::EB => Cow::Borrowed("EB"),
            Unit::KiB => Cow::Borrowed("KiB"),
            Unit::MiB => Cow::Borrowed("MiB"),
            Unit::GiB => Cow::Borrowed("GiB"),
            Unit::TiB => Cow::Borrowed("TiB"),
            Unit::PiB => Cow::Borrowed("PiB"),
            Unit::EiB => Cow::Borrowed("EiB"),
            Unit::Request => Cow::Borrowed("req"),
            Unit::Query => Cow::Borrowed("query"),
            Unit::Percent => Cow::Borrowed("%"),
            Unit::RateUnit(b1, b2) => {
                // Dynamically construct the display name for generic rates (only allocates when needed)
                Cow::Owned(format!("{}/{}", b1.display_name(), b2.display_name()))
            }
        }
    }

    /// Convert a data unit to its corresponding rate unit (per second)
    pub fn to_rate_unit(&self) -> Result<Unit, UnitConversionError> {
        match self {
            // Bit units
            Unit::Bit
            | Unit::Kb
            | Unit::Mb
            | Unit::Gb
            | Unit::Tb
            | Unit::Pb
            | Unit::Eb
            | Unit::Kib
            | Unit::Mib
            | Unit::Gib
            | Unit::Tib
            | Unit::Pib
            | Unit::Eib => Ok(Unit::RateUnit(
                Box::new(self.clone()),
                Box::new(Unit::Second),
            )),
            // Data units
            Unit::Byte
            | Unit::KB
            | Unit::MB
            | Unit::GB
            | Unit::TB
            | Unit::PB
            | Unit::EB
            | Unit::KiB
            | Unit::MiB
            | Unit::GiB
            | Unit::TiB
            | Unit::PiB
            | Unit::EiB => Ok(Unit::RateUnit(
                Box::new(self.clone()),
                Box::new(Unit::Second),
            )),
            // Request/Query units
            Unit::Request | Unit::Query => Ok(Unit::RateUnit(
                Box::new(self.clone()),
                Box::new(Unit::Second),
            )),
            _ => Err(UnitConversionError),
        }
    }

    /// Convert a rate unit to its corresponding data unit
    pub fn to_data_unit(&self) -> Result<Unit, UnitConversionError> {
        match self {
            Unit::RateUnit(b1, _) => Ok(*b1.clone()),
            _ => Err(UnitConversionError),
        }
    }

    /// Convert a request rate unit to its corresponding count unit
    pub fn to_request_unit(&self) -> Result<Unit, UnitConversionError> {
        match self {
            Unit::RateUnit(b1, _) => match b1.as_ref() {
                Unit::Request | Unit::Query => Ok(*b1.clone()),
                _ => Err(UnitConversionError),
            },
            _ => Err(UnitConversionError),
        }
    }

    /// Check if two units are compatible for addition/subtraction
    pub fn is_compatible_for_addition(&self, other: &Unit) -> bool {
        let self_type = self.unit_type();
        let other_type = other.unit_type();

        // Direct unit type match (this covers most cases including exact rate matches)
        if self_type == other_type {
            return true;
        }

        // Special case for rate units with different time units but same data units
        match (self, other) {
            (Unit::RateUnit(self_data, self_time), Unit::RateUnit(other_data, other_time)) => {
                // Both must be time denominators
                if self_time.unit_type() != UnitType::Time
                    || other_time.unit_type() != UnitType::Time
                {
                    return false;
                }

                // For data rates, we need EXACT data unit type compatibility
                // This means GiB (base-2) and MB (base-10) are NOT compatible
                let self_data_type = self_data.unit_type();
                let other_data_type = other_data.unit_type();

                match (self_data_type, other_data_type) {
                    // Bit rates are only compatible with other bit rates
                    (UnitType::Bit, UnitType::Bit) => true,
                    // Request rates are only compatible with other request rates
                    (UnitType::Request, UnitType::Request) => true,
                    // Data rates are compatible only if from same base system
                    (UnitType::Data, UnitType::Data) => {
                        // Check if both are base-2 or both are base-10
                        self_data.is_base2_data() == other_data.is_base2_data()
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Check if this is a base-2 data unit (KiB, MiB, GiB, etc.)
    fn is_base2_data(&self) -> bool {
        matches!(
            self,
            Unit::KiB | Unit::MiB | Unit::GiB | Unit::TiB | Unit::PiB | Unit::EiB
        )
    }

    /// Get the SI prefix info for this unit if applicable
    pub fn prefix_info(&self) -> Option<(Prefix, BaseUnit)> {
        match self {
            // Time units with SI prefixes
            Unit::Nanosecond => Some((Prefix::Nano, BaseUnit::Second)),
            Unit::Microsecond => Some((Prefix::Micro, BaseUnit::Second)),
            Unit::Millisecond => Some((Prefix::Milli, BaseUnit::Second)),

            // Base-10 bit units
            Unit::Kb => Some((Prefix::Kilo, BaseUnit::Bit)),
            Unit::Mb => Some((Prefix::Mega, BaseUnit::Bit)),
            Unit::Gb => Some((Prefix::Giga, BaseUnit::Bit)),
            Unit::Tb => Some((Prefix::Tera, BaseUnit::Bit)),
            Unit::Pb => Some((Prefix::Peta, BaseUnit::Bit)),
            Unit::Eb => Some((Prefix::Exa, BaseUnit::Bit)),

            // Base-10 byte units
            Unit::KB => Some((Prefix::Kilo, BaseUnit::Byte)),
            Unit::MB => Some((Prefix::Mega, BaseUnit::Byte)),
            Unit::GB => Some((Prefix::Giga, BaseUnit::Byte)),
            Unit::TB => Some((Prefix::Tera, BaseUnit::Byte)),
            Unit::PB => Some((Prefix::Peta, BaseUnit::Byte)),
            Unit::EB => Some((Prefix::Exa, BaseUnit::Byte)),

            _ => None,
        }
    }

    /// Get the binary prefix info for this unit if applicable
    pub fn binary_prefix_info(&self) -> Option<(BinaryPrefix, BaseUnit)> {
        match self {
            // Base-2 bit units
            Unit::Kib => Some((BinaryPrefix::Ki, BaseUnit::Bit)),
            Unit::Mib => Some((BinaryPrefix::Mi, BaseUnit::Bit)),
            Unit::Gib => Some((BinaryPrefix::Gi, BaseUnit::Bit)),
            Unit::Tib => Some((BinaryPrefix::Ti, BaseUnit::Bit)),
            Unit::Pib => Some((BinaryPrefix::Pi, BaseUnit::Bit)),
            Unit::Eib => Some((BinaryPrefix::Ei, BaseUnit::Bit)),

            // Base-2 byte units
            Unit::KiB => Some((BinaryPrefix::Ki, BaseUnit::Byte)),
            Unit::MiB => Some((BinaryPrefix::Mi, BaseUnit::Byte)),
            Unit::GiB => Some((BinaryPrefix::Gi, BaseUnit::Byte)),
            Unit::TiB => Some((BinaryPrefix::Ti, BaseUnit::Byte)),
            Unit::PiB => Some((BinaryPrefix::Pi, BaseUnit::Byte)),
            Unit::EiB => Some((BinaryPrefix::Ei, BaseUnit::Byte)),

            _ => None,
        }
    }

    /// Get the conversion factor for this unit using prefix information
    pub fn get_conversion_factor(&self) -> f64 {
        // Try to use prefix-based calculation first
        if let Some((prefix, base)) = self.prefix_info() {
            return prefix.factor() * base.factor();
        }

        if let Some((prefix, base)) = self.binary_prefix_info() {
            return prefix.factor() * base.factor();
        }

        // Fall back to existing to_base_value for non-prefixed units
        self.to_base_value(1.0)
    }
}

/// Base units that can have prefixes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaseUnit {
    Second,
    Bit,
    Byte,
}

impl BaseUnit {
    /// Get the base conversion factor for this unit
    pub fn factor(&self) -> f64 {
        match self {
            BaseUnit::Second => 1.0,
            BaseUnit::Bit => 1.0,
            BaseUnit::Byte => 1.0,
        }
    }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    #[test]
    fn test_prefix_info_si_units() {
        assert_eq!(
            Unit::Nanosecond.prefix_info(),
            Some((Prefix::Nano, BaseUnit::Second))
        );
        assert_eq!(
            Unit::Microsecond.prefix_info(),
            Some((Prefix::Micro, BaseUnit::Second))
        );
        assert_eq!(
            Unit::Millisecond.prefix_info(),
            Some((Prefix::Milli, BaseUnit::Second))
        );

        assert_eq!(Unit::KB.prefix_info(), Some((Prefix::Kilo, BaseUnit::Byte)));
        assert_eq!(Unit::MB.prefix_info(), Some((Prefix::Mega, BaseUnit::Byte)));
        assert_eq!(Unit::GB.prefix_info(), Some((Prefix::Giga, BaseUnit::Byte)));

        assert_eq!(Unit::Kb.prefix_info(), Some((Prefix::Kilo, BaseUnit::Bit)));
        assert_eq!(Unit::Mb.prefix_info(), Some((Prefix::Mega, BaseUnit::Bit)));
        assert_eq!(Unit::Gb.prefix_info(), Some((Prefix::Giga, BaseUnit::Bit)));
    }

    #[test]
    fn test_binary_prefix_info() {
        assert_eq!(
            Unit::KiB.binary_prefix_info(),
            Some((BinaryPrefix::Ki, BaseUnit::Byte))
        );
        assert_eq!(
            Unit::MiB.binary_prefix_info(),
            Some((BinaryPrefix::Mi, BaseUnit::Byte))
        );
        assert_eq!(
            Unit::GiB.binary_prefix_info(),
            Some((BinaryPrefix::Gi, BaseUnit::Byte))
        );

        assert_eq!(
            Unit::Kib.binary_prefix_info(),
            Some((BinaryPrefix::Ki, BaseUnit::Bit))
        );
        assert_eq!(
            Unit::Mib.binary_prefix_info(),
            Some((BinaryPrefix::Mi, BaseUnit::Bit))
        );
        assert_eq!(
            Unit::Gib.binary_prefix_info(),
            Some((BinaryPrefix::Gi, BaseUnit::Bit))
        );
    }

    #[test]
    fn test_get_conversion_factor() {
        // Test SI prefixed units
        assert_eq!(Unit::KB.get_conversion_factor(), 1000.0);
        assert_eq!(Unit::MB.get_conversion_factor(), 1_000_000.0);
        assert_eq!(Unit::Millisecond.get_conversion_factor(), 0.001);

        // Test binary prefixed units
        assert_eq!(Unit::KiB.get_conversion_factor(), 1024.0);
        assert_eq!(Unit::MiB.get_conversion_factor(), 1_048_576.0);

        // Test non-prefixed units (should fall back to existing logic)
        assert_eq!(Unit::Second.get_conversion_factor(), 1.0);
        assert_eq!(Unit::Minute.get_conversion_factor(), 60.0);
        assert_eq!(Unit::Byte.get_conversion_factor(), 1.0);
    }

    #[test]
    fn test_prefix_vs_existing_conversion() {
        // Verify that prefix-based calculation matches existing hardcoded values
        assert_eq!(
            Unit::KB.get_conversion_factor(),
            Unit::KB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::MB.get_conversion_factor(),
            Unit::MB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::GB.get_conversion_factor(),
            Unit::GB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::TB.get_conversion_factor(),
            Unit::TB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::PB.get_conversion_factor(),
            Unit::PB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::EB.get_conversion_factor(),
            Unit::EB.to_base_value(1.0)
        );

        assert_eq!(
            Unit::KiB.get_conversion_factor(),
            Unit::KiB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::MiB.get_conversion_factor(),
            Unit::MiB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::GiB.get_conversion_factor(),
            Unit::GiB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::TiB.get_conversion_factor(),
            Unit::TiB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::PiB.get_conversion_factor(),
            Unit::PiB.to_base_value(1.0)
        );
        assert_eq!(
            Unit::EiB.get_conversion_factor(),
            Unit::EiB.to_base_value(1.0)
        );

        assert_eq!(
            Unit::Millisecond.get_conversion_factor(),
            Unit::Millisecond.to_base_value(1.0)
        );
        assert_eq!(
            Unit::Microsecond.get_conversion_factor(),
            Unit::Microsecond.to_base_value(1.0)
        );
        assert_eq!(
            Unit::Nanosecond.get_conversion_factor(),
            Unit::Nanosecond.to_base_value(1.0)
        );
    }
}
