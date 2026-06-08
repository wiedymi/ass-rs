//! Incremental reparse support for editor-driven text changes.
//!
//! Implements [`Script::affected_sections`], which maps a text change to the
//! sections it touches, and [`Script::parse_incremental`], which reparses only
//! those sections while shifting the spans of unchanged sections.

use alloc::{vec, vec::Vec};

use crate::parser::ast::{Section, SectionType};
use crate::parser::main::Parser;
use crate::Result;

use super::Script;

impl<'a> Script<'a> {
    // Incremental parsing support

    /// Determine which sections are affected by a text change
    ///
    /// # Arguments
    ///
    /// * `change` - The text change to analyze
    ///
    /// # Returns
    ///
    /// A vector of section types that are affected by the change
    #[must_use]
    pub fn affected_sections(
        &self,
        change: &crate::parser::incremental::TextChange,
    ) -> Vec<SectionType> {
        self.sections
            .iter()
            .filter(|section| {
                section.span().is_some_and(|span| {
                    let section_range = span.start..span.end;

                    // Check if change overlaps with section
                    let overlaps = change.range.start < section_range.end
                        && change.range.end > section_range.start;

                    // Also check if this is an insertion at the end of the section
                    // This handles cases like adding a new event at the end of the Events section
                    let inserts_at_end =
                        change.range.is_empty() && change.range.start == section_range.end;

                    overlaps || inserts_at_end
                })
            })
            .map(Section::section_type)
            .collect()
    }

    /// Parse only changed portions and create new Script
    ///
    /// This method performs incremental parsing by identifying affected sections
    /// and reparsing only those sections while preserving others.
    ///
    /// # Arguments
    ///
    /// * `new_source` - The complete new source text after the change
    /// * `change` - Description of what changed in the text
    ///
    /// # Returns
    ///
    /// A new Script with the changes applied
    ///
    /// # Errors
    ///
    /// Returns parse errors if affected sections cannot be reparsed
    pub fn parse_incremental(
        &self,
        new_source: &'a str,
        change: &crate::parser::incremental::TextChange,
    ) -> Result<Self> {
        use crate::parser::sections::SectionFormats;

        // Step 1: Identify affected sections
        let affected_sections = self.affected_sections(change);

        if affected_sections.is_empty() {
            // Change was in whitespace/comments only
            return Ok(Script::from_parts(
                new_source,
                self.version(),
                self.sections.clone(),
                vec![], // Clear issues, will be recalculated
                self.styles_format.clone(),
                self.events_format.clone(),
            ));
        }

        // Step 2: Build section formats from existing script
        let formats = SectionFormats {
            styles_format: self.styles_format().map(<[&str]>::to_vec),
            events_format: self.events_format().map(<[&str]>::to_vec),
        };

        // Step 3: Prepare new sections
        let mut new_sections = Vec::with_capacity(self.sections.len());

        // We need to find where each section actually starts in the document
        // including its header. The current spans only track content.
        let section_headers = [
            ("[Script Info]", SectionType::ScriptInfo),
            ("[V4+ Styles]", SectionType::Styles),
            ("[Events]", SectionType::Events),
            ("[Fonts]", SectionType::Fonts),
            ("[Graphics]", SectionType::Graphics),
        ];

        // Step 4: Process each section
        for (idx, section) in self.sections.iter().enumerate() {
            let section_type = section.section_type();

            if affected_sections.contains(&section_type) {
                // Find the section header in the new source
                let header_str = section_headers
                    .iter()
                    .find(|(_, t)| *t == section_type)
                    .map_or("[Unknown]", |(h, _)| *h);

                // Find where this section starts in the new source
                if let Some(header_pos) = new_source.find(header_str) {
                    // Find the end of this section (start of next section or end of file)
                    let section_end = if idx + 1 < self.sections.len() {
                        // Find the next section's header
                        let next_section_type = self.sections[idx + 1].section_type();
                        let next_header = section_headers
                            .iter()
                            .find(|(_, t)| *t == next_section_type)
                            .map_or("[Unknown]", |(h, _)| *h);

                        new_source[header_pos + header_str.len()..]
                            .find(next_header)
                            .map_or(new_source.len(), |pos| header_pos + header_str.len() + pos)
                    } else {
                        new_source.len()
                    };

                    // Extract the full section text including header
                    let section_text = &new_source[header_pos..section_end];

                    // Parse this section using a fresh parser
                    let parser = Parser::new(section_text);
                    let parsed_script = parser.parse();

                    // The parser returns a Script, extract sections from it
                    // We only want the one matching our type
                    if let Some(parsed_section) = parsed_script
                        .sections
                        .into_iter()
                        .find(|s| s.section_type() == section_type)
                    {
                        new_sections.push(parsed_section);
                    }
                }
            } else {
                // Section unchanged, but might need span adjustment if change was before it
                let section_span = section.span();
                if let Some(span) = section_span {
                    if change.range.end <= span.start {
                        // Change was before this section, adjust its spans
                        new_sections.push(Self::adjust_section_spans(section, change));
                    } else {
                        // Change was after this section, keep as-is
                        new_sections.push(section.clone());
                    }
                } else {
                    new_sections.push(section.clone());
                }
            }
        }

        // Step 5: Create new Script with updated sections
        Ok(Script::from_parts(
            new_source,
            self.version(),
            new_sections,
            vec![], // Issues will be recalculated
            formats.styles_format.clone(),
            formats.events_format.clone(),
        ))
    }
}
