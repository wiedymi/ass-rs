//! Numeric-format validation for transform override-tag arguments.
//!
//! Provides the shared integer/decimal numeric check used by the rotation,
//! scale, shear, and spacing [`TagHandler`](crate::plugin::TagHandler)
//! implementations in this module.

/// Validate if argument is a valid number (integer or decimal)
#[inline]
pub(super) fn validate_numeric_arg(args: &str) -> bool {
    let args = args.trim();
    if args.is_empty() {
        return false;
    }

    let mut chars = args.chars();
    let first = chars.next().unwrap();

    // Check for optional sign
    let has_sign = first == '-' || first == '+';
    if has_sign && args.len() == 1 {
        return false;
    }

    let mut has_decimal = false;
    let start_idx = usize::from(has_sign);

    for (i, c) in args.chars().enumerate().skip(start_idx) {
        match c {
            '0'..='9' => {}
            '.' => {
                // No leading/trailing dot, only one decimal point
                if has_decimal || i == start_idx || i == args.len() - 1 {
                    return false;
                }
                has_decimal = true;
            }
            _ => return false,
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_numeric_valid() {
        assert!(validate_numeric_arg("0"));
        assert!(validate_numeric_arg("123"));
        assert!(validate_numeric_arg("-123"));
        assert!(validate_numeric_arg("+123"));
        assert!(validate_numeric_arg("123.45"));
        assert!(validate_numeric_arg("-123.45"));
        assert!(validate_numeric_arg("0.0"));
        assert!(!validate_numeric_arg(".5")); // Invalid - no leading digit
    }

    #[test]
    fn validate_numeric_invalid() {
        assert!(!validate_numeric_arg(""));
        assert!(!validate_numeric_arg("-"));
        assert!(!validate_numeric_arg("+"));
        assert!(!validate_numeric_arg("."));
        assert!(!validate_numeric_arg("123."));
        assert!(!validate_numeric_arg(".123"));
        assert!(!validate_numeric_arg("12.34.56"));
        assert!(!validate_numeric_arg("abc"));
        assert!(!validate_numeric_arg("123abc"));
        assert!(!validate_numeric_arg("12 3"));
        assert!(!validate_numeric_arg("1e5")); // No scientific notation
    }

    #[test]
    fn numeric_edge_cases() {
        // Very large numbers
        assert!(validate_numeric_arg("999999999"));
        assert!(validate_numeric_arg("-999999999"));

        // Very small decimals
        assert!(validate_numeric_arg("0.00001"));
        assert!(validate_numeric_arg("-0.00001"));

        // Zero variations
        assert!(validate_numeric_arg("0"));
        assert!(validate_numeric_arg("0.0"));
        assert!(validate_numeric_arg("-0"));
        assert!(validate_numeric_arg("+0"));
    }
}
