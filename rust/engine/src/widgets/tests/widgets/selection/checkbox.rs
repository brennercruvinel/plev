//! Selection tests: the checkbox's disabled check.

use super::*;

/// A disabled box that is on keeps its check: the check is the token's
/// color that reads on the faded box (`theme/tests/contrast.rs`), drawn
/// whole, never the knob's paper faded with the box. Under the core kit
/// that is a gray, the paper vanishing there.
#[test]
fn a_disabled_checked_box_keeps_its_check() {
    use crate::widgets::form::Checkbox;
    let core = Theme::hoff_core();
    assert_ne!(
        core.selection.checkbox.mark_disabled,
        core.selection.checkbox.mark
    );
    for theme in [core.clone(), Theme::hoff()] {
        let t = &theme.selection.checkbox;
        let mut c = Compositor::new();
        c.begin_frame();
        Checkbox::new(true)
            .disabled(true)
            .render(&mut c, Rect::new(0.0, 0.0, 18.0, 18.0), &theme);
        let check = c
            .layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .find_map(|n| match n {
                SceneNode::Path { data } => data.vertices.first().map(|v| v.color),
                _ => None,
            });
        assert_eq!(check, Some(t.mark_disabled.0));
    }
}
