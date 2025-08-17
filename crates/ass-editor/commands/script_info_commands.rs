//! Script Info management commands for ASS documents
//!
//! Provides commands for managing script metadata like Title, Author, Resolution,
//! and other properties in the `[Script Info]` section.

use super::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Command to set a script info property in the ASS document
///
/// Sets or updates properties like Title, Author, PlayResX, PlayResY, etc.
/// in the `[Script Info]` section. Creates the section if it doesn't exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetScriptInfoCommand {
    /// Property name (e.g., "Title", "Author", "PlayResX")
    pub property: String,
    /// New value for the property
    pub value: String,
}

impl SetScriptInfoCommand {
    /// Create a new set script info command
    pub fn new(property: String, value: String) -> Self {
        Self { property, value }
    }
}

impl EditorCommand for SetScriptInfoCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Script Info] section
        let script_info_start = content
            .find("[Script Info]")
            .ok_or_else(|| EditorError::command_failed("No [Script Info] section found"))?;

        // Find the end of Script Info section (next section or end of file)
        let script_info_end = content[script_info_start..]
            .find("\n[")
            .map(|pos| script_info_start + pos)
            .unwrap_or(content.len());

        // Check if property already exists
        let property_pattern = format!("{}: ", self.property);
        let search_range = &content[script_info_start..script_info_end];

        if let Some(prop_pos) = search_range.find(&property_pattern) {
            // Property exists, update it
            let absolute_pos = script_info_start + prop_pos;
            let line_start = absolute_pos;
            let line_end = content[absolute_pos..]
                .find('\n')
                .map(|pos| absolute_pos + pos)
                .unwrap_or(content.len());

            let new_line = format!("{}: {}", self.property, self.value);
            let range = Range::new(Position::new(line_start), Position::new(line_end));

            document.replace(range, &new_line)?;

            Ok(CommandResult::success_with_change(
                Range::new(
                    Position::new(line_start),
                    Position::new(line_start + new_line.len()),
                ),
                Position::new(line_start + new_line.len()),
            ))
        } else {
            // Property doesn't exist, add it after [Script Info]
            let insert_pos = content[script_info_start..]
                .find('\n')
                .map(|pos| script_info_start + pos + 1)
                .unwrap_or(script_info_start + 13); // Length of "[Script Info]"

            let new_line = format!("{}: {}\n", self.property, self.value);
            document.insert_raw(Position::new(insert_pos), &new_line)?;

            Ok(CommandResult::success_with_change(
                Range::new(
                    Position::new(insert_pos),
                    Position::new(insert_pos + new_line.len()),
                ),
                Position::new(insert_pos + new_line.len()),
            ))
        }
    }

    fn description(&self) -> &str {
        "Set script info property"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.property.len() + self.value.len()
    }
}

/// Command to get a script info property from the ASS document
///
/// Retrieves the value of a specific property from the `[Script Info]` section.
/// Returns an error if the property doesn't exist. value
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetScriptInfoCommand {
    /// Property name to retrieve
    pub property: String,
}

impl GetScriptInfoCommand {
    /// Create a new get script info command
    pub fn new(property: String) -> Self {
        Self { property }
    }

    /// Execute and return the property value
    pub fn get_value(&self, document: &EditorDocument) -> Result<Option<String>> {
        let content = document.text();

        // Find [Script Info] section
        let script_info_start = content
            .find("[Script Info]")
            .ok_or_else(|| EditorError::command_failed("No [Script Info] section found"))?;

        // Find the end of Script Info section
        let script_info_end = content[script_info_start..]
            .find("\n[")
            .map(|pos| script_info_start + pos)
            .unwrap_or(content.len());

        // Look for the property
        let property_pattern = format!("{}: ", self.property);
        let search_range = &content[script_info_start..script_info_end];

        if let Some(prop_pos) = search_range.find(&property_pattern) {
            let value_start = prop_pos + property_pattern.len();
            let value_end = search_range[value_start..]
                .find('\n')
                .map(|pos| value_start + pos)
                .unwrap_or(search_range.len());

            let value = search_range[value_start..value_end].trim().to_string();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

/// Command to delete a script info property from the ASS document
///
/// Removes a specific property from the `[Script Info]` section.
/// Does not remove the section itself even if it becomes empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteScriptInfoCommand {
    /// Property name to delete
    pub property: String,
}

impl DeleteScriptInfoCommand {
    /// Create a new delete script info command
    pub fn new(property: String) -> Self {
        Self { property }
    }
}

impl EditorCommand for DeleteScriptInfoCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Script Info] section
        let script_info_start = content
            .find("[Script Info]")
            .ok_or_else(|| EditorError::command_failed("No [Script Info] section found"))?;

