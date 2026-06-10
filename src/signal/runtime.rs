//! Reactive runtime — data-only graph storage.
//!
//! Contains `ReactiveRuntime`, `ReactiveNode`, `NodeState`, `NodeKind`,
//! the `RUNTIME` thread-local, and the `with_runtime` helper.

use indexmap::IndexSet;
use rustc_hash::FxBuildHasher;
use slotmap::SlotMap;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;

// ---------------------------------------------------------------------------
// Key type
// ---------------------------------------------------------------------------

slotmap::new_key_type! {
    /// Identifies a reactive node in the runtime.
    pub struct NodeId;
}

// ---------------------------------------------------------------------------
// Node state & kind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeState {
    Clean,
    Check,
    Dirty,
}

pub(crate) type MemoFn = Rc<dyn Fn() -> Box<dyn Any>>;
pub(crate) type CompareFn = Rc<dyn Fn(&dyn Any, &dyn Any) -> bool>;

pub(crate) enum NodeKind {
    Signal,
    Memo { f: MemoFn, compare: CompareFn },
    Effect { f: Rc<dyn Fn()> },
}

// ---------------------------------------------------------------------------
// ReactiveNode
// ---------------------------------------------------------------------------

pub(crate) struct ReactiveNode {
    pub(crate) value: Option<Box<dyn Any>>,
    pub(crate) kind: NodeKind,
    pub(crate) state: NodeState,
    pub(crate) sources: FxIndexSet<NodeId>,
    pub(crate) subscribers: FxIndexSet<NodeId>,
    pub(crate) running: bool,
}

// ---------------------------------------------------------------------------
// Runtime (data only -- no user code execution)
// ---------------------------------------------------------------------------

pub(crate) struct ReactiveRuntime {
    pub(crate) nodes: SlotMap<NodeId, ReactiveNode>,
    pub(crate) observer_stack: Vec<NodeId>,
    pub(crate) pending_effects: Vec<NodeId>,
    pub(crate) batch_depth: u32,
}

impl ReactiveRuntime {
    pub(crate) fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            observer_stack: Vec::new(),
            pending_effects: Vec::new(),
            batch_depth: 0,
        }
    }

    pub(crate) fn track(&mut self, source_id: NodeId) {
        if let Some(&observer_id) = self.observer_stack.last() {
            if let Some(source) = self.nodes.get_mut(source_id) {
                source.subscribers.insert(observer_id);
            }
            if let Some(observer) = self.nodes.get_mut(observer_id) {
                observer.sources.insert(source_id);
            }
        }
    }

    pub(crate) fn clear_sources(&mut self, id: NodeId) {
        let old_sources: Vec<NodeId> = self
            .nodes
            .get(id)
            .map(|n| n.sources.iter().copied().collect())
            .unwrap_or_default();
        for src in &old_sources {
            if let Some(source) = self.nodes.get_mut(*src) {
                source.subscribers.swap_remove(&id);
            }
        }
        if let Some(node) = self.nodes.get_mut(id) {
            node.sources.clear();
        }
    }

    pub(crate) fn notify_subscribers(&mut self, id: NodeId) {
        let subs: Vec<NodeId> = self
            .nodes
            .get(id)
            .map(|n| n.subscribers.iter().copied().collect())
            .unwrap_or_default();

        for sub_id in subs {
            if let Some(sub) = self.nodes.get(sub_id)
                && sub.running
            {
                panic!(
                    "Circular dependency detected: node {:?} is notified while running",
                    sub_id
                );
            }
            if let Some(sub) = self.nodes.get_mut(sub_id) {
                match (&sub.kind, sub.state) {
                    (NodeKind::Effect { .. }, NodeState::Clean) => {
                        sub.state = NodeState::Dirty;
                        self.pending_effects.push(sub_id);
                    }
                    (NodeKind::Memo { .. }, NodeState::Clean) => {
                        sub.state = NodeState::Check;
                        self.propagate_check(sub_id);
                    }
                    (NodeKind::Effect { .. }, NodeState::Check) => {
                        sub.state = NodeState::Dirty;
                    }
                    _ => {}
                }
            }
        }
    }

    fn propagate_check(&mut self, id: NodeId) {
        let subs: Vec<NodeId> = self
            .nodes
            .get(id)
            .map(|n| n.subscribers.iter().copied().collect())
            .unwrap_or_default();

        for sub_id in subs {
            if let Some(sub) = self.nodes.get_mut(sub_id)
                && sub.state == NodeState::Clean
            {
                match &sub.kind {
                    NodeKind::Effect { .. } => {
                        sub.state = NodeState::Check;
                        self.pending_effects.push(sub_id);
                    }
                    NodeKind::Memo { .. } => {
                        sub.state = NodeState::Check;
                        self.propagate_check(sub_id);
                    }
                    NodeKind::Signal => {}
                }
            }
        }
    }

    pub(crate) fn dispose(&mut self, id: NodeId) {
        self.clear_sources(id);
        if let Some(node) = self.nodes.get(id) {
            let subs: Vec<NodeId> = node.subscribers.iter().copied().collect();
            for sub_id in subs {
                if let Some(sub) = self.nodes.get_mut(sub_id) {
                    sub.sources.swap_remove(&id);
                }
            }
        }
        self.nodes.remove(id);
    }
}

// ---------------------------------------------------------------------------
// Thread-local runtime
// ---------------------------------------------------------------------------

thread_local! {
    pub(crate) static RUNTIME: RefCell<ReactiveRuntime> = RefCell::new(ReactiveRuntime::new());
}

/// Brief borrow -- NEVER call this across user closure execution.
pub(crate) fn with_runtime<R>(f: impl FnOnce(&mut ReactiveRuntime) -> R) -> R {
    RUNTIME.with(|rt| f(&mut rt.borrow_mut()))
}
