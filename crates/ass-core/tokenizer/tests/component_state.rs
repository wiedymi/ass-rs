//! Context-state, scanner, and SIMD component-level tokenizer tests.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

#[test]
fn tokenizer_uuencoded_data_context() {
    use crate::tokenizer::state::TokenContext;

    // Test UuEncodedData context behavior
    let context = TokenContext::UuEncodedData;
    assert!(!context.allows_whitespace_skipping());
    assert!(!context.is_delimited_block());
    assert_eq!(context.closing_delimiter(), None);
    assert_eq!(context.enter_field_value(), TokenContext::UuEncodedData);
    assert_eq!(context.reset_to_document(), TokenContext::Document);
}

#[test]
fn tokenizer_drawing_commands_context() {
    use crate::tokenizer::state::TokenContext;

    // Test DrawingCommands context behavior
    let context = TokenContext::DrawingCommands;
    assert!(context.allows_whitespace_skipping());
    assert!(!context.is_delimited_block());
    assert_eq!(context.closing_delimiter(), None);
    assert_eq!(context.enter_field_value(), TokenContext::DrawingCommands);
    assert_eq!(context.reset_to_document(), TokenContext::Document);
}

#[test]
fn tokenizer_critical_issues() {
    use crate::tokenizer::state::{IssueCollector, IssueLevel, TokenIssue};

    let mut collector = IssueCollector::new();

    // Test critical issue handling
    let critical_issue = TokenIssue::critical("Critical error".to_string(), "test", 1, 1);
    collector.add_issue(critical_issue);

    assert!(collector.has_issues());
    assert!(collector.has_errors());
    assert_eq!(collector.issue_count(), 1);

    // Test issue level properties
    assert!(IssueLevel::Critical.is_error());
    assert!(IssueLevel::Critical.should_abort());
    assert_eq!(IssueLevel::Critical.as_str(), "critical");
}

#[test]
fn tokenizer_simd_edge_cases() {
    #[cfg(feature = "simd")]
    {
        use crate::tokenizer::simd;

        // Test SIMD with exactly 16 bytes (chunk boundary)
        let text_16 = "0123456789abcdef";
        assert_eq!(text_16.len(), 16);
        let result = simd::scan_delimiters(text_16);
        assert!(result.is_none());

        // Test SIMD with delimiter at chunk boundary
        let text_with_delimiter = "0123456789abcde:";
        let result = simd::scan_delimiters(text_with_delimiter);
        assert_eq!(result, Some(15));

        // Test hex parsing edge cases
        let invalid_hex = simd::parse_hex_u32("GGGG");
        assert!(invalid_hex.is_none());

        // Test UTF-8 validation with mixed content
        let mixed_utf8 = "Hello, 世界! 🦀";
        let result = simd::validate_utf8_batch(mixed_utf8.as_bytes());
        assert!(result.is_ok());
    }
}

#[test]
fn tokenizer_scanner_edge_cases() {
    use crate::tokenizer::scanner::TokenScanner;

    // Test scanner with empty input
    let scanner = TokenScanner::new("", 0, 1, 1);
    assert!(scanner.navigator().is_at_end());

    // Test scanner navigation methods
    let mut scanner = TokenScanner::new("test", 0, 1, 1);
    let nav = scanner.navigator_mut();

    // Test character advancement and position tracking
    let ch = nav.advance_char();
    assert!(ch.is_ok());
    assert_eq!(ch.unwrap(), 't');
    assert_eq!(nav.position(), 1);
    assert_eq!(nav.column(), 2);
}

#[test]
fn tokenizer_hex_value_detection() {
    use crate::tokenizer::scanner::TokenScanner;

    let mut scanner = TokenScanner::new("&H123ABC", 0, 1, 1);

    // Test hex value scanning
    let result = scanner.scan_text(crate::tokenizer::state::TokenContext::Document);
    assert!(result.is_ok());
}

#[test]
fn tokenizer_field_value_comprehensive() {
    use crate::tokenizer::scanner::TokenScanner;

    // Test field value scanning with various content types
    let test_cases = vec![
        "simple_value",
        "123.45",
        "&H123ABC",
        "value,with,commas",
        "value:with:colons",
    ];

    for test_input in test_cases {
        let mut scanner = TokenScanner::new(test_input, 0, 1, 1);
        let result = scanner.scan_field_value();
        assert!(result.is_ok(), "Failed to scan field value: {test_input}");
    }
}
