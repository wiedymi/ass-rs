//! Built-in auto-completion extension for ASS/SSA files
//!
//! Provides intelligent auto-completion for:
//! - Section names
//! - Field names based on current section
//! - Style names when referenced in events
//! - Override tags and their parameters
//! - Color codes and common values

use crate::core::{EditorDocument, Position, Result};
use crate::extensions::{
    EditorExtension, ExtensionCapability, ExtensionCommand, ExtensionContext, ExtensionInfo,
    ExtensionResult, ExtensionState, MessageLevel,
};
use ass_core::parser::{Script, Section};

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};
#[cfg(feature = "std")]
use std::collections::HashMap;

/// Type of completion being provided
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionType {
    /// Section header completion
    Section,
    /// Field name completion
    Field,
    /// Field value completion
    Value,
    /// Style name reference
    StyleRef,
    /// Override tag
    Tag,
    /// Tag parameter
    TagParam,
    /// Color value
    Color,
    /// Time code
    Time,
}

/// A single completion suggestion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    /// Text to insert
    pub insert_text: String,
    /// Display label
    pub label: String,
    /// Type of completion
    pub completion_type: CompletionType,
    /// Description of the item
    pub description: Option<String>,
    /// Additional details
    pub detail: Option<String>,
    /// Sort priority (lower = higher priority)
    pub sort_order: u32,
}

impl CompletionItem {
    /// Create a new completion item
    pub fn new(insert_text: String, label: String, completion_type: CompletionType) -> Self {
        Self {
            insert_text,
            label,
            completion_type,
            description: None,
            detail: None,
            sort_order: 999,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set detail
    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Set sort order
    pub fn with_sort_order(mut self, order: u32) -> Self {
        self.sort_order = order;
        self
    }
}

/// Completion context at a position
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Current line text
    pub line: String,
    /// Position within the line
    pub column: usize,
    /// Current section (if any)
    pub section: Option<String>,
    /// Whether we're inside an override tag
    pub in_override_tag: bool,
    /// Current tag being typed (if any)
    pub current_tag: Option<String>,
}

/// Auto-completion extension
pub struct AutoCompleteExtension {
    info: ExtensionInfo,
    state: ExtensionState,
    /// Known style names from the document
    style_names: Vec<String>,
    /// Configuration
    config: AutoCompleteConfig,
}

/// Configuration for auto-completion
#[derive(Debug, Clone)]
pub struct AutoCompleteConfig {
    /// Enable field name completion
    pub complete_fields: bool,
    /// Enable style reference completion
    pub complete_styles: bool,
    /// Enable override tag completion
    pub complete_tags: bool,
    /// Enable value completion
    pub complete_values: bool,
    /// Maximum suggestions to show
    pub max_suggestions: usize,
    /// Minimum characters before triggering
    pub min_chars: usize,
}

impl Default for AutoCompleteConfig {
    fn default() -> Self {
        Self {
            complete_fields: true,
            complete_styles: true,
            complete_tags: true,
            complete_values: true,
            max_suggestions: 20,
            min_chars: 1,
        }
    }
}

impl AutoCompleteExtension {
    /// Create a new auto-complete extension
    pub fn new() -> Self {
        let info = ExtensionInfo::new(
            "auto-complete".to_string(),
            "1.0.0".to_string(),
            "ASS-RS Team".to_string(),
            "Built-in auto-completion for ASS/SSA files".to_string(),
        )
        .with_capability(ExtensionCapability::CodeCompletion)
        .with_license("MIT".to_string());

        Self {
            info,
            state: ExtensionState::Uninitialized,
            style_names: Vec::new(),
            config: AutoCompleteConfig::default(),
        }
    }

    /// Get completions at a position
    pub fn get_completions(
        &mut self,
        document: &EditorDocument,
        position: Position,
    ) -> Result<Vec<CompletionItem>> {
        // Update style names from document
        self.update_style_names(document)?;

        // Get completion context
        let context = self.get_completion_context(document, position)?;

        // Generate completions based on context
        let mut completions = Vec::new();

        // Section completions
        if context.line.is_empty() || context.line.starts_with('[') {
            completions.extend(self.get_section_completions(&context));
        }

        // Field completions
        if let Some(ref section) = context.section {
            if !context.in_override_tag && self.config.complete_fields {
                completions.extend(self.get_field_completions(section, &context));
            }
        }

        // Override tag completions
        if context.in_override_tag && self.config.complete_tags {
            completions.extend(self.get_tag_completions(&context));
        }

        // Style reference completions
        if self.should_complete_style(&context) && self.config.complete_styles {
            completions.extend(self.get_style_completions(&context));
        }

        // Sort and limit completions
        completions.sort_by_key(|c| c.sort_order);
        completions.truncate(self.config.max_suggestions);

        Ok(completions)
    }

