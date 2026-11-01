//! The loading pieces (front C of task-104): the kit's scene card
//! skeleton, its video player rail as the progress bar, and the spinner
//! derived from that rail, over every width they are given.

use super::*;
use crate::widgets::content::{Skeleton, SkeletonShape};
use crate::widgets::feedback::{ProgressBar, Spinner, SpinnerSize};

fn themes() -> [Theme; 4] {
    [core(), Theme::hoff(), Theme::dark(), Theme::light()]
}

fn drawn(render: impl FnOnce(&mut Compositor)) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    render(&mut c);
    c.layer(LayerId::DEFAULT)
        .map(|l| l.nodes().to_vec())
        .unwrap_or_default()
}

fn pills(nodes: &[SceneNode]) -> Vec<(Rect, f32, [f32; 4])> {
    nodes
        .iter()
        .filter_map(|n| match *n {
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                corner_radius,
                color,
                ..
            } => Some((Rect::new(x, y, w, h), corner_radius, color)),
            _ => None,
        })
        .collect()
}

/// The core carries `SceneCardSkeleton`: lines 8 tall at a 2 px corner in
/// #F1F1F1 at rest, blocks in #F8F7F7 at 16, the last line 72 of 128.
#[test]
fn a_skeleton_draws_the_kit_s_bars_and_blocks() {
    let t = core();
    let s = &t.message.skeleton;
    assert_eq!((s.line_fill, s.block), (SHADES[2], SHADES[1]));
    let sk = Skeleton::new(SkeletonShape::Lines).lines(3);
    let bars = pills(&drawn(|c| {
        sk.render(c, Rect::new(0.0, 0.0, 256.0, 60.0), &t)
    }));
    assert_eq!(bars.len(), 3);
    for (r, radius, color) in &bars {
        assert_eq!((r.h, *radius), (8.0, 2.0));
        assert_eq!(*color, SHADES[2].0);
    }
    assert_eq!(bars[1].0.y - bars[0].0.y, 20.0, "p-1 h-2 p-1 mb-1");
    assert_eq!(bars[0].0.x, 4.0, "p-1");
    assert_eq!(bars[2].0.w, (256.0 - 8.0) * 72.0 / 128.0);
    assert_eq!(
        sk.preferred_height(&t),
        Some(56.0),
        "three boxes of 16, two gaps of 4"
    );
}

/// The scene card: as tall as its width keeps the image's aspect, and at
/// every width its image and lines inside it; at the kit's 260 the lines
/// are the kit's 128 and 72.
#[test]
fn a_scene_card_skeleton_holds_its_parts_at_every_width() {
    let card = Skeleton::new(SkeletonShape::Card);
    let t = core();
    let lines: Vec<f32> = {
        let h = card.height_for(260.0, &t).unwrap();
        let p = pills(&drawn(|c| {
            card.render(c, Rect::new(0.0, 0.0, 260.0, h), &t)
        }));
        p.iter().skip(2).map(|(r, _, _)| r.w).collect()
    };
    assert!(
        (lines[0] - 128.0).abs() < 0.01 && (lines[1] - 72.0).abs() < 0.01,
        "{lines:?}"
    );
    for t in themes() {
        for w in (0..=640).step_by(4) {
            let w = w as f32;
            let h = card.height_for(w, &t).unwrap();
            let bounds = Rect::new(4.0, 8.0, w, h);
            for n in drawn(|c| card.render(c, bounds, &t)) {
                if let SceneNode::RoundedRect {
                    x, y, w: nw, h: nh, ..
                } = n
                {
                    let r = Rect::new(x, y, nw, nh);
                    assert!(
                        r.x >= bounds.x - 0.01 && r.right() <= bounds.right() + 0.01,
                        "{w}"
                    );
                    assert!(
                        r.y >= bounds.y - 0.01 && r.bottom() <= bounds.bottom() + 0.01,
                        "{w}"
                    );
                }
            }
        }
    }
}

/// The rail: as thin as the theme's track, the fill to the value, the
/// head at its end, all inside the bounds at every value and width; the
/// head reads at 3:1 on the paper and on the track, per intent and theme.
#[test]
fn the_rail_s_head_marks_the_value_and_reads() {
    let t = core();
    let bar = ProgressBar::new(0.4);
    let l = bar.layout(Rect::new(0.0, 0.0, 200.0, 20.0), &t);
    assert_eq!(l.track.h, 3.0, "h-0.75");
    assert_eq!(l.fill.w, 80.0);
    assert_eq!(
        (l.head.right(), l.head.w),
        (80.0, 5.0),
        "after:w-1.25 right-0"
    );
    assert_eq!(
        t.message.progress.head.neutral, SHADES[8],
        "after:bg-[#121212]"
    );
    for t in themes() {
        let p = &t.message.progress;
        let paper = on(
            Color([t.colors.bg.0[0], t.colors.bg.0[1], t.colors.bg.0[2], 1.0]),
            Color::BLACK,
        );
        let rail = on(p.track, paper);
        for intent in [
            Intent::Neutral,
            Intent::Informational,
            Intent::Constructive,
            Intent::Destructive,
        ] {
            let head = p.head.of(intent);
            assert!(ratio(head, paper) >= 3.0, "{intent:?} on the paper");
            assert!(ratio(head, rail) >= 3.0, "{intent:?} on the track");
            for w in (0..=400).step_by(10) {
                for v in [0.0, 0.01, 0.5, 1.0] {
                    let b = Rect::new(2.0, 2.0, w as f32, 12.0);
                    let l = ProgressBar::new(v).intent(intent).layout(b, &t);
                    for r in [l.track, l.fill, l.head] {
                        assert!(
                            r.x >= b.x - 0.01 && r.right() <= b.right() + 0.01,
                            "{w} {v}"
                        );
                    }
                }
            }
        }
    }
}

/// A reader hears the rail as a progress bar with its value in 0..1.
#[test]
fn the_rail_projects_its_value_and_range() {
    let root = id("page");
    let seen = snap(|b| {
        b.node(&root, Role::Region, "Progress");
        ProgressBar::new(0.25).project(b, &id("page.bar"), Some(&root), "Upload");
    });
    let n = seen.get(&id("page.bar")).unwrap();
    assert_eq!((n.role, n.label.as_str()), (Role::ProgressBar, "Upload"));
    assert_eq!(n.state.get("value"), Some(&accord::Value::Num(0.25)));
    assert!(n.actions.is_empty());
}

/// The spinner is the rail wound: the track ring in its color, an arc in
/// the head's that reads at 3:1 on the paper, at every size.
#[test]
fn the_spinner_is_the_rail_wound_into_a_ring() {
    for t in themes() {
        let s = &t.message.spinner;
        let paper = on(
            Color([t.colors.bg.0[0], t.colors.bg.0[1], t.colors.bg.0[2], 1.0]),
            Color::BLACK,
        );
        assert!(ratio(s.arc, paper) >= 3.0);
        for size in [SpinnerSize::Sm, SpinnerSize::Md, SpinnerSize::Lg] {
            let n = drawn(|c| {
                Spinner::new()
                    .size(size)
                    .render(c, Rect::new(0.0, 0.0, 64.0, 64.0), &t)
            });
            assert!(
                n.iter()
                    .any(|n| matches!(n, SceneNode::RoundedRect { border_color, .. } if *border_color == s.track.0)),
                "the track ring"
            );
        }
    }
    assert_eq!(core().message.spinner.stroke, 3.0, "the rail's h-0.75");
}
