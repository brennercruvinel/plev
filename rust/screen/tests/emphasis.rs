//! Emphasis drawn by `stat` and `badge` (task-70 phase 2): the tokens
//! `emphasize` moves, read by the materializer without a change to the
//! vocabulary or to comps. A stat's value takes the size its
//! `typography` names (the inline readout, the card's, the hero's), and
//! a badge filled by its `background` is the filled shape of comps, in
//! the intent the role names, where the plain one is the glass tag.

// shared with the other surface tests: this binary uses the page helpers.
#[allow(dead_code)]
mod support;

use accord::{Actor, Role, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::styling::theme::Theme;
use engine::widgets::Rect;
use plevir::{Capabilities, GenerationPolicy, Op, verify_json};
use screen::SurfaceHost;
use support::{id, settle};

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 800.0,
    h: 600.0,
};

const PANEL: &str = r#"{"plev_ir": 1, "surface": {"id": "panel", "title": "counts",
    "root": {"sem": "panel.root", "layout": {"stack": {"axis": "vertical", "gap": "md"}},
      "children": [
        {"sem": "panel.small", "component": "stat", "label": "small",
         "props": {"typography": "base_m"},
         "bind": {"state": {"value": "counts:total"}}},
        {"sem": "panel.total", "component": "stat", "label": "total",
         "bind": {"state": {"value": "counts:total"}}},
        {"sem": "panel.state", "component": "badge",
         "bind": {"state": {"text": "counts:state"}}}
    ]}}}"#;

fn rect(c: &Compositor, at: &str) -> [f32; 4] {
    c.binding(&id(at))
        .unwrap_or_else(|| panic!("{at} is bound"))
        .rect
}

/// Every fill drawn in `color`.
fn fills(c: &Compositor, color: [f32; 4]) -> usize {
    c.layer(LayerId::DEFAULT)
        .map(|l| {
            l.nodes()
                .iter()
                .filter(|n| matches!(n, SceneNode::RoundedRect { color: k, .. } if *k == color))
                .count()
        })
        .unwrap_or(0)
}

#[test]
fn a_stat_grows_by_its_typography_and_a_badge_fills_by_its_background() {
    let theme = Theme::hoff_core();
    let mut b = SnapshotBuilder::new();
    b.node("counts", Role::Status, "counts")
        .state("total", 12i64)
        .state("state", "pending");
    let app = b.finish().unwrap();
    let caps = Capabilities::from_snapshot(&app);
    let ir = verify_json(PANEL, &caps, &GenerationPolicy::default()).unwrap();
    let mut host = SurfaceHost::new();
    let panel = id("panel");
    host.open(ir, &app).expect("opens");
    let (c, _) = settle(host.get_mut(&panel).unwrap(), AREA, &theme);
    let small = rect(&c, "panel.small");
    let card = rect(&c, "panel.total");
    let tag = rect(&c, "panel.state");
    // the inline readout is shorter than the card's
    assert!(small[3] < card[3], "{small:?} {card:?}");
    let accent = theme.colors.accent.0;
    let filled_before = fills(&c, accent);

    for at in ["panel.small", "panel.total", "panel.state"] {
        host.apply(&panel, Op::Emphasize { id: id(at) }, Actor::Human, &app)
            .unwrap_or_else(|e| panic!("{at}: {e}"));
    }
    let (c, _) = settle(host.get_mut(&panel).unwrap(), AREA, &theme);
    let small_up = rect(&c, "panel.small");
    let hero = rect(&c, "panel.total");
    assert!(
        (small_up[3] - card[3]).abs() <= 1.0,
        "base_m to title: the card's size, {small_up:?} {card:?}"
    );
    assert!(hero[3] > card[3], "title to headline: the hero's");
    let pill = rect(&c, "panel.state");
    assert_ne!(pill, tag, "the filled shape, not the tag");
    assert_eq!(
        fills(&c, accent),
        filled_before + 1,
        "the badge filled with the accent"
    );
}
