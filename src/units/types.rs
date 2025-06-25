//! Unit type definitions and conversions

use std::borrow::Cow;

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
    Week,
    Month,
    Quarter,
    Year,

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

    // Currency units (no conversion between different currencies)
    USD, // US Dollar
    EUR, // Euro
    GBP, // British Pound Sterling
    JPY, // Japanese Yen
    CNY, // Chinese Yuan
    CAD, // Canadian Dollar
    AUD, // Australian Dollar
    CHF, // Swiss Franc
    INR, // Indian Rupee
    KRW, // South Korean Won

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
    DataRate { time_multiplier: f64 },
    RequestRate,
    Percentage,
    Currency,
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
            Unit::Week => value * 604800.0, // 7 days * 86400 seconds/day
            Unit::Month => value * 2629746.0, // 30.44 days * 86400 seconds/day (average month)
            Unit::Quarter => value * 7889238.0, // 3 months * 2629746 seconds/month
            Unit::Year => value * 31557600.0, // 365.25 days * 86400 seconds/day (accounting for leap years)

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

            // Currency units (no conversion, base value is the same)
            Unit::USD
            | Unit::EUR
            | Unit::GBP
            | Unit::JPY
            | Unit::CNY
            | Unit::CAD
            | Unit::AUD
            | Unit::CHF
            | Unit::INR
            | Unit::KRW => value,

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
            Unit::Week => base_value / 604800.0,
            Unit::Month => base_value / 2629746.0,
            Unit::Quarter => base_value / 7889238.0,
            Unit::Year => base_value / 31557600.0,

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

            // Currency units (no conversion, value is the same)
            Unit::USD
            | Unit::EUR
            | Unit::GBP
            | Unit::JPY
            | Unit::CNY
            | Unit::CAD
            | Unit::AUD
            | Unit::CHF
            | Unit::INR
            | Unit::KRW => base_value,

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
            | Unit::Day
            | Unit::Week
            | Unit::Month
            | Unit::Quarter
            | Unit::Year => UnitType::Time,
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
            Unit::USD
            | Unit::EUR
            | Unit::GBP
            | Unit::JPY
            | Unit::CNY
            | Unit::CAD
            | Unit::AUD
            | Unit::CHF
            | Unit::INR
            | Unit::KRW => UnitType::Currency,
            Unit::RateUnit(b1, b2) => {
                match (b1.unit_type(), b2.unit_type()) {
                    // Traditional rates with time denominators
                    (UnitType::Bit, UnitType::Time) => UnitType::BitRate,
                    (UnitType::Data, UnitType::Time) => UnitType::DataRate {
                        time_multiplier: b2.to_base_value(1.0),
                    },
                    (UnitType::Request, UnitType::Time) => UnitType::RequestRate,
                    (UnitType::Currency, UnitType::Time) => UnitType::DataRate {
                        time_multiplier: b2.to_base_value(1.0),
                    }, // Currency/time rates behave like data rates for arithmetic

                    // Currency rates with data denominators (e.g., $/GiB)
                    (UnitType::Currency, UnitType::Data) => UnitType::DataRate {
                        time_multiplier: 1.0, // No time component for currency/data rates
                    },

                    _ => panic!(
                        "Rate type not supported: {:?}/{:?}",
                        b1.unit_type(),
                        b2.unit_type()
                    ),
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
            Unit::Week => Cow::Borrowed("week"),
            Unit::Month => Cow::Borrowed("month"),
            Unit::Quarter => Cow::Borrowed("quarter"),
            Unit::Year => Cow::Borrowed("year"),
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
            Unit::USD => Cow::Borrowed("$"),
            Unit::EUR => Cow::Borrowed("€"),
            Unit::GBP => Cow::Borrowed("£"),
            Unit::JPY => Cow::Borrowed("¥"),
            Unit::CNY => Cow::Borrowed("¥"),
            Unit::CAD => Cow::Borrowed("C$"),
            Unit::AUD => Cow::Borrowed("A$"),
            Unit::CHF => Cow::Borrowed("CHF"),
            Unit::INR => Cow::Borrowed("₹"),
            Unit::KRW => Cow::Borrowed("₩"),
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

        // For currencies, only allow addition of the exact same currency
        if self_type == UnitType::Currency && other_type == UnitType::Currency {
            return self == other;
        }

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
}
