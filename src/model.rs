//! What marks a test, an assertion, a mock, a fixture.
//!
//! The only framework-specific layer in the design, and a data table rather
//! than code so that adding a framework is an entry rather than a module. It
//! is passed into [`crate::inspect`] instead of being read from anywhere,
//! which is what lets a host extend it: a project with its own assertion
//! wrappers has to say so, and saying so is configuration.
//!
//! Four shapes mark a test, and a language that supports one usually supports
//! no other. Detection has to cover all four or the rules are decorative in
//! most of the languages treebank publishes a pack for:
//!
//! | shape | looks like | languages |
//! |---|---|---|
//! | a name | `def test_adds`, `void test_adds()` | python, ruby, bash, c |
//! | an attribute | `#[test]`, `@Test` | rust, java |
//! | an invocation taking a body | `it("adds", () => …)`, `TEST(S, Adds)` | typescript, javascript, ruby, c, c++ |
//! | a node kind of its own | `test "adds" { … }` | zig |
//!
//! Only the first of those was implemented at first, so `#[test] fn adds()`
//! and `it("adds", …)` were not tests at all and every rule was silent on
//! them. Renaming the same Rust function `test_adds` made findings appear,
//! which is the tell that detection rather than analysis was the gap.

use crate::node::{Node, Visit, walk};
use crate::role::Role;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestModel {
    pub language: String,
    /// A callable whose name starts with one of these is a test.
    pub test_name_prefixes: Vec<String>,
    /// A callable carrying one of these attributes, annotations or decorators
    /// is a test. Matched against the attribute's last path segment, so
    /// `test` covers `#[test]` and `#[tokio::test]` alike.
    pub test_attributes: Vec<String>,
    /// An invocation of one of these that takes a body is a test: jest's
    /// `it(…, () => …)`, RSpec's `it … do`, googletest's `TEST(Suite, Case)`.
    ///
    /// Matched exactly rather than as a prefix. `describe` is deliberately
    /// absent from every preset: a suite is not a test, and a loop directly
    /// inside one is generating cases rather than computing an expectation.
    pub test_invocations: Vec<String>,
    /// Node kinds that are a test outright, for a language whose grammar says
    /// so -- zig's `test_declaration` is the whole of this today.
    pub test_kinds: Vec<String>,
    /// Methods that iterate a collection over a block.
    ///
    /// `xs.each do |x|` carries `_callable` and no `_loop`, because it is a
    /// method call taking a block and the vocabulary is not lying about that.
    /// To a reader it is a loop, and a rule about loops in test bodies that
    /// cannot see the form Ruby and JavaScript actually use is a rule about
    /// nothing. Naming them here keeps that judgement in the table with every
    /// other framework fact rather than buried in a rule.
    pub iteration_methods: Vec<String>,
    /// An invocation whose callee starts with one of these is an assertion.
    pub assertion_prefixes: Vec<String>,
    /// Node kinds that assert without being invocations, such as Python's
    /// `assert` statement.
    pub assertion_kinds: Vec<String>,
}

