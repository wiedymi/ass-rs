//! Tests for handler and processor argument validation.

use super::mocks::{MockSectionProcessor, MockTagHandler};
use crate::plugin::{SectionProcessor, TagHandler};
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn tag_handler_validation() {
    let handler = MockTagHandler::new("test");
    assert!(handler.validate("valid_args"));
    assert!(!handler.validate(""));
}

#[test]
fn section_processor_validation() {
    let processor = MockSectionProcessor::new("test");
    let lines = vec!["line1"];
    assert!(processor.validate("header", &lines));
    assert!(!processor.validate("", &lines));
    assert!(!processor.validate("header", &[]));
}
