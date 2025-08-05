//! Style management commands for ASS documents
//!
//! Provides commands for creating, editing, deleting, cloning, and applying styles
//! with proper validation and delta tracking.

use super::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result, StyleBuilder};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

/// Command to create a new style
#[derive(Debug, Clone)]
pub struct CreateStyleCommand {
    pub style_name: String,
    pub style_builder: StyleBuilder,
    pub description: Option<String>,
}

impl CreateStyleCommand {
    /// Create a new style creation command
    pub fn new(style_name: String, style_builder: StyleBuilder) -> Self {
        Self {
            style_name,
            style_builder,
            description: None,
        }
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for CreateStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        // Build the style line with the provided name
        let mut builder = self.style_builder.clone();
        builder = builder.name(&self.style_name);
        let style_line = builder.build()?;

        // Find the styles section or create one
        let content = document.text();
        let styles_section_pos = content
            .find("[V4+ Styles]")
            .or_else(|| content.find("[V4 Styles]"))
            .or_else(|| content.find("[Styles]"));

        if let Some(section_start) = styles_section_pos {
            // Find the end of the format line to insert after it
            let section_content = &content[section_start..];
            if let Some(format_line_end) = section_content.find('\n') {
                let format_end_pos = section_start + format_line_end + 1;

                // Find next format line or end of section
                let insert_pos = if let Some(next_line_start) = content[format_end_pos..].find('\n')
                {
                    format_end_pos + next_line_start + 1
                } else {
                    format_end_pos
                };

                // Insert the new style
                let insert_text = format!("{style_line}\n");
                document.insert(Position::new(insert_pos), &insert_text)?;

                let end_pos = Position::new(insert_pos + insert_text.len());
                return Ok(CommandResult::success_with_change(
                    Range::new(Position::new(insert_pos), end_pos),
                    end_pos,
                )
                .with_message(format!("Created style '{}'", self.style_name)));
            }
        }

        // No styles section found, create one
        let styles_section = format!(
            "\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n{style_line}\n"
        );

        // Insert before [Events] section if it exists, otherwise at the end
        let insert_pos = if let Some(events_pos) = content.find("[Events]") {
            events_pos
        } else {
            content.len()
        };

        document.insert(Position::new(insert_pos), &styles_section)?;

        let end_pos = Position::new(insert_pos + styles_section.len());
        Ok(CommandResult::success_with_change(
            Range::new(Position::new(insert_pos), end_pos),
            end_pos,
        )
        .with_message(format!(
            "Created styles section and style '{}'",
            self.style_name
        )))
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Create style")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.style_name.len()
            + self.description.as_ref().map_or(0, |d| d.len())
            + 200 // Estimated StyleBuilder size
    }
}

/// Command to edit an existing style
#[derive(Debug, Clone)]
pub struct EditStyleCommand {
    pub style_name: String,
    pub field_updates: HashMap<String, String>,
    pub description: Option<String>,
}

impl EditStyleCommand {
    /// Create a new style edit command
    pub fn new(style_name: String) -> Self {
        Self {
            style_name,
            field_updates: HashMap::new(),
            description: None,
        }
    }

    /// Set a field value
    pub fn set_field(mut self, field: &str, value: String) -> Self {
        self.field_updates.insert(field.to_string(), value);
        self
    }

    /// Set font name
    pub fn set_font(self, font: &str) -> Self {
        self.set_field("Fontname", font.to_string())
    }

    /// Set font size
    pub fn set_size(self, size: u32) -> Self {
        self.set_field("Fontsize", size.to_string())
    }

    /// Set primary color
    pub fn set_color(self, color: &str) -> Self {
        self.set_field("PrimaryColour", color.to_string())
    }

    /// Set bold
    pub fn set_bold(self, bold: bool) -> Self {
        self.set_field("Bold", if bold { "-1" } else { "0" }.to_string())
    }

