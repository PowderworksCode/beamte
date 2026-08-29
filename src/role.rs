//! The treebank node vocabulary, as the terms beamte consumes.
//!
//! Treebank enforces these in the parse table and `treebank roles` checks the
//! closed lists, so the authority lives there. Beamte carries its own enum
//! rather than depending on `treebank-core` because a host has to map its
//! tree's roles onto *something* at the trait boundary anyway, and because a
//! library with no dependencies is cheaper to adopt. When `treebank-core` is
//! published, a test should assert this enum against its term lists.

/// A role a node carries. Names match treebank's underscore-prefixed terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    // Structural.
    Statement,
    Expression,
    Declaration,
    Member,
    Directive,
    Body,
    // Operational.
    ControlFlow,
    Branch,
    Loop,
    Jump,
    Assignment,
    Invocation,
    Access,
    Literal,
    // Naming and shape.
    Name,
    Identifier,
    Attribute,
    Modifier,
    Type,
    Pattern,
    Interpolation,
    Str,
    Comment,
    // Callables and their pieces.
    Callable,
    Parameter,
    Argument,
    // Binding and structure.
    Binding,
    Scope,
    Clause,
}

impl Role {
    /// Every role, in declaration order.
    pub const ALL: [Role; 29] = [
        Role::Statement,
        Role::Expression,
        Role::Declaration,
        Role::Member,
        Role::Directive,
        Role::Body,
        Role::ControlFlow,
        Role::Branch,
        Role::Loop,
        Role::Jump,
        Role::Assignment,
        Role::Invocation,
        Role::Access,
        Role::Literal,
        Role::Name,
        Role::Identifier,
        Role::Attribute,
        Role::Modifier,
        Role::Type,
        Role::Pattern,
        Role::Interpolation,
        Role::Str,
        Role::Comment,
        Role::Callable,
        Role::Parameter,
        Role::Argument,
        Role::Binding,
        Role::Scope,
        Role::Clause,
    ];

    /// The treebank term, as it appears in a query or in `roles.json`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::Statement => "_statement",
            Role::Expression => "_expression",
            Role::Declaration => "_declaration",
            Role::Member => "_member",
            Role::Directive => "_directive",
            Role::Body => "_body",
            Role::ControlFlow => "_control_flow",
            Role::Branch => "_branch",
            Role::Loop => "_loop",
            Role::Jump => "_jump",
            Role::Assignment => "_assignment",
            Role::Invocation => "_invocation",
            Role::Access => "_access",
            Role::Literal => "_literal",
            Role::Name => "_name",
            Role::Identifier => "_identifier",
            Role::Attribute => "_attribute",
            Role::Modifier => "_modifier",
            Role::Type => "_type",
            Role::Pattern => "_pattern",
            Role::Interpolation => "_interpolation",
            Role::Str => "_string",
            Role::Comment => "_comment",
            Role::Callable => "_callable",
            Role::Parameter => "_parameter",
            Role::Argument => "_argument",
            Role::Binding => "_binding",
            Role::Scope => "_scope",
            Role::Clause => "_clause",
        }
    }

    /// The role a treebank term names, if beamte knows it.
    ///
    /// `None` means the vocabulary has grown a term this enum has not learned
    /// yet. A host should treat that as a role it cannot reason about rather
    /// than as an error; the test in `tests/vocabulary.rs` is what keeps the
    /// gap from going unnoticed.
    pub fn from_term(term: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|role| role.as_str() == term)
    }

    const fn bit(self) -> u64 {
        1u64 << (self as u64)
    }
}

/// The set of roles one node carries. A node is usually several things at
/// once: a `function_definition` is a `_declaration`, a `_scope` and a
/// `_callable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RoleSet(u64);

impl RoleSet {
    pub const fn empty() -> Self {
        RoleSet(0)
    }

    pub const fn of(role: Role) -> Self {
        RoleSet(role.bit())
    }

    pub const fn with(self, role: Role) -> Self {
        RoleSet(self.0 | role.bit())
    }

    pub const fn contains(self, role: Role) -> bool {
        self.0 & role.bit() != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The roles in the set, in `Role::ALL` order.
    pub fn iter(self) -> impl Iterator<Item = Role> {
        Role::ALL
            .into_iter()
            .filter(move |&role| self.contains(role))
    }
}

impl FromIterator<Role> for RoleSet {
    fn from_iter<I: IntoIterator<Item = Role>>(roles: I) -> Self {
        roles.into_iter().fold(RoleSet::empty(), RoleSet::with)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_holds_the_roles_put_into_it() {
        let roles = RoleSet::of(Role::Loop).with(Role::ControlFlow);

        assert!(roles.contains(Role::Loop));
        assert!(roles.contains(Role::ControlFlow));
        assert!(!roles.contains(Role::Branch));
    }

    #[test]
    fn an_empty_set_contains_nothing() {
        let roles = RoleSet::empty();

        assert!(roles.is_empty());
        assert!(!roles.contains(Role::Loop));
    }

    #[test]
    fn every_role_has_a_distinct_bit() {
        let all: RoleSet = Role::ALL.into_iter().collect();

        assert_eq!(all.iter().count(), Role::ALL.len());
    }
}
