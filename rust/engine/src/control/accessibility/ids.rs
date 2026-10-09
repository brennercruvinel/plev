//! `NodeId`s of the accessibility tree, derived from each node's `SemId`.
//!
//! A screen reader keeps its place by node id. When the scene reorganizes
//! (a wider window, a scrolled list, a row that moved), the node it was on
//! keeps its id, so the reader keeps its place: the id is a hash of the
//! `SemId`'s canonical text, never a position, an emission order or a
//! per-frame counter. The hash is FNV-1a 64 over the text bytes, computed
//! here by hand, so it is the same on every run and every target.
//!
//! Two ids of one frame that share a hash are resolved deterministically,
//! whatever order they were projected in: the smallest in `SemId` order
//! keeps the plain hash, and each other one, in `SemId` order, probes
//! salted hashes (its text, a `0xff` byte that utf-8 never holds, the
//! attempt as u32 little endian) until it reaches a value no other node of
//! the frame claims. `ROOT` belongs to the window and no node takes it.
//! Never two nodes with one `NodeId`.

use accesskit::NodeId;
use accord::{ActionName, SemId};
use rustc_hash::{FxHashMap, FxHashSet};

/// The window, the root of every tree this module builds.
pub const ROOT: NodeId = NodeId(0);

/// Salted attempts a colliding id makes before it takes the next free
/// value by counting up (a guard: one salted attempt settles any real
/// collision).
const PROBES: u32 = 64;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The node id of a `SemId`'s text: attempt 0 is the plain FNV-1a 64 of
/// the text, attempt `n > 0` the hash of the text, `0xff` and `n`.
pub fn hash(text: &str, attempt: u32) -> u64 {
    let mut h = FNV_OFFSET;
    let mut eat = |byte: u8| {
        h ^= u64::from(byte);
        h = h.wrapping_mul(FNV_PRIME);
    };
    text.bytes().for_each(&mut eat);
    if attempt > 0 {
        eat(0xff);
        attempt.to_le_bytes().into_iter().for_each(&mut eat);
    }
    h
}

/// The node ids of one frame's tree, both ways, and what each control
/// the tree presents stands for (`present`).
#[derive(Debug, Default)]
pub struct NodeIds {
    by_sem: FxHashMap<SemId, NodeId>,
    by_node: FxHashMap<NodeId, SemId>,
    /// A presented control's id: the item it is inside and the item's
    /// action a click on it asks for.
    stands_for: FxHashMap<SemId, (SemId, ActionName)>,
}

