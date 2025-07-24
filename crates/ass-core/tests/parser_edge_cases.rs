//! Edge case and error handling tests for the ASS parser.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the parser, focusing on error recovery, edge cases, and security limits.

use ass_core::{
    parser::{IssueCategory, IssueSeverity},
    utils::errors::{encoding::validate_bom_handling, resource::check_input_size_limit},
    Script, Section,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test input size limit exceeded error path (L90-L96)
    #[test]
    fn test_input_size_limit_exceeded() {
        // Test the utility function directly with a smaller limit
        const TEST_LIMIT: usize = 1024;
        let large_source = "x".repeat(TEST_LIMIT + 1);

        let result = check_input_size_limit(large_source.len(), TEST_LIMIT);
        assert!(result.is_err());

        // The actual parser uses 50MB limit, so we can't easily test that in a unit test
        // Instead, test that a reasonably sized script doesn't trigger the limit
        let normal_script = "[Script Info]\nTitle: Test\n[Events]\nDialogue: 0:00:00.00,0:00:05.00,Default,,0,0,0,,Test";
        let script = Script::parse(normal_script).expect("Script parsing should work");

        // Should not have security errors for normal sized input
        assert!(!script
            .issues()
            .iter()
            .any(|issue| matches!(issue.category, IssueCategory::Security)));
    }

    /// Test BOM validation error path (L99-L105)
    #[test]
    #[allow(clippy::similar_names)]
    fn test_invalid_bom_handling() {
        // UTF-16 LE BOM should trigger warning
        let utf16_le_bytes = [0xFF, 0xFE, b'[', b'S', 0x00, b'c', 0x00];
        let result = validate_bom_handling(&utf16_le_bytes);
        assert!(result.is_err());

        // UTF-16 BE BOM should trigger warning
        let utf16_be_bytes = [0xFE, 0xFF, 0x00, b'[', 0x00, b'S'];
        let result = validate_bom_handling(&utf16_be_bytes);
        assert!(result.is_err());

        // Malformed UTF-8 BOM should trigger error
        let malformed_bom = [0xEF, 0xBB, b'X']; // Missing final BF byte
        let result = validate_bom_handling(&malformed_bom);
        assert!(result.is_err());

        // Test parser behavior with invalid BOM
        let source_with_utf16_bom = String::from_utf8_lossy(&utf16_le_bytes);
        let script = Script::parse(&source_with_utf16_bom).expect("Script parsing should work");

        // Should have a format warning
        assert!(script
            .issues()
            .iter()
            .any(|issue| matches!(issue.category, IssueCategory::Format)));
    }

    /// Test malformed section causing `parse_section` error (L113-L126)
    #[test]
    fn test_malformed_section_error_recovery() {
        let malformed_script = r"
[Script Info]
Title: Test

[Malformed Section
This section has no closing bracket

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(malformed_script).expect("Script parsing should work");

        // Should have issues for malformed section (might be warnings, not necessarily errors)
        assert!(script
            .issues()
            .iter()
            .any(|issue| { matches!(issue.category, IssueCategory::Structure) }));

        // Should still parse the valid sections
        assert!(!script.sections().is_empty());
    }

    /// Test `ExpectedSectionHeader` error (L131-L133)
    #[test]
    fn test_expected_section_header_error() {
        let script_no_bracket = r"
Script Info]
Title: Test
";

        let script = Script::parse(script_no_bracket).expect("Script parsing should work");

        assert!(script
            .issues()
            .iter()
            .any(|issue| matches!(issue.severity, IssueSeverity::Error)));
    }

    /// Test `UnclosedSectionHeader` error (L135-L137)
    #[test]
    fn test_unclosed_section_header_error() {
        let script_unclosed = r"
[Script Info
Title: Test
";

        let script = Script::parse(script_unclosed).expect("Script parsing should work");

        assert!(script
            .issues()
            .iter()
            .any(|issue| matches!(issue.severity, IssueSeverity::Error)));
    }

    /// Test Fonts section parsing (L171-L185)
    #[test]
    fn test_fonts_section_parsing() {
        let script_with_fonts = r"
[Script Info]
Title: Test

[Fonts]
fontname: Arial
0
M 0 0 L 100 100

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(script_with_fonts).expect("Script parsing should work");

        // Should successfully parse fonts section
        assert!(script
            .sections()
            .iter()
            .any(|section| matches!(section, Section::Fonts(_))));
    }

    /// Test Graphics section parsing (L171-L185)
    #[test]
    fn test_graphics_section_parsing() {
        let script_with_graphics = r"
[Script Info]
Title: Test

[Graphics]
filename: logo.png
0
89504E470D0A1A0A

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(script_with_graphics).expect("Script parsing should work");

        // Should successfully parse graphics section
        assert!(script
            .sections()
            .iter()
            .any(|section| matches!(section, Section::Graphics(_))));
    }

    /// Test unknown section with suggestion logic (L187-L212)
    #[test]
    fn test_unknown_section_with_suggestions() {
        let script_with_typo = r"
[Script Info]
Title: Test

[Event]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(script_with_typo).expect("Script parsing should work");

        // Should have warning for unknown section
        assert!(script
            .issues()
            .iter()
            .any(|issue| matches!(issue.severity, IssueSeverity::Warning)));

        // Should have info suggestion
        assert!(script
            .issues()
            .iter()
            .any(|issue| matches!(issue.severity, IssueSeverity::Info)));
    }

    /// Test `skip_to_next_section` suggestion logic (L245-L281)
    #[test]
    fn test_skip_to_next_section_suggestions() {
        // Test Style: line suggesting V4+ Styles section
        let script_style_suggestion = r"
[Unknown Section]
Style: Default,Arial,20,&Hffffff,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(script_style_suggestion).expect("Script parsing should work");

        assert!(script.issues().iter().any(|issue| {
            matches!(issue.severity, IssueSeverity::Info) && issue.message.contains("V4+ Styles")
        }));

        // Test Dialogue: line suggesting Events section
        let script_dialogue_suggestion = r"
[Wrong Events]
Dialogue: 0:00:00.00,0:00:05.00,Default,Test text
";

        let script = Script::parse(script_dialogue_suggestion).expect("Script parsing should work");

        assert!(script.issues().iter().any(|issue| {
            matches!(issue.severity, IssueSeverity::Info) && issue.message.contains("Events")
        }));

        // Test Title: line suggesting Script Info section
        let script_title_suggestion = r"
[Bad Section]
Title: My Subtitle File
";

        let script = Script::parse(script_title_suggestion).expect("Script parsing should work");

        assert!(script.issues().iter().any(|issue| {
            matches!(issue.severity, IssueSeverity::Info) && issue.message.contains("Script Info")
        }));
    }

    /// Test file ending abruptly during parsing
    #[test]
    fn test_abrupt_file_ending() {
        let truncated_script = "[Script Info]\nTitle: Test\n[Events";

        let script = Script::parse(truncated_script).expect("Script parsing should work");

        // Should handle truncated file gracefully
        assert!(script
            .issues()
            .iter()
            .any(|issue| matches!(issue.severity, IssueSeverity::Error)));
    }

    /// Test empty section name
    #[test]
    fn test_empty_section_name() {
        let empty_section = "[]";

        let script = Script::parse(empty_section).expect("Script parsing should work");

        // Should have some kind of issue (error or warning) for empty section name
        assert!(!script.issues().is_empty());
    }

    /// Test section name with only whitespace
    #[test]
    fn test_whitespace_only_section_name() {
        let whitespace_section = "[   \t  ]";

        let script = Script::parse(whitespace_section).expect("Script parsing should work");

        // Should have some kind of issue (error or warning) for whitespace-only section name
        assert!(!script.issues().is_empty());
    }

    /// Test multiple consecutive unknown sections
    #[test]
    fn test_multiple_unknown_sections() {
        let multi_unknown = r"
[Unknown1]
Some content

[Unknown2]
More content

[Unknown3]
Even more content

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(multi_unknown).expect("Script parsing should work");

        // Should have multiple warnings for unknown sections
        assert!(
            script
                .issues()
                .iter()
                .filter(|issue| {
                    matches!(issue.severity, IssueSeverity::Warning)
                        && issue.message.contains("Unknown section")
                })
                .count()
                >= 3
        );
    }

    /// Test section header without content
    #[test]
    fn test_empty_section_content() {
        let empty_content = r"
[Script Info]

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(empty_content).expect("Script parsing should work");

        // Should parse successfully even with empty sections
        assert!(!script.sections().is_empty());
    }
}
