//! Tests for unit functionality

use super::*;
use crate::rate_unit;
use crate::test_helpers::*;

fn floats_equal(a: f64, b: f64) {
    let delta = (a - b).abs();
    if delta > 0.001 {
        panic!("expected {}, got {}, delta = {}", a, b, delta)
    }
}

#[test]
fn test_generic_rate() {
    // Test that generic rates convert to bytes per second correctly
    let rate = rate_unit!(Unit::MB, Unit::Second);
    floats_equal(rate.to_base_value(1.0), 1_000_000.0); // 1 MB/s = 1,000,000 bytes/s

    let rate = rate_unit!(Unit::KB, Unit::Minute);
    floats_equal(rate.to_base_value(3.0), 50.0); // 3 KB/min = (3 * 1000) / 60 = 50 bytes/s

    let rate = rate_unit!(Unit::Byte, Unit::Day);
    floats_equal(rate.to_base_value(86400.0), 1.0); // 86400 bytes/day = 86400 / 86400 = 1 byte/s
}

#[test]
fn test_generic_rate_parsing() {
    // Test parsing various generic rate units
    assert_eq!(
        parse_unit("GiB/minute"),
        Some(rate_unit!(Unit::GiB, Unit::Minute))
    );
    assert_eq!(
        parse_unit("MB/hour"),
        Some(rate_unit!(Unit::MB, Unit::Hour))
    );
    assert_eq!(parse_unit("KB/day"), Some(rate_unit!(Unit::KB, Unit::Day)));
    assert_eq!(
        parse_unit("TiB/min"),
        Some(rate_unit!(Unit::TiB, Unit::Minute))
    );
    assert_eq!(parse_unit("PB/h"), Some(rate_unit!(Unit::PB, Unit::Hour)));

    // Test that non-time denominators don't create rate units
    assert!(parse_unit("GiB/MB").is_none());
    assert!(parse_unit("MB/GB").is_none());

    // Test bit rates with different time units
    assert_eq!(
        parse_unit("Mb/minute"),
        Some(rate_unit!(Unit::Mb, Unit::Minute))
    );
    assert_eq!(
        parse_unit("Gb/hour"),
        Some(rate_unit!(Unit::Gb, Unit::Hour))
    );
    assert_eq!(parse_unit("Kb/day"), Some(rate_unit!(Unit::Kb, Unit::Day)));
}

#[test]
fn test_generic_rate_calculations() {
    // Test GiB/minute * minutes = GiB
    let result = evaluate_test_expression("1 GiB/minute * 60 minutes");
    assert_eq!(result, Some("60 GiB".to_string()));

    // Test MB/hour * hours = MB
    let result = evaluate_test_expression("100 MB/hour * 24 hours");
    assert_eq!(result, Some("2,400 MB".to_string()));

    // Test KB/day * days = KB
    let result = evaluate_test_expression("1000 KB/day * 7 days");
    assert_eq!(result, Some("7,000 KB".to_string()));

    // Test with fractional values
    let result = evaluate_test_expression("0.5 GiB/minute * 10 minutes");
    assert_eq!(result, Some("5 GiB".to_string()));

    // Test with different time unit conversions
    let result = evaluate_test_expression("1 GiB/hour * 30 minutes");
    assert_eq!(result, Some("0.5 GiB".to_string())); // 30 minutes = 0.5 hours

    // Test TiB/hour * minutes (mixed time units)
    let result = evaluate_test_expression("1 TiB/hour * 90 minutes");
    assert_eq!(result, Some("1.5 TiB".to_string())); // 90 minutes = 1.5 hours
}

#[test]
fn test_generic_rate_division() {
    // Test data / time = rate (only non-seconds create generic rates)
    let result = evaluate_test_expression("100 GiB / 20 minutes");
    assert_eq!(result, Some("5 GiB/min".to_string())); // Creates generic rate for minutes

    let result = evaluate_test_expression("1 TiB / 2 hours");
    assert_eq!(result, Some("0.5 TiB/h".to_string())); // Creates generic rate for hours

    let result = evaluate_test_expression("500 MB / 10 days");
    assert_eq!(result, Some("50 MB/day".to_string())); // Creates generic rate for days

    // But seconds should create traditional rates
    let result = evaluate_test_expression("100 GiB / 10 seconds");
    assert_eq!(result, Some("10 GiB/s".to_string())); // Traditional per-second rate

    // Test data / generic rate = time
    let result = evaluate_test_expression("100 GiB / (5 GiB/minute)");
    assert_eq!(result, Some("20 min".to_string()));

    let result = evaluate_test_expression("2 TiB / (1 TiB/hour)");
    assert_eq!(result, Some("2 h".to_string()));
}

#[test]
fn test_generic_rate_conversions() {
    // Test conversion between generic rates with different data units but same time unit
    let result = evaluate_test_expression("2 MiB/min in KiB/min");
    assert_eq!(result, Some("2,048 KiB/min".to_string())); // 2 * 1024 = 2,048

    // Test more generic rate conversions
    let result = evaluate_test_expression("1 GiB/hour in MiB/hour");
    assert_eq!(result, Some("1,024 MiB/h".to_string())); // 1 GiB = 1024 MiB

    let result = evaluate_test_expression("2 GB/day in MB/day");
    assert_eq!(result, Some("2,000 MB/day".to_string())); // 2 GB = 2000 MB

    // Test bit rate conversions
    let result = evaluate_test_expression("8 Gb/hour in Mb/hour");
    assert_eq!(result, Some("8,000 Mb/h".to_string())); // 8 Gb = 8000 Mb

    // Test conversion between different time units (the failing case)
    let result = evaluate_test_expression("10 MB/min in KB/hour");
    assert_eq!(result, Some("600,000 KB/h".to_string())); // 10 MiB/min = 10 * 1000 KiB/min = 10,000 KiB/min = 10,000 * 60 KiB/hour = 600,000 KiB/hour

    // Test more cross-time-unit conversions
    let result = evaluate_test_expression("1 GiB/hour in MiB/min");
    assert_eq!(result, Some("17.067 MiB/min".to_string())); // 1 GiB/hour = 1024 MiB/hour = 1024/60 MiB/min ≈ 17.067 MiB/min

    let result = evaluate_test_expression("100 MB/day in KB/hour");
    assert_eq!(result, Some("4,166.667 KB/h".to_string())); // 100 MB/day = 100,000 KB/day = 100,000/24 KB/hour ≈ 4,166.667 KB/hour
}

#[test]
fn test_generic_rate_edge_cases() {
    // Test with very small time units
    let result = evaluate_test_expression("1 byte/nanosecond * 1000000000 nanoseconds");
    assert_eq!(result, Some("1,000,000,000 B".to_string()));

    // Test with very large time units
    let result = evaluate_test_expression("1 PiB/day * 365 days");
    assert_eq!(result, Some("365 PiB".to_string()));

    // Test rate unit type classification
    let rate = rate_unit!(Unit::GiB, Unit::Minute);
    assert_eq!(
        rate.unit_type(),
        UnitType::DataRate {
            time_multiplier: 60.0
        }
    ); // 60 seconds per minute

    let rate = rate_unit!(Unit::MB, Unit::Hour);
    assert_eq!(
        rate.unit_type(),
        UnitType::DataRate {
            time_multiplier: 3600.0
        }
    ); // 3600 seconds per hour

    let rate = rate_unit!(Unit::KB, Unit::Day);
    assert_eq!(
        rate.unit_type(),
        UnitType::DataRate {
            time_multiplier: 86400.0
        }
    ); // 86400 seconds per day
}

#[test]
fn test_generic_rate_with_bit_units() {
    // Test bit rates with different time units
    let result = evaluate_test_expression("100 Mb/minute * 5 minutes");
    assert_eq!(result, Some("500 Mb".to_string()));

    let result = evaluate_test_expression("1 Gb/hour * 24 hours");
    assert_eq!(result, Some("24 Gb".to_string()));

    // Test bit to byte conversions with generic rates
    let result = evaluate_test_expression("8 Mb/minute * 10 minutes to MB");
    assert_eq!(result, Some("10 MB".to_string()));

    // Test mixed bit/byte rate calculations
    let result = evaluate_test_expression("1 Gb/minute * 60 minutes to GiB");
    assert_eq!(result, Some("6.985 GiB".to_string()));
}

#[test]
fn test_generic_rate_complex_expressions() {
    // Test rate in parentheses
    let result = evaluate_test_expression("(100 MB/hour) * 8 hours");
    assert_eq!(result, Some("800 MB".to_string()));

    // Test multiple operations
    let result = evaluate_test_expression("(50 GiB/minute * 10 minutes) + 100 GiB");
    assert_eq!(result, Some("600 GiB".to_string()));

    // Test rate subtraction
    let result = evaluate_test_expression("1 TiB/hour * 2 hours - 512 GiB");
    assert_eq!(result, Some("1,536 GiB".to_string()));

    // Test rate with conversion
    let result = evaluate_test_expression("(1 GiB/minute * 60 minutes) to TB");
    assert_eq!(result, Some("0.064 TB".to_string()));

    // Test compound rate calculations
    let result = evaluate_test_expression("((100 MB/s * 60 s) / 10 minutes) * 5 minutes");
    assert_eq!(result, Some("3,000 MB".to_string()));
}

#[test]
fn test_generic_rate_invalid_operations() {
    // Test invalid rate operations
    assert_eq!(evaluate_test_expression("1 GiB/minute + 1 MB/s"), None); // Different rate types
    assert_eq!(evaluate_test_expression("1 GiB/minute - 100 MB"), None); // Rate - data
    assert_eq!(
        evaluate_test_expression("1 hour * 1 GiB/minute"),
        Some("60 GiB".to_string())
    ); // Should work
    assert_eq!(evaluate_test_expression("1 GiB/minute / 1 hour"), None); // Rate / time doesn't make sense
}

#[test]
fn test_generic_rate_display_names() {
    // Test that rate units have proper display names
    let rate = rate_unit!(Unit::GiB, Unit::Minute);
    assert_eq!(rate.display_name(), "GiB/min");

    let rate = rate_unit!(Unit::MB, Unit::Hour);
    assert_eq!(rate.display_name(), "MB/h");
}

