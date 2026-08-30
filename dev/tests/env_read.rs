//! `env-read` over real parsed Python.
//!
//! One language is exercised against a real tree; the other languages'
//! surfaces are data, tested as data in the rule's own module. Straitjacket
//! runs the same rule through wasm packs, which is where a second grammar
//! first meets it.

use beamte::node::Unit;
use beamte::{Finding, Selection, TestModel, inspect_with};
use beamte_dev::Parsed;

fn findings(source: &str) -> Vec<Finding> {
    let parsed = Parsed::python(source).expect("the fixture parses");
    let unit = Unit::new("fixture.py", parsed.source(), parsed.root());
    inspect_with(
        &unit,
        &TestModel::python(),
        Selection::Only(&[beamte::rules::env_read::RULE.id]),
    )
}

#[test]
fn flags_a_getenv_call_and_names_the_variable() {
    let found = findings(
        r#"
def load():
    return os.getenv("SLOTH_WALKS")
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule.as_str(), "env-read");
    assert_eq!(
        found[0].message,
        "`SLOTH_WALKS` read from the process environment (`os.getenv`)"
    );
    assert_eq!(found[0].span.line, 3);
    assert_eq!(found[0].property, Some(beamte::Property::Resilience));
    assert!(found[0].help.is_some());
}

#[test]
fn flags_a_subscript_read_once_not_once_per_layer() {
    let found = findings(
        r#"
def load():
    return os.environ["PATH"]
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].message,
        "`PATH` read from the process environment (`os.environ`)"
    );
}

#[test]
fn flags_a_read_through_a_bare_import() {
    let found = findings(
        r#"
from os import environ

def load():
    return environ["TERM"]
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].message,
        "`TERM` read from the process environment (`environ`)"
    );
}

#[test]
fn flags_a_read_with_no_literal_name_without_guessing_one() {
    let found = findings(
        r#"
def load(name):
    return os.environ.get(name)
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].message,
        "the process environment read through `os.environ.get`"
    );
}

#[test]
fn a_literal_default_after_a_dynamic_name_does_not_name_the_read() {
    let found = findings(
        r#"
def load(name):
    return os.environ.get(name, "sh")
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].message, "the process environment read through `os.environ.get`",
        "the read is of `name`, and `sh` must not be claimed as it"
    );
}

#[test]
fn a_mention_in_a_comment_or_a_string_is_not_a_read() {
    let found = findings(
        r#"
def describe():
    # os.getenv("HOME") would be wrong here
    return "the os.environ['PATH'] table"
"#,
    );

    assert_eq!(found, Vec::new());
}

#[test]
fn an_unrelated_call_is_not_a_read() {
    let found = findings(
        r#"
def load(config):
    return config.getenv_table["PATH"]
"#,
    );

    assert_eq!(found, Vec::new());
}

#[test]
fn fires_inside_a_test_body_too() {
    let found = findings(
        r#"
def test_respects_the_home_directory():
    assert loader(os.environ["HOME"]).ok
"#,
    );

    assert_eq!(found.len(), 1);
}

#[test]
fn a_language_with_no_surface_yields_nothing_rather_than_guesses() {
    let parsed = Parsed::python("x = os.getenv(\"HOME\")\n").expect("the fixture parses");
    let unit = Unit::new("fixture.sh", parsed.source(), parsed.root());

    let found = inspect_with(
        &unit,
        &TestModel::bash(),
        Selection::Only(&[beamte::rules::env_read::RULE.id]),
    );

    assert_eq!(found, Vec::new());
    assert!(!beamte::rules::env_read::covers("bash"));
}

#[test]
fn the_default_selection_includes_the_rule_and_except_removes_it() {
    let parsed = Parsed::python("x = os.getenv(\"HOME\")\n").expect("the fixture parses");
    let unit = Unit::new("fixture.py", parsed.source(), parsed.root());
    let model = TestModel::python();

    let all = beamte::inspect(&unit, &model);
    assert_eq!(all.len(), 1, "Selection::All runs every rule, this one too");

    let none = inspect_with(
        &unit,
        &model,
        Selection::Except(&[beamte::rules::env_read::RULE.id]),
    );
    assert_eq!(none, Vec::new());
}
