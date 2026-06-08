//! Fluent API builders for style operations.

use crate::commands::{
    ApplyStyleCommand, CloneStyleCommand, CreateStyleCommand, DeleteStyleCommand, EditStyleCommand,
    EditorCommand,
};
use crate::core::{EditorDocument, Result, StyleBuilder};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Fluent API builder for style operations
pub struct StyleOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> StyleOps<'a> {
    /// Create a new style operations builder
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Create a new style
    pub fn create(self, name: &str, builder: StyleBuilder) -> Result<&'a mut EditorDocument> {
        let command = CreateStyleCommand::new(name.to_string(), builder);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Edit an existing style
    pub fn edit(self, name: &str) -> StyleEditor<'a> {
        StyleEditor::new(self.document, name.to_string())
    }

    /// Delete a style
    pub fn delete(self, name: &str) -> Result<&'a mut EditorDocument> {
        let command = DeleteStyleCommand::new(name.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Clone a style
    pub fn clone(self, source: &str, target: &str) -> Result<&'a mut EditorDocument> {
        let command = CloneStyleCommand::new(source.to_string(), target.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Apply a style to events
    pub fn apply(self, old_style: &str, new_style: &str) -> StyleApplicator<'a> {
        StyleApplicator::new(self.document, old_style.to_string(), new_style.to_string())
    }
}

/// Fluent API builder for editing a specific style
pub struct StyleEditor<'a> {
    document: &'a mut EditorDocument,
    command: EditStyleCommand,
}

impl<'a> StyleEditor<'a> {
    /// Create a new style editor
    pub(crate) fn new(document: &'a mut EditorDocument, style_name: String) -> Self {
        let command = EditStyleCommand::new(style_name);
        Self { document, command }
    }

    /// Set font name
    pub fn font(mut self, font: &str) -> Self {
        self.command = self.command.set_font(font);
        self
    }

    /// Set font size
    pub fn size(mut self, size: u32) -> Self {
        self.command = self.command.set_size(size);
        self
    }

    /// Set primary color
    pub fn color(mut self, color: &str) -> Self {
        self.command = self.command.set_color(color);
        self
    }

    /// Set bold
    pub fn bold(mut self, bold: bool) -> Self {
        self.command = self.command.set_bold(bold);
        self
    }

    /// Set italic
    pub fn italic(mut self, italic: bool) -> Self {
        self.command = self.command.set_italic(italic);
        self
    }

    /// Set alignment
    pub fn alignment(mut self, alignment: u32) -> Self {
        self.command = self.command.set_alignment(alignment);
        self
    }

    /// Set a custom field
    pub fn field(mut self, name: &str, value: &str) -> Self {
        self.command = self.command.set_field(name, value.to_string());
        self
    }

    /// Apply the changes
    pub fn apply(self) -> Result<&'a mut EditorDocument> {
        self.command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API builder for applying styles to events
pub struct StyleApplicator<'a> {
    document: &'a mut EditorDocument,
    command: ApplyStyleCommand,
}

impl<'a> StyleApplicator<'a> {
    /// Create a new style applicator
    pub(crate) fn new(
        document: &'a mut EditorDocument,
        old_style: String,
        new_style: String,
    ) -> Self {
        let command = ApplyStyleCommand::new(old_style, new_style);
        Self { document, command }
    }

    /// Only apply to events containing specific text
    pub fn with_filter(mut self, filter: &str) -> Self {
        self.command = self.command.with_filter(filter.to_string());
        self
    }

    /// Apply the style changes
    pub fn apply(self) -> Result<&'a mut EditorDocument> {
        self.command.execute(self.document)?;
        Ok(self.document)
    }
}
