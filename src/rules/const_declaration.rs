//! `const-declaration` — a named constant, and where it was declared.
//!
//! A constant is a decision the program has made: a limit, a retry count, a
//! path, a key, a magic number somebody named. Scattered through a tree those
//! decisions cannot be read as a set, and the same one gets made twice under
//! two names. This rule reports each `SCREAMING_SNAKE_CASE` name at the point
//! it is *introduced*, and stops there — *where* a constant is allowed to be
//! declared is policy about a repository, so the host names those files, the
//! way it names the environment's edge for `env-read`.
//!
//! **Declarations, not uses.** This is the whole reason the rule needs a tree.
//! `MAX_SIZE` mentioned in an expression is the point of having a constant,
//! and a rule that flagged those could not be satisfied. Text cannot tell the
//! two apart without a per-language table of declaration keywords, which is a
//! parser written badly; the vocabulary answers it directly, in one form, for
//! every grammar treebank ships:
//!
//! | shape | roles | verdict |
//! |---|---|---|
//! | `MAX_SIZE = 3` | `_binding` | declared |
//! | `const MAX_SIZE: u8 = 3` | `_binding` | declared |
//! | `from os import MAX_SIZE` | `_binding` `_directive` | imported, not declared |
//! | `def f(MAX_SIZE)` | `_binding` `_parameter` | a parameter, not a constant |
//! | `n > MAX_SIZE` | none | a use |
//!
//! So the signal is one line: a `_binding` that is neither a `_directive` nor
//! a `_parameter`, whose bound name is screaming snake case.
//!
//! **A body's locals are not constants.** A name bound inside a `_callable`
//! is a local; it cannot be moved to another file and reporting it would be
//! asking for something impossible. The walk stops at every callable, which
//! also means a nested function's locals are skipped for the same reason.
//!
//! One miss is accepted rather than guessed at: an enum member written as an
//! assignment in a class body — Python's `RED_ONE = 1` inside
//! `class Colour(Enum)` — is a binding outside any callable and is reported,
//! though it cannot move. Telling an enum from a class needs the base class,
//! which is a fact about a library rather than about the tree. The host
//! licenses those the way it licenses any other file.

use crate::finding::{Finding, Rule, RuleId, Scope};
use crate::model::TestModel;
use crate::node::{Node, Unit, Visit, walk};
use crate::role::Role;

pub const RULE: Rule = Rule {
    id: RuleId::new("const-declaration"),
    // Not a test property. A scattered constant is a fact about how code is
    // arranged, and none of fidelity, resilience or precision is a claim
    // about that -- notes/DESIGN.md §5.6.
    property: None,
    scope: Scope::File,
    summary: "a named constant is declared here",
    instruction: "Declare SCREAMING_SNAKE_CASE constants where the project \
                  keeps its constants, and reference them from everywhere \
                  else, so the decisions a program has made can be read as a \
                  set rather than hunted for.",
    citation: None,
};

pub fn check<'t, N: Node<'t>>(unit: &Unit<'t, N>, _model: &TestModel, out: &mut Vec<Finding>) {
    walk(unit.root, &mut |node| {
        // A callable's body holds locals, which are nobody's to gather.
        // Its own name is checked first, then the subtree is left alone.
        let inside_a_body = node.has_role(Role::Callable);

        if node.has_role(Role::Binding)
            && !node.has_role(Role::Directive)
            && !node.has_role(Role::Parameter)
        {
            for (name, span) in bound_names(node) {
                if !screaming_snake(&name) {
                    continue;
                }
                out.push(
                    Finding::new(&RULE, span, format!("`{name}` is declared here")).with_help(
                        "declare it where the project keeps its constants and \
                         reference it from here"
                            .to_string(),
                    ),
                );
            }
        }

        if inside_a_body {
            Visit::Skip
        } else {
            Visit::Descend
        }
    });
}

/// The names a binding introduces, with where each one sits.
///
/// The grammar's `name` field is exact where a grammar has one. Otherwise the
/// target is the binding's first child that names something, and every
/// identifier under *that* child is a bound name -- which is what makes
/// `A_ONE, B_TWO = 1, 2` two findings rather than one. Reading only the first
/// target is what keeps the value on the right of the `=` out of it: in
/// `MAX_SIZE = OTHER_NAME` only `MAX_SIZE` is declared.
fn bound_names<'t, N: Node<'t>>(node: N) -> Vec<(String, crate::node::Span)> {
    if let Some(name) = node.child_by_field("name") {
        return vec![(name.text().to_string(), name.span())];
    }
    // The target is the binding's first child, and every name under it is
    // bound: `A_ONE, B_TWO = 1, 2` binds two. Reading only that child is what
    // keeps the value out of it -- in `MAX_SIZE = OTHER_LIMIT` the value is a
    // sibling, so only `MAX_SIZE` is declared. A grammar that puts a modifier
    // first yields nothing there, and the fallback scans the rest.
    if let Some(target) = node.child(0) {
        let names = names_in(target);
        if !names.is_empty() {
            return names;
        }
    }
    node.children().flat_map(names_in).take(1).collect()
}

/// Every name a subtree introduces, in source order.
fn names_in<'t, N: Node<'t>>(node: N) -> Vec<(String, crate::node::Span)> {
    let mut names = Vec::new();
    walk(node, &mut |inner| {
        if inner.has_role(Role::Name) && inner.child_count() == 0 {
            names.push((inner.text().to_string(), inner.span()));
            return Visit::Skip;
        }
        Visit::Descend
    });
    names
}

/// Whether a name is screaming snake case: at least two words, joined by
/// underscores.
///
/// The underscore is required, which is the whole difference between a rule
/// and a nuisance. A single all-caps word is ambiguous in every language that
/// has one -- `PI`, `OK`, `HTTP`, a Go export, a C header guard, a type
/// parameter -- and flagging those would bury the constants among them.
fn screaming_snake(name: &str) -> bool {
    let mut chars = name.chars();
    if !chars.next().is_some_and(|first| first.is_ascii_uppercase()) {
        return false;
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        return false;
    }
    name.contains('_') && !name.ends_with('_')
}

#[cfg(test)]
mod tests {
    use super::screaming_snake;

    #[test]
    fn a_constant_is_at_least_two_words_joined_by_underscores() {
        assert!(screaming_snake("MAX_SIZE"));
        assert!(screaming_snake("API_BASE_URL"));
        assert!(screaming_snake("HTTP_2_PORT"));
    }

    #[test]
    fn a_single_word_is_too_ambiguous_to_be_one() {
        assert!(!screaming_snake("PI"));
        assert!(!screaming_snake("HTTP"));
        assert!(!screaming_snake("X"));
    }

    #[test]
    fn a_name_that_is_not_all_caps_is_not_one() {
        assert!(!screaming_snake("max_size"));
        assert!(!screaming_snake("MaxSize"));
        assert!(!screaming_snake("Max_Size"));
        assert!(!screaming_snake("_PRIVATE_THING"));
        assert!(!screaming_snake("TRAILING_"));
    }
}
