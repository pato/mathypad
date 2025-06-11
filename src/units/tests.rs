//! Tests for unit functionality

use super::*;
use crate::test_helpers::*;

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
    assert_eq!(parse_unit("qps"), Some(Unit::QueriesPerSecond));
    assert_eq!(parse_unit("QPS"), Some(Unit::QueriesPerSecond));
    assert_eq!(parse_unit("queries/s"), Some(Unit::QueriesPerSecond));
    assert_eq!(parse_unit("queries/sec"), Some(Unit::QueriesPerSecond));
    assert_eq!(parse_unit("qpm"), Some(Unit::QueriesPerMinute));
    assert_eq!(parse_unit("queries/min"), Some(Unit::QueriesPerMinute));
    assert_eq!(parse_unit("queries/minute"), Some(Unit::QueriesPerMinute));
    assert_eq!(parse_unit("qph"), Some(Unit::QueriesPerHour));
    assert_eq!(parse_unit("queries/h"), Some(Unit::QueriesPerHour));
    assert_eq!(parse_unit("queries/hour"), Some(Unit::QueriesPerHour));

    // Test request rate unit parsing
    assert_eq!(parse_unit("req/s"), Some(Unit::RequestsPerSecond));
    assert_eq!(parse_unit("requests/s"), Some(Unit::RequestsPerSecond));
    assert_eq!(parse_unit("rps"), Some(Unit::RequestsPerSecond));
    assert_eq!(parse_unit("req/min"), Some(Unit::RequestsPerMinute));
    assert_eq!(parse_unit("requests/min"), Some(Unit::RequestsPerMinute));
    assert_eq!(parse_unit("rpm"), Some(Unit::RequestsPerMinute));
    assert_eq!(parse_unit("req/h"), Some(Unit::RequestsPerHour));
    assert_eq!(parse_unit("req/hour"), Some(Unit::RequestsPerHour));
    assert_eq!(parse_unit("requests/h"), Some(Unit::RequestsPerHour));
    assert_eq!(parse_unit("requests/hour"), Some(Unit::RequestsPerHour));
    assert_eq!(parse_unit("rph"), Some(Unit::RequestsPerHour));

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
        Some("1 QPS".to_string())
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
        Some("8.333 QPS".to_string())
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
        Some("150 QPS".to_string())
    );

    assert_eq!(
        evaluate_test_expression("200 req/min - 80 req/min"),
        Some("120 req/min".to_string())
    );

    // Test adding different rate units (should convert to common base)
    assert_eq!(
        evaluate_test_expression("100 QPS + 60 QPM"),
        Some("6,060 QPM".to_string())
    );

    assert_eq!(
        evaluate_test_expression("3600 QPH - 30 QPM"),
        Some("1,800 QPH".to_string())
    );

    // Test mixed request rate families
    assert_eq!(
        evaluate_test_expression("100 QPS + 100 req/s"),
        Some("200 req/s".to_string())
    );

    assert_eq!(
        evaluate_test_expression("1000 req/min + 500 QPM"),
        Some("1,500 QPM".to_string())
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
        evaluate_test_expression("Daily requests: 86400 req / 1 day"),
        Some("1 req/s".to_string())
    );

    // Test load balancing scenarios
    assert_eq!(
        evaluate_test_expression("Total load: 250 QPS + 150 QPS + 100 QPS"),
        Some("500 QPS".to_string())
    );

    assert_eq!(
        evaluate_test_expression("Per server: 1500 QPS / 3"),
        Some("500 QPS".to_string())
    );

    // Test capacity planning
    assert_eq!(
        evaluate_test_expression("Monthly queries: 100 QPS * 30 days"),
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
        Some("0.1 QPS".to_string())
    );

    // Test very large QPS rates
    assert_eq!(
        evaluate_test_expression("1000000 QPS * 1 s"),
        Some("1,000,000 query".to_string())
    );

    // Test fractional results
    assert_eq!(
        evaluate_test_expression("100 QPS / 3"),
        Some("33.333 QPS".to_string())
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
    assert_eq!(Unit::QueriesPerSecond.display_name(), "QPS");
    assert_eq!(Unit::QueriesPerMinute.display_name(), "QPM");
    assert_eq!(Unit::QueriesPerHour.display_name(), "QPH");
    assert_eq!(Unit::RequestsPerSecond.display_name(), "req/s");
    assert_eq!(Unit::RequestsPerMinute.display_name(), "req/min");
    assert_eq!(Unit::RequestsPerHour.display_name(), "req/h");
    assert_eq!(Unit::Request.display_name(), "req");
    assert_eq!(Unit::Query.display_name(), "query");
}

