//! Structural mutation of sections, styles, and events.
//!
//! Implements the single-item insertion and removal operations that add or
//! drop whole sections, append styles to `[V4+ Styles]`, and append events to
//! `[Events]`, creating the target section on demand.

use alloc::vec;

use crate::parser::ast::{Event, Section, Style};
use crate::parser::errors::ParseError;

use super::types::Change;
use super::Script;

impl<'a> Script<'a> {
    /// Add a new section to the script
    ///
    /// # Arguments
    ///
    /// * `section` - The section to add
    ///
    /// # Returns
    ///
    /// The index of the added section
    pub fn add_section(&mut self, section: Section<'a>) -> usize {
        let index = self.sections.len();
        self.change_tracker.record(Change::SectionAdded {
            section: section.clone(),
            index,
        });
        self.sections.push(section);
        index
    }

    /// Remove a section by index
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the section to remove
    ///
    /// # Returns
    ///
    /// The removed section if successful
    ///
    /// # Errors
    ///
    /// Returns error if index is out of bounds
    pub fn remove_section(
        &mut self,
        index: usize,
    ) -> core::result::Result<Section<'a>, ParseError> {
        if index < self.sections.len() {
            let section = self.sections.remove(index);
            self.change_tracker.record(Change::SectionRemoved {
                section_type: section.section_type(),
                index,
            });
            Ok(section)
        } else {
            Err(ParseError::IndexOutOfBounds)
        }
    }

    /// Add a style to the [V4+ Styles] section
    ///
    /// Creates the section if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `style` - The style to add
    ///
    /// # Returns
    ///
    /// The index of the style within the styles section
    pub fn add_style(&mut self, style: Style<'a>) -> usize {
        // Find or create styles section
        let styles_section_index = self
            .sections
            .iter()
            .position(|s| matches!(s, Section::Styles(_)));

        if let Some(index) = styles_section_index {
            if let Section::Styles(styles) = &mut self.sections[index] {
                styles.push(style);
                styles.len() - 1
            } else {
                unreachable!("Section type mismatch");
            }
        } else {
            // Create new styles section
            self.sections.push(Section::Styles(vec![style]));
            0
        }
    }

    /// Add an event to the `[Events\]` section
    ///
    /// Creates the section if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to add
    ///
    /// # Returns
    ///
    /// The index of the event within the events section
    pub fn add_event(&mut self, event: Event<'a>) -> usize {
        // Find or create events section
        let events_section_index = self
            .sections
            .iter()
            .position(|s| matches!(s, Section::Events(_)));

        if let Some(index) = events_section_index {
            if let Section::Events(events) = &mut self.sections[index] {
                events.push(event);
                events.len() - 1
            } else {
                unreachable!("Section type mismatch");
            }
        } else {
            // Create new events section
            self.sections.push(Section::Events(vec![event]));
            0
        }
    }
}
