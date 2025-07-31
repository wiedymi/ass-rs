//! Karaoke management commands for ASS karaoke timing
//!
//! Provides commands for generating, splitting, adjusting, and applying
//! ASS karaoke timing tags like \k, \kf, \ko, \kt with proper syllable
//! detection and timing validation.

use crate::core::{EditorDocument, EditorError, Position, Range, Result};
use super::{CommandResult, EditorCommand};

#[cfg(not(feature = "std"))]
use alloc::{format, string::{String, ToString}, vec, vec::Vec};

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

/// ASS karaoke tag types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KaraokeType {
    /// \k - standard karaoke (highlights during duration)
    Standard,
    /// \kf or \K - fill karaoke (sweeps from left to right)
    Fill,
    /// \ko - outline karaoke (outline changes during duration)
    Outline,
    /// \kt - transition karaoke (for advanced effects)
    Transition,
}

impl KaraokeType {
    /// Get the ASS tag string for this karaoke type
    pub fn tag_string(self) -> &'static str {
        match self {
            KaraokeType::Standard => "k",
            KaraokeType::Fill => "kf",
            KaraokeType::Outline => "ko",
            KaraokeType::Transition => "kt",
        }
    }
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
            if ch.is_whitespace() || 
               (i > 0 && self.is_syllable_boundary(&chars, i)) {
                
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
                "Cannot generate karaoke for text containing override blocks"
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

/// Split karaoke timing at specific points
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitKaraokeCommand {
    /// Range containing karaoke to split
    pub range: Range,
    /// Character positions to split at (relative to range start)
    pub split_positions: Vec<usize>,
    /// New duration for each split segment
    pub new_duration: Option<u32>,
}

impl SplitKaraokeCommand {
    /// Create a new split karaoke command
    pub fn new(range: Range, split_positions: Vec<usize>) -> Self {
        Self {
            range,
            split_positions,
            new_duration: None,
        }
    }

    /// Set new duration for split segments
    #[must_use]
    pub fn duration(mut self, duration: u32) -> Self {
        self.new_duration = Some(duration);
        self
    }
}

impl EditorCommand for SplitKaraokeCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let original_text = document.text_range(self.range)?;
        let processed_text = self.split_karaoke_text(&original_text)?;
        
        document.replace_raw(self.range, &processed_text)?;
        
        let end_pos = Position::new(self.range.start.offset + processed_text.len());
        let range = Range::new(self.range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        "Split karaoke timing"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.split_positions.len() * core::mem::size_of::<usize>()
    }
}

impl SplitKaraokeCommand {
    /// Split karaoke text at specified positions
    fn split_karaoke_text(&self, text: &str) -> Result<String> {
        // For now, return simplified version
        // In practice, this would parse existing karaoke tags and split them
        let mut result = String::new();
        let mut last_pos = 0;
        
        for &pos in &self.split_positions {
            if pos <= last_pos || pos >= text.len() {
                continue;
            }
            
            let segment = &text[last_pos..pos];
            if !segment.is_empty() {
                let duration = self.new_duration.unwrap_or(50);
                result.push_str(&format!("{{\\k{duration}}}{segment}"));
            }
            last_pos = pos;
        }
        
        // Add remaining text
        if last_pos < text.len() {
            let segment = &text[last_pos..];
            if !segment.is_empty() {
                let duration = self.new_duration.unwrap_or(50);
                result.push_str(&format!("{{\\k{duration}}}{segment}"));
            }
        }
        
        Ok(result)
    }
}

/// Adjust timing of existing karaoke tags
#[derive(Debug, Clone, PartialEq)]
pub struct AdjustKaraokeCommand {
    /// Range containing karaoke to adjust
    pub range: Range,
    /// Timing adjustment operation
    pub adjustment: TimingAdjustment,
}

/// Karaoke timing adjustment operations
#[derive(Debug, Clone, PartialEq)]
pub enum TimingAdjustment {
    /// Scale all timings by a factor (e.g., 1.2 to make 20% longer)
    Scale(f32),
    /// Add/subtract centiseconds to/from all timings
    Offset(i32),
    /// Set all timings to a specific duration
    SetAll(u32),
    /// Apply custom timing to each syllable
    Custom(Vec<u32>),
}

impl AdjustKaraokeCommand {
    /// Create a scaling adjustment command
    pub fn scale(range: Range, factor: f32) -> Self {
        Self {
            range,
            adjustment: TimingAdjustment::Scale(factor),
        }
    }

    /// Create an offset adjustment command
    pub fn offset(range: Range, offset: i32) -> Self {
        Self {
            range,
            adjustment: TimingAdjustment::Offset(offset),
        }
    }

    /// Create a set-all adjustment command
    pub fn set_all(range: Range, duration: u32) -> Self {
        Self {
            range,
            adjustment: TimingAdjustment::SetAll(duration),
        }
    }

