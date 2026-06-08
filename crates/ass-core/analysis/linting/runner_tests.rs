//! Unit tests for the linting entry points.

use super::*;
use crate::analysis::{AnalysisConfig, ScriptAnalysis};
use crate::parser::Script;
use alloc::vec::Vec;

#[test]
fn lint_script_empty_script() {
    let script_content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

    let script = Script::parse(script_content).unwrap();
    let config = LintConfig::default();

    let issues = lint_script(&script, &config);
    assert!(issues.is_ok());
}

#[test]
fn lint_script_with_analysis_empty() {
    let script_content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

    let script = Script::parse(script_content).unwrap();
    let analysis = ScriptAnalysis {
        script: &script,
        lint_issues: Vec::new(),
        resolved_styles: Vec::new(),
        dialogue_info: Vec::new(),
        config: AnalysisConfig::default(),
        #[cfg(feature = "plugins")]
        registry: None,
    };

    let config = LintConfig::default();
    let issues = lint_script_with_analysis(&analysis, &config);
    assert!(issues.is_ok());
}

#[test]
fn lint_script_with_max_issues() {
    let script_content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

    let script = Script::parse(script_content).unwrap();
    let config = LintConfig::default().with_max_issues(1);

    let issues = lint_script(&script, &config);
    assert!(issues.is_ok());
    if let Ok(issues) = issues {
        assert!(issues.len() <= 1);
    }
}

#[test]
fn lint_script_with_disabled_rule() {
    // Test to cover the continue statement when rules are disabled (line 318)
    let script_content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

    let script = Script::parse(script_content).unwrap();
    let analysis = ScriptAnalysis {
        script: &script,
        lint_issues: Vec::new(),
        resolved_styles: Vec::new(),
        dialogue_info: Vec::new(),
        config: AnalysisConfig::default(),
        #[cfg(feature = "plugins")]
        registry: None,
    };

    // Create config with specific rules disabled to trigger continue path
    let mut config = LintConfig::default();
    config.disabled_rules.push("accessibility_contrast");
    config.disabled_rules.push("encoding_format");
    config.disabled_rules.push("invalid_color");

    let issues = lint_script_with_analysis(&analysis, &config);
    assert!(issues.is_ok());
}
