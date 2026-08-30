//! Advanced signal tests: dispose, nested effects, peek, panic safety,
//! ordering guarantees.

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::signal::api::*;
    use crate::signal::execution::execute_effect;
    use crate::signal::runtime::{
        FxIndexSet, NodeKind, NodeState, RUNTIME, ReactiveNode, ReactiveRuntime,
    };

    fn reset_runtime() {
        RUNTIME.with(|rt| *rt.borrow_mut() = ReactiveRuntime::new());
    }

    #[test]
    fn dispose_removes_subscriptions() {
        reset_runtime();
        let (read, write) = create_signal(0);
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        let effect_id = crate::signal::runtime::with_runtime(|rt| {
            rt.nodes.insert(ReactiveNode {
                value: None,
                kind: NodeKind::Effect {
                    f: Rc::new(move || {
                        let _ = read.get();
                        c.set(c.get() + 1);
                    }),
                },
                state: NodeState::Dirty,
                sources: FxIndexSet::default(),
                subscribers: FxIndexSet::default(),
                running: false,
            })
        });
        execute_effect(effect_id);
        assert_eq!(count.get(), 1);
        write.set(1);
        assert_eq!(count.get(), 2);
        dispose_node(effect_id);
        write.set(2);
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn nested_effects() {
        reset_runtime();
        let (a, set_a) = create_signal(1);
        let (b, set_b) = create_signal(10);
        let outer_count = Rc::new(Cell::new(0));
        let inner_count = Rc::new(Cell::new(0));
        let oc = outer_count.clone();
        let ic = inner_count.clone();
        create_effect(move || {
            let _ = a.get();
            oc.set(oc.get() + 1);
            let ic2 = ic.clone();
            create_effect(move || {
                let _ = b.get();
                ic2.set(ic2.get() + 1);
            });
        });
        assert_eq!(outer_count.get(), 1);
        assert_eq!(inner_count.get(), 1);
        set_b.set(20);
        assert_eq!(outer_count.get(), 1);
        assert_eq!(inner_count.get(), 2);
        set_a.set(2);
        assert_eq!(outer_count.get(), 2);
    }

    #[test]
    fn signal_with_borrow() {
        reset_runtime();
        let (read, write) = create_signal(vec![1, 2, 3]);
        let len = read.with(|v| v.len());
        assert_eq!(len, 3);
        write.update(|v| v.push(4));
        let len = read.with(|v| v.len());
        assert_eq!(len, 4);
    }

    #[test]
    fn peek_does_not_track() {
        reset_runtime();
        let (a, set_a) = create_signal(1);
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        create_effect(move || {
            let _ = a.peek(); // peek, not get
            c.set(c.get() + 1);
        });
        assert_eq!(count.get(), 1);
        set_a.set(2);
        // Effect should NOT re-run because peek() doesn't track
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn peek_returns_current_value() {
        reset_runtime();
        let (read, write) = create_signal(42);
        assert_eq!(read.peek(), 42);
        write.set(99);
        assert_eq!(read.peek(), 99);
    }

    #[test]
    fn observer_guard_panic_safety() {
        reset_runtime();
        let (read, _write) = create_signal(0);

        // Create an effect that panics on first run but is caught
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            create_effect(move || {
                let v = read.get();
                c.set(c.get() + 1);
                if v == 0 {
                    panic!("intentional test panic");
                }
            });
        }));
        assert!(result.is_err(), "Effect should have panicked");

        // After panic, the runtime should still be functional
        // (observer stack restored by RAII guard)
        let (read2, write2) = create_signal(10);
        let val = Rc::new(Cell::new(0));
        let v = val.clone();
        create_effect(move || {
            v.set(read2.get());
        });
        assert_eq!(val.get(), 10);
        write2.set(20);
        assert_eq!(val.get(), 20);
    }

    #[test]
    fn fxindexset_preserves_order() {
        reset_runtime();
        let (a, set_a) = create_signal(0);
        let order = Rc::new(RefCell::new(Vec::new()));
        let o1 = order.clone();
        create_effect(move || {
            let _ = a.get();
            o1.borrow_mut().push(1);
        });
        let o2 = order.clone();
        create_effect(move || {
            let _ = a.get();
            o2.borrow_mut().push(2);
        });
        // Both effects ran on creation
        assert_eq!(*order.borrow(), vec![1, 2]);
        order.borrow_mut().clear();
        set_a.set(1);
        // Effects should run in insertion order (1 then 2)
        assert_eq!(*order.borrow(), vec![1, 2]);
    }
}
