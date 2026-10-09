//! The probe of task-79 phase 2 (the mirror answers, the secret does not
//! cross): over every gallery page in both main themes, what a viewer
//! finds under a click from the bindings a frame carries, and what a
//! mask of the secrets would reach. A measurement, ignored; it prints:
//!
//!     cargo test -p showcase --test mirror_probe -- --ignored --nocapture
//!
//! Per page: the bound controls; those the gallery's projection does not
//! hold (a mask that fails closed would hide them); those marked secret;
//! the bindings on a hidden layer (`Compositor::hit` skips them); the
//! pairs of bound controls that overlap where the bind order and the
//! stacking order disagree (a viewer that takes the last binding holding
//! a point, in bind order, would hit another control than the app); and
//! the action a click would press on each bound control
//! (`engine::accessibility::primary`, a screen reader's click).
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::collections::BTreeMap;

use engine::drawing::compositor::Compositor;
use uigallery::views::{PAGES, ShowcaseView};

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 800.0;

fn settle(view: &mut ShowcaseView) {
    let mut dt = 0.0;
    for _ in 0..240 {
        if !view.tick(dt) {
            return;
        }
        dt = 1.0 / 60.0;
    }
}

/// `(z, rank, order)`: where a binding stacks, as `Compositor::hit` ranks.
fn stack(c: &Compositor, id: &accord::SemId) -> Option<(i32, usize, u32)> {
    let b = c.binding(id)?;
    let (rank, layer) = c
        .layers()
        .iter()
        .enumerate()
        .find(|(_, l)| l.id == b.layer)?;
    Some((layer.z_order, rank, b.order))
}

fn overlap(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}

#[test]
#[ignore = "the probe of task-79 phase 2: run with --ignored --nocapture"]
fn probe() {
    let mut presses: BTreeMap<String, usize> = BTreeMap::new();
    let (mut bound_all, mut unheld_all, mut secret_all, mut hidden_all, mut crossed_all) =
        (0, 0, 0, 0, 0);
    let mut pages_crossed = Vec::new();
    let mut pages_unheld = Vec::new();
    for theme in ["core", "hoff"] {
        for p in PAGES {
            let mut view = ShowcaseView::new(WIDTH, HEIGHT);
            view.jump_to_page(p.title);
            view.apply_theme(theme);
            view.resize(WIDTH, HEIGHT, 1.0);
            settle(&mut view);
            let mut c = Compositor::new();
            view.render(&mut c);
            let snap = view.gallery_semantics().ok();
            let bound: Vec<(accord::SemId, [f32; 4])> = c
                .visible_bindings()
                .map(|(id, r)| (id.clone(), r))
                .collect();
            let unheld: Vec<String> = bound
                .iter()
                .filter(|(id, _)| snap.as_ref().is_none_or(|s| s.get(id).is_none()))
                .map(|(id, _)| id.to_string())
                .collect();
            let secret = snap.as_ref().map_or(0, |s| {
                s.nodes().values().filter(|n| n.constraints.secret).count()
            });
            let hidden = c
                .bindings()
                .filter(|(_, b)| {
                    c.layer(b.layer)
                        .is_none_or(|l| !l.visible || l.opacity <= 0.0)
                })
                .count();
            let mut crossed = 0;
            for (i, (a, ra)) in bound.iter().enumerate() {
                for (b, rb) in bound.iter().skip(i + 1) {
                    // a was bound before b: the bind order puts b on top
                    if overlap(*ra, *rb) && stack(&c, a) > stack(&c, b) {
                        crossed += 1;
                    }
                }
            }
            for (id, _) in &bound {
                let press = snap.as_ref().and_then(|s| s.get(id)).map_or(
                    "(not projected)".to_owned(),
                    |n| {
                        engine::control::accessibility::primary(n)
                            .map_or("(none)".to_owned(), |a| a.name.to_string())
                    },
                );
                *presses.entry(press).or_default() += 1;
            }
            if !unheld.is_empty() {
                pages_unheld.push(format!("{} / {theme}: {}", p.title, unheld.join(", ")));
            }
            if crossed > 0 {
                pages_crossed.push(format!("{} / {theme}: {crossed}", p.title));
            }
            bound_all += bound.len();
            unheld_all += unheld.len();
            secret_all += secret;
            hidden_all += hidden;
            crossed_all += crossed;
        }
    }
    println!("\n{} frames", PAGES.len() * 2);
    println!("bound controls, visible: {bound_all}");
    println!("bound but not in the gallery's projection: {unheld_all}");
    for line in &pages_unheld {
        println!("  {line}");
    }
    println!("nodes marked secret in the projection: {secret_all}");
    println!("bindings on a hidden layer: {hidden_all}");
    println!(
        "overlapping pairs where bind order and stacking disagree: {crossed_all} on {} frames",
        pages_crossed.len()
    );
    for line in &pages_crossed {
        println!("  {line}");
    }
    println!("what a click presses on a bound control (primary):");
    for (press, n) in &presses {
        println!("  {press:<20} {n}");
    }
}
