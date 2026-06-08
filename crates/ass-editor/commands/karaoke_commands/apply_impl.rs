//! Execution and template-application logic for [`ApplyKaraokeCommand`].

use super::{ApplyKaraokeCommand, KaraokeTemplate, KaraokeType};
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

impl EditorCommand for ApplyKaraokeCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let original_text = document.text_range(self.event_range)?;
        let karaoke_text = self.apply_karaoke_template(&original_text, document)?;

        document.replace_raw(self.event_range, &karaoke_text)?;

        let end_pos = Position::new(self.event_range.start.offset + karaoke_text.len());
        let range = Range::new(self.event_range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        match &self.karaoke_template {
            KaraokeTemplate::Equal { .. } => "Apply equal karaoke timing",
            KaraokeTemplate::Beat { .. } => "Apply beat-based karaoke timing",
            KaraokeTemplate::Pattern { .. } => "Apply pattern-based karaoke timing",
            KaraokeTemplate::ImportFrom { .. } => "Import karaoke timing",
        }
    }

    fn memory_usage(&self) -> usize {
        let template_size = match &self.karaoke_template {
            KaraokeTemplate::Pattern { durations, .. } => {
                durations.len() * core::mem::size_of::<u32>()
            }
            _ => 0,
        };
        core::mem::size_of::<Self>() + template_size
    }
}

impl ApplyKaraokeCommand {
    /// Apply karaoke template to text
    fn apply_karaoke_template(&self, text: &str, _document: &EditorDocument) -> Result<String> {
        // Extract text content from event (skip override blocks for syllable detection)
        let clean_text = self.extract_clean_text(text);
        let syllables = self.detect_syllables(&clean_text);

        match &self.karaoke_template {
            KaraokeTemplate::Equal {
                syllable_duration,
                karaoke_type,
            } => self.apply_equal_timing(&syllables, *syllable_duration, *karaoke_type),
            KaraokeTemplate::Beat {
                beats_per_minute,
                beats_per_syllable,
                karaoke_type,
            } => self.apply_beat_timing(
                &syllables,
                *beats_per_minute,
                *beats_per_syllable,
                *karaoke_type,
            ),
            KaraokeTemplate::Pattern {
                durations,
                karaoke_type,
            } => self.apply_pattern_timing(&syllables, durations, *karaoke_type),
            KaraokeTemplate::ImportFrom {
                source_event_index: _,
            } => {
                // Simplified - would need to parse other events
                Ok(text.to_string())
            }
        }
    }

    /// Extract clean text without override blocks
    fn extract_clean_text(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Skip override block
                let mut brace_count = 1;
                for inner_ch in chars.by_ref() {
                    if inner_ch == '{' {
                        brace_count += 1;
                    } else if inner_ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Detect syllables in clean text
    fn detect_syllables(&self, text: &str) -> Vec<String> {
        // Simple syllable detection - split on spaces and vowel boundaries
        text.split_whitespace()
            .flat_map(|word| {
                // For now, treat each word as one syllable
                // In practice, you'd want more sophisticated syllable detection
                vec![word.to_string()]
            })
            .collect()
    }

    /// Apply equal timing to syllables
    fn apply_equal_timing(
        &self,
        syllables: &[String],
        duration: u32,
        karaoke_type: KaraokeType,
    ) -> Result<String> {
        let tag = karaoke_type.tag_string();
        let mut result = String::new();

        for (i, syllable) in syllables.iter().enumerate() {
            if i > 0 {
                result.push(' '); // Add space between syllables
            }
            result.push_str(&format!("{{\\{tag}{duration}}}{syllable}"));
        }

        Ok(result)
    }

    /// Apply beat-based timing to syllables
    fn apply_beat_timing(
        &self,
        syllables: &[String],
        bpm: u32,
        beats_per_syllable: f32,
        karaoke_type: KaraokeType,
    ) -> Result<String> {
        // Calculate duration in centiseconds: (60 seconds / BPM) * beats_per_syllable * 100 cs/s
        let duration = ((60.0 / bpm as f32) * beats_per_syllable * 100.0) as u32;
        self.apply_equal_timing(syllables, duration, karaoke_type)
    }

    /// Apply pattern-based timing to syllables
    fn apply_pattern_timing(
        &self,
        syllables: &[String],
        durations: &[u32],
        karaoke_type: KaraokeType,
    ) -> Result<String> {
        let tag = karaoke_type.tag_string();
        let mut result = String::new();

        for (i, syllable) in syllables.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            let duration = durations.get(i % durations.len()).copied().unwrap_or(50);
            result.push_str(&format!("{{\\{tag}{duration}}}{syllable}"));
        }

        Ok(result)
    }
}
