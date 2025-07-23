#[allow(unused)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ass_core::parser::Script;
    
    let script_text = r#"[Script Info]
Title: Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&H00FFFFFF&

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
"#;

    let script = Script::parse(script_text)?;
    println!("Successfully parsed script with {} sections", script.sections().len());
    println!("Script version: {:?}", script.version());
    
    Ok(())
}