#[test]
fn test_qps_unit_type_classification() {
    // Test that QPS units are properly classified
    assert_eq!(Unit::QueriesPerSecond.unit_type(), UnitType::RequestRate);
    assert_eq!(Unit::QueriesPerMinute.unit_type(), UnitType::RequestRate);
    assert_eq!(Unit::QueriesPerHour.unit_type(), UnitType::RequestRate);
    assert_eq!(Unit::RequestsPerSecond.unit_type(), UnitType::RequestRate);
    assert_eq!(Unit::RequestsPerMinute.unit_type(), UnitType::RequestRate);
    assert_eq!(Unit::RequestsPerHour.unit_type(), UnitType::RequestRate);
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
    assert_eq!(parse_unit("pb/s"), Some(Unit::PBPerSecond));
    assert_eq!(parse_unit("pbps"), Some(Unit::PBPerSecond));
    assert_eq!(parse_unit("eb/s"), Some(Unit::EBPerSecond));
    assert_eq!(parse_unit("ebps"), Some(Unit::EBPerSecond));
    assert_eq!(parse_unit("pib/s"), Some(Unit::PiBPerSecond));
    assert_eq!(parse_unit("pibps"), Some(Unit::PiBPerSecond));
    assert_eq!(parse_unit("eib/s"), Some(Unit::EiBPerSecond));
    assert_eq!(parse_unit("eibps"), Some(Unit::EiBPerSecond));
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

    // Test rate calculations with large units
    assert_eq!(
        evaluate_test_expression("1 PB / 1 hour"),
        Some("0 PB/s".to_string())
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
        Some("0.006 EiB/s".to_string())
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
    assert_eq!(Unit::PBPerSecond.display_name(), "PB/s");
    assert_eq!(Unit::EBPerSecond.display_name(), "EB/s");
    assert_eq!(Unit::PiBPerSecond.display_name(), "PiB/s");
    assert_eq!(Unit::EiBPerSecond.display_name(), "EiB/s");
}

#[test]
fn test_large_data_unit_type_classification() {
    // Test that large data units are properly classified
    assert_eq!(Unit::PB.unit_type(), UnitType::Data);
    assert_eq!(Unit::EB.unit_type(), UnitType::Data);
    assert_eq!(Unit::PiB.unit_type(), UnitType::Data);
    assert_eq!(Unit::EiB.unit_type(), UnitType::Data);

    // Test that large rate units are properly classified
    assert_eq!(Unit::PBPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::EBPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::PiBPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::EiBPerSecond.unit_type(), UnitType::DataRate);
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
    assert_eq!(parse_unit("bps"), Some(Unit::BitsPerSecond));
    assert_eq!(parse_unit("Kbps"), Some(Unit::KbPerSecond));
    assert_eq!(parse_unit("Mbps"), Some(Unit::MbPerSecond));
    assert_eq!(parse_unit("Gbps"), Some(Unit::GbPerSecond));
    assert_eq!(parse_unit("Kibps"), Some(Unit::KibPerSecond));
    assert_eq!(parse_unit("Mibps"), Some(Unit::MibPerSecond));
    assert_eq!(parse_unit("Gibps"), Some(Unit::GibPerSecond));

    // Test byte rate units (bytes per second)
    assert_eq!(parse_unit("B/s"), Some(Unit::BytesPerSecond));
    assert_eq!(parse_unit("KB/s"), Some(Unit::KBPerSecond));
    assert_eq!(parse_unit("MB/s"), Some(Unit::MBPerSecond));
    assert_eq!(parse_unit("GB/s"), Some(Unit::GBPerSecond));
    assert_eq!(parse_unit("KiB/s"), Some(Unit::KiBPerSecond));
    assert_eq!(parse_unit("MiB/s"), Some(Unit::MiBPerSecond));
    assert_eq!(parse_unit("GiB/s"), Some(Unit::GiBPerSecond));
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
        Some("400 Mbps".to_string())
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
    assert_eq!(Unit::BitsPerSecond.display_name(), "bps");
    assert_eq!(Unit::KbPerSecond.display_name(), "Kbps");
    assert_eq!(Unit::MbPerSecond.display_name(), "Mbps");
    assert_eq!(Unit::GbPerSecond.display_name(), "Gbps");
    assert_eq!(Unit::KibPerSecond.display_name(), "Kibps");
    assert_eq!(Unit::MibPerSecond.display_name(), "Mibps");
    assert_eq!(Unit::GibPerSecond.display_name(), "Gibps");

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
    assert_eq!(Unit::BitsPerSecond.unit_type(), UnitType::BitRate);
    assert_eq!(Unit::KbPerSecond.unit_type(), UnitType::BitRate);
    assert_eq!(Unit::MbPerSecond.unit_type(), UnitType::BitRate);
    assert_eq!(Unit::GbPerSecond.unit_type(), UnitType::BitRate);
    assert_eq!(Unit::KibPerSecond.unit_type(), UnitType::BitRate);
    assert_eq!(Unit::MibPerSecond.unit_type(), UnitType::BitRate);
    assert_eq!(Unit::GibPerSecond.unit_type(), UnitType::BitRate);

    // Test that byte units are still classified as Data type
    assert_eq!(Unit::Byte.unit_type(), UnitType::Data);
    assert_eq!(Unit::KB.unit_type(), UnitType::Data);
    assert_eq!(Unit::MB.unit_type(), UnitType::Data);
    assert_eq!(Unit::GB.unit_type(), UnitType::Data);
    assert_eq!(Unit::KiB.unit_type(), UnitType::Data);
    assert_eq!(Unit::MiB.unit_type(), UnitType::Data);
    assert_eq!(Unit::GiB.unit_type(), UnitType::Data);

    // Test that byte rate units are still classified as DataRate type
    assert_eq!(Unit::BytesPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::KBPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::MBPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::GBPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::KiBPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::MiBPerSecond.unit_type(), UnitType::DataRate);
    assert_eq!(Unit::GiBPerSecond.unit_type(), UnitType::DataRate);
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

    // Bandwidth calculations
    assert_eq!(
        evaluate_test_expression("Bandwidth used: 1,000 GiB / 1 hour"),
        Some("0.278 GiB/s".to_string())
    );

    // Data conversion scenarios
    let result = evaluate_with_unit_info("How many KiB in 5 MiB?");
    assert!(result.is_some()); // Will find "5 MiB" as a valid expression

    let result = evaluate_with_unit_info("5 MiB to KiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 5120.0).abs() < 0.001);
}
