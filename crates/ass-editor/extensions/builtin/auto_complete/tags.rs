//! Override-tag completion generator.
//!
//! Provides the catalogue of ASS override tags with example insert text and
//! prefix-aware filtering for [`CompletionItem`] suggestions.

use super::extension::AutoCompleteExtension;
use super::types::{CompletionContext, CompletionItem, CompletionType};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec, vec::Vec};

impl AutoCompleteExtension {
    /// Get override tag completions
    pub(super) fn get_tag_completions(&self, context: &CompletionContext) -> Vec<CompletionItem> {
        let tags = vec![
            ("\\b", "Bold (0/1 or weight)", "\\b1"),
            ("\\i", "Italic (0/1)", "\\i1"),
            ("\\u", "Underline (0/1)", "\\u1"),
            ("\\s", "Strikeout (0/1)", "\\s1"),
            ("\\bord", "Border width", "\\bord2"),
            ("\\shad", "Shadow distance", "\\shad2"),
            ("\\be", "Blur edges", "\\be1"),
            ("\\fn", "Font name", "\\fnArial"),
            ("\\fs", "Font size", "\\fs20"),
            ("\\fscx", "Font X scale %", "\\fscx100"),
            ("\\fscy", "Font Y scale %", "\\fscy100"),
            ("\\fsp", "Font spacing", "\\fsp0"),
            ("\\frx", "X rotation", "\\frx0"),
            ("\\fry", "Y rotation", "\\fry0"),
            ("\\frz", "Z rotation", "\\frz0"),
            ("\\fr", "Z rotation (legacy)", "\\fr0"),
            ("\\fax", "X shear", "\\fax0"),
            ("\\fay", "Y shear", "\\fay0"),
            ("\\c", "Primary color", "\\c&H0000FF&"),
            ("\\1c", "Primary color", "\\1c&H0000FF&"),
            ("\\2c", "Secondary color", "\\2c&H00FF00&"),
            ("\\3c", "Outline color", "\\3c&HFF0000&"),
            ("\\4c", "Shadow color", "\\4c&H000000&"),
            ("\\alpha", "Overall alpha", "\\alpha&H00&"),
            ("\\1a", "Primary alpha", "\\1a&H00&"),
            ("\\2a", "Secondary alpha", "\\2a&H00&"),
            ("\\3a", "Outline alpha", "\\3a&H00&"),
            ("\\4a", "Shadow alpha", "\\4a&H00&"),
            ("\\an", "Alignment (numpad)", "\\an5"),
            ("\\a", "Alignment (legacy)", "\\a2"),
            ("\\k", "Karaoke duration", "\\k100"),
            ("\\kf", "Karaoke fill", "\\kf100"),
            ("\\ko", "Karaoke outline", "\\ko100"),
            ("\\K", "Karaoke sweep", "\\K100"),
            ("\\q", "Wrap style", "\\q2"),
            ("\\r", "Reset to style", "\\r"),
            ("\\pos", "Position", "\\pos(640,360)"),
            ("\\move", "Movement", "\\move(0,0,100,100)"),
            ("\\org", "Rotation origin", "\\org(640,360)"),
            ("\\fad", "Fade in/out", "\\fad(200,200)"),
            ("\\fade", "Complex fade", "\\fade(255,0,0,0,1000,2000,3000)"),
            ("\\t", "Animation", "\\t(\\fs30)"),
            ("\\clip", "Clipping rectangle", "\\clip(0,0,100,100)"),
            ("\\iclip", "Inverse clip", "\\iclip(0,0,100,100)"),
            ("\\p", "Drawing mode", "\\p1"),
            ("\\pbo", "Baseline offset", "\\pbo0"),
        ];

        let prefix = if let Some(ref tag) = context.current_tag {
            tag
        } else {
            // Look for backslash prefix
            context.line[..context.column]
                .rfind('\\')
                .map(|pos| &context.line[pos + 1..context.column])
                .unwrap_or("")
        };

        tags.into_iter()
            .filter(|(name, _, _)| {
                if prefix.is_empty() {
                    true
                } else {
                    name[1..].starts_with(prefix)
                }
            })
            .enumerate()
            .map(|(i, (name, desc, example))| {
                CompletionItem::new(example.to_string(), name.to_string(), CompletionType::Tag)
                    .with_description(desc.to_string())
                    .with_detail(example.to_string())
                    .with_sort_order(i as u32)
            })
            .collect()
    }
}
