#![cfg(feature = "dev")]

//! Beamte's [`Role`] enum against treebank's own term lists.
//!
//! notes/DESIGN.md §2 says a rule is portable because the vocabulary is enforced in
//! the parse table. That only holds while beamte's enum and treebank's terms
//! agree, and the failure mode is silent: an unknown term simply never matches
//! a rule, so every test stays green while a rule quietly stops firing.

use beamte::dev::{RoleTable, python_roles};
use beamte::role::Role;

#[test]
fn every_term_treebank_declares_is_a_role_beamte_knows() {
    let table = python_roles();

    assert_eq!(
        table.unknown_terms(),
        &[] as &[String],
        "treebank declares terms beamte has no Role for"
    );
}

#[test]
fn the_table_is_built_from_the_grammars_own_manifests() {
    let table = RoleTable::from_manifests(treebank_python::NODE_TYPES, treebank_python::ROLES)
        .expect("the manifests parse");

    assert!(!table.is_empty());
}

#[test]
fn supertype_membership_is_transitive() {
    let table = python_roles();
    let roles = table.roles("while_statement");

    // while_statement derives from _loop, and _loop from _statement. Reading
    // one level of the subtype lists would find only the first.
    assert!(roles.contains(Role::Loop));
    assert!(roles.contains(Role::Statement));
}

#[test]
fn a_term_this_grammar_does_not_thread_carries_no_membership() {
    // notes/DESIGN.md §3.1.1: a term's tier is per-grammar. The Python grammar does
    // not carry _control_flow as a supertype at all -- _loop derives straight
    // from _statement -- so a rule asking for _control_flow would silently
    // never fire here. This is why `test-logic` asks for _loop and _branch.
    let table = python_roles();

    assert!(!table.roles("while_statement").contains(Role::ControlFlow));
    assert!(table.roles("while_statement").contains(Role::Loop));
}

#[test]
fn facet_roles_arrive_from_roles_json() {
    let table = python_roles();

    // _callable cross-cuts derivations, so it cannot be a supertype and is
    // only ever found in roles.json.
    assert!(table.roles("function_definition").contains(Role::Callable));
    assert!(table.roles("lambda").contains(Role::Callable));
}

#[test]
fn an_unknown_kind_carries_no_roles() {
    let table = python_roles();

    assert!(table.roles("not_a_node_kind").is_empty());
}

#[test]
fn a_role_round_trips_through_its_treebank_term() {
    let round_tripped: Vec<Role> = Role::ALL
        .into_iter()
        .filter_map(|role| Role::from_term(role.as_str()))
        .collect();

    assert_eq!(round_tripped, Role::ALL);
}
