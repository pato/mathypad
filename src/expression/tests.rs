//! Tests for expression parsing and evaluation

use super::*;
use crate::test_helpers::*;
use crate::units::Unit;

#[test]
fn test_basic_arithmetic() {
    // Basic operations
    assert_eq!(evaluate_test_expression("2 + 3"), Some("5".to_string()));
    assert_eq!(evaluate_test_expression("10 - 4"), Some("6".to_string()));
    assert_eq!(evaluate_test_expression("6 * 7"), Some("42".to_string()));
    assert_eq!(evaluate_test_expression("15 / 3"), Some("5".to_string()));

    // Order of operations
    assert_eq!(
        evaluate_test_expression("2 + 3 * 4"),
        Some("14".to_string())
    );
    assert_eq!(
        evaluate_test_expression("(2 + 3) * 4"),
        Some("20".to_string())
    );
    assert_eq!(
        evaluate_test_expression("10 - 2 * 3"),
        Some("4".to_string())
    );

    // Decimal numbers
    assert_eq!(evaluate_test_expression("1.5 + 2.5"), Some("4".to_string()));
    assert_eq!(
        evaluate_test_expression("3.14 * 2"),
        Some("6.28".to_string())
    );

    // Numbers with commas
    assert_eq!(
        evaluate_test_expression("1,000 + 500"),
        Some("1,500".to_string())
    );
    assert_eq!(
        evaluate_test_expression("1,234,567 / 1000"),
        Some("1,234.567".to_string())
    );
}

#[test]
fn test_inline_expressions() {
    // Test expressions within text
    assert_eq!(
        evaluate_test_expression("The result is 5 + 3"),
        Some("8".to_string())
    );
    assert_eq!(
        evaluate_test_expression("Cost: 100 * 12 dollars"),
        Some("1,200".to_string())
    );
    assert_eq!(
        evaluate_test_expression("Total (10 + 20) items"),
        Some("30".to_string())
    );
}

#[test]
fn test_complex_expressions() {
    // Complex arithmetic
    assert_eq!(
        evaluate_test_expression("(10 + 5) * 2 - 8 / 4"),
        Some("28".to_string())
    );
    assert_eq!(
        evaluate_test_expression("100 / (5 + 5) + 3 * 2"),
        Some("16".to_string())
    );

    // Large numbers with commas
    assert_eq!(
        evaluate_test_expression("1,000,000 + 500,000"),
        Some("1,500,000".to_string())
    );
    assert_eq!(
        evaluate_test_expression("2,500 * 1,000"),
        Some("2,500,000".to_string())
    );

    // Complex unit expressions
    assert_eq!(
        evaluate_test_expression("Transfer: 5 GiB/s * 10 minutes"),
        Some("3,000 GiB".to_string())
    );
}

#[test]
fn test_edge_cases() {
    // Division by zero
    println!("Testing 5 / 0: {:?}", evaluate_test_expression("5 / 0"));
    assert_eq!(evaluate_test_expression("5 / 0"), None);

    // Invalid expressions
    let expressions = find_math_expression("5 +");
    println!("Found expressions for '5 +': {:?}", expressions);
    for expr in &expressions {
        println!(
            "Expression '{}' is valid: {}",
            expr,
            is_valid_math_expression(expr)
        );
    }
    println!("Testing 5 +: {:?}", evaluate_test_expression("5 +"));
    assert_eq!(evaluate_test_expression("5 +"), None);
    assert_eq!(evaluate_test_expression("* 5"), None);
    assert_eq!(evaluate_test_expression("((5)"), None);

    // Empty or invalid input
    assert_eq!(evaluate_test_expression(""), None);
    assert_eq!(evaluate_test_expression("hello world"), None);

    // Incompatible unit operations
    assert_eq!(evaluate_test_expression("5 GiB + 10 seconds"), None);
    assert_eq!(evaluate_test_expression("1 hour - 500 MB"), None);
}

