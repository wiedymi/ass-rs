//! Tests for style inheritance resolution across parent chains.

use super::*;

#[test]
fn analyzer_style_inheritance_basic() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: BaseStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *BaseStyle,DerivedStyle,Verdana,24,&HFF00FFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    assert_eq!(analyzer.resolved_styles().len(), 2);

    let base_style = analyzer.resolve_style("BaseStyle").unwrap();
    assert_eq!(base_style.font_name(), "Arial");
    assert!((base_style.font_size() - 20.0).abs() < f32::EPSILON);
    assert!(!base_style.is_bold());

    let derived_style = analyzer.resolve_style("DerivedStyle").unwrap();
    assert_eq!(derived_style.font_name(), "Verdana");
    assert!((derived_style.font_size() - 24.0).abs() < f32::EPSILON);
    assert!(derived_style.is_bold());
    // Should inherit colors from base
    assert_eq!(derived_style.primary_color(), [255, 255, 0, 255]); // Overridden
    assert_eq!(
        derived_style.secondary_color(),
        base_style.secondary_color()
    );
    assert_eq!(derived_style.outline_color(), base_style.outline_color());
}

#[test]
fn analyzer_style_inheritance_partial_override() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: BaseStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,1,0,0,100,100,0,0,1,2,3,2,10,10,10,1
Style: *BaseStyle,DerivedStyle,Verdana,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,2,3,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let base_style = analyzer.resolve_style("BaseStyle").unwrap();
    let derived_style = analyzer.resolve_style("DerivedStyle").unwrap();

    // Should override font name
    assert_eq!(derived_style.font_name(), "Verdana");
    // Should override font size
    assert!((derived_style.font_size() - 24.0).abs() < f32::EPSILON);
    // Should inherit colors
    assert_eq!(derived_style.primary_color(), base_style.primary_color());
    // Should inherit bold but override italic
    assert!(derived_style.is_bold());
    assert!(!derived_style.is_italic());
    // Should inherit shadow
    assert!((derived_style.shadow() - 3.0).abs() < f32::EPSILON);
}

#[test]
fn analyzer_style_inheritance_chain() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: GrandParent,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *GrandParent,Parent,Verdana,24,&H00FFFF00,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,3,0,2,15,15,15,1
Style: *Parent,Child,Times,28,&H00FF00FF,&H000000FF,&H00000000,&H00000000,1,1,0,0,100,100,0,0,1,4,0,2,20,20,20,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    assert_eq!(analyzer.resolved_styles().len(), 3);

    let grandparent = analyzer.resolve_style("GrandParent").unwrap();
    let parent = analyzer.resolve_style("Parent").unwrap();
    let child = analyzer.resolve_style("Child").unwrap();

    // GrandParent properties
    assert_eq!(grandparent.font_name(), "Arial");
    assert!((grandparent.font_size() - 20.0).abs() < f32::EPSILON);
    assert!(!grandparent.is_bold());
    assert!((grandparent.outline() - 2.0).abs() < f32::EPSILON);

    // Parent inherits and overrides
    assert_eq!(parent.font_name(), "Verdana");
    assert!((parent.font_size() - 24.0).abs() < f32::EPSILON);
    assert!(parent.is_bold());
    assert!((parent.outline() - 3.0).abs() < f32::EPSILON);
    assert_eq!(parent.margin_l(), 15);

    // Child inherits from Parent and overrides
    assert_eq!(child.font_name(), "Times");
    assert!((child.font_size() - 28.0).abs() < f32::EPSILON);
    assert!(child.is_bold());
    assert!(child.is_italic());
    assert!((child.outline() - 4.0).abs() < f32::EPSILON);
    assert_eq!(child.margin_l(), 20);
}
