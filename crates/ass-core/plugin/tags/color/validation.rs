//! Hex-format validation for color and alpha override-tag arguments.
//!
//! Provides the shared `&Hbbggrr&` (color) and `&Haa&` (alpha) format checks
//! used by the color and alpha [`TagHandler`](crate::plugin::TagHandler)
//! implementations in this module.

/// Validate color arguments in &Hbbggrr& format
#[inline]
pub(super) fn validate_color_args(args: &str) -> bool {
    let args = args.trim();

    // Check format: &Hbbggrr&
    if !args.starts_with("&H") || !args.ends_with('&') || args.len() != 9 {
        return false;
    }

    // Validate hex digits (between &H and &)
    args[2..8].chars().all(|c| c.is_ascii_hexdigit())
}

/// Validate alpha arguments in &Haa& format
#[inline]
pub(super) fn validate_alpha_args(args: &str) -> bool {
    let args = args.trim();

    // Check format: &Haa&
    if !args.starts_with("&H") || !args.ends_with('&') || args.len() != 5 {
        return false;
    }

    // Validate hex digits (between &H and &)
    args[2..4].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_color_valid() {
        assert!(validate_color_args("&H000000&"));
        assert!(validate_color_args("&HFFFFFF&"));
        assert!(validate_color_args("&H123456&"));
        assert!(validate_color_args("&HABCDEF&"));
        assert!(validate_color_args("&Habcdef&"));
        assert!(validate_color_args(" &H000000& "));
    }

    #[test]
    fn validate_color_invalid() {
        assert!(!validate_color_args(""));
        assert!(!validate_color_args("&H&"));
        assert!(!validate_color_args("&H00&"));
        assert!(!validate_color_args("&H0000&"));
        assert!(!validate_color_args("&H00000000&")); // Too long
        assert!(!validate_color_args("H000000&")); // Missing &
        assert!(!validate_color_args("&H000000")); // Missing &
        assert!(!validate_color_args("&HGGGGGG&")); // Invalid hex
        assert!(!validate_color_args("&H00 000&")); // Space in hex
    }

    #[test]
    fn validate_alpha_valid() {
        assert!(validate_alpha_args("&H00&"));
        assert!(validate_alpha_args("&HFF&"));
        assert!(validate_alpha_args("&H7F&"));
        assert!(validate_alpha_args("&HAB&"));
        assert!(validate_alpha_args("&Hab&"));
        assert!(validate_alpha_args(" &H00& "));
    }

    #[test]
    fn validate_alpha_invalid() {
        assert!(!validate_alpha_args(""));
        assert!(!validate_alpha_args("&H&"));
        assert!(!validate_alpha_args("&H0&"));
        assert!(!validate_alpha_args("&H000&")); // Too long
        assert!(!validate_alpha_args("H00&")); // Missing &
        assert!(!validate_alpha_args("&H00")); // Missing &
        assert!(!validate_alpha_args("&HGG&")); // Invalid hex
        assert!(!validate_alpha_args("&H 0&")); // Space in hex
    }

    #[test]
    fn hex_validation_edge_cases() {
        // Mixed case
        assert!(validate_color_args("&HaAbBcC&"));
        assert!(validate_alpha_args("&HaA&"));

        // All same digit
        assert!(validate_color_args("&H000000&"));
        assert!(validate_color_args("&HFFFFFF&"));
        assert!(validate_alpha_args("&H00&"));
        assert!(validate_alpha_args("&HFF&"));
    }

    #[test]
    fn whitespace_handling() {
        // Leading/trailing whitespace
        assert!(validate_color_args("  &H123456&  "));
        assert!(validate_alpha_args("  &H12&  "));

        // But not internal whitespace
        assert!(!validate_color_args("&H12 3456&"));
        assert!(!validate_alpha_args("&H1 2&"));
    }

    #[test]
    fn case_sensitivity() {
        // H must be uppercase
        assert!(!validate_color_args("&h123456&"));
        assert!(!validate_alpha_args("&h12&"));

        // But hex digits can be either case
        assert!(validate_color_args("&HABCDEF&"));
        assert!(validate_color_args("&Habcdef&"));
        assert!(validate_color_args("&HaBcDeF&"));
    }
}
