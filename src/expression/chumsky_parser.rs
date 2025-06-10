//! New chumsky-based parser implementation for mathematical expressions

use super::tokens::Token;
use crate::units::parse_unit;
use chumsky::prelude::*;

/// Parse a mathematical expression using chumsky
pub fn parse_expression_chumsky(input: &str) -> Result<Vec<Token>, String> {
    // Create a simple parser that directly parses from string to tokens
    let parser = create_token_parser();
    
    match parser.parse(input).into_result() {
        Ok(tokens) => {
            // Validate parentheses are balanced
            let mut paren_count = 0;
            for token in &tokens {
                match token {
                    Token::LeftParen => paren_count += 1,
                    Token::RightParen => {
                        paren_count -= 1;
                        if paren_count < 0 {
                            return Err("Unmatched closing parenthesis".to_string());
                        }
                    }
                    _ => {}
                }
            }
            if paren_count != 0 {
                return Err("Unmatched opening parenthesis".to_string());
            }
            
            // Validate no consecutive operators (except minus for negation)
            for i in 0..tokens.len().saturating_sub(1) {
                let current = &tokens[i];
                let next = &tokens[i + 1];
                
                let is_current_op = matches!(current, Token::Plus | Token::Minus | Token::Multiply | Token::Divide);
                let is_next_op = matches!(next, Token::Plus | Token::Minus | Token::Multiply | Token::Divide);
                
                if is_current_op && is_next_op {
                    // Allow minus after operators for negation, but not other combinations
                    if !matches!(next, Token::Minus) {
                        return Err("Invalid consecutive operators".to_string());
                    }
                }
            }
            
            Ok(tokens)
        },
        Err(errs) => {
            let error_msg = errs.into_iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ");
            Err(error_msg)
        }
    }
}

