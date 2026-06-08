//! Tests for inheritance conflict detection and inheritance metadata tracking.

use super::*;
use crate::analysis::styles::validation::ConflictType;

#[test]
fn analyzer_style_inheritance_missing_parent() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: *NonExistent,Orphan,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    // Should still resolve the style without inheritance
    assert_eq!(analyzer.resolved_styles().len(), 1);
    let orphan = analyzer.resolve_style("Orphan").unwrap();
    assert_eq!(orphan.font_name(), "Arial");

    // Should have a conflict for missing parent
    let conflicts = analyzer.conflicts();
    assert!(!conflicts.is_empty());
    assert!(conflicts
        .iter()
        .any(|c| matches!(c.conflict_type, ConflictType::MissingReference)));
}

#[test]
fn analyzer_style_circular_inheritance() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: *StyleB,StyleA,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *StyleA,StyleB,Verdana,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    // Should still resolve styles without inheritance due to circular dependency
    assert_eq!(analyzer.resolved_styles().len(), 2);

    // Should detect circular inheritance
    let conflicts = analyzer.conflicts();
    assert!(conflicts
        .iter()
        .any(|c| matches!(c.conflict_type, ConflictType::CircularInheritance)));
}

#[test]
fn analyzer_style_self_inheritance() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: *SelfRef,SelfRef,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    // Should resolve without inheritance due to self-reference
    assert_eq!(analyzer.resolved_styles().len(), 1);

    // Should detect circular inheritance
    let conflicts = analyzer.conflicts();
    assert!(conflicts
        .iter()
        .any(|c| matches!(c.conflict_type, ConflictType::CircularInheritance)));
}

#[test]
fn analyzer_inheritance_info_tracking() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: BaseStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *BaseStyle,Child1,Verdana,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *BaseStyle,Child2,Times,18,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let inheritance_info = analyzer.inheritance_info();
    assert_eq!(inheritance_info.len(), 3);

    // Check BaseStyle has no parent
    let base_info = inheritance_info.get("BaseStyle").unwrap();
    assert!(base_info.is_root());
    assert!(base_info.parents.is_empty());

    // Check Child1 has BaseStyle as parent
    let child1_info = inheritance_info.get("Child1").unwrap();
    assert!(!child1_info.is_root());
    assert_eq!(child1_info.parents.len(), 1);
    assert_eq!(child1_info.parents[0], "BaseStyle");

    // Check Child2 has BaseStyle as parent
    let child2_info = inheritance_info.get("Child2").unwrap();
    assert!(!child2_info.is_root());
    assert_eq!(child2_info.parents.len(), 1);
    assert_eq!(child2_info.parents[0], "BaseStyle");
}
