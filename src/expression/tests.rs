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

    // Complex unit expressions with generic rate...
    assert_eq!(
        evaluate_test_expression("Transfer: 1 GiB/minute * 8 minutes"),
        Some("8 GiB".to_string())
    );

    // More generic rate tests
    assert_eq!(
        evaluate_test_expression("Backup speed: 500 MB/hour * 12 hours"),
        Some("6,000 MB".to_string())
    );
    assert_eq!(
        evaluate_test_expression("Download: 2.5 GiB/minute * 4 minutes"),
        Some("10 GiB".to_string())
    );
}

#[test]
fn test_edge_cases() {
    // Division by zero
    assert_eq!(evaluate_test_expression("5 / 0"), None);

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

#[test]
fn test_variable_assignments() {
    use std::collections::HashMap;

    // Test simple variable assignment
    let variables = HashMap::new();
    let previous_results = vec![];
    let (result, assignment) =
        evaluate_with_variables("servers = 40", &variables, &previous_results, 0);
    assert_eq!(result, Some("40".to_string()));
    assert_eq!(assignment, Some(("servers".to_string(), "40".to_string())));

    // Test variable assignment with units
    let (result, assignment) =
        evaluate_with_variables("ram = 1 TiB", &variables, &previous_results, 0);
    assert_eq!(result, Some("1 TiB".to_string()));
    assert_eq!(assignment, Some(("ram".to_string(), "1 TiB".to_string())));

    // Test variable assignment with expression
    let (result, assignment) =
        evaluate_with_variables("total = 10 + 20", &variables, &previous_results, 0);
    assert_eq!(result, Some("30".to_string()));
    assert_eq!(assignment, Some(("total".to_string(), "30".to_string())));

    // Test variable assignment with unit expression
    let (result, assignment) = evaluate_with_variables(
        "storage = 2 GiB + 512 MiB",
        &variables,
        &previous_results,
        0,
    );
    assert_eq!(result, Some("2,560 MiB".to_string()));
    assert_eq!(
        assignment,
        Some(("storage".to_string(), "2,560 MiB".to_string()))
    );
}

#[test]
fn test_variable_references() {
    use std::collections::HashMap;

    // Set up variables
    let mut variables = HashMap::new();
    variables.insert("servers".to_string(), "40".to_string());
    variables.insert("ram".to_string(), "1 TiB".to_string());
    variables.insert("speed".to_string(), "100 MB/s".to_string());

    let previous_results = vec![];

    // Test simple variable reference
    let (result, assignment) = evaluate_with_variables("servers", &variables, &previous_results, 0);
    assert_eq!(result, Some("40".to_string()));
    assert_eq!(assignment, None);

    // Test variable reference with unit
    let (result, assignment) = evaluate_with_variables("ram", &variables, &previous_results, 0);
    assert_eq!(result, Some("1 TiB".to_string()));
    assert_eq!(assignment, None);

    // Test variable arithmetic
    let (result, assignment) =
        evaluate_with_variables("servers * 2", &variables, &previous_results, 0);
    assert_eq!(result, Some("80".to_string()));
    assert_eq!(assignment, None);

    // Test variable with unit arithmetic
    let (result, assignment) =
        evaluate_with_variables("ram + 512 GiB", &variables, &previous_results, 0);
    assert_eq!(result, Some("1,536 GiB".to_string()));
    assert_eq!(assignment, None);

    // Test two variables together
    let (result, assignment) =
        evaluate_with_variables("servers * ram", &variables, &previous_results, 0);
    assert_eq!(result, Some("40 TiB".to_string()));
    assert_eq!(assignment, None);
}

#[test]
fn test_multiline_variable_scenario() {
    use std::collections::HashMap;

    // Simulate the multiline notebook scenario: servers = 40, ram = 1 TiB, servers * ram
    let mut variables = HashMap::new();
    let mut previous_results = vec![];

    // Line 1: servers = 40
    let (result1, assignment1) =
        evaluate_with_variables("servers = 40", &variables, &previous_results, 0);
    assert_eq!(result1, Some("40".to_string()));
    assert_eq!(assignment1, Some(("servers".to_string(), "40".to_string())));

    // Store the variable assignment
    if let Some((var_name, var_value)) = assignment1 {
        variables.insert(var_name, var_value);
    }
    previous_results.push(result1);

    // Line 2: ram = 1 TiB
    let (result2, assignment2) =
        evaluate_with_variables("ram = 1 TiB", &variables, &previous_results, 1);
    assert_eq!(result2, Some("1 TiB".to_string()));
    assert_eq!(assignment2, Some(("ram".to_string(), "1 TiB".to_string())));

    // Store the variable assignment
    if let Some((var_name, var_value)) = assignment2 {
        variables.insert(var_name, var_value);
    }
    previous_results.push(result2);

    // Line 3: servers * ram
    let (result3, assignment3) =
        evaluate_with_variables("servers * ram", &variables, &previous_results, 2);
    assert_eq!(result3, Some("40 TiB".to_string()));
    assert_eq!(assignment3, None); // No assignment, just evaluation
}

#[test]
fn test_variable_with_line_references() {
    use std::collections::HashMap;

    let mut variables = HashMap::new();
    variables.insert("multiplier".to_string(), "3".to_string());

    // Simulate previous line results
    let previous_results = vec![Some("10 GiB".to_string()), Some("5".to_string())];

    // Test variable with line reference
    let (result, assignment) =
        evaluate_with_variables("line1 * multiplier", &variables, &previous_results, 2);
    assert_eq!(result, Some("30 GiB".to_string()));
    assert_eq!(assignment, None);

    // Test assigning line reference to variable
    let (result, assignment) =
        evaluate_with_variables("backup = line1 + 5 GiB", &variables, &previous_results, 2);
    assert_eq!(result, Some("15 GiB".to_string()));
    assert_eq!(
        assignment,
        Some(("backup".to_string(), "15 GiB".to_string()))
    );
}

#[test]
fn test_variable_conversions() {
    use std::collections::HashMap;

    let mut variables = HashMap::new();
    variables.insert("storage".to_string(), "1024 GiB".to_string());
    variables.insert("time".to_string(), "8 minutes".to_string());

    let previous_results = vec![];

    // Test variable conversion
    let (result, assignment) =
        evaluate_with_variables("storage to TB", &variables, &previous_results, 0);
    assert_eq!(result, Some("1.1 TB".to_string()));
    assert_eq!(assignment, None);

    // Test variable in complex conversion expression with generic rates
    let (result, assignment) =
        evaluate_with_variables("storage / time", &variables, &previous_results, 0);
    assert_eq!(result, Some("128 GiB/min".to_string())); // Creates generic rate
    assert_eq!(assignment, None);
}

#[test]
fn test_variable_edge_cases() {
    use std::collections::HashMap;

    let variables = HashMap::new();
    let previous_results = vec![];

    // Test undefined variable
    let (result, assignment) =
        evaluate_with_variables("undefined_var + 5", &variables, &previous_results, 0);
    assert_eq!(result, None);
    assert_eq!(assignment, None);

    // Test variable name conflicts with units - now parses as [Unit, Assign, Number]
    // which doesn't match assignment pattern, so evaluates "GiB" as standalone unit
    let (result, assignment) = evaluate_with_variables("GiB = 5", &variables, &previous_results, 0);
    assert_eq!(result, Some("1 GiB".to_string())); // Evaluates "GiB" as standalone unit
    assert_eq!(assignment, None); // No variable assignment

    // Test variable name conflicts with keywords - this actually parses as [To, Assign, Number(10)]
    // which doesn't match variable assignment pattern but does parse the number 10
    let (result, assignment) = evaluate_with_variables("to = 10", &variables, &previous_results, 0);
    assert_eq!(result, Some("10".to_string())); // Parses the "10" part
    assert_eq!(assignment, None); // No variable assignment

    // Test variable name conflicts with line references - now parses as [LineReference, Assign, Number]
    // which doesn't match assignment pattern, so evaluates "20" from the expression
    let (result, assignment) =
        evaluate_with_variables("line1 = 20", &variables, &previous_results, 0);
    assert_eq!(result, Some("20".to_string())); // Evaluates "20" from the expression
    assert_eq!(assignment, None); // No variable assignment
}

#[test]
fn test_complex_variable_expressions() {
    use std::collections::HashMap;

    let mut variables = HashMap::new();
    variables.insert("servers".to_string(), "10".to_string());
    variables.insert("ram_per_server".to_string(), "32 GiB".to_string());
    variables.insert("cpu_cores".to_string(), "8".to_string());
    variables.insert("disk_size".to_string(), "1 TiB".to_string());

    let previous_results = vec![];

    // Test complex variable expression
    let (result, assignment) = evaluate_with_variables(
        "total_ram = servers * ram_per_server",
        &variables,
        &previous_results,
        0,
    );
    assert_eq!(result, Some("320 GiB".to_string()));
    assert_eq!(
        assignment,
        Some(("total_ram".to_string(), "320 GiB".to_string()))
    );

    // Test expression with multiple variables and units
    let (result, assignment) = evaluate_with_variables(
        "(servers * disk_size) to GiB",
        &variables,
        &previous_results,
        0,
    );
    assert_eq!(result, Some("10,240 GiB".to_string()));
    assert_eq!(assignment, None);

    // Test complex arithmetic with variables
    let (result, assignment) = evaluate_with_variables(
        "servers * (ram_per_server + disk_size) to TiB",
        &variables,
        &previous_results,
        0,
    );
    assert_eq!(result, Some("10.312 TiB".to_string()));
    assert_eq!(assignment, None);
}

#[test]
fn test_user_multiline_scenario() {
    use std::collections::HashMap;

    // Test the specific user scenario: "memory = 40 GiB\ntime = 18 s\nmemory / time"
    let mut variables = HashMap::new();
    let mut previous_results = vec![];

    // Line 1: memory = 40 GiB
    let (result1, assignment1) =
        evaluate_with_variables("memory = 40 GiB", &variables, &previous_results, 0);
    assert_eq!(result1, Some("40 GiB".to_string()));
    assert_eq!(
        assignment1,
        Some(("memory".to_string(), "40 GiB".to_string()))
    );

    // Store the variable assignment
    if let Some((var_name, var_value)) = assignment1 {
        variables.insert(var_name, var_value);
    }
    previous_results.push(result1);

    // Line 2: time = 18 s
    let (result2, assignment2) =
        evaluate_with_variables("time = 18 s", &variables, &previous_results, 1);
    assert_eq!(result2, Some("18 s".to_string()));
    assert_eq!(assignment2, Some(("time".to_string(), "18 s".to_string())));

    // Store the variable assignment
    if let Some((var_name, var_value)) = assignment2 {
        variables.insert(var_name, var_value);
    }
    previous_results.push(result2);

    // Line 3: memory / time
    let (result3, assignment3) =
        evaluate_with_variables("memory / time", &variables, &previous_results, 2);
    assert_eq!(result3, Some("2.222 GiB/s".to_string()));
    assert_eq!(assignment3, None); // No assignment, just evaluation
}

#[test]
fn test_percentage_conversions() {
    // Test converting decimal to percentage
    assert_eq!(
        evaluate_test_expression("0.1 to %"),
        Some("10 %".to_string())
    );
    assert_eq!(
        evaluate_test_expression("0.25 to %"),
        Some("25 %".to_string())
    );
    assert_eq!(
        evaluate_test_expression("1 to %"),
        Some("100 %".to_string())
    );
    assert_eq!(
        evaluate_test_expression("1.5 to %"),
        Some("150 %".to_string())
    );

    // Test percentage parsing (just check it works)
    assert_eq!(evaluate_test_expression("50%"), Some("50 %".to_string()));

    // Test division result to percentage
    assert_eq!(
        evaluate_test_expression("1/10 to %"),
        Some("10 %".to_string())
    );
    assert_eq!(
        evaluate_test_expression("3/4 to %"),
        Some("75 %".to_string())
    );
    assert_eq!(
        evaluate_test_expression("1/3 to %"),
        Some("33.333 %".to_string())
    );
}

#[test]
fn test_percentage_of_operations() {
    // Test basic percentage of operations
    assert_eq!(evaluate_test_expression("10% of 50"), Some("5".to_string()));
    assert_eq!(
        evaluate_test_expression("25% of 100"),
        Some("25".to_string())
    );
    assert_eq!(
        evaluate_test_expression("50% of 200"),
        Some("100".to_string())
    );
    assert_eq!(
        evaluate_test_expression("150% of 40"),
        Some("60".to_string())
    );

    // Test percentage of values with units
    assert_eq!(
        evaluate_test_expression("20% of 100 GiB"),
        Some("20 GiB".to_string())
    );
    assert_eq!(
        evaluate_test_expression("75% of 8 hours"),
        Some("6 h".to_string())
    );
    assert_eq!(
        evaluate_test_expression("12.5% of 80 MB"),
        Some("10 MB".to_string())
    );

    // Test fractional percentages
    assert_eq!(
        evaluate_test_expression("0.5% of 1000"),
        Some("5".to_string())
    );
    assert_eq!(
        evaluate_test_expression("33.33% of 300"),
        Some("99.99".to_string())
    );
}

#[test]
fn test_percentage_with_variables() {
    use std::collections::HashMap;

    // Test percentage operations with variables
    let mut variables = HashMap::new();
    let mut previous_results = vec![];

    // Line 1: total = 100
    let (result1, assignment1) =
        evaluate_with_variables("total = 100", &variables, &previous_results, 0);
    assert_eq!(result1, Some("100".to_string()));
    assert_eq!(assignment1, Some(("total".to_string(), "100".to_string())));

    if let Some((var_name, var_value)) = assignment1 {
        variables.insert(var_name, var_value);
    }
    previous_results.push(result1);

    // Line 2: 15% of total
    let (result2, assignment2) =
        evaluate_with_variables("15% of total", &variables, &previous_results, 1);
    assert_eq!(result2, Some("15".to_string()));
    assert_eq!(assignment2, None);
}

#[test]
fn test_generic_rates_with_variables_and_references() {
    use std::collections::HashMap;

    // Test generic rates with variables
    let mut variables = HashMap::new();
    variables.insert("backup_rate".to_string(), "250 MB/hour".to_string());
    variables.insert("download_time".to_string(), "30 minutes".to_string());
    variables.insert("upload_rate".to_string(), "1 GiB/minute".to_string());

    let previous_results = vec![];

    // Test variable containing generic rate
    let (result, _) = evaluate_with_variables("backup_rate", &variables, &previous_results, 0);
    assert_eq!(result, Some("250 MB/h".to_string())); // Note: display shows "MB/h"

    // Test generic rate variable * time
    let (result, _) =
        evaluate_with_variables("backup_rate * 4 hours", &variables, &previous_results, 0);
    assert_eq!(result, Some("1,000 MB".to_string()));

    // Test generic rate variable * time variable (should fail - can't parse "30 minutes" as single variable)
    // This would require more complex parsing to work

    // Test with line references
    let previous_results = vec![
        Some("100 GiB/hour".to_string()),
        Some("2.5 hours".to_string()),
        Some("500 MB/minute".to_string()),
    ];

    // Test line reference with generic rate
    assert_eq!(
        evaluate_expression_with_context("line1 * 0.5 hours", &previous_results, 3),
        Some("50 GiB".to_string())
    );

    // Test multiple line references with generic rates
    assert_eq!(
        evaluate_expression_with_context("line3 * 6 seconds", &previous_results, 3),
        Some("50 MB".to_string())
    );

    // Test complex expression with line references
    // line1 is 100 GiB/hour, line3 is 500 MB/minute
    // (100 GiB/hour * 2 hours) + (500 MB/minute * 30 minutes)
    // = 200 GiB + 15,000 MB = 200 GiB + 15 GB ≈ 214.7 GiB ≈ 229,748 MB
    assert_eq!(
        evaluate_expression_with_context(
            "(line1 * 2 hours) + (line3 * 30 minutes)",
            &previous_results,
            3
        ),
        Some("229,748.365 MB".to_string())
    );
}

#[test]
fn test_generic_rates_real_world_scenarios() {
    // Data migration scenario
    assert_eq!(
        evaluate_test_expression("Migration: 50 GiB/hour * 8 hours"),
        Some("400 GiB".to_string())
    );

    // Bandwidth calculation
    assert_eq!(
        evaluate_test_expression("Monthly usage: 10 GB/day * 30 days"),
        Some("300 GB".to_string())
    );

    // Storage growth projection
    assert_eq!(
        evaluate_test_expression("Growth: 100 MB/day * 365 days to GiB"),
        Some("33.993 GiB".to_string())
    );

    // Video streaming data transfer calculation
    assert_eq!(
        evaluate_test_expression("Streaming: 25 Mb/minute * 120 minutes to GB"),
        Some("0.375 GB".to_string())
    );
}

#[test]
fn test_percentage_edge_cases() {
    // Test 0% and 100%
    assert_eq!(evaluate_test_expression("0% of 100"), Some("0".to_string()));
    assert_eq!(
        evaluate_test_expression("100% of 50"),
        Some("50".to_string())
    );

    // Test very small percentages
    assert_eq!(
        evaluate_test_expression("0.01% of 10000"),
        Some("1".to_string())
    );

    // Test very large percentages
    assert_eq!(
        evaluate_test_expression("1000% of 5"),
        Some("50".to_string())
    );

    // Test percentage parsing variations
    assert_eq!(
        evaluate_test_expression("25 % of 80"),
        Some("20".to_string())
    );
}
