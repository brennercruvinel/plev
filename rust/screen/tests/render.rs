//! Every fixture of task-68 materializes and draws: each node it gives a
//! role is bound to the rect it is drawn at, inside the area the surface
//! was given, at a narrow and a wide width; and a surface at rest asks for
//! no frame of its own.

// shared with the other surface tests: this binary uses the fixtures.
#[allow(dead_code)]
mod support;

use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::Rect;
use plevir::IrNode;
use screen::Surface;
use support::{CASES, snapshot, verified};

/// The fixed ids of `node`'s subtree that take a role (a repeat's
/// template ids are left out: their instances are keyed by item).
fn roled(node: &IrNode, out: &mut Vec<String>) {
    if plevir::role_of(node.component, &node.bind).is_some() {
        out.push(node.sem.to_string());
    }
    for child in &node.children {
        roled(child, out);
    }
}

#[test]
fn every_fixture_draws_and_binds_each_node_it_projects() {
    for (ir_name, snap_name) in CASES {
        for width in [320.0, 1200.0] {
            for theme in [Theme::hoff_core(), Theme::light()] {
                let app = snapshot(snap_name);
                let mut surface = Surface::materialize(verified(ir_name, &app), &app);
                let area = Rect::new(16.0, 24.0, width, 800.0);
                let mut c = Compositor::new();
                surface.render(&mut c, area, &theme);
                let height = surface.height(width, &theme);
                assert!(height > 0.0, "{ir_name} at {width}");

                let mut ids = Vec::new();
                roled(&support::ir(ir_name).surface.root, &mut ids);
                for id in &ids {
                    let b = c
                        .binding(&support::id(id))
                        .unwrap_or_else(|| panic!("{ir_name} at {width}: {id} is not bound"));
                    let [x, y, w, h] = b.rect;
                    assert!(
                        w > 0.0 && h > 0.0,
                        "{ir_name} at {width}: {id} {:?}",
                        b.rect
                    );
                    assert!(
                        x >= area.x - 0.5 && x + w <= area.x + width + 0.5,
                        "{id}: {:?}",
                        b.rect
                    );
                    assert!(
                        y >= area.y - 0.5 && y + h <= area.y + height + 0.5,
                        "{id}: {:?}",
                        b.rect
                    );
                }
            }
        }
    }
}

#[test]
fn a_surface_at_rest_asks_for_no_frame() {
    for (ir_name, snap_name) in CASES {
        let app = snapshot(snap_name);
        let mut surface = Surface::materialize(verified(ir_name, &app), &app);
        let mut c = Compositor::new();
        surface.render(
            &mut c,
            Rect::new(0.0, 0.0, 800.0, 600.0),
            &Theme::hoff_core(),
        );
        let wake = surface.wake(1.0 / 60.0);
        assert!(!wake.now && wake.next.is_none(), "{ir_name}: {wake:?}");
        // and the same state gives the same frame: nothing moved
        assert!(!surface.sync(&app));
    }
}