    /// Create a custom timing adjustment command
    pub fn custom(range: Range, timings: Vec<u32>) -> Self {
        Self {
            range,
            adjustment: TimingAdjustment::Custom(timings),
        }
    }
}

impl EditorCommand for AdjustKaraokeCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let original_text = document.text_range(self.range)?;
        let adjusted_text = self.adjust_karaoke_timing(&original_text)?;
        
        document.replace_raw(self.range, &adjusted_text)?;
        
        let end_pos = Position::new(self.range.start.offset + adjusted_text.len());
        let range = Range::new(self.range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        match self.adjustment {
            TimingAdjustment::Scale(_) => "Scale karaoke timing",
            TimingAdjustment::Offset(_) => "Offset karaoke timing",
            TimingAdjustment::SetAll(_) => "Set karaoke timing",
            TimingAdjustment::Custom(_) => "Apply custom karaoke timing",
        }
    }

    fn memory_usage(&self) -> usize {
        let adjustment_size = match &self.adjustment {
            TimingAdjustment::Custom(vec) => vec.len() * core::mem::size_of::<u32>(),
            _ => 0,
        };
        core::mem::size_of::<Self>() + adjustment_size
    }
}

impl AdjustKaraokeCommand {
    /// Adjust karaoke timing in text using ass-core's ExtensionRegistry system
    fn adjust_karaoke_timing(&self, text: &str) -> Result<String> {
        use ass_core::plugin::{ExtensionRegistry, tags::karaoke::create_karaoke_handlers};
        use ass_core::analysis::events::tags::parse_override_block_with_registry;
        
        // Create registry with karaoke handlers
        let mut registry = ExtensionRegistry::new();
        for handler in create_karaoke_handlers() {
            registry.register_tag_handler(handler).map_err(|e| 
                crate::core::errors::EditorError::ValidationError { 
                    message: format!("Failed to register karaoke handler: {e:?}") 
                }
            )?;
        }
        
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        let mut custom_index = 0;
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Found override block - extract content
                let mut override_content = String::new();
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
                    override_content.push(inner_ch);
                }
                
                // Use ass-core's registry-based parser
                let mut tags = Vec::new();
                let mut diagnostics = Vec::new();
                parse_override_block_with_registry(&override_content, 0, &mut tags, &mut diagnostics, Some(&registry));
                
                // Process karaoke tags using ass-core's validated data
                let processed_content = self.adjust_karaoke_tags_with_registry(&override_content, &tags, &mut custom_index)?;
                
                result.push('{');
                result.push_str(&processed_content);
                result.push('}');
            } else {
                result.push(ch);
            }
        }
        
        Ok(result)
    }
    
    /// Adjust karaoke tags using registry-validated tag information
    fn adjust_karaoke_tags_with_registry(&self, original_content: &str, tags: &[ass_core::analysis::events::tags::OverrideTag], custom_index: &mut usize) -> Result<String> {
        let mut result = original_content.to_string();
        
        // Process tags in reverse order to maintain position accuracy
        for tag in tags.iter().rev() {
            if tag.name().starts_with('k') {
                // This tag was validated by ass-core's karaoke handlers
                let tag_name = tag.name();
                let args = tag.args();
                
                // Extract duration from args (ass-core already validated this)
                let current_duration: u32 = args.trim().parse().unwrap_or(0);
                
                // Calculate new duration based on adjustment type
                let new_duration = match &self.adjustment {
                    TimingAdjustment::Scale(factor) => ((current_duration as f32 * factor) as u32).max(1),
                    TimingAdjustment::Offset(offset) => ((current_duration as i32 + offset).max(1)) as u32,
                    TimingAdjustment::SetAll(duration) => *duration,
                    TimingAdjustment::Custom(timings) => {
                        if *custom_index < timings.len() {
                            let timing = timings[*custom_index];
                            *custom_index += 1;
                            timing
                        } else {
                            current_duration
                        }
                    }
                };
                
                // Replace the validated tag with adjusted version
                let old_tag = format!("\\{tag_name}{current_duration}");
                let new_tag = format!("\\{tag_name}{new_duration}");
                result = result.replace(&old_tag, &new_tag);
            }
        }
        
        Ok(result)
    }
    
}

/// Apply karaoke timing to event text
#[derive(Debug, Clone, PartialEq)]
pub struct ApplyKaraokeCommand {
    /// Event range to apply karaoke to
    pub event_range: Range,
    /// Karaoke template or pattern to apply
    pub karaoke_template: KaraokeTemplate,
}

