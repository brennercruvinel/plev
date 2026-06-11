use super::*;
use plev::compositor::SceneNode;

/// Every rect a layout hands out, flattened (for bounds checks).
fn all_rects(l: &Layout) -> Vec<Rect> {
    let mut v = vec![l.tabs, l.select];
    v.extend(l.fields);
    v.extend(l.checkboxes);
    v.extend(l.switches);
    v.extend(l.sliders);
    v.extend(l.progresses);
    v
}

/// Narrow viewport (~600px window): the columns must stack instead of
/// cropping column B, and nothing may reach past `content.w`.
#[test]
fn forms_columns_stack_in_narrow_content_without_overflow() {
    let theme = Theme::hoff();
    let section = FormsSection::new(&theme);
    let content = Rect::new(288.0, 80.0, 600.0, 700.0);
    let layout = section.layout(content);

    // Column B starts at the left edge, below column A.
    assert_eq!(layout.sliders[0].x, content.x, "column B must stack left");
    assert!(
        layout.sliders[0].y > layout.switches[2].y + layout.switches[2].h,
        "stacked column B must start below column A"
    );

    let right = content.x + content.w;
    for r in all_rects(&layout) {
        assert!(
            r.x + r.w <= right + 0.5,
            "rect {:?} overflows content right edge {right}",
            (r.x, r.y, r.w, r.h)
        );
    }
}

/// Wide viewport (~1600px window): the two columns must actually use
/// the available width instead of huddling at fixed offsets.
#[test]
fn forms_columns_spread_in_wide_content() {
    let theme = Theme::hoff();
    let section = FormsSection::new(&theme);
    let content = Rect::new(288.0, 80.0, 1272.0, 700.0);
    let layout = section.layout(content);

    // Two real columns, side by side.
    assert!(
        layout.sliders[0].x > content.x,
        "wide content must keep column B beside column A"
    );
    assert_eq!(layout.sliders[0].y - LABEL_H, content.y);

    // The columns span well past the old fixed 700px footprint.
    let span = layout.progresses[0].x + layout.progresses[0].w - content.x;
    assert!(
        span >= content.w * 0.6,
        "columns use only {span:.0}px of {:.0}px content",
        content.w
    );
    // …but each column stays clamped for legibility.
    assert!(layout.progresses[0].w <= COL_MAX_W);
    // The select follows its column instead of the old fixed x+380.
    assert_eq!(layout.select.x, layout.sliders[0].x);
    assert!(layout.select.w <= layout.progresses[0].w);
}

/// Every tab label must sit FOLGADO inside its segment: the GOLDEN_SPEC
/// flags cramped tabs as the broken state. We require at least 16px of
/// horizontal breathing room on each side (text centered in the segment).
#[test]
fn forms_tabs_keep_every_label_folgado() {
    let theme = Theme::hoff();
    let section = FormsSection::new(&theme);
    // A representative content rect (matches the page layout origin).
    let content = Rect::new(288.0, 80.0, 760.0, 700.0);
    let layout = section.layout(content);
    let rects = section.tabs.item_rects(layout.tabs);
    let style = TypographyScale::hoff().base_2sm();

    assert_eq!(rects.len(), section.tabs.labels.len());
    for (label, rect) in section.tabs.labels.iter().zip(&rects) {
        let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
        let slack = rect.w - text_w;
        assert!(
            slack >= 32.0,
            "tab '{label}' is cramped: segment {:.1}px holds {:.1}px of text ({:.1}px slack, need >=32)",
            rect.w,
            text_w,
            slack
        );
    }
}

// ---------------------------------------------------------------------------
// Text fields + section focus
// ---------------------------------------------------------------------------

/// Content rect matching the page layout origin.
fn content() -> Rect {
    Rect::new(288.0, 80.0, 760.0, 700.0)
}

