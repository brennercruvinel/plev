//! Effect and memo execution — manages borrow boundaries carefully.
//!
//! Every function that runs user closures releases the runtime borrow BEFORE
//! calling the closure, then re-borrows afterward.

use std::rc::Rc;

use super::runtime::{CompareFn, MemoFn, NodeId, NodeKind, NodeState, with_runtime};

// ---------------------------------------------------------------------------
// RAII Observer Guard — restores observer stack + running flag on drop
// ---------------------------------------------------------------------------

struct ObserverGuard {
    id: NodeId,
}

impl Drop for ObserverGuard {
    fn drop(&mut self) {
        with_runtime(|rt| {
            rt.observer_stack.pop();
            if let Some(node) = rt.nodes.get_mut(self.id) {
                node.running = false;
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Standalone execution functions (manage borrows carefully)
// ---------------------------------------------------------------------------

enum EffectAction {
    Run(Rc<dyn Fn()>),
    UpdateMemosFirst(Vec<NodeId>),
}

pub(crate) fn execute_effect(id: NodeId) {
    let action = with_runtime(|rt| {
        let state = match rt.nodes.get(id) {
            Some(n) => n.state,
            None => return None,
        };

        match state {
            NodeState::Clean => return None,
            NodeState::Check => {
                let sources: Vec<NodeId> = rt
                    .nodes
                    .get(id)
                    .map(|n| n.sources.iter().copied().collect())
                    .unwrap_or_default();

                let mut any_dirty = false;
                let mut check_memos = Vec::new();

                for src in &sources {
                    if let Some(sn) = rt.nodes.get(*src) {
                        if sn.state == NodeState::Dirty {
                            any_dirty = true;
                            break;
                        }
                        if matches!(sn.kind, NodeKind::Memo { .. }) && sn.state != NodeState::Clean
                        {
                            check_memos.push(*src);
                        }
                    }
                }

                if !any_dirty && !check_memos.is_empty() {
                    return Some(EffectAction::UpdateMemosFirst(check_memos));
                }

                if !any_dirty {
                    if let Some(node) = rt.nodes.get_mut(id) {
                        node.state = NodeState::Clean;
                    }
                    return None;
                }
            }
            NodeState::Dirty => {}
        }

        rt.clear_sources(id);
        let node = rt.nodes.get_mut(id)?;
        if node.running {
            panic!(
                "Circular dependency detected: effect {:?} is already running",
                id
            );
        }
        node.running = true;
        let f = match &node.kind {
            NodeKind::Effect { f } => Rc::clone(f),
            _ => return None,
        };
        rt.observer_stack.push(id);
        Some(EffectAction::Run(f))
    });

    match action {
        Some(EffectAction::Run(f)) => {
            let _guard = ObserverGuard { id };
            f();
            // Guard will pop observer + clear running on drop (even if f() panics).
            // On success path, also set Clean:
            with_runtime(|rt| {
                if let Some(node) = rt.nodes.get_mut(id) {
                    node.state = NodeState::Clean;
                }
            });
        }
        Some(EffectAction::UpdateMemosFirst(memos)) => {
            for memo_id in memos {
                execute_memo_update(memo_id);
            }
            execute_effect(id);
        }
        None => {}
    }
}

enum MemoAction {
    Run(MemoFn, CompareFn),
    UpdateSourcesFirst(Vec<NodeId>),
}

pub(crate) fn execute_memo_update(id: NodeId) {
    let action = with_runtime(|rt| {
        let state = match rt.nodes.get(id) {
            Some(n) => n.state,
            None => return None,
        };

        match state {
            NodeState::Clean => return None,
            NodeState::Check => {
                let sources: Vec<NodeId> = rt
                    .nodes
                    .get(id)
                    .map(|n| n.sources.iter().copied().collect())
                    .unwrap_or_default();

                let mut any_dirty = false;
                let mut check_memos = Vec::new();

                for src in &sources {
                    if let Some(sn) = rt.nodes.get(*src) {
                        if sn.state == NodeState::Dirty {
                            any_dirty = true;
                            break;
                        }
                        if matches!(sn.kind, NodeKind::Memo { .. }) && sn.state != NodeState::Clean
                        {
                            check_memos.push(*src);
                        }
                    }
                }

                if !any_dirty && !check_memos.is_empty() {
                    return Some(MemoAction::UpdateSourcesFirst(check_memos));
                }

                if !any_dirty {
                    if let Some(node) = rt.nodes.get_mut(id) {
                        node.state = NodeState::Clean;
                    }
                    return None;
                }
            }
            NodeState::Dirty => {}
        }

        rt.clear_sources(id);
        let node = rt.nodes.get_mut(id)?;
        if node.running {
            panic!(
                "Circular dependency detected: memo {:?} is already running",
                id
            );
        }
        node.running = true;
        let (f, compare) = match &node.kind {
            NodeKind::Memo { f, compare } => (Rc::clone(f), Rc::clone(compare)),
            _ => return None,
        };
        rt.observer_stack.push(id);
        Some(MemoAction::Run(f, compare))
    });

    match action {
        Some(MemoAction::Run(f, compare)) => {
            let _guard = ObserverGuard { id };
            let new_value = f();
            // Guard handles pop + running=false on drop (panic-safe).
            // On success, update value and notify:
            with_runtime(|rt| {
                if let Some(node) = rt.nodes.get_mut(id) {
                    let changed = match &node.value {
                        Some(old) => !compare(old.as_ref(), new_value.as_ref()),
                        None => true,
                    };
                    node.value = Some(new_value);
                    node.state = NodeState::Clean;
                    if changed {
                        rt.notify_subscribers(id);
                    }
                }
            });
        }
        Some(MemoAction::UpdateSourcesFirst(memos)) => {
            for memo_id in memos {
                execute_memo_update(memo_id);
            }
            execute_memo_update(id);
        }
        None => {}
    }
}

pub(crate) fn flush_pending_effects() {
    loop {
        let effects = with_runtime(|rt| {
            let raw = std::mem::take(&mut rt.pending_effects);
            let mut seen = super::runtime::FxIndexSet::<NodeId>::default();
            for id in raw {
                seen.insert(id);
            }
            seen.into_iter().collect::<Vec<_>>()
        });
        if effects.is_empty() {
            break;
        }
        for id in effects {
            execute_effect(id);
        }
    }
}
