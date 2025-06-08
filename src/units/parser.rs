//! Unit parsing functionality

use super::types::Unit;

/// Parse a unit string into a Unit enum variant
pub fn parse_unit(text: &str) -> Option<Unit> {
    match text.to_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Some(Unit::Second),
        "min" | "minute" | "minutes" => Some(Unit::Minute),
        "h" | "hr" | "hour" | "hours" => Some(Unit::Hour),
        "day" | "days" => Some(Unit::Day), // Remove single "d" to avoid conflicts

        "b" | "byte" | "bytes" => Some(Unit::Byte),
        "kb" => Some(Unit::KB),
        "mb" => Some(Unit::MB),
        "gb" => Some(Unit::GB),
        "tb" => Some(Unit::TB),
        "pb" => Some(Unit::PB),
        "eb" => Some(Unit::EB),

        "kib" => Some(Unit::KiB),
        "mib" => Some(Unit::MiB),
        "gib" => Some(Unit::GiB),
        "tib" => Some(Unit::TiB),
        "pib" => Some(Unit::PiB),
        "eib" => Some(Unit::EiB),

        "req" | "request" | "requests" => Some(Unit::Request),
        "query" | "queries" => Some(Unit::Query),

        "b/s" | "bytes/s" | "bps" => Some(Unit::BytesPerSecond),
        "kb/s" | "kbps" => Some(Unit::KBPerSecond),
        "mb/s" | "mbps" => Some(Unit::MBPerSecond),
        "gb/s" | "gbps" => Some(Unit::GBPerSecond),
        "tb/s" | "tbps" => Some(Unit::TBPerSecond),
        "pb/s" | "pbps" => Some(Unit::PBPerSecond),
        "eb/s" | "ebps" => Some(Unit::EBPerSecond),
        "kib/s" | "kibps" => Some(Unit::KiBPerSecond),
        "mib/s" | "mibps" => Some(Unit::MiBPerSecond),
        "gib/s" | "gibps" => Some(Unit::GiBPerSecond),
        "tib/s" | "tibps" => Some(Unit::TiBPerSecond),
        "pib/s" | "pibps" => Some(Unit::PiBPerSecond),
        "eib/s" | "eibps" => Some(Unit::EiBPerSecond),

        "req/s" | "requests/s" | "rps" => Some(Unit::RequestsPerSecond),
        "req/min" | "requests/min" | "rpm" => Some(Unit::RequestsPerMinute),
        "req/h" | "req/hour" | "requests/h" | "requests/hour" | "rph" => {
            Some(Unit::RequestsPerHour)
        }
        "qps" | "queries/s" | "queries/sec" => Some(Unit::QueriesPerSecond),
        "qpm" | "queries/min" | "queries/minute" => Some(Unit::QueriesPerMinute),
        "qph" | "queries/h" | "queries/hour" => Some(Unit::QueriesPerHour),

        _ => None,
    }
}