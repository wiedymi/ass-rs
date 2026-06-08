//! Span-validation tests for the `Style` AST node (debug builds only).

use super::super::Span;
use super::*;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(debug_assertions)]
#[test]
fn style_validate_spans() {
    let source = "Default,Arial,20,&Hffffff,&H0000ff,&H000000,&H000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1";
    let source_start = source.as_ptr() as usize;
    let source_end = source_start + source.len();
    let source_range = source_start..source_end;

    // Parse all fields from the source to ensure all references are within range
    let fields: Vec<&str> = source.split(',').collect();
    assert_eq!(fields.len(), 23); // ASS v4+ style has 23 fields

    // Create style with all references within the source range
    let style = Style {
        name: fields[0],
        parent: None,
        fontname: fields[1],
        fontsize: fields[2],
        primary_colour: fields[3],
        secondary_colour: fields[4],
        outline_colour: fields[5],
        back_colour: fields[6],
        bold: fields[7],
        italic: fields[8],
        underline: fields[9],
        strikeout: fields[10],
        scale_x: fields[11],
        scale_y: fields[12],
        spacing: fields[13],
        angle: fields[14],
        border_style: fields[15],
        outline: fields[16],
        shadow: fields[17],
        alignment: fields[18],
        margin_l: fields[19],
        margin_r: fields[20],
        margin_v: fields[21],
        margin_t: None,
        margin_b: None,
        encoding: fields[22],
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    // Actually call the validate_spans method
    assert!(style.validate_spans(&source_range));

    // Verify the fields are correct
    assert_eq!(style.name, "Default");
    assert_eq!(style.fontname, "Arial");
    assert_eq!(style.fontsize, "20");
}

#[cfg(debug_assertions)]
#[test]
fn style_validate_spans_invalid() {
    let source1 = "Default,Arial,20";
    let source2 = "Other,Times,16";
    let source1_start = source1.as_ptr() as usize;
    let source1_end = source1_start + source1.len();
    let source1_range = source1_start..source1_end;

    // Create style with references from different source
    let style = Style {
        name: &source2[0..5], // "Other" - from different source
        parent: None,
        fontname: &source1[8..13],  // "Arial" - from source1
        fontsize: &source1[14..16], // "20" - from source1
        ..Style::default()
    };

    // This should fail since name is from different source
    assert!(!style.validate_spans(&source1_range));
}

#[cfg(debug_assertions)]
#[test]
fn style_validate_spans_comprehensive() {
    let source = "Name,Font,Size,Primary,Secondary,Outline,Back,Bold,Italic,Under,Strike,ScX,ScY,Sp,Ang,Border,Out,Shad,Align,ML,MR,MV,Enc";
    let source_start = source.as_ptr() as usize;
    let source_end = source_start + source.len();
    let source_range = source_start..source_end;

    let fields: Vec<&str> = source.split(',').collect();
    let style = Style {
        name: fields[0],
        parent: None,
        fontname: fields[1],
        fontsize: fields[2],
        primary_colour: fields[3],
        secondary_colour: fields[4],
        outline_colour: fields[5],
        back_colour: fields[6],
        bold: fields[7],
        italic: fields[8],
        underline: fields[9],
        strikeout: fields[10],
        scale_x: fields[11],
        scale_y: fields[12],
        spacing: fields[13],
        angle: fields[14],
        border_style: fields[15],
        outline: fields[16],
        shadow: fields[17],
        alignment: fields[18],
        margin_l: fields[19],
        margin_r: fields[20],
        margin_v: fields[21],
        margin_t: None,
        margin_b: None,
        encoding: fields[22],
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    // Should validate successfully since all fields are from source
    assert!(style.validate_spans(&source_range));
}
