//! What marks a test, an assertion, a mock, a fixture.
//!
//! The only framework-specific layer in the design, and a data table rather
//! than code so that adding a framework is an entry rather than a module. It
//! is passed into [`crate::inspect`] instead of being read from anywhere,
//! which is what lets a host extend it: a project with its own assertion
//! wrappers has to say so, and saying so is configuration.

use crate::node::Node;
use crate::role::Role;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestModel {
    pub language: String,
    /// A callable whose name starts with one of these is a test.
    pub test_name_prefixes: Vec<String>,
    /// A callable carrying one of these attributes or decorators is a test.
    pub test_attributes: Vec<String>,
    /// An invocation whose callee starts with one of these is an assertion.
    pub assertion_prefixes: Vec<String>,
    /// Node kinds that assert without being invocations, such as Python's
    /// `assert` statement.
    pub assertion_kinds: Vec<String>,
}

impl TestModel {
    /// pytest and `unittest`.
    pub fn python() -> Self {
        TestModel {
            language: "python".into(),
            test_name_prefixes: vec!["test_".into(), "test".into()],
            test_attributes: vec![],
            assertion_prefixes: vec!["assert".into(), "self.assert".into(), "expect".into()],
            assertion_kinds: vec!["assert_statement".into()],
        }
    }

    /// Whether a `_callable` node is a test.
    pub fn is_test<'t, N: Node<'t>>(&self, callable: N) -> bool {
        if !callable.has_role(Role::Callable) {
            return false;
        }
        let named = callable
            .child_by_field("name")
            .map(|name| name.text())
            .unwrap_or_default();
        self.test_name_prefixes
            .iter()
            .any(|prefix| named.starts_with(prefix.as_str()))
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
            .unwrap_or_default();
        self.assertion_prefixes
            .iter()
            .any(|prefix| callee.starts_with(prefix.as_str()))
    }
}
