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
    Second,
    Minute,
    Hour,
    Day,

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitType {
    Time,
    Data,
    Request,
    DataRate,
    RequestRate,
}

impl Unit {
    /// Convert a value in this unit to the base unit for its type
    pub fn to_base_value(&self, value: f64) -> f64 {
        match self {
            // Time units (convert to seconds)
            Unit::Second => value,
            Unit::Minute => value * 60.0,
            Unit::Hour => value * 3600.0,
            Unit::Day => value * 86400.0,

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
        }
    }

    /// Convert a base value to this unit
    #[allow(clippy::wrong_self_convention)]
    pub fn from_base_value(self, base_value: f64) -> f64 {
        match self {
            // Time units (from seconds)
            Unit::Second => base_value,
            Unit::Minute => base_value / 60.0,
            Unit::Hour => base_value / 3600.0,
            Unit::Day => base_value / 86400.0,

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
        }
    }

    /// Get the unit type for this unit
    pub fn unit_type(&self) -> UnitType {
        match self {
            Unit::Second | Unit::Minute | Unit::Hour | Unit::Day => UnitType::Time,
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
        }
    }

    /// Get the display name for this unit
    pub fn display_name(&self) -> &'static str {
        match self {
            Unit::Second => "s",
            Unit::Minute => "min",
            Unit::Hour => "h",
            Unit::Day => "day",
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
        }
    }

    /// Convert a data unit to its corresponding rate unit
    pub fn to_rate_unit(&self) -> Result<Unit, UnitConversionError> {
        match self {
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
}
