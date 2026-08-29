//! Roles read out of a treebank grammar's own manifests.
//!
//! Every host needs this: a treebank tree gives concrete node kinds, and the
//! mapping onto the vocabulary lives in the two manifests a grammar ships --
//! `node-types.json` for the table tier, `roles.json` for facets. A wasm pack
//! carries both (`tb_node_types()`, `tb_roles()`), as does a grammar crate
//! (`NODE_TYPES`, `ROLES`). Doing this once here is the point of the library;
//! doing it per host is how two hosts come to disagree about what a `_loop` is.
//!
//! Nothing here decides what a node is. Table-tier roles are real supertypes
//! in the parse table and arrive in `node-types.json`; facet-tier roles cross
//! cut derivations and arrive in `roles.json`. Both ship inside the grammar
//! crate, so the mapping is treebank's answer rather than this crate's guess.
//!
//! Supertypes nest — in the Python grammar `while_statement` derives from
//! `_loop` and `_loop` from `_statement` — so membership is the transitive
//! closure over the subtype lists, not one level of it.
//!
//! Which terms are threaded is per-grammar (DESIGN.md §3.1.1). Python carries
//! no `_control_flow` supertype at all, so asking a node for it there yields
//! nothing. That is a fact about the grammar, and reading it from the manifest
//! is how a rule finds out rather than assuming.

use std::collections::{HashMap, HashSet};

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
        let node_types: serde_json::Value = serde_json::from_str(node_types)
            .map_err(|error| format!("node-types.json: {error}"))?;
        let roles: serde_json::Value =
            serde_json::from_str(roles).map_err(|error| format!("roles.json: {error}"))?;

        let mut table = RoleTable::default();
        let mut unknown = HashSet::new();

        // Table tier: every entry carrying `subtypes` is a supertype, and each
        // of its subtypes derives from it.
        let mut supertypes_of: HashMap<&str, Vec<&str>> = HashMap::new();
        let entries = node_types
            .as_array()
            .ok_or_else(|| "node-types.json is not an array".to_string())?;
        for entry in entries {
            let (Some(name), Some(subtypes)) = (
                entry.get("type").and_then(serde_json::Value::as_str),
                entry.get("subtypes").and_then(serde_json::Value::as_array),
            ) else {
                continue;
            };
            for subtype in subtypes {
                let Some(subtype) = subtype.get("type").and_then(serde_json::Value::as_str) else {
                    continue;
                };
                supertypes_of.entry(subtype).or_default().push(name);
            }
        }

        for entry in entries {
            let Some(kind) = entry.get("type").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let mut terms = HashSet::new();
            ancestors(kind, &supertypes_of, &mut terms);
            let mut set = RoleSet::empty();
            for term in &terms {
                match Role::from_term(term) {
                    Some(role) => set = set.with(role),
                    None => {
                        unknown.insert((*term).to_string());
                    }
                }
            }
            table.merge(kind, set);
        }

        // Facet tier: direct membership, listed per facet.
        let facets = roles
            .get("facets")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "roles.json carries no facets object".to_string())?;
        for (term, members) in facets {
            let role = match Role::from_term(term) {
                Some(role) => role,
                None => {
                    unknown.insert(term.clone());
                    continue;
                }
            };
            let Some(members) = members.as_array() else {
                continue;
            };
            for member in members {
                let Some(kind) = member.as_str() else {
                    continue;
                };
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

/// Every supertype reachable from `kind`, transitively, excluding `kind`.
fn ancestors<'a>(
    kind: &'a str,
    supertypes_of: &HashMap<&'a str, Vec<&'a str>>,
    out: &mut HashSet<&'a str>,
) {
    let Some(parents) = supertypes_of.get(kind) else {
        return;
    };
    for parent in parents {
        if out.insert(parent) {
            ancestors(parent, supertypes_of, out);
        }
    }
}
