//! LiveGraph without a window: the force layout (d3's behavior), a node drag
//! with the simulation warm, compound groups that hold only their members,
//! hits that prefer the child over its group, group and child drags, and the
//! overlay layer that a hover alone changes.

use super::hit::Hit;
use super::live::LiveGraph;
use super::net::{ForceParams, LabelAt, NetEdge, NetGroup, NetLayout, NetNode, NetSpec, Tone};
use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::widget::{Rect, WidgetEvent};

fn node(label: &str, radius: f32) -> NetNode {
    NetNode {
        label: label.into(),
        radius,
        fill: Tone::Fill,
        stroke: Tone::Divider,
        label_at: LabelAt::Center,
    }
}

fn edge(from: usize, to: usize, arrow: bool) -> NetEdge {
    NetEdge {
        from,
        to,
        width: 1.0,
        alpha: 1.0,
        tone: Tone::TextDim,
        arrow,
    }
}

/// A ring of 24 with chords: enough to show d3's settling.
fn force_spec() -> NetSpec {
    let nodes = (0..24).map(|i| node(&format!("n{i}"), 7.0)).collect();
    let mut edges: Vec<NetEdge> = (0..24).map(|i| edge(i, (i + 1) % 24, false)).collect();
    edges.extend((0..24).step_by(3).map(|i| edge(i, (i + 7) % 24, false)));
    let params = ForceParams {
        link_distance: 60.0,
        charge: -200.0,
        collide: 15.0,
    };
    NetSpec {
        nodes,
        edges,
        groups: Vec::new(),
        layout: NetLayout::Force {
            params,
            world: (700.0, 500.0),
        },
        node_font: 9.0,
        group_font: 13.0,
    }
}

/// Cytoscape's compound example: frontend {react, next, vite}, backend {express, fastify, prisma}.
fn compound_spec() -> NetSpec {
    let names = ["react", "next", "vite", "express", "fastify", "prisma"];
    let nodes = names.iter().map(|n| node(n, 38.0)).collect();
    let edges = vec![
        edge(0, 1, true),
        edge(1, 3, true),
        edge(1, 4, true),
        edge(2, 0, true),
        edge(3, 5, true),
        edge(4, 5, true),
    ];
    let groups = vec![
        NetGroup {
            label: "Frontend".into(),
            nodes: vec![0, 1, 2],
            groups: Vec::new(),
        },
        NetGroup {
            label: "Backend".into(),
            nodes: vec![3, 4, 5],
            groups: Vec::new(),
        },
    ];
    let params = ForceParams {
        link_distance: 120.0,
        charge: -400.0,
        collide: 48.0,
    };
    NetSpec {
        nodes,
        edges,
        groups,
        layout: NetLayout::Compound { params },
        node_font: 11.0,
        group_font: 13.0,
    }
}

const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 600.0,
};

fn frame(g: &mut LiveGraph) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    g.render(&mut c, VIEW, &Theme::default());
    c
}

fn intersects(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}

#[test]
fn force_settles_centered_without_overlap_and_the_same_every_time() {
    let (mut a, mut b) = (LiveGraph::new(force_spec()), LiveGraph::new(force_spec()));
    a.settle();
    b.settle();
    assert_eq!(a.positions(), b.positions(), "deterministic");
    let p = a.positions();
    let mean = p.iter().fold((0.0, 0.0), |m, q| (m.0 + q.0, m.1 + q.1));
    let n = p.len() as f32;
    assert!(
        (mean.0 / n - 350.0).abs() < 1.0 && (mean.1 / n - 250.0).abs() < 1.0,
        "{mean:?}"
    );
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let d = (p[i].0 - p[j].0).hypot(p[i].1 - p[j].1);
            assert!(d > 24.0, "n{i} and n{j} at {d}: collision radius 15");
        }
    }
    assert!(!a.moving(), "a settled graph asks for no frame");
}

#[test]
fn a_drag_pins_the_node_and_its_neighbor_follows() {
    let mut g = LiveGraph::new(force_spec());
    g.settle();
    frame(&mut g);
    let start = g.to_screen(g.positions()[0]);
    let n0 = g.positions()[1];
    g.handle_event(
        &WidgetEvent::MouseDown {
            x: start.0 + 2.0,
            y: start.1 + 2.0,
        },
        VIEW,
    );
    assert!(g.moving(), "a held node warms the simulation");
    let (dx, dy) = (150.0, 90.0);
    for k in 1..=60 {
        let f = k as f32 / 60.0;
        g.handle_event(
            &WidgetEvent::MouseMove {
                x: start.0 + 2.0 + dx * f,
                y: start.1 + 2.0 + dy * f,
            },
            VIEW,
        );
        g.tick(1.0 / 60.0);
        frame(&mut g);
    }
    let held = g.to_screen(g.positions()[0]);
    assert!(
        (held.0 - start.0 - dx).abs() < 0.01 && (held.1 - start.1 - dy).abs() < 0.01,
        "under the cursor, offset kept: {held:?}"
    );
    let n1 = g.positions()[1];
    assert!(
        (n1.0 - n0.0) * dx + (n1.1 - n0.1) * dy > 0.0,
        "the neighbor followed"
    );
    let up = g.handle_event(
        &WidgetEvent::MouseUp {
            x: start.0 + 2.0 + dx,
            y: start.1 + 2.0 + dy,
        },
        VIEW,
    );
    assert!(!up.clicked, "it moved: a drag, not a click");
    assert_eq!(g.selected(), None);
}

