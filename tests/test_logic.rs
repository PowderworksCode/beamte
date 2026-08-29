#![cfg(feature = "dev")]

//! `test-logic` over real parsed Python.
//!
//! Fixtures are written DAMP — literal, self-contained, and with no loops or
//! conditionals of their own. A fixture corpus for a rule that bans logic in
//! tests must not contain logic in tests.

use beamte::dev::Parsed;
use beamte::node::Unit;
use beamte::{Finding, TestModel, inspect};

fn findings(source: &str) -> Vec<Finding> {
    let parsed = Parsed::python(source).expect("the fixture parses");
    let unit = Unit::new("fixture.py", parsed.source(), parsed.root());
    inspect(&unit, &TestModel::python())
}

#[test]
fn flags_a_for_loop_in_a_test_body() {
    let found = findings(
        r#"
def test_registers_every_user():
    for user in users:
        forum.register(user)
    assert forum.count() == 2
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule.as_str(), "test-logic");
    assert_eq!(found[0].message, "loop (`for_statement`) in a test body");
    assert_eq!(found[0].span.line, 3);
}

#[test]
fn flags_a_while_loop_in_a_test_body() {
    let found = findings(
        r#"
def test_drains_the_queue():
    while queue.peek():
        queue.pop()
    assert queue.empty()
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].message, "loop (`while_statement`) in a test body");
}

#[test]
fn flags_a_conditional_in_a_test_body() {
    let found = findings(
        r#"
def test_reports_the_balance():
    if account.is_open():
        assert account.balance() == 5
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].message,
        "conditional (`if_statement`) in a test body"
    );
}

#[test]
fn reports_the_outermost_loop_only() {
    let found = findings(
        r#"
def test_visits_every_cell():
    for row in grid:
        for cell in row:
            visited.add(cell)
    assert len(visited) == 4
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].span.line, 3);
}

#[test]
fn reports_each_independent_loop() {
    let found = findings(
        r#"
def test_registers_then_removes():
    for user in users:
        forum.register(user)
    for user in users:
        forum.remove(user)
    assert forum.count() == 0
"#,
    );

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].span.line, 3);
    assert_eq!(found[1].span.line, 5);
}

#[test]
fn accepts_a_test_that_states_its_cases_directly() {
    let found = findings(
        r#"
def test_registers_alice():
    forum.register(alice)
    assert forum.has_registered(alice)

def test_registers_bob():
    forum.register(bob)
    assert forum.has_registered(bob)
"#,
    );

    assert!(found.is_empty(), "expected no findings, got {found:?}");
}

#[test]
fn ignores_logic_outside_a_test() {
    let found = findings(
        r#"
def build_users(names):
    for name in names:
        yield User(name)

def test_registers_alice():
    forum.register(alice)
    assert forum.has_registered(alice)
"#,
    );

    assert!(found.is_empty(), "expected no findings, got {found:?}");
}

#[test]
fn flags_logic_in_a_helper_nested_inside_a_test() {
    let found = findings(
        r#"
def test_registers_every_user():
    def register_all(people):
        for person in people:
            forum.register(person)
    register_all(users)
    assert forum.count() == 2
"#,
    );

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].span.line, 4);
}

#[test]
fn a_finding_carries_its_property_and_help() {
    let found = findings(
        r#"
def test_registers_every_user():
    for user in users:
        forum.register(user)
"#,
    );

    assert_eq!(found[0].property, beamte::Property::Precision);
    assert!(found[0].help.is_some());
}

#[test]
fn every_rule_cites_the_post_it_restates() {
    let catalogue = beamte::catalogue();

    assert!(!catalogue.is_empty());
    assert_eq!(catalogue[0].id.as_str(), "test-logic");
    assert_eq!(catalogue[0].citation.date, "2014-07-31");
    assert!(
        catalogue[0]
            .citation
            .url
            .starts_with("https://testing.googleblog.com/")
    );
}
