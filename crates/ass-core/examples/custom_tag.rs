use ass_core::plugin::{register_tag, Tag, TagArgument, TagParseError};
use ass_core::{override_parser, Span};

// Simple tag plugin that just logs its arguments when parsed.
struct HelloTag;
impl Tag for HelloTag {
    fn name(&self) -> &'static str {
        "hello"
    }

    fn parse_args(&self, args: &[u8]) -> Result<Vec<TagArgument>, TagParseError> {
        println!(
            "HelloTag args: {}",
            std::str::from_utf8(args).unwrap_or("<non-utf8>")
        );
        // Return empty vector of arguments for this simple example
        Ok(vec![])
    }
}

fn main() {
    // Register plugin.
    static HELLO: HelloTag = HelloTag;
    register_tag(&HELLO);

    let block_bytes = b"\\hello(world)\\b1"; // represents {\hello(world)\b1}
    let span = Span {
        start: 0,
        end: block_bytes.len(),
    };
    let tags = override_parser::parse_override_block(block_bytes, span);
    for t in tags {
        if let Some(tag) = t.plugin {
            println!("Found tag: {}", tag.name());
            match tag.parse_args(&block_bytes[t.args.start..t.args.end]) {
                Ok(args) => println!("Parsed {} arguments successfully", args.len()),
                Err(e) => println!("Failed to parse arguments: {:?}", e),
            }
        } else {
            println!("Unknown tag");
        }
    }
}