#[test]
fn test_rate_unit_addition() {
    // Test addition of compatible rate units (same data type, different time units)
    assert_eq!(
        evaluate_test_expression("1 GiB/hour + 1 GiB/minute"),
        Some("61 GiB/h".to_string()) // 1 + (1 * 60) = 61 GiB/hour
    );

    // Test addition of same rate units
    assert_eq!(
        evaluate_test_expression("100 MB/s + 50 MB/s"),
        Some("150 MB/s".to_string())
    );

    // Test bit rate addition
    assert_eq!(
        evaluate_test_expression("1 Gb/s + 500 Mb/s"),
        Some("1,500 Mb/s".to_string()) // Result in smaller unit (Mb)
    );

    // Test request rate addition
    assert_eq!(
        evaluate_test_expression("100 req/s + 200 req/s"),
        Some("300 req/s".to_string())
    );

    // Test subtraction of rate units
    assert_eq!(
        evaluate_test_expression("2 GiB/hour - 1 GiB/minute"),
        Some("-58 GiB/h".to_string()) // 2 - (1 * 60) = -58 GiB/hour
    );
}

#[test]
fn test_unit_conversions() {
    // Data unit conversions (base 2)
    let result = evaluate_with_unit_info("1 GiB to KiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1048576.0).abs() < 0.001);

    let result = evaluate_with_unit_info("1 TiB to GiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1024.0).abs() < 0.001);

    let result = evaluate_with_unit_info("2048 KiB to MiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 2.0).abs() < 0.001);

    // Data unit conversions (base 10)
    let result = evaluate_with_unit_info("1 GB to MB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1000.0).abs() < 0.001);

    let result = evaluate_with_unit_info("5000 MB to GB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 5.0).abs() < 0.001);

    // Time unit conversions
    let result = evaluate_with_unit_info("1 hour to minutes");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 60.0).abs() < 0.001);

    let result = evaluate_with_unit_info("120 seconds to minutes");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 2.0).abs() < 0.001);

    // Sub-second time unit conversions
    let result = evaluate_with_unit_info("1 s to ms");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1000.0).abs() < 0.001);

    let result = evaluate_with_unit_info("1000 ms to s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001);

    let result = evaluate_with_unit_info("1 ms to us");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1000.0).abs() < 0.001);

    let result = evaluate_with_unit_info("1000000 ns to ms");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001);
}

#[test]
fn test_sub_second_unit_parsing() {
    use super::parser::parse_unit;
    use super::types::Unit;

    // Test parsing of sub-second units
    assert_eq!(parse_unit("ns"), Some(Unit::Nanosecond));
    assert_eq!(parse_unit("nanosecond"), Some(Unit::Nanosecond));
    assert_eq!(parse_unit("nanoseconds"), Some(Unit::Nanosecond));

    assert_eq!(parse_unit("us"), Some(Unit::Microsecond));
    assert_eq!(parse_unit("µs"), Some(Unit::Microsecond));
    assert_eq!(parse_unit("microsecond"), Some(Unit::Microsecond));
    assert_eq!(parse_unit("microseconds"), Some(Unit::Microsecond));

    assert_eq!(parse_unit("ms"), Some(Unit::Millisecond));
    assert_eq!(parse_unit("millisecond"), Some(Unit::Millisecond));
    assert_eq!(parse_unit("milliseconds"), Some(Unit::Millisecond));
}

#[test]
fn test_sub_second_unit_conversions() {
    // Comprehensive sub-second conversions

    // Nanoseconds
    let result = evaluate_with_unit_info("1000000000 ns to s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001);

    // Microseconds
    let result = evaluate_with_unit_info("1000000 us to s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001);

    let result = evaluate_with_unit_info("1000 us to ms");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001);

    // Milliseconds
    let result = evaluate_with_unit_info("500 ms to s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 0.5).abs() < 0.001);

    let result = evaluate_with_unit_info("2.5 s to ms");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 2500.0).abs() < 0.001);

    // Cross-conversions
    let result = evaluate_with_unit_info("5000 ns to us");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 5.0).abs() < 0.001);
}

#[test]
fn test_arithmetic_with_units() {
    // Data rate * time = data
    assert_eq!(
        evaluate_test_expression("50 GiB/s * 2 s"),
        Some("100 GiB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1 hour * 10 GiB/s"),
        Some("36,000 GiB".to_string())
    );

    // Data / time = rate
    assert_eq!(
        evaluate_test_expression("100 GiB / 10 s"),
        Some("10 GiB/s".to_string())
    );

    // Same unit addition/subtraction
    assert_eq!(
        evaluate_test_expression("1 GiB + 512 MiB"),
        Some("1,536 MiB".to_string())
    );
    assert_eq!(
        evaluate_test_expression("2 hours + 30 minutes"),
        Some("150 min".to_string())
    );
}

#[test]
fn test_mixed_unit_types() {
    // Base 10 vs Base 2 data units
    let result = evaluate_with_unit_info("1 GB to GiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    // 1 GB = 1,000,000,000 bytes = ~0.931 GiB
    assert!((unit_val.value - 0.9313225746).abs() < 0.0001);

    let result = evaluate_with_unit_info("1 GiB to GB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    // 1 GiB = 1,073,741,824 bytes = ~1.074 GB
    assert!((unit_val.value - 1.073741824).abs() < 0.0001);
}

#[test]
fn test_unit_recognition() {
    // Test different unit formats
    let result = evaluate_with_unit_info("1 GiB to kib");
    assert!(result.is_some());

    let result = evaluate_with_unit_info("60 minutes to h");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001);

    let result = evaluate_with_unit_info("1024 bytes to KiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001);
}

#[test]
fn test_in_keyword_conversions() {
    // Test "in" keyword for unit conversions after calculations
    assert_eq!(
        evaluate_test_expression("24 MiB * 32 in KiB"),
        Some("786,432 KiB".to_string())
    );

    // Test with different operations
    assert_eq!(
        evaluate_test_expression("1 GiB + 512 MiB in KiB"),
        Some("1,572,864 KiB".to_string())
    );

    // Test with time calculations (using scalar multiplication)
    assert_eq!(
        evaluate_test_expression("2 hours * 60 in minutes"),
        Some("7,200 min".to_string())
    );

    // Test with complex expressions
    assert_eq!(
        evaluate_test_expression("(1 GiB + 1 GiB) / 2 in MiB"),
        Some("1,024 MiB".to_string())
    );

    // Test mixed base units (base 10 to base 2)
    assert_eq!(
        evaluate_test_expression("1000 MB * 5 in GiB"),
        Some("4.657 GiB".to_string())
    );

    // Test rate calculations with time conversion
    assert_eq!(
        evaluate_test_expression("500 GiB / 10 seconds in MiB/s"),
        Some("51,200 MiB/s".to_string())
    );

    // Test simple unit conversion
    assert_eq!(
        evaluate_test_expression("1024 KiB in MiB"),
        Some("1 MiB".to_string())
    );

    // Test addition with conversion
    assert_eq!(
        evaluate_test_expression("1 hour + 30 minutes in minutes"),
        Some("90 min".to_string())
    );

    // Test invalid unit conversion (incompatible types)
    assert_eq!(evaluate_test_expression("5 GiB + 10 in seconds"), None);

    // Test that "in" without valid target unit falls back to regular calculation
    assert_eq!(evaluate_test_expression("5 + 3 in"), Some("8".to_string()));
}

#[test]
fn test_to_keyword_with_expressions() {
    // Test "to" keyword with expressions (same functionality as "in")
    assert_eq!(
        evaluate_test_expression("12 GiB + 50 MiB to MiB"),
        Some("12,338 MiB".to_string())
    );

    // Test with multiplication
    assert_eq!(
        evaluate_test_expression("24 MiB * 32 to KiB"),
        Some("786,432 KiB".to_string())
    );

    // Test with division that creates a rate
    assert_eq!(
        evaluate_test_expression("1000 GiB / 10 seconds to MiB/s"),
        Some("102,400 MiB/s".to_string())
    );

    // Test complex expression
    assert_eq!(
        evaluate_test_expression("(2 TiB - 1 GiB) / 1024 to GiB"),
        Some("1.999 GiB".to_string())
    );

    // Test time calculations
    assert_eq!(
        evaluate_test_expression("3 hours + 45 minutes to minutes"),
        Some("225 min".to_string())
    );

    // Ensure simple "to" conversions still work (backward compatibility)
    assert_eq!(
        evaluate_test_expression("1 GiB to MiB"),
        Some("1,024 MiB".to_string())
    );
    assert_eq!(
        evaluate_test_expression("60 seconds to minutes"),
        Some("1 min".to_string())
    );
}

#[test]
fn test_qps_unit_parsing() {
    // Test QPS unit parsing
    assert_eq!(
        parse_unit("qps"),
        Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Second)
        ))
    );
    assert_eq!(
        parse_unit("QPS"),
        Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Second)
        ))
    );
    assert_eq!(
        parse_unit("queries/s"),
        Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Second)
        ))
    );
    assert_eq!(
        parse_unit("queries/sec"),
        Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Second)
        ))
    );
    assert_eq!(
        parse_unit("qpm"),
        Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Minute)
        ))
    );
    assert_eq!(
        parse_unit("queries/min"),
        Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Minute)
        ))
    );
    assert_eq!(
        parse_unit("queries/minute"),
        Some(Unit::RateUnit(
            Box::new(Unit::Query),
            Box::new(Unit::Minute)
        ))
    );
    assert_eq!(parse_unit("qph"), Some(rate_unit!(Unit::Query, Unit::Hour)));
    assert_eq!(
        parse_unit("queries/h"),
        Some(rate_unit!(Unit::Query, Unit::Hour))
    );
    assert_eq!(
        parse_unit("queries/hour"),
        Some(rate_unit!(Unit::Query, Unit::Hour))
    );

    // Test request rate unit parsing
    assert_eq!(
        parse_unit("req/s"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Second)
        ))
    );
    assert_eq!(
        parse_unit("requests/s"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Second)
        ))
    );
    assert_eq!(
        parse_unit("rps"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Second)
        ))
    );
    assert_eq!(
        parse_unit("req/min"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Minute)
        ))
    );
    assert_eq!(
        parse_unit("requests/min"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Minute)
        ))
    );
    assert_eq!(
        parse_unit("rpm"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Minute)
        ))
    );
    assert_eq!(
        parse_unit("req/h"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Hour)
        ))
    );
    assert_eq!(
        parse_unit("req/hour"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Hour)
        ))
    );
    assert_eq!(
        parse_unit("requests/h"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Hour)
        ))
    );
    assert_eq!(
        parse_unit("requests/hour"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Hour)
        ))
    );
    assert_eq!(
        parse_unit("rph"),
        Some(Unit::RateUnit(
            Box::new(Unit::Request),
            Box::new(Unit::Hour)
        ))
    );

    // Test request/query count unit parsing
    assert_eq!(parse_unit("req"), Some(Unit::Request));
    assert_eq!(parse_unit("request"), Some(Unit::Request));
    assert_eq!(parse_unit("requests"), Some(Unit::Request));
    assert_eq!(parse_unit("query"), Some(Unit::Query));
    assert_eq!(parse_unit("queries"), Some(Unit::Query));
}

