//! Core signal tests: signal read/write, effects, memos, batch, diamond.

#[cfg(test)]
// The inner module only carries the cfg(test) gate for this
// tests.rs file; the same-name nesting is deliberate.
#[allow(clippy::module_inception)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use crate::signal::api::*;
    use crate::signal::runtime::{RUNTIME, ReactiveRuntime};

    fn reset_runtime() {
        RUNTIME.with(|rt| *rt.borrow_mut() = ReactiveRuntime::new());
    }

    #[test]
    fn signal_read_write() {
        reset_runtime();
        let (read, write) = create_signal(42);
        assert_eq!(read.get(), 42);
        write.set(100);
        assert_eq!(read.get(), 100);
    }

    #[test]
    fn effect_runs_on_creation() {
        reset_runtime();
        let ran = Rc::new(Cell::new(false));
        let r = ran.clone();
        create_effect(move || {
            r.set(true);
        });
        assert!(ran.get());
    }

    #[test]
    fn effect_reruns_on_signal_change() {
        reset_runtime();
        let (read, write) = create_signal(0);
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        create_effect(move || {
            let _ = read.get();
            c.set(c.get() + 1);
        });
        assert_eq!(count.get(), 1);
        write.set(1);
        assert_eq!(count.get(), 2);
        write.set(2);
        assert_eq!(count.get(), 3);
    }

    #[test]
    fn effect_tracks_automatically() {
        reset_runtime();
        let (a, _set_a) = create_signal(1);
        let (_b, set_b) = create_signal(10);
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        create_effect(move || {
            let _ = a.get();
            c.set(c.get() + 1);
        });
        assert_eq!(count.get(), 1);
        set_b.set(20);
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn memo_caches_value() {
        reset_runtime();
        let (read, _write) = create_signal(5);
        let compute_count = Rc::new(Cell::new(0));
        let cc = compute_count.clone();
        let memo = create_memo(move || {
            cc.set(cc.get() + 1);
            read.get() * 2
        });
        assert_eq!(memo.get(), 10);
        assert_eq!(compute_count.get(), 1);
        assert_eq!(memo.get(), 10);
        assert_eq!(compute_count.get(), 1);
    }

    #[test]
    fn memo_recomputes_on_change() {
        reset_runtime();
        let (read, write) = create_signal(5);
        let memo = create_memo(move || read.get() * 2);
        assert_eq!(memo.get(), 10);
        write.set(7);
        assert_eq!(memo.get(), 14);
    }

    #[test]
    fn memo_stops_propagation() {
        reset_runtime();
        let (read, write) = create_signal(5);
        let memo = create_memo(move || {
            let v = read.get();
            if v > 10 { 10 } else { v }
        });
        let effect_count = Rc::new(Cell::new(0));
        let ec = effect_count.clone();
        create_effect(move || {
            let _ = memo.get();
            ec.set(ec.get() + 1);
        });
        assert_eq!(effect_count.get(), 1);
        write.set(6);
        assert_eq!(effect_count.get(), 2);
        write.set(20);
        assert_eq!(effect_count.get(), 3);
        write.set(30);
        assert_eq!(effect_count.get(), 3);
    }

    #[test]
    fn batch_defers_effects() {
        reset_runtime();
        let (a, set_a) = create_signal(0);
        let (b, set_b) = create_signal(0);
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        create_effect(move || {
            let _ = a.get();
            let _ = b.get();
            c.set(c.get() + 1);
        });
        assert_eq!(count.get(), 1);
        batch(|| {
            set_a.set(1);
            set_b.set(2);
        });
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn diamond_problem() {
        reset_runtime();
        let (a, set_a) = create_signal(1);
        let b = create_memo(move || a.get() + 1);
        let c = create_memo(move || a.get() * 2);
        let effect_count = Rc::new(Cell::new(0));
        let ec = effect_count.clone();
        let last_value = Rc::new(Cell::new(0));
        let lv = last_value.clone();
        create_effect(move || {
            let val = b.get() + c.get();
            lv.set(val);
            ec.set(ec.get() + 1);
        });
        assert_eq!(last_value.get(), 4);
        assert_eq!(effect_count.get(), 1);
        set_a.set(3);
        assert_eq!(last_value.get(), 10);
        assert_eq!(effect_count.get(), 2);
    }

    #[test]
    #[should_panic(expected = "Circular dependency")]
    fn circular_dependency_panics() {
        reset_runtime();
        let (read, write) = create_signal(0);
        create_effect(move || {
            let v = read.get();
            write.set(v + 1);
        });
    }

    #[test]
    fn dynamic_dependencies() {
        reset_runtime();
        let (cond, set_cond) = create_signal(true);
        let (a, set_a) = create_signal(1);
        let (b, set_b) = create_signal(10);
        let result = Rc::new(Cell::new(0));
        let r = result.clone();
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        create_effect(move || {
            let val = if cond.get() { a.get() } else { b.get() };
            r.set(val);
            c.set(c.get() + 1);
        });
        assert_eq!(result.get(), 1);
        assert_eq!(count.get(), 1);
        set_b.set(20);
        assert_eq!(count.get(), 1);
        set_cond.set(false);
        assert_eq!(result.get(), 20);
        assert_eq!(count.get(), 2);
        set_a.set(5);
        assert_eq!(count.get(), 2);
        set_b.set(30);
        assert_eq!(result.get(), 30);
        assert_eq!(count.get(), 3);
    }

    #[test]
    fn zero_work_when_unchanged() {
        reset_runtime();
        let (read, _write) = create_signal(42);
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        create_effect(move || {
            let _ = read.get();
            c.set(c.get() + 1);
        });
        assert_eq!(count.get(), 1);
        assert_eq!(count.get(), 1);
    }
}