    /// Set italic
    pub fn set_italic(self, italic: bool) -> Self {
        self.set_field("Italic", if italic { "-1" } else { "0" }.to_string())
    }

    /// Set alignment
    pub fn set_alignment(self, alignment: u32) -> Self {
        self.set_field("Alignment", alignment.to_string())
    }

    /// Set custom description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for EditStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let style_pattern = format!("Style: {}", self.style_name);

        if let Some(style_start) = content.find(&style_pattern) {
            // Find the end of the style line
            let line_start = content[..style_start]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let line_end = content[style_start..]
                .find('\n')
                .map(|pos| style_start + pos)
                .unwrap_or(content.len());

            let style_line = &content[line_start..line_end];
            let fields: Vec<&str> = style_line.split(',').collect();

            if fields.len() < 2 {
                return Err(EditorError::command_failed("Invalid style format"));
            }

            // Find format line to determine field order
            let styles_section_start = content[..line_start]
                .rfind("[V4+ Styles]")
                .or_else(|| content[..line_start].rfind("[V4 Styles]"))
                .or_else(|| content[..line_start].rfind("[Styles]"))
                .ok_or_else(|| EditorError::command_failed("Could not find styles section"))?;

            let format_line_start = content[styles_section_start..]
                .find("Format:")
                .map(|pos| styles_section_start + pos)
                .ok_or_else(|| EditorError::command_failed("Could not find format line"))?;

            let format_line_end = content[format_line_start..]
                .find('\n')
                .map(|pos| format_line_start + pos)
                .unwrap_or(content.len());

            let format_line = &content[format_line_start..format_line_end];
            let format_fields: Vec<&str> = format_line
                .strip_prefix("Format: ")
                .unwrap_or(format_line)
                .split(", ")
                .collect();

            // Build updated style line
            let mut updated_fields = fields
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>();

            for (field_name, new_value) in &self.field_updates {
                if let Some(field_index) = format_fields.iter().position(|f| f == field_name) {
                    if field_index < updated_fields.len() {
                        updated_fields[field_index] = new_value.clone();
                    }
                }
            }

            let new_style_line = updated_fields.join(",");
            let range = Range::new(Position::new(line_start), Position::new(line_end));

            document.replace(range, &new_style_line)?;

            let end_pos = Position::new(line_start + new_style_line.len());
            Ok(CommandResult::success_with_change(
                Range::new(Position::new(line_start), end_pos),
                end_pos,
            )
            .with_message(format!("Updated style '{}'", self.style_name)))
        } else {
            Err(EditorError::command_failed(format!(
                "Style '{}' not found",
                self.style_name
            )))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Edit style")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.style_name.len()
            + self
                .field_updates
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}

/// Command to delete a style
#[derive(Debug, Clone)]
pub struct DeleteStyleCommand {
    pub style_name: String,
    pub description: Option<String>,
}

impl DeleteStyleCommand {
    /// Create a new style deletion command
    pub fn new(style_name: String) -> Self {
        Self {
            style_name,
            description: None,
        }
    }

    /// Set custom description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for DeleteStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let style_pattern = format!("Style: {}", self.style_name);