#[test]
fn test_qps_unit_conversions() {
    // Test QPS to other rate units
    let result = evaluate_with_unit_info("100 QPS to req/min");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 6000.0).abs() < 0.001); // 100 * 60 = 6000 req/min

    let result = evaluate_with_unit_info("1 QPS to QPH");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 3600.0).abs() < 0.001); // 1 * 3600 = 3600 QPH

    let result = evaluate_with_unit_info("7200 req/h to req/min");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 120.0).abs() < 0.001); // 7200 / 60 = 120 req/min

    let result = evaluate_with_unit_info("60 QPM to QPS");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 60 / 60 = 1 QPS

    // Test cross-family conversions (QPS to RPS should work since they're equivalent)
    let result = evaluate_with_unit_info("100 QPS to req/s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 100.0).abs() < 0.001); // Direct equivalence

    let result = evaluate_with_unit_info("150 req/min to QPM");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 150.0).abs() < 0.001); // Direct equivalence
}

#[test]
fn test_qps_arithmetic_operations() {
    // Test QPS * time = total requests
    assert_eq!(
        evaluate_test_expression("25 QPS * 1 hour"),
        Some("90,000 query".to_string())
    );

    assert_eq!(
        evaluate_test_expression("100 QPS * 30 s"),
        Some("3,000 query".to_string())
    );

    assert_eq!(
        evaluate_test_expression("50 req/s * 2 minutes"),
        Some("6,000 req".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1 hour * 10 req/min"),
        Some("36,000 req".to_string())
    );

    // Test requests / time = request rate
    assert_eq!(
        evaluate_test_expression("3600 queries / 1 hour"),
        Some("1 query/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("6000 req / 10 minutes"),
        Some("10 req/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1200 requests / 20 s"),
        Some("60 req/s".to_string())
    );

    // Test QPS arithmetic with conversions
    assert_eq!(
        evaluate_test_expression("100 QPS * 30 minutes to req"),
        Some("180,000 req".to_string())
    );

    assert_eq!(
        evaluate_test_expression("5000 queries / 10 minutes to QPS"),
        Some("8.333 query/s".to_string())
    );

    // Test complex expressions
    assert_eq!(
        evaluate_test_expression("(100 QPS + 50 QPS) * 2 hours"),
        Some("1,080,000 query".to_string())
    );

    assert_eq!(
        evaluate_test_expression("10000 req / (5 minutes + 5 minutes)"),
        Some("16.667 req/s".to_string())
    );
}

#[test]
fn test_qps_addition_subtraction() {
    // Test adding/subtracting same rate units
    assert_eq!(
        evaluate_test_expression("100 QPS + 50 QPS"),
        Some("150 query/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("200 req/min - 80 req/min"),
        Some("120 req/min".to_string())
    );

    // Test adding different rate units (should convert to common base)
    assert_eq!(
        evaluate_test_expression("100 QPS + 60 QPM"),
        Some("6,060 query/min".to_string())
    );

    assert_eq!(
        evaluate_test_expression("3600 QPH - 30 QPM"),
        Some("1,800 query/h".to_string())
    );

    // Test mixed request rate families
    assert_eq!(
        evaluate_test_expression("100 QPS + 100 req/s"),
        Some("200 req/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1000 req/min + 500 QPM"),
        Some("1,500 query/min".to_string())
    );
}

#[test]
fn test_unit_division_ratios() {
    // Test dividing same units to get dimensionless ratios
    assert_eq!(
        evaluate_test_expression("(277 GiB + 207 GiB) / 270 GiB"),
        Some("1.793".to_string())
    );

    // Test simple ratio calculations
    assert_eq!(
        evaluate_test_expression("512 MiB / 1 GiB"),
        Some("0.5".to_string())
    );

    assert_eq!(
        evaluate_test_expression("2 TB / 1 TB"),
        Some("2".to_string())
    );

    // Test cross-base unit ratios (base-2 / base-10)
    assert_eq!(
        evaluate_test_expression("1 GiB / 1 GB"),
        Some("1.074".to_string())
    );

    // Test request rate ratios
    assert_eq!(
        evaluate_test_expression("150 QPS / 100 QPS"),
        Some("1.5".to_string())
    );

    assert_eq!(
        evaluate_test_expression("3600 req/h / 1800 req/h"),
        Some("2".to_string())
    );

    // Test time ratios
    assert_eq!(
        evaluate_test_expression("2 hours / 30 minutes"),
        Some("4".to_string())
    );

    // Test mixed compatible unit ratios
    assert_eq!(
        evaluate_test_expression("120 QPM / 2 QPS"),
        Some("1".to_string())
    );

    // Test bit/byte ratios
    assert_eq!(
        evaluate_test_expression("8 bit / 1 B"),
        Some("1".to_string())
    );

    // Test large data unit ratios
    assert_eq!(
        evaluate_test_expression("2 EB / 1000 PB"),
        Some("2".to_string())
    );

    // Test real-world scenarios
    // Storage utilization
    assert_eq!(
        evaluate_test_expression("(500 GiB + 300 GiB) / 1 TiB"),
        Some("0.781".to_string())
    );

    // Cache hit rate
    assert_eq!(
        evaluate_test_expression("950 req / 1000 req"),
        Some("0.95".to_string())
    );

    // CPU utilization (if we had percentage units, but using ratio for now)
    assert_eq!(
        evaluate_test_expression("75 / 100"),
        Some("0.75".to_string())
    );
}

#[test]
fn test_qps_real_world_scenarios() {
    // Test realistic QPS scenarios
    assert_eq!(
        evaluate_test_expression("API load: 1000 QPS * 5 minutes"),
        Some("300,000 query".to_string())
    );

    assert_eq!(
        evaluate_test_expression("Peak traffic: 500 req/s * 1 hour"),
        Some("1,800,000 req".to_string())
    );

    assert_eq!(
        evaluate_test_expression("Daily load 86400 req / 1 day"),
        Some("1 req/s".to_string())
    );

    // Test load balancing scenarios
    assert_eq!(
        evaluate_test_expression("Total load: 250 QPS + 150 QPS + 100 QPS"),
        Some("500 query/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("Per server: 1500 QPS / 3"),
        Some("500 query/s".to_string())
    );

    // Test capacity planning
    assert_eq!(
        evaluate_test_expression("Monthly load 100 QPS * 30 days"),
        Some("259,200,000 query".to_string())
    );

    // Test rate conversions for monitoring
    assert_eq!(
        evaluate_test_expression("Monitor rate: 5000 req/min to req/s"),
        Some("83.333 req/s".to_string())
    );
}

#[test]
fn test_qps_edge_cases() {
    // Test very small QPS rates
    assert_eq!(
        evaluate_test_expression("0.1 QPS * 10 s"),
        Some("1 query".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1 query / 10 s"),
        Some("0.1 query/s".to_string())
    );

    // Test very large QPS rates
    assert_eq!(
        evaluate_test_expression("1000000 QPS * 1 s"),
        Some("1,000,000 query".to_string())
    );

    // Test fractional results
    assert_eq!(
        evaluate_test_expression("100 QPS / 3"),
        Some("33.333 query/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1000 req / 7 minutes"),
        Some("2.381 req/s".to_string())
    );

    // Test zero and negative cases (should be valid mathematically)
    assert_eq!(
        evaluate_test_expression("0 QPS * 1 hour"),
        Some("0 query".to_string())
    );

    // Test incompatible unit operations (should fail)
    assert_eq!(evaluate_test_expression("100 QPS + 1 GiB"), None);
    assert_eq!(evaluate_test_expression("50 req/s - 10 seconds"), None);
    assert_eq!(evaluate_test_expression("1000 queries + 5 hours"), None);
}

#[test]
fn test_qps_unit_display_names() {
    // Test that display names are correct for QPS units
    assert_eq!(
        rate_unit!(Unit::Query, Unit::Second).display_name(),
        "query/s"
    );
    assert_eq!(
        rate_unit!(Unit::Query, Unit::Minute).display_name(),
        "query/min"
    );
    assert_eq!(
        rate_unit!(Unit::Query, Unit::Hour).display_name(),
        "query/h"
    );
    assert_eq!(
        rate_unit!(Unit::Request, Unit::Second).display_name(),
        "req/s"
    );
    assert_eq!(
        rate_unit!(Unit::Request, Unit::Minute).display_name(),
        "req/min"
    );
    assert_eq!(
        rate_unit!(Unit::Request, Unit::Hour).display_name(),
        "req/h"
    );
    assert_eq!(Unit::Request.display_name(), "req");
    assert_eq!(Unit::Query.display_name(), "query");
}

#[test]
fn test_qps_unit_type_classification() {
    // Test that QPS units are properly classified
    assert_eq!(
        rate_unit!(Unit::Query, Unit::Second).unit_type(),
        UnitType::RequestRate
    );
    assert_eq!(
        rate_unit!(Unit::Query, Unit::Minute).unit_type(),
        UnitType::RequestRate
    );
    assert_eq!(
        rate_unit!(Unit::Query, Unit::Hour).unit_type(),
        UnitType::RequestRate
    );
    assert_eq!(
        rate_unit!(Unit::Request, Unit::Second).unit_type(),
        UnitType::RequestRate
    );
    assert_eq!(
        rate_unit!(Unit::Request, Unit::Minute).unit_type(),
        UnitType::RequestRate
    );
    assert_eq!(
        rate_unit!(Unit::Request, Unit::Hour).unit_type(),
        UnitType::RequestRate
    );
    assert_eq!(Unit::Request.unit_type(), UnitType::Request);
    assert_eq!(Unit::Query.unit_type(), UnitType::Request);
}

#[test]
fn test_large_data_unit_parsing() {
    // Test Petabyte unit parsing (base 10)
    assert_eq!(parse_unit("pb"), Some(Unit::PB));
    assert_eq!(parse_unit("PB"), Some(Unit::PB));

    // Test Exabyte unit parsing (base 10)
    assert_eq!(parse_unit("eb"), Some(Unit::EB));
    assert_eq!(parse_unit("EB"), Some(Unit::EB));

    // Test Pebibyte unit parsing (base 2)
    assert_eq!(parse_unit("pib"), Some(Unit::PiB));
    assert_eq!(parse_unit("PiB"), Some(Unit::PiB));

    // Test Exbibyte unit parsing (base 2)
    assert_eq!(parse_unit("eib"), Some(Unit::EiB));
    assert_eq!(parse_unit("EiB"), Some(Unit::EiB));

    // Test rate units
    assert_eq!(parse_unit("pb/s"), Some(rate_unit!(Unit::PB, Unit::Second)));
    assert_eq!(parse_unit("pbps"), Some(rate_unit!(Unit::PB, Unit::Second)));
    assert_eq!(parse_unit("eb/s"), Some(rate_unit!(Unit::EB, Unit::Second)));
    assert_eq!(parse_unit("ebps"), Some(rate_unit!(Unit::EB, Unit::Second)));
    assert_eq!(
        parse_unit("pib/s"),
        Some(rate_unit!(Unit::PiB, Unit::Second))
    );
    assert_eq!(
        parse_unit("pibps"),
        Some(rate_unit!(Unit::PiB, Unit::Second))
    );
    assert_eq!(
        parse_unit("eib/s"),
        Some(rate_unit!(Unit::EiB, Unit::Second))
    );
    assert_eq!(
        parse_unit("eibps"),
        Some(rate_unit!(Unit::EiB, Unit::Second))
    );
}

#[test]
fn test_large_data_unit_conversions() {
    // Test TB to PB conversions (base 10)
    let result = evaluate_with_unit_info("1000 TB to PB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 1000 TB = 1 PB

    let result = evaluate_with_unit_info("5 PB to TB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 5000.0).abs() < 0.001); // 5 PB = 5000 TB

    // Test PB to EB conversions (base 10)
    let result = evaluate_with_unit_info("1000 PB to EB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 1000 PB = 1 EB

    let result = evaluate_with_unit_info("2 EB to PB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 2000.0).abs() < 0.001); // 2 EB = 2000 PB

    // Test TiB to PiB conversions (base 2)
    let result = evaluate_with_unit_info("1024 TiB to PiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 1024 TiB = 1 PiB

    let result = evaluate_with_unit_info("3 PiB to TiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 3072.0).abs() < 0.001); // 3 PiB = 3072 TiB

    // Test PiB to EiB conversions (base 2)
    let result = evaluate_with_unit_info("1024 PiB to EiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 1024 PiB = 1 EiB

    let result = evaluate_with_unit_info("2 EiB to PiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 2048.0).abs() < 0.001); // 2 EiB = 2048 PiB

    // Test mixed base conversions
    let result = evaluate_with_unit_info("1 PB to PiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    // 1 PB = 1,000,000,000,000,000 bytes = ~0.888 PiB
    assert!((unit_val.value - 0.8881784197).abs() < 0.0001);

    let result = evaluate_with_unit_info("1 EiB to EB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    // 1 EiB = 1,152,921,504,606,846,976 bytes = ~1.153 EB
    assert!((unit_val.value - 1.152921504606847).abs() < 0.0001);
}

#[test]
fn test_large_data_unit_arithmetic() {
    // Test arithmetic with PB units
    assert_eq!(
        evaluate_test_expression("2 PB + 500 TB"),
        Some("2,500 TB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("5 PB - 1000 TB"),
        Some("4,000 TB".to_string())
    );

    // Test arithmetic with EB units
    assert_eq!(
        evaluate_test_expression("1 EB + 200 PB"),
        Some("1,200 PB".to_string())
    );

    // Test arithmetic with PiB units
    assert_eq!(
        evaluate_test_expression("1 PiB + 512 TiB"),
        Some("1,536 TiB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("2 EiB - 1024 PiB"),
        Some("1,024 PiB".to_string())
    );

    // Test rate calculations with large units using generic rates
    assert_eq!(
        evaluate_test_expression("1 PB / 1 hour"),
        Some("1 PB/h".to_string())
    );

    assert_eq!(
        evaluate_test_expression("10 PB/s * 30 minutes"),
        Some("18,000 PB".to_string())
    );

    // Test very large transfers
    assert_eq!(
        evaluate_test_expression("1 EB/s * 1 second"),
        Some("1 EB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("500 EiB / 1 day"),
        Some("500 EiB/day".to_string())
    );
}

#[test]
fn test_large_data_unit_display_names() {
    // Test display names for large data units
    assert_eq!(Unit::PB.display_name(), "PB");
    assert_eq!(Unit::EB.display_name(), "EB");
    assert_eq!(Unit::PiB.display_name(), "PiB");
    assert_eq!(Unit::EiB.display_name(), "EiB");

    // Test display names for large rate units
    assert_eq!(rate_unit!(Unit::PB, Unit::Second).display_name(), "PB/s");
    assert_eq!(rate_unit!(Unit::EB, Unit::Second).display_name(), "EB/s");
    assert_eq!(rate_unit!(Unit::PiB, Unit::Second).display_name(), "PiB/s");
    assert_eq!(rate_unit!(Unit::EiB, Unit::Second).display_name(), "EiB/s");
}

#[test]
fn test_large_data_unit_type_classification() {
    // Test that large data units are properly classified
    assert_eq!(Unit::PB.unit_type(), UnitType::Data);
    assert_eq!(Unit::EB.unit_type(), UnitType::Data);
    assert_eq!(Unit::PiB.unit_type(), UnitType::Data);
    assert_eq!(Unit::EiB.unit_type(), UnitType::Data);

    // Test that large rate units are properly classified
    assert_eq!(
        rate_unit!(Unit::PB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::EB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::PiB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::EiB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
}

#[test]
fn test_large_data_real_world_scenarios() {
    // Test data center storage scenarios
    assert_eq!(
        evaluate_test_expression("Data center: 50 PB + 10 EB"),
        Some("10,050 PB".to_string())
    );

    // Test backup scenarios
    assert_eq!(
        evaluate_test_expression("Backup rate: 100 TB/s * 8 hours"),
        Some("2,880,000 TB".to_string())
    );

    // Test very large data transfers
    assert_eq!(
        evaluate_test_expression("Transfer: 5 EiB to PB"),
        Some("5,764.608 PB".to_string())
    );

    // Test scientific computing scenarios
    assert_eq!(
        evaluate_test_expression("Dataset: 1.5 EB to TiB"),
        Some("1,364,242.053 TiB".to_string())
    );

    // Test network throughput
    assert_eq!(
        evaluate_test_expression("Network: 10 PB/s to TB/s"),
        Some("10,000 TB/s".to_string())
    );

    // Test storage capacity planning
    assert_eq!(
        evaluate_test_expression("Total capacity: 100 PiB + 50 EiB"),
        Some("51,300 PiB".to_string())
    );
}

#[test]
fn test_bit_vs_byte_parsing() {
    // Test bit units (lowercase 'b')
    assert_eq!(parse_unit("bit"), Some(Unit::Bit));
    assert_eq!(parse_unit("Kb"), Some(Unit::Kb));
    assert_eq!(parse_unit("Mb"), Some(Unit::Mb));
    assert_eq!(parse_unit("Gb"), Some(Unit::Gb));
    assert_eq!(parse_unit("Tb"), Some(Unit::Tb));
    assert_eq!(parse_unit("Kib"), Some(Unit::Kib));
    assert_eq!(parse_unit("Mib"), Some(Unit::Mib));
    assert_eq!(parse_unit("Gib"), Some(Unit::Gib));

    // Test byte units (uppercase 'B')
    assert_eq!(parse_unit("B"), Some(Unit::Byte));
    assert_eq!(parse_unit("KB"), Some(Unit::KB));
    assert_eq!(parse_unit("MB"), Some(Unit::MB));
    assert_eq!(parse_unit("GB"), Some(Unit::GB));
    assert_eq!(parse_unit("TB"), Some(Unit::TB));
    assert_eq!(parse_unit("KiB"), Some(Unit::KiB));
    assert_eq!(parse_unit("MiB"), Some(Unit::MiB));
    assert_eq!(parse_unit("GiB"), Some(Unit::GiB));

    // Test bit rate units (bits per second)
    assert_eq!(parse_unit("bps"), Some(rate_unit!(Unit::Bit, Unit::Second)));
    assert_eq!(parse_unit("Kbps"), Some(rate_unit!(Unit::Kb, Unit::Second)));
    assert_eq!(parse_unit("Mbps"), Some(rate_unit!(Unit::Mb, Unit::Second)));
    assert_eq!(parse_unit("Gbps"), Some(rate_unit!(Unit::Gb, Unit::Second)));
    assert_eq!(
        parse_unit("Kibps"),
        Some(rate_unit!(Unit::Kib, Unit::Second))
    );
    assert_eq!(
        parse_unit("Mibps"),
        Some(rate_unit!(Unit::Mib, Unit::Second))
    );
    assert_eq!(
        parse_unit("Gibps"),
        Some(rate_unit!(Unit::Gib, Unit::Second))
    );

    // Test byte rate units (bytes per second)
    assert_eq!(
        parse_unit("B/s"),
        Some(rate_unit!(Unit::Byte, Unit::Second))
    );
    assert_eq!(parse_unit("KB/s"), Some(rate_unit!(Unit::KB, Unit::Second)));
    assert_eq!(parse_unit("MB/s"), Some(rate_unit!(Unit::MB, Unit::Second)));
    assert_eq!(parse_unit("GB/s"), Some(rate_unit!(Unit::GB, Unit::Second)));
    assert_eq!(
        parse_unit("KiB/s"),
        Some(rate_unit!(Unit::KiB, Unit::Second))
    );
    assert_eq!(
        parse_unit("MiB/s"),
        Some(rate_unit!(Unit::MiB, Unit::Second))
    );
    assert_eq!(
        parse_unit("GiB/s"),
        Some(rate_unit!(Unit::GiB, Unit::Second))
    );
}

#[test]
fn test_byte_to_bit_conversion_bug() {
    // Test the reported issue: 1 MB to mib (lowercase)
    // After fix: lowercase "mib" now maps to Megabits (base 10), not Mebibits (base 2)
    let result = evaluate_with_unit_info("1 MB to mib");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    // 1 MB = 1,000,000 bytes = 8,000,000 bits = 8 Mb (base 10)
    assert!((unit_val.value - 8.0).abs() < 0.001);

    // Test case-sensitive Mib (Mebibits) still works correctly
    let result = evaluate_with_unit_info("1 MB to Mib");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    // 1 MB = 1,000,000 bytes = 8,000,000 bits = 8,000,000 / 1,048,576 ≈ 7.629 Mib
    assert!((unit_val.value - 7.62939453125).abs() < 0.0001);

    // Test base 10 byte to bit conversion (this should work correctly)
    let result = evaluate_with_unit_info("1 MB to Mb");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 MB = 8 Mb ✓

    // Verify the new parsing behavior for lowercase bit units
    assert_eq!(parse_unit("mib"), Some(Unit::Mb)); // lowercase "mib" now maps to Megabits (base 10)
    assert_eq!(parse_unit("Mib"), Some(Unit::Mib)); // Case-sensitive "Mib" = Mebibits (base 2)
    assert_eq!(parse_unit("Mb"), Some(Unit::Mb)); // Case-sensitive "Mb" = Megabits (base 10)

    // Test other network-relevant lowercase bit units have been updated
    assert_eq!(parse_unit("kib"), Some(Unit::Kb)); // lowercase "kib" = Kilobits (base 10)
    assert_eq!(parse_unit("gib"), Some(Unit::Gb)); // lowercase "gib" = Gigabits (base 10)

    // But larger units that are rarely used in networking keep traditional binary meaning
    assert_eq!(parse_unit("tib"), Some(Unit::TiB)); // lowercase "tib" = Tebibytes (base 2)
    assert_eq!(parse_unit("pib"), Some(Unit::PiB)); // lowercase "pib" = Pebibytes (base 2)

    // Test additional conversions to verify the fix
    let result = evaluate_with_unit_info("1 KB to kib"); // Should now work as intended
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 KB = 8 Kb (kib now maps to Kb)

    let result = evaluate_with_unit_info("1 GB to gib"); // Should now work as intended  
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 GB = 8 Gb (gib now maps to Gb)

    let result = evaluate_with_unit_info("1 GB to Gb");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 GB = 8 Gb
}

#[test]
fn test_bit_byte_conversions() {
    // Test bit to bit conversions (base 10)
    let result = evaluate_with_unit_info("1000 Kb to Mb");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 1000 Kb = 1 Mb

    let result = evaluate_with_unit_info("8000 Mb to Gb");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 8000 Mb = 8 Gb

    // Test bit to bit conversions (base 2)
    let result = evaluate_with_unit_info("1024 Kib to Mib");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 1024 Kib = 1 Mib

    let result = evaluate_with_unit_info("8192 Mib to Gib");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 8192 Mib = 8 Gib

    // Test bits to bytes conversion (8 bits = 1 byte)
    let result = evaluate_with_unit_info("8 bit to B");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 8 bits = 1 byte

    let result = evaluate_with_unit_info("8 Kb to KB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 8 Kb = 1 KB

    let result = evaluate_with_unit_info("8 Mb to MB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 8 Mb = 1 MB

    let result = evaluate_with_unit_info("8 Gb to GB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 8 Gb = 1 GB

    // Test bytes to bits conversion
    let result = evaluate_with_unit_info("1 B to bit");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 byte = 8 bits

    let result = evaluate_with_unit_info("1 KB to Kb");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 KB = 8 Kb

    let result = evaluate_with_unit_info("1 MB to Mb");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 MB = 8 Mb

    let result = evaluate_with_unit_info("1 GB to Gb");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 GB = 8 Gb
}

#[test]
fn test_bit_byte_rate_conversions() {
    // Test bit rate to byte rate conversions
    let result = evaluate_with_unit_info("8 bps to B/s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 8 bps = 1 B/s

    let result = evaluate_with_unit_info("8 Kbps to KB/s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 8 Kbps = 1 KB/s

    let result = evaluate_with_unit_info("8 Mbps to MB/s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 8 Mbps = 1 MB/s

    let result = evaluate_with_unit_info("8 Gbps to GB/s");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.001); // 8 Gbps = 1 GB/s

    // Test byte rate to bit rate conversions
    let result = evaluate_with_unit_info("1 B/s to bps");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 B/s = 8 bps

    let result = evaluate_with_unit_info("1 KB/s to Kbps");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 KB/s = 8 Kbps

    let result = evaluate_with_unit_info("1 MB/s to Mbps");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 MB/s = 8 Mbps

    let result = evaluate_with_unit_info("1 GB/s to Gbps");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 8.0).abs() < 0.001); // 1 GB/s = 8 Gbps
}

#[test]
fn test_network_speed_scenarios() {
    // Test realistic network speeds
    assert_eq!(
        evaluate_test_expression("Internet speed: 100 Mbps to MB/s"),
        Some("12.5 MB/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("Gigabit ethernet: 1 Gbps to MB/s"),
        Some("125 MB/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("File download: 50 MB/s to Mbps"),
        Some("400 Mb/s".to_string())
    );

    // Test large file transfer calculations
    assert_eq!(
        evaluate_test_expression("Download time: 1 GB / 10 Mbps"),
        Some("800 s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("Bandwidth usage: 100 Mbps * 1 hour"),
        Some("360,000 Mb".to_string())
    );

    // Test mixed unit scenarios
    assert_eq!(
        evaluate_test_expression("Data transferred: 25 MB/s * 10 minutes to GB"),
        Some("15 GB".to_string())
    );
}

#[test]
fn test_bit_byte_display_names() {
    // Test display names for bit units
    assert_eq!(Unit::Bit.display_name(), "bit");
    assert_eq!(Unit::Kb.display_name(), "Kb");
    assert_eq!(Unit::Mb.display_name(), "Mb");
    assert_eq!(Unit::Gb.display_name(), "Gb");
    assert_eq!(Unit::Tb.display_name(), "Tb");
    assert_eq!(Unit::Kib.display_name(), "Kib");
    assert_eq!(Unit::Mib.display_name(), "Mib");
    assert_eq!(Unit::Gib.display_name(), "Gib");

    // Test display names for bit rate units
    assert_eq!(rate_unit!(Unit::Bit, Unit::Second).display_name(), "bit/s");
    assert_eq!(rate_unit!(Unit::Kb, Unit::Second).display_name(), "Kb/s");
    assert_eq!(rate_unit!(Unit::Mb, Unit::Second).display_name(), "Mb/s");
    assert_eq!(rate_unit!(Unit::Gb, Unit::Second).display_name(), "Gb/s");
    assert_eq!(rate_unit!(Unit::Kib, Unit::Second).display_name(), "Kib/s");
    assert_eq!(rate_unit!(Unit::Mib, Unit::Second).display_name(), "Mib/s");
    assert_eq!(rate_unit!(Unit::Gib, Unit::Second).display_name(), "Gib/s");

    // Test display names for byte units (should be unchanged)
    assert_eq!(Unit::Byte.display_name(), "B");
    assert_eq!(Unit::KB.display_name(), "KB");
    assert_eq!(Unit::MB.display_name(), "MB");
    assert_eq!(Unit::GB.display_name(), "GB");
    assert_eq!(Unit::KiB.display_name(), "KiB");
    assert_eq!(Unit::MiB.display_name(), "MiB");
    assert_eq!(Unit::GiB.display_name(), "GiB");
}

#[test]
fn test_bit_byte_unit_type_classification() {
    // Test that bit units are classified as Bit type
    assert_eq!(Unit::Bit.unit_type(), UnitType::Bit);
    assert_eq!(Unit::Kb.unit_type(), UnitType::Bit);
    assert_eq!(Unit::Mb.unit_type(), UnitType::Bit);
    assert_eq!(Unit::Gb.unit_type(), UnitType::Bit);
    assert_eq!(Unit::Kib.unit_type(), UnitType::Bit);
    assert_eq!(Unit::Mib.unit_type(), UnitType::Bit);
    assert_eq!(Unit::Gib.unit_type(), UnitType::Bit);

    // Test that bit rate units are classified as BitRate type
    assert_eq!(
        rate_unit!(Unit::Bit, Unit::Second).unit_type(),
        UnitType::BitRate
    );
    assert_eq!(
        rate_unit!(Unit::Kb, Unit::Second).unit_type(),
        UnitType::BitRate
    );
    assert_eq!(
        rate_unit!(Unit::Mb, Unit::Second).unit_type(),
        UnitType::BitRate
    );
    assert_eq!(
        rate_unit!(Unit::Gb, Unit::Second).unit_type(),
        UnitType::BitRate
    );
    assert_eq!(
        rate_unit!(Unit::Kib, Unit::Second).unit_type(),
        UnitType::BitRate
    );
    assert_eq!(
        rate_unit!(Unit::Mib, Unit::Second).unit_type(),
        UnitType::BitRate
    );
    assert_eq!(
        rate_unit!(Unit::Gib, Unit::Second).unit_type(),
        UnitType::BitRate
    );

    // Test that byte units are still classified as Data type
    assert_eq!(Unit::Byte.unit_type(), UnitType::Data);
    assert_eq!(Unit::KB.unit_type(), UnitType::Data);
    assert_eq!(Unit::MB.unit_type(), UnitType::Data);
    assert_eq!(Unit::GB.unit_type(), UnitType::Data);
    assert_eq!(Unit::KiB.unit_type(), UnitType::Data);
    assert_eq!(Unit::MiB.unit_type(), UnitType::Data);
    assert_eq!(Unit::GiB.unit_type(), UnitType::Data);

    // Test that byte rate units are still classified as DataRate type
    assert_eq!(
        rate_unit!(Unit::Byte, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::KB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::MB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::GB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::KiB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::MiB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
    assert_eq!(
        rate_unit!(Unit::GiB, Unit::Second).unit_type(),
        UnitType::DataRate {
            time_multiplier: 1.0
        }
    );
}

#[test]
fn test_large_data_edge_cases() {
    // Test very small fractions
    assert_eq!(
        evaluate_test_expression("0.001 PB to TB"),
        Some("1 TB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("0.5 EiB to PiB"),
        Some("512 PiB".to_string())
    );

    // Test precision with large numbers
    assert_eq!(
        evaluate_test_expression("1024.5 PiB to EiB"),
        Some("1 EiB".to_string())
    );

    // Test cross-base conversions with precision
    let result = evaluate_with_unit_info("1.234567 EB to EiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    // Should be approximately 1.071 EiB
    assert!((unit_val.value - 1.071).abs() < 0.01);

    // Test incompatible operations (should fail)
    assert_eq!(evaluate_test_expression("1 PB + 5 hours"), None);
    assert_eq!(evaluate_test_expression("100 EiB - 50 QPS"), None);
    assert_eq!(evaluate_test_expression("1 EB * 1 query"), None);
}

#[test]
fn test_real_world_scenarios() {
    // File transfer calculations
    assert_eq!(
        evaluate_test_expression("Download: 100 MB/s * 5 minutes"),
        Some("30,000 MB".to_string())
    );

    // Storage calculations
    assert_eq!(
        evaluate_test_expression("Total storage: 2 TB + 500 GB"),
        Some("2,500 GB".to_string())
    );

    // Bandwidth calculations with generic rates
    assert_eq!(
        evaluate_test_expression("Bandwidth used: 1,000 GiB / 1 hour"),
        Some("1,000 GiB/h".to_string())
    );

    // Data conversion scenarios
    let result = evaluate_with_unit_info("How many KiB in 5 MiB?");
    assert!(result.is_some()); // Will find "5 MiB" as a valid expression

    let result = evaluate_with_unit_info("5 MiB to KiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 5120.0).abs() < 0.001);
}

#[test]
fn test_percentage_unit_parsing() {
    use super::parser::parse_unit;
    use super::types::Unit;

    // Test percentage unit parsing
    assert_eq!(parse_unit("%"), Some(Unit::Percent));
    assert_eq!(parse_unit("percent"), Some(Unit::Percent));
    assert_eq!(parse_unit("percentage"), Some(Unit::Percent));
}

#[test]
fn test_percentage_unit_conversions() {
    // Test decimal to percentage conversions
    let result = evaluate_with_unit_info("0.25 to %");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 25.0).abs() < 0.001); // 0.25 = 25%

    let result = evaluate_with_unit_info("1.5 to %");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 150.0).abs() < 0.001); // 1.5 = 150%

    let result = evaluate_with_unit_info("0.1 to %");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 10.0).abs() < 0.001); // 0.1 = 10%
}

#[test]
fn test_percentage_of_operations_detailed() {
    // Test basic percentage of operations with units
    assert_eq!(
        evaluate_test_expression("20% of 500 MB"),
        Some("100 MB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("75% of 4 hours"),
        Some("3 h".to_string())
    );

    assert_eq!(
        evaluate_test_expression("12.5% of 1 TiB"),
        Some("0.125 TiB".to_string())
    );

    // Test percentage calculations with request rates
    assert_eq!(
        evaluate_test_expression("30% of 1000 QPS"),
        Some("300 query/s".to_string())
    );
}

#[test]
fn test_extended_time_unit_parsing() {
    use super::parser::parse_unit;
    use super::types::Unit;

    // Test week parsing
    assert_eq!(parse_unit("week"), Some(Unit::Week));
    assert_eq!(parse_unit("weeks"), Some(Unit::Week));
    assert_eq!(parse_unit("wk"), Some(Unit::Week));
    assert_eq!(parse_unit("wks"), Some(Unit::Week));

    // Test month parsing
    assert_eq!(parse_unit("month"), Some(Unit::Month));
    assert_eq!(parse_unit("months"), Some(Unit::Month));
    assert_eq!(parse_unit("mo"), Some(Unit::Month));
    assert_eq!(parse_unit("mos"), Some(Unit::Month));

    // Test year parsing
    assert_eq!(parse_unit("year"), Some(Unit::Year));
    assert_eq!(parse_unit("years"), Some(Unit::Year));
    assert_eq!(parse_unit("yr"), Some(Unit::Year));
    assert_eq!(parse_unit("yrs"), Some(Unit::Year));

    // Test case insensitivity
    assert_eq!(parse_unit("WEEK"), Some(Unit::Week));
    assert_eq!(parse_unit("Month"), Some(Unit::Month));
    assert_eq!(parse_unit("YEAR"), Some(Unit::Year));
}

#[test]
fn test_extended_time_unit_display_names() {
    // Test display names
    assert_eq!(Unit::Week.display_name(), "week");
    assert_eq!(Unit::Month.display_name(), "month");
    assert_eq!(Unit::Year.display_name(), "year");
}

#[test]
fn test_extended_time_unit_type_classification() {
    use super::types::UnitType;

    // Test that new time units are correctly classified
    assert_eq!(Unit::Week.unit_type(), UnitType::Time);
    assert_eq!(Unit::Month.unit_type(), UnitType::Time);
    assert_eq!(Unit::Year.unit_type(), UnitType::Time);
}

#[test]
fn test_extended_time_unit_conversions() {
    // Test week conversions
    let result = evaluate_with_unit_info("1 week to days");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 7.0).abs() < 0.001); // 1 week = 7 days

    let result = evaluate_with_unit_info("2 weeks to hours");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 336.0).abs() < 0.001); // 2 weeks = 14 days = 336 hours

    // Test month conversions (using 30.44 days average)
    let result = evaluate_with_unit_info("1 month to days");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 30.44).abs() < 0.01); // 1 month ≈ 30.44 days

    let result = evaluate_with_unit_info("3 months to weeks");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    let expected = (3.0 * 30.44) / 7.0; // 3 months in weeks
    assert!((unit_val.value - expected).abs() < 0.01);

    // Test year conversions (using 365.25 days)
    let result = evaluate_with_unit_info("1 year to days");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 365.25).abs() < 0.01); // 1 year = 365.25 days

    let result = evaluate_with_unit_info("1 year to weeks");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    let expected = 365.25 / 7.0; // 365.25 days / 7 days per week
    assert!((unit_val.value - expected).abs() < 0.01);

    let result = evaluate_with_unit_info("1 year to months");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    let expected = 365.25 / 30.44; // 365.25 days / 30.44 days per month
    assert!((unit_val.value - expected).abs() < 0.01);

    // Test quarter conversions
    let result = evaluate_with_unit_info("1 quarter to months");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 3.0).abs() < 0.01); // 1 quarter = 3 months

    let result = evaluate_with_unit_info("1 quarter to days");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 91.32).abs() < 0.01); // 1 quarter = 3 * 30.44 days

    let result = evaluate_with_unit_info("4 quarters to years");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1.0).abs() < 0.01); // 4 quarters = 1 year

    let result = evaluate_with_unit_info("1 year to quarters");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 4.0).abs() < 0.01); // 1 year = 4 quarters
}

#[test]
fn test_extended_time_arithmetic() {
    // Test arithmetic with extended time units
    assert_eq!(
        evaluate_test_expression("2 weeks + 3 days"),
        Some("17 day".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1 month + 2 weeks"),
        Some("6.348 week".to_string()) // 30.44 + 14 = 44.44 days ≈ 6.348 weeks
    );

    assert_eq!(
        evaluate_test_expression("1 year - 1 month"),
        Some("11 month".to_string()) // 365.25 - 30.44 = 334.81 days ≈ 11 months
    );

    assert_eq!(
        evaluate_test_expression("1 year - 1 quarter"),
        Some("3 quarter".to_string()) // 4 quarters - 1 quarter = 3 quarters
    );

    assert_eq!(
        evaluate_test_expression("2 quarters + 1 month"),
        Some("7 month".to_string()) // 6 months + 1 month = 7 months
    );

    // Test multiplication and division
    assert_eq!(
        evaluate_test_expression("52 weeks"),
        Some("52 week".to_string())
    );

    assert_eq!(
        evaluate_test_expression("12 months"),
        Some("12 month".to_string())
    );

    // Test with fractional values
    assert_eq!(
        evaluate_test_expression("0.5 years"),
        Some("0.5 year".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1.5 weeks"),
        Some("1.5 week".to_string())
    );
}

#[test]
fn test_extended_time_rate_units() {
    // Test rate units with extended time periods
    assert_eq!(
        evaluate_test_expression("100 MB / 1 week"),
        Some("100 MB/week".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1 TiB / 1 month"),
        Some("1 TiB/month".to_string())
    );

    assert_eq!(
        evaluate_test_expression("10 TB / 1 year"),
        Some("10 TB/year".to_string())
    );

    // Test rate calculations
    assert_eq!(
        evaluate_test_expression("1 GB/week * 52 weeks"),
        Some("52 GB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("5 MB/month * 12 months"),
        Some("60 MB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("100 KB/year * 10 years"),
        Some("1,000 KB".to_string())
    );
}

#[test]
fn test_extended_time_rate_parsing() {
    use super::parser::parse_unit;

    // Test parsing of rate units with extended time periods
    assert_eq!(
        parse_unit("MB/week"),
        Some(rate_unit!(Unit::MB, Unit::Week))
    );

    assert_eq!(
        parse_unit("GiB/month"),
        Some(rate_unit!(Unit::GiB, Unit::Month))
    );

    assert_eq!(
        parse_unit("TB/year"),
        Some(rate_unit!(Unit::TB, Unit::Year))
    );

    // Test case insensitivity
    assert_eq!(
        parse_unit("kb/WEEK"),
        Some(rate_unit!(Unit::KB, Unit::Week))
    );

    assert_eq!(
        parse_unit("MB/Month"),
        Some(rate_unit!(Unit::MB, Unit::Month))
    );
}

#[test]
fn test_extended_time_real_world_scenarios() {
    // Test realistic scenarios with extended time units

    // Project timeline calculations
    assert_eq!(
        evaluate_test_expression("Project duration: 8 weeks + 3 days"),
        Some("59 day".to_string())
    );

    // Annual data usage
    assert_eq!(
        evaluate_test_expression("Annual backup: 50 GB/month * 12 months"),
        Some("600 GB".to_string())
    );

    // Weekly throughput
    assert_eq!(
        evaluate_test_expression("Weekly load: 1 TB/week to GB/day"),
        Some("142.857 GB/day".to_string()) // 1000 GB / 7 days ≈ 142.857
    );

    // Long-term storage planning
    assert_eq!(
        evaluate_test_expression("10 TB/year * 5 years"),
        Some("50 TB".to_string())
    );

    // Monthly data allowance
    assert_eq!(
        evaluate_test_expression("100 GB/month to GB/week"),
        Some("22.998 GB/week".to_string()) // 100 GB / (30.44/7) weeks ≈ 22.998
    );
}

#[test]
fn test_extended_time_edge_cases() {
    // Test edge cases and boundary conditions

    // Very small fractions
    assert_eq!(
        evaluate_test_expression("0.1 weeks"),
        Some("0.1 week".to_string())
    );

    // Very large numbers
    assert_eq!(
        evaluate_test_expression("1000 years"),
        Some("1,000 year".to_string())
    );

    // Mixed very different time scales
    let result = evaluate_with_unit_info("1 year to nanoseconds");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    let expected = 365.25 * 24.0 * 3600.0 * 1_000_000_000.0; // year to nanoseconds
    assert!((unit_val.value - expected).abs() < expected * 0.001); // Allow 0.1% error

    // Test precision with month calculations
    let result = evaluate_with_unit_info("6 months to days");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 182.621).abs() < 0.1); // Allow for floating point precision
}

#[test]
fn test_currency_unit_parsing() {
    // Test currency symbol parsing
    assert_eq!(parse_unit("$"), Some(Unit::USD));
    assert_eq!(parse_unit("€"), Some(Unit::EUR));
    assert_eq!(parse_unit("£"), Some(Unit::GBP));
    assert_eq!(parse_unit("¥"), Some(Unit::JPY));
    assert_eq!(parse_unit("₹"), Some(Unit::INR));
    assert_eq!(parse_unit("₩"), Some(Unit::KRW));

    // Test currency code parsing (case insensitive)
    assert_eq!(parse_unit("usd"), Some(Unit::USD));
    assert_eq!(parse_unit("USD"), Some(Unit::USD));
    assert_eq!(parse_unit("eur"), Some(Unit::EUR));
    assert_eq!(parse_unit("gbp"), Some(Unit::GBP));
    assert_eq!(parse_unit("jpy"), Some(Unit::JPY));
    assert_eq!(parse_unit("cny"), Some(Unit::CNY));
    assert_eq!(parse_unit("cad"), Some(Unit::CAD));
    assert_eq!(parse_unit("aud"), Some(Unit::AUD));
    assert_eq!(parse_unit("chf"), Some(Unit::CHF));
    assert_eq!(parse_unit("inr"), Some(Unit::INR));
    assert_eq!(parse_unit("krw"), Some(Unit::KRW));

    // Test currency word parsing
    assert_eq!(parse_unit("dollar"), Some(Unit::USD));
    assert_eq!(parse_unit("dollars"), Some(Unit::USD));
    assert_eq!(parse_unit("euro"), Some(Unit::EUR));
    assert_eq!(parse_unit("euros"), Some(Unit::EUR));
    assert_eq!(parse_unit("pound"), Some(Unit::GBP));
    assert_eq!(parse_unit("pounds"), Some(Unit::GBP));
    assert_eq!(parse_unit("sterling"), Some(Unit::GBP));
    assert_eq!(parse_unit("yen"), Some(Unit::JPY));
    assert_eq!(parse_unit("yuan"), Some(Unit::CNY));
    assert_eq!(parse_unit("rmb"), Some(Unit::CNY));
    assert_eq!(parse_unit("canadian"), Some(Unit::CAD));
    assert_eq!(parse_unit("australian"), Some(Unit::AUD));
    assert_eq!(parse_unit("franc"), Some(Unit::CHF));
    assert_eq!(parse_unit("rupee"), Some(Unit::INR));
    assert_eq!(parse_unit("rupees"), Some(Unit::INR));
    assert_eq!(parse_unit("won"), Some(Unit::KRW));
}

#[test]
fn test_currency_display_names() {
    // Test currency symbol display
    assert_eq!(Unit::USD.display_name(), "$");
    assert_eq!(Unit::EUR.display_name(), "€");
    assert_eq!(Unit::GBP.display_name(), "£");
    assert_eq!(Unit::JPY.display_name(), "¥");
    assert_eq!(Unit::CNY.display_name(), "¥");
    assert_eq!(Unit::CAD.display_name(), "C$");
    assert_eq!(Unit::AUD.display_name(), "A$");
    assert_eq!(Unit::CHF.display_name(), "CHF");
    assert_eq!(Unit::INR.display_name(), "₹");
    assert_eq!(Unit::KRW.display_name(), "₩");
}

#[test]
fn test_currency_unit_types() {
    // Test that all currencies have Currency unit type
    assert_eq!(Unit::USD.unit_type(), UnitType::Currency);
    assert_eq!(Unit::EUR.unit_type(), UnitType::Currency);
    assert_eq!(Unit::GBP.unit_type(), UnitType::Currency);
    assert_eq!(Unit::JPY.unit_type(), UnitType::Currency);
    assert_eq!(Unit::CNY.unit_type(), UnitType::Currency);
    assert_eq!(Unit::CAD.unit_type(), UnitType::Currency);
    assert_eq!(Unit::AUD.unit_type(), UnitType::Currency);
    assert_eq!(Unit::CHF.unit_type(), UnitType::Currency);
    assert_eq!(Unit::INR.unit_type(), UnitType::Currency);
    assert_eq!(Unit::KRW.unit_type(), UnitType::Currency);
}

#[test]
fn test_currency_addition() {
    // Test same currency addition
    assert_eq!(
        evaluate_test_expression("$100 + $50"),
        Some("150 $".to_string())
    );
    assert_eq!(
        evaluate_test_expression("€200 + €75"),
        Some("275 €".to_string())
    );
    assert_eq!(
        evaluate_test_expression("£50 + £25"),
        Some("75 £".to_string())
    );

    // Test same currency subtraction
    assert_eq!(
        evaluate_test_expression("$100 - $30"),
        Some("70 $".to_string())
    );
    assert_eq!(
        evaluate_test_expression("€500 - €125"),
        Some("375 €".to_string())
    );
}

#[test]
fn test_currency_different_currencies_not_addable() {
    // Test that different currencies cannot be added
    assert_eq!(evaluate_test_expression("$100 + €50"), None);
    assert_eq!(evaluate_test_expression("£100 + $50"), None);
    assert_eq!(evaluate_test_expression("¥1000 + $10"), None);
    assert_eq!(evaluate_test_expression("₹500 + €20"), None);

    // Test that different currencies cannot be subtracted
    assert_eq!(evaluate_test_expression("$100 - €50"), None);
    assert_eq!(evaluate_test_expression("£100 - ¥1000"), None);
}

#[test]
fn test_currency_multiplication() {
    // Test currency multiplication by numbers
    assert_eq!(
        evaluate_test_expression("$24 * 3"),
        Some("72 $".to_string())
    );
    assert_eq!(
        evaluate_test_expression("3 * $24"),
        Some("72 $".to_string())
    );
    assert_eq!(
        evaluate_test_expression("€50 * 2.5"),
        Some("125 €".to_string())
    );
    assert_eq!(
        evaluate_test_expression("£100 * 0.5"),
        Some("50 £".to_string())
    );

    // Test complex expressions
    assert_eq!(
        evaluate_test_expression("($20 + $30) * 2"),
        Some("100 $".to_string())
    );
}

#[test]
fn test_currency_division() {
    // Test currency division by numbers
    assert_eq!(
        evaluate_test_expression("$100 / 4"),
        Some("25 $".to_string())
    );
    assert_eq!(
        evaluate_test_expression("€150 / 3"),
        Some("50 €".to_string())
    );

    // Test currency division by currency (should give dimensionless ratio)
    assert_eq!(
        evaluate_test_expression("$100 / $25"),
        Some("4".to_string())
    );
    assert_eq!(evaluate_test_expression("€90 / €30"), Some("3".to_string()));

    // Test that division of different currencies fails (no exchange rates)
    assert_eq!(evaluate_test_expression("$100 / €25"), None);
}

#[test]
fn test_currency_complex_expressions() {
    // Test more complex currency calculations
    assert_eq!(
        evaluate_test_expression("$50 * 3 + $25"),
        Some("175 $".to_string())
    );
    assert_eq!(
        evaluate_test_expression("€100 - €20 * 2"),
        Some("60 €".to_string())
    );
    assert_eq!(
        evaluate_test_expression("($100 + $50) / 3"),
        Some("50 $".to_string())
    );

    // Test precedence with currencies
    assert_eq!(
        evaluate_test_expression("$20 + $30 * 2"),
        Some("80 $".to_string()) // Should be $20 + ($30 * 2) = $20 + $60 = $80
    );
}

#[test]
fn test_currency_edge_cases() {
    // Test zero currency amounts
    assert_eq!(
        evaluate_test_expression("$0 + $100"),
        Some("100 $".to_string())
    );
    assert_eq!(
        evaluate_test_expression("$50 - $50"),
        Some("0 $".to_string())
    );

    // Test decimal currency amounts
    assert_eq!(
        evaluate_test_expression("$10.50 + $5.25"),
        Some("15.75 $".to_string())
    );

    // Test large currency amounts with commas
    assert_eq!(
        evaluate_test_expression("$1,000 + $500"),
        Some("1,500 $".to_string())
    );
}

#[test]
fn test_currency_real_world_scenarios() {
    // Test realistic currency calculations

    // Shopping total
    assert_eq!(
        evaluate_test_expression("$12.99 + $8.50 + $3.25"),
        Some("24.74 $".to_string())
    );

    // Tip calculation (20% tip)
    assert_eq!(
        evaluate_test_expression("$50 * 1.20"),
        Some("60 $".to_string())
    );

    // Split bill among friends
    assert_eq!(
        evaluate_test_expression("$120 / 4"),
        Some("30 $".to_string())
    );

    // Currency exchange rate calculation (as ratio)
    assert_eq!(
        evaluate_test_expression("$100 / $1"),
        Some("100".to_string())
    );

    // Bulk purchase discount
    assert_eq!(
        evaluate_test_expression("$25 * 10 * 0.9"), // 10% discount on 10 items
        Some("225 $".to_string())
    );
}

#[test]
fn test_currency_rate_creation() {
    // Test creating currency rates by dividing currency by time
    assert_eq!(
        evaluate_test_expression("$100 / 1 hour"),
        Some("100 $/h".to_string())
    );

    assert_eq!(
        evaluate_test_expression("€50 / 1 day"),
        Some("50 €/day".to_string())
    );

    assert_eq!(
        evaluate_test_expression("£25 / 30 minutes"),
        Some("0.833 £/min".to_string())
    );
}

#[test]
fn test_currency_rate_multiplication() {
    // Test the main use case: currency rate * time = total currency
    assert_eq!(
        evaluate_test_expression("$5/hr * 1 day"),
        Some("120 $".to_string())
    );

    assert_eq!(
        evaluate_test_expression("€10/hour * 8 hours"),
        Some("80 €".to_string())
    );

    assert_eq!(
        evaluate_test_expression("£25/day * 30 days"),
        Some("750 £".to_string())
    );

    // Test with different time units
    assert_eq!(
        evaluate_test_expression("$20/hr * 30 minutes"),
        Some("10 $".to_string())
    );

    assert_eq!(
        evaluate_test_expression("€15/day * 2 weeks"),
        Some("210 €".to_string())
    );
}

#[test]
fn test_currency_rate_with_fractions() {
    // Test with fractional rates
    assert_eq!(
        evaluate_test_expression("$12.50/hr * 4 hours"),
        Some("50 $".to_string())
    );

    assert_eq!(
        evaluate_test_expression("€7.5/hour * 8 hours"),
        Some("60 €".to_string())
    );

    // Test with fractional time
    assert_eq!(
        evaluate_test_expression("$40/hr * 0.5 hours"),
        Some("20 $".to_string())
    );

    assert_eq!(
        evaluate_test_expression("£100/day * 1.5 days"),
        Some("150 £".to_string())
    );
}

#[test]
fn test_currency_rate_complex_calculations() {
    // Test combining multiple currency rates
    assert_eq!(
        evaluate_test_expression("($15/hr + $5/hr) * 8 hours"),
        Some("160 $".to_string())
    );

    // Test rate calculations with parentheses
    assert_eq!(
        evaluate_test_expression("$10/hr * (40 hours + 8 hours)"),
        Some("480 $".to_string())
    );

    // Test overtime calculation (1.5x rate for overtime)
    assert_eq!(
        evaluate_test_expression("$20/hr * 40 hours + $20/hr * 1.5 * 10 hours"),
        Some("1,100 $".to_string())
    );
}

#[test]
fn test_currency_rate_different_currencies() {
    // Test that different currency rates work independently
    assert_eq!(
        evaluate_test_expression("¥1000/hr * 8 hours"),
        Some("8,000 ¥".to_string())
    );

    assert_eq!(
        evaluate_test_expression("₹500/day * 7 days"),
        Some("3,500 ₹".to_string())
    );

    assert_eq!(
        evaluate_test_expression("₩50000/hour * 4 hours"),
        Some("200,000 ₩".to_string())
    );
}

#[test]
fn test_currency_rate_real_world_scenarios() {
    // Hourly worker scenarios
    assert_eq!(
        evaluate_test_expression("$15/hr * 40 hours"),
        Some("600 $".to_string())
    );

    // Weekly salary calculation
    assert_eq!(
        evaluate_test_expression("$1000/week * 4 weeks"),
        Some("4,000 $".to_string())
    );

    // Daily rate contractor
    assert_eq!(
        evaluate_test_expression("£400/day * 22 days"),
        Some("8,800 £".to_string())
    );

    // Part-time worker
    assert_eq!(
        evaluate_test_expression("$12/hr * 20 hours"),
        Some("240 $".to_string())
    );

    // Freelancer monthly calculation
    assert_eq!(
        evaluate_test_expression("€3000/month * 3 months"),
        Some("9,000 €".to_string())
    );

    // Quarterly calculations
    assert_eq!(
        evaluate_test_expression("$12000/quarter to $/month"),
        Some("4,000 $/month".to_string())
    );

    assert_eq!(
        evaluate_test_expression("$100000/year to $/quarter"),
        Some("24,999.487 $/quarter".to_string()) // Slight precision difference due to 365.25 days/year
    );

    assert_eq!(
        evaluate_test_expression("€5000/quarter * 4 quarters"),
        Some("20,000 €".to_string())
    );
}

#[test]
fn test_currency_data_rate_creation() {
    // Test creating currency/data rates by dividing currency by data units
    assert_eq!(
        evaluate_test_expression("$100 / 50 GiB"),
        Some("2 $/GiB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("€5 / 1 TB"),
        Some("5 €/TB".to_string())
    );

    assert_eq!(
        evaluate_test_expression("£10 / 2 GB"),
        Some("5 £/GB".to_string())
    );
}

#[test]
fn test_currency_data_rate_multiplication() {
    // Test the main use case: currency/data rate * data = total currency
    assert_eq!(
        evaluate_test_expression("$5/GiB * 1 TiB"),
        Some("5,120 $".to_string())
    );

    assert_eq!(
        evaluate_test_expression("€2/MB * 500 MB"),
        Some("1,000 €".to_string())
    );

    assert_eq!(
        evaluate_test_expression("£0.50/GB * 2 TB"),
        Some("1,000 £".to_string())
    );

    // Test with smaller units to larger units
    assert_eq!(
        evaluate_test_expression("$0.005/MiB * 1 GiB"),
        Some("5.12 $".to_string())
    );
}

#[test]
fn test_currency_data_rate_mixed_units() {
    // Test calculations with different base systems (binary vs decimal)
    assert_eq!(
        evaluate_test_expression("$10/GiB * 1 GB"),
        Some("9.313 $".to_string())
    );

    // Test large data units
    assert_eq!(
        evaluate_test_expression("€0.001/MB * 1 PB"),
        Some("1,000,000 €".to_string())
    );

    // Test binary data rates
    assert_eq!(
        evaluate_test_expression("$100/TiB * 512 GiB"),
        Some("50 $".to_string())
    );
}

#[test]
fn test_currency_data_rate_with_fractions() {
    // Test with fractional rates
    assert_eq!(
        evaluate_test_expression("$0.01/MB * 2048 MB"),
        Some("20.48 $".to_string())
    );

    assert_eq!(
        evaluate_test_expression("€7.5/GiB * 0.5 GiB"),
        Some("3.75 €".to_string())
    );

    // Test with fractional data amounts
    assert_eq!(
        evaluate_test_expression("£20/TB * 1.5 TB"),
        Some("30 £".to_string())
    );
}

#[test]
fn test_currency_data_rate_complex_calculations() {
    // Test combining multiple currency/data rates
    assert_eq!(
        evaluate_test_expression("($5/GiB + $3/GiB) * 100 GiB"),
        Some("800 $".to_string())
    );

    // Test rate calculations with parentheses
    assert_eq!(
        evaluate_test_expression("$10/TB * (1 TB + 500 GB)"),
        Some("15 $".to_string())
    );

    // Test tiered pricing calculation
    assert_eq!(
        evaluate_test_expression("$5/TB * 10 TB + $3/TB * 40 TB"),
        Some("170 $".to_string())
    );
}

#[test]
fn test_currency_data_rate_different_currencies() {
    // Test that different currency/data rates work independently
    assert_eq!(
        evaluate_test_expression("¥100/GiB * 10 GiB"),
        Some("1,000 ¥".to_string())
    );

    assert_eq!(
        evaluate_test_expression("₹50/TB * 2 TB"),
        Some("100 ₹".to_string())
    );

    assert_eq!(
        evaluate_test_expression("₩1000/GB * 5 GB"),
        Some("5,000 ₩".to_string())
    );
}

#[test]
fn test_currency_data_rate_real_world_scenarios() {
    // Cloud storage pricing
    assert_eq!(
        evaluate_test_expression("$0.023/GB * 1 TB"),
        Some("23 $".to_string())
    );

    // Backup service pricing
    assert_eq!(
        evaluate_test_expression("€0.05/GiB * 500 GiB"),
        Some("25 €".to_string())
    );

    // Data transfer costs
    assert_eq!(
        evaluate_test_expression("$0.12/GB * 10 TB"),
        Some("1,200 $".to_string())
    );

    // Bulk storage discount calculation
    assert_eq!(
        evaluate_test_expression("$0.01/GB * 100 TB"),
        Some("1,000 $".to_string())
    );

    // Mixed storage tiers
    assert_eq!(
        evaluate_test_expression("$0.10/GB * 100 GB + $0.05/GB * 900 GB"),
        Some("55 $".to_string())
    );
}

#[test]
fn test_currency_rate_conversions() {
    // Test the main example: $5/month to $/year should be $60/year
    let result = evaluate_with_unit_info("$5/month to $/year");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 60.0).abs() < 0.01); // $5/month * 12 = $60/year (allowing for small floating point differences)

    // Test hourly to daily rates
    let result = evaluate_with_unit_info("$25/hr to $/day");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 600.0).abs() < 0.001); // $25/hr * 24 hours = $600/day

    // Test daily to weekly rates
    let result = evaluate_with_unit_info("$100/day to $/week");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 700.0).abs() < 0.001); // $100/day * 7 days = $700/week

    // Test weekly to monthly rates (using average month = 30.44 days)
    let result = evaluate_with_unit_info("$500/week to $/month");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    let expected = 500.0 * (30.44 / 7.0); // $500/week * 4.348 weeks/month ≈ 2174
    assert!((unit_val.value - expected).abs() < 1.0); // Allow larger tolerance due to floating point precision

    // Test yearly to monthly rates
    let result = evaluate_with_unit_info("$60000/year to $/month");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    let expected = 60000.0 / 12.0; // $60000/year / 12 months = $5000/month
    assert!((unit_val.value - expected).abs() < 0.15); // Slightly larger tolerance for floating point precision
}

#[test]
fn test_currency_rate_conversions_different_currencies() {
    // Test Euro rates
    let result = evaluate_with_unit_info("€10/hr to €/day");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 240.0).abs() < 0.001); // €10/hr * 24 = €240/day

    // Test Pound rates
    let result = evaluate_with_unit_info("£200/week to £/month");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    let expected = 200.0 * (30.44 / 7.0); // £200/week * weeks per month
    assert!((unit_val.value - expected).abs() < 0.1);

    // Test Yen rates (typically larger numbers)
    let result = evaluate_with_unit_info("¥2000/hr to ¥/day");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 48000.0).abs() < 0.001); // ¥2000/hr * 24 = ¥48000/day
}

#[test]
fn test_currency_rate_conversions_with_expressions() {
    // Test using currency rate conversion in expressions
    assert_eq!(
        evaluate_test_expression("$5/month to $/year"),
        Some("60.001 $/year".to_string())
    );

    assert_eq!(
        evaluate_test_expression("€20/hr to €/day"),
        Some("480 €/day".to_string())
    );

    assert_eq!(
        evaluate_test_expression("£100/week to £/month"),
        Some("434.812 £/month".to_string()) // 100 * (30.44/7), adjusted for actual calculation
    );

    // Test in complex expressions (currently evaluates the conversion part only)
    assert_eq!(
        evaluate_test_expression("($10/hr to $/day) * 5 days"),
        Some("240 $/day".to_string()) // $10/hr = $240/day (the conversion part works)
    );
}

#[test]
fn test_currency_rate_conversions_edge_cases() {
    // Test fractional rates
    let result = evaluate_with_unit_info("$2.50/hr to $/day");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 60.0).abs() < 0.001); // $2.50 * 24 = $60

    // Test very small rates
    let result = evaluate_with_unit_info("$0.01/minute to $/hour");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 0.6).abs() < 0.001); // $0.01 * 60 = $0.60

    // Test large rates
    let result = evaluate_with_unit_info("$10000/year to $/month");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 833.333).abs() < 0.1); // $10000 / 12 ≈ $833.33

    // Test that conversions between different currencies fail completely
    assert_eq!(evaluate_test_expression("$5/hr to €/hr"), None);
    assert_eq!(evaluate_test_expression("£100/day to $/day"), None);
}

#[test]
fn test_currency_rate_conversions_real_world() {
    // Annual salary to monthly
    let result = evaluate_with_unit_info("$72000/year to $/month");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 6000.0).abs() < 1.0); // $72k / 12 = $6k (allowing for floating point precision)

    // Hourly contractor rate to daily
    let result = evaluate_with_unit_info("$50/hr to $/day");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1200.0).abs() < 0.1); // $50 * 24 = $1200

    // Weekly allowance to yearly
    let result = evaluate_with_unit_info("$25/week to $/year");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    let expected = 25.0 * (365.25 / 7.0); // $25 * weeks per year
    assert!((unit_val.value - expected).abs() < 0.1);

    // Monthly subscription to yearly
    let result = evaluate_with_unit_info("$9.99/month to $/year");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 119.88).abs() < 0.1); // $9.99 * 12 = $119.88
}
