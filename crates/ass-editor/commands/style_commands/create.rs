//! Style creation command for ASS documents.
//!
//! Provides [`CreateStyleCommand`] to insert a new style into the styles
//! section, creating the section when it does not yet exist.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result, StyleBuilder};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

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
