//! Read commands for retrieving `[Script Info]` properties.

use crate::core::{EditorDocument, EditorError, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

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
