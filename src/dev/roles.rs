//! Upstream node kinds mapped onto treebank roles, for the dev harness only.
//!
//! Treebank threads roles through the parse table, so a real treebank grammar
//! answers `roles()` from the tree itself and this table does not exist. It is
//! here so rules can be exercised before treebank's grammars are published,
//! and it covers only what the implemented rules ask for.
//!
//! The mapping is also the clearest illustration of why the vocabulary is
//! worth having: upstream Python calls an invocation `call`, treebank calls it
//! `call_expression`, and a rule written against `_invocation` does not care.

use crate::role::{Role, RoleSet};

/// Roles for a node kind in the upstream Python grammar.
pub fn python(kind: &str) -> RoleSet {
    use Role::*;

    match kind {
        "function_definition" => RoleSet::of(Declaration)
            .with(Callable)
            .with(Scope)
            .with(Binding)
            .with(Statement),
        "lambda" => RoleSet::of(Callable).with(Scope).with(Expression),
        "class_definition" => RoleSet::of(Declaration)
            .with(Scope)
            .with(Binding)
            .with(Statement),

        "for_statement" | "while_statement" => RoleSet::of(Loop).with(ControlFlow).with(Statement),
        "if_statement" | "match_statement" => RoleSet::of(Branch).with(ControlFlow).with(Statement),
        "conditional_expression" => RoleSet::of(Branch).with(ControlFlow).with(Expression),
        "try_statement" | "with_statement" => RoleSet::of(ControlFlow).with(Statement),
        "return_statement" | "break_statement" | "continue_statement" | "raise_statement" => {
            RoleSet::of(Jump).with(ControlFlow).with(Statement)
        }

        "call" => RoleSet::of(Invocation).with(Expression),
        "attribute" | "subscript" => RoleSet::of(Access).with(Expression),
        "assignment" | "augmented_assignment" => {
            RoleSet::of(Assignment).with(Binding).with(Expression)
        }

        "integer" | "float" | "true" | "false" | "none" => RoleSet::of(Literal).with(Expression),
        "string" | "concatenated_string" => RoleSet::of(Literal).with(Str).with(Expression),
        "identifier" => RoleSet::of(Identifier).with(Expression),
        "comment" => RoleSet::of(Comment),

        "import_statement" | "import_from_statement" => {
            RoleSet::of(Directive).with(Binding).with(Statement)
        }
        "expression_statement" | "assert_statement" | "pass_statement" => RoleSet::of(Statement),

        _ => RoleSet::empty(),
    }
}
