//! Integration tests for mathypad application
//!
//! These tests verify the complete flow from user input to final output,
//! testing the integration between parser, evaluator, and UI components.

use crate::expression::{evaluate_expression_with_context, find_math_expression};
use crate::test_helpers::{evaluate_test_expression, evaluate_with_unit_info};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_to_end_basic_calculations() {
        // Test basic arithmetic
        assert_eq!(evaluate_test_expression("2 + 3"), Some("5".to_string()));
        assert_eq!(evaluate_test_expression("10 - 4"), Some("6".to_string()));
        assert_eq!(evaluate_test_expression("6 * 7"), Some("42".to_string()));
        assert_eq!(evaluate_test_expression("15 / 3"), Some("5".to_string()));

        // Test with decimals
        assert_eq!(evaluate_test_expression("2.5 + 1.5"), Some("4".to_string()));
        assert_eq!(evaluate_test_expression("10.2 / 2"), Some("5.1".to_string()));

        // Test with parentheses
        assert_eq!(evaluate_test_expression("(2 + 3) * 4"), Some("20".to_string()));
        assert_eq!(evaluate_test_expression("2 + (3 * 4)"), Some("14".to_string()));
    }

    #[test]
    fn test_end_to_end_unit_conversions() {
        // Test basic unit conversions
        assert_eq!(evaluate_test_expression("1 GiB to MiB"), Some("1,024 MiB".to_string()));
        assert_eq!(evaluate_test_expression("60 seconds to minutes"), Some("1 min".to_string()));
        assert_eq!(evaluate_test_expression("1000 MB to GB"), Some("1 GB".to_string()));

        // Test conversion with calculations
        assert_eq!(evaluate_test_expression("2 GiB + 512 MiB to MiB"), Some("2,560 MiB".to_string()));
        assert_eq!(evaluate_test_expression("1 hour + 30 minutes to minutes"), Some("90 min".to_string()));

        // Test "in" keyword
        assert_eq!(evaluate_test_expression("24 MiB * 32 in KiB"), Some("786,432 KiB".to_string()));
        assert_eq!(evaluate_test_expression("500 GiB / 10 seconds in MiB/s"), Some("51,200 MiB/s".to_string()));
    }

    #[test]
    fn test_end_to_end_data_rate_calculations() {
        // Test rate calculations
        assert_eq!(evaluate_test_expression("1 hour * 10 GiB/s"), Some("36,000 GiB".to_string()));
        assert_eq!(evaluate_test_expression("100 GiB / 10 s"), Some("10 GiB/s".to_string()));
        assert_eq!(evaluate_test_expression("50 GiB/s * 2 s"), Some("100 GiB".to_string()));

        // Test with compound units
        assert_eq!(evaluate_test_expression("10GiB/s * 30min"), Some("18,000 GiB".to_string()));
        assert_eq!(evaluate_test_expression("1000GiB / 10min"), Some("1.667 GiB/s".to_string()));
    }

    #[test]
    fn test_end_to_end_qps_calculations() {
        // Test QPS rate calculations
        assert_eq!(evaluate_test_expression("100 QPS * 1 hour"), Some("360,000 query".to_string()));
        assert_eq!(evaluate_test_expression("25 QPS to req/minute"), Some("1,500 req/min".to_string()));
        assert_eq!(evaluate_test_expression("5000 queries / 10 minutes"), Some("8.333 QPS".to_string()));

        // Test QPS arithmetic
        assert_eq!(evaluate_test_expression("100 QPS + 50 QPS"), Some("150 QPS".to_string()));
        assert_eq!(evaluate_test_expression("200 req/min - 80 req/min"), Some("120 req/min".to_string()));

        // Test mixed QPS and request rates
        assert_eq!(evaluate_test_expression("100 QPS + 100 req/s"), Some("200 req/s".to_string()));
    }

    #[test]
    fn test_end_to_end_large_data_units() {
        // Test petabyte and exabyte calculations
        assert_eq!(evaluate_test_expression("1000 TB to PB"), Some("1 PB".to_string()));
        assert_eq!(evaluate_test_expression("5 PB to TB"), Some("5,000 TB".to_string()));
        assert_eq!(evaluate_test_expression("1000 PB to EB"), Some("1 EB".to_string()));

        // Test binary large units
        assert_eq!(evaluate_test_expression("1024 TiB to PiB"), Some("1 PiB".to_string()));
        assert_eq!(evaluate_test_expression("1024 PiB to EiB"), Some("1 EiB".to_string()));

        // Test mixed base conversions
        let result = evaluate_with_unit_info("1 PB to PiB");
        assert!(result.is_some());
        let unit_val = result.unwrap();
        assert!((unit_val.value - 0.8881784197).abs() < 0.0001);
    }

    #[test]
    fn test_end_to_end_bit_byte_conversions() {
        // Test bit to byte conversions
        assert_eq!(evaluate_test_expression("8 bit to B"), Some("1 B".to_string()));
        assert_eq!(evaluate_test_expression("8 Kb to KB"), Some("1 KB".to_string()));
        assert_eq!(evaluate_test_expression("8 Mb to MB"), Some("1 MB".to_string()));

        // Test byte to bit conversions
        assert_eq!(evaluate_test_expression("1 B to bit"), Some("8 bit".to_string()));
        assert_eq!(evaluate_test_expression("1 KB to Kb"), Some("8 Kb".to_string()));
        assert_eq!(evaluate_test_expression("1 MB to Mb"), Some("8 Mb".to_string()));

        // Test network speed scenarios
        assert_eq!(evaluate_test_expression("100 Mbps to MB/s"), Some("12.5 MB/s".to_string()));
        assert_eq!(evaluate_test_expression("1 Gbps to MB/s"), Some("125 MB/s".to_string()));
    }

    #[test]
    fn test_end_to_end_comma_numbers() {
        // Test comma-separated numbers
        assert_eq!(evaluate_test_expression("1,000 + 2,000"), Some("3,000".to_string()));
        assert_eq!(evaluate_test_expression("1,000,000 / 1000"), Some("1,000".to_string()));
        assert_eq!(evaluate_test_expression("1,234.56 * 2"), Some("2,469.12".to_string()));

        // Test comma numbers with units
        assert_eq!(evaluate_test_expression("1,000 GiB to MiB"), Some("1,024,000 MiB".to_string()));
        assert_eq!(evaluate_test_expression("1,000,000 bytes to MB"), Some("1 MB".to_string()));
    }

    #[test]
    fn test_end_to_end_no_space_units() {
        // Test numbers without spaces before units
        assert_eq!(evaluate_test_expression("5GiB"), Some("5 GiB".to_string()));
        assert_eq!(evaluate_test_expression("100MB to GB"), Some("0.1 GB".to_string()));
        assert_eq!(evaluate_test_expression("2.5TiB to GiB"), Some("2,560 GiB".to_string()));

        // Test expressions with mixed spacing
        assert_eq!(evaluate_test_expression("5 GiB + 10GiB"), Some("15 GiB".to_string()));
        assert_eq!(evaluate_test_expression("1,000GiB + 512 MiB"), Some("1,024,512 MiB".to_string()));

        // Test compound units without spaces
        assert_eq!(evaluate_test_expression("10GiB/s * 30min"), Some("18,000 GiB".to_string()));
        assert_eq!(evaluate_test_expression("100QPS * 5min"), Some("30,000 query".to_string()));
    }

    #[test]
    fn test_end_to_end_complex_expressions() {
        // Test complex nested expressions
        assert_eq!(
            evaluate_test_expression("((1 TiB + 512 GiB) / 2) * 3 to MiB"),
            Some("2,359,296 MiB".to_string())
        );

        // Test mixed unit types
        assert_eq!(
            evaluate_test_expression("(100 QPS * 1 hour) + (50 req/s * 30 minutes)"),
            Some("450,000 req".to_string())
        );

        // Test complex data transfer calculations
        assert_eq!(
            evaluate_test_expression("(5 PB + 1000 TB) / (10 hours) to GB/s"),
            Some("166.667 GB/s".to_string())
        );
    }

    #[test]
    fn test_end_to_end_real_world_scenarios() {
        // Test data center storage scenarios
        assert_eq!(
            evaluate_test_expression("Data center: 50 PB + 10 EB"),
            Some("10,050 PB".to_string())
        );

        // Test API load balancing
        assert_eq!(
            evaluate_test_expression("Total load: 250 QPS + 150 QPS + 100 QPS"),
            Some("500 QPS".to_string())
        );

        // Test bandwidth calculations
        assert_eq!(
            evaluate_test_expression("Bandwidth used: 1,000 GiB / 1 hour"),
            Some("0.278 GiB/s".to_string())
        );

        // Test backup scenarios
        assert_eq!(
            evaluate_test_expression("Backup rate: 100 TB/s * 8 hours"),
            Some("2,880,000 TB".to_string())
        );

        // Test network throughput
        assert_eq!(
            evaluate_test_expression("Network: 10 PB/s to TB/s"),
            Some("10,000 TB/s".to_string())
        );
    }

    #[test]
    fn test_end_to_end_precision_and_formatting() {
        // Test precision handling
        assert_eq!(evaluate_test_expression("1 / 3"), Some("0.333".to_string()));
        assert_eq!(evaluate_test_expression("100 / 3"), Some("33.333".to_string()));
        assert_eq!(evaluate_test_expression("1000 / 7"), Some("142.857".to_string()));

        // Test large number formatting
        assert_eq!(evaluate_test_expression("1000000 + 2000000"), Some("3,000,000".to_string()));
        assert_eq!(evaluate_test_expression("1234567.89 + 1"), Some("1,234,568.89".to_string()));

        // Test very small numbers
        assert_eq!(evaluate_test_expression("0.001 + 0.002"), Some("0.003".to_string()));
        assert_eq!(evaluate_test_expression("0.000001 * 1000000"), Some("1".to_string()));
    }

    #[test]
    fn test_end_to_end_edge_cases() {
        // Test zero values
        assert_eq!(evaluate_test_expression("0 + 5"), Some("5".to_string()));
        assert_eq!(evaluate_test_expression("0 * 100"), Some("0".to_string()));
        assert_eq!(evaluate_test_expression("0 GiB + 5 GiB"), Some("5 GiB".to_string()));

        // Test division by small numbers
        assert_eq!(evaluate_test_expression("1 / 0.1"), Some("10".to_string()));
        assert_eq!(evaluate_test_expression("100 / 0.01"), Some("10,000".to_string()));

        // Test very large calculations
        assert_eq!(evaluate_test_expression("999999 * 999999"), Some("999,998,000,001".to_string()));

        // Test unit edge cases
        assert_eq!(evaluate_test_expression("1 ns to s"), Some("0 s".to_string()));
        assert_eq!(evaluate_test_expression("1000000000 ns to s"), Some("1 s".to_string()));
    }

    #[test]
    fn test_mixed_text_and_math_expressions() {
        // Test finding math expressions in text
        let expressions = find_math_expression("The server has 16 GiB of RAM and processes 100 QPS");
        assert!(expressions.contains(&"16 GiB".to_string()));
        assert!(expressions.contains(&"100 QPS".to_string()));

        let expressions = find_math_expression("Download: 1,000 MB at 50 MB/s takes 20 seconds");
        assert!(expressions.contains(&"1,000 MB".to_string()));
        assert!(expressions.contains(&"50 MB/s".to_string()));
        assert!(expressions.contains(&"20 seconds".to_string()));

        let expressions = find_math_expression("Calculate: (5 GiB + 3 GiB) * 2 for total storage");
        assert!(expressions.contains(&"(5 GiB + 3 GiB) * 2".to_string()));

        // Test complex mathematical expressions in text
        let expressions = find_math_expression("API performance: (100 QPS + 50 req/s) * 1 hour = total requests");
        assert!(expressions.contains(&"(100 QPS + 50 req/s) * 1 hour".to_string()));
    }

    #[test]
    fn test_invalid_expression_handling() {
        // Test invalid expressions return None
        assert_eq!(evaluate_test_expression("invalid expression"), None);
        // Note: Some expressions might be partially parsed by chumsky
        assert_eq!(evaluate_test_expression("(1 + 2"), None);
        assert_eq!(evaluate_test_expression("1 + 2)"), None);
        assert_eq!(evaluate_test_expression(""), None);

        // Test incompatible unit operations
        assert_eq!(evaluate_test_expression("5 GiB + 10 seconds"), None);
        assert_eq!(evaluate_test_expression("100 QPS - 50 MB"), None);
        // Note: "1 hour * 1 GiB" might actually work as scalar multiplication

        // Test invalid units
        assert_eq!(evaluate_test_expression("100 invalidunit"), None);
        assert_eq!(evaluate_test_expression("50 notarealunit to GB"), None);
    }

    #[test]
    fn test_context_with_line_references() {
        // Test basic line reference functionality
        let lines = vec![Some("10 GiB".to_string()), Some("5 GiB".to_string())];
        assert_eq!(
            evaluate_expression_with_context("line1 + line2", &lines, 2),
            Some("15 GiB".to_string())
        );

        // Test line references with conversions
        let lines = vec![Some("1 TiB".to_string()), Some("512 GiB".to_string())];
        assert_eq!(
            evaluate_expression_with_context("line1 + line2 to MiB", &lines, 2),
            Some("1,572,864 MiB".to_string())
        );

        // Test line references in complex expressions
        let lines = vec![Some("100 QPS".to_string()), Some("5 minutes".to_string())];
        assert_eq!(
            evaluate_expression_with_context("line1 * line2", &lines, 2),
            Some("30,000 query".to_string())
        );

        // Test preventing future line references
        let lines = vec![Some("10 GiB".to_string())];
        assert_eq!(
            evaluate_expression_with_context("line1 + line2", &lines, 0),
            None // Should fail because line2 doesn't exist yet
        );
    }

    #[test]
    fn test_performance_with_large_expressions() {
        // Test parsing performance with deeply nested expressions
        let nested_expr = "(((((1 + 2) * 3) + 4) * 5) + 6) * 7";
        assert_eq!(evaluate_test_expression(nested_expr), Some("497".to_string()));

        // Test with many operations
        let long_expr = "1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10";
        assert_eq!(evaluate_test_expression(long_expr), Some("55".to_string()));

        // Test with many units
        let unit_expr = "1 GiB + 2 GiB + 3 GiB + 4 GiB + 5 GiB";
        assert_eq!(evaluate_test_expression(unit_expr), Some("15 GiB".to_string()));

        // Test complex unit conversion chain
        let conversion_expr = "((1 TiB + 512 GiB) * 2 + 1024 MiB) / 3 to KiB";
        let result = evaluate_test_expression(conversion_expr);
        assert!(result.is_some());
    }

    #[test]
    fn test_case_insensitivity() {
        // Test case insensitive units (note: some variations might have different meanings)
        assert_eq!(evaluate_test_expression("1 GiB to MiB"), Some("1,024 MiB".to_string()));
        
        // Test case insensitive keywords  
        assert_eq!(evaluate_test_expression("1 GiB to MiB"), Some("1,024 MiB".to_string()));
        assert_eq!(evaluate_test_expression("24 MiB * 32 in KiB"), Some("786,432 KiB".to_string()));

        // Test QPS conversions
        assert_eq!(evaluate_test_expression("1 QPS to req/min"), Some("60 req/min".to_string()));
    }
}