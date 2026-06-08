//! Standard override-block tag parser.
//!
//! Walks a tag block character-by-character, extracting [`OverrideTag`]s and
//! collecting [`TagDiagnostic`]s for malformed or empty override syntax without
//! allocating beyond the supplied output vectors.

use super::complexity::calculate_tag_complexity;
use super::types::{DiagnosticKind, OverrideTag, TagDiagnostic};
use alloc::vec::Vec;

/// Parse override tags within a tag block
///
/// Extracts and analyzes all override tags found within an ASS tag block.
/// Handles malformed syntax gracefully by collecting diagnostics rather than failing.
///
/// # Arguments
///
/// * `content` - Text content within the override block (without braces)
/// * `start_pos` - Byte offset of the block start in original text
/// * `tags` - Vector to collect parsed tags
/// * `diagnostics` - Vector to collect parsing issues
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::tags::{parse_override_block, OverrideTag, TagDiagnostic};
/// let mut tags = Vec::new();
/// let mut diagnostics = Vec::new();
///
/// parse_override_block("\\b1\\i1", 0, &mut tags, &mut diagnostics);
/// assert_eq!(tags.len(), 2);
/// ```
/// Scan an override tag name, returning the position just past it
///
/// A tag name is an optional leading digit (for color/alpha tags like `\1c`)
/// followed by ASCII letters. `\r` (reset style) and `\fn` (font name) are the
/// only tags whose arguments may begin with ASCII letters without a delimiter
/// (e.g. `\fnArial`, `\rDefault`), so scanning stops as soon as either name is
/// matched to avoid swallowing the argument into the name.
pub(super) fn scan_tag_name(
    content: &str,
    chars: &[char],
    mut char_pos: usize,
    mut byte_pos: usize,
    name_start: usize,
) -> (usize, usize) {
    let mut tag_name_len = 0;
    while char_pos < chars.len() {
        if tag_name_len == 0 && chars[char_pos].is_ascii_digit() {
            byte_pos += 1;
            char_pos += 1;
            tag_name_len += 1;
        } else if chars[char_pos].is_ascii_alphabetic() {
            byte_pos += 1;
            char_pos += 1;
            tag_name_len += 1;

            let name_so_far = &content[name_start..byte_pos];
            if name_so_far == "r" || name_so_far == "fn" {
                break;
            }
        } else {
            break;
        }
    }
    (char_pos, byte_pos)
}

pub fn parse_override_block<'a>(
    content: &'a str,
    start_pos: usize,
    tags: &mut Vec<OverrideTag<'a>>,
    diagnostics: &mut Vec<TagDiagnostic<'a>>,
) {
    let mut char_pos = 0;
    let mut byte_pos = 0;
    let chars: Vec<char> = content.chars().collect();

    while char_pos < chars.len() {
        if chars[char_pos] == '\\' {
            let tag_start = byte_pos;
            byte_pos += 1; // '\' is ascii, 1 byte in utf8
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
                                byte_pos += 1; // Include the closing parenthesis
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
