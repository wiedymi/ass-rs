//! Issue-collector and token-type property component-level tokenizer tests.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec};

#[test]
fn tokenizer_issue_collector_comprehensive() {
    use crate::tokenizer::state::{IssueCollector, TokenIssue};

    let mut collector = IssueCollector::new();

    // Test all issue addition methods
    collector.add_warning("Warning message".to_string(), "span", 1, 1);
    collector.add_error("Error message".to_string(), "span", 2, 1);
    collector.add_critical("Critical message".to_string(), "span", 3, 1);

    assert_eq!(collector.issue_count(), 3);
    assert!(collector.has_issues());
    assert!(collector.has_errors());

    // Test taking issues
    let issues = collector.take_issues();
    assert_eq!(issues.len(), 3);
    assert_eq!(collector.issue_count(), 0);
    assert!(!collector.has_issues());

    // Test issue formatting
    let issue = TokenIssue::warning("Test warning".to_string(), "test_span", 1, 5);
    let location = issue.location_string();
    assert!(location.contains("1:5"));

    let formatted = issue.format_issue();
    assert!(formatted.contains("warning"));
    assert!(formatted.contains("Test warning"));
}

#[test]
fn tokenizer_all_token_types_properties() {
    use crate::tokenizer::tokens::TokenType;

    let all_types = vec![
        TokenType::Text,
        TokenType::Number,
        TokenType::HexValue,
        TokenType::Colon,
        TokenType::Comma,
        TokenType::Newline,
        TokenType::SectionOpen,
        TokenType::SectionClose,
        TokenType::SectionName,
        TokenType::SectionHeader,
        TokenType::OverrideOpen,
        TokenType::OverrideClose,
        TokenType::OverrideBlock,
        TokenType::Comment,
        TokenType::Whitespace,
        TokenType::DrawingScale,
        TokenType::UuEncodedLine,
        TokenType::FontFilename,
        TokenType::GraphicFilename,
        TokenType::FormatLine,
        TokenType::EventType,
        TokenType::TimeValue,
        TokenType::BooleanValue,
        TokenType::PercentageValue,
        TokenType::StringLiteral,
        TokenType::Invalid,
        TokenType::Eof,
    ];

    for token_type in all_types {
        // Test that name() doesn't panic and returns non-empty string
        let name = token_type.name();
        assert!(
            !name.is_empty(),
            "Token type name should not be empty: {token_type:?}"
        );

        // Test Display implementation
        let display_str = format!("{token_type}");
        assert!(
            !display_str.is_empty(),
            "Token type display should not be empty: {token_type:?}"
        );

        // Test consistency of classification methods
        if token_type.is_delimiter() {
            assert!(
                !token_type.is_content(),
                "Delimiter should not be content: {token_type:?}"
            );
        }

        if token_type.is_structural() {
            // Structural tokens define document structure
            assert!(
                matches!(
                    token_type,
                    TokenType::SectionHeader
                        | TokenType::SectionOpen
                        | TokenType::SectionClose
                        | TokenType::FormatLine
                        | TokenType::Newline
                ),
                "Unexpected structural token: {token_type:?}"
            );
        }
    }
}
