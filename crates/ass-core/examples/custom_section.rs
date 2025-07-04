use ass_core::plugin::{register_section, Section, SectionFactory, SectionKind};
use ass_core::tokenizer::AssToken;
use ass_core::Script;

// -------------------------------------------------------------------------------------------------
// Custom section implementation
// -------------------------------------------------------------------------------------------------

struct CustomData {
    lines: Vec<String>,
}

impl Section for CustomData {
    fn kind(&self) -> SectionKind {
        // Use a unique static name for Custom section kind
        SectionKind::Custom("Custom Data")
    }

    fn parse_token(&mut self, token: &AssToken, src: &[u8]) {
        if let AssToken::Raw { span } = token {
            if let Some(s) = span.as_str(src) {
                self.lines.push(s.to_owned());
            }
        }
    }

    fn serialize(&self, dst: &mut String) {
        dst.push_str("[Custom Data]\n");
        for l in &self.lines {
            dst.push_str(l);
            dst.push('\n');
        }
    }
}

struct CustomDataFactory;
impl SectionFactory for CustomDataFactory {
    fn create(&self) -> Box<dyn Section> {
        Box::new(CustomData { lines: Vec::new() })
    }
}

fn main() {
    // Register our custom section plugin *before* parsing.
    static FACTORY: CustomDataFactory = CustomDataFactory;
    register_section(&FACTORY);

    // Example ASS snippet containing our new section.
    let ass_text = b"[Script Info]\nTitle: Demo\n\n[Custom Data]\nfoo\nbar\n";

    let script = Script::parse(ass_text);
    for sec in script.sections() {
        if let SectionKind::Custom(name) = sec.kind() {
            println!("Found custom section: {name}");
        }
    }

    println!("Round-trip:\n{}", script.serialize());
}
