//! Coverage tests for `TokenScanner`, `CharNavigator`, and `IssueCollector`.
//!
//! Exercises character navigation, scanner context handling, issue collection,
//! and scanner state preservation primitives directly.

use ass_core::tokenizer::{
    scanner::{CharNavigator, TokenScanner},
    state::{IssueCollector, TokenContext},
};

/// Test scanner character navigation edge cases
#[test]
fn test_char_navigator_edge_cases() {
    // Test with empty input
    let mut navigator = CharNavigator::new("", 0, 1, 1);
    assert!(navigator.is_at_end());
    assert!(navigator.peek_char().is_err());

    // Test with single character
    let mut navigator2 = CharNavigator::new("x", 0, 1, 1);
    assert!(!navigator2.is_at_end());
    assert_eq!(navigator2.peek_char().unwrap(), 'x');

    // Advance past end
    navigator2.advance_char().unwrap();
    assert!(navigator2.is_at_end());
    assert!(navigator2.peek_char().is_err());
}

/// Test token scanner in different contexts
#[test]
fn test_token_scanner_context_handling() {
    let source = "test{override}[section]:value;comment";
    let mut scanner = TokenScanner::new(source, 0, 1, 1);

    // Test scanning in different contexts
    let text_result = scanner.scan_text(TokenContext::Document);
    assert!(text_result.is_ok());

    // Move to override position and test
    let mut scanner2 = TokenScanner::new(&source[4..], 0, 1, 5);
    let override_result = scanner2.scan_style_override();
    assert!(override_result.is_ok());
}

/// Test issue collector functionality directly
#[test]
fn test_issue_collector_comprehensive() {
    let mut collector = IssueCollector::new();

    // Initially empty
    assert!(collector.issues().is_empty());

    // Clear should work on empty collector
    collector.clear();
    assert!(collector.issues().is_empty());

    // Test that collector doesn't panic on repeated operations
    for _i in 0..10 {
        collector.clear();
        let _issues = collector.issues();
    }
}

/// Test scanner state preservation across operations
#[test]
fn test_scanner_state_preservation() {
    let source = "test content";
    let scanner = TokenScanner::new(source, 0, 1, 1);

    // Verify initial state
    assert_eq!(scanner.navigator().position(), 0);
    assert_eq!(scanner.navigator().line(), 1);
    assert_eq!(scanner.navigator().column(), 1);
    assert!(!scanner.navigator().is_at_end());

    // Create another scanner at different position
    let scanner2 = TokenScanner::new(source, 5, 2, 3);
    assert_eq!(scanner2.navigator().position(), 5);
    assert_eq!(scanner2.navigator().line(), 2);
    assert_eq!(scanner2.navigator().column(), 3);
}