/// Default-layer scene of the whole section (headless, no GPU).
fn scene_nodes(section: &FormsSection, content: Rect) -> Vec<SceneNode> {
    let theme = Theme::hoff();
    let mut c = Compositor::new();
    c.begin_frame();
    let overlay = c.create_layer(100);
    section.render(&mut c, overlay, content, &theme);
    c.layer(plev::compositor::LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .to_vec()
}

/// Focus rings in the scene: border-only rounded rects, 2px, theme accent.
fn ring_positions(nodes: &[SceneNode]) -> Vec<(f32, f32)> {
    let accent = Theme::hoff().colors.accent.0;
    nodes
        .iter()
        .filter_map(|n| match *n {
            SceneNode::RoundedRect {
                x,
                y,
                color,
                border_width,
                border_color,
                ..
            } if color[3] == 0.0 && border_width == 2.0 && border_color == accent => Some((x, y)),
            _ => None,
        })
        .collect()
}

/// Tab walks the order (fields first), wraps at the end, Escape blurs.
#[test]
fn tab_cycles_fields_then_widgets_and_wraps() {
    let theme = Theme::hoff();
    let mut s = FormsSection::new(&theme);
    assert_eq!(s.focus_index(), None);

    // First Tab lands on the first text field.
    assert!(s.handle_edit_key(EditKey::Tab));
    assert_eq!(s.focus_index(), Some(0));
    assert_eq!(s.fields.focused(), Some(0));

    // Through the fields, onto the library widgets.
    s.handle_edit_key(EditKey::Tab);
    s.handle_edit_key(EditKey::Tab);
    assert_eq!(s.fields.focused(), Some(2));
    s.handle_edit_key(EditKey::Tab);
    assert_eq!(s.fields.focused(), None, "fields blur when focus moves on");
    assert!(s.tabs.is_focused(), "slot 3 is the tab strip");

    // March to the last slot (select), then wrap back to field 0.
    for _ in 0..7 {
        s.handle_edit_key(EditKey::Tab);
    }
    assert!(s.select.is_focused(), "last slot is the select");
    s.handle_edit_key(EditKey::Tab);
    assert_eq!(s.focus_index(), Some(0), "Tab wraps to the first field");
    assert!(!s.select.is_focused());

    // Escape blurs everything; a second Escape reports nothing to do.
    assert!(s.handle_escape());
    assert_eq!(s.focus_index(), None);
    assert_eq!(s.fields.focused(), None);
    assert!(!s.handle_escape());
}

/// A click inside a field focuses it and places the caret on the clicked
/// glyph (real shaping via TextMeasurer, not chars * factor).
#[test]
fn click_focuses_field_and_places_caret() {
    let theme = Theme::hoff();
    let mut s = FormsSection::new(&theme);
    let layout = s.layout(content());
    let field = layout.fields[1];

    // Click the empty second field: focus + caret at 0.
    let (cx, cy) = field.center();
    let r = s.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, content());
    assert!(r.changed);
    assert_eq!(s.focus_index(), Some(1));

    // Type, then click exactly where the caret for byte 4 sits
    // (8px inner padding before the text).
    assert!(s.handle_text("Brenner"));
    assert_eq!(s.fields.value(1), "Brenner");
    let x = field.x + 8.0 + TextMeasurer::cursor_x("Brenner", fields::FIELD_FONT, 4);
    s.handle_event(&WidgetEvent::MouseDown { x, y: cy }, content());
    assert_eq!(s.fields.cursor(1), 4);

    // A click outside every field blurs the section but is not swallowed.
    let r = s.handle_event(
        &WidgetEvent::MouseDown {
            x: content().x + 1.0,
            y: content().y + content().h - 1.0,
        },
        content(),
    );
    assert_eq!(s.focus_index(), None);
    assert!(r.changed);
}

