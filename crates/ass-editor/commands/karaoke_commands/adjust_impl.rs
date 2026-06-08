//! Execution and timing-rewrite logic for [`AdjustKaraokeCommand`].

use super::{AdjustKaraokeCommand, TimingAdjustment};
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

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
        use ass_core::analysis::events::tags::parse_override_block_with_registry;
        use ass_core::plugin::{tags::karaoke::create_karaoke_handlers, ExtensionRegistry};

        // Create registry with karaoke handlers
        let mut registry = ExtensionRegistry::new();
        for handler in create_karaoke_handlers() {
            registry.register_tag_handler(handler).map_err(|e| {
                crate::core::errors::EditorError::ValidationError {
                    message: format!("Failed to register karaoke handler: {e:?}"),
                }
            })?;
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
                parse_override_block_with_registry(
                    &override_content,
                    0,
                    &mut tags,
                    &mut diagnostics,
                    Some(&registry),
                );

                // Process karaoke tags using ass-core's validated data
                let processed_content = self.adjust_karaoke_tags_with_registry(
                    &override_content,
                    &tags,
                    &mut custom_index,
                )?;

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
    fn adjust_karaoke_tags_with_registry(
        &self,
        original_content: &str,
        tags: &[ass_core::analysis::events::tags::OverrideTag],
        custom_index: &mut usize,
    ) -> Result<String> {
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
                    TimingAdjustment::Scale(factor) => {
                        ((current_duration as f32 * factor) as u32).max(1)
                    }
                    TimingAdjustment::Offset(offset) => {
                        ((current_duration as i32 + offset).max(1)) as u32
                    }
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
