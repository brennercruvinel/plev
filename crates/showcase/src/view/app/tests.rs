use super::*;
use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::text::TextMeasurer;
use engine::theme::Theme;

fn content_at(vw: f32) -> Rect {
    Rect::new(288.0, 118.0, (vw - 328.0).max(200.0), 682.0)
}

fn lay(s: &AppSection, content: Rect) -> Layout {
    compute(content, &s.counter_text())
}

fn focus(s: &mut AppSection, content: Rect) {
    let (cx, cy) = lay(s, content).input.center();
    s.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, content);
}

/// Click in the panel margin above the field: blurs, hits nothing else.
fn blur(s: &mut AppSection, content: Rect) {
    let l = lay(s, content);
    let (x, y) = (l.panel.x + 2.0, l.panel.y + 4.0);
    s.handle_event(&WidgetEvent::MouseDown { x, y }, content);
}

fn add(s: &mut AppSection, content: Rect, text: &str) {
    focus(s, content);
    for ch in text.chars() {
        s.handle_text(&ch.to_string());
    }
    s.handle_enter();
}

fn click(s: &mut AppSection, content: Rect, x: f32, y: f32) -> EventResult {
    let r = s.handle_event(&WidgetEvent::MouseDown { x, y }, content);
    r.merge(s.handle_event(&WidgetEvent::MouseUp { x, y }, content))
}

fn settle(s: &mut AppSection) {
    for _ in 0..600 {
        if !s.tick(1.0 / 60.0) {
            return;
        }
    }
    panic!("tick never settles (busy loop)");
}

fn render(s: &mut AppSection, content: Rect) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    s.render(&mut c, content, &Theme::hoff());
    c
}

fn label_alpha(c: &Compositor, needle: &str) -> Option<f32> {
    c.layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .find_map(|n| match n {
            SceneNode::Text { key, color, .. } if key.text == needle => Some(color[3]),
            _ => None,
        })
}

#[test]
fn layout_holds_at_narrow_and_wide_widths() {
    for vw in [600.0_f32, 1500.0] {
        let content = content_at(vw);
        let s = AppSection::new();
        let l = lay(&s, content);
        assert_eq!(l.panel.w, content.w.min(MAX_W), "vw {vw}");
        assert!(l.panel.x >= content.x);
        // Field height is TextInput's own font_size * 2 rule.
        assert_eq!(l.input.h, INPUT_FONT * 2.0);
        assert!(l.input.x + l.input.w <= l.panel.x + l.panel.w);
        assert!(
            l.list.y + l.list.h <= l.divider_y,
            "list never overflows the footer"
        );
        let last = l.pills[2];
        assert!(last.x + last.w <= l.panel.x + l.panel.w - PAD + 0.5);
        assert!(
            l.pills[2].w > l.pills[0].w,
            "Completed must out-measure All"
        );
        // The counter shares the row only when it fits; otherwise stacked.
        let cw = TextMeasurer::measure_styled(&s.counter_text(), &counter_style(), None).0;
        assert!(
            l.counter.0 + cw <= l.pills[0].x || l.counter.1 + 16.0 <= l.pills[0].y,
            "vw {vw}: counter must never collide with the pills"
        );
        assert_eq!(s.content_height(content), content.h);
    }
}

#[test]
fn short_viewport_hands_overflow_to_page_scroll() {
    let s = AppSection::new();
    let content = Rect::new(288.0, 118.0, 800.0, 120.0);
    assert_eq!(s.content_height(content), MIN_H);
    let l = lay(&s, content);
    assert_eq!(l.panel.h, MIN_H);
    assert!(l.list.h > 0.0, "the card minimum keeps a usable list");
}

#[test]
fn typing_needs_focus_and_enter_adds_trimmed_then_clears() {
    let content = content_at(1500.0);
    let mut s = AppSection::new();
    assert!(!s.handle_text("x"), "unfocused keys stay with the chrome");
    assert!(!s.handle_enter());
    assert!(!s.handle_edit_key(EditKey::Backspace));
    assert!(!s.handle_escape());

    let before = s.model.counts().total;
    focus(&mut s, content);
    assert!(s.input.focused);
    for ch in [" ", "h", "i", "i", " "] {
        assert!(s.handle_text(ch));
    }
    assert!(s.handle_edit_key(EditKey::Left));
    assert!(s.handle_edit_key(EditKey::Backspace));
    assert!(s.handle_enter());
    assert_eq!(s.model.counts().total, before + 1);
    assert_eq!(s.model.visible_items().last().unwrap().text(), "hi");
    assert!(s.input.buffer.is_empty(), "enter clears the field");
    assert!(s.handle_enter(), "empty enter is consumed");
    assert_eq!(s.model.counts().total, before + 1, "but adds nothing");

    // Digits and Space go to the field while focused (the chrome calls
    // handle_text before its hotkeys); Tab stays with the chrome and
    // Escape blurs, handing the hotkeys back.
    assert!(s.handle_text("1"));
    assert_eq!(s.input.buffer.text(), "1");
    assert!(!s.handle_edit_key(EditKey::Tab));
    assert!(s.handle_escape());
    assert!(!s.input.focused);
    assert!(!s.handle_text("1"));
    settle(&mut s);
}

