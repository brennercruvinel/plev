use super::*;
use crate::theme::{SHADOW_MENU, SHADOW_MODAL};

fn nodes(compositor: &Compositor) -> &[SceneNode] {
    compositor.layer(LayerId::DEFAULT).unwrap().nodes()
}

#[test]
fn shadow_emulates_negative_spread_by_shrinking_rect() {
    let mut c = Compositor::new();
    c.begin_frame();
    // SHADOW_MENU: 0 24px 32px -12px rgba(18,18,18,.10)
    shadow(
        &mut c,
        LayerId::DEFAULT,
        100.0,
        50.0,
        240.0,
        180.0,
        24.0,
        &SHADOW_MENU,
    );
    let n = nodes(&c);
    assert_eq!(n.len(), 1);
    match &n[0] {
        SceneNode::Shadow {
            x,
            y,
            w,
            h,
            corner_radius,
            blur_radius,
            offset,
            color,
            ..
        } => {
            // spread -12 shrinks the casting rect by 12px each side
            assert_eq!((*x, *y), (112.0, 62.0));
            assert_eq!((*w, *h), (216.0, 156.0));
            assert_eq!(*corner_radius, 12.0);
            assert_eq!(*blur_radius, 32.0);
            assert_eq!(*offset, [0.0, 24.0]);
            assert!((color[3] - 0.10).abs() < 1e-6);
        }
        other => panic!("expected Shadow node, got {other:?}"),
    }
}

#[test]
fn shadow_with_spread_collapsing_rect_is_skipped() {
    let mut c = Compositor::new();
    c.begin_frame();
    shadow(
        &mut c,
        LayerId::DEFAULT,
        0.0,
        0.0,
        20.0,
        20.0,
        8.0,
        &SHADOW_MODAL[0],
    );
    // spread -16 on a 20px rect collapses it -> nothing drawn
    assert!(nodes(&c).is_empty());
}

#[test]
fn shadow_stack_pushes_one_node_per_spec() {
    let mut c = Compositor::new();
    c.begin_frame();
    shadow_stack(
        &mut c,
        LayerId::DEFAULT,
        0.0,
        0.0,
        400.0,
        216.0,
        32.0,
        &SHADOW_MODAL,
    );
    let count = nodes(&c)
        .iter()
        .filter(|n| matches!(n, SceneNode::Shadow { .. }))
        .count();
    assert_eq!(count, SHADOW_MODAL.len());
}

#[test]
fn edge_light_draws_two_masked_rings() {
    let mut c = Compositor::new();
    c.begin_frame();
    let color = Color::rgba(1.0, 1.0, 1.0, 0.10);
    edge_light(
        &mut c,
        LayerId::DEFAULT,
        0.0,
        0.0,
        100.0,
        40.0,
        12.0,
        1.5,
        color,
    );
    let n = nodes(&c);
    // clip + ring + pop, twice
    assert_eq!(n.len(), 6);
    assert!(matches!(n[0], SceneNode::PushClip { .. }));
    assert!(matches!(n[2], SceneNode::PopClip));
    assert!(matches!(n[3], SceneNode::PushClip { .. }));
    assert!(matches!(n[5], SceneNode::PopClip));
    let ring_alpha = |node: &SceneNode| match node {
        SceneNode::RoundedRect {
            border_width,
            border_color,
            color,
            ..
        } => {
            assert_eq!(*border_width, 1.5);
            assert_eq!(color[3], 0.0, "ring must be border-only");
            border_color[3]
        }
        other => panic!("expected RoundedRect ring, got {other:?}"),
    };
    let full = ring_alpha(&n[1]);
    let faded = ring_alpha(&n[4]);
    assert!((full - 0.10).abs() < 1e-6);
    assert!((faded - 0.05).abs() < 1e-6, "tail ring is half strength");
}