#[test]
fn compound_boxes_hold_their_members_and_no_one_else() {
    let mut g = LiveGraph::new(compound_spec());
    frame(&mut g);
    for (k, group) in g.spec().groups.clone().iter().enumerate() {
        let b = g.group_box(k).unwrap();
        for (i, &(x, y)) in g.positions().iter().enumerate() {
            let r = g.spec().nodes[i].radius;
            let nb = [x - r, y - r, 2.0 * r, 2.0 * r];
            let inside = nb[0] >= b[0]
                && nb[1] >= b[1]
                && nb[0] + nb[2] <= b[0] + b[2]
                && nb[1] + nb[3] <= b[1] + b[3];
            if group.nodes.contains(&i) {
                assert!(inside, "node {i} in group {k}");
            } else {
                assert!(
                    !intersects(nb, b),
                    "node {i} is not in group {k}: the box must not cover it"
                );
            }
        }
    }
    assert!(
        !intersects(g.group_box(0).unwrap(), g.group_box(1).unwrap()),
        "the compounds do not overlap"
    );
}

#[test]
fn the_child_is_hit_over_its_group_and_the_corner_is_the_group() {
    let mut g = LiveGraph::new(compound_spec());
    frame(&mut g);
    let react = g.to_screen(g.positions()[0]);
    g.handle_event(
        &WidgetEvent::MouseMove {
            x: react.0,
            y: react.1,
        },
        VIEW,
    );
    assert_eq!(g.hovered(), Some(Hit::Node(0)));
    let [x, y, _, _] = g.group_box(0).unwrap();
    let corner = g.to_screen((x + 2.0, y + 2.0));
    g.handle_event(
        &WidgetEvent::MouseMove {
            x: corner.0,
            y: corner.1,
        },
        VIEW,
    );
    assert_eq!(g.hovered(), Some(Hit::Group(0)));
}

#[test]
fn a_group_drag_brings_its_children_and_a_child_drag_grows_the_box() {
    let mut g = LiveGraph::new(compound_spec());
    frame(&mut g);
    let before: Vec<(f32, f32)> = g.positions().to_vec();
    let [x, y, _, _] = g.group_box(0).unwrap();
    let grab = g.to_screen((x + 2.0, y + 2.0));
    let (dx, dy) = (40.0, -25.0);
    g.handle_event(
        &WidgetEvent::MouseDown {
            x: grab.0,
            y: grab.1,
        },
        VIEW,
    );
    g.handle_event(
        &WidgetEvent::MouseMove {
            x: grab.0 + dx,
            y: grab.1 + dy,
        },
        VIEW,
    );
    g.handle_event(
        &WidgetEvent::MouseUp {
            x: grab.0 + dx,
            y: grab.1 + dy,
        },
        VIEW,
    );
    let s = g.scale();
    for (i, (p, q)) in before.iter().zip(g.positions()).take(3).enumerate() {
        assert!(
            (q.0 - p.0 - dx / s).abs() < 1e-3 && (q.1 - p.1 - dy / s).abs() < 1e-3,
            "node {i} moved with its group"
        );
    }
    assert_eq!(
        g.positions()[3],
        before[3],
        "a node of the other group stays"
    );
    frame(&mut g);
    let b = g.group_box(1).unwrap();
    let express = g.positions()[3];
    let away = if express.0 > b[0] + b[2] / 2.0 {
        200.0
    } else {
        -200.0
    };
    let at = g.to_screen(express);
    g.handle_event(&WidgetEvent::MouseDown { x: at.0, y: at.1 }, VIEW);
    g.handle_event(
        &WidgetEvent::MouseMove {
            x: at.0 + away,
            y: at.1,
        },
        VIEW,
    );
    g.handle_event(
        &WidgetEvent::MouseUp {
            x: at.0 + away,
            y: at.1,
        },
        VIEW,
    );
    assert!(
        g.group_box(1).unwrap()[2] > b[2] + 100.0 / s,
        "the box grew to keep the child"
    );
}

#[test]
fn a_hover_changes_the_overlay_and_leaves_the_base_layer_alone() {
    let mut g = LiveGraph::new(compound_spec());
    let c0 = frame(&mut g);
    let react = g.to_screen(g.positions()[0]);
    g.handle_event(
        &WidgetEvent::MouseMove {
            x: react.0,
            y: react.1,
        },
        VIEW,
    );
    let c1 = frame(&mut g);
    let nodes =
        |c: &Compositor, id: LayerId| c.layer(id).map(|l| l.nodes.clone()).unwrap_or_default();
    assert_eq!(
        nodes(&c0, LayerId::DEFAULT),
        nodes(&c1, LayerId::DEFAULT),
        "the base layer is the same scene"
    );
    let overlay = c1
        .layers()
        .iter()
        .map(|l| l.id)
        .find(|id| *id != LayerId::DEFAULT)
        .unwrap();
    assert!(
        nodes(&c1, overlay).len() > nodes(&c0, overlay).len(),
        "the hover ring is in the overlay"
    );
    let click = g.handle_event(
        &WidgetEvent::MouseDown {
            x: react.0,
            y: react.1,
        },
        VIEW,
    );
    assert!(click.handled);
    let up = g.handle_event(
        &WidgetEvent::MouseUp {
            x: react.0,
            y: react.1,
        },
        VIEW,
    );
    assert!(
        up.clicked && g.selected() == Some(Hit::Node(0)),
        "a press without travel selects"
    );
}
