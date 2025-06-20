//! Unit type definitions and conversions

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

    // Bit rate units
    BitsPerSecond,
    KbPerSecond,
    MbPerSecond,
    GbPerSecond,
    TbPerSecond,
    PbPerSecond,
    EbPerSecond,
    KibPerSecond,
    MibPerSecond,
    GibPerSecond,
    TibPerSecond,
    PibPerSecond,
    EibPerSecond,

    // Data rate units
    BytesPerSecond,
    KBPerSecond,
    MBPerSecond,
    GBPerSecond,
    TBPerSecond,
    PBPerSecond,
    EBPerSecond,
    KiBPerSecond,
    MiBPerSecond,
    GiBPerSecond,
    TiBPerSecond,
    PiBPerSecond,
    EiBPerSecond,

    // Request/Query rate units (base: requests per second)
    RequestsPerSecond,
    RequestsPerMinute,
    RequestsPerHour,
    QueriesPerSecond,
    QueriesPerMinute,
    QueriesPerHour,

    // Percentage unit (base: decimal value 0.0-1.0)
    Percent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitType {
    Time,
    Bit,
    Data,
    Request,
    BitRate,
    DataRate,
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

            // Bit rate units (convert to bits per second)
            Unit::BitsPerSecond => value,
            Unit::KbPerSecond => value * 1_000.0,
            Unit::MbPerSecond => value * 1_000_000.0,
            Unit::GbPerSecond => value * 1_000_000_000.0,
            Unit::TbPerSecond => value * 1_000_000_000_000.0,
            Unit::PbPerSecond => value * 1_000_000_000_000_000.0,
            Unit::EbPerSecond => value * 1_000_000_000_000_000_000.0,
            Unit::KibPerSecond => value * 1_024.0,
            Unit::MibPerSecond => value * 1_048_576.0,
            Unit::GibPerSecond => value * 1_073_741_824.0,
            Unit::TibPerSecond => value * 1_099_511_627_776.0,
            Unit::PibPerSecond => value * 1_125_899_906_842_624.0,
            Unit::EibPerSecond => value * 1_152_921_504_606_846_976.0,

            // Data rate units (convert to bytes per second)
            Unit::BytesPerSecond => value,
            Unit::KBPerSecond => value * 1_000.0,
            Unit::MBPerSecond => value * 1_000_000.0,
            Unit::GBPerSecond => value * 1_000_000_000.0,
            Unit::TBPerSecond => value * 1_000_000_000_000.0,
            Unit::PBPerSecond => value * 1_000_000_000_000_000.0,
            Unit::EBPerSecond => value * 1_000_000_000_000_000_000.0,
            Unit::KiBPerSecond => value * 1_024.0,
            Unit::MiBPerSecond => value * 1_048_576.0,
            Unit::GiBPerSecond => value * 1_073_741_824.0,
            Unit::TiBPerSecond => value * 1_099_511_627_776.0,
            Unit::PiBPerSecond => value * 1_125_899_906_842_624.0,
            Unit::EiBPerSecond => value * 1_152_921_504_606_846_976.0,

            // Request/Query rate units (convert to requests per second)
            Unit::RequestsPerSecond => value,
            Unit::RequestsPerMinute => value / 60.0,
            Unit::RequestsPerHour => value / 3600.0,
            Unit::QueriesPerSecond => value, // QPS = requests per second
            Unit::QueriesPerMinute => value / 60.0,
            Unit::QueriesPerHour => value / 3600.0,

            // Percentage unit (convert to decimal 0.0-1.0)
            Unit::Percent => value / 100.0,
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

            // Bit rate units (from bits per second)
            Unit::BitsPerSecond => base_value,
            Unit::KbPerSecond => base_value / 1_000.0,
            Unit::MbPerSecond => base_value / 1_000_000.0,
            Unit::GbPerSecond => base_value / 1_000_000_000.0,
            Unit::TbPerSecond => base_value / 1_000_000_000_000.0,
            Unit::PbPerSecond => base_value / 1_000_000_000_000_000.0,
            Unit::EbPerSecond => base_value / 1_000_000_000_000_000_000.0,
            Unit::KibPerSecond => base_value / 1_024.0,
            Unit::MibPerSecond => base_value / 1_048_576.0,
            Unit::GibPerSecond => base_value / 1_073_741_824.0,
            Unit::TibPerSecond => base_value / 1_099_511_627_776.0,
            Unit::PibPerSecond => base_value / 1_125_899_906_842_624.0,
            Unit::EibPerSecond => base_value / 1_152_921_504_606_846_976.0,

            // Data rate units (from bytes per second)
            Unit::BytesPerSecond => base_value,
            Unit::KBPerSecond => base_value / 1_000.0,
            Unit::MBPerSecond => base_value / 1_000_000.0,
            Unit::GBPerSecond => base_value / 1_000_000_000.0,
            Unit::TBPerSecond => base_value / 1_000_000_000_000.0,
            Unit::PBPerSecond => base_value / 1_000_000_000_000_000.0,
            Unit::EBPerSecond => base_value / 1_000_000_000_000_000_000.0,
            Unit::KiBPerSecond => base_value / 1_024.0,
            Unit::MiBPerSecond => base_value / 1_048_576.0,
            Unit::GiBPerSecond => base_value / 1_073_741_824.0,
            Unit::TiBPerSecond => base_value / 1_099_511_627_776.0,
            Unit::PiBPerSecond => base_value / 1_125_899_906_842_624.0,
            Unit::EiBPerSecond => base_value / 1_152_921_504_606_846_976.0,

            // Request/Query rate units (from requests per second)
            Unit::RequestsPerSecond => base_value,
            Unit::RequestsPerMinute => base_value * 60.0,
            Unit::RequestsPerHour => base_value * 3600.0,
            Unit::QueriesPerSecond => base_value,
            Unit::QueriesPerMinute => base_value * 60.0,
            Unit::QueriesPerHour => base_value * 3600.0,

            // Percentage unit (from decimal 0.0-1.0)
            Unit::Percent => base_value * 100.0,
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
            Unit::BitsPerSecond
            | Unit::KbPerSecond
            | Unit::MbPerSecond
            | Unit::GbPerSecond
            | Unit::TbPerSecond
            | Unit::PbPerSecond
            | Unit::EbPerSecond
            | Unit::KibPerSecond
            | Unit::MibPerSecond
            | Unit::GibPerSecond
            | Unit::TibPerSecond
            | Unit::PibPerSecond
            | Unit::EibPerSecond => UnitType::BitRate,
            Unit::BytesPerSecond
            | Unit::KBPerSecond
            | Unit::MBPerSecond
            | Unit::GBPerSecond
            | Unit::TBPerSecond
            | Unit::PBPerSecond
            | Unit::EBPerSecond
            | Unit::KiBPerSecond
            | Unit::MiBPerSecond
            | Unit::GiBPerSecond
            | Unit::TiBPerSecond
            | Unit::PiBPerSecond
            | Unit::EiBPerSecond => UnitType::DataRate,
            Unit::RequestsPerSecond
            | Unit::RequestsPerMinute
            | Unit::RequestsPerHour
            | Unit::QueriesPerSecond
            | Unit::QueriesPerMinute
            | Unit::QueriesPerHour => UnitType::RequestRate,
            Unit::Percent => UnitType::Percentage,
        }
    }

    /// Get the display name for this unit
    pub fn display_name(&self) -> &'static str {
        match self {
            Unit::Nanosecond => "ns",
            Unit::Microsecond => "us",
            Unit::Millisecond => "ms",
            Unit::Second => "s",
            Unit::Minute => "min",
            Unit::Hour => "h",
            Unit::Day => "day",
            Unit::Bit => "bit",
            Unit::Kb => "Kb",
            Unit::Mb => "Mb",
            Unit::Gb => "Gb",
            Unit::Tb => "Tb",
            Unit::Pb => "Pb",
            Unit::Eb => "Eb",
            Unit::Kib => "Kib",
            Unit::Mib => "Mib",
            Unit::Gib => "Gib",
            Unit::Tib => "Tib",
            Unit::Pib => "Pib",
            Unit::Eib => "Eib",
            Unit::Byte => "B",
            Unit::KB => "KB",
            Unit::MB => "MB",
            Unit::GB => "GB",
            Unit::TB => "TB",
            Unit::PB => "PB",
            Unit::EB => "EB",
            Unit::KiB => "KiB",
            Unit::MiB => "MiB",
            Unit::GiB => "GiB",
            Unit::TiB => "TiB",
            Unit::PiB => "PiB",
            Unit::EiB => "EiB",
            Unit::Request => "req",
            Unit::Query => "query",
            Unit::BitsPerSecond => "bps",
            Unit::KbPerSecond => "Kbps",
            Unit::MbPerSecond => "Mbps",
            Unit::GbPerSecond => "Gbps",
            Unit::TbPerSecond => "Tbps",
            Unit::PbPerSecond => "Pbps",
            Unit::EbPerSecond => "Ebps",
            Unit::KibPerSecond => "Kibps",
            Unit::MibPerSecond => "Mibps",
            Unit::GibPerSecond => "Gibps",
            Unit::TibPerSecond => "Tibps",
            Unit::PibPerSecond => "Pibps",
            Unit::EibPerSecond => "Eibps",
            Unit::BytesPerSecond => "B/s",
            Unit::KBPerSecond => "KB/s",
            Unit::MBPerSecond => "MB/s",
            Unit::GBPerSecond => "GB/s",
            Unit::TBPerSecond => "TB/s",
            Unit::PBPerSecond => "PB/s",
            Unit::EBPerSecond => "EB/s",
            Unit::KiBPerSecond => "KiB/s",
            Unit::MiBPerSecond => "MiB/s",
            Unit::GiBPerSecond => "GiB/s",
            Unit::TiBPerSecond => "TiB/s",
            Unit::PiBPerSecond => "PiB/s",
            Unit::EiBPerSecond => "EiB/s",
            Unit::RequestsPerSecond => "req/s",
            Unit::RequestsPerMinute => "req/min",
            Unit::RequestsPerHour => "req/h",
            Unit::QueriesPerSecond => "QPS",
            Unit::QueriesPerMinute => "QPM",
            Unit::QueriesPerHour => "QPH",
            Unit::Percent => "%",
        }
    }

    /// Convert a data unit to its corresponding rate unit
    pub fn to_rate_unit(&self) -> Result<Unit, UnitConversionError> {
        match self {
            Unit::Bit => Ok(Unit::BitsPerSecond),
            Unit::Kb => Ok(Unit::KbPerSecond),
            Unit::Mb => Ok(Unit::MbPerSecond),
            Unit::Gb => Ok(Unit::GbPerSecond),
            Unit::Tb => Ok(Unit::TbPerSecond),
            Unit::Pb => Ok(Unit::PbPerSecond),
            Unit::Eb => Ok(Unit::EbPerSecond),
            Unit::Kib => Ok(Unit::KibPerSecond),
            Unit::Mib => Ok(Unit::MibPerSecond),
            Unit::Gib => Ok(Unit::GibPerSecond),
            Unit::Tib => Ok(Unit::TibPerSecond),
            Unit::Pib => Ok(Unit::PibPerSecond),
            Unit::Eib => Ok(Unit::EibPerSecond),
            Unit::Byte => Ok(Unit::BytesPerSecond),
            Unit::KB => Ok(Unit::KBPerSecond),
            Unit::MB => Ok(Unit::MBPerSecond),
            Unit::GB => Ok(Unit::GBPerSecond),
            Unit::TB => Ok(Unit::TBPerSecond),
            Unit::PB => Ok(Unit::PBPerSecond),
            Unit::EB => Ok(Unit::EBPerSecond),
            Unit::KiB => Ok(Unit::KiBPerSecond),
            Unit::MiB => Ok(Unit::MiBPerSecond),
            Unit::GiB => Ok(Unit::GiBPerSecond),
            Unit::TiB => Ok(Unit::TiBPerSecond),
            Unit::PiB => Ok(Unit::PiBPerSecond),
            Unit::EiB => Ok(Unit::EiBPerSecond),
            Unit::Request => Ok(Unit::RequestsPerSecond),
            Unit::Query => Ok(Unit::QueriesPerSecond),
            _ => Err(UnitConversionError),
        }
    }

    /// Convert a rate unit to its corresponding data unit
    pub fn to_data_unit(&self) -> Result<Unit, UnitConversionError> {
        match self {
            Unit::BitsPerSecond => Ok(Unit::Bit),
            Unit::KbPerSecond => Ok(Unit::Kb),
            Unit::MbPerSecond => Ok(Unit::Mb),
            Unit::GbPerSecond => Ok(Unit::Gb),
            Unit::TbPerSecond => Ok(Unit::Tb),
            Unit::PbPerSecond => Ok(Unit::Pb),
            Unit::EbPerSecond => Ok(Unit::Eb),
            Unit::KibPerSecond => Ok(Unit::Kib),
            Unit::MibPerSecond => Ok(Unit::Mib),
            Unit::GibPerSecond => Ok(Unit::Gib),
            Unit::TibPerSecond => Ok(Unit::Tib),
            Unit::PibPerSecond => Ok(Unit::Pib),
            Unit::EibPerSecond => Ok(Unit::Eib),
            Unit::BytesPerSecond => Ok(Unit::Byte),
            Unit::KBPerSecond => Ok(Unit::KB),
            Unit::MBPerSecond => Ok(Unit::MB),
            Unit::GBPerSecond => Ok(Unit::GB),
            Unit::TBPerSecond => Ok(Unit::TB),
            Unit::PBPerSecond => Ok(Unit::PB),
            Unit::EBPerSecond => Ok(Unit::EB),
            Unit::KiBPerSecond => Ok(Unit::KiB),
            Unit::MiBPerSecond => Ok(Unit::MiB),
            Unit::GiBPerSecond => Ok(Unit::GiB),
            Unit::TiBPerSecond => Ok(Unit::TiB),
            Unit::PiBPerSecond => Ok(Unit::PiB),
            Unit::EiBPerSecond => Ok(Unit::EiB),
            _ => Err(UnitConversionError),
        }
    }

    /// Convert a request rate unit to its corresponding count unit
    pub fn to_request_unit(&self) -> Result<Unit, UnitConversionError> {
        match self {
            Unit::RequestsPerSecond => Ok(Unit::Request),
            Unit::RequestsPerMinute => Ok(Unit::Request),
            Unit::RequestsPerHour => Ok(Unit::Request),
            Unit::QueriesPerSecond => Ok(Unit::Query),
            Unit::QueriesPerMinute => Ok(Unit::Query),
            Unit::QueriesPerHour => Ok(Unit::Query),
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
                if self_time.unit_type() != UnitType::Time || other_time.unit_type() != UnitType::Time {
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
        matches!(self, Unit::KiB | Unit::MiB | Unit::GiB | Unit::TiB | Unit::PiB | Unit::EiB)
    }
}