    /// Update known style names from document
    fn update_style_names(&mut self, document: &EditorDocument) -> Result<()> {
        self.style_names.clear();

        if let Ok(script) = Script::parse(&document.text()) {
            for section in script.sections() {
                if let Section::Styles(styles) = section {
                    for style in styles {
                        self.style_names.push(style.name.to_string());
                    }
                }
            }
        }

        Ok(())
    }

    /// Get completion context at position
    fn get_completion_context(
        &self,
        document: &EditorDocument,
        position: Position,
    ) -> Result<CompletionContext> {
        let content = document.text();
        let offset = position.offset;

        // Find current line
        let line_start = content[..offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let line_end = content[offset..]
            .find('\n')
            .map(|p| offset + p)
            .unwrap_or(content.len());

        let line = content[line_start..line_end].to_string();
        let column = offset - line_start;

        // Find current section
        let mut current_section = None;
        for line in content[..line_start].lines().rev() {
            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(line[1..line.len() - 1].to_string());
                break;
            }
        }

        // Check if we're in an override tag
        let before_cursor = &line[..column.min(line.len())];
        let in_override_tag = before_cursor
            .rfind('{')
            .is_some_and(|open| before_cursor[open..].find('}').is_none());

        // Get current tag if in override
        let current_tag = if in_override_tag {
            before_cursor.rfind('{').and_then(|pos| {
                let tag_text = &before_cursor[pos + 1..];
                tag_text.rfind('\\').map(|slash| {
                    let tag_start = &tag_text[slash + 1..];
                    tag_start
                        .find(|c: char| !c.is_alphanumeric())
                        .map(|end| tag_start[..end].to_string())
                        .unwrap_or_else(|| tag_start.to_string())
                })
            })
        } else {
            None
        };

        Ok(CompletionContext {
            line,
            column,
            section: current_section,
            in_override_tag,
            current_tag,
        })
    }

