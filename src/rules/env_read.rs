//! `env-read` — Test Sizes (2010-12-13).
//!
//! > | Feature           | Small (test) | Medium | Large |
//! > |-------------------|--------------|--------|-------|
//! > | System properties | No           | Yes    | Yes   |
//!
//! The size contract bans a small test from touching the process's ambient
//! configuration, and a component that reads the environment mid-body forces
//! that violation on every small test that executes it. The read is an input
//! no signature admits to: the function behaves differently on two machines
//! and nothing in its declaration says why, which is the definition of a test
//! that fails when nothing broke.
//!
//! The first rule in the catalogue with something to say about every file,
//! not only test files — `scope: File`, and notes/DESIGN.md §5.5 records the
//! decision. Where the environment *may* be read — a designated
//! configuration module, an entry point — is policy about a repository, so
//! naming those files is the host's configuration, exactly as severity is.
//!
//! Detection is structural. A read is an `_invocation` whose callee path is a
//! language's environment surface (`std::env::var`, `os.getenv`,
//! `System.getenv`, `ENV.fetch`) or an `_access` that reaches it directly
//! (`process.env`, `os.environ[…]`, `ENV[…]`). Matching the node rather than
//! the text is what keeps a mention in a comment or a string from being a
//! finding. Two misses are accepted and named here rather than guessed at: a
//! read behind an alias (`use std::env::var as v`) and one behind a local
//! wrapper are invisible to a single tree, and a write through the same
//! surface (`os.environ["X"] = …`) is reported in the same words as a read —
//! the surface is the finding, and the fix is the same edge either way.
//!
//! Rust's `env!` and `option_env!` are deliberately not flagged: they resolve
//! when the build system runs, against variables the build declares, and a
//! build that fails on an undeclared variable is the announced channel this
//! rule is steering reads toward.

use crate::finding::{Citation, Finding, Property, Rule, RuleId, Scope};
use crate::model::TestModel;
use crate::node::{Node, Unit, Visit, walk};
use crate::role::Role;

pub const RULE: Rule = Rule {
    id: RuleId::new("env-read"),
    property: Property::Resilience,
    scope: Scope::File,
    summary: "code reads the process environment where nothing declares it",
    instruction: "Do not read environment variables in the middle of ordinary \
                  code. An ambient read is configuration no signature admits \
                  to, and no small test of that code can stay hermetic. Read \
                  the environment once, at a declared configuration edge, and \
                  pass the values on as arguments.",
    citation: Citation {
        title: "Test Sizes",
        url: "https://testing.googleblog.com/2010/12/test-sizes.html",
        date: "2010-12-13",
    },
};

/// The languages this rule has an environment surface for.
///
/// Bash is deliberately absent: `$VAR` is the language's own variable model,
/// and a rule that flags every expansion flags the language. A host should
/// treat a language not listed here as one this rule does not read, per
/// notes/DESIGN.md §7.3, rather than reporting the file clean.
pub const LANGUAGES: &[&str] = &[
    "c",
    "cpp",
    "java",
    "javascript",
    "python",
    "ruby",
    "rust",
    "typescript",
    "zig",
];

/// Whether the rule can read a language at all.
pub fn covers(language: &str) -> bool {
    surface(language).is_some()
}

pub fn check<'t, N: Node<'t>>(unit: &Unit<'t, N>, model: &TestModel, out: &mut Vec<Finding>) {
    let Some(surface) = surface(&model.language) else {
        return;
    };
    walk(unit.root, &mut |node| {
        let Some(spelling) = surface.read_at(node) else {
            return Visit::Descend;
        };
        out.push(finding(node, spelling));
        // `os.environ["X"]` is one read, not a subscript finding plus an
        // attribute finding; pruning here is the deduplication.
        Visit::Skip
    });
}