impl NodeIds {
    /// Gives each id its node id (see the module docs). An id given twice
    /// counts once.
    pub fn assign<'a>(ids: impl IntoIterator<Item = &'a SemId>) -> Self {
        Self::assign_with(ids, hash)
    }

    /// `assign` with the hash as a parameter, so a test can force the
    /// collisions a 64-bit hash never shows in practice.
    pub(crate) fn assign_with<'a>(
        ids: impl IntoIterator<Item = &'a SemId>,
        hash: impl Fn(&str, u32) -> u64,
    ) -> Self {
        let ids: Vec<(&SemId, u64)> = ids
            .into_iter()
            .map(|id| (id, hash(id.as_str(), 0)))
            .collect();
        // The owner of each plain hash: the smallest id that has it.
        let mut owner: FxHashMap<u64, &SemId> = FxHashMap::default();
        for &(id, plain) in &ids {
            owner
                .entry(plain)
                .and_modify(|held| *held = (*held).min(id))
                .or_insert(id);
        }
        let mut taken: FxHashSet<u64> = owner.keys().copied().collect();
        taken.insert(ROOT.0);

        let mut out = NodeIds::default();
        let mut losers: Vec<&SemId> = Vec::new();
        for &(id, plain) in &ids {
            if out.by_sem.contains_key(id) {
                continue;
            }
            if plain != ROOT.0 && owner.get(&plain) == Some(&id) {
                out.insert(id, NodeId(plain));
            } else if !losers.contains(&id) {
                losers.push(id);
            }
        }
        losers.sort();
        for id in losers {
            let probed = (1..=PROBES)
                .map(|attempt| hash(id.as_str(), attempt))
                .find(|v| !taken.contains(v));
            let value = probed.unwrap_or_else(|| {
                let mut v = hash(id.as_str(), PROBES).wrapping_add(1);
                while taken.contains(&v) {
                    v = v.wrapping_add(1);
                }
                v
            });
            taken.insert(value);
            out.insert(id, NodeId(value));
        }
        out
    }

    fn insert(&mut self, id: &SemId, node: NodeId) {
        self.by_sem.insert(id.clone(), node);
        self.by_node.insert(node, id.clone());
    }

    /// The node id of `id`, when it is in this frame's tree.
    pub fn node(&self, id: &SemId) -> Option<NodeId> {
        self.by_sem.get(id).copied()
    }

    /// The `SemId` behind a node id; `None` for the root and for an id
    /// this frame does not hold.
    pub fn sem(&self, node: NodeId) -> Option<&SemId> {
        self.by_node.get(&node)
    }

    /// Records that the presented control `id` stands for `action` of
    /// `item` (the id itself is assigned with the others).
    pub(crate) fn stand_for(&mut self, id: SemId, item: SemId, action: ActionName) {
        self.stands_for.insert(id, (item, action));
    }

    /// The item and the action a presented control stands for; `None`
    /// for a node of the snapshot.
    pub fn presented(&self, id: &SemId) -> Option<(&SemId, &str)> {
        self.stands_for
            .get(id)
            .map(|(item, action)| (item, action.as_str()))
    }

    /// How many nodes have an id (the root not counted).
    pub fn len(&self) -> usize {
        self.by_sem.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_sem.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sid(text: &str) -> SemId {
        match SemId::parse(text) {
            Ok(id) => id,
            Err(e) => panic!("{text}: {e}"),
        }
    }

    fn ids(texts: &[&str]) -> Vec<SemId> {
        texts.iter().map(|t| sid(t)).collect()
    }

    #[test]
    fn a_node_id_is_the_fnv_hash_of_the_semid_text() {
        // FNV-1a 64 of the empty input is the offset basis; of "a", the
        // published reference value.
        assert_eq!(hash("", 0), FNV_OFFSET);
        assert_eq!(hash("a", 0), 0xaf63_dc4c_8601_ec8c);
        let all = ids(&["todo", "todo.item[3]"]);
        let a = NodeIds::assign(&all);
        assert_eq!(a.node(&all[1]), Some(NodeId(hash("todo.item[3]", 0))));
        assert_eq!(a.sem(NodeId(hash("todo", 0))), Some(&all[0]));
        assert_eq!(a.sem(ROOT), None);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn the_same_ids_get_the_same_node_ids_in_any_order_and_run() {
        let all = ids(&["todo", "todo.input", "todo.item[1]", "todo.item[2]"]);
        let a = NodeIds::assign(&all);
        let mut reversed = all.clone();
        reversed.reverse();
        let b = NodeIds::assign(&reversed);
        let fewer = NodeIds::assign(&all[1..]);
        for id in &all {
            assert_eq!(a.node(id), b.node(id), "{id}");
        }
        for id in &all[1..] {
            assert_eq!(a.node(id), fewer.node(id), "{id} depends on its neighbours");
        }
    }

    #[test]
    fn colliding_ids_get_distinct_node_ids_deterministically() {
        // Every text hashes to 7 on its first attempt; salted attempts
        // differ by text and attempt.
        let clash = |text: &str, attempt: u32| {
            if attempt == 0 { 7 } else { hash(text, attempt) }
        };
        let all = ids(&["c", "a", "b", "a.x"]);
        let first = NodeIds::assign_with(&all, clash);
        let mut reversed = all.clone();
        reversed.reverse();
        let second = NodeIds::assign_with(&reversed, clash);

        let mut seen = FxHashSet::default();
        for id in &all {
            let node = first.node(id).unwrap();
            assert!(seen.insert(node), "{id} shares {node:?}");
            assert_ne!(node, ROOT);
            assert_eq!(second.node(id), Some(node), "{id} depends on order");
            assert_eq!(first.sem(node), Some(id));
        }
        // The smallest id keeps the plain hash; the rest probe.
        assert_eq!(first.node(&sid("a")), Some(NodeId(7)));
        assert_eq!(first.node(&sid("a.x")), Some(NodeId(hash("a.x", 1))));
    }

    #[test]
    fn no_node_takes_the_root_even_when_its_hash_is_zero() {
        let zero = |text: &str, attempt: u32| {
            if text == "todo" && attempt == 0 {
                ROOT.0
            } else {
                hash(text, attempt)
            }
        };
        let all = ids(&["todo"]);
        let a = NodeIds::assign_with(&all, zero);
        assert_eq!(a.node(&all[0]), Some(NodeId(hash("todo", 1))));
    }

    #[test]
    fn a_hash_that_never_varies_still_ends_with_distinct_ids() {
        let stuck = |_: &str, _: u32| 42;
        let all = ids(&["a", "b", "c"]);
        let a = NodeIds::assign_with(&all, stuck);
        let nodes: FxHashSet<NodeId> = all.iter().filter_map(|id| a.node(id)).collect();
        assert_eq!(nodes.len(), 3);
        assert!(!nodes.contains(&ROOT));
    }

    #[test]
    fn an_id_given_twice_counts_once() {
        let all = ids(&["a", "a"]);
        let a = NodeIds::assign(&all);
        assert_eq!(a.len(), 1);
        assert_eq!(a.node(&all[0]), Some(NodeId(hash("a", 0))));
    }
}