    /// Get section header completions
    fn get_section_completions(&self, context: &CompletionContext) -> Vec<CompletionItem> {
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
    fn get_field_completions(
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

    /// Get override tag completions
    fn get_tag_completions(&self, context: &CompletionContext) -> Vec<CompletionItem> {
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

    /// Get style name completions
    fn get_style_completions(&self, _context: &CompletionContext) -> Vec<CompletionItem> {
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

    /// Check if we should complete style names
    fn should_complete_style(&self, context: &CompletionContext) -> bool {
        if let Some(ref section) = context.section {
            if section == "Events" {
                // Check if we're in the style field of an event
                let line = context.line.trim_start();
                if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                    // Count commas to determine field
                    let before_cursor = &context.line[..context.column];
                    let comma_count = before_cursor.matches(',').count();
                    // Style is the 4th field (after 3 commas)
                    return comma_count == 3;
                }
            }
        }
        false
    }
}

impl Default for AutoCompleteExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorExtension for AutoCompleteExtension {
    fn info(&self) -> &ExtensionInfo {
        &self.info
    }

    fn initialize(&mut self, context: &mut dyn ExtensionContext) -> Result<()> {
        self.state = ExtensionState::Active;

        // Load configuration
        if let Some(fields) = context.get_config("autocomplete.complete_fields") {
            self.config.complete_fields = fields == "true";
        }
        if let Some(styles) = context.get_config("autocomplete.complete_styles") {
            self.config.complete_styles = styles == "true";
        }
        if let Some(tags) = context.get_config("autocomplete.complete_tags") {
            self.config.complete_tags = tags == "true";
        }
        if let Some(values) = context.get_config("autocomplete.complete_values") {
            self.config.complete_values = values == "true";
        }
        if let Some(max) = context.get_config("autocomplete.max_suggestions") {
            if let Ok(max_val) = max.parse() {
                self.config.max_suggestions = max_val;
            }
        }

        context.show_message("Auto-completion initialized", MessageLevel::Info)?;
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut dyn ExtensionContext) -> Result<()> {
        self.state = ExtensionState::Shutdown;
        self.style_names.clear();
        Ok(())
    }

    fn state(&self) -> ExtensionState {
        self.state
    }

    fn execute_command(
        &mut self,
        command_id: &str,
        args: &HashMap<String, String>,
        context: &mut dyn ExtensionContext,
    ) -> Result<ExtensionResult> {
        match command_id {
            "autocomplete.trigger" => {
                if let Some(doc) = context.current_document() {
                    // Get position from args or use end of document
                    let position = if let Some(offset_str) = args.get("position") {
                        if let Ok(offset) = offset_str.parse() {
                            Position::new(offset)
                        } else {
                            Position::new(doc.len_bytes())
                        }
                    } else {
                        Position::new(doc.len_bytes())
                    };

                    let completions = self.get_completions(doc, position)?;
                    let mut result = ExtensionResult::success_with_message(format!(
                        "Found {} completions",
                        completions.len()
                    ));

                    // Add completion data
                    for (i, completion) in completions.iter().take(10).enumerate() {
                        result
                            .data
                            .insert(format!("completion_{i}"), completion.insert_text.clone());
                    }

                    Ok(result)
                } else {
                    Ok(ExtensionResult::failure("No active document".to_string()))
                }
            }
            "autocomplete.update_styles" => {
                if let Some(doc) = context.current_document() {
                    self.update_style_names(doc)?;
                    Ok(ExtensionResult::success_with_message(format!(
                        "Updated {} style names",
                        self.style_names.len()
                    )))
                } else {
                    Ok(ExtensionResult::failure("No active document".to_string()))
                }
            }
            _ => Ok(ExtensionResult::failure(format!(
                "Unknown command: {command_id}"
            ))),
        }
    }

    fn commands(&self) -> Vec<ExtensionCommand> {
        vec![
            ExtensionCommand::new(
                "autocomplete.trigger".to_string(),
                "Trigger Completion".to_string(),
                "Get completion suggestions at cursor position".to_string(),
            )
            .with_category("Completion".to_string()),
            ExtensionCommand::new(
                "autocomplete.update_styles".to_string(),
                "Update Style Names".to_string(),
                "Update known style names from document".to_string(),
            )
            .with_category("Completion".to_string()),
        ]
    }

    fn config_schema(&self) -> HashMap<String, String> {
        let mut schema = HashMap::new();
        schema.insert(
            "autocomplete.complete_fields".to_string(),
            "boolean".to_string(),
        );
        schema.insert(
            "autocomplete.complete_styles".to_string(),
            "boolean".to_string(),
        );
        schema.insert(
            "autocomplete.complete_tags".to_string(),
            "boolean".to_string(),
        );
        schema.insert(
            "autocomplete.complete_values".to_string(),
            "boolean".to_string(),
        );
        schema.insert(
            "autocomplete.max_suggestions".to_string(),
            "number".to_string(),
        );
        schema.insert("autocomplete.min_chars".to_string(), "number".to_string());
        schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_item() {
        let item = CompletionItem::new(
            "\\pos(100,200)".to_string(),
            "\\pos".to_string(),
            CompletionType::Tag,
        )
        .with_description("Position tag".to_string())
        .with_sort_order(1);

        assert_eq!(item.insert_text, "\\pos(100,200)");
        assert_eq!(item.label, "\\pos");
        assert_eq!(item.sort_order, 1);
    }

    #[test]
    fn test_auto_complete_extension_creation() {
        let ext = AutoCompleteExtension::new();
        assert_eq!(ext.info().name, "auto-complete");
        assert!(ext
            .info()
            .has_capability(&ExtensionCapability::CodeCompletion));
    }

    #[test]
    fn test_section_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "[Scr".to_string(),
            column: 4,
            section: None,
            in_override_tag: false,
            current_tag: None,
        };

        let completions = ext.get_section_completions(&context);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "[Script Info]"));
    }

    #[test]
    fn test_field_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "Ti".to_string(),
            column: 2,
            section: Some("Script Info".to_string()),
            in_override_tag: false,
            current_tag: None,
        };

        let completions = ext.get_field_completions("Script Info", &context);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "Title:"));
    }

    #[test]
    fn test_tag_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "{\\po".to_string(),
            column: 4,
            section: Some("Events".to_string()),
            in_override_tag: true,
            current_tag: Some("po".to_string()),
        };

        let completions = ext.get_tag_completions(&context);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "\\pos"));
    }
}

// Include extended tests
#[cfg(test)]
#[path = "auto_complete_tests.rs"]
mod extended_tests;
