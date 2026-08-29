//! The boundary. Beamte walks trees through this trait and owns no parser.
//!
//! Two implementations are expected: the host's, over treebank's `tb_*` wasm
//! pack ABI, and the one behind `--features dev` here, over native
//! tree-sitter. Having two is what keeps the trait an abstraction rather than
//! a mirror of whichever backend was written first.

use std::marker::PhantomData;

use crate::role::{Role, RoleSet};

/// Where a node is. Lines and columns are 1-based, which is what a host
/// reporting to a human wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_byte: usize,
    pub end_byte: usize,
    pub line: usize,
    pub column: usize,
}

/// One node of a parsed file.
///
/// Implementors expose *named* children only: the punctuation a grammar needs
/// is below the layer any rule here reasons about.
pub trait Node<'t>: Copy + Sized {
    /// The grammar's own name for this node, such as `function_definition`.
    /// Rules should prefer roles; kinds are for diagnostics and for the few
    /// places a language-specific fact is unavoidable.
    fn kind(&self) -> &'t str;

    /// The treebank roles this node carries.
    fn roles(&self) -> RoleSet;

    fn span(&self) -> Span;

    fn child_count(&self) -> usize;

    fn child(&self, index: usize) -> Option<Self>;

    /// The child under a grammar field, such as the `name` of a definition.
    fn child_by_field(&self, name: &str) -> Option<Self>;

    /// The source text this node spans.
    fn text(&self) -> &'t str;

    fn has_role(&self, role: Role) -> bool {
        self.roles().contains(role)
    }

    fn children(&self) -> Children<'t, Self> {
        Children {
            parent: *self,
            index: 0,
            count: self.child_count(),
            marker: PhantomData,
        }
    }
}

/// The named children of a node.
pub struct Children<'t, N: Node<'t>> {
    parent: N,
    index: usize,
    count: usize,
    marker: PhantomData<&'t ()>,
}

impl<'t, N: Node<'t>> Iterator for Children<'t, N> {
    type Item = N;

    fn next(&mut self) -> Option<N> {
        while self.index < self.count {
            let index = self.index;
            self.index += 1;
            if let Some(child) = self.parent.child(index) {
                return Some(child);
            }
        }
        None
    }
}

/// Whether a walk should descend into the node it was just handed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visit {
    Descend,
    Skip,
}

/// Pre-order walk from `node`, including `node` itself.
///
/// Returning [`Visit::Skip`] prunes the subtree, which is how a rule reports
/// the outermost occurrence of something without also reporting every nested
/// one.
pub fn walk<'t, N: Node<'t>>(node: N, visit: &mut impl FnMut(N) -> Visit) {
    if visit(node) == Visit::Descend {
        for child in node.children() {
            walk(child, visit);
        }
    }
}

/// A file to inspect, and optionally the code it tests.
///
/// `under_test` is what the semantic rules need and is unresolved for now;
/// DESIGN.md §10.2 is the open question that gates them.
pub struct Unit<'t, N: Node<'t>> {
    pub path: &'t str,
    pub source: &'t str,
    pub root: N,
    pub under_test: Option<N>,
}

impl<'t, N: Node<'t>> Unit<'t, N> {
    pub fn new(path: &'t str, source: &'t str, root: N) -> Self {
        Unit {
            path,
            source,
            root,
            under_test: None,
        }
    }
}