        if let Some(style_start) = content.find(&style_pattern) {
            // Find the complete line including the newline
            let line_start = content[..style_start]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let line_end = content[style_start..]
                .find('\n')
                .map(|pos| style_start + pos + 1) // Include the newline
                .unwrap_or(content.len());

            let range = Range::new(Position::new(line_start), Position::new(line_end));
            document.delete(range)?;

            Ok(CommandResult::success_with_change(
                Range::new(Position::new(line_start), Position::new(line_start)),
                Position::new(line_start),
            )
            .with_message(format!("Deleted style '{}'", self.style_name)))
        } else {
            Err(EditorError::command_failed(format!(
                "Style '{}' not found",
                self.style_name
            )))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Delete style")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.style_name.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}

/// Command to clone an existing style with a new name
#[derive(Debug, Clone)]
pub struct CloneStyleCommand {
    pub source_style: String,
    pub target_style: String,
    pub description: Option<String>,
}

impl CloneStyleCommand {
    /// Create a new style cloning command
    pub fn new(source_style: String, target_style: String) -> Self {
        Self {
            source_style,
            target_style,
            description: None,
        }
    }

    /// Set custom description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for CloneStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let source_pattern = format!("Style: {}", self.source_style);

        // Check if target style already exists
        let target_pattern = format!("Style: {}", self.target_style);
        if content.contains(&target_pattern) {
            return Err(EditorError::command_failed(format!(
                "Style '{}' already exists",
                self.target_style
            )));
        }

        if let Some(source_start) = content.find(&source_pattern) {
            // Find the complete source style line
            let line_start = content[..source_start]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let line_end = content[source_start..]
                .find('\n')
                .map(|pos| source_start + pos)
                .unwrap_or(content.len());

            let source_line = &content[line_start..line_end];

            // Replace the style name in the cloned line
            let cloned_line = source_line.replace(
                &format!("Style: {}", self.source_style),
                &format!("Style: {}", self.target_style),
            );

            // Find where to insert the new style (after the source style)
            let insert_pos = line_end;
            let insert_text = format!("\n{cloned_line}");

            document.insert(Position::new(insert_pos), &insert_text)?;

            let end_pos = Position::new(insert_pos + insert_text.len());
            Ok(CommandResult::success_with_change(
                Range::new(Position::new(insert_pos), end_pos),
                end_pos,
            )
            .with_message(format!(
                "Cloned style '{}' to '{}'",
                self.source_style, self.target_style
            )))
        } else {
            Err(EditorError::command_failed(format!(
                "Source style '{}' not found",
                self.source_style
            )))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Clone style")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.source_style.len()
            + self.target_style.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}

/// Command to apply a style to events (change all events using one style to another)
#[derive(Debug, Clone)]
pub struct ApplyStyleCommand {
    pub old_style: String,
    pub new_style: String,
    pub event_filter: Option<String>, // Optional filter for event text
    pub description: Option<String>,
}

impl ApplyStyleCommand {
    /// Create a new style application command
    pub fn new(old_style: String, new_style: String) -> Self {
        Self {
            old_style,
            new_style,
            event_filter: None,
            description: None,
        }
    }

    /// Only apply to events containing specific text
    pub fn with_filter(mut self, filter: String) -> Self {
        self.event_filter = Some(filter);
        self
    }

    /// Set custom description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for ApplyStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let mut content = document.text();
        let mut changes_made = 0;
        let mut total_range: Option<Range> = None;

        // Find events section
        let events_start = content
            .find("[Events]")
            .ok_or_else(|| EditorError::command_failed("Events section not found"))?;

        // Skip format line
        let events_content_start = content[events_start..]
            .find('\n')
            .map(|pos| events_start + pos + 1)
            .ok_or_else(|| EditorError::command_failed("Invalid events section format"))?;

        // Skip format line again (Format: ...)
        let first_event_start = content[events_content_start..]
            .find('\n')
            .map(|pos| events_content_start + pos + 1)
            .unwrap_or(events_content_start);

        let mut search_pos = first_event_start;

        while search_pos < content.len() {
            // Find next event line
            let line_start = search_pos;
            let line_end = content[line_start..]
                .find('\n')
                .map(|pos| line_start + pos)
                .unwrap_or(content.len());

            if line_start >= line_end {
                break;
            }

            let line = &content[line_start..line_end];

            // Check if this is an event line
            if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                let parts: Vec<&str> = line.split(',').collect();

                // Check if this event uses the old style (typically 4th field after event type)
                if parts.len() > 4 && parts[3].trim() == self.old_style {
                    // Apply filter if specified
                    let should_update = if let Some(ref filter) = self.event_filter {
                        line.contains(filter)
                    } else {
                        true
                    };

                    if should_update {
                        // Replace the old style with new style
                        let updated_line = line.replace(
                            &format!(",{},", self.old_style),
                            &format!(",{},", self.new_style),
                        );

                        // Update the document
                        let range = Range::new(Position::new(line_start), Position::new(line_end));
                        document.replace(range, &updated_line)?;

                        // Update content for next iteration (this is inefficient but correct)
                        content = document.text();

                        // Track overall range
                        let change_range = Range::new(
                            Position::new(line_start),
                            Position::new(line_start + updated_line.len()),
                        );
                        total_range = Some(match total_range {
                            Some(existing) => existing.union(&change_range),
                            None => change_range,
                        });

                        changes_made += 1;
                    }
                }
            } else if line.starts_with('[') && line != "[Events]" {
                // Stop at next section
                break;
            }

            search_pos = line_end + 1;
        }

        if changes_made > 0 {
            Ok(CommandResult::success_with_change(
                total_range.unwrap_or(Range::new(Position::new(0), Position::new(0))),
                Position::new(content.len()),
            )
            .with_message(format!(
                "Applied style '{}' to {} events",
                self.new_style, changes_made
            )))
        } else {
            Ok(CommandResult::success()
                .with_message("No events found matching the criteria".to_string()))
        }
    }

    fn description(&self) -> &str {
        self.description
            .as_deref()
            .unwrap_or("Apply style to events")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.old_style.len()
            + self.new_style.len()
            + self.event_filter.as_ref().map_or(0, |f| f.len())
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    use crate::core::EditorDocument;
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Hello world!
"#;

    #[test]
    fn test_create_style_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let style_builder = StyleBuilder::new()
            .font("Comic Sans MS")
            .size(24)
            .bold(true);

        let command = CreateStyleCommand::new("NewStyle".to_string(), style_builder);
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("Style: NewStyle"));
        assert!(doc.text().contains("Comic Sans MS"));
    }

