//! Unit parsing functionality

use super::types::Unit;
use crate::UnitType;
use crate::rate_unit;

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

        // Traditional rate unit patterns - create generic rates
        "bps" | "bit/s" | "bits/s" => {
            return Some(rate_unit!(Unit::Bit, Unit::Second));
        }
        "Kbps" | "Kb/s" => return Some(rate_unit!(Unit::Kb, Unit::Second)),
        "Mbps" | "Mb/s" => return Some(rate_unit!(Unit::Mb, Unit::Second)),
        "Gbps" | "Gb/s" => return Some(rate_unit!(Unit::Gb, Unit::Second)),
        "Tbps" | "Tb/s" => return Some(rate_unit!(Unit::Tb, Unit::Second)),
        "Pbps" | "Pb/s" => return Some(rate_unit!(Unit::Pb, Unit::Second)),
        "Ebps" | "Eb/s" => return Some(rate_unit!(Unit::Eb, Unit::Second)),
        "Kibps" | "Kib/s" => {
            return Some(rate_unit!(Unit::Kib, Unit::Second));
        }
        "Mibps" | "Mib/s" => {
            return Some(rate_unit!(Unit::Mib, Unit::Second));
        }
        "Gibps" | "Gib/s" => {
            return Some(rate_unit!(Unit::Gib, Unit::Second));
        }
        "Tibps" | "Tib/s" => {
            return Some(rate_unit!(Unit::Tib, Unit::Second));
        }
        "Pibps" | "Pib/s" => {
            return Some(rate_unit!(Unit::Pib, Unit::Second));
        }
        "Eibps" | "Eib/s" => {
            return Some(rate_unit!(Unit::Eib, Unit::Second));
        }

        // Byte rates (uppercase 'B/s' for bytes per second)
        "B/s" => return Some(rate_unit!(Unit::Byte, Unit::Second)),
        "KB/s" => return Some(rate_unit!(Unit::KB, Unit::Second)),
        "MB/s" => return Some(rate_unit!(Unit::MB, Unit::Second)),
        "GB/s" => return Some(rate_unit!(Unit::GB, Unit::Second)),
        "TB/s" => return Some(rate_unit!(Unit::TB, Unit::Second)),
        "PB/s" => return Some(rate_unit!(Unit::PB, Unit::Second)),
        "EB/s" => return Some(rate_unit!(Unit::EB, Unit::Second)),
        "KiB/s" => return Some(rate_unit!(Unit::KiB, Unit::Second)),
        "MiB/s" => return Some(rate_unit!(Unit::MiB, Unit::Second)),
        "GiB/s" => return Some(rate_unit!(Unit::GiB, Unit::Second)),
        "TiB/s" => return Some(rate_unit!(Unit::TiB, Unit::Second)),
        "PiB/s" => return Some(rate_unit!(Unit::PiB, Unit::Second)),
        "EiB/s" => return Some(rate_unit!(Unit::EiB, Unit::Second)),

        _ => {} // Fall through to case-insensitive matching
    }

    // Case-insensitive matching for remaining units
    match text.to_lowercase().as_str() {
        "ns" | "nanosec" | "nanosecond" | "nanoseconds" => Some(Unit::Nanosecond),
        "us" | "µs" | "microsec" | "microsecond" | "microseconds" => Some(Unit::Microsecond),
        "ms" | "millisec" | "millisecond" | "milliseconds" => Some(Unit::Millisecond),
        "s" | "sec" | "second" | "seconds" => Some(Unit::Second),
        "min" | "minute" | "minutes" => Some(Unit::Minute),
        "h" | "hr" | "hour" | "hours" => Some(Unit::Hour),
        "day" | "days" => Some(Unit::Day),
        "week" | "weeks" | "wk" | "wks" => Some(Unit::Week),
        "month" | "months" | "mo" | "mos" => Some(Unit::Month),
        "year" | "years" | "yr" | "yrs" => Some(Unit::Year),

        // Case-insensitive parsing (backwards compatibility)
        // For ambiguous lowercase units, follow networking conventions:
        // - Byte units (kb, mb, gb) default to bytes
        // - Bit units (kib, mib, gib when lowercase) default to base 10 bits for simplicity
        "b" | "byte" | "bytes" => Some(Unit::Byte),
        "kb" => Some(Unit::KB), // Kilobytes
        "mb" => Some(Unit::MB), // Megabytes
        "gb" => Some(Unit::GB), // Gigabytes
        "tb" => Some(Unit::TB),
        "pb" => Some(Unit::PB),
        "eb" => Some(Unit::EB),

        // For lowercase "ib" units - network-relevant sizes map to base 10 bits
        // Large units that are rarely used in networking keep traditional binary interpretation
        "kib" => Some(Unit::Kb), // Kilobits (base 10) - commonly used in networking
        "mib" => Some(Unit::Mb), // Megabits (base 10) - commonly used in networking
        "gib" => Some(Unit::Gb), // Gigabits (base 10) - commonly used in networking
        "tib" => Some(Unit::TiB), // Keep as Tebibytes - rarely used in networking
        "pib" => Some(Unit::PiB), // Keep as Pebibytes - rarely used in networking
        "eib" => Some(Unit::EiB), // Keep as Exbibytes - rarely used in networking

        // Case-insensitive rate parsing - create generic rates
        // For "bps" suffix: bits take precedence (network convention)
        // For "/s" suffix: bytes take precedence (file transfer convention)
        "b/s" | "bytes/s" => Some(rate_unit!(Unit::Byte, Unit::Second)),
        "kb/s" => Some(rate_unit!(Unit::KB, Unit::Second)),
        "mb/s" => Some(rate_unit!(Unit::MB, Unit::Second)),
        "gb/s" => Some(rate_unit!(Unit::GB, Unit::Second)),
        "tb/s" => Some(rate_unit!(Unit::TB, Unit::Second)),
        "pb/s" => Some(rate_unit!(Unit::PB, Unit::Second)),
        "eb/s" => Some(rate_unit!(Unit::EB, Unit::Second)),
        "kib/s" => Some(rate_unit!(Unit::KiB, Unit::Second)),
        "mib/s" => Some(rate_unit!(Unit::MiB, Unit::Second)),
        "gib/s" => Some(rate_unit!(Unit::GiB, Unit::Second)),
        "tib/s" => Some(rate_unit!(Unit::TiB, Unit::Second)),
        "pib/s" => Some(rate_unit!(Unit::PiB, Unit::Second)),
        "eib/s" => Some(rate_unit!(Unit::EiB, Unit::Second)),

        // For "bps" suffix: default to bits (network convention)
        // Exception: very large units (PB/EB) default to bytes for backwards compatibility
        "bps" => Some(rate_unit!(Unit::Bit, Unit::Second)),
        "kbps" => Some(rate_unit!(Unit::Kb, Unit::Second)),
        "mbps" => Some(rate_unit!(Unit::Mb, Unit::Second)),
        "gbps" => Some(rate_unit!(Unit::Gb, Unit::Second)),
        "tbps" => Some(rate_unit!(Unit::Tb, Unit::Second)),
        "pbps" => Some(rate_unit!(Unit::PB, Unit::Second)), // Exception: PB default to bytes
        "ebps" => Some(rate_unit!(Unit::EB, Unit::Second)), // Exception: EB default to bytes
        "kibps" => Some(rate_unit!(Unit::Kib, Unit::Second)),
        "mibps" => Some(rate_unit!(Unit::Mib, Unit::Second)),
        "gibps" => Some(rate_unit!(Unit::Gib, Unit::Second)),
        "tibps" => Some(rate_unit!(Unit::Tib, Unit::Second)),
        "pibps" => Some(rate_unit!(Unit::PiB, Unit::Second)), // Exception: PiB default to bytes
        "eibps" => Some(rate_unit!(Unit::EiB, Unit::Second)), // Exception: EiB default to bytes

        "req" | "request" | "requests" => Some(Unit::Request),
        "query" | "queries" => Some(Unit::Query),

        "req/s" | "requests/s" | "rps" => Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Second),
        )),
        "req/min" | "req/minute" | "requests/min" | "requests/minute" | "rpm" => {
            Some(rate_unit!(Unit::Request, Unit::Minute))
        }
        "req/h" | "req/hour" | "requests/h" | "requests/hour" | "rph" => Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Hour),
        )),
        "qps" | "queries/s" | "queries/sec" => Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Second),
        )),
        "qpm" | "queries/min" | "queries/minute" => Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Minute),
        )),
        "qph" | "queries/h" | "queries/hour" => Some(rate_unit!(Unit::Query, Unit::Hour)),

        "%" | "percent" | "percentage" => Some(Unit::Percent),

        // Currency symbols and codes
        "$" | "usd" | "dollar" | "dollars" => Some(Unit::USD),
        "€" | "eur" | "euro" | "euros" => Some(Unit::EUR),
        "£" | "gbp" | "pound" | "pounds" | "sterling" => Some(Unit::GBP),
        "¥" | "jpy" | "yen" => Some(Unit::JPY),
        "cny" | "yuan" | "rmb" => Some(Unit::CNY),
        "c$" | "cad" | "canadian" => Some(Unit::CAD),
        "a$" | "aud" | "australian" => Some(Unit::AUD),
        "chf" | "franc" => Some(Unit::CHF),
        "₹" | "inr" | "rupee" | "rupees" => Some(Unit::INR),
        "₩" | "krw" | "won" => Some(Unit::KRW),

        _ => {
            let mut rate_type = None;
            if let Some(slash_pos) = text.find('/') {
                if slash_pos < text.len() - 1 {
                    let left_unit = parse_unit(&text[0..slash_pos]);
                    let right_unit = parse_unit(&text[slash_pos + 1..]);
                    if let (Some(left_unit), Some(right_unit)) = (left_unit, right_unit) {
                        if right_unit.unit_type() == UnitType::Time {
                            rate_type = Some(rate_unit!(left_unit, right_unit))
                        }
                    }
                }
            }
            rate_type
        }
    }
}