        // Find the end of Script Info section
        let script_info_end = content[script_info_start..]
            .find("\n[")
            .map(|pos| script_info_start + pos)
            .unwrap_or(content.len());

        // Look for the property
        let property_pattern = format!("{}: ", self.property);
        let search_range = &content[script_info_start..script_info_end];

        if let Some(prop_pos) = search_range.find(&property_pattern) {
            let absolute_pos = script_info_start + prop_pos;
            let line_end = content[absolute_pos..]
                .find('\n')
                .map(|pos| absolute_pos + pos + 1) // Include newline
                .unwrap_or(content.len());

            let range = Range::new(Position::new(absolute_pos), Position::new(line_end));
            document.delete(range)?;

            Ok(CommandResult::success_with_change(
                Range::new(Position::new(absolute_pos), Position::new(absolute_pos)),
                Position::new(absolute_pos),
            ))
        } else {
            Ok(CommandResult::success())
        }
    }

    fn description(&self) -> &str {
        "Delete script info property"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.property.len()
    }
}

/// Command to get all script info properties from the ASS document
///
/// Returns all properties in the `[Script Info]` section as key-value pairs.
/// Returns an empty vector if the section doesn't exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAllScriptInfoCommand;

impl GetAllScriptInfoCommand {
    /// Create a new get all script info command
    pub fn new() -> Self {
        Self
    }

    /// Execute and return all properties as key-value pairs
    pub fn get_all(&self, document: &EditorDocument) -> Result<Vec<(String, String)>> {
        let content = document.text();

        // Find [Script Info] section
        let script_info_start = content
            .find("[Script Info]")
            .ok_or_else(|| EditorError::command_failed("No [Script Info] section found"))?;

        // Find the end of Script Info section
        let script_info_end = content[script_info_start..]
            .find("\n[")
            .map(|pos| script_info_start + pos)
            .unwrap_or(content.len());

        let search_range = &content[script_info_start..script_info_end];
        let mut properties = Vec::new();

        for line in search_range.lines() {
            // Skip the section header and empty lines
            if line.starts_with('[') || line.trim().is_empty() {
                continue;
            }

            // Parse property: value pairs
            if let Some(colon_pos) = line.find(':') {
                let property = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                properties.push((property, value));
            }
        }

        Ok(properties)
    }
}

impl Default for GetAllScriptInfoCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EditorDocument;

    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test Subtitle
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

    #[test]
    fn test_set_existing_property() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = SetScriptInfoCommand::new("Title".to_string(), "New Title".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("Title: New Title"));
        assert!(!doc.text().contains("Title: Test Subtitle"));
    }

    #[test]
    fn test_set_new_property() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = SetScriptInfoCommand::new("Author".to_string(), "John Doe".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("Author: John Doe"));
    }

    #[test]
    fn test_get_existing_property() {
        let doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = GetScriptInfoCommand::new("Title".to_string());
        let value = command.get_value(&doc).unwrap();

        assert_eq!(value, Some("Test Subtitle".to_string()));
    }

    #[test]
    fn test_get_nonexistent_property() {
        let doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = GetScriptInfoCommand::new("Author".to_string());
        let value = command.get_value(&doc).unwrap();

        assert_eq!(value, None);
    }

    #[test]
    fn test_delete_property() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = DeleteScriptInfoCommand::new("PlayResX".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(!doc.text().contains("PlayResX: 1920"));
        assert!(doc.text().contains("PlayResY: 1080")); // Other properties remain
    }

    #[test]
    fn test_get_all_properties() {
        let doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = GetAllScriptInfoCommand::new();
        let properties = command.get_all(&doc).unwrap();

        assert_eq!(properties.len(), 4);
        assert!(properties.contains(&("Title".to_string(), "Test Subtitle".to_string())));
        assert!(properties.contains(&("ScriptType".to_string(), "v4.00+".to_string())));
        assert!(properties.contains(&("PlayResX".to_string(), "1920".to_string())));
        assert!(properties.contains(&("PlayResY".to_string(), "1080".to_string())));
    }
}
