//! The tooltip's key (the toolbar's `Action`) and its focus: shown at
//! once, hidden at once, and the key's tokens on core and elsewhere.

use super::*;
use crate::widgets::overlay::tokens::TooltipTokens;

fn drawn(tip: &Tooltip, theme: &Theme, at: Rect) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    tip.render_at(&mut c, LayerId::DEFAULT, theme, at);
    c.layer(LayerId::DEFAULT)
        .map(|l| l.nodes().to_vec())
        .unwrap_or_default()
}

#[test]
fn tooltip_key_tokens_are_the_toolbar_s_on_core_and_the_theme_s_elsewhere() {
    let k = &Theme::hoff_core().overlay.tooltip.key;
    // size-5 rounded-sm, !p-0.5 !pl-2, gap-2.
    assert_eq!((k.size, k.radius, k.gap), (20.0, 4.0, 8.0));
    assert_eq!(k.pad, [2.0, 2.0, 2.0, 8.0]);
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let t = &theme.overlay.tooltip;
        assert_eq!(*t, TooltipTokens::derive(&theme));
        assert!(t.key.size > t.text.line_height, "the key holds its letter");
    }
}

#[test]
fn a_keyed_tooltip_draws_its_key_after_its_text_and_a_plain_one_does_not() {
    let theme = Theme::hoff_core();
    let at = Rect::new(10.0, 20.0, 0.0, 0.0);
    let texts = |nodes: &[SceneNode]| -> Vec<f32> {
        nodes
            .iter()
            .filter_map(|n| match n {
                SceneNode::Text { x, .. } => Some(*x),
                _ => None,
            })
            .collect()
    };
    let plain = Tooltip::new("Copy prompt");
    let (w, h) = plain.size(&theme);
    let nodes = drawn(&plain, &theme, Rect::new(at.x, at.y, w, h));
    let t = &theme.overlay.tooltip;
    assert_eq!(texts(&nodes), [at.x + t.pad_x]);
    let keyed = Tooltip::new("Redo").key("R");
    let (w, h) = keyed.size(&theme);
    let nodes = drawn(&keyed, &theme, Rect::new(at.x, at.y, w, h));
    let xs = texts(&nodes);
    assert_eq!(xs.len(), 2, "the text and the key");
    assert_eq!(xs[0], at.x + t.key.pad[3]);
    assert!(
        xs[1] > at.x + w - t.key.pad[1] - t.key.size,
        "the key at the end"
    );
}

#[test]
fn a_tooltip_shows_at_once_for_the_focus_and_hides_at_once() {
    let mut tip = Tooltip::new("Undo").key("U");
    let anchor = Rect::new(100.0, 100.0, 40.0, 40.0);
    assert!(tip.show(anchor), "it shows");
    assert!(tip.is_visible());
    assert!(!tip.show(anchor), "already shown");
    assert!(!tip.tick(0.1), "no delay to wait out");
    assert!(tip.hide());
    assert!(!tip.is_visible());
    assert!(!tip.hide(), "already hidden");
}
