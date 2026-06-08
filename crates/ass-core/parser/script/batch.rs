//! Bulk and transactional editing operations.
//!
//! Implements batched line updates, batched style/event insertion, and the
//! [`Script::atomic_batch_update`] driver that validates every operation before
//! applying changes to a cloned script for all-or-nothing semantics.

use alloc::vec::Vec;

use crate::parser::ast::Section;
use crate::parser::errors::ParseError;

use super::types::{BatchUpdateResult, EventBatch, StyleBatch, UpdateOperation};
use super::Script;

impl<'a> Script<'a> {
    /// Perform multiple line updates in a single operation
    ///
    /// Updates are performed in the order provided. If an update fails,
    /// it's recorded in the failed list but doesn't stop other updates.
    ///
    /// # Arguments
    ///
    /// * `operations` - List of update operations to perform
    ///
    /// # Returns
    ///
    /// Result containing successful updates and failures
    pub fn batch_update_lines(
        &mut self,
        operations: Vec<UpdateOperation<'a>>,
    ) -> BatchUpdateResult<'a> {
        let mut result = BatchUpdateResult {
            updated: Vec::with_capacity(operations.len()),
            failed: Vec::new(),
        };

        // Sort operations by offset to process in order
        let mut sorted_ops = operations;
        sorted_ops.sort_by_key(|op| op.offset);

        for op in sorted_ops {
            match self.update_line_at_offset(op.offset, op.new_line, op.line_number) {
                Ok(old_content) => {
                    result.updated.push((op.offset, old_content));
                }
                Err(e) => {
                    result.failed.push((op.offset, e));
                }
            }
        }

        result
    }

    /// Add multiple styles in a single operation
    ///
    /// Creates the styles section if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `batch` - Batch of styles to add
    ///
    /// # Returns
    ///
    /// Indices of the added styles within the styles section
    pub fn batch_add_styles(&mut self, batch: StyleBatch<'a>) -> Vec<usize> {
        let mut indices = Vec::with_capacity(batch.styles.len());

        // Find or create styles section
        let styles_section_index = self
            .sections
            .iter()
            .position(|s| matches!(s, Section::Styles(_)));

        if let Some(index) = styles_section_index {
            if let Section::Styles(styles) = &mut self.sections[index] {
                let start_index = styles.len();
                styles.extend(batch.styles);
                indices.extend(start_index..styles.len());
            }
        } else {
            // Create new styles section
            let count = batch.styles.len();
            self.sections.push(Section::Styles(batch.styles));
            indices.extend(0..count);
        }

        indices
    }

    /// Add multiple events in a single operation
    ///
    /// Creates the events section if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `batch` - Batch of events to add
    ///
    /// # Returns
    ///
    /// Indices of the added events within the events section
    pub fn batch_add_events(&mut self, batch: EventBatch<'a>) -> Vec<usize> {
        let mut indices = Vec::with_capacity(batch.events.len());

        // Find or create events section
        let events_section_index = self
            .sections
            .iter()
            .position(|s| matches!(s, Section::Events(_)));

        if let Some(index) = events_section_index {
            if let Section::Events(events) = &mut self.sections[index] {
                let start_index = events.len();
                events.extend(batch.events);
                indices.extend(start_index..events.len());
            }
        } else {
            // Create new events section
            let count = batch.events.len();
            self.sections.push(Section::Events(batch.events));
            indices.extend(0..count);
        }

        indices
    }

    /// Apply a batch of mixed operations atomically
    ///
    /// All operations are validated first. If any validation fails,
    /// no changes are made. This provides transactional semantics.
    ///
    /// # Arguments
    ///
    /// * `updates` - Line updates to perform
    /// * `style_additions` - Styles to add
    /// * `event_additions` - Events to add
    ///
    /// # Returns
    ///
    /// Ok if all operations succeed, Err with the first validation error
    ///
    /// # Errors
    ///
    /// Returns error if any operation would fail, without making changes
    pub fn atomic_batch_update(
        &mut self,
        updates: Vec<UpdateOperation<'a>>,
        style_additions: Option<StyleBatch<'a>>,
        event_additions: Option<EventBatch<'a>>,
    ) -> core::result::Result<(), ParseError> {
        // First, validate all updates
        for op in &updates {
            // Check if offset is valid
            let section_found = self.sections.iter().any(|s| {
                s.span()
                    .is_some_and(|span| span.start <= op.offset && op.offset < span.end)
            });
            if !section_found {
                return Err(ParseError::SectionNotFound);
            }

            // Try parsing the line
            self.parse_line_auto(op.new_line, op.line_number)?;
        }

        // All validations passed, now apply changes
        // Clone self to preserve original state in case of failure
        let mut temp_script = self.clone();

        // Apply updates
        for op in updates {
            temp_script.update_line_at_offset(op.offset, op.new_line, op.line_number)?;
        }

        // Apply style additions
        if let Some(styles) = style_additions {
            temp_script.batch_add_styles(styles);
        }

        // Apply event additions
        if let Some(events) = event_additions {
            temp_script.batch_add_events(events);
        }

        // All operations succeeded, commit changes
        *self = temp_script;
        Ok(())
    }
}