#[test]
fn checkbox_click_toggles_and_strike_animates_to_one() {
    let content = content_at(1500.0);
    let mut s = AppSection::new();
    let cb = checkbox_rect(row_rect(lay(&s, content).list, 0, 0.0));
    let before = s.model.counts().completed;

    let r = click(&mut s, content, cb.x + 9.0, cb.y + ROW_H / 2.0);
    assert!(r.clicked && r.changed);
    assert_eq!(s.model.counts().completed, before + 1);
    assert!(s.model.visible_items()[0].completed());
    assert_eq!(s.model.visible_items()[0].strike_progress(), 0.0);
    assert!(s.tick(0.05), "strike tween must request frames");
    let mid = s.model.visible_items()[0].strike_progress();
    assert!(mid > 0.0 && mid < 1.0, "mid progress, got {mid}");
    settle(&mut s);
    assert!(s.model.visible_items()[0].strike_progress() >= 0.999);
}

#[test]
fn delete_hovers_then_removes() {
    let content = content_at(1500.0);
    let mut s = AppSection::new();
    let del = delete_rect(row_rect(lay(&s, content).list, 1, 0.0));
    let (dx, dy) = del.center();

    let r = s.handle_event(&WidgetEvent::MouseMove { x: dx, y: dy }, content);
    assert!(r.changed, "delete hover must request redraw");
    assert_eq!(s.hover_delete, Some(s.model.visible_items()[1].id()));

    let total = s.model.counts().total;
    let r = s.handle_event(&WidgetEvent::MouseDown { x: dx, y: dy }, content);
    assert!(r.clicked);
    assert_eq!(s.model.counts().total, total - 1);
}

#[test]
fn filter_pills_filter_and_only_changes_redraw() {
    let content = content_at(1500.0);
    let mut s = AppSection::new();
    let (px, py) = lay(&s, content).pills[1].center();

    let r = s.handle_event(&WidgetEvent::MouseDown { x: px, y: py }, content);
    assert!(r.changed && r.clicked);
    assert_eq!(s.model.filter(), Filter::Active);
    assert!(s.model.visible_items().iter().all(|i| !i.completed()));

    let r = s.handle_event(&WidgetEvent::MouseDown { x: px, y: py }, content);
    assert!(r.handled && !r.changed, "re-click must not request redraw");
}

#[test]
fn strike_width_is_measured_not_estimated() {
    let content = content_at(1500.0);
    let mut s = AppSection::new(); // seed: third item completed and settled
    let l = lay(&s, content);
    let (lx, lw) = label_span(row_rect(l.list, 2, 0.0));
    let expect = TextMeasurer::measure_styled(
        "Port the todo domain (tested first)",
        &item_style(),
        Some(lw),
    )
    .0
    .min(lw);
    assert!(expect > 0.0);

    let c = render(&mut s, content);
    let strike = c
        .layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .find_map(|n| match *n {
            SceneNode::Rect { x, w, h, .. } if h == STRIKE_H && (x - lx).abs() < 0.01 => Some(w),
            _ => None,
        })
        .expect("completed row draws a strike");
    assert!(
        (strike - expect).abs() < 0.5,
        "strike {strike} vs measured {expect}"
    );
}

#[test]
fn enter_progress_drives_label_alpha() {
    let content = content_at(1500.0);
    let mut s = AppSection::new();
    add(&mut s, content, "fade me in");
    blur(&mut s, content);

    let a0 = label_alpha(&render(&mut s, content), "fade me in").unwrap();
    assert_eq!(a0, 0.0, "fresh item starts transparent");
    assert!(s.tick(1.0 / 60.0), "enter tween must request frames");
    let a1 = label_alpha(&render(&mut s, content), "fade me in").unwrap();
    assert!(a1 > a0, "alpha follows the tween, got {a1}");
    settle(&mut s);
    let a2 = label_alpha(&render(&mut s, content), "fade me in").unwrap();
    assert!(a2 > 0.5, "settled item reads at full token alpha, got {a2}");
}

#[test]
fn list_scrolls_inside_and_footer_stays_pinned() {
    let content = content_at(1500.0);
    let mut s = AppSection::new();
    for i in 0..14 {
        add(&mut s, content, &format!("item {i}"));
    }
    blur(&mut s, content);
    assert!(s.scroll.is_scrollable());
    let max = s.scroll.offset();
    assert!(max > 0.0, "enter reveals the newest row");

    let l = lay(&s, content);
    let (cx, cy) = l.list.center();
    let r = s.handle_event(
        &WidgetEvent::Scroll {
            x: cx,
            y: cy,
            delta: -60.0,
        },
        content,
    );
    assert!(r.changed && r.handled);
    assert!(s.scroll.offset() < max);

    // Wheel outside the list is left to the page scroll, and the footer
    // never moves: pills stay pinned inside the panel.
    let r = s.handle_event(
        &WidgetEvent::Scroll {
            x: cx,
            y: l.panel.y + 4.0,
            delta: 30.0,
        },
        content,
    );
    assert!(!r.handled);
    assert!(l.pills[0].y + PILL_H <= l.panel.y + l.panel.h);
}