#[test]
fn glass_fills_background_then_rims() {
    let mut c = Compositor::new();
    c.begin_frame();
    let bg = Color::rgba(40.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0, 0.7);
    let edge = Color::rgba(1.0, 1.0, 1.0, 0.05);
    glass(
        &mut c,
        LayerId::DEFAULT,
        0.0,
        0.0,
        240.0,
        120.0,
        24.0,
        bg,
        Some((1.0, edge)),
    );
    let n = nodes(&c);
    // 1 fill + 6 edge-light nodes
    assert_eq!(n.len(), 7);
    match &n[0] {
        SceneNode::RoundedRect {
            color,
            corner_radius,
            border_width,
            ..
        } => {
            assert!((color[3] - 0.7).abs() < 1e-6);
            assert_eq!(*corner_radius, 24.0);
            assert_eq!(*border_width, 0.0);
        }
        other => panic!("expected fill RoundedRect, got {other:?}"),
    }
}

#[test]
fn measure_text_uses_real_shaping_not_per_char_heuristic() {
    // "Commit" at 14px weight 600 (the button label style). The old
    // per-char heuristic (`chars * size * 0.58`) gave 48.72px; the real
    // Rubik SemiBold shaping is ~53.7px (verified against the
    // rasterizer's FontSystem). Sizing pills with the heuristic made
    // labels overflow their shapes by up to ~10%.
    let style = TextStyle::new(14.0).with_weight(600);
    let measured = measure_text("Commit", &style);
    let old_heuristic = "Commit".chars().count() as f32 * 14.0 * 0.58;
    assert!(
        (measured - old_heuristic).abs() > 1.0,
        "measure_text must differ from the old heuristic: real {measured} vs heuristic {old_heuristic}"
    );
    assert!(
        (50.0..60.0).contains(&measured),
        "Commit @14/600 should shape to ~53.7px, got {measured}"
    );
}

#[test]
fn measure_text_respects_font_weight() {
    // Weight is part of the style: SemiBold advances are wider than
    // Regular for the same string — the heuristic could not see this.
    let regular = measure_text("MODIFIED", &TextStyle::new(14.0));
    let semibold = measure_text("MODIFIED", &TextStyle::new(14.0).with_weight(600));
    assert!(
        semibold > regular,
        "600 ({semibold}) must measure wider than 400 ({regular})"
    );
}

#[test]
fn glass_without_edge_is_a_single_fill() {
    let mut c = Compositor::new();
    c.begin_frame();
    glass(
        &mut c,
        LayerId::DEFAULT,
        0.0,
        0.0,
        100.0,
        44.0,
        16.0,
        Color::rgba(1.0, 1.0, 1.0, 0.02),
        None,
    );
    assert_eq!(nodes(&c).len(), 1);
}

// -- truncate_to_width (single-line ellipsis via real shaping) --------------

fn style_14_600() -> TextStyle {
    TextStyle::new(14.0)
        .with_line_height(14.0 * 1.4)
        .with_weight(600)
}

#[test]
fn truncate_to_width_keeps_short_strings() {
    let s = "fix: bug";
    assert_eq!(truncate_to_width(s, 500.0, &style_14_600()), s);
}

#[test]
fn truncate_to_width_ellipsizes_long_strings_to_one_line() {
    let s = "a very long commit message that would otherwise wrap onto two lines";
    let style = style_14_600();
    let out = truncate_to_width(s, 120.0, &style);
    assert!(out.ends_with('\u{2026}'));
    assert!(out.chars().count() < s.chars().count());
    // The result must REALLY fit: measured with the same shaping the
    // renderer uses, no heuristic slack. (The old per-char model only
    // guaranteed `<= avail + one estimated char`.)
    assert!(measure_text(&out, &style) <= 120.0);
}

#[test]
fn truncate_to_width_maximizes_kept_text() {
    // Regression against over-truncation: the cut must land exactly at
    // the real shaped limit (largest prefix whose "prefix…" fits), not
    // at a per-char guess. Brute-force the optimum and compare.
    let s = "a very long commit message that would otherwise wrap onto two lines";
    let style = style_14_600();
    let avail = 120.0;
    let out = truncate_to_width(s, avail, &style);

    let chars: Vec<char> = s.chars().collect();
    let candidate = |n: usize| -> String {
        let t: String = chars[..n].iter().collect();
        format!("{}\u{2026}", t.trim_end())
    };
    let best = (0..chars.len())
        .rev()
        .find(|&n| measure_text(&candidate(n), &style) <= avail)
        .expect("at least the bare ellipsis fits 120px");
    assert_eq!(out, candidate(best));
}
