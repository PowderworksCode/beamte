//! The rule catalogue. One module per rule, each naming the post it restates.

pub mod test_logic;

use crate::finding::Rule;

/// Every rule beamte implements.
pub const fn all() -> &'static [Rule] {
    &[test_logic::RULE]
}
