//! Shared mock implementations for plugin system tests.

use crate::plugin::{SectionProcessor, SectionResult, TagHandler, TagResult};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// Mock tag handler for testing
pub(super) struct MockTagHandler {
    name: &'static str,
    should_process: bool,
    should_fail: bool,
}

impl MockTagHandler {
    pub(super) fn new(name: &'static str) -> Self {
        Self {
            name,
            should_process: true,
            should_fail: false,
        }
    }

    pub(super) fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

impl TagHandler for MockTagHandler {
    fn name(&self) -> &'static str {
        self.name
    }

    fn process(&self, _args: &str) -> TagResult {
        if self.should_fail {
            TagResult::Failed("Mock failure".to_string())
        } else if self.should_process {
            TagResult::Processed
        } else {
            TagResult::Ignored
        }
    }

    fn validate(&self, args: &str) -> bool {
        !args.is_empty()
    }
}

/// Mock section processor for testing
pub(super) struct MockSectionProcessor {
    name: &'static str,
    should_process: bool,
    should_fail: bool,
}

impl MockSectionProcessor {
    pub(super) fn new(name: &'static str) -> Self {
        Self {
            name,
            should_process: true,
            should_fail: false,
        }
    }
}

impl SectionProcessor for MockSectionProcessor {
    fn name(&self) -> &'static str {
        self.name
    }

    fn process(&self, _header: &str, _lines: &[&str]) -> SectionResult {
        if self.should_fail {
            SectionResult::Failed("Mock failure".to_string())
        } else if self.should_process {
            SectionResult::Processed
        } else {
            SectionResult::Ignored
        }
    }

    fn validate(&self, header: &str, lines: &[&str]) -> bool {
        !header.is_empty() && !lines.is_empty()
    }
}
