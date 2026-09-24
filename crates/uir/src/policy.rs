//! `GenerationPolicy`: who proposed a surface and the limits it lives
//! within. the defaults are conservative: an agent wrote it, and it is
//! sized for a surface a person reads at once.

use sem::Actor;

/// how deep a tree is ever walked, whatever the policy says: past it the
/// decoder and the verifier stop descending, so a pathological input
/// cannot exhaust the stack.
pub const HARD_DEPTH: usize = 256;

/// the limits and permissions a surface is verified under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerationPolicy {
    /// who wrote the IR. it may bind only the capabilities this actor may
    /// ask for itself (`Policy::actors`), so no surface lends an agent an
    /// action the app keeps from agents.
    pub author: Actor,
    /// the most nodes a surface may have, templates counted once.
    pub max_nodes: usize,
    /// the deepest a node may sit, the root at depth 1.
    pub max_depth: usize,
    /// the longest a title, a label or a description may be, in
    /// characters.
    pub max_text: usize,
}

impl GenerationPolicy {
    /// the default limits for an IR `author` wrote.
    pub fn by(author: Actor) -> GenerationPolicy {
        GenerationPolicy {
            author,
            ..GenerationPolicy::default()
        }
    }
}

impl Default for GenerationPolicy {
    /// an agent (`agent:unknown`) wrote it; 256 nodes, depth 16, 256
    /// characters.
    fn default() -> Self {
        GenerationPolicy {
            author: Actor::Agent {
                name: "unknown".to_owned(),
            },
            max_nodes: 256,
            max_depth: 16,
            max_text: 256,
        }
    }
}