/// Karaoke template for applying timing patterns
#[derive(Debug, Clone, PartialEq)]
pub enum KaraokeTemplate {
    /// Simple equal timing for all syllables
    Equal {
        syllable_duration: u32,
        karaoke_type: KaraokeType,
    },
    /// Beat-based timing (e.g., 4/4 time signature)
    Beat {
        beats_per_minute: u32,
        beats_per_syllable: f32,
        karaoke_type: KaraokeType,
    },
    /// Custom timing pattern that repeats
    Pattern {
        durations: Vec<u32>,
        karaoke_type: KaraokeType,
    },
    /// Import timing from another event
    ImportFrom {
        source_event_index: usize,
    },
}

impl ApplyKaraokeCommand {
    /// Create an equal timing command
    pub fn equal(event_range: Range, duration: u32, karaoke_type: KaraokeType) -> Self {
        Self {
            event_range,
            karaoke_template: KaraokeTemplate::Equal {
                syllable_duration: duration,
                karaoke_type,
            },
        }
    }

    /// Create a beat-based timing command
    pub fn beat(event_range: Range, bpm: u32, beats_per_syllable: f32, karaoke_type: KaraokeType) -> Self {
        Self {
            event_range,
            karaoke_template: KaraokeTemplate::Beat {
                beats_per_minute: bpm,
                beats_per_syllable,
                karaoke_type,
            },
        }
    }

    /// Create a pattern-based timing command
    pub fn pattern(event_range: Range, durations: Vec<u32>, karaoke_type: KaraokeType) -> Self {
        Self {
            event_range,
            karaoke_template: KaraokeTemplate::Pattern {
                durations,
                karaoke_type,
            },
        }
    }

    /// Create an import timing command
    pub fn import_from(event_range: Range, source_event_index: usize) -> Self {
        Self {
            event_range,
            karaoke_template: KaraokeTemplate::ImportFrom {
                source_event_index,
            },
        }
    }
}

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
            KaraokeTemplate::Pattern { durations, .. } => durations.len() * core::mem::size_of::<u32>(),
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
            KaraokeTemplate::Equal { syllable_duration, karaoke_type } => {
                self.apply_equal_timing(&syllables, *syllable_duration, *karaoke_type)
            }
            KaraokeTemplate::Beat { beats_per_minute, beats_per_syllable, karaoke_type } => {
                self.apply_beat_timing(&syllables, *beats_per_minute, *beats_per_syllable, *karaoke_type)
            }
            KaraokeTemplate::Pattern { durations, karaoke_type } => {
                self.apply_pattern_timing(&syllables, durations, *karaoke_type)
            }
            KaraokeTemplate::ImportFrom { source_event_index: _ } => {
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
    fn apply_equal_timing(&self, syllables: &[String], duration: u32, karaoke_type: KaraokeType) -> Result<String> {
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
    fn apply_beat_timing(&self, syllables: &[String], bpm: u32, beats_per_syllable: f32, karaoke_type: KaraokeType) -> Result<String> {
        // Calculate duration in centiseconds: (60 seconds / BPM) * beats_per_syllable * 100 cs/s
        let duration = ((60.0 / bpm as f32) * beats_per_syllable * 100.0) as u32;
        self.apply_equal_timing(syllables, duration, karaoke_type)
    }
    
    /// Apply pattern-based timing to syllables
    fn apply_pattern_timing(&self, syllables: &[String], durations: &[u32], karaoke_type: KaraokeType) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EditorDocument;

    #[test]
    fn generate_karaoke_basic() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(0), Position::new(11));
        let command = GenerateKaraokeCommand::new(range, 50);
        
        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert!(doc.text().contains("\\k50"));
    }

    #[test]
    fn split_karaoke() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(0), Position::new(11));
        let command = SplitKaraokeCommand::new(range, vec![5]).duration(30);
        
        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert!(doc.text().contains("\\k30"));
    }

    #[test]
    fn adjust_karaoke_scale() {
        let mut doc = EditorDocument::from_content("{\\k50}Hello").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));
        let command = AdjustKaraokeCommand::scale(range, 2.0);
        
        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert!(doc.text().contains("\\k100"));
    }

    #[test]
    fn apply_karaoke_equal() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(0), Position::new(11));
        let command = ApplyKaraokeCommand::equal(range, 40, KaraokeType::Fill);
        
        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert!(doc.text().contains("\\kf40"));
    }

    #[test]
    fn apply_karaoke_beat() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(0), Position::new(11));
        let command = ApplyKaraokeCommand::beat(range, 120, 0.5, KaraokeType::Standard);
        
        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        // Beat timing: (60/120) * 0.5 * 100 = 25 centiseconds
        assert!(doc.text().contains("\\k25"));
    }

    #[test]
    fn karaoke_types() {
        assert_eq!(KaraokeType::Standard.tag_string(), "k");
        assert_eq!(KaraokeType::Fill.tag_string(), "kf");
        assert_eq!(KaraokeType::Outline.tag_string(), "ko");
        assert_eq!(KaraokeType::Transition.tag_string(), "kt");
    }
}