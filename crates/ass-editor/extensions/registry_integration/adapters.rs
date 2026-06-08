//! Adapters bridging editor extensions to ass-core handler traits.
//!
//! Provides [`EditorTagHandlerAdapter`] and [`EditorSectionProcessorAdapter`],
//! which let editor extensions act as ass-core `TagHandler`s and
//! `SectionProcessor`s respectively.

use crate::extensions::EditorExtension;
use ass_core::plugin::{SectionProcessor, SectionResult, TagHandler, TagResult};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};

/// Adapter that allows editor extensions to provide tag handlers
#[allow(dead_code)]
pub struct EditorTagHandlerAdapter {
    extension_name: String,
    tag_name: String,
    extension: Box<dyn EditorExtension>,
}

impl EditorTagHandlerAdapter {
    /// Create a new tag handler adapter
    pub fn new(
        extension_name: String,
        tag_name: String,
        extension: Box<dyn EditorExtension>,
    ) -> Self {
        Self {
            extension_name,
            tag_name,
            extension,
        }
    }
}

impl TagHandler for EditorTagHandlerAdapter {
    fn name(&self) -> &'static str {
        // This is a limitation - we need to leak the string to get a 'static lifetime
        Box::leak(self.tag_name.clone().into_boxed_str())
    }

    fn process(&self, _args: &str) -> TagResult {
        // Extensions process tags through their command system
        // This is a simplified implementation
        TagResult::Processed
    }

    fn validate(&self, args: &str) -> bool {
        !args.is_empty()
    }
}

/// Adapter that allows editor extensions to provide section processors
#[allow(dead_code)]
pub struct EditorSectionProcessorAdapter {
    extension_name: String,
    section_name: String,
    extension: Box<dyn EditorExtension>,
}

impl EditorSectionProcessorAdapter {
    /// Create a new section processor adapter
    pub fn new(
        extension_name: String,
        section_name: String,
        extension: Box<dyn EditorExtension>,
    ) -> Self {
        Self {
            extension_name,
            section_name,
            extension,
        }
    }
}

impl SectionProcessor for EditorSectionProcessorAdapter {
    fn name(&self) -> &'static str {
        // This is a limitation - we need to leak the string to get a 'static lifetime
        Box::leak(self.section_name.clone().into_boxed_str())
    }

    fn process(&self, _header: &str, _lines: &[&str]) -> SectionResult {
        // Extensions process sections through their command system
        // This is a simplified implementation
        SectionResult::Processed
    }

    fn validate(&self, header: &str, lines: &[&str]) -> bool {
        !header.is_empty() && !lines.is_empty()
    }
}
