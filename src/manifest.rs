//! Roles read out of a treebank grammar's own manifests.
//!
//! Every host needs this: a treebank tree gives concrete node kinds, and the
//! mapping onto the vocabulary lives in the two manifests a grammar ships --
//! `node-types.json` for the table tier, `roles.json` for facets. A wasm pack
//! carries both (`tb_node_types()`, `tb_roles()`), as does a grammar crate
//! (`NODE_TYPES`, `ROLES`). Doing this once here is the point of the library;
//! doing it per host is how two hosts come to disagree about what a `_loop` is.
//!
//! Which is exactly why the reading itself is treebank's rather than ours.
//! `NodeTypes` and `RolesManifest` are the same types treebank uses to answer
//! the same questions, so a disagreement between beamte and treebank about
//! what derives from what is not possible rather than merely unlikely. It was
//! possible: a hand-rolled closure here filtered nothing, and gave Java's
//! anonymous `";"` the role `_member` because `_member` lists it as an
//! unnamed subtype. Treebank drops unnamed subtypes; now so does this.
//!
//! Nothing here decides what a node is. Table-tier roles are real supertypes
//! in the parse table and arrive in `node-types.json`; facet-tier roles cross
//! cut derivations and arrive in `roles.json`. Both ship inside the grammar
//! crate, so the mapping is treebank's answer rather than this crate's guess.
//!
//! Supertypes nest -- in the Python grammar `while_statement` derives from
//! `_loop` and `_loop` from `_statement` -- so membership is the transitive
//! closure over the subtype lists, not one level of it.
//!
//! Which terms are threaded is per-grammar (DESIGN.md §3.1.1). Python carries
//! no `_control_flow` supertype at all, so asking a node for it there yields
//! nothing. That is a fact about the grammar, and reading it from the manifest
//! is how a rule finds out rather than assuming.

use std::collections::{HashMap, HashSet};

use treebank::node_types::NodeTypes;
use treebank::roles::RolesManifest;

use crate::role::{Role, RoleSet};

/// Node kind to roles, for one grammar.
#[derive(Debug, Clone, Default)]
pub struct RoleTable {
    by_kind: HashMap<String, RoleSet>,
    /// Terms the manifests carry that [`Role`] does not know.
    unknown_terms: Vec<String>,
}

impl RoleTable {
    /// Build from a grammar's `NODE_TYPES` and `ROLES` manifests.
    pub fn from_manifests(node_types: &str, roles: &str) -> Result<RoleTable, String> {
        let node_types =
            NodeTypes::parse(node_types).map_err(|error| format!("node-types.json: {error}"))?;
        let roles = RolesManifest::parse(roles).map_err(|error| format!("roles.json: {error}"))?;

        let mut table = RoleTable::default();
        let mut unknown = HashSet::new();

        // Table tier. The closure is treebank's, walking down from a supertype
        // to everything that derives from it, nested supertypes included.
        for supertype in node_types.supertypes.keys() {
            let Some(role) = Role::from_term(supertype) else {
                unknown.insert(supertype.clone());
                continue;
            };
            for kind in node_types.closure(supertype) {
                // The closure carries the supertype itself. No parse ever
                // yields a node of an abstract kind, so recording one would be
                // an entry nothing can look up.
                if kind == *supertype {
                    continue;
                }
                table.merge(&kind, RoleSet::of(role));
            }
        }

        // Facet tier: direct membership, listed per facet.
        for (term, members) in &roles.facets {
            let Some(role) = Role::from_term(term) else {
                unknown.insert(term.clone());
                continue;
            };
            for kind in members {
                table.merge(kind, RoleSet::of(role));
            }
        }

        table.unknown_terms = unknown.into_iter().collect();
        table.unknown_terms.sort();
        Ok(table)
    }

    fn merge(&mut self, kind: &str, roles: RoleSet) {
        let entry = self.by_kind.entry(kind.to_string()).or_default();
        for role in roles.iter() {
            *entry = entry.with(role);
        }
    }

    pub fn roles(&self, kind: &str) -> RoleSet {
        self.by_kind.get(kind).copied().unwrap_or_default()
    }

    /// Terms the grammar declares that [`Role`] has no variant for. Empty is
    /// the healthy state; anything here means the vocabulary moved.
    pub fn unknown_terms(&self) -> &[String] {
        &self.unknown_terms
    }

    pub fn len(&self) -> usize {
        self.by_kind.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_kind.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::RoleTable;
    use crate::role::Role;

    /// Java's `_member` lists `";"` among its subtypes, unnamed. A reading
    /// that does not drop unnamed subtypes gives a semicolon the role
    /// `_member`, which treebank's own closure never does. This is the
    /// disagreement that reading the manifests twice produced.
    const UNNAMED_SUBTYPE: &str = r#"[
      {"type": "_member", "named": true, "subtypes": [
        {"type": "field_declaration", "named": true},
        {"type": ";", "named": false}
      ]},
      {"type": "field_declaration", "named": true},
      {"type": ";", "named": false}
    ]"#;

    const NO_FACETS: &str = r#"{"vocabulary": "test", "facets": {}}"#;

    #[test]
    fn an_unnamed_subtype_is_not_a_member() {
        let table = RoleTable::from_manifests(UNNAMED_SUBTYPE, NO_FACETS).expect("manifests parse");

        assert!(
            table.roles(";").is_empty(),
            "an anonymous token carries no role"
        );
        assert!(table.roles("field_declaration").contains(Role::Member));
    }

    /// Supertypes nest, and only the closure over the whole chain finds it.
    const NESTED: &str = r#"[
      {"type": "_statement", "named": true, "subtypes": [{"type": "_loop", "named": true}]},
      {"type": "_loop", "named": true, "subtypes": [{"type": "while_statement", "named": true}]},
      {"type": "while_statement", "named": true}
    ]"#;

    #[test]
    fn membership_is_transitive_through_nested_supertypes() {
        let table = RoleTable::from_manifests(NESTED, NO_FACETS).expect("manifests parse");
        let roles = table.roles("while_statement");

        assert!(roles.contains(Role::Loop));
        assert!(roles.contains(Role::Statement));
    }

    #[test]
    fn an_abstract_supertype_is_not_a_kind_of_its_own() {
        let table = RoleTable::from_manifests(NESTED, NO_FACETS).expect("manifests parse");

        // A parse never yields a node whose kind is `_statement`, so an entry
        // for it would be one nothing can look up. `_loop` still carries
        // `_statement`, because it really does derive from it.
        assert!(table.roles("_statement").is_empty());
        assert!(table.roles("_loop").contains(Role::Statement));
    }

    #[test]
    fn a_term_the_vocabulary_does_not_know_is_reported() {
        let unknown = r#"[
          {"type": "_nonsense", "named": true, "subtypes": [{"type": "x", "named": true}]},
          {"type": "x", "named": true}
        ]"#;
        let table = RoleTable::from_manifests(unknown, NO_FACETS).expect("manifests parse");

        assert_eq!(table.unknown_terms(), &["_nonsense".to_string()]);
        assert!(table.roles("x").is_empty());
    }

    #[test]
    fn facet_membership_arrives_from_the_roles_manifest() {
        let facets = r#"{"vocabulary": "test", "facets": {"_callable": ["lambda"]}}"#;
        let table = RoleTable::from_manifests(NESTED, facets).expect("manifests parse");

        assert!(table.roles("lambda").contains(Role::Callable));
    }
}
