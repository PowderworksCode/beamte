//! Development harness. Not part of the published library.
//!
//! Rules cannot be written blind: developing one means running it against a
//! real file and seeing what the analysis saw, repeatedly. This module parses
//! with native tree-sitter so that tests and the `beamte` binary can do that.
//!
//! It is **not** how a host parses. A host loads treebank wasm packs and
//! implements [`Node`] over the `tb_*` ABI. Treebank's own grammars are not
//! published yet, so this uses the upstream grammar plus the role table in
//! [`roles`] — throwaway scaffolding that treebank's grammars replace, and
//! which is exactly why DESIGN.md §10.1 wants a fixture proving a native tree
//! and a pack tree agree.

pub mod roles;

use crate::node::{Node, Span};
use crate::role::RoleSet;

/// A parsed file, owning its tree.
pub struct Parsed {
    tree: tree_sitter::Tree,
    source: String,
}

impl Parsed {
    /// Parse Python source.
    pub fn python(source: impl Into<String>) -> Result<Parsed, String> {
        let source = source.into();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|error| format!("loading the python grammar: {error}"))?;
        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| "the parser returned no tree".to_string())?;
        Ok(Parsed { tree, source })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn root(&self) -> TsNode<'_> {
        TsNode {
            inner: self.tree.root_node(),
            source: &self.source,
        }
    }
}

/// A tree-sitter node, wearing beamte's [`Node`] trait.
#[derive(Clone, Copy)]
pub struct TsNode<'t> {
    inner: tree_sitter::Node<'t>,
    source: &'t str,
}

impl<'t> Node<'t> for TsNode<'t> {
    fn kind(&self) -> &'t str {
        self.inner.kind()
    }

    fn roles(&self) -> RoleSet {
        roles::python(self.inner.kind())
    }

    fn span(&self) -> Span {
        let start = self.inner.start_position();
        Span {
            start_byte: self.inner.start_byte(),
            end_byte: self.inner.end_byte(),
            line: start.row + 1,
            column: start.column + 1,
        }
    }

    fn child_count(&self) -> usize {
        self.inner.named_child_count()
    }

    fn child(&self, index: usize) -> Option<Self> {
        self.inner.named_child(index).map(|inner| TsNode {
            inner,
            source: self.source,
        })
    }

    fn child_by_field(&self, name: &str) -> Option<Self> {
        self.inner.child_by_field_name(name).map(|inner| TsNode {
            inner,
            source: self.source,
        })
    }

    fn text(&self) -> &'t str {
        self.source
            .get(self.inner.start_byte()..self.inner.end_byte())
            .unwrap_or_default()
    }
}