    #[test]
    fn test_edit_style_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = EditStyleCommand::new("Default".to_string())
            .set_font("Helvetica")
            .set_size(24)
            .set_bold(true);

        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("Helvetica"));
        assert!(doc.text().contains("24"));
        assert!(doc.text().contains("-1")); // Bold = true
    }

    #[test]
    fn test_delete_style_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = DeleteStyleCommand::new("Default".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(!doc.text().contains("Style: Default"));
    }

    #[test]
    fn test_clone_style_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = CloneStyleCommand::new("Default".to_string(), "DefaultCopy".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("Style: Default")); // Original should still exist
        assert!(doc.text().contains("Style: DefaultCopy")); // Clone should exist
    }

    #[test]
    fn test_apply_style_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // First create a new style to apply
        let create_cmd = CreateStyleCommand::new(
            "NewStyle".to_string(),
            StyleBuilder::new().font("Verdana").size(18),
        );
        create_cmd.execute(&mut doc).unwrap();

        // Now apply the new style to events
        let command = ApplyStyleCommand::new("Default".to_string(), "NewStyle".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("NewStyle")); // Event should now use NewStyle
    }

    #[test]
    fn test_apply_style_with_filter() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Create a new style
        let create_cmd = CreateStyleCommand::new(
            "FilteredStyle".to_string(),
            StyleBuilder::new().font("Times").size(22),
        );
        create_cmd.execute(&mut doc).unwrap();

        // Apply style only to events containing "Hello"
        let command = ApplyStyleCommand::new("Default".to_string(), "FilteredStyle".to_string())
            .with_filter("Hello".to_string());

        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("FilteredStyle"));
    }

    #[test]
    fn test_edit_nonexistent_style() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = EditStyleCommand::new("NonExistent".to_string()).set_font("Arial");

        let result = command.execute(&mut doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_clone_to_existing_style() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = CloneStyleCommand::new("Default".to_string(), "Default".to_string());
        let result = command.execute(&mut doc);

        assert!(result.is_err());
    }
}
