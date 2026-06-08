//! Generate karaoke timing tags for plain text with syllable detection.

use super::KaraokeType;
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// Generate karaoke timing tags for text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateKaraokeCommand {
    /// Range of text to add karaoke to
    pub range: Range,
    /// Default syllable duration in centiseconds
    pub default_duration: u32,
    /// Type of karaoke tag to use (\k, \kf, \ko, \kt)
    pub karaoke_type: KaraokeType,
    /// Whether to automatically detect syllables
    pub auto_detect_syllables: bool,
}

impl GenerateKaraokeCommand {
    /// Create a new generate karaoke command
    pub fn new(range: Range, default_duration: u32) -> Self {
        Self {
            range,
            default_duration,
            karaoke_type: KaraokeType::Standard,
            auto_detect_syllables: true,
        }
    }

    /// Set the karaoke type
    #[must_use]
    pub fn karaoke_type(mut self, karaoke_type: KaraokeType) -> Self {
        self.karaoke_type = karaoke_type;
        self
    }

    /// Disable automatic syllable detection
    #[must_use]
    pub fn manual_syllables(mut self) -> Self {
        self.auto_detect_syllables = false;
        self
    }

    /// Split text into syllables automatically
    fn split_into_syllables(&self, text: &str) -> Vec<String> {
        if !self.auto_detect_syllables {
            return vec![text.to_string()];
        }

        // Simple syllable detection based on vowels and common patterns
        let mut syllables = Vec::new();
        let mut current_start = 0;
        let chars: Vec<char> = text.chars().collect();

        if chars.is_empty() {
            return vec![text.to_string()];
        }

        for (i, &ch) in chars.iter().enumerate() {
            // Split on spaces and common syllable boundaries
            if ch.is_whitespace() || (i > 0 && self.is_syllable_boundary(&chars, i)) {
                if current_start < i {
                    let syllable: String = chars[current_start..i].iter().collect();
                    if !syllable.trim().is_empty() {
                        syllables.push(syllable);
                    }
                }

                // Handle whitespace
                if ch.is_whitespace() {
                    let mut end = i + 1;
                    while end < chars.len() && chars[end].is_whitespace() {
                        end += 1;
                    }
                    if end > i + 1 {
                        let whitespace: String = chars[i..end].iter().collect();
                        syllables.push(whitespace);
                        current_start = end;
                        continue;
                    }
                }

                current_start = i;
            }
        }

        // Add remaining text
        if current_start < chars.len() {
            let remaining: String = chars[current_start..].iter().collect();
            if !remaining.trim().is_empty() {
                syllables.push(remaining);
            }
        }

        // Return syllables or whole text if none found
        if syllables.is_empty() {
            vec![text.to_string()]
        } else {
            syllables
        }
    }

    /// Check if position is a syllable boundary
    fn is_syllable_boundary(&self, chars: &[char], pos: usize) -> bool {
        if pos == 0 || pos >= chars.len() {
            return false;
        }

        let prev = chars[pos - 1];
        let curr = chars[pos];

        // Split on vowel-consonant or consonant-vowel boundaries
        let prev_vowel = "aeiouAEIOU".contains(prev);
        let curr_vowel = "aeiouAEIOU".contains(curr);

        // Simple heuristic: split when transitioning from vowel to consonant
        // or when encountering certain consonant clusters
        prev_vowel && !curr_vowel && !curr.is_whitespace()
    }
}

impl EditorCommand for GenerateKaraokeCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let original_text = document.text_range(self.range)?;

        // Skip if text is already in override blocks
        if original_text.contains('{') || original_text.contains('}') {
            return Err(EditorError::command_failed(
                "Cannot generate karaoke for text containing override blocks",
            ));
        }

        let syllables = self.split_into_syllables(&original_text);
        let tag = self.karaoke_type.tag_string();

        let mut karaoke_text = String::new();
        for (i, syllable) in syllables.iter().enumerate() {
            if i == 0 {
                // First syllable gets the karaoke tag
                karaoke_text.push_str(&format!("{{\\{tag}{}}}", self.default_duration));
            } else if !syllable.trim().is_empty() {
                // Subsequent syllables get their own timing
                karaoke_text.push_str(&format!("{{\\{tag}{}}}", self.default_duration));
            }
            karaoke_text.push_str(syllable);
        }

        document.replace_raw(self.range, &karaoke_text)?;

        let end_pos = Position::new(self.range.start.offset + karaoke_text.len());
        let range = Range::new(self.range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        "Generate karaoke timing"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}
