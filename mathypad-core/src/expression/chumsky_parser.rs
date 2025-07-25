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

                let is_current_op = matches!(
                    current,
                    Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Power
                );
                let is_next_op = matches!(
                    next,
                    Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Power
                );

                if is_current_op && is_next_op {
                    // Allow minus after operators for negation, but not other combinations
                    if !matches!(next, Token::Minus) {
                        return Err("Invalid consecutive operators".to_string());
                    }
                }
            }

            Ok(tokens)
        }
        Err(errs) => {
            let error_msg = errs
                .into_iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ");
            Err(error_msg)
        }
    }
}

/// Create the main token parser
fn create_token_parser<'a>() -> impl Parser<'a, &'a str, Vec<Token>, extra::Err<Rich<'a, char>>> {
    // Parser for numerical suffixes like "k" for thousands
    let number_suffix = choice((just('k').to(1_000.0), just('K').to(1_000.0)));

    // Parser for numbers (integers and decimals with optional commas and suffixes)
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
    .then(number_suffix.or_not())
    .map(|(s, suffix_opt): (&str, Option<f64>)| {
        let cleaned = s.replace(",", "");
        let base_value = cleaned.parse::<f64>().unwrap_or(0.0);
        if let Some(multiplier) = suffix_opt {
            base_value * multiplier
        } else {
            base_value
        }
    });

    // Parser for identifiers (words, but not compound with slashes - those are handled separately)
    let identifier = text::ascii::ident().map(|s: &str| s.to_string());

    // Parser for the percent symbol
    let percent_symbol = just('%').map(|_| "%".to_string());

    // Parser for currency symbols
    let currency_symbol = choice((
        just('$').to("$"),
        just('€').to("€"),
        just('£').to("£"),
        just('¥').to("¥"),
        just('₹').to("₹"),
        just('₩').to("₩"),
    ))
    .map(|s: &str| s.to_string());

    // Parser for compound identifiers (like "GiB/s") - only for valid units
    let compound_identifier = text::ascii::ident()
        .then(
            just('/')
                .padded() // Allow spaces around the slash
                .then(text::ascii::ident()),
        )
        .try_map(|(base, (_, suffix)): (&str, (char, &str)), span| {
            let compound = format!("{}/{}", base, suffix);
            // Only allow compound identifiers if they form a valid unit
            if parse_unit(&compound).is_some() {
                Ok(compound)
            } else {
                Err(Rich::custom(
                    span,
                    "Invalid compound identifier - not a valid unit",
                ))
            }
        });

    // Parser for currency rate units (like "$/year", "€/month") - currency symbol followed by /time
    let currency_rate = currency_symbol
        .then(just('/'))
        .then(text::ascii::ident())
        .try_map(
            |((currency_str, _), time_str): ((String, char), &str), span| {
                let compound = format!("{}/{}", currency_str, time_str);
                // Only allow if it forms a valid rate unit
                if parse_unit(&compound).is_some() {
                    Ok(compound)
                } else {
                    Err(Rich::custom(span, "Invalid currency rate unit"))
                }
            },
        );

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
        text::keyword("of").to(Token::Of),
    ));

    // Parser for operators (including assignment)
    let operator = choice((
        just('+').to(Token::Plus),
        just('-').to(Token::Minus),
        just('*').to(Token::Multiply),
        just('/').to(Token::Divide),
        just('^').to(Token::Power),
        just('(').to(Token::LeftParen),
        just(')').to(Token::RightParen),
        just('=').to(Token::Assign),
    ));

    // Combined unit parser (tries currency rates first, then compound units, then simple identifiers, then percent, then currency)
    let unit_identifier = choice((
        currency_rate, // Must come first to match $/year before $ is parsed separately
        compound_identifier,
        identifier,
        percent_symbol,
        currency_symbol,
    ));

    // Parser for numbers with optional units
    let number_with_unit = number
        .then(
            just(' ')
                .repeated()
                .then(unit_identifier)
                .try_map(|(_, unit_str): ((), String), span| {
                    // Don't treat keywords as units in this context
                    if unit_str == "to" || unit_str == "in" || unit_str == "of" {
                        Err(Rich::custom(span, "Keywords are not units"))
                    } else if let Some(unit) = parse_unit(&unit_str) {
                        Ok(unit)
                    } else {
                        Err(Rich::custom(span, format!("Unknown unit: {}", unit_str)))
                    }
                })
                .or_not(),
        )
        .map(|(num, unit_opt)| {
            if let Some(unit) = unit_opt {
                Token::NumberWithUnit(num, unit)
            } else {
                Token::Number(num)
            }
        });

    // Parser for currency rate amounts (like "$5/hr", "€10/day")
    #[allow(clippy::type_complexity)]
    let currency_rate_amount = currency_symbol
        .then(just(' ').repeated()) // Optional spaces
        .then(number)
        .then(just('/'))
        .then(text::ascii::ident())
        .try_map(|parsed: ((((String, ()), f64), char), &str), span| {
            let ((((currency_str, _), amount), _), time_str) = parsed;
            let compound = format!("{}/{}", currency_str, time_str);
            // Only allow if it forms a valid rate unit
            if let Some(unit) = parse_unit(&compound) {
                Ok(Token::NumberWithUnit(amount, unit))
            } else {
                Err(Rich::custom(span, "Invalid currency rate unit"))
            }
        });

    // Parser for currency amounts (currency symbol followed by number)
    let currency_amount = currency_symbol
        .then(just(' ').repeated()) // Optional spaces
        .then(number)
        .map(|((currency_str, _), amount)| {
            if let Some(unit) = parse_unit(&currency_str) {
                Token::NumberWithUnit(amount, unit)
            } else {
                Token::Number(amount) // Fallback, should not happen
            }
        });

    // Parser for standalone units (for conversions like "to KiB")
    let standalone_unit = unit_identifier.try_map(|word: String, span| {
        if let Some(unit) = parse_unit(&word) {
            Ok(Token::NumberWithUnit(1.0, unit))
        } else {
            // Don't fail - let it be handled as a variable instead
            Err(Rich::custom(span, "Not a unit"))
        }
    });

    // Parser for function calls (known function names followed by '(')
    let function = identifier
        .then_ignore(just(' ').repeated())
        .then_ignore(just('(').rewind())
        .try_map(|name: String, span| match name.to_lowercase().as_str() {
            "sqrt" => Ok(Token::Function(name)),
            "sum_above" => Ok(Token::Function(name)),
            _ => Err(Rich::custom(span, "Unknown function")),
        });

    // Parser for variables (catch-all for any identifier not handled above)
    let variable = identifier.map(|word: String| Token::Variable(word));

    // Main token parser - try each option in order (most specific first)
    let token = choice((
        line_ref,             // Must come first to catch "line1" before "line" is treated as unit
        keyword,              // "to" and "in" keywords
        currency_rate_amount, // Currency rate amounts like "$5/hr" (must come before currency_amount)
        currency_amount, // Currency symbols followed by numbers (must come before number_with_unit)
        number_with_unit, // Numbers with optional units
        operator,        // Mathematical operators
        function,        // Function calls (must come before variable)
        standalone_unit, // Standalone units for conversions
        variable,        // Variables (identifiers that aren't units/keywords/line refs)
    ));

    // Parser for punctuation/separators to skip
    let punctuation = choice((
        just(':'),
        just(';'),
        just(','),
        just('!'),
        just('?'),
        just('.'), // Keep it simple - decimal points in numbers are handled in number parser
        just('"'),
        just('\''),
        just('`'),
        just('|'),
        just('&'),
        just('#'),
        just('@'),
        just('~'),
        just('['),
        just(']'),
        just('{'),
        just('}'),
        just('<'),
        just('>'),
    ));

    // Combined parser that tries tokens first, then skips punctuation
    let element = choice((token.map(Some), punctuation.to(None)));

    // Parse elements separated by whitespace, filter out None (punctuation)
    element
        .padded()
        .repeated()
        .collect::<Vec<_>>()
        .map(|elements| elements.into_iter().flatten().collect())
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
        // Should parse as: NumberWithUnit(1.0, Hour), Multiply, NumberWithUnit(10.0, RateUnit(GiB, Second))
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
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(1000.0, Unit::GiB)
        ));

        let result = parse_expression_chumsky("1,234.56 MB");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(1234.56, Unit::MB)
        ));

        let result = parse_expression_chumsky("1,000,000 bytes");
        assert!(result.is_ok(), "Parsing failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(1000000.0, Unit::Byte)
        ));
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
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(1000.0, Unit::GiB)
        ));

        // Test compound units without spaces
        let result = parse_expression_chumsky("10GiB/s");
        assert!(result.is_ok(), "Parsing '10GiB/s' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(10.0, Unit::RateUnit(_, _))
        ));
        if let Token::NumberWithUnit(_, Unit::RateUnit(ref unit1, ref unit2)) = tokens[0] {
            assert_eq!(**unit1, Unit::GiB);
            assert_eq!(**unit2, Unit::Second);
        }

        // Test expressions with multiple units without spaces
        let result = parse_expression_chumsky("1,000GiB + 512MiB");
        assert!(
            result.is_ok(),
            "Parsing '1,000GiB + 512MiB' failed: {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(1000.0, Unit::GiB)
        ));
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
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(999999999.99, Unit::TB)
        ));

        // Test very small decimal
        let result = parse_expression_chumsky("0.000001 seconds");
        assert!(result.is_ok(), "Parsing small decimal failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(0.000001, Unit::Second)
        ));
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
        assert!(
            result.is_ok(),
            "Parsing nested parentheses failed: {:?}",
            result
        );
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
        assert!(
            result.is_ok(),
            "Parsing multiple line refs failed: {:?}",
            result
        );
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
        assert!(
            result.is_ok(),
            "Parsing binary data units failed: {:?}",
            result
        );

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
        assert!(
            result.is_ok(),
            "Parsing line ref + keyword failed: {:?}",
            result
        );

        // Test keywords with complex expressions
        let result = parse_expression_chumsky("(1 GiB + 512 MiB) * 2 to TB");
        assert!(
            result.is_ok(),
            "Parsing complex + keyword failed: {:?}",
            result
        );
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
    fn test_exponentiation_parsing() {
        // Test basic exponentiation
        let result = parse_expression_chumsky("2^3");
        assert!(result.is_ok(), "Parsing '2^3' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Number(2.0)));
        assert!(matches!(tokens[1], Token::Power));
        assert!(matches!(tokens[2], Token::Number(3.0)));

        // Test with spaces
        let result = parse_expression_chumsky("2 ^ 3");
        assert!(
            result.is_ok(),
            "Parsing '2 ^ 3' with spaces failed: {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);

        // Test chained exponentiation
        let result = parse_expression_chumsky("2^3^2");
        assert!(result.is_ok(), "Parsing '2^3^2' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_function_parsing() {
        // Test sqrt function
        let result = parse_expression_chumsky("sqrt(4)");
        assert!(result.is_ok(), "Parsing 'sqrt(4)' failed: {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], Token::Function(ref name) if name == "sqrt"));
        assert!(matches!(tokens[1], Token::LeftParen));
        assert!(matches!(tokens[2], Token::Number(4.0)));
        assert!(matches!(tokens[3], Token::RightParen));

        // Test function with spaces
        let result = parse_expression_chumsky("sqrt (9)");
        assert!(
            result.is_ok(),
            "Parsing 'sqrt (9)' with space failed: {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], Token::Function(ref name) if name == "sqrt"));

        // Test function in expression
        let result = parse_expression_chumsky("2 + sqrt(16)");
        assert!(
            result.is_ok(),
            "Parsing '2 + sqrt(16)' failed: {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 6); // 2, +, sqrt, (, 16, )
    }

    #[test]
    fn test_compound_units_with_spaces() {
        // Test compound units with spaces around slash
        let result = parse_expression_chumsky("100 MB / s");
        assert!(
            result.is_ok(),
            "Parsing 'MB / s' with spaces failed: {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(100.0, Unit::RateUnit(_, _))
        ));
        if let Token::NumberWithUnit(_, Unit::RateUnit(ref unit1, ref unit2)) = tokens[0] {
            assert_eq!(**unit1, Unit::MB);
            assert_eq!(**unit2, Unit::Second);
        }

        // Test compound units without spaces (should still work)
        let result = parse_expression_chumsky("100 MB/s");
        assert!(
            result.is_ok(),
            "Parsing 'MB/s' without spaces failed: {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(100.0, Unit::RateUnit(_, _))
        ));
        if let Token::NumberWithUnit(_, Unit::RateUnit(ref unit1, ref unit2)) = tokens[0] {
            assert_eq!(**unit1, Unit::MB);
            assert_eq!(**unit2, Unit::Second);
        }

        // Test conversion with compound units with spaces
        let result = parse_expression_chumsky("25 QPS to req / min");
        assert!(
            result.is_ok(),
            "Parsing QPS conversion with spaces failed: {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(25.0, Unit::RateUnit(_, _))
        ));
        if let Token::NumberWithUnit(_, Unit::RateUnit(ref unit1, ref unit2)) = tokens[0] {
            assert_eq!(**unit1, Unit::Query);
            assert_eq!(**unit2, Unit::Second);
        }
        assert!(matches!(tokens[1], Token::To));
        assert!(matches!(
            tokens[2],
            Token::NumberWithUnit(1.0, Unit::RateUnit(_, _))
        ));
        if let Token::NumberWithUnit(_, Unit::RateUnit(ref unit1, ref unit2)) = tokens[2] {
            assert_eq!(**unit1, Unit::Request);
            assert_eq!(**unit2, Unit::Minute);
        }

        // Test various request rate units with spaces
        let result = parse_expression_chumsky("50 req / s + 30 requests / min");
        assert!(
            result.is_ok(),
            "Parsing request rates with spaces failed: {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(
            tokens[0],
            Token::NumberWithUnit(50.0, Unit::RateUnit(_, _))
        ));
        if let Token::NumberWithUnit(_, Unit::RateUnit(ref unit1, ref unit2)) = tokens[0] {
            assert_eq!(**unit1, Unit::Request);
            assert_eq!(**unit2, Unit::Second);
        }
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(
            tokens[2],
            Token::NumberWithUnit(30.0, Unit::RateUnit(_, _))
        ));
        if let Token::NumberWithUnit(_, Unit::RateUnit(ref unit1, ref unit2)) = tokens[2] {
            assert_eq!(**unit1, Unit::Request);
            assert_eq!(**unit2, Unit::Minute);
        }
    }

    #[test]
    fn test_error_cases() {
        // Test invalid unit - now that we have variables, this parses as Number + Variable
        // The error happens at evaluation time, not parse time
        let result = parse_expression_chumsky("1 invalidunit");
        assert!(result.is_ok(), "Should parse as number + variable");

        // Test invalid line reference
        let result = parse_expression_chumsky("line0");
        assert!(
            result.is_ok(),
            "line0 should be valid (0-indexed internally)"
        );

        // Note: "1 +" might actually parse as just "1" in chumsky due to how we handle it
        // The incomplete operator is handled during evaluation, not parsing

        // Note: The chumsky parser might be more lenient with some syntax errors
        // depending on how the combinators are set up

        let result = parse_expression_chumsky("1 + 2)");
        assert!(result.is_err(), "Should fail on unmatched parentheses");

        // Test double operators
        let result = parse_expression_chumsky("1 ++ 2");
        assert!(result.is_err(), "Should fail on double operators");

        // Test that malformed decimals are now parsed as separate tokens
        let result = parse_expression_chumsky("1.2.3");
        assert!(result.is_ok(), "Should parse as separate tokens: 1.2 and 3");
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 2); // Should be [Number(1.2), Number(3)]
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
        assert!(
            result.is_ok(),
            "Complex data center calc failed: {:?}",
            result
        );

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

    #[test]
    fn test_k_suffix_parsing() {
        // Test basic 'k' suffix
        let result = parse_expression_chumsky("50k");
        assert!(result.is_ok(), "Failed to parse '50k': {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        if let Token::Number(val) = &tokens[0] {
            assert_eq!(*val, 50000.0);
        } else {
            panic!("Expected Number token, got {:?}", tokens[0]);
        }

        // Test uppercase 'K' suffix
        let result = parse_expression_chumsky("25K");
        assert!(result.is_ok(), "Failed to parse '25K': {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        if let Token::Number(val) = &tokens[0] {
            assert_eq!(*val, 25000.0);
        } else {
            panic!("Expected Number token, got {:?}", tokens[0]);
        }

        // Test decimal with 'k' suffix
        let result = parse_expression_chumsky("3.5k");
        assert!(result.is_ok(), "Failed to parse '3.5k': {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        if let Token::Number(val) = &tokens[0] {
            assert_eq!(*val, 3500.0);
        } else {
            panic!("Expected Number token, got {:?}", tokens[0]);
        }
    }

    #[test]
    fn test_k_suffix_with_currency() {
        // Test currency with 'k' suffix
        let result = parse_expression_chumsky("$50k");
        assert!(result.is_ok(), "Failed to parse '$50k': {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        if let Token::NumberWithUnit(val, unit) = &tokens[0] {
            assert_eq!(*val, 50000.0);
            assert_eq!(*unit, Unit::USD);
        } else {
            panic!("Expected NumberWithUnit token, got {:?}", tokens[0]);
        }

        // Test different currencies with 'k' suffix
        let result = parse_expression_chumsky("€100K");
        assert!(result.is_ok(), "Failed to parse '€100K': {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        if let Token::NumberWithUnit(val, unit) = &tokens[0] {
            assert_eq!(*val, 100000.0);
            assert_eq!(*unit, Unit::EUR);
        } else {
            panic!("Expected NumberWithUnit token, got {:?}", tokens[0]);
        }
    }

    #[test]
    fn test_k_suffix_with_arithmetic() {
        // Test arithmetic with 'k' suffix numbers
        let result = parse_expression_chumsky("50k + 25K");
        assert!(result.is_ok(), "Failed to parse '50k + 25K': {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3); // Number, Operator, Number
        if let Token::Number(val1) = &tokens[0] {
            assert_eq!(*val1, 50000.0);
        }
        if let Token::Number(val2) = &tokens[2] {
            assert_eq!(*val2, 25000.0);
        }

        // Test with units
        let result = parse_expression_chumsky("100k MB");
        assert!(result.is_ok(), "Failed to parse '100k MB': {:?}", result);
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        if let Token::NumberWithUnit(val, unit) = &tokens[0] {
            assert_eq!(*val, 100000.0);
            assert_eq!(*unit, Unit::MB);
        } else {
            panic!("Expected NumberWithUnit token, got {:?}", tokens[0]);
        }
    }

    #[test]
    fn test_sum_above_function_parsing() {
        // Test basic sum_above() parsing
        let result = parse_expression_chumsky("sum_above()");
        assert!(
            result.is_ok(),
            "Failed to parse 'sum_above()': {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3); // Function, LeftParen, RightParen
        if let Token::Function(func_name) = &tokens[0] {
            assert_eq!(func_name, "sum_above");
        } else {
            panic!("Expected Function token, got {:?}", tokens[0]);
        }
        assert!(matches!(tokens[1], Token::LeftParen));
        assert!(matches!(tokens[2], Token::RightParen));

        // Test sum_above() with arithmetic
        let result = parse_expression_chumsky("sum_above() + 100");
        assert!(
            result.is_ok(),
            "Failed to parse 'sum_above() + 100': {:?}",
            result
        );
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 5); // Function, LeftParen, RightParen, Plus, Number
        if let Token::Function(func_name) = &tokens[0] {
            assert_eq!(func_name, "sum_above");
        } else {
            panic!("Expected Function token, got {:?}", tokens[0]);
        }

        // Test case insensitivity
        let result = parse_expression_chumsky("SUM_ABOVE()");
        assert!(
            result.is_ok(),
            "Failed to parse 'SUM_ABOVE()': {:?}",
            result
        );
        let tokens = result.unwrap();
        if let Token::Function(func_name) = &tokens[0] {
            assert_eq!(func_name, "SUM_ABOVE");
        } else {
            panic!("Expected Function token, got {:?}", tokens[0]);
        }
    }
}