/// The languages treebank publishes a pack for, as the names it serves them
/// under. A host asking for anything else gets `None` rather than a guess.
pub const LANGUAGES: &[&str] = &[
    "bash",
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

fn owned(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

impl TestModel {
    /// The preset for a language, by the name treebank serves its pack under.
    ///
    /// `javascript` is its own entry even though it shares typescript's pack,
    /// because a host asking for it by name should not have to know that.
    pub fn for_language(language: &str) -> Option<TestModel> {
        Some(match language {
            "python" => TestModel::python(),
            "ruby" => TestModel::ruby(),
            "rust" => TestModel::rust(),
            "java" => TestModel::java(),
            "typescript" => TestModel::typescript(),
            "javascript" => TestModel::javascript(),
            "c" => TestModel::c(),
            "cpp" => TestModel::cpp(),
            "bash" => TestModel::bash(),
            "zig" => TestModel::zig(),
            _ => return None,
        })
    }

    fn base(language: &str) -> TestModel {
        TestModel {
            language: language.to_string(),
            test_name_prefixes: Vec::new(),
            test_attributes: Vec::new(),
            test_invocations: Vec::new(),
            test_kinds: Vec::new(),
            iteration_methods: Vec::new(),
            assertion_prefixes: Vec::new(),
            assertion_kinds: Vec::new(),
        }
    }

    /// pytest and `unittest`.
    pub fn python() -> Self {
        TestModel {
            test_name_prefixes: owned(&["test_", "test"]),
            assertion_prefixes: owned(&["assert", "self.assert", "self.fail", "expect"]),
            assertion_kinds: owned(&["assert_statement"]),
            ..TestModel::base("python")
        }
    }

    /// minitest by name, RSpec and ActiveSupport by block.
    pub fn ruby() -> Self {
        TestModel {
            test_name_prefixes: owned(&["test_"]),
            test_invocations: owned(&["it", "test", "specify", "example", "scenario"]),
            iteration_methods: owned(&[
                "each",
                "each_with_index",
                "each_with_object",
                "each_pair",
                "map",
                "flat_map",
                "times",
                "select",
                "reject",
                "detect",
                "upto",
                "downto",
            ]),
            assertion_prefixes: owned(&["assert", "refute", "expect", "should"]),
            ..TestModel::base("ruby")
        }
    }

    /// `#[test]`, and the attribute macros that wrap it.
    pub fn rust() -> Self {
        TestModel {
            test_name_prefixes: owned(&["test_"]),
            test_attributes: owned(&["test", "rstest", "proptest", "test_case", "bench"]),
            assertion_prefixes: owned(&["assert", "debug_assert", "expect", "panic"]),
            ..TestModel::base("rust")
        }
    }

    /// JUnit and TestNG.
    pub fn java() -> Self {
        TestModel {
            test_name_prefixes: owned(&["test"]),
            test_attributes: owned(&[
                "Test",
                "ParameterizedTest",
                "RepeatedTest",
                "TestFactory",
                "TestTemplate",
            ]),
            assertion_prefixes: owned(&["assert", "Assert.", "assertThat", "verify", "expect"]),
            ..TestModel::base("java")
        }
    }

    /// jest, mocha, vitest and jasmine, which all spell it the same way.
    pub fn typescript() -> Self {
        TestModel {
            test_invocations: owned(&["it", "test", "specify", "bench"]),
            iteration_methods: owned(&["forEach", "map", "flatMap", "filter", "each"]),
            assertion_prefixes: owned(&["expect", "assert", "should"]),
            ..TestModel::base("typescript")
        }
    }

    /// The same frameworks, under the pack typescript's grammar also serves.
    pub fn javascript() -> Self {
        TestModel::typescript().relabel("javascript")
    }

    /// googletest and the Boost/Unity family, plus a plain `test_` name.
    pub fn c() -> Self {
        TestModel {
            test_name_prefixes: owned(&["test_"]),
            test_invocations: owned(&[
                "TEST",
                "TEST_F",
                "TEST_P",
                "TEST_CASE",
                "RUN_TEST",
                "BOOST_AUTO_TEST_CASE",
            ]),
            assertion_prefixes: owned(&["EXPECT_", "ASSERT_", "TEST_ASSERT", "CHECK", "REQUIRE"]),
            ..TestModel::base("c")
        }
    }

    /// googletest, Catch2 and doctest.
    pub fn cpp() -> Self {
        TestModel {
            test_invocations: owned(&[
                "TEST",
                "TEST_F",
                "TEST_P",
                "TYPED_TEST",
                "TYPED_TEST_P",
                "TEST_CASE",
                "SCENARIO",
                "BOOST_AUTO_TEST_CASE",
            ]),
            ..TestModel::c()
        }
        .relabel("cpp")
    }

    /// bats and the shell suites that just name a function.
    pub fn bash() -> Self {
        TestModel {
            test_name_prefixes: owned(&["test_", "it_", "should_"]),
            assertion_prefixes: owned(&["assert", "refute"]),
            ..TestModel::base("bash")
        }
    }

    /// Zig has no framework: a test is a declaration the language itself has.
    pub fn zig() -> Self {
        TestModel {
            test_kinds: owned(&["test_declaration"]),
            assertion_prefixes: owned(&["expect", "try expect", "std.testing."]),
            ..TestModel::base("zig")
        }
    }

    fn relabel(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }

    /// Whether a node is a test, in any of the four shapes a language uses to
    /// say so.
    pub fn is_test<'t, N: Node<'t>>(&self, node: N) -> bool {
        if self.test_kinds.iter().any(|kind| kind == node.kind()) {
            return true;
        }
        if node.has_role(Role::Callable) {
            if self.has_test_attribute(node) {
                return true;
            }
            if callable_name(node).is_some_and(|name| self.names_a_test(name)) {
                return true;
            }
        }
        // A `TEST(Suite, Case)` macro parses as a callable rather than a call,
        // so the invocation shape is checked for both and the name above
        // already covers the macro.
        if node.has_role(Role::Invocation)
            && callee_name(node)
                .is_some_and(|callee| self.test_invocations.iter().any(|name| name == callee))
        {
            return takes_a_body(node);
        }
        false
    }

    fn names_a_test(&self, name: &str) -> bool {
        self.test_invocations.iter().any(|exact| exact == name)
            || self
                .test_name_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix.as_str()))
    }

    /// Whether an attribute in the callable's header marks it as a test.
    ///
    /// The header is every child that is not the body: Rust hangs
    /// `attribute_item` directly off the function, Java buries `annotation`
    /// under `modifiers`, and Python's `decorator` sits beside the name. All
    /// three are found by searching those children and none of them can be
    /// reached by looking at direct children alone.
    fn has_test_attribute<'t, N: Node<'t>>(&self, callable: N) -> bool {
        if self.test_attributes.is_empty() {
            return false;
        }
        let mut found = false;
        for child in callable.children() {
            if child.has_role(Role::Body) || child.has_role(Role::Scope) {
                continue;
            }
            walk(child, &mut |node| {
                if !node.has_role(Role::Attribute) {
                    return Visit::Descend;
                }
                let name = attribute_name(node.text());
                if self.test_attributes.iter().any(|wanted| wanted == name) {
                    found = true;
                }
                Visit::Skip
            });
            if found {
                return true;
            }
        }
        false
    }

    /// Whether a node iterates a collection over a block.
    ///
    /// Not a `_loop` in the vocabulary, and correctly so -- it is a method
    /// call. See [`TestModel::iteration_methods`].
    pub fn is_iteration<'t, N: Node<'t>>(&self, node: N) -> bool {
        self.iteration_method(node).is_some()
    }

    /// The iterating method's name, for a finding that can say `each` rather
    /// than `call_expression`.
    pub fn iteration_method<'t, N: Node<'t>>(&self, node: N) -> Option<&'t str> {
        if self.iteration_methods.is_empty() || !node.has_role(Role::Invocation) {
            return None;
        }
        let callee = callee_name(node)?;
        if self.iteration_methods.iter().any(|name| name == callee) && takes_a_body(node) {
            Some(callee)
        } else {
            None
        }
    }

    /// Whether a node asserts something.
    pub fn is_assertion<'t, N: Node<'t>>(&self, node: N) -> bool {
        if self.assertion_kinds.iter().any(|kind| kind == node.kind()) {
            return true;
        }
        if !node.has_role(Role::Invocation) {
            return false;
        }
        let callee = node
            .child_by_field("function")
            .map(|function| function.text())
            .or_else(|| callee_name(node))
            .unwrap_or_default();
        self.assertion_prefixes
            .iter()
            .any(|prefix| callee.starts_with(prefix.as_str()))
    }
}

