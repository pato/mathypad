//! Unit parsing functionality

use super::types::Unit;

/// Parse a unit string into a Unit enum variant
pub fn parse_unit(text: &str) -> Option<Unit> {
    // First try case-sensitive matching for bits vs bytes disambiguation
    match text {
        // Bit units (lowercase 'b' for bits)
        "bit" | "bits" => return Some(Unit::Bit),
        "Kb" => return Some(Unit::Kb),
        "Mb" => return Some(Unit::Mb),
        "Gb" => return Some(Unit::Gb),
        "Tb" => return Some(Unit::Tb),
        "Pb" => return Some(Unit::Pb),
        "Eb" => return Some(Unit::Eb),
        "Kib" => return Some(Unit::Kib),
        "Mib" => return Some(Unit::Mib),
        "Gib" => return Some(Unit::Gib),
        "Tib" => return Some(Unit::Tib),
        "Pib" => return Some(Unit::Pib),
        "Eib" => return Some(Unit::Eib),

        // Byte units (uppercase 'B' for bytes)
        "B" | "byte" | "bytes" => return Some(Unit::Byte),
        "KB" => return Some(Unit::KB),
        "MB" => return Some(Unit::MB),
        "GB" => return Some(Unit::GB),
        "TB" => return Some(Unit::TB),
        "PB" => return Some(Unit::PB),
        "EB" => return Some(Unit::EB),
        "KiB" => return Some(Unit::KiB),
        "MiB" => return Some(Unit::MiB),
        "GiB" => return Some(Unit::GiB),
        "TiB" => return Some(Unit::TiB),
        "PiB" => return Some(Unit::PiB),
        "EiB" => return Some(Unit::EiB),

        // Bit rates (bits per second)
        "bps" | "bit/s" | "bits/s" => return Some(Unit::BitsPerSecond),
        "Kbps" | "Kb/s" => return Some(Unit::KbPerSecond),
        "Mbps" | "Mb/s" => return Some(Unit::MbPerSecond),
        "Gbps" | "Gb/s" => return Some(Unit::GbPerSecond),
        "Tbps" | "Tb/s" => return Some(Unit::TbPerSecond),
        "Pbps" | "Pb/s" => return Some(Unit::PbPerSecond),
        "Ebps" | "Eb/s" => return Some(Unit::EbPerSecond),
        "Kibps" | "Kib/s" => return Some(Unit::KibPerSecond),
        "Mibps" | "Mib/s" => return Some(Unit::MibPerSecond),
        "Gibps" | "Gib/s" => return Some(Unit::GibPerSecond),
        "Tibps" | "Tib/s" => return Some(Unit::TibPerSecond),
        "Pibps" | "Pib/s" => return Some(Unit::PibPerSecond),
        "Eibps" | "Eib/s" => return Some(Unit::EibPerSecond),

        // Byte rates (uppercase 'B/s' for bytes per second)
        "B/s" => return Some(Unit::BytesPerSecond),
        "KB/s" => return Some(Unit::KBPerSecond),
        "MB/s" => return Some(Unit::MBPerSecond),
        "GB/s" => return Some(Unit::GBPerSecond),
        "TB/s" => return Some(Unit::TBPerSecond),
        "PB/s" => return Some(Unit::PBPerSecond),
        "EB/s" => return Some(Unit::EBPerSecond),
        "KiB/s" => return Some(Unit::KiBPerSecond),
        "MiB/s" => return Some(Unit::MiBPerSecond),
        "GiB/s" => return Some(Unit::GiBPerSecond),
        "TiB/s" => return Some(Unit::TiBPerSecond),
        "PiB/s" => return Some(Unit::PiBPerSecond),
        "EiB/s" => return Some(Unit::EiBPerSecond),

        _ => {} // Fall through to case-insensitive matching
    }

    // Case-insensitive matching for remaining units
    match text.to_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Some(Unit::Second),
        "min" | "minute" | "minutes" => Some(Unit::Minute),
        "h" | "hr" | "hour" | "hours" => Some(Unit::Hour),
        "day" | "days" => Some(Unit::Day),

        // Case-insensitive byte unit parsing (backwards compatibility)
        // For ambiguous units (kb, mb, gb, kib, mib, gib), bytes take precedence in lowercase
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

        // Case-insensitive rate parsing  
        // For "bps" suffix: bits take precedence (network convention)
        // For "/s" suffix: bytes take precedence (file transfer convention)
        "b/s" | "bytes/s" => Some(Unit::BytesPerSecond),
        "kb/s" => Some(Unit::KBPerSecond),
        "mb/s" => Some(Unit::MBPerSecond),
        "gb/s" => Some(Unit::GBPerSecond),
        "tb/s" => Some(Unit::TBPerSecond),
        "pb/s" => Some(Unit::PBPerSecond),
        "eb/s" => Some(Unit::EBPerSecond),
        "kib/s" => Some(Unit::KiBPerSecond),
        "mib/s" => Some(Unit::MiBPerSecond),
        "gib/s" => Some(Unit::GiBPerSecond),
        "tib/s" => Some(Unit::TiBPerSecond),
        "pib/s" => Some(Unit::PiBPerSecond),
        "eib/s" => Some(Unit::EiBPerSecond),
        
        // For "bps" suffix: default to bits (network convention)
        // Exception: very large units (PB/EB) default to bytes for backwards compatibility
        "bps" => Some(Unit::BitsPerSecond),
        "kbps" => Some(Unit::KbPerSecond),
        "mbps" => Some(Unit::MbPerSecond),
        "gbps" => Some(Unit::GbPerSecond),
        "tbps" => Some(Unit::TbPerSecond),
        "pbps" => Some(Unit::PBPerSecond), // Exception: PB default to bytes
        "ebps" => Some(Unit::EBPerSecond), // Exception: EB default to bytes
        "kibps" => Some(Unit::KibPerSecond),
        "mibps" => Some(Unit::MibPerSecond),
        "gibps" => Some(Unit::GibPerSecond),
        "tibps" => Some(Unit::TibPerSecond),
        "pibps" => Some(Unit::PiBPerSecond), // Exception: PiB default to bytes
        "eibps" => Some(Unit::EiBPerSecond), // Exception: EiB default to bytes

        "req" | "request" | "requests" => Some(Unit::Request),
        "query" | "queries" => Some(Unit::Query),

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