#[test]
fn test_precision() {
    // Test decimal precision
    assert_eq!(
        evaluate_test_expression("1.234 + 2.567"),
        Some("3.801".to_string())
    );
    assert_eq!(
        evaluate_test_expression("10.5 / 3"),
        Some("3.5".to_string())
    );

    // Test with units requiring precision
    let result = evaluate_with_unit_info("1.5 GiB to MiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1536.0).abs() < 0.001);
}

#[test]
fn test_whitespace_handling() {
    // Various whitespace formats
    assert_eq!(evaluate_test_expression("5+3"), Some("8".to_string()));
    assert_eq!(
        evaluate_test_expression("  5  +  3  "),
        Some("8".to_string())
    );
    assert_eq!(evaluate_test_expression("5 * 3"), Some("15".to_string()));

    // Units with whitespace
    let result = evaluate_with_unit_info("1   GiB   to   KiB");
    assert!(result.is_some());
    let unit_val = result.unwrap();
    assert!((unit_val.value - 1048576.0).abs() < 0.001);
}

#[test]
fn test_line_references() {
    // Test parsing line references
    assert_eq!(parse_line_reference("line1"), Some(0));
    assert_eq!(parse_line_reference("line5"), Some(4));
    assert_eq!(parse_line_reference("line123"), Some(122));
    assert_eq!(parse_line_reference("Line1"), Some(0)); // Case insensitive
    assert_eq!(parse_line_reference("LINE1"), Some(0)); // Case insensitive

    // Test invalid line references
    assert_eq!(parse_line_reference("line0"), None); // Line numbers start at 1
    assert_eq!(parse_line_reference("line"), None); // No number
    assert_eq!(parse_line_reference("lineabc"), None); // Invalid number
    assert_eq!(parse_line_reference("myline1"), None); // Doesn't start with "line"
    assert_eq!(parse_line_reference("1line"), None); // Doesn't start with "line"

    // Test line reference resolution with context
    let previous_results = vec![
        Some("10 GiB".to_string()),
        Some("5".to_string()),
        None,
        Some("1,024 MiB".to_string()),
    ];

    // Test valid line references
    assert_eq!(
        evaluate_expression_with_context("line1 + 4 GiB", &previous_results, 4),
        Some("14 GiB".to_string())
    );
    assert_eq!(
        evaluate_expression_with_context("line2 * 3", &previous_results, 4),
        Some("15".to_string())
    );
    assert_eq!(
        evaluate_expression_with_context("line4 to GiB", &previous_results, 4),
        Some("1 GiB".to_string())
    );

    // Test circular reference prevention
    assert_eq!(
        evaluate_expression_with_context("line1 + 2", &previous_results, 0),
        None
    ); // Can't reference self
    assert_eq!(
        evaluate_expression_with_context("line5 + 2", &previous_results, 4),
        None
    ); // Can't reference future lines

    // Test reference to line with no result
    assert_eq!(
        evaluate_expression_with_context("line3 + 5", &previous_results, 4),
        None
    ); // Line 3 has no result

    // Test complex expressions with line references
    assert_eq!(
        evaluate_expression_with_context("(line1 + line4) / 2", &previous_results, 4),
        Some("5,632 MiB".to_string())
    );
    assert_eq!(
        evaluate_expression_with_context("line1 * line2 to MiB", &previous_results, 4),
        Some("51,200 MiB".to_string())
    );
}

#[test]
fn test_line_reference_parsing_edge_cases() {
    // Test result string parsing
    assert!(parse_result_string("10 GiB").is_some());
    assert!(parse_result_string("1,024").is_some());
    assert!(parse_result_string("42").is_some());
    assert!(parse_result_string("3.14 MiB/s").is_some());

    // Test invalid result strings
    assert!(parse_result_string("").is_none());
    assert!(parse_result_string("invalid").is_none());
    assert!(parse_result_string("GiB 10").is_none()); // Wrong order

    // Test line reference in tokenizer
    let tokens = tokenize_with_units("line1 + 5 GiB").unwrap();
    assert!(matches!(tokens[0], Token::LineReference(0)));
    assert!(matches!(tokens[1], Token::Plus));
    assert!(matches!(tokens[2], Token::NumberWithUnit(5.0, Unit::GiB)));
}
