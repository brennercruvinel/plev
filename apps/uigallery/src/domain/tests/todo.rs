//! tests for the todo domain model: filters, counts, toggle, delete and
//! dt-based animation convergence. no gpu, no mocks, real tweens.

use super::todo::{Counts, Filter, TodoModel};

/// One simulated frame at 60 fps.
const DT: f32 = 1.0 / 60.0;

fn settle(model: &mut TodoModel) {
    // 2 seconds of frames; far beyond every tween duration in the model.
    for _ in 0..120 {
        model.update(DT);
    }
}

fn model_with(texts: &[&str]) -> TodoModel {
    let mut m = TodoModel::new();
    for t in texts {
        m.add(t).expect("non-empty add succeeds");
    }
    m
}

// -- add ---------------------------------------------------------------

#[test]
fn add_trims_and_assigns_increasing_ids() {
    let mut m = TodoModel::new();
    let a = m.add("  buy milk  ").unwrap();
    let b = m.add("walk dog").unwrap();
    assert!(b > a);
    let visible = m.visible_items();
    assert_eq!(visible[0].text(), "buy milk");
    assert_eq!(visible[1].text(), "walk dog");
}

#[test]
fn add_rejects_whitespace_only_input() {
    let mut m = TodoModel::new();
    assert_eq!(m.add("   "), None);
    assert_eq!(m.add(""), None);
    assert_eq!(m.counts().total, 0);
}

// -- filters -----------------------------------------------------------

#[test]
fn filters_partition_items_and_preserve_order() {
    let mut m = model_with(&["a", "b", "c"]);
    let ids: Vec<u64> = m.visible_items().iter().map(|i| i.id()).collect();
    m.toggle(ids[1]);

    assert_eq!(m.filter(), Filter::All);
    assert_eq!(m.visible_items().len(), 3);

    m.set_filter(Filter::Active);
    let active: Vec<&str> = m.visible_items().iter().map(|i| i.text()).collect();
    assert_eq!(active, ["a", "c"]);

    m.set_filter(Filter::Completed);
    let done: Vec<&str> = m.visible_items().iter().map(|i| i.text()).collect();
    assert_eq!(done, ["b"]);
}

#[test]
fn set_filter_reports_change_for_invalidation() {
    let mut m = TodoModel::new();
    assert!(!m.set_filter(Filter::All));
    assert!(m.set_filter(Filter::Active));
    assert!(!m.set_filter(Filter::Active));
}

#[test]
fn filter_labels_match_display_order() {
    let labels: Vec<&str> = Filter::ALL.iter().map(|f| f.label()).collect();
    assert_eq!(labels, ["All", "Active", "Completed"]);
}

// -- counts ------------------------------------------------------------

#[test]
fn counts_track_toggle_and_delete() {
    let mut m = model_with(&["a", "b", "c"]);
    let ids: Vec<u64> = m.visible_items().iter().map(|i| i.id()).collect();

    assert_eq!(
        m.counts(),
        Counts {
            total: 3,
            active: 3,
            completed: 0
        }
    );

    m.toggle(ids[0]);
    assert_eq!(
        m.counts(),
        Counts {
            total: 3,
            active: 2,
            completed: 1
        }
    );

    assert!(m.delete(ids[0]));
    assert_eq!(
        m.counts(),
        Counts {
            total: 2,
            active: 2,
            completed: 0
        }
    );
}

// -- toggle and delete error paths ---------------------------------------

#[test]
fn toggle_round_trips_and_rejects_unknown_id() {
    let mut m = model_with(&["a"]);
    let id = m.visible_items()[0].id();

    assert!(m.toggle(id));
    assert!(m.visible_items()[0].completed());
    assert!(m.toggle(id));
    assert!(!m.visible_items()[0].completed());

    assert!(!m.toggle(999));
}

#[test]
fn delete_rejects_unknown_id_and_is_idempotent() {
    let mut m = model_with(&["a"]);
    let id = m.visible_items()[0].id();
    assert!(!m.delete(999));
    assert!(m.delete(id));
    assert!(!m.delete(id));
    assert_eq!(m.counts().total, 0);
}

// -- animation progress --------------------------------------------------

#[test]
fn enter_progress_starts_at_zero_and_converges_to_one() {
    let mut m = model_with(&["a"]);
    assert_eq!(m.visible_items()[0].enter_progress(), 0.0);

    m.update(0.1);
    let mid = m.visible_items()[0].enter_progress();
    assert!(mid > 0.0 && mid < 1.0, "mid-flight progress was {mid}");

    settle(&mut m);
    assert_eq!(m.visible_items()[0].enter_progress(), 1.0);
}

#[test]
fn strike_progress_converges_to_one_then_back_to_zero() {
    let mut m = model_with(&["a"]);
    let id = m.visible_items()[0].id();
    settle(&mut m);
    assert_eq!(m.visible_items()[0].strike_progress(), 0.0);

    m.toggle(id);
    m.update(0.05);
    let mid = m.visible_items()[0].strike_progress();
    assert!(mid > 0.0 && mid < 1.0, "mid-flight strike was {mid}");
    settle(&mut m);
    assert_eq!(m.visible_items()[0].strike_progress(), 1.0);

    m.toggle(id);
    settle(&mut m);
    assert_eq!(m.visible_items()[0].strike_progress(), 0.0);
}

#[test]
fn update_reports_animating_until_settled_then_goes_quiet() {
    let mut m = model_with(&["a"]);
    assert!(m.update(DT), "fresh item must request frames");
    settle(&mut m);
    assert!(!m.update(DT), "settled model must not burn frames");

    let id = m.visible_items()[0].id();
    m.toggle(id);
    assert!(m.update(DT), "strike must request frames");
    settle(&mut m);
    assert!(!m.update(DT));
}

#[test]
fn progress_is_dt_based_not_frame_based() {
    // same wall time, different frame sizes: both must finish the enter.
    let mut coarse = model_with(&["a"]);
    let mut fine = model_with(&["a"]);
    for _ in 0..4 {
        coarse.update(0.25);
    }
    for _ in 0..100 {
        fine.update(0.01);
    }
    assert_eq!(coarse.visible_items()[0].enter_progress(), 1.0);
    assert_eq!(fine.visible_items()[0].enter_progress(), 1.0);
}
