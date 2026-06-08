//! Tests for bidirectional-text and complex-Unicode detection.

use super::*;

#[test]
fn text_analysis_bidi_text_arabic() {
    let text = "Hello مرحبا world";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert!(analysis.has_bidi_text());
    assert!(analysis.has_complex_unicode());
}

#[test]
fn text_analysis_bidi_text_hebrew() {
    let text = "Hello שלום world";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert!(analysis.has_bidi_text());
    assert!(analysis.has_complex_unicode());
}

#[test]
fn text_analysis_complex_unicode_emoji() {
    let text = "Hello 🌍 world";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert!(!analysis.has_bidi_text());
    assert!(analysis.has_complex_unicode());
}

#[test]
fn text_analysis_complex_unicode_control_chars() {
    let text = "Text\u{200C}with\u{200D}controls";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert!(analysis.has_complex_unicode());
}

#[test]
fn text_analysis_basic_latin_only() {
    let text = "Basic ASCII text 123!@#";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert!(!analysis.has_bidi_text());
    assert!(!analysis.has_complex_unicode());
}

#[test]
fn text_analysis_extended_latin() {
    let text = "Café naïve résumé";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert!(!analysis.has_bidi_text());
    assert!(!analysis.has_complex_unicode()); // These are still in Latin-1 range
}
