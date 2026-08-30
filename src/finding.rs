//! What a rule returns.
//!
//! Deliberately not a report: no severity, no formatting, no suppression.
//! Those are the host's, per notes/DESIGN.md §6.

use crate::node::Span;

/// The property of a good test that a rule defends.
///
/// Beamte states the property and stops. A severity is a policy about a
/// repository and belongs to whoever runs the scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Property {
    /// When the code is broken, the test fails.
    Fidelity,
    /// The test fails only when the code is broken.
    Resilience,
    /// When it fails, you know where to look.
    Precision,
}

impl Property {
    pub const fn as_str(self) -> &'static str {
        match self {
            Property::Fidelity => "fidelity",
            Property::Resilience => "resilience",
            Property::Precision => "precision",
        }
    }
}

/// What a rule reads: the tests in a file, or the file whole.
///
/// The catalogue began as test-quality rules only, and most of it still is.
/// `env-read` was the first rule with something to say about every file, and
/// a host has to know which kind it is holding: a `Tests` rule belongs on
/// files that look like tests, a `File` rule on everything the host can
/// parse, and a host that runs "all rules" over test files alone would be
/// silently skipping the second kind. notes/DESIGN.md §5.5 records the
/// decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The rule reads test bodies; a file with no tests yields nothing.
    Tests,
    /// The rule reads any source file, test or not.
    File,
}

impl Scope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Scope::Tests => "tests",
            Scope::File => "file",
        }
    }
}

/// A rule's stable identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuleId(&'static str);

impl RuleId {
    pub const fn new(name: &'static str) -> Self {
        RuleId(name)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// The post a rule was issued under. Rules here are not the author's
/// opinion, so every finding can name its authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Citation {
    pub title: &'static str,
    pub url: &'static str,
    pub date: &'static str,
}

/// A rule's metadata, for a host that wants to list rules or render them as
/// instructions before an agent writes any tests.
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub id: RuleId,
    pub property: Property,
    pub scope: Scope,
    pub summary: &'static str,
    pub instruction: &'static str,
    pub citation: Citation,
}

/// One step of the reasoning behind a finding.
///
/// A host with somewhere to put these should: straitjacket renders them as a
/// SARIF code flow, which is how a finding shows its argument rather than
/// asserting it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceStep {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: RuleId,
    pub property: Property,
    pub span: Span,
    pub message: String,
    pub help: Option<String>,
    pub evidence: Vec<EvidenceStep>,
}

impl Finding {
    pub fn new(rule: &Rule, span: Span, message: impl Into<String>) -> Self {
        Finding {
            rule: rule.id,
            property: rule.property,
            span,
            message: message.into(),
            help: None,
            evidence: Vec::new(),
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}
