//! Tests for the [`BuiltinRules`] registry facade.

use super::*;

#[test]
fn all_rules_count_correct() {
    let rules = BuiltinRules::all_rules();
    assert_eq!(rules.len(), 8);
}

#[test]
fn all_rules_have_unique_ids() {
    let rules = BuiltinRules::all_rules();
    let mut ids = alloc::vec::Vec::new();

    for rule in rules {
        let id = rule.id();
        assert!(!ids.contains(&id), "Duplicate rule ID: {id}");
        ids.push(id);
    }
}

#[test]
fn rule_by_id_works() {
    let rule = BuiltinRules::rule_by_id("timing-overlap");
    assert!(rule.is_some());
    assert_eq!(rule.unwrap().id(), "timing-overlap");

    let missing = BuiltinRules::rule_by_id("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn all_rule_ids_complete() {
    let ids = BuiltinRules::all_rule_ids();
    let expected_ids = [
        "timing-overlap",
        "negative-duration",
        "invalid-color",
        "missing-style",
        "invalid-tag",
        "performance",
        "encoding",
        "accessibility",
    ];

    for expected_id in expected_ids {
        assert!(ids.contains(&expected_id), "Missing rule ID: {expected_id}");
    }
}
