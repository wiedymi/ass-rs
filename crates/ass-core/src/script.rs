//! High-level ASS Script representation and parser.

use std::sync::Once;

use crate::builtins;
use crate::plugin::{self, Section, SectionFactory, SectionKind};
use crate::tokenizer::{AssToken, AssTokenizer};

/// A full ASS script consisting of ordered sections.
pub struct Script {
    sections: Vec<Box<dyn Section>>,
}

impl Script {
    /// Parse a byte slice containing an ASS file into a Script instance.
    pub fn parse(src: &[u8]) -> Self {
        ensure_builtins();

        let mut sections: Vec<Box<dyn Section>> = Vec::new();
        let mut current: Option<Box<dyn Section>> = None;
        let factories = plugin::section_factories();

        let mut push_current = |cur: &mut Option<Box<dyn Section>>| {
            if let Some(sec) = cur.take() {
                sections.push(sec);
            }
        };

        let tokenizer = AssTokenizer::new(src);
        for token in tokenizer {
            match &token {
                AssToken::SectionHeader { name } => {
                    // Flush previous section.
                    push_current(&mut current);

                    // Determine SectionKind.
                    let name_str = name.as_str(src).unwrap_or("");
                    let kind = map_section_name(name_str);

                    // Create appropriate section via factory or fallback raw.
                    let section: Box<dyn Section> = match find_factory(&factories, kind) {
                        Some(factory) => factory.create(),
                        None => Box::new(RawSection::new(name_str)),
                    };
                    current = Some(section);
                }
                _ => {
                    if let Some(ref mut sec) = current {
                        sec.parse_token(&token, src);
                    }
                }
            }
        }
        // push last
        push_current(&mut current);

        Script { sections }
    }

    /// Serialize the script back to ASS text.
    pub fn serialize(&self) -> String {
        let mut out = String::new();
        for (idx, sec) in self.sections.iter().enumerate() {
            if idx > 0 {
                // Ensure blank line between sections (common in ASS files).
                if !out.ends_with('\n') {
                    out.push('\n');
                }
            }
            sec.serialize(&mut out);
            // Ensure newline at end of section.
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }

    pub fn sections(&self) -> &[Box<dyn Section>] {
        &self.sections
    }

    /// Get a simple view of events for testing purposes.
    /// Returns a Vec with length equal to the number of dialogue lines.
    pub fn events(&self) -> Vec<String> {
        use crate::plugin::SectionKind;

        for section in &self.sections {
            if let SectionKind::Events = section.kind() {
                // Count dialogue lines in the events section
                let events_section = section.as_ref();
                // This is a bit of a hack - we need to access the raw_lines
                // For now, let's serialize and count dialogue lines
                let mut serialized = String::new();
                events_section.serialize(&mut serialized);

                return serialized
                    .lines()
                    .filter(|line| line.trim().starts_with("Dialogue:"))
                    .map(|line| line.to_string())
                    .collect();
            }
        }
        Vec::new()
    }
}

// -------------------------------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------------------------------

fn map_section_name(name: &str) -> SectionKind {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "script info" => SectionKind::ScriptInfo,
        "v4 styles" | "v4+ styles" | "v4+styles" => SectionKind::V4Styles,
        "events" => SectionKind::Events,
        _ => {
            // Leak the name to obtain 'static str for SectionKind::Custom.
            let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
            SectionKind::Custom(leaked)
        }
    }
}

fn find_factory<'a>(
    factories: &'a [&'static dyn SectionFactory],
    kind: SectionKind,
) -> Option<&'a &'static dyn SectionFactory> {
    factories.iter().find(|f| f.kind() == kind)
}

/// Ensure built-in plugins are registered exactly once.
fn ensure_builtins() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        builtins::register_builtins();
    });
}

// -------------------------------------------------------------------------------------------------
// Raw fallback section for custom/unknown kinds
// -------------------------------------------------------------------------------------------------

struct RawSection {
    header: &'static str,
    lines: Vec<String>,
}

impl RawSection {
    fn new(name: &str) -> Self {
        Self {
            header: Box::leak(name.to_owned().into_boxed_str()),
            lines: Vec::new(),
        }
    }
}

impl Section for RawSection {
    fn kind(&self) -> SectionKind {
        SectionKind::Custom(self.header)
    }

    fn parse_token(&mut self, token: &AssToken, src: &[u8]) {
        match token {
            AssToken::Raw { span }
            | AssToken::KeyValue {
                key: span,
                value: _,
            }
            | AssToken::Comment { span } => {
                if let Some(s) = span.as_str(src) {
                    self.lines.push(s.to_owned());
                }
            }
            AssToken::Empty => {
                // Represent empty line explicitly
                self.lines.push(String::new());
            }
            _ => {}
        }
    }

    fn serialize(&self, dst: &mut String) {
        dst.push('[');
        dst.push_str(self.header);
        dst.push_str("]\n");
        for l in &self.lines {
            dst.push_str(l);
            dst.push('\n');
        }
    }
}