fn finding<'t, N: Node<'t>>(node: N, spelling: &str) -> Finding {
    let message = match named_variable(node) {
        Some(name) => format!("`{name}` read from the process environment (`{spelling}`)"),
        None => format!("the process environment read through `{spelling}`"),
    };
    Finding::new(&RULE, node.span(), message).with_help(
        "take the value as an argument, or read it at the one declared \
         configuration edge and pass it down"
            .to_string(),
    )
}

/// The variable being read, when the read names it with a literal.
///
/// The first string under the read is the name in every surface here —
/// `var("SLOTH_WALKS")`, `environ["PATH"]`, `getenv("HOME")` — and a read
/// with no literal is reported without one rather than guessed at.
fn named_variable<'t, N: Node<'t>>(node: N) -> Option<String> {
    let mut name = None;
    walk(node, &mut |inner| {
        if name.is_some() {
            return Visit::Skip;
        }
        if inner.has_role(Role::Str) {
            name = Some(unquote(inner.text()));
            return Visit::Skip;
        }
        Visit::Descend
    });
    name.filter(|name| !name.is_empty() && name.len() <= 60)
}

fn unquote(text: &str) -> String {
    text.trim_start_matches(['b', 'r', 'f', 'u'])
        .trim_matches(['"', '\'', '`'])
        .to_string()
}

/// One language's ways of reaching the environment.
struct Surface {
    language: &'static str,
    /// Callee paths that read the environment when invoked. An entry ending
    /// in `.` names a receiver whose every method is a read (`ENV.`); any
    /// other entry matches the callee exactly or as its trailing path
    /// (`env::var` matches `std::env::var`).
    calls: &'static [&'static str],
    /// Surfaces read without a call: an `_access` whose text is the entry,
    /// or the entry followed by `[`, `.` or `?` — `process.env.PATH`,
    /// `os.environ["X"]`.
    reads: &'static [&'static str],
}

/// One table, not one module per language: a language's environment surface
/// is a fact like its test attributes, and facts belong in tables.
const SURFACES: &[Surface] = &[
    Surface {
        language: "c",
        calls: &["getenv", "secure_getenv", "getenv_s", "_dupenv_s"],
        reads: &[],
    },
    Surface {
        language: "cpp",
        calls: &["getenv", "secure_getenv", "getenv_s", "_dupenv_s"],
        reads: &[],
    },
    Surface {
        language: "java",
        // `System.getProperty` rides along because it is the same ambient
        // channel under another name, and the post's table names it first.
        calls: &["System.getenv", "System.getProperty"],
        reads: &[],
    },
    Surface {
        language: "javascript",
        calls: &["Deno.env."],
        reads: &["process.env", "import.meta.env"],
    },
    Surface {
        language: "python",
        calls: &["os.getenv", "getenv", "os.environ.", "environ."],
        reads: &["os.environb", "os.environ", "environ"],
    },
    Surface {
        language: "ruby",
        calls: &["ENV."],
        reads: &["ENV"],
    },
    Surface {
        language: "rust",
        calls: &["env::var", "env::var_os", "env::vars", "env::vars_os"],
        reads: &[],
    },
    Surface {
        language: "typescript",
        calls: &["Deno.env."],
        reads: &["process.env", "import.meta.env"],
    },
    Surface {
        language: "zig",
        calls: &[
            "process.getEnvVarOwned",
            "process.getEnvMap",
            "process.hasEnvVar",
            "process.hasEnvVarConstant",
            "posix.getenv",
            "os.getenv",
        ],
        reads: &[],
    },
];

fn surface(language: &str) -> Option<&'static Surface> {
    SURFACES.iter().find(|entry| entry.language == language)
}

impl Surface {
    /// The spelling of the read this node makes, if it makes one.
    ///
    /// Both shapes are tried rather than dispatched on, because a grammar may
    /// honestly give one node both roles — a subscript is an access, and in
    /// Ruby it is an invocation too.
    fn read_at<'t, N: Node<'t>>(&self, node: N) -> Option<&'t str> {
        if node.has_role(Role::Invocation)
            && let Some(callee) = callee_path(node)
            && self.calls.iter().any(|wanted| calls(callee, wanted))
        {
            return Some(callee);
        }
        if node.has_role(Role::Access) {
            let text = first_line(node.text());
            if let Some(wanted) = self.reads.iter().find(|wanted| reads(text, wanted)) {
                return Some(wanted);
            }
        }
        None
    }
}

