//! ASS source serialization for the [`Script`] container.
//!
//! Implements [`Script::to_ass_string`], which renders every section back to
//! canonical ASS text, honoring the stored styles and events format lines when
//! present.

use alloc::string::String;

use crate::parser::ast::Section;

use super::Script;

impl Script<'_> {
    /// Convert script to ASS string representation
    ///
    /// Generates the complete ASS script with all sections in order.
    /// Respects the stored format lines for styles and events if available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::Script;
    /// let script = Script::parse("[Script Info]\nTitle: Test").unwrap();
    /// let ass_string = script.to_ass_string();
    /// assert!(ass_string.contains("[Script Info]"));
    /// assert!(ass_string.contains("Title: Test"));
    /// ```
    #[must_use]
    pub fn to_ass_string(&self) -> alloc::string::String {
        let mut result = String::new();

        for (idx, section) in self.sections.iter().enumerate() {
            // Add newline between sections (but not before first)
            if idx > 0 {
                result.push('\n');
            }

            match section {
                Section::ScriptInfo(info) => {
                    result.push_str(&info.to_ass_string());
                }
                Section::Styles(styles) => {
                    result.push_str("[V4+ Styles]\n");

                    // Add format line if available
                    if let Some(format) = &self.styles_format {
                        result.push_str("Format: ");
                        result.push_str(&format.join(", "));
                        result.push('\n');
                    }

                    // Add each style
                    for style in styles {
                        if let Some(format) = &self.styles_format {
                            result.push_str(&style.to_ass_string_with_format(format));
                        } else {
                            result.push_str(&style.to_ass_string());
                        }
                        result.push('\n');
                    }
                }
                Section::Events(events) => {
                    result.push_str("[Events]\n");

                    // Add format line if available
                    if let Some(format) = &self.events_format {
                        result.push_str("Format: ");
                        result.push_str(&format.join(", "));
                        result.push('\n');
                    }

                    // Add each event
                    for event in events {
                        if let Some(format) = &self.events_format {
                            result.push_str(&event.to_ass_string_with_format(format));
                        } else {
                            result.push_str(&event.to_ass_string());
                        }
                        result.push('\n');
                    }
                }
                Section::Fonts(fonts) => {
                    result.push_str("[Fonts]\n");
                    for font in fonts {
                        result.push_str(&font.to_ass_string());
                    }
                }
                Section::Graphics(graphics) => {
                    result.push_str("[Graphics]\n");
                    for graphic in graphics {
                        result.push_str(&graphic.to_ass_string());
                    }
                }
            }
        }

        result
    }
}
