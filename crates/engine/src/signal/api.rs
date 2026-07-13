//! Public signal API — ReadSignal, WriteSignal, create_signal, create_effect,
//! create_memo, batch, dispose_node.

use std::any::Any;
use std::marker::PhantomData;
use std::rc::Rc;

use super::execution::{execute_memo_update, flush_pending_effects};
use super::runtime::{
    CompareFn, FxIndexSet, MemoFn, NodeId, NodeKind, NodeState, ReactiveNode, with_runtime,
};

// ---------------------------------------------------------------------------
// Public handles
// ---------------------------------------------------------------------------

/// Read-only handle to a signal or memo.
pub struct ReadSignal<T: 'static> {
    id: NodeId,
    _marker: PhantomData<T>,
}

impl<T: 'static> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for ReadSignal<T> {}

impl<T: Clone + 'static> ReadSignal<T> {
    /// Read the value, automatically tracking the dependency.
    pub fn get(&self) -> T {
        let needs_update = with_runtime(|rt| {
            rt.nodes.get(self.id).is_some_and(|n| {
                matches!(n.kind, NodeKind::Memo { .. }) && n.state != NodeState::Clean
            })
        });
        if needs_update {
            execute_memo_update(self.id);
        }
        with_runtime(|rt| {
            rt.track(self.id);
            let node = rt.nodes.get(self.id).expect("signal node not found");
            node.value
                .as_ref()
                .expect("signal has no value")
                .downcast_ref::<T>()
                .expect("type mismatch in signal read")
                .clone()
        })
    }
}

impl<T: Clone + 'static> ReadSignal<T> {
    /// Read the value WITHOUT tracking -- no dependency created.
    pub fn peek(&self) -> T {
        let needs_update = with_runtime(|rt| {
            rt.nodes.get(self.id).is_some_and(|n| {
                matches!(n.kind, NodeKind::Memo { .. }) && n.state != NodeState::Clean
            })
        });
        if needs_update {
            execute_memo_update(self.id);
        }
        with_runtime(|rt| {
            // No track() call -- that's the point of peek
            let node = rt.nodes.get(self.id).expect("signal node not found");
            node.value
                .as_ref()
                .expect("signal has no value")
                .downcast_ref::<T>()
                .expect("type mismatch in signal peek")
                .clone()
        })
    }
}

impl<T: 'static> ReadSignal<T> {
    /// Borrow the value, automatically tracking the dependency.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let needs_update = with_runtime(|rt| {
            rt.nodes.get(self.id).is_some_and(|n| {
                matches!(n.kind, NodeKind::Memo { .. }) && n.state != NodeState::Clean
            })
        });
        if needs_update {
            execute_memo_update(self.id);
        }
        with_runtime(|rt| {
            rt.track(self.id);
            let node = rt.nodes.get(self.id).expect("signal node not found");
            let val = node
                .value
                .as_ref()
                .expect("signal has no value")
                .downcast_ref::<T>()
                .expect("type mismatch in signal with");
            f(val)
        })
    }

    /// Return the underlying NodeId.
    pub fn id(&self) -> NodeId {
        self.id
    }
}

/// Write-only handle to a signal.
pub struct WriteSignal<T: 'static> {
    id: NodeId,
    _marker: PhantomData<T>,
}

impl<T: 'static> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for WriteSignal<T> {}

impl<T: 'static> WriteSignal<T> {
    /// Replace the value and trigger subscribers.
    pub fn set(&self, value: T) {
        let should_flush = with_runtime(|rt| {
            let node = rt.nodes.get_mut(self.id).expect("signal node not found");
            node.value = Some(Box::new(value));
            node.state = NodeState::Dirty;
            rt.notify_subscribers(self.id);
            rt.batch_depth == 0
        });
        if should_flush {
            flush_pending_effects();
        }
    }

    /// Mutate the value in-place and trigger subscribers.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let should_flush = with_runtime(|rt| {
            let node = rt.nodes.get_mut(self.id).expect("signal node not found");
            let val = node
                .value
                .as_mut()
                .expect("signal has no value")
                .downcast_mut::<T>()
                .expect("type mismatch in signal update");
            f(val);
            node.state = NodeState::Dirty;
            rt.notify_subscribers(self.id);
            rt.batch_depth == 0
        });
        if should_flush {
            flush_pending_effects();
        }
    }

    /// Return the underlying NodeId.
    pub fn id(&self) -> NodeId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Public API -- free functions
// ---------------------------------------------------------------------------

/// Create a new signal with the given initial value.
pub fn create_signal<T: Clone + 'static>(initial: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let id = with_runtime(|rt| {
        rt.nodes.insert(ReactiveNode {
            value: Some(Box::new(initial)),
            kind: NodeKind::Signal,
            state: NodeState::Clean,
            sources: FxIndexSet::default(),
            subscribers: FxIndexSet::default(),
            running: false,
        })
    });
    (
        ReadSignal {
            id,
            _marker: PhantomData,
        },
        WriteSignal {
            id,
            _marker: PhantomData,
        },
    )
}

/// Create an effect that automatically tracks signal dependencies.
/// Runs immediately and re-runs whenever its dependencies change.
pub fn create_effect(f: impl Fn() + 'static) {
    let id = with_runtime(|rt| {
        rt.nodes.insert(ReactiveNode {
            value: None,
            kind: NodeKind::Effect { f: Rc::new(f) },
            state: NodeState::Dirty,
            sources: FxIndexSet::default(),
            subscribers: FxIndexSet::default(),
            running: false,
        })
    });
    super::execution::execute_effect(id);
}

/// Create a memoized computed value. Re-computes only when dependencies change.
/// Cuts propagation if new value == old value (solves diamond problem).
pub fn create_memo<T: PartialEq + Clone + 'static>(f: impl Fn() -> T + 'static) -> ReadSignal<T> {
    let wrapped: MemoFn = Rc::new(move || -> Box<dyn Any> { Box::new(f()) });
    let compare: CompareFn =
        Rc::new(
            |a: &dyn Any, b: &dyn Any| match (a.downcast_ref::<T>(), b.downcast_ref::<T>()) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            },
        );
    let id = with_runtime(|rt| {
        rt.nodes.insert(ReactiveNode {
            value: None,
            kind: NodeKind::Memo {
                f: wrapped,
                compare,
            },
            state: NodeState::Dirty,
            sources: FxIndexSet::default(),
            subscribers: FxIndexSet::default(),
            running: false,
        })
    });
    execute_memo_update(id);
    ReadSignal {
        id,
        _marker: PhantomData,
    }
}

/// Batch multiple signal writes -- effects only run once at the end.
pub fn batch(f: impl FnOnce()) {
    with_runtime(|rt| rt.batch_depth += 1);
    f();
    let should_flush = with_runtime(|rt| {
        rt.batch_depth -= 1;
        rt.batch_depth == 0
    });
    if should_flush {
        flush_pending_effects();
    }
}

/// Remove a reactive node and clean up all its subscriptions.
pub fn dispose_node(id: NodeId) {
    with_runtime(|rt| rt.dispose(id));
}