/// The invoked path, as written: `std::env::var`, `ENV.fetch`, `getenv`.
///
/// The grammar's `function` field is exact where it exists. The fallback —
/// the call's first line up to its argument list — is what covers grammars
/// that name the field differently or not at all, and it is head-anchored,
/// so a mention further down a callee's line cannot match.
fn callee_path<'t, N: Node<'t>>(node: N) -> Option<&'t str> {
    if let Some(callee) = node.child_by_field("function") {
        let callee = first_line(callee.text());
        if !callee.contains('(') {
            return Some(callee);
        }
    }
    let head = first_line(node.text());
    Some(head.split(['(', '[', '{']).next().unwrap_or(head).trim())
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or(text).trim()
}

/// Whether an invoked path is one of a surface's calls.
///
/// `env::var` matches `std::env::var` but not `my_env::var`: the segment
/// boundary is part of the match, because a name that merely ends in the
/// letters is a different name.
fn calls(callee: &str, wanted: &str) -> bool {
    if let Some(receiver) = wanted.strip_suffix('.') {
        return callee
            .strip_prefix(receiver)
            .is_some_and(|rest| rest.starts_with('.') || rest.starts_with("::"));
    }
    callee == wanted
        || callee
            .strip_suffix(wanted)
            .is_some_and(|head| head.ends_with("::") || head.ends_with('.'))
}

/// Whether an accessed text is a surface read: the surface itself, or the
/// surface followed by a subscript or a member.
fn reads(text: &str, wanted: &str) -> bool {
    text.strip_prefix(wanted)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(['[', '.', '?']))
}

#[cfg(test)]
mod tests {
    use super::{LANGUAGES, calls, covers, reads, surface, unquote};

    #[test]
    fn a_call_matches_on_segment_boundaries_only() {
        assert!(calls("std::env::var", "env::var"));
        assert!(calls("env::var", "env::var"));
        assert!(calls("os.getenv", "getenv"));
        assert!(!calls("my_env::var", "env::var"));
        assert!(!calls("std::env::var_os", "env::var"));
        assert!(!calls("agetenv", "getenv"));
    }

    #[test]
    fn a_dotted_receiver_entry_matches_every_method_on_it() {
        assert!(calls("ENV.fetch", "ENV."));
        assert!(calls("ENV.values_at", "ENV."));
        assert!(!calls("MYENV.fetch", "ENV."));
        assert!(!calls("ENV", "ENV."));
    }

    #[test]
    fn a_read_is_the_surface_or_the_surface_subscripted() {
        assert!(reads("process.env", "process.env"));
        assert!(reads("process.env.NODE_ENV", "process.env"));
        assert!(reads("os.environ[\"PATH\"]", "os.environ"));
        assert!(!reads("process.envelope", "process.env"));
        assert!(!reads("my.process.env", "process.env"));
    }

    #[test]
    fn every_covered_language_is_listed_and_bash_is_not() {
        for language in LANGUAGES {
            assert!(covers(language), "{language} is listed but has no surface");
            assert!(
                surface(language).is_some_and(|surface| {
                    !surface.calls.is_empty() || !surface.reads.is_empty()
                }),
                "{language} has an empty surface"
            );
        }
        assert!(
            !covers("bash"),
            "bash's every expansion is an environment read; flagging the \
             language is not a rule"
        );
    }

    #[test]
    fn a_variable_name_loses_its_quotes_and_prefixes() {
        assert_eq!(unquote("\"SLOTH_WALKS\""), "SLOTH_WALKS");
        assert_eq!(unquote("'HOME'"), "HOME");
        assert_eq!(unquote("b\"PATH\""), "PATH");
    }
}
