//! Numeric, percentage, and boolean parsing tests for the utils module.

use crate::utils::{NumericValue, parse_boolean_value, parse_numeric_string, parse_percentage};

#[test]
fn numeric_parsing_integers() {
    assert_eq!(parse_numeric_string("0").unwrap(), NumericValue::Integer(0));
    assert_eq!(
        parse_numeric_string("42").unwrap(),
        NumericValue::Integer(42)
    );
    assert_eq!(
        parse_numeric_string("-123").unwrap(),
        NumericValue::Integer(-123)
    );
    assert_eq!(
        parse_numeric_string("1000").unwrap(),
        NumericValue::Integer(1000)
    );
}

#[test]
fn numeric_parsing_floats() {
    assert_eq!(
        parse_numeric_string("3.14").unwrap(),
        NumericValue::Float(3.14)
    );
    assert_eq!(
        parse_numeric_string("-2.5").unwrap(),
        NumericValue::Float(-2.5)
    );
    assert_eq!(
        parse_numeric_string("0.0").unwrap(),
        NumericValue::Float(0.0)
    );
}

#[test]
fn numeric_parsing_invalid() {
    assert!(parse_numeric_string("").is_err());
    assert!(parse_numeric_string("abc").is_err());
    assert!(parse_numeric_string("12.34.56").is_err());
    assert!(parse_numeric_string("1.2.3").is_err());
    assert!(parse_numeric_string("not_a_number").is_err());
}

#[test]
fn percentage_parsing() {
    assert_eq!(parse_percentage("50%").unwrap(), 0.5);
    assert_eq!(parse_percentage("100%").unwrap(), 1.0);
    assert_eq!(parse_percentage("0%").unwrap(), 0.0);
    assert_eq!(parse_percentage("25.5%").unwrap(), 0.255);
}

#[test]
fn percentage_parsing_invalid() {
    assert!(parse_percentage("50").is_err()); // Missing %
    assert!(parse_percentage("%50").is_err()); // % at start
    assert!(parse_percentage("abc%").is_err()); // Invalid number
    assert!(parse_percentage("").is_err()); // Empty
}

#[test]
fn boolean_parsing() {
    assert_eq!(parse_boolean_value("1").unwrap(), true);
    assert_eq!(parse_boolean_value("-1").unwrap(), true);
    assert_eq!(parse_boolean_value("0").unwrap(), false);
    assert_eq!(parse_boolean_value("true").unwrap(), true);
    assert_eq!(parse_boolean_value("false").unwrap(), false);
    assert_eq!(parse_boolean_value("yes").unwrap(), true);
    assert_eq!(parse_boolean_value("no").unwrap(), false);
}

#[test]
fn boolean_parsing_case_insensitive() {
    assert_eq!(parse_boolean_value("TRUE").unwrap(), true);
    assert_eq!(parse_boolean_value("False").unwrap(), false);
    assert_eq!(parse_boolean_value("YES").unwrap(), true);
    assert_eq!(parse_boolean_value("No").unwrap(), false);
}

#[test]
fn boolean_parsing_invalid() {
    assert!(parse_boolean_value("maybe").is_err());
    assert!(parse_boolean_value("2").is_err());
    assert!(parse_boolean_value("").is_err());
    assert!(parse_boolean_value("on").is_err());
    assert!(parse_boolean_value("off").is_err());
}
