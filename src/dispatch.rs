use std::any::Any;

/// Marker trait for typed widget actions.
///
/// Implement this on enums or structs that represent a component's output events.
/// The parent drains the [`ActionQueue`] each frame, pattern-matches on concrete
/// types, and updates its own state accordingly.
///
/// # Example
/// ```rust
/// use plev::dispatch::{ActionQueue, WidgetAction};
///
/// #[derive(Debug, PartialEq)]
/// enum FileAction { Stage(String), Discard(String) }
/// impl WidgetAction for FileAction {}
///
/// let mut q = ActionQueue::new();
/// q.emit(1, FileAction::Stage("main.rs".into()));
/// let actions = q.drain_typed::<FileAction>();
/// assert_eq!(actions[0].1, FileAction::Stage("main.rs".into()));
/// ```
pub trait WidgetAction: Any + Send + 'static {}

// ---------------------------------------------------------------------------
// Internal storage
// ---------------------------------------------------------------------------

struct RawAction {
    /// Identifier of the component that emitted this action (typically a u64
    /// derived from an index or pointer — callers decide the scheme).
    source: u64,
    /// Type-erased payload. Concrete type is recovered via `downcast`.
    payload: Box<dyn Any + Send>,
}

// ---------------------------------------------------------------------------
// ActionQueue
// ---------------------------------------------------------------------------

/// Per-frame queue for typed component-to-parent communication.
///
/// Components call [`emit`] during event handling; parents call [`drain_typed`]
/// once per frame to consume actions of a specific type while leaving others
/// untouched.  There is no global bus — callers own and pass the queue.
///
/// [`emit`]: ActionQueue::emit
/// [`drain_typed`]: ActionQueue::drain_typed
#[derive(Default)]
pub struct ActionQueue {
    actions: Vec<RawAction>,
}

impl ActionQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue a typed action from `source`.
    pub fn emit<A: WidgetAction>(&mut self, source: u64, action: A) {
        self.actions.push(RawAction {
            source,
            payload: Box::new(action),
        });
    }

    /// Drain **only** actions whose concrete type is `A`.
    ///
    /// Actions of other types remain in the queue, preserving insertion order.
    pub fn drain_typed<A: WidgetAction>(&mut self) -> Vec<(u64, A)> {
        let mut result = Vec::new();
        let mut remaining = Vec::new();

        for raw in self.actions.drain(..) {
            match raw.payload.downcast::<A>() {
                Ok(action) => result.push((raw.source, *action)),
                Err(payload) => remaining.push(RawAction {
                    source: raw.source,
                    payload,
                }),
            }
        }

        self.actions = remaining;
        result
    }

    /// Drain every action regardless of type.
    pub fn drain_all(&mut self) -> Vec<(u64, Box<dyn Any + Send>)> {
        self.actions
            .drain(..)
            .map(|r| (r.source, r.payload))
            .collect()
    }

    /// `true` when no actions are pending.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Number of pending actions.
    pub fn len(&self) -> usize {
        self.actions.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum FileAction {
        Stage(String),
        Discard(String),
    }
    impl WidgetAction for FileAction {}

    #[derive(Debug, PartialEq)]
    enum ModalAction {
        Confirmed,
        Cancelled,
    }
    impl WidgetAction for ModalAction {}

    #[test]
    fn new_queue_is_empty() {
        let q = ActionQueue::new();
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn emit_and_drain_typed() {
        let mut q = ActionQueue::new();
        q.emit(1, FileAction::Stage("foo.rs".into()));
        let drained = q.drain_typed::<FileAction>();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0], (1, FileAction::Stage("foo.rs".into())));
        assert!(q.is_empty());
    }

    #[test]
    fn drain_wrong_type_returns_empty_leaves_queue_intact() {
        let mut q = ActionQueue::new();
        q.emit(1, FileAction::Stage("foo.rs".into()));
        let modals = q.drain_typed::<ModalAction>();
        assert!(modals.is_empty());
        // Original action still in queue
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn multiple_actions_same_source() {
        let mut q = ActionQueue::new();
        q.emit(5, FileAction::Stage("a.rs".into()));
        q.emit(5, FileAction::Discard("b.rs".into()));
        let drained = q.drain_typed::<FileAction>();
        assert_eq!(drained.len(), 2);
        assert!(drained.iter().all(|(src, _)| *src == 5));
    }

    #[test]
    fn drain_typed_filters_by_type_preserves_order() {
        let mut q = ActionQueue::new();
        q.emit(1, FileAction::Stage("a.rs".into()));
        q.emit(2, ModalAction::Confirmed);
        q.emit(3, FileAction::Discard("b.rs".into()));

        let files = q.drain_typed::<FileAction>();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], (1, FileAction::Stage("a.rs".into())));
        assert_eq!(files[1], (3, FileAction::Discard("b.rs".into())));

        // ModalAction still in queue
        assert_eq!(q.len(), 1);
        let modals = q.drain_typed::<ModalAction>();
        assert_eq!(modals.len(), 1);
        assert_eq!(modals[0], (2, ModalAction::Confirmed));
        assert!(q.is_empty());
    }

    #[test]
    fn drain_all_empties_queue() {
        let mut q = ActionQueue::new();
        q.emit(1, FileAction::Stage("x.rs".into()));
        q.emit(2, ModalAction::Cancelled);
        let all = q.drain_all();
        assert_eq!(all.len(), 2);
        assert!(q.is_empty());
    }

    #[test]
    fn drain_typed_twice_is_idempotent_when_empty() {
        let mut q = ActionQueue::new();
        q.emit(1, FileAction::Stage("z.rs".into()));
        q.drain_typed::<FileAction>();
        let second = q.drain_typed::<FileAction>();
        assert!(second.is_empty());
        assert!(q.is_empty());
    }

    #[test]
    fn source_ids_are_preserved() {
        let mut q = ActionQueue::new();
        q.emit(42, FileAction::Stage("x".into()));
        q.emit(99, FileAction::Discard("y".into()));
        let drained = q.drain_typed::<FileAction>();
        assert_eq!(drained[0].0, 42);
        assert_eq!(drained[1].0, 99);
    }
}
