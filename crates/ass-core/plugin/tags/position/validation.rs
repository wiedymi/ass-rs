//! Numeric validation for position and movement override-tag arguments.
//!
//! Provides the shared numeric (integer or decimal) format check used by the
//! position and movement [`TagHandler`](crate::plugin::TagHandler)
//! implementations in this module.

/// Validate if a string represents a valid number (integer or decimal)
#[inline]
pub(super) fn is_numeric(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // Check for optional negative sign
    let has_sign = first == '-' || first == '+';
    if has_sign && s.len() == 1 {
        return false;
    }

    let mut has_decimal = false;
    let start_idx = usize::from(has_sign);

    for (i, c) in s.chars().enumerate().skip(start_idx) {
        match c {
            '0'..='9' => {}
            '.' => {
                if has_decimal || i == start_idx || i == s.len() - 1 {
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
    fn is_numeric_valid() {
        assert!(is_numeric("123"));
        assert!(is_numeric("123.45"));
        assert!(is_numeric("-123"));
        assert!(is_numeric("-123.45"));
        assert!(is_numeric("+123"));
        assert!(is_numeric("0"));
        assert!(is_numeric("0.0"));
    }

    #[test]
    fn is_numeric_invalid() {
        assert!(!is_numeric(""));
        assert!(!is_numeric("-"));
        assert!(!is_numeric("+"));
        assert!(!is_numeric("123."));
        assert!(!is_numeric(".123"));
        assert!(!is_numeric("12.34.56"));
        assert!(!is_numeric("abc"));
        assert!(!is_numeric("123abc"));
        assert!(!is_numeric("12 3"));
    }
}
