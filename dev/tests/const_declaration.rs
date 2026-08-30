//! `const-declaration` over real parsed Python.
//!
//! The whole reason this rule needs a tree rather than a regex is the
//! difference between declaring a name and using one, so most of this file is
//! that distinction from several directions.

use beamte::node::Unit;
use beamte::{Finding, Selection, TestModel, inspect_with};
use beamte_dev::Parsed;

fn findings(source: &str) -> Vec<Finding> {
    let parsed = Parsed::python(source).expect("the fixture parses");
    let unit = Unit::new("fixture.py", parsed.source(), parsed.root());
    inspect_with(
        &unit,
        &TestModel::python(),
        Selection::Only(&[beamte::rules::const_declaration::RULE.id]),
    )
}

fn names(source: &str) -> Vec<String> {
    findings(source)
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn flags_a_declaration_and_points_at_the_name() {
    let found = findings("MAX_SIZE = 3\n");

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule.as_str(), "const-declaration");
    assert_eq!(found[0].message, "`MAX_SIZE` is declared here");
    assert_eq!(found[0].span.line, 1);
    assert_eq!(found[0].span.column, 1);
}

#[test]
fn a_use_is_not_a_declaration() {
    assert_eq!(
        names("def f(n):\n    return n > MAX_SIZE and OTHER_LIMIT\n"),
        Vec::<String>::new(),
        "flagging uses would make the rule impossible to satisfy"
    );
}

#[test]
fn an_import_binds_a_name_without_declaring_it() {
    assert_eq!(names("from os import MAX_IMPORTED\n"), Vec::<String>::new());
    assert_eq!(names("import SOME_MODULE\n"), Vec::<String>::new());
}

#[test]
fn a_local_inside_a_body_is_nobodys_to_gather() {
    assert_eq!(
        names("def f():\n    LOCAL_MAX = 4\n    return LOCAL_MAX\n"),
        Vec::<String>::new()
    );
}

#[test]
fn a_parameter_is_not_a_constant() {
    assert_eq!(
        names("def f(MAX_SIZE):\n    return MAX_SIZE\n"),
        Vec::<String>::new()
    );
}

#[test]
fn a_single_word_name_is_too_ambiguous_to_flag() {
    assert_eq!(names("PI = 3.14\nHTTP = 1\n"), Vec::<String>::new());
}

#[test]
fn a_lowercase_binding_is_not_a_constant() {
    assert_eq!(names("max_size = 3\nMaxSize = 4\n"), Vec::<String>::new());
}

#[test]
fn the_value_side_is_not_a_declaration() {
    let found = names("MAX_SIZE = OTHER_LIMIT\n");

    assert_eq!(found, ["`MAX_SIZE` is declared here"]);
}

#[test]
fn several_targets_on_one_line_are_each_declared() {
    let found = names("A_ONE, B_TWO = 1, 2\n");

    assert_eq!(
        found,
        ["`A_ONE` is declared here", "`B_TWO` is declared here"]
    );
}

#[test]
fn a_class_level_constant_is_declared() {
    let found = names("class Settings:\n    MAX_SIZE = 3\n");

    assert_eq!(found, ["`MAX_SIZE` is declared here"]);
}

#[test]
fn the_rule_reads_any_file_rather_than_only_tests() {
    use beamte::Scope;

    let rule = beamte::rule("const-declaration").expect("the rule exists");

    assert_eq!(rule.scope, Scope::File);
    assert_eq!(rule.property, None, "not a claim about a test");
    assert!(rule.citation.is_none(), "states a structural fact");
}