/// A callable's name, however the grammar chooses to expose it.
///
/// C and C++ put it inside a declarator with no `name` field, and a
/// googletest macro is indistinguishable from one, so the token before the
/// argument list is the only thing either has. Reading one line keeps that
/// fallback from walking a whole function body when a grammar surprises us.
fn callable_name<'t, N: Node<'t>>(callable: N) -> Option<&'t str> {
    if let Some(name) = callable.child_by_field("name") {
        return Some(name.text());
    }
    let head = callable.text().lines().next()?;
    let head = head.split('(').next().unwrap_or(head);
    head.split_whitespace().last()
}

/// The name being invoked, without its receiver or its path.
///
/// Grammars disagree about the field -- typescript says `function`, ruby says
/// `method`, and several say nothing -- so the fields are tried first and the
/// text is the fallback that works everywhere.
fn callee_name<'t, N: Node<'t>>(node: N) -> Option<&'t str> {
    if let Some(callee) = node
        .child_by_field("function")
        .or_else(|| node.child_by_field("method"))
    {
        return Some(last_segment(callee.text()));
    }
    let head = node.text().lines().next()?;
    let head = head.split(['(', '{']).next().unwrap_or(head);
    Some(last_segment(head.split_whitespace().next()?))
}

/// Whether an invocation carries a body: a lambda, a block, a `do`.
///
/// `it` on its own is a variable; `it("adds", () => …)` is a test. Requiring
/// the body is what keeps the difference.
fn takes_a_body<'t, N: Node<'t>>(node: N) -> bool {
    let mut found = false;
    for child in node.children() {
        walk(child, &mut |inner| {
            if inner.has_role(Role::Callable) || inner.has_role(Role::Body) {
                found = true;
                return Visit::Skip;
            }
            Visit::Descend
        });
        if found {
            return true;
        }
    }
    false
}

