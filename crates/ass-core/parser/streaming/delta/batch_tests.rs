//! Behavioural tests for the [`DeltaBatch`] collection.

use super::*;
use crate::parser::ast::{ScriptInfo, Section, Span};
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec};

#[test]
fn delta_batch_operations() {
    let mut batch = DeltaBatch::new();
    assert!(batch.is_empty());
    assert_eq!(batch.len(), 0);

    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    batch.push(ParseDelta::add_section(section));
    batch.push(ParseDelta::parse_issue("Warning".to_string()));

    assert!(!batch.is_empty());
    assert_eq!(batch.len(), 2);
    assert!(batch.has_errors());

    let structural = batch.structural_only();
    assert_eq!(structural.len(), 1);
    assert!(!structural.has_errors());

    let errors = batch.errors_only();
    assert_eq!(errors.len(), 1);
    assert!(errors.has_errors());
}

#[test]
fn batch_from_iterator() {
    let deltas = vec![
        ParseDelta::remove_section(0),
        ParseDelta::parse_issue("Error".to_string()),
    ];

    let batch: DeltaBatch = deltas.into_iter().collect();
    assert_eq!(batch.len(), 2);
}

#[test]
fn batch_default() {
    let batch = DeltaBatch::default();
    assert!(batch.is_empty());
    assert_eq!(batch.len(), 0);
    assert!(!batch.has_errors());
}

#[test]
fn batch_debug_and_clone() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let mut batch = DeltaBatch::new();
    batch.push(ParseDelta::add_section(section));

    let debug_str = format!("{batch:?}");
    assert!(debug_str.contains("DeltaBatch"));

    let cloned = batch.clone();
    assert_eq!(batch.len(), cloned.len());
    assert_eq!(batch.is_empty(), cloned.is_empty());
}

#[test]
fn batch_extend_operations() {
    let mut batch = DeltaBatch::new();
    let section1 = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let section2 = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });

    let deltas = vec![
        ParseDelta::add_section(section1),
        ParseDelta::update_section(0, section2),
        ParseDelta::remove_section(1),
    ];

    batch.extend(deltas);
    assert_eq!(batch.len(), 3);
    assert!(!batch.has_errors());
}

#[test]
fn batch_from_deltas() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let deltas = vec![
        ParseDelta::add_section(section),
        ParseDelta::parse_issue("Warning".to_string()),
    ];

    let batch = DeltaBatch::from_deltas(deltas);
    assert_eq!(batch.len(), 2);
    assert!(batch.has_errors());
}

#[test]
fn batch_into_deltas() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let mut batch = DeltaBatch::new();
    batch.push(ParseDelta::add_section(section));
    batch.push(ParseDelta::remove_section(0));

    let deltas = batch.into_deltas();
    assert_eq!(deltas.len(), 2);
}

#[test]
fn batch_complex_filtering() {
    let section1 = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let section2 = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let mut batch = DeltaBatch::new();

    batch.push(ParseDelta::add_section(section1));
    batch.push(ParseDelta::update_section(0, section2));
    batch.push(ParseDelta::remove_section(1));
    batch.push(ParseDelta::parse_issue("Error 1".to_string()));
    batch.push(ParseDelta::parse_issue("Error 2".to_string()));

    assert_eq!(batch.len(), 5);

    let structural = batch.structural_only();
    assert_eq!(structural.len(), 3);
    assert!(!structural.has_errors());

    let errors = batch.errors_only();
    assert_eq!(errors.len(), 2);
    assert!(errors.has_errors());

    // Custom filter
    let only_adds = batch.filter(|delta| matches!(delta, ParseDelta::AddSection(_)));
    assert_eq!(only_adds.len(), 1);
}

#[test]
fn batch_empty_operations() {
    let batch = DeltaBatch::new();

    let structural = batch.structural_only();
    assert!(structural.is_empty());

    let errors = batch.errors_only();
    assert!(errors.is_empty());

    assert!(!batch.has_errors());
    assert_eq!(batch.deltas().len(), 0);
}

#[test]
fn batch_iterator_trait() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let deltas = [
        ParseDelta::add_section(section),
        ParseDelta::remove_section(0),
        ParseDelta::parse_issue("Test".to_string()),
    ];

    let batch: DeltaBatch = deltas.into_iter().collect();
    assert_eq!(batch.len(), 3);

    // Test that we can collect from any iterator
    let filtered_deltas = batch
        .deltas()
        .iter()
        .filter(|&d| d.is_structural())
        .cloned();
    let filtered_batch: DeltaBatch = filtered_deltas.collect();
    assert_eq!(filtered_batch.len(), 2);
}
