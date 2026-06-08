//! Numeric-format validation shared by the miscellaneous tag handlers.
//!
//! Provides the [`is_numeric`] check used to validate rotation degrees and
//! origin coordinates for the `\fr` and `\org` handlers in this module.

/// Validate if a string represents a valid number
#[inline]
pub(super) fn is_numeric(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // Check for optional sign
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
    fn is_numeric_edge_cases() {
        assert!(is_numeric("0"));
        assert!(is_numeric("-0"));
        assert!(is_numeric("+0"));
        assert!(is_numeric("123"));
        assert!(is_numeric("-123"));
        assert!(is_numeric("123.45"));
        assert!(is_numeric("0.001"));
        assert!(is_numeric("999999"));

        assert!(!is_numeric(""));
        assert!(!is_numeric("-"));
        assert!(!is_numeric("."));
        assert!(!is_numeric("123."));
        assert!(!is_numeric(".123"));
        assert!(!is_numeric("1.2.3"));
        assert!(!is_numeric("1e5")); // No scientific notation
    }
}
