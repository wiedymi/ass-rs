//! Parser for override blocks (e.g. `{\b1\i1}`) into tag instances.
//! This keeps zero-copy references into the original ASS line buffer.

use crate::plugin::{self, Tag};
use crate::tokenizer::Span;

#[derive(Clone, Copy)]
pub struct TagInstance {
    pub plugin: Option<&'static dyn Tag>,
    /// Span of arguments bytes (may be empty) relative to original script slice.
    pub args: Span,
}

impl core::fmt::Debug for TagInstance {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(tag) = self.plugin {
            write!(f, "TagInstance {{ name: {} }}", tag.name())
        } else {
            write!(f, "TagInstance {{ name: <unknown> }}")
        }
    }
}

/// Parse one override block (inside { }) and return tag instances
pub fn parse_override_block(src: &[u8], span: Span) -> Vec<TagInstance> {
    let mut out = Vec::new();
    let mut i = span.start;
    let end = span.end;

    while i < end {
        if src[i] != b'\\' {
            // Skip until next backslash.
            i += 1;
            continue;
        }
        // start of tag
        let name_start = i + 1;
        let mut name_end = name_start;
        while name_end < end {
            let c = src[name_end];
            if (c as char).is_ascii_alphabetic() {
                name_end += 1;
            } else {
                break;
            }
        }
        if name_start == name_end {
            i += 1;
            continue;
        }
        let tag_name = &src[name_start..name_end];
        let lower_name = to_ascii_lowercase_bytes(tag_name);
        // look for args end (next backslash or end)
        let mut args_end = name_end;
        while args_end < end && src[args_end] != b'\\' {
            args_end += 1;
        }

        let arg_span = Span {
            start: name_end,
            end: args_end,
        };

        let plugin = plugin::get_tag(core::str::from_utf8(&lower_name).unwrap_or(""));
        out.push(TagInstance {
            plugin,
            args: arg_span,
        });

        i = args_end;
    }

    out
}

fn to_ascii_lowercase_bytes(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().map(|b| b.to_ascii_lowercase()).collect()
}
