//! Charts view tests: pure layout at narrow and wide widths, reveal
//! lifecycle (replay on click, settles, no busy loop) and a headless
//! mid-reveal scene probe. No GPU anywhere.

use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::theme::Theme;
use engine::ui::widgets::{Rect, WidgetEvent};

use super::ChartsSection;

/// The showcase content rect at a given window width (mod.rs geometry).
fn content_at(vw: f32) -> Rect {
    Rect::new(288.0, 118.0, (vw - 368.0).max(200.0), 682.0)
}

#[test]
fn narrow_600px_stacks_the_grid_into_one_column() {
    let s = ChartsSection::new();
    let content = content_at(600.0);
    let rects = s.layout(content);
    assert_eq!(rects.len(), 4);
    assert!(rects.iter().all(|r| r.x == content.x && r.w == content.w));
    for pair in rects.windows(2) {
        assert!(pair[1].y >= pair[0].y + pair[0].h, "panels must stack");
    }
}

#[test]
fn wide_1500px_lays_two_columns_of_two_rows() {
    let s = ChartsSection::new();
    let rects = s.layout(content_at(1500.0));
    let dedup = |mut v: Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.dedup();
        v.len()
    };
    assert_eq!(dedup(rects.iter().map(|r| r.x).collect()), 2, "2 columns");
    assert_eq!(dedup(rects.iter().map(|r| r.y).collect()), 2, "2 rows");
    assert!(rects.iter().all(|r| r.w == rects[0].w));
}

#[test]
fn no_panel_exceeds_the_content_rect_at_any_probed_width() {
    let s = ChartsSection::new();
    for vw in [600.0_f32, 800.0, 1000.0, 1200.0, 1500.0, 2200.0] {
        let content = content_at(vw);
        for r in s.layout(content) {
            assert!(r.x >= content.x);
            assert!(r.x + r.w <= content.x + content.w + 0.5, "overflow at {vw}");
        }
        assert!(s.content_height(content) > 0.0);
    }
    // Stacking makes the page taller than the two-column grid.
    assert!(s.content_height(content_at(600.0)) > s.content_height(content_at(1500.0)));
}

#[test]
fn click_inside_replays_the_reveal_and_settles_click_outside_is_ignored() {
    let mut s = ChartsSection::new();
    assert!(!s.tick(0.016), "construction must not animate");
    let content = content_at(1200.0);
    let outside = WidgetEvent::MouseDown {
        x: content.x - 50.0,
        y: content.y - 50.0,
    };
    let r = s.handle_event(&outside, content);
    assert!(!r.changed && !r.handled && !s.tick(0.016));

    let (cx, cy) = s.layout(content)[0].center();
    let r = s.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, content);
    assert!(r.changed, "replay must request a redraw");
    assert!(s.tick(0.016), "reveal must keep frames flowing");
    let mut frames = 0;
    while s.tick(1.0 / 60.0) {
        frames += 1;
        assert!(frames < 200, "reveal must settle (no busy loop)");
    }
    assert_eq!(s.reveal.get(), 1.0, "reveal completes fully drawn");
}

#[test]
fn mid_reveal_render_emits_paths_labels_and_a_reveal_clip() {
    let mut s = ChartsSection::new();
    let content = content_at(1200.0);
    let (cx, cy) = s.layout(content)[0].center();
    s.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, content);
    s.tick(0.4);
    let mut c = Compositor::new();
    s.render(&mut c, content, &Theme::hoff());
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
    let count = |f: fn(&SceneNode) -> bool| nodes.iter().filter(|n| f(n)).count();
    let paths = count(|n| matches!(n, SceneNode::Path { .. }));
    let texts = count(|n| matches!(n, SceneNode::Text { .. }));
    let clips = count(|n| matches!(n, SceneNode::PushClip { .. }));
    assert!(paths >= 10, "line, dots, bands and slices: got {paths}");
    assert!(texts >= 10, "titles, ticks, values and legend: got {texts}");
    assert_eq!(clips, 1, "the line chart reveal clips its plot");
}
