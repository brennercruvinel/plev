//! The modal's sheet: its text wrapped and centered, its art, its footer.

use super::*;

const LONG: &str = "Are you sure you want to delete this file? It leaves every folder it is in, and nothing brings it back.";

#[test]
fn wrap_packs_words_into_the_width_and_keeps_a_hard_break() {
    let style = &Theme::hoff_core().overlay.modal.body;
    let lines = wrap(LONG, style, 200.0);
    assert!(lines.len() > 1, "it wraps");
    for (line, w) in &lines {
        assert!(*w <= 200.0 || !line.contains(' '), "{line} fits");
    }
    let joined: Vec<&str> = lines.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(joined.join(" "), LONG, "every word, in order");
    assert_eq!(wrap("one\ntwo", style, 200.0).len(), 2);
    assert_eq!(
        wrap("", style, 200.0).len(),
        1,
        "an empty line still holds its place"
    );
}

/// On core each line of the body is centered on the sheet on its own,
/// and the sheet is as tall as its lines.
#[test]
fn a_centered_sheet_centers_each_line_and_grows_with_them() {
    let theme = Theme::hoff_core();
    let short = Modal::new("Delete file?", "Gone for good.", "Delete", "Cancel");
    let long = Modal::new("Delete file?", LONG, "Delete", "Cancel");
    let dialog = long.dialog_rect(&theme, 1280.0, 800.0);
    assert!(dialog.h > short.dialog_rect(&theme, 1280.0, 800.0).h);
    let mut c = Compositor::new();
    c.begin_frame();
    long.render_in(&mut c, LayerId::DEFAULT, &theme, dialog);
    let m = &theme.overlay.modal;
    let lines = wrap(LONG, &m.body, dialog.w - m.pad * 2.0);
    let xs: Vec<f32> = c
        .layer(LayerId::DEFAULT)
        .map(|l| l.nodes().to_vec())
        .unwrap_or_default()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { x, color, .. } if *color == m.body_color.0 => Some(*x),
            _ => None,
        })
        .collect();
    assert_eq!(xs.len(), lines.len(), "a node per line");
    let middle = dialog.x + dialog.w / 2.0;
    for (x, (line, w)) in xs.iter().zip(&lines) {
        assert!((x + w / 2.0 - middle).abs() < 0.01, "{line} is centered");
    }
}

/// With art the sheet's text starts under it, the art rises above the
/// sheet by its height less `art_bottom`, and the sheet with its art
/// is what is centered.
#[test]
fn the_art_rises_above_the_sheet_and_is_centered_with_it() {
    let theme = Theme::hoff_core();
    let m = &theme.overlay.modal;
    let pixels = vec![255u8; 8 * 8 * 4];
    let Ok(image) = crate::drawing::gpu::image::load_image_rgba(8, 8, pixels) else {
        panic!("a square image loads");
    };
    let plain = Modal::new("Delete this file?", "Gone.", "Delete", "Cancel");
    let art = plain.clone().art(image);
    let bounds = Rect::new(0.0, 0.0, 1280.0, 1000.0);
    let d = art.dialog_rect_in(&theme, bounds);
    let over = art.overhang(&theme, d.w);
    assert_eq!(
        over,
        d.w - m.art_bottom,
        "a square art, as wide as the sheet"
    );
    assert_eq!(plain.overhang(&theme, d.w), 0.0);
    let grew = art.height(&theme, d.w) - plain.height(&theme, d.w);
    assert_eq!(grew, m.art_pad_top - m.pad, "the text starts under it");
    let above = d.y - over - bounds.y;
    let below = bounds.bottom() - d.bottom();
    assert!((above - below).abs() < 0.01, "centered with its art");
    let mut c = Compositor::new();
    c.begin_frame();
    art.render_in(&mut c, LayerId::DEFAULT, &theme, d);
    let drawn = c
        .layer(LayerId::DEFAULT)
        .map(|l| l.nodes().to_vec())
        .unwrap_or_default();
    let at = drawn.iter().find_map(|n| match n {
        SceneNode::Image { y, h, .. } => Some((*y, *h)),
        _ => None,
    });
    assert_eq!(at, Some((d.y + m.art_bottom - d.w, d.w)));
}

/// The buttons share the footer while their labels fit their halves; in
/// a sheet too narrow for a label they stack across it, the cancel over
/// the confirm, inside the sheet.
#[test]
fn the_buttons_stack_when_a_label_has_no_room() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let m = Modal::new("Delete file?", "Gone.", "Yes, delete it", "Cancel");
        let at = |w: f32| m.dialog_rect_in(&theme, Rect::new(0.0, 0.0, w, 2000.0));
        let wide = at(1280.0);
        let (confirm, cancel) = m.button_rects(&theme, wide);
        assert_eq!(confirm.y, cancel.y, "side by side at {}", wide.w);
        let (cw, _) = m.confirm.preferred_size(&theme);
        assert!(confirm.w >= cw);

        let narrow = at(240.0);
        let (confirm, cancel) = m.button_rects(&theme, narrow);
        let o = &theme.overlay.modal;
        assert!(confirm.y >= cancel.bottom(), "stacked at {}", narrow.w);
        assert_eq!(confirm.w, narrow.w - o.pad * 2.0);
        assert!(confirm.w >= cw && cancel.bottom() <= confirm.y - o.button_gap + 0.01);
        assert!(confirm.bottom() <= narrow.bottom());
    }
}