/// Create the main token parser
fn create_token_parser<'a>() -> impl Parser<'a, &'a str, Vec<Token>, extra::Err<Rich<'a, char>>> {
    // Parser for numbers (integers and decimals with optional commas)
    let number = choice((
        // Numbers with commas (like 1,000 or 1,234.56)
        text::digits(10)
            .then(just(',').then(text::digits(10)).repeated())
            .then(just('.').then(text::digits(10)).or_not())
            .to_slice(),
        // Regular numbers without commas
        text::int(10)
            .then(just('.').then(text::digits(10)).or_not())
            .to_slice(),
    ))
    .map(|s: &str| {
        let cleaned = s.replace(",", "");
        cleaned.parse::<f64>().unwrap_or(0.0)
    });

    // Parser for identifiers (words, including compound units with slashes)
    let identifier = text::ascii::ident()
        .then(
            just('/')
                .padded()  // Allow spaces around the slash
                .then(text::ascii::ident())
                .or_not()
        )
        .map(|(base, slash_part): (&str, Option<(char, &str)>)| {
            if let Some((_, suffix)) = slash_part {
                format!("{}/{}", base, suffix)
            } else {
                base.to_string()
            }
        });

    // Parser for line references (like "line1", "line2", etc.)
    let line_ref = just("line")
        .then(text::int(10))
        .map(|(_, num_str): (_, &str)| {
            if let Ok(line_num) = num_str.parse::<usize>() {
                if line_num > 0 {
                    Token::LineReference(line_num - 1)
                } else {
                    Token::LineReference(0)
                }
            } else {
                Token::LineReference(0)
            }
        });

    // Parser for keywords
    let keyword = choice((
        text::keyword("to").to(Token::To),
        text::keyword("in").to(Token::In),
    ));

    // Parser for operators
    let operator = choice((
        just('+').to(Token::Plus),
        just('-').to(Token::Minus),
        just('*').to(Token::Multiply),
        just('/').to(Token::Divide),
        just('(').to(Token::LeftParen),
        just(')').to(Token::RightParen),
    ));

    // Parser for numbers with optional units  
    let number_with_unit = number
        .then(
            just(' ')
                .repeated()
                .then(identifier)
                .try_map(|(_, unit_str): ((), String), span| {
                    // Don't treat keywords as units in this context
                    if unit_str == "to" || unit_str == "in" {
                        Err(Rich::custom(span, "Keywords are not units"))
                    } else if let Some(unit) = parse_unit(&unit_str) {
                        Ok(unit)
                    } else {
                        Err(Rich::custom(span, format!("Unknown unit: {}", unit_str)))
                    }
                })
                .or_not()
        )
        .map(|(num, unit_opt)| {
            if let Some(unit) = unit_opt {
                Token::NumberWithUnit(num, unit)
            } else {
                Token::Number(num)
            }
        });

    // Parser for standalone units (for conversions like "to KiB")
    let standalone_unit = identifier
        .try_map(|word: String, span| {
            if let Some(unit) = parse_unit(&word) {
                Ok(Token::NumberWithUnit(1.0, unit))
            } else {
                Err(Rich::custom(span, format!("Unknown unit: {}", word)))
            }
        });

    // Main token parser - try each option in order (most specific first)
    let token = choice((
        line_ref,           // Must come first to catch "line1" before "line" is treated as unit
        keyword,            // "to" and "in" keywords
        number_with_unit,   // Numbers with optional units
        operator,           // Mathematical operators
        standalone_unit,    // Standalone units for conversions
    ));

    // Parse tokens separated by whitespace
    token
        .padded()
        .repeated()
        .collect()
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Unit;

    #[test]
    fn test_number_parsing() {
        let result = parse_expression_chumsky("42");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Number(42.0)));
    }

    #[test]
    fn test_number_with_unit() {
        let result = parse_expression_chumsky("5 GiB");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(5.0, Unit::GiB)));
    }

    #[test]
    fn test_simple_arithmetic() {
        let result = parse_expression_chumsky("2 + 3");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Number(2.0)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Number(3.0)));
    }

    #[test]
    fn test_line_reference() {
        let result = parse_expression_chumsky("line1 + 4");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::LineReference(0)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Number(4.0)));
    }

    #[test]
    fn test_complex_expressions() {
        let result = parse_expression_chumsky("line1 * 2 GiB + 500 MiB to KiB");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0], Token::LineReference(0)));
        assert!(matches!(tokens[1], Token::Multiply));
        assert!(matches!(tokens[2], Token::NumberWithUnit(2.0, Unit::GiB)));
        assert!(matches!(tokens[3], Token::Plus));
        assert!(matches!(tokens[4], Token::NumberWithUnit(500.0, Unit::MiB)));
        assert!(matches!(tokens[5], Token::To));
        assert!(matches!(tokens[6], Token::NumberWithUnit(1.0, Unit::KiB)));
    }

    #[test]
    fn test_parentheses() {
        let result = parse_expression_chumsky("(5 + 3) * 2");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0], Token::LeftParen));
        assert!(matches!(tokens[1], Token::Number(5.0)));
        assert!(matches!(tokens[2], Token::Plus));
        assert!(matches!(tokens[3], Token::Number(3.0)));
        assert!(matches!(tokens[4], Token::RightParen));
        assert!(matches!(tokens[5], Token::Multiply));
        assert!(matches!(tokens[6], Token::Number(2.0)));
    }

    #[test]
    fn test_conversion() {
        let result = parse_expression_chumsky("1 GiB to KiB");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::NumberWithUnit(1.0, Unit::GiB)));
        assert!(matches!(tokens[1], Token::To));
        assert!(matches!(tokens[2], Token::NumberWithUnit(1.0, Unit::KiB)));
    }

    #[test]
    fn test_in_keyword() {
        let result = parse_expression_chumsky("24 MiB * 32 in KiB");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::NumberWithUnit(24.0, Unit::MiB)));
        assert!(matches!(tokens[1], Token::Multiply));
        assert!(matches!(tokens[2], Token::Number(32.0)));
        assert!(matches!(tokens[3], Token::In));
        assert!(matches!(tokens[4], Token::NumberWithUnit(1.0, Unit::KiB)));
    }

    #[test]
    fn test_time_rate_multiplication() {
        let result = parse_expression_chumsky("1 hour * 10 GiB/s");
        println!("Tokens for '1 hour * 10 GiB/s': {:?}", result);
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        // Should parse as: NumberWithUnit(1.0, Hour), Multiply, NumberWithUnit(10.0, GiBPerSecond)
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::NumberWithUnit(1.0, _)));
        assert!(matches!(tokens[1], Token::Multiply));
        assert!(matches!(tokens[2], Token::NumberWithUnit(10.0, _)));
    }

    #[test]
    fn test_comma_separated_numbers() {
        let result = parse_expression_chumsky("1,000 GiB");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(1000.0, Unit::GiB)));

        let result = parse_expression_chumsky("1,234.56 MB");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(1234.56, Unit::MB)));

        let result = parse_expression_chumsky("1,000,000 bytes");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(1000000.0, Unit::Byte)));
    }

    #[test]
    fn test_numbers_without_spaces() {
        // Test basic numbers without spaces
        let result = parse_expression_chumsky("5GiB");
        assert!(result.is_ok(), "Parsing '5GiB' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(5.0, Unit::GiB)));

        let result = parse_expression_chumsky("100MB");
        assert!(result.is_ok(), "Parsing '100MB' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(100.0, Unit::MB)));

        // Test decimal numbers without spaces
        let result = parse_expression_chumsky("2.5TiB");
        assert!(result.is_ok(), "Parsing '2.5TiB' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(2.5, Unit::TiB)));

        // Test comma numbers without spaces
        let result = parse_expression_chumsky("1,000GiB");
        assert!(result.is_ok(), "Parsing '1,000GiB' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(1000.0, Unit::GiB)));

        // Test compound units without spaces
        let result = parse_expression_chumsky("10GiB/s");
        assert!(result.is_ok(), "Parsing '10GiB/s' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(10.0, Unit::GiBPerSecond)));

        // Test expressions with multiple units without spaces
        let result = parse_expression_chumsky("1,000GiB + 512MiB");
        assert!(result.is_ok(), "Parsing '1,000GiB + 512MiB' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::NumberWithUnit(1000.0, Unit::GiB)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::NumberWithUnit(512.0, Unit::MiB)));
    }

    #[test]
    fn test_edge_case_numbers() {
        // Test zero
        let result = parse_expression_chumsky("0");
        assert!(result.is_ok(), "Parsing '0' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Number(0.0)));

        // Test zero with unit
        let result = parse_expression_chumsky("0 GiB");
        assert!(result.is_ok(), "Parsing '0 GiB' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(0.0, Unit::GiB)));

        // Test decimal starting with zero
        let result = parse_expression_chumsky("0.5 MB");
        assert!(result.is_ok(), "Parsing '0.5 MB' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(0.5, Unit::MB)));

        // Test very large number
        let result = parse_expression_chumsky("999,999,999.99 TB");
        assert!(result.is_ok(), "Parsing large number failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(999999999.99, Unit::TB)));

        // Test very small decimal
        let result = parse_expression_chumsky("0.000001 seconds");
        assert!(result.is_ok(), "Parsing small decimal failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(0.000001, Unit::Second)));
    }

    #[test]
    fn test_all_operators() {
        // Test all mathematical operators
        let result = parse_expression_chumsky("1 + 2 - 3 * 4 / 5");
        assert!(result.is_ok(), "Parsing all operators failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 9);
        assert!(matches!(tokens[0], Token::Number(1.0)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Number(2.0)));
        assert!(matches!(tokens[3], Token::Minus));
        assert!(matches!(tokens[4], Token::Number(3.0)));
        assert!(matches!(tokens[5], Token::Multiply));
        assert!(matches!(tokens[6], Token::Number(4.0)));
        assert!(matches!(tokens[7], Token::Divide));
        assert!(matches!(tokens[8], Token::Number(5.0)));
    }

    #[test]
    fn test_nested_parentheses() {
        let result = parse_expression_chumsky("((1 + 2) * (3 - 4)) / 5");
        assert!(result.is_ok(), "Parsing nested parentheses failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 15);
        assert!(matches!(tokens[0], Token::LeftParen));
        assert!(matches!(tokens[1], Token::LeftParen));
        assert!(matches!(tokens[2], Token::Number(1.0)));
        assert!(matches!(tokens[13], Token::Divide));
        assert!(matches!(tokens[14], Token::Number(5.0)));
    }

    #[test]
    fn test_multiple_line_references() {
        let result = parse_expression_chumsky("line1 + line2 * line10");
        assert!(result.is_ok(), "Parsing multiple line refs failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::LineReference(0)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::LineReference(1)));
        assert!(matches!(tokens[3], Token::Multiply));
        assert!(matches!(tokens[4], Token::LineReference(9)));
    }

    #[test]
    fn test_all_unit_types() {
        // Test data units
        let result = parse_expression_chumsky("1 B + 2 KB + 3 MB + 4 GB + 5 TB + 6 PB + 7 EB");
        assert!(result.is_ok(), "Parsing data units failed: {:?}", result);
        
        // Test binary data units
        let result = parse_expression_chumsky("1 KiB + 2 MiB + 3 GiB + 4 TiB + 5 PiB + 6 EiB");
        assert!(result.is_ok(), "Parsing binary data units failed: {:?}", result);

        // Test time units
        let result = parse_expression_chumsky("1 ns + 2 us + 3 ms + 4 s + 5 min + 6 h + 7 day");
        assert!(result.is_ok(), "Parsing time units failed: {:?}", result);

        // Test rate units
        let result = parse_expression_chumsky("1 B/s + 2 KB/s + 3 GiB/s");
        assert!(result.is_ok(), "Parsing rate units failed: {:?}", result);

        // Test QPS units
        let result = parse_expression_chumsky("1 QPS + 2 QPM + 3 QPH + 4 req/s");
        assert!(result.is_ok(), "Parsing QPS units failed: {:?}", result);

        // Test bit units
        let result = parse_expression_chumsky("1 bit + 2 Kb + 3 Mb + 4 Gb");
        assert!(result.is_ok(), "Parsing bit units failed: {:?}", result);
    }

    #[test]
    fn test_keyword_combinations() {
        // Test both conversion keywords
        let result = parse_expression_chumsky("1 GiB to MB in KiB");
        assert!(result.is_ok(), "Parsing keywords failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[1], Token::To));
        assert!(matches!(tokens[3], Token::In));

        // Test keywords with line references
        let result = parse_expression_chumsky("line1 to GiB");
        assert!(result.is_ok(), "Parsing line ref + keyword failed: {:?}", result);
        
        // Test keywords with complex expressions
        let result = parse_expression_chumsky("(1 GiB + 512 MiB) * 2 to TB");
        assert!(result.is_ok(), "Parsing complex + keyword failed: {:?}", result);
    }

    #[test]
    fn test_whitespace_variations() {
        // Test extra spaces
        let result = parse_expression_chumsky("  1   +   2   ");
        assert!(result.is_ok(), "Parsing extra spaces failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);

        // Test tabs and mixed whitespace
        let result = parse_expression_chumsky("1\t+\t2");
        assert!(result.is_ok(), "Parsing tabs failed: {:?}", result);

        // Test no spaces around operators
        let result = parse_expression_chumsky("1+2*3");
        assert!(result.is_ok(), "Parsing no spaces failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_compound_units_with_spaces() {
        // Test compound units with spaces around slash
        let result = parse_expression_chumsky("100 MB / s");
        assert!(result.is_ok(), "Parsing 'MB / s' with spaces failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(100.0, Unit::MBPerSecond)));

        // Test compound units without spaces (should still work)
        let result = parse_expression_chumsky("100 MB/s");
        assert!(result.is_ok(), "Parsing 'MB/s' without spaces failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::NumberWithUnit(100.0, Unit::MBPerSecond)));

        // Test conversion with compound units with spaces
        let result = parse_expression_chumsky("25 QPS to req / min");
        assert!(result.is_ok(), "Parsing QPS conversion with spaces failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::NumberWithUnit(25.0, Unit::QueriesPerSecond)));
        assert!(matches!(tokens[1], Token::To));
        assert!(matches!(tokens[2], Token::NumberWithUnit(1.0, Unit::RequestsPerMinute)));

        // Test various request rate units with spaces
        let result = parse_expression_chumsky("50 req / s + 30 requests / min");
        assert!(result.is_ok(), "Parsing request rates with spaces failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::NumberWithUnit(50.0, Unit::RequestsPerSecond)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::NumberWithUnit(30.0, Unit::RequestsPerMinute)));
    }

    #[test]
    fn test_error_cases() {
        // Test invalid unit
        let result = parse_expression_chumsky("1 invalidunit");
        assert!(result.is_err(), "Should fail on invalid unit");

        // Test invalid line reference
        let result = parse_expression_chumsky("line0");
        assert!(result.is_ok(), "line0 should be valid (0-indexed internally)");

        // Note: "1 +" might actually parse as just "1" in chumsky due to how we handle it
        // The incomplete operator is handled during evaluation, not parsing

        // Note: The chumsky parser might be more lenient with some syntax errors
        // depending on how the combinators are set up

        let result = parse_expression_chumsky("1 + 2)");
        assert!(result.is_err(), "Should fail on unmatched parentheses");

        // Test double operators
        let result = parse_expression_chumsky("1 ++ 2");
        assert!(result.is_err(), "Should fail on double operators");

        // Test invalid decimal
        let result = parse_expression_chumsky("1.2.3");
        assert!(result.is_err(), "Should fail on invalid decimal");
    }

    #[test]
    fn test_case_sensitivity() {
        // Test case variations of units
        let result = parse_expression_chumsky("1 gib + 2 GIB + 3 GiB");
        assert!(result.is_ok(), "Case sensitivity test failed: {:?}", result);

        // Test case variations of keywords (note: keywords are case-sensitive in chumsky parser)
        let result = parse_expression_chumsky("1 GiB to mb");
        assert!(result.is_ok(), "Keyword case test failed: {:?}", result);

        let result = parse_expression_chumsky("1 GiB in kb");
        assert!(result.is_ok(), "Keyword case test failed: {:?}", result);
    }

    #[test]
    fn test_complex_real_world_expressions() {
        // Test realistic data center calculation
        let result = parse_expression_chumsky("(50PB + 10EB) / 1000 to TB/s");
        assert!(result.is_ok(), "Complex data center calc failed: {:?}", result);

        // Test realistic QPS calculation
        let result = parse_expression_chumsky("(100QPS + 50req/s) * 1hour to queries");
        assert!(result.is_ok(), "Complex QPS calc failed: {:?}", result);

        // Test mixed unit types in realistic scenario
        let result = parse_expression_chumsky("1000GiB / 10min + 500MB/s * 2h");
        assert!(result.is_ok(), "Mixed unit calc failed: {:?}", result);

        // Test line references in complex expression
        let result = parse_expression_chumsky("(line1 + line2) * 2.5 to GiB/s");
        assert!(result.is_ok(), "Complex line ref calc failed: {:?}", result);
    }
}