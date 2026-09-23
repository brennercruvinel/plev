//! What a screen reader asks, read back into the contract. The tree
//! advertises a click on a node that has a primary action: the first
//! of `activate`, `toggle`, `submit` and `select` it declares without a
//! parameter (a click carries none).

use sem::{ActionDecl, Node};

/// The actions a click may stand for, in the order the primary one is
/// picked.
pub const PRIMARY: [&str; 4] = ["activate", "toggle", "submit", "select"];

/// The action a click on `node` asks for (see `PRIMARY`).
pub fn primary(node: &Node) -> Option<&ActionDecl> {
    PRIMARY.iter().find_map(|name| {
        node.actions
            .iter()
            .find(|a| a.name.as_str() == *name && a.param.is_none())
    })
}
