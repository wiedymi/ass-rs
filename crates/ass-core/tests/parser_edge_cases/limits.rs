//! Tests for input size limits and byte-order-mark validation.
//!
//! Covers the security size-limit error path and BOM handling for
//! UTF-16 and malformed UTF-8 inputs.

use ass_core::{
    parser::IssueCategory,
    utils::errors::{encoding::validate_bom_handling, resource::check_input_size_limit},
    Script,
};

#[cfg(not(feature = "std"))]
use alloc::string::String;

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