/// An attribute's name, with the syntax each language wraps it in removed.
///
/// `#[test]`, `#[tokio::test]`, `@Test` and `@pytest.mark.parametrize(…)` all
/// reduce to their last path segment, which is the part a table can name
/// without also naming every crate that re-exports it.
fn attribute_name(text: &str) -> &str {
    let text = text.trim();
    let text = text.trim_start_matches(['#', '@', '[']);
    let text = text.split('(').next().unwrap_or(text);
    last_segment(text.trim_end_matches([']', ',']).trim())
}

fn last_segment(text: &str) -> &str {
    text.rsplit(['.', ':']).next().unwrap_or(text).trim()
}

#[cfg(test)]
mod tests {
    use super::{LANGUAGES, TestModel, attribute_name, last_segment};

    fn model_is_sound(language: &str) {
        let model = TestModel::for_language(language)
            .unwrap_or_else(|| panic!("no test model for {language}"));
        assert_eq!(&model.language, language);
        assert!(
            !model.test_name_prefixes.is_empty()
                || !model.test_attributes.is_empty()
                || !model.test_invocations.is_empty()
                || !model.test_kinds.is_empty(),
            "{language} has no way to recognise a test at all"
        );
        assert!(
            !model.test_invocations.iter().any(|name| name == "describe"),
            "{language} treats `describe` as a test, so every loop that \
             generates cases is a finding"
        );
    }

    #[test]
    fn bash_has_a_sound_model() {
        model_is_sound("bash");
    }

    #[test]
    fn c_has_a_sound_model() {
        model_is_sound("c");
    }

    #[test]
    fn cpp_has_a_sound_model() {
        model_is_sound("cpp");
    }

    #[test]
    fn java_has_a_sound_model() {
        model_is_sound("java");
    }

    #[test]
    fn javascript_has_a_sound_model() {
        model_is_sound("javascript");
    }

    #[test]
    fn python_has_a_sound_model() {
        model_is_sound("python");
    }

    #[test]
    fn ruby_has_a_sound_model() {
        model_is_sound("ruby");
    }

    #[test]
    fn rust_has_a_sound_model() {
        model_is_sound("rust");
    }

    #[test]
    fn typescript_has_a_sound_model() {
        model_is_sound("typescript");
    }

    #[test]
    fn zig_has_a_sound_model() {
        model_is_sound("zig");
    }

    #[test]
    fn the_languages_named_above_are_every_language_treebank_serves() {
        assert_eq!(
            LANGUAGES,
            [
                "bash",
                "c",
                "cpp",
                "java",
                "javascript",
                "python",
                "ruby",
                "rust",
                "typescript",
                "zig"
            ],
            "a language moved in or out of LANGUAGES; give it a test above, or \
             take its test away"
        );
    }

    #[test]
    fn a_language_treebank_has_no_pack_for_gets_no_model() {
        assert!(
            TestModel::for_language("cobol").is_none(),
            "a language with no pack should not get a guessed model"
        );
    }

    #[test]
    fn an_attribute_reduces_to_the_name_a_table_can_carry() {
        assert_eq!(attribute_name("#[test]"), "test");
        assert_eq!(attribute_name("#[tokio::test]"), "test");
        assert_eq!(attribute_name("#[test_case(1, 2)]"), "test_case");
        assert_eq!(attribute_name("@Test"), "Test");
        assert_eq!(
            attribute_name("@pytest.mark.parametrize('n', xs)"),
            "parametrize"
        );
        assert_eq!(attribute_name("test"), "test");
    }

    #[test]
    fn a_path_reduces_to_its_last_segment() {
        assert_eq!(last_segment("xs.each"), "each");
        assert_eq!(last_segment("std::testing::expect"), "expect");
        assert_eq!(last_segment("it"), "it");
    }
}
