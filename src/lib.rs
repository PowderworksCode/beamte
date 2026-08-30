//! Test-quality rules over treebank trees. Trees in, findings out.
//!
//! Beamte takes a parsed test file and returns findings about it. Every rule
//! restates a post from the Google Testing Blog and carries its citation.
//!
//! It parses nothing, reads no files, writes no output format and owns no
//! configuration: those belong to the host running the scan. `DESIGN.md` is
//! the authoritative document.

pub mod finding;
#[cfg(feature = "manifests")]
pub mod manifest;
pub mod model;
pub mod node;
pub mod role;
pub mod rules;

pub use finding::{Citation, EvidenceStep, Finding, Property, Rule, RuleId};
#[cfg(feature = "manifests")]
pub use manifest::RoleTable;
pub use model::{LANGUAGES, TestModel};
pub use node::{Node, Span, Unit, Visit, walk};
pub use role::{Role, RoleSet};

/// Which rules to run.
///
/// A host has one call into beamte and says here what it wants from it, so
/// that turning a rule off is configuration rather than a second entry point.
/// `Only` and `Except` are both offered because a project that wants one rule
/// and a project that dislikes one rule should each be able to say so
/// directly, rather than restating the whole catalogue whenever it grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection<'a> {
    #[default]
    All,
    Only(&'a [RuleId]),
    Except(&'a [RuleId]),
}

impl Selection<'_> {
    fn wants(&self, rule: &Rule) -> bool {
        match self {
            Selection::All => true,
            Selection::Only(ids) => ids.contains(&rule.id),
            Selection::Except(ids) => !ids.contains(&rule.id),
        }
    }
}

/// Run every rule over one unit.
///
/// Findings come back in the order the rules produced them; a host that wants
/// them sorted knows better than this library what to sort by.
pub fn inspect<'t, N: Node<'t>>(unit: &Unit<'t, N>, model: &TestModel) -> Vec<Finding> {
    inspect_with(unit, model, Selection::All)
}

/// Run a chosen set of rules over one unit.
///
/// The one entry point a host needs: it selects, runs and returns, so that a
/// host adds no rule dispatch of its own and gains every rule added here
/// without changing a line. A selection naming a rule that does not exist is
/// the host's to reject -- see [`rule`] -- because only the host knows where
/// the name came from and how to blame the right file.
pub fn inspect_with<'t, N: Node<'t>>(
    unit: &Unit<'t, N>,
    model: &TestModel,
    selection: Selection<'_>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if selection.wants(&rules::test_logic::RULE) {
        rules::test_logic::check(unit, model, &mut findings);
    }
    findings
}

/// Every rule beamte implements, with its property and citation.
pub fn catalogue() -> &'static [Rule] {
    rules::all()
}

/// The rule a name identifies, for a host validating its own configuration.
pub fn rule(id: &str) -> Option<&'static Rule> {
    rules::all().iter().find(|rule| rule.id.as_str() == id)
}
