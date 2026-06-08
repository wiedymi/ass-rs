//! Section, field, and style-name completion generators.
//!
//! Produces [`CompletionItem`] suggestions for section headers, section-aware
//! field names, and known style references.

use super::extension::AutoCompleteExtension;
use super::types::{CompletionContext, CompletionItem, CompletionType};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec, vec::Vec};

impl AutoCompleteExtension {
    /// Get section header completions
    pub(super) fn get_section_completions(
        &self,
        context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        let sections = vec![
            ("[Script Info]", "Script metadata and properties"),
            ("[V4+ Styles]", "Style definitions for V4+ format"),
            ("[V4 Styles]", "Style definitions for V4 format"),
            ("[Events]", "Dialogue and comment events"),
            ("[Fonts]", "Embedded font data"),
            ("[Graphics]", "Embedded graphics data"),
        ];

        let prefix = if context.line.starts_with('[') {
            &context.line[1..context.column.min(context.line.len())]
        } else {
            ""
        };

        sections
            .into_iter()
            .filter(|(name, _)| {
                if prefix.is_empty() {
                    true
                } else {
                    name[1..].to_lowercase().starts_with(&prefix.to_lowercase())
                }
            })
            .enumerate()
            .map(|(i, (name, desc))| {
                CompletionItem::new(name.to_string(), name.to_string(), CompletionType::Section)
                    .with_description(desc.to_string())
                    .with_sort_order(i as u32)
            })
            .collect()
    }

    /// Get field completions for a section
    pub(super) fn get_field_completions(
        &self,
        section: &str,
        context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        let fields = match section {
            "Script Info" => vec![
                ("Title:", "Script title"),
                ("Original Script:", "Original author"),
                ("Original Translation:", "Original translator"),
                ("Original Editing:", "Original editor"),
                ("Original Timing:", "Original timer"),
                ("Synch Point:", "Synchronization point"),
                ("Script Updated By:", "Last editor"),
                ("Update Details:", "Update description"),
                ("ScriptType:", "Script type (usually v4.00+)"),
                ("Collisions:", "Collision handling (Normal/Reverse)"),
                ("PlayResX:", "Playback X resolution"),
                ("PlayResY:", "Playback Y resolution"),
                ("PlayDepth:", "Color depth"),
                ("Timer:", "Timer speed percentage"),
                ("WrapStyle:", "Line wrapping style (0-3)"),
                (
                    "ScaledBorderAndShadow:",
                    "Scale borders with video (yes/no)",
                ),
                ("YCbCr Matrix:", "Color matrix"),
            ],
            "V4+ Styles" | "V4 Styles" => vec![
                ("Format:", "Column format definition"),
                ("Style:", "Style definition"),
            ],
            "Events" => vec![
                ("Format:", "Column format definition"),
                ("Dialogue:", "Dialogue event"),
                ("Comment:", "Comment event"),
                ("Picture:", "Picture event"),
                ("Sound:", "Sound event"),
                ("Movie:", "Movie event"),
                ("Command:", "Command event"),
            ],
            _ => vec![],
        };

        let prefix = context.line.trim_start();

        fields
            .into_iter()
            .filter(|(name, _)| {
                prefix.is_empty() || name.to_lowercase().starts_with(&prefix.to_lowercase())
            })
            .enumerate()
            .map(|(i, (name, desc))| {
                CompletionItem::new(name.to_string(), name.to_string(), CompletionType::Field)
                    .with_description(desc.to_string())
                    .with_sort_order(i as u32)
            })
            .collect()
    }

    /// Get style name completions
    pub(super) fn get_style_completions(
        &self,
        _context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        self.style_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                CompletionItem::new(name.clone(), name.clone(), CompletionType::StyleRef)
                    .with_description("Style reference".to_string())
                    .with_sort_order(i as u32)
            })
            .collect()
    }
}
