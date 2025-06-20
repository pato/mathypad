//! Unit parsing functionality

use super::types::Unit;
use super::{BinaryPrefix, Prefix};
use crate::UnitType;
use crate::rate_unit;

/// Try to parse a unit string using SI prefix + base unit pattern
fn try_parse_with_si_prefix(text: &str) -> Option<Unit> {
    // Try to match patterns like "kb", "MB", "Gb", etc.
    for prefix in [
        Prefix::Kilo,
        Prefix::Mega,
        Prefix::Giga,
        Prefix::Tera,
        Prefix::Peta,
        Prefix::Exa,
        Prefix::Milli,
        Prefix::Micro,
        Prefix::Nano,
    ] {
        let prefix_symbols = [prefix.symbol(), prefix.name()];

        for prefix_str in prefix_symbols {
            // Try bit units (lowercase b)
            if text == format!("{}b", prefix_str) || text == format!("{}bit", prefix_str) {
                return match prefix {
                    Prefix::Kilo => Some(Unit::Kb),
                    Prefix::Mega => Some(Unit::Mb),
                    Prefix::Giga => Some(Unit::Gb),
                    Prefix::Tera => Some(Unit::Tb),
                    Prefix::Peta => Some(Unit::Pb),
                    Prefix::Exa => Some(Unit::Eb),
                    Prefix::Milli => Some(Unit::Millisecond),
                    Prefix::Micro => Some(Unit::Microsecond),
                    Prefix::Nano => Some(Unit::Nanosecond),
                    _ => None,
                };
            }

            // Try byte units (uppercase B)
            if text == format!("{}B", prefix_str) || text == format!("{}byte", prefix_str) {
                return match prefix {
                    Prefix::Kilo => Some(Unit::KB),
                    Prefix::Mega => Some(Unit::MB),
                    Prefix::Giga => Some(Unit::GB),
                    Prefix::Tera => Some(Unit::TB),
                    Prefix::Peta => Some(Unit::PB),
                    Prefix::Exa => Some(Unit::EB),
                    _ => None,
                };
            }
        }
    }
    None
}

/// Try to parse a unit string using binary prefix + base unit pattern
fn try_parse_with_binary_prefix(text: &str) -> Option<Unit> {
    // Try to match patterns like "KiB", "Mib", etc.
    for prefix in [
        BinaryPrefix::Ki,
        BinaryPrefix::Mi,
        BinaryPrefix::Gi,
        BinaryPrefix::Ti,
        BinaryPrefix::Pi,
        BinaryPrefix::Ei,
    ] {
        let prefix_str = prefix.symbol();

        // Try bit units (lowercase b)
        if text == format!("{}b", prefix_str) {
            return match prefix {
                BinaryPrefix::Ki => Some(Unit::Kib),
                BinaryPrefix::Mi => Some(Unit::Mib),
                BinaryPrefix::Gi => Some(Unit::Gib),
                BinaryPrefix::Ti => Some(Unit::Tib),
                BinaryPrefix::Pi => Some(Unit::Pib),
                BinaryPrefix::Ei => Some(Unit::Eib),
            };
        }

        // Try byte units (uppercase B)
        if text == format!("{}B", prefix_str) {
            return match prefix {
                BinaryPrefix::Ki => Some(Unit::KiB),
                BinaryPrefix::Mi => Some(Unit::MiB),
                BinaryPrefix::Gi => Some(Unit::GiB),
                BinaryPrefix::Ti => Some(Unit::TiB),
                BinaryPrefix::Pi => Some(Unit::PiB),
                BinaryPrefix::Ei => Some(Unit::EiB),
            };
        }
    }
    None
}

/// Parse a unit string into a Unit enum variant
pub fn parse_unit(text: &str) -> Option<Unit> {
    // Try prefix-based parsing first for common patterns
    if let Some(unit) = try_parse_with_si_prefix(text) {
        return Some(unit);
    }
    if let Some(unit) = try_parse_with_binary_prefix(text) {
        return Some(unit);
    }

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

#[cfg(test)]
mod parser_prefix_tests {
    use super::*;

    #[test]
    fn test_prefix_based_parsing() {
        // Test SI prefix parsing
        assert_eq!(parse_unit("kB"), Some(Unit::KB));
        assert_eq!(parse_unit("MB"), Some(Unit::MB));
        assert_eq!(parse_unit("GB"), Some(Unit::GB));
        assert_eq!(parse_unit("kb"), Some(Unit::Kb));
        assert_eq!(parse_unit("Mb"), Some(Unit::Mb));
        assert_eq!(parse_unit("Gb"), Some(Unit::Gb));

        // Test binary prefix parsing
        assert_eq!(parse_unit("KiB"), Some(Unit::KiB));
        assert_eq!(parse_unit("MiB"), Some(Unit::MiB));
        assert_eq!(parse_unit("GiB"), Some(Unit::GiB));
        assert_eq!(parse_unit("Kib"), Some(Unit::Kib));
        assert_eq!(parse_unit("Mib"), Some(Unit::Mib));
        assert_eq!(parse_unit("Gib"), Some(Unit::Gib));

        // Test time prefix parsing
        assert_eq!(parse_unit("ns"), Some(Unit::Nanosecond));
        assert_eq!(parse_unit("µs"), Some(Unit::Microsecond));
        assert_eq!(parse_unit("ms"), Some(Unit::Millisecond));
    }

    #[test]
    fn test_prefix_parsing_precedence() {
        // Ensure prefix-based parsing works alongside existing explicit patterns
        // These should still work through the existing explicit match patterns
        assert_eq!(parse_unit("KB"), Some(Unit::KB));
        assert_eq!(parse_unit("Kb"), Some(Unit::Kb));
        assert_eq!(parse_unit("KiB"), Some(Unit::KiB));
        assert_eq!(parse_unit("Kib"), Some(Unit::Kib));

        // And that we didn't break any existing functionality
        assert_eq!(parse_unit("bit"), Some(Unit::Bit));
        assert_eq!(parse_unit("byte"), Some(Unit::Byte));
        assert_eq!(parse_unit("s"), Some(Unit::Second));
        assert_eq!(parse_unit("min"), Some(Unit::Minute));
    }
}
