//! `GenerationPolicy`: the limits a surface lives within. the defaults are
//! conservative, sized for a surface a person reads at once.

/// how deep a tree is ever walked, whatever the policy says: past it the
/// decoder and the verifier stop descending, so a pathological input
/// cannot exhaust the stack.
pub const HARD_DEPTH: usize = 256;

/// the limits and permissions a surface is verified under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerationPolicy {
    /// the most nodes a surface may have, templates counted once.
    pub max_nodes: usize,
    /// the deepest a node may sit, the root at depth 1.
    pub max_depth: usize,
    /// the longest a title, a label or a description may be, in
    /// characters.
    pub max_text: usize,
}

impl Default for GenerationPolicy {
    /// 256 nodes, depth 16, 256 characters.
    fn default() -> Self {
        GenerationPolicy {
            max_nodes: 256,
            max_depth: 16,
            max_text: 256,
        }
    }
}
