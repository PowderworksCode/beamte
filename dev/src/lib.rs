//! Development harness. Not part of the published library.
//!
//! Rules cannot be written blind: developing one means running it against a
//! real file and seeing what the analysis saw, repeatedly. This module parses
//! with native tree-sitter so that tests and the `beamte` binary can do that.
//!
//! It is **not** how a host parses: a host loads treebank wasm packs and
//! implements [`Node`] over the `tb_*` ABI. It is the same grammar either way,
//! though — this links treebank's Python grammar natively and reads its roles
//! out of the manifests the crate ships, so a rule is exercised against
//! treebank's own answers rather than against a table written here.
//!
//! That the two paths agree is still unproven; notes/DESIGN.md §10.1 wants a fixture
//! comparing a native tree against a pack tree, and this makes that comparison
//! possible rather than settling it.

use std::sync::OnceLock;

use beamte::node::{Node, Span};
use beamte::role::RoleSet;

pub use beamte::manifest::RoleTable;

/// The Python role table, built once from the grammar's own manifests.
pub fn python_roles() -> &'static RoleTable {
    static TABLE: OnceLock<RoleTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        RoleTable::from_manifests(treebank_python::NODE_TYPES, treebank_python::ROLES)
            .expect("treebank-python ships well-formed manifests")
    })
}

/// A parsed file, owning its tree.
pub struct Parsed {
    tree: tree_sitter::Tree,
    source: String,
    roles: &'static RoleTable,
}

impl Parsed {
    /// Parse Python source.
    pub fn python(source: impl Into<String>) -> Result<Parsed, String> {
        let source = source.into();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&treebank_python::LANGUAGE.into())
            .map_err(|error| format!("loading the python grammar: {error}"))?;
        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| "the parser returned no tree".to_string())?;
        Ok(Parsed {
            tree,
            source,
            roles: python_roles(),
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn root(&self) -> TsNode<'_> {
        TsNode {
            inner: self.tree.root_node(),
            source: &self.source,
            roles: self.roles,
        }
    }
}

/// A tree-sitter node, wearing beamte's [`Node`] trait.
#[derive(Clone, Copy)]
pub struct TsNode<'t> {
    inner: tree_sitter::Node<'t>,
    source: &'t str,
    roles: &'static RoleTable,
}

impl<'t> Node<'t> for TsNode<'t> {
    fn kind(&self) -> &'t str {
        self.inner.kind()
    }

    fn roles(&self) -> RoleSet {
        self.roles.roles(self.inner.kind())
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
        self.inner.named_child(index as u32).map(|inner| TsNode {
            inner,
            source: self.source,
            roles: self.roles,
        })
    }

    fn child_by_field(&self, name: &str) -> Option<Self> {
        self.inner.child_by_field_name(name).map(|inner| TsNode {
            inner,
            source: self.source,
            roles: self.roles,
        })
    }

    fn text(&self) -> &'t str {
        self.source
            .get(self.inner.start_byte()..self.inner.end_byte())
            .unwrap_or_default()
    }
}
