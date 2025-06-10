//! New chumsky-based parser implementation for mathematical expressions

use super::tokens::Token;
use crate::units::parse_unit;
use chumsky::prelude::*;

/// Parse a mathematical expression using chumsky
pub fn parse_expression_chumsky(input: &str) -> Result<Vec<Token>, String> {
    // Create a simple parser that directly parses from string to tokens
    let parser = create_token_parser();
    
    match parser.parse(input).into_result() {
        Ok(tokens) => Ok(tokens),
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
    let number = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| {
            let cleaned = s.replace(",", "");
            cleaned.parse::<f64>().unwrap_or(0.0)
        });

    // Parser for identifiers (words)
    let identifier = text::ascii::ident();

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
                .at_least(1)
                .then(identifier)
                .or_not()
        )
        .map(|(num, unit_opt)| {
            if let Some((_, unit_str)) = unit_opt {
                if let Some(unit) = parse_unit(unit_str) {
                    Token::NumberWithUnit(num, unit)
                } else {
                    Token::Number(num) // Fallback if unit parsing fails
                }
            } else {
                Token::Number(num)
            }
        });

    // Parser for standalone units (for conversions like "to KiB")
    let standalone_unit = identifier
        .try_map(|word, span| {
            if let Some(unit) = parse_unit(word) {
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
}