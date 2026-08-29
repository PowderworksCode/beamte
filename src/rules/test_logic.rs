//! `test-logic` — Don't Put Logic in Tests (2014-07-31).
//!
//! > Unlike production code, simplicity is more important than flexibility in
//! > tests. Most unit tests verify that a single, known input produces a
//! > single, known output. Tests can avoid complexity by stating their inputs
//! > and outputs directly rather than computing them. Otherwise it's easy for
//! > tests to develop their own bugs.
//!
//! This implements the control-flow half of the signal in DESIGN.md §5.1: a
//! `_loop` or `_branch` under a test `_callable`. The other half — an
//! arithmetic or concatenation operator in an assertion argument — needs a
//! notion of "operator" that the treebank vocabulary does not currently
//! carry, and is not implemented here.
//!
//! Only the outermost occurrence is reported. A doubly nested loop is one
//! problem, not two, and the fix for the outer one removes both.

use crate::finding::{Citation, Finding, Property, Rule, RuleId};
use crate::model::TestModel;
use crate::node::{Node, Unit, Visit, walk};
use crate::role::Role;

pub const RULE: Rule = Rule {
    id: RuleId::new("test-logic"),
    property: Property::Precision,
    summary: "a test computes its own expectations instead of stating them",
    instruction: "Do not put loops or conditionals in a test body. A test is a \
                  concrete input/output pair: state the values directly rather \
                  than computing them, and split the cases into separate tests.",
    citation: Citation {
        title: "Testing on the Toilet: Don't Put Logic in Tests",
        url: "https://testing.googleblog.com/2014/07/testing-on-toilet-dont-put-logic-in.html",
        date: "2014-07-31",
    },
};

pub fn check<'t, N: Node<'t>>(unit: &Unit<'t, N>, model: &TestModel, out: &mut Vec<Finding>) {
    walk(unit.root, &mut |node| {
        if !model.is_test(node) {
            return Visit::Descend;
        }
        check_body(node, out);
        // The test's own subtree has been walked by `check_body`, and a
        // callable nested inside a test is part of that test.
        Visit::Skip
    });
}

fn check_body<'t, N: Node<'t>>(test: N, out: &mut Vec<Finding>) {
    let mut first = true;
    walk(test, &mut |node| {
        // `test` itself carries no control-flow role, but skipping the root
        // explicitly keeps the intent legible.
        if first {
            first = false;
            return Visit::Descend;
        }
        let Some(kind) = Logic::of(node) else {
            return Visit::Descend;
        };
        out.push(
            Finding::new(&RULE, node.span(), kind.message(node.kind()))
                .with_help(kind.help().to_string()),
        );
        // Report the outermost occurrence only.
        Visit::Skip
    });
}

#[derive(Clone, Copy)]
enum Logic {
    Loop,
    Branch,
}

impl Logic {
    fn of<'t, N: Node<'t>>(node: N) -> Option<Logic> {
        // `_loop` first: a construct that is both is a loop to a reader.
        if node.has_role(Role::Loop) {
            Some(Logic::Loop)
        } else if node.has_role(Role::Branch) {
            Some(Logic::Branch)
        } else {
            None
        }
    }

    fn message(self, kind: &str) -> String {
        match self {
            Logic::Loop => format!("loop (`{kind}`) in a test body"),
            Logic::Branch => format!("conditional (`{kind}`) in a test body"),
        }
    }

    fn help(self) -> &'static str {
        match self {
            Logic::Loop => {
                "state each case directly instead of looping, so a failure \
                 names the case that failed"
            }
            Logic::Branch => {
                "split the branches into separate tests, so each one states \
                 the scenario it covers"
            }
        }
    }
}
