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

#[cfg(feature = "dev")]
pub mod dev;

pub use finding::{Citation, EvidenceStep, Finding, Property, Rule, RuleId};
#[cfg(feature = "manifests")]
pub use manifest::RoleTable;
pub use model::TestModel;
pub use node::{Node, Span, Unit, Visit, walk};
pub use role::{Role, RoleSet};

/// Run every rule over one unit.
///
/// Findings come back in the order the rules produced them; a host that wants
/// them sorted knows better than this library what to sort by.
pub fn inspect<'t, N: Node<'t>>(unit: &Unit<'t, N>, model: &TestModel) -> Vec<Finding> {
    let mut findings = Vec::new();
    rules::test_logic::check(unit, model, &mut findings);
    findings
}

/// Every rule beamte implements, with its property and citation.
pub fn catalogue() -> &'static [Rule] {
    rules::all()
}
