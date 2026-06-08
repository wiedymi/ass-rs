//! Registry-aware override-block tag parser.
//!
//! Extends the standard parser with optional [`ExtensionRegistry`] support so
//! custom [`TagHandler`](crate::plugin::TagHandler) implementations can claim
//! tags before falling back to built-in complexity scoring.

use super::complexity::calculate_tag_complexity;
use super::parser::scan_tag_name;
use super::types::{DiagnosticKind, OverrideTag, TagDiagnostic};
use crate::plugin::{ExtensionRegistry, TagResult};
use alloc::vec::Vec;

/// Parse override block with extension registry support
///
/// Enhanced version of [`parse_override_block`] that can use custom tag handlers
/// from an extension registry. Unknown tags are first checked against the registry
/// before falling back to standard processing.
///
/// # Arguments
///
/// * `content` - Text content to parse (e.g., "\\b1\\i1")
/// * `start_pos` - Byte offset in the original source text
/// * `tags` - Mutable vector to collect parsed tags
/// * `diagnostics` - Mutable vector to collect parsing diagnostics
/// * `registry` - Optional registry for custom tag handlers
#[cfg(feature = "plugins")]
pub fn parse_override_block_with_registry<'a>(
    content: &'a str,
    start_pos: usize,
    tags: &mut Vec<OverrideTag<'a>>,
    diagnostics: &mut Vec<TagDiagnostic<'a>>,
    registry: Option<&ExtensionRegistry>,
) {
    let mut char_pos = 0;
    let mut byte_pos = 0;
    let chars: Vec<char> = content.chars().collect();

    while char_pos < chars.len() {
        if chars[char_pos] == '\\' {
            let tag_start = byte_pos;
            byte_pos += 1;
            char_pos += 1;

            let name_char_start = char_pos;
            let name_start = byte_pos;
            (char_pos, byte_pos) = scan_tag_name(content, &chars, char_pos, byte_pos, name_start);

            if char_pos > name_char_start {
                let name_end = byte_pos;
                let args_start = byte_pos;

                // Special handling for \t tag which can contain nested tags in parentheses
                let tag_name_preview = &content[name_start..name_end];
                if tag_name_preview == "t" && char_pos < chars.len() && chars[char_pos] == '(' {
                    // For \t tag with parentheses, we need to find the matching closing parenthesis
                    // and include everything inside, even if it contains backslashes
                    let mut paren_depth = 0;
                    while char_pos < chars.len() {
                        if chars[char_pos] == '(' {
                            paren_depth += 1;
                        } else if chars[char_pos] == ')' {
                            paren_depth -= 1;
                            if paren_depth == 0 {
                                byte_pos += 1;
                                char_pos += 1; // Include the closing parenthesis
                                break;
                            }
                        }
                        byte_pos += chars[char_pos].len_utf8();
                        char_pos += 1;
                    }
                } else {
                    // For other tags, stop at the next backslash
                    while char_pos < chars.len() && chars[char_pos] != '\\' {
                        byte_pos += chars[char_pos].len_utf8();
                        char_pos += 1;
                    }
                }

                let tag_name = &content[name_start..name_end];
                let args = &content[args_start..byte_pos];

                // Try to process with registry first
                let mut handled_by_plugin = false;
                if let Some(registry) = registry {
                    if let Some(result) = registry.process_tag(tag_name, args) {
                        match result {
                            TagResult::Processed => {
                                // Plugin successfully handled the tag
                                handled_by_plugin = true;

                                // Still add to tags for downstream analysis
                                tags.push(OverrideTag {
                                    name: tag_name,
                                    args,
                                    complexity: 2, // Default complexity for plugin tags
                                    position: start_pos + tag_start,
                                });
                            }
                            TagResult::Failed(_msg) => {
                                // Plugin failed to process the tag
                                diagnostics.push(TagDiagnostic {
                                    span: &content[tag_start..byte_pos],
                                    offset: start_pos + tag_start,
                                    kind: DiagnosticKind::MalformedTag,
                                });
                            }
                            TagResult::Ignored => {
                                // Plugin ignored the tag, fall back to standard processing
                            }
                        }
                    }
                }

                // If not handled by plugin, use standard processing
                if !handled_by_plugin {
                    let complexity = calculate_tag_complexity(tag_name);

                    if tag_name.trim().is_empty() {
                        diagnostics.push(TagDiagnostic {
                            span: &content[tag_start..byte_pos],
                            offset: start_pos + tag_start,
                            kind: DiagnosticKind::EmptyOverride,
                        });
                    } else {
                        tags.push(OverrideTag {
                            name: tag_name,
                            args,
                            complexity,
                            position: start_pos + tag_start,
                        });
                    }
                }
            } else {
                let span_end = (tag_start + 2).min(content.len());
                diagnostics.push(TagDiagnostic {
                    span: &content[tag_start..span_end],
                    offset: start_pos + tag_start,
                    kind: DiagnosticKind::EmptyOverride,
                });
                if char_pos < chars.len() {
                    byte_pos += chars[char_pos].len_utf8();
                    char_pos += 1;
                }
            }
        } else {
            byte_pos += chars[char_pos].len_utf8();
            char_pos += 1;
        }
    }
}