/// Characters only reach a field while one is focused, and editing keys
/// edit the focused buffer.
#[test]
fn typing_requires_focus_and_editing_keys_edit() {
    let theme = Theme::hoff();
    let mut s = FormsSection::new(&theme);
    assert!(!s.handle_text("x"), "no focus: the shell keeps the key");

    s.handle_edit_key(EditKey::Tab);
    s.handle_text("abc");
    s.handle_edit_key(EditKey::Left);
    s.handle_edit_key(EditKey::Backspace);
    assert_eq!(s.fields.value(0), "ac");
    s.handle_edit_key(EditKey::End);
    s.handle_edit_key(EditKey::Delete); // nothing right of the caret
    assert_eq!(s.fields.value(0), "ac");

    // Editing keys without a field focused (focus on the tab strip) are
    // not consumed.
    for _ in 0..3 {
        s.handle_edit_key(EditKey::Tab);
    }
    assert!(s.tabs.is_focused());
    assert!(!s.handle_edit_key(EditKey::Backspace));
}

/// The scene mirrors the typed value live (field text + preview line).
#[test]
fn scene_shows_typed_value_and_live_preview() {
    let theme = Theme::hoff();
    let mut s = FormsSection::new(&theme);

    let texts = |s: &FormsSection| -> Vec<String> {
        scene_nodes(s, content())
            .iter()
            .filter_map(|n| match n {
                SceneNode::Text { key, .. } => Some(key.text.clone()),
                _ => None,
            })
            .collect()
    };
    let before = texts(&s);
    assert!(before.iter().any(|t| t == "TEXT FIELDS"), "group label");
    assert!(
        before.iter().any(|t| t == "Full name"),
        "placeholder shows while empty"
    );
    assert!(before.iter().any(|t| t == "live value: (empty)"));

    s.handle_edit_key(EditKey::Tab);
    s.handle_text("Ada");
    let after = texts(&s);
    assert!(after.iter().any(|t| t == "Ada"), "typed value rendered");
    assert!(
        after.iter().any(|t| t == "live value: Ada"),
        "preview mirrors the buffer live"
    );
}

/// The accent focus ring follows the focused widget through the scene.
#[test]
fn focus_ring_tracks_focus_through_the_scene() {
    let theme = Theme::hoff();
    let mut s = FormsSection::new(&theme);
    let layout = s.layout(content());

    assert!(
        ring_positions(&scene_nodes(&s, content())).is_empty(),
        "no focus, no ring"
    );

    // Field 0 focused: one ring, 4px (offset+stroke) outside the field.
    s.handle_edit_key(EditKey::Tab);
    let rings = ring_positions(&scene_nodes(&s, content()));
    assert_eq!(
        rings,
        vec![(layout.fields[0].x - 4.0, layout.fields[0].y - 4.0)]
    );

    // Move to the tab strip: the ring follows (drawn by the widget).
    for _ in 0..3 {
        s.handle_edit_key(EditKey::Tab);
    }
    let rings = ring_positions(&scene_nodes(&s, content()));
    assert_eq!(rings, vec![(layout.tabs.x - 4.0, layout.tabs.y - 4.0)]);
}

/// While a field is focused the section keeps requesting frames (cursor
/// blink) and the view-level shortcuts stay out of the buffer's way.
#[test]
fn focused_field_animates_and_captures_view_shortcuts() {
    use super::super::{Section, ShowcaseView};

    let theme = Theme::hoff();
    let mut s = FormsSection::new(&theme);
    assert!(!s.tick(0.016), "idle section requests no frames");
    s.handle_edit_key(EditKey::Tab);
    assert!(s.tick(0.016), "blink needs frames while focused");

    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.section = Section::Forms;
    assert!(view.handle_key("t"), "shortcut consumed (theme toggles)");
    assert_eq!(view.theme_name, "dark");
    view.forms.handle_edit_key(EditKey::Tab);
    assert!(view.handle_key("t"), "focused field captures the char");
    assert_eq!(view.theme_name, "dark", "theme must NOT toggle");
    assert_eq!(view.forms.fields.value(0), "t");

    // Escape blurs via the same path the shell uses.
    assert!(view.close_top_overlay());
    assert_eq!(view.forms.focus_index(), None);
}
