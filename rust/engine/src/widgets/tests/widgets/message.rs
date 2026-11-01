//! conversation widgets (front C of task-104): the message, its tokens
//! and its layout over the content it is given.

use super::*;
use crate::widgets::message::message::{Message, MessageKind, MessageLayout};
use crate::widgets::message::tokens::MessageTokens;
use accord::{Role, SemId, Snapshot, SnapshotBuilder};

fn core() -> Theme {
    Theme::hoff_core()
}

const LUNA: &str = "I love this bag, it looks great. Could you make it with leather material? 🤔";

const LONG: &str = "These draggable sliders look really cool. Maybe these could be displayed \
     when you hold shift, to rotate exactly on the X or Y. But by default I don't think we \
     need these controllers, you could just rotate the object by clicking and dragging \
     anywhere as expected on any 3D tool (theoretically).";

/// Six scripts and emoji in one body (adr-0071).
const SCRIPTS: &str =
    "Olá! مرحبا بالعالم · שלום עולם · नमस्ते दुनिया · 你好世界 · こんにちは · 안녕하세요 🎉👍🏽";

fn thread(body: &str) -> Message {
    Message::new("Luna Starfield", "2s", body)
}

fn quick(body: &str) -> Message {
    Message::new("Randomdash", "1h", body)
        .kind(MessageKind::Quick)
        .reaction("2")
}

fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn at(w: f32) -> Rect {
    Rect::new(10.0, 20.0, w, 0.0)
}

fn nodes(m: &Message, bounds: Rect, theme: &Theme) -> (Compositor, Vec<SceneNode>) {
    let mut c = Compositor::new();
    c.begin_frame();
    m.render(&mut c, bounds, theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    (c, nodes)
}

fn texts(nodes: &[SceneNode]) -> Vec<String> {
    nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect()
}

fn finite(r: Rect) -> bool {
    [r.x, r.y, r.w, r.h].iter().all(|v| v.is_finite()) && r.w >= 0.0 && r.h >= 0.0
}

fn within(inner: Rect, outer: Rect) -> bool {
    const EPS: f32 = 0.5;
    inner.x >= outer.x - EPS
        && inner.y >= outer.y - EPS
        && inner.right() <= outer.right() + EPS
        && inner.bottom() <= outer.bottom() + EPS
}

/// Every placed part of `l`, by name.
fn parts(l: &MessageLayout) -> Vec<(&'static str, Rect)> {
    let mut out = vec![("avatar", l.avatar)];
    for (name, p) in [("name", &l.name), ("time", &l.time), ("body", &l.body)] {
        if let Some(p) = p {
            out.push((name, p.rect));
        }
    }
    out.extend(l.seal.map(|r| ("seal", r)));
    out.extend(l.reaction.map(|r| ("reaction", r)));
    out.extend(l.divider.map(|r| ("divider", r)));
    out
}

// tokens

#[test]
fn the_core_theme_carries_the_kit_and_the_others_derive_it() {
    let kit = MessageTokens::hoff_core();
    assert_eq!(*core().message, kit);
    let t = &kit.thread;
    assert_eq!((t.pad, t.gap, t.avatar), (16.0, 16.0, 32.0));
    assert_eq!(
        (t.name.size, t.name.line_height, t.name.weight),
        (12.0, 16.0, 500)
    );
    assert_eq!((t.time.size, t.time_gap), (11.0, 8.0));
    assert_eq!(
        (t.body.size, t.body.line_height, t.body_lines),
        (13.0, 19.52, None)
    );
    let q = &kit.quick;
    assert_eq!((q.pad, q.gap, q.avatar), (12.0, 12.0, 36.0));
    assert_eq!(
        (q.name.weight, q.body.line_height, q.body_lines),
        (600, 20.0, Some(3))
    );
    assert_eq!(kit.divider_width, 1.0);

    let hoff = Theme::hoff();
    let derived = MessageTokens::derive(&hoff);
    assert_eq!(*hoff.message, derived);
    assert_eq!(derived.thread.name.color, hoff.colors.text);
    assert_eq!(derived.thread.time.color, hoff.colors.text_mid);
    assert_eq!(derived.quick.body.color, hoff.colors.text_mid);
    assert_eq!(derived.divider, hoff.colors.divider);
    assert_eq!(derived.mention, hoff.colors.info);
    // The geometry is the kit's under every palette.
    assert_eq!(derived.thread.pad, kit.thread.pad);
    assert_eq!(derived.quick.body_lines, kit.quick.body_lines);
}

// layout

#[test]
fn the_row_is_as_tall_as_its_content() {
    let theme = core();
    let s = theme.message.thread;
    let empty = Message::new("", "", "");
    let l = empty.layout(at(320.0), &theme);
    assert_eq!(
        (l.name.clone(), l.time.clone(), l.body.clone()),
        (None, None, None)
    );
    assert_eq!(
        l.row.h,
        s.pad * 2.0 + s.avatar,
        "an empty message is its avatar"
    );

    let one = thread("ok").layout(at(320.0), &theme);
    let body = one.body.clone().unwrap();
    assert_eq!(
        body.rect.y,
        one.name.clone().unwrap().rect.bottom() + s.body_gap
    );
    let content_bottom = body.rect.bottom().max(one.avatar.bottom());
    assert!((one.row.bottom() - (content_bottom + s.pad)).abs() < 1e-3);

    let long = thread(LONG).layout(at(320.0), &theme);
    assert!(long.row.h > one.row.h, "a longer body grows the row");
    assert_eq!(long.row.w, 320.0, "the row takes the width it is given");
}

#[test]
fn a_narrower_row_wraps_into_more_lines_never_fewer() {
    let theme = core();
    for m in [thread(LONG), thread(SCRIPTS), quick(LONG)] {
        let mut prev = 0.0;
        for w in (200..=1600).rev().step_by(8) {
            let h = m.height(w as f32, &theme);
            assert!(h.is_finite() && h >= prev, "{w}: {h} after {prev}");
            prev = h;
        }
    }
}

#[test]
fn every_part_stays_in_the_row_over_the_whole_width_range() {
    let theme = core();
    let messages = [
        thread(LUNA).divider(true).verified(true),
        thread(LONG).verified(true),
        thread(SCRIPTS),
        Message::new("سارة الأحمد", "3m", SCRIPTS),
        Message::new(
            "Maria Eduarda Albuquerque de Souza Vasconcelos",
            "12 min",
            LUNA,
        ),
        quick(LONG),
        quick(SCRIPTS).reaction_disabled(true),
    ];
    for m in &messages {
        for w in (m.min_width(&theme) as usize..=1600).step_by(8) {
            let l = m.layout(at(w as f32), &theme);
            for (part, r) in parts(&l) {
                assert!(finite(r), "{}@{w}: {part} {r:?}", m.author());
                assert!(
                    within(r, l.row),
                    "{}@{w}: {part} {r:?} out of {:?}",
                    m.author(),
                    l.row
                );
            }
            if let (Some(n), Some(t)) = (&l.name, &l.time) {
                assert!(
                    n.rect.right() <= t.rect.x,
                    "{}@{w}: name over time",
                    m.author()
                );
            }
            if let (Some(n), Some(seal)) = (&l.name, l.seal) {
                assert!(
                    n.rect.right() <= seal.x,
                    "{}@{w}: name over seal",
                    m.author()
                );
                if let Some(t) = &l.time {
                    assert!(
                        seal.right() <= t.rect.x,
                        "{}@{w}: seal over time",
                        m.author()
                    );
                }
            }
        }
    }
}

#[test]
fn a_long_name_is_ellipsized_and_the_time_stays_whole() {
    let theme = core();
    let m = Message::new(
        "Maria Eduarda Albuquerque de Souza Vasconcelos Starfield",
        "12 min ago",
        LUNA,
    );
    let l = m.layout(at(300.0), &theme);
    let (name, time) = (l.name.unwrap(), l.time.unwrap());
    assert!(name.text.ends_with('\u{2026}'), "{:?}", name.text);
    assert_eq!(time.text, "12 min ago");
    let col_right = l.row.right() - theme.message.thread.pad;
    assert!(
        time.rect.right() <= col_right + 0.5,
        "the time fits the column"
    );
    assert_eq!(
        time.rect.x,
        name.rect.right() + theme.message.thread.time_gap
    );
}

#[test]
fn a_row_too_narrow_for_its_column_draws_no_text_and_stays_finite() {
    let theme = core();
    for w in [0.0, 1.0, 40.0, 80.0] {
        let l = thread(LONG).layout(at(w), &theme);
        assert_eq!((l.name.is_none(), l.body.is_none()), (true, true), "{w}");
        for (part, r) in parts(&l) {
            assert!(finite(r), "{w}: {part} {r:?}");
        }
    }
}

#[test]
fn the_quick_body_is_clamped_to_three_lines_and_the_thread_body_is_not() {
    let theme = core();
    let q = quick(LONG).layout(at(320.0), &theme);
    let body = q.body.unwrap();
    let lh = theme.message.quick.body.line_height;
    assert!(body.text.ends_with('\u{2026}'), "{:?}", body.text);
    assert_eq!((body.rect.h / lh).round(), 3.0);
    let chip = q.reaction.unwrap();
    assert_eq!(
        chip.y,
        body.rect.bottom() + theme.message.quick.reaction_gap
    );

    let t = thread(LONG).layout(at(320.0), &theme).body.unwrap();
    assert_eq!(t.text, LONG);
    assert!(t.rect.h / theme.message.thread.body.line_height > 3.5);
}

#[test]
fn the_hairline_closes_the_row_only_when_asked() {
    let theme = core();
    let l = thread(LUNA).divider(true).layout(at(320.0), &theme);
    let d = l.divider.unwrap();
    assert_eq!(
        (d.x, d.w, d.h),
        (l.row.x, l.row.w, theme.message.divider_width)
    );
    assert!(
        (d.bottom() - l.row.bottom()).abs() < 1e-3,
        "the hairline closes the row"
    );
    assert!(thread(LUNA).layout(at(320.0), &theme).divider.is_none());
}

// events

#[test]
fn the_like_chip_takes_hover_and_click_at_its_rect_only() {
    let theme = core();
    let bounds = at(320.0);
    let mut m = quick(LUNA);
    let chip = m.layout(bounds, &theme).reaction.unwrap();
    let (x, y) = chip.center();
    assert!(m.handle_event(&move_to(x, y), bounds, &theme).changed);
    assert!(m.handle_event(&down(x, y), bounds, &theme).changed);
    assert!(m.handle_event(&up(x, y), bounds, &theme).clicked);
    let (ax, ay) = m.layout(bounds, &theme).avatar.center();
    assert!(!m.handle_event(&down(ax, ay), bounds, &theme).clicked);

    let mut off = quick(LUNA).reaction_disabled(true);
    off.handle_event(&down(x, y), bounds, &theme);
    assert!(
        !off.handle_event(&up(x, y), bounds, &theme).clicked,
        "disabled"
    );

    let mut plain = thread(LUNA);
    assert_eq!(
        plain.handle_event(&down(x, y), bounds, &theme),
        EventResult::IGNORED
    );
}

// render

#[test]
fn the_render_draws_each_run_with_its_text_and_binds_the_row_and_the_chip() {
    let theme = core();
    let bounds = at(320.0);
    let row = id("msg");
    let m = quick(LUNA).sem(row.clone());
    let (c, drawn) = nodes(&m, bounds, &theme);
    let words = texts(&drawn);
    for want in ["Randomdash", "1h", LUNA, "2"] {
        assert!(words.iter().any(|t| t == want), "{want} in {words:?}");
    }
    let l = m.layout(bounds, &theme);
    let bound = c.binding(&row).unwrap();
    assert_eq!(bound.rect, [l.row.x, l.row.y, l.row.w, l.row.h]);
    let chip = c.binding(&id("msg.reaction")).unwrap();
    let r = l.reaction.unwrap();
    assert_eq!(chip.rect, [r.x, r.y, r.w, r.h]);
}

#[test]
fn the_focus_ring_shows_only_while_focused() {
    let theme = core();
    let mut m = thread(LUNA);
    let before = nodes(&m, at(320.0), &theme).1.len();
    m.set_focused(true);
    assert!(m.is_focused());
    let after = nodes(&m, at(320.0), &theme).1.len();
    assert_eq!(after, before + 1, "one ring");
}

#[test]
fn every_theme_renders_a_message_without_a_non_finite_node() {
    for name in ["hoff", "core", "plev", "nord", "gruvbox"] {
        let theme = Theme::named(name).unwrap();
        for m in [thread(SCRIPTS).divider(true), quick(LONG)] {
            for w in [280.0, 390.0, 1440.0] {
                let l = m.layout(at(w), &theme);
                for (part, r) in parts(&l) {
                    assert!(finite(r), "{name}@{w}: {part}");
                }
                assert!(!nodes(&m, at(w), &theme).1.is_empty());
            }
        }
    }
}

// projection

fn snap(project: impl FnOnce(&mut SnapshotBuilder)) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    project(&mut b);
    b.finish().unwrap()
}

#[test]
fn a_message_projects_a_focusable_item_read_whole_and_its_chip() {
    let row = id("msg");
    let m = quick(LONG).sem(row.clone());
    let s = snap(|b| {
        m.project(b, &row, None);
    });
    let node = s.get(&row).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::ListItem, "Randomdash, 1h")
    );
    assert_eq!(
        node.description.as_deref(),
        Some(LONG),
        "the clamped body is read whole"
    );
    assert!(node.focusable);
    let chip = s.get(&id("msg.reaction")).unwrap();
    assert_eq!(
        (chip.role, chip.parent.as_ref()),
        (Role::Button, Some(&row))
    );
    assert_eq!(chip.description.as_deref(), Some("like"));
    assert!(chip.actions.iter().any(|a| a.name.as_str() == "activate"));

    let bare = Message::new("Sam", "", "");
    let s = snap(|b| {
        bare.project(b, &row, None);
    });
    let node = s.get(&row).unwrap();
    assert_eq!(
        (node.label.as_str(), node.description.as_deref()),
        ("Sam", None)
    );
    assert_eq!(s.children(&row).count(), 0, "no chip, no child");
}

// bubble

use crate::widgets::message::bubble::{Bubble, BubbleSide};

fn row(w: f32) -> Rect {
    Rect::new(10.0, 20.0, w, 0.0)
}

#[test]
fn a_bubble_grows_with_its_text_up_to_its_share_of_the_row_then_wraps() {
    let theme = core();
    let t = &theme.message.bubble;
    let short = Bubble::new("ok").size(600.0, &theme);
    let line = t.text.line_height + (t.pad_y + t.border_width) * 2.0;
    assert_eq!(short.1, line, "one line");
    let long = Bubble::new(LONG).size(600.0, &theme);
    assert!(long.0 <= 600.0 * t.max_fraction + 0.5, "{long:?}");
    assert!(long.1 > line * 2.0, "it wraps: {long:?}");
    let empty = Bubble::new("").size(600.0, &theme);
    assert_eq!(empty.1, line, "an empty bubble keeps one line");
}

#[test]
fn a_bubble_sits_at_its_side_and_stays_in_its_row_over_the_whole_range() {
    let theme = core();
    for side in [BubbleSide::Start, BubbleSide::End, BubbleSide::Center] {
        for text in ["ok", LUNA, LONG, SCRIPTS] {
            let b = Bubble::new(text).side(side);
            for w in (40..=1600).step_by(8) {
                let r = b.rect(row(w as f32), &theme);
                assert!(finite(r), "{side:?}@{w}");
                assert!(
                    r.x >= 10.0 - 0.5 && r.right() <= 10.0 + w as f32 + 0.5,
                    "{side:?}@{w}: {r:?}"
                );
            }
            let r = b.rect(row(600.0), &theme);
            match side {
                BubbleSide::Start => assert_eq!(r.x, 10.0),
                BubbleSide::End => assert!((r.right() - 610.0).abs() < 1e-3),
                BubbleSide::Center => assert!((r.center().0 - 310.0).abs() < 1e-3),
            }
        }
    }
}

#[test]
fn the_tail_corner_is_square_on_the_speaker_s_side_and_the_border_tells_unread() {
    let theme = core();
    let t = theme.message.bubble.clone();
    let patches = |b: &Bubble| {
        nodes_of(b, &theme)
            .into_iter()
            .filter_map(|n| match n {
                SceneNode::Rect { x, color, .. } => Some((x, color)),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let start = Bubble::new("hi");
    let r = start.rect(row(600.0), &theme);
    let p = patches(&start);
    assert_eq!(p.len(), 2, "border and fill patches");
    assert!(
        p.iter().all(|(x, _)| *x < r.center().0),
        "tail at the start"
    );
    assert_eq!(p[0].1, t.border.0, "read: the hairline");
    let end = Bubble::new("hi").side(BubbleSide::End).unread(true);
    let r = end.rect(row(600.0), &theme);
    let p = patches(&end);
    assert!(p.iter().all(|(x, _)| *x > r.center().0), "tail at the end");
    assert_eq!(p[0].1, t.unread.0, "unread: the accent");
    let hue = crate::styling::color::Color::hex(0x5C8C80);
    assert_eq!(patches(&Bubble::new("hi").accent(hue))[0].1, hue.0);
    assert!(
        patches(&Bubble::new("hi").side(BubbleSide::Center)).is_empty(),
        "a note has no tail"
    );
}

fn nodes_of(b: &Bubble, theme: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    b.render(&mut c, row(600.0), theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

#[test]
fn a_bubble_draws_its_text_inside_and_binds_and_projects_itself() {
    let theme = core();
    let id = id("bubble");
    let b = Bubble::new(SCRIPTS).unread(true).sem(id.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    b.render(&mut c, row(400.0), &theme);
    let r = b.rect(row(400.0), &theme);
    assert_eq!(c.binding(&id).unwrap().rect, [r.x, r.y, r.w, r.h]);
    let drawn = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    let text = drawn.iter().find_map(|n| match n {
        SceneNode::Text { key, x, y, .. } => Some((key.clone(), *x, *y)),
        _ => None,
    });
    let (key, x, y) = text.expect("the text is drawn");
    assert_eq!(key.text, SCRIPTS);
    assert!(x > r.x && y > r.y);
    let wrap = f32::from_bits(key.max_width_bits.unwrap());
    assert!(
        x + wrap <= r.right() + 1.0,
        "the wrap box stays in the bubble"
    );

    let s = snap(|sb| {
        b.project(sb, &id, None);
    });
    let node = s.get(&id).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::ListItem, SCRIPTS));
    assert_eq!(node.state.get("unread"), Some(&accord::Value::Bool(true)));
    assert!(node.focusable);
}

#[test]
fn every_theme_draws_a_bubble_with_an_opaque_fill() {
    for name in ["hoff", "core", "plev", "nord", "gruvbox"] {
        let theme = Theme::named(name).unwrap();
        assert_eq!(theme.message.bubble.fill.0[3], 1.0, "{name}");
        for w in [280.0, 390.0, 1440.0] {
            let r = Bubble::new(LONG).rect(row(w), &theme);
            assert!(finite(r), "{name}@{w}");
        }
    }
}

#[test]
fn a_verified_author_s_seal_follows_the_name_and_keeps_its_place_on_a_narrow_row() {
    let theme = core();
    let s = theme.message.thread;
    let m = Message::new("Luna Starfield", "2s", LUNA).verified(true);
    let l = m.layout(at(320.0), &theme);
    let (name, seal, time) = (l.name.unwrap(), l.seal.unwrap(), l.time.unwrap());
    assert_eq!(seal.x, name.rect.right() + s.seal_gap);
    assert_eq!(seal.h, s.name.line_height, "one name line tall");
    assert_eq!(time.rect.x, seal.right() + s.time_gap);
    let (_, drawn) = nodes(&m, at(320.0), &theme);
    assert!(
        drawn.iter().any(|n| matches!(n, SceneNode::Path { .. })),
        "the icon is drawn"
    );

    let long = Message::new(
        "Maria Eduarda Albuquerque de Souza Vasconcelos",
        "12 min ago",
        LUNA,
    )
    .verified(true);
    let l = long.layout(at(280.0), &theme);
    assert!(
        l.name.unwrap().text.ends_with('\u{2026}'),
        "the name gives way first"
    );
    assert!(l.seal.is_some() && l.time.unwrap().text == "12 min ago");

    let row = id("msg");
    let snap = snap(|b| {
        m.project(b, &row, None);
    });
    assert_eq!(
        snap.get(&row).unwrap().state.get("verified"),
        Some(&accord::Value::Bool(true))
    );
}

// scroller

use crate::widgets::message::scroller::MessageScroller;

fn thread_of(n: usize) -> MessageScroller {
    let messages = (0..n)
        .map(|i| Message::new(format!("Person {i}"), format!("{i}m"), LUNA).divider(true))
        .collect();
    MessageScroller::new("Comments", "No comments yet", messages).sem(id("thread"))
}

fn surface(w: f32, h: f32) -> Rect {
    Rect::new(10.0, 20.0, w, h)
}

#[test]
fn a_scroller_is_as_tall_as_its_messages_up_to_its_cap() {
    let theme = core();
    let cap = theme.message.scroller.max_h;
    let two = thread_of(2);
    let h = two.height(320.0, &theme);
    assert!(h < cap, "two messages fit: {h}");
    let b = theme.message.scroller.border_width;
    assert!((h - (two.content_h(320.0, &theme) + b * 2.0)).abs() < 1e-3);
    assert_eq!(
        thread_of(40).height(320.0, &theme),
        cap,
        "a long thread stops at the cap"
    );
    let empty = thread_of(0);
    assert!(
        empty.height(320.0, &theme) > b * 2.0,
        "an empty thread keeps its line"
    );
}

#[test]
fn the_wheel_scrolls_the_thread_until_its_end_then_leaves_it_to_the_page() {
    let theme = core();
    let mut s = thread_of(12);
    let bounds = surface(320.0, s.height(320.0, &theme));
    let (x, y) = bounds.center();
    let r = s.handle_event(&wheel(x, y, 120.0), bounds, &theme);
    assert!(r.handled && r.changed && s.offset() > 0.0);
    for _ in 0..200 {
        s.handle_event(&wheel(x, y, 120.0), bounds, &theme);
    }
    let end = s.offset();
    let r = s.handle_event(&wheel(x, y, 120.0), bounds, &theme);
    assert!(!r.handled, "at the end the page takes the wheel");
    assert_eq!(s.offset(), end);
    let r = s.handle_event(&wheel(x, bounds.bottom() + 50.0, -120.0), bounds, &theme);
    assert!(
        !r.handled,
        "a wheel outside the surface is not the thread's"
    );
    let rows = s.rows(bounds, &theme);
    let last = rows.last().unwrap();
    let inner = bounds.inset(theme.message.scroller.border_width);
    assert!(
        (last.bottom() - inner.bottom()).abs() < 0.5,
        "the end shows the last message"
    );
}

#[test]
fn reveal_scrolls_the_least_that_shows_a_message_whole() {
    let theme = core();
    let mut s = thread_of(12);
    let bounds = surface(320.0, s.height(320.0, &theme));
    let inner = bounds.inset(theme.message.scroller.border_width);
    assert!(!s.reveal(0, bounds, &theme), "the first is shown");
    assert!(s.reveal(7, bounds, &theme));
    let r = s.rows(bounds, &theme)[7];
    assert!(
        r.y >= inner.y - 0.5 && r.bottom() <= inner.bottom() + 0.5,
        "{r:?} in {inner:?}"
    );
    assert!(
        (r.bottom() - inner.bottom()).abs() < 0.5,
        "the least: it lands at the bottom"
    );
    assert!(s.reveal(0, bounds, &theme));
    assert_eq!(s.offset(), 0.0);
    assert!(!s.reveal(99, bounds, &theme), "no such message");
}

#[test]
fn a_scroller_draws_only_what_it_shows_clipped_and_projects_every_message() {
    let theme = core();
    let s = thread_of(12);
    let bounds = surface(320.0, s.height(320.0, &theme));
    let (c, drawn) = {
        let mut c = Compositor::new();
        c.begin_frame();
        s.render(&mut c, bounds, &theme);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
        (c, nodes)
    };
    assert!(
        drawn
            .iter()
            .any(|n| matches!(n, SceneNode::PushClip { .. }))
    );
    let shown = texts(&drawn)
        .iter()
        .filter(|t| t.starts_with("Person "))
        .count();
    assert!(
        shown > 0 && shown < 12,
        "only the messages in view: {shown}"
    );
    assert!(c.binding(&id("thread[0]")).is_some());
    assert!(
        c.binding(&id("thread[11]")).is_none(),
        "a message far below is not drawn"
    );

    let snap = snap(|b| {
        s.project(b, &id("thread"), None);
    });
    let list = snap.get(&id("thread")).unwrap();
    assert_eq!((list.role, list.label.as_str()), (Role::List, "Comments"));
    assert_eq!(list.state.get("count"), Some(&accord::Value::Int(12)));
    assert_eq!(
        snap.children(&id("thread")).count(),
        12,
        "every message, shown or not"
    );

    let empty = thread_of(0);
    let mut c = Compositor::new();
    c.begin_frame();
    empty.render(&mut c, surface(320.0, empty.height(320.0, &theme)), &theme);
    let words = texts(c.layer(LayerId::DEFAULT).unwrap().nodes());
    assert_eq!(words, ["No comments yet"]);
}

#[test]
fn a_scroller_stays_whole_over_the_width_range_under_every_theme() {
    for name in ["core", "hoff", "plev", "nord"] {
        let theme = Theme::named(name).unwrap();
        for n in [0, 1, 12] {
            let s = thread_of(n);
            for w in (120..=1600).step_by(8) {
                let h = s.height(w as f32, &theme);
                assert!(
                    h.is_finite() && h > 0.0 && h <= theme.message.scroller.max_h + 0.5,
                    "{name} {n}@{w}: {h}"
                );
            }
        }
    }
}

#[test]
fn the_scroller_s_border_closes_the_last_message_without_its_hairline() {
    let theme = core();
    let s = thread_of(3);
    let rows = s.rows(surface(320.0, 400.0), &theme);
    let laid: Vec<_> = s
        .messages()
        .iter()
        .zip(&rows)
        .map(|(m, r)| m.layout(*r, &theme).divider.is_some())
        .collect();
    assert_eq!(laid, [true, true, false]);
}

// alert

use crate::widgets::buttons::ButtonVariant;
use crate::widgets::feedback::alert::{Alert, AlertLayout, NotificationKind};

/// The kit's invite: an avatar, unread, its two answers.
fn invite() -> Alert {
    Alert::new(
        Intent::Constructive,
        "3D object is generated",
        "Minimalist Architecture Scene",
    )
    .time("4h ago")
    .avatar("Hoff")
    .unread(true)
    .action("Decline", ButtonVariant::Primary)
    .action("Accept", ButtonVariant::Secondary)
}

/// The kit's comment: the content clamped under the detail.
fn comment(content: &str) -> Alert {
    Alert::new(
        Intent::Neutral,
        "Randomdash",
        "Commented on Classic Car in Studio",
    )
    .time("1h ago")
    .avatar("Randomdash")
    .content(content)
}

fn alert_parts(l: &AlertLayout) -> Vec<(&'static str, Rect)> {
    let mut out = vec![("lead", l.lead)];
    out.extend(l.badge.map(|r| ("badge", r)));
    out.extend(l.divider.map(|r| ("divider", r)));
    out.extend(l.dot.map(|r| ("dot", r)));
    for (name, p) in [
        ("title", &l.title),
        ("time", &l.time),
        ("detail", &l.detail),
        ("content", &l.content),
    ] {
        if let Some(p) = p {
            out.push((name, p.rect));
        }
    }
    out.extend(l.actions.iter().map(|r| ("action", *r)));
    out
}

fn alert_nodes(a: &Alert, bounds: Rect, theme: &Theme) -> (Compositor, Vec<SceneNode>) {
    let mut c = Compositor::new();
    c.begin_frame();
    a.render(&mut c, bounds, theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    (c, nodes)
}

#[test]
fn the_alert_carries_the_kit_in_core_and_derives_the_palette_elsewhere() {
    let kit = MessageTokens::hoff_core().alert;
    assert_eq!(
        (kit.pad, kit.pad_end, kit.lead, kit.gap),
        (20.0, 16.0, 48.0, 16.0)
    );
    assert_eq!(
        (kit.badge, kit.badge_ring, kit.badge_icon),
        (22.0, 2.0, 12.0)
    );
    assert_eq!(kit.content_lines, 2);
    assert_eq!(core().message.alert, kit);
    let hoff = Theme::hoff();
    let derived = MessageTokens::derive(&hoff).alert;
    let c = &hoff.colors;
    assert_eq!(derived.title.color, c.text);
    assert_eq!(derived.time.color, c.text_mid);
    assert_eq!(derived.unread.0[3], 1.0, "the dot is opaque");
    assert_eq!(derived.intents.of(Intent::Destructive), c.danger);
    assert_eq!(derived.ring.0[3], 1.0, "the ring is opaque");
    assert_eq!(derived.lead, kit.lead, "the geometry is the kit's");
}

#[test]
fn a_short_alert_is_as_tall_as_its_lead_with_the_text_centered_against_it() {
    let theme = core();
    let t = &theme.message.alert;
    let a = Alert::new(Intent::Informational, "Saved", "Your scene is in the cloud");
    let l = a.layout(at(386.0), &theme);
    assert_eq!(l.row.h, t.pad * 2.0 + t.lead, "the lead sets the height");
    let title = l.title.as_ref().unwrap().rect;
    let detail = l.detail.as_ref().unwrap().rect;
    let mid = (title.y + t.title_lift + detail.bottom()) / 2.0;
    assert!((mid - l.lead.center().1).abs() < 0.5, "centered: {mid}");
    assert!(l.badge.is_none(), "without an avatar the lead is the mark");
    assert!(l.dot.is_none() && l.actions.is_empty());
}

#[test]
fn the_content_is_clamped_to_two_lines_and_the_answers_add_their_row() {
    let theme = core();
    let t = &theme.message.alert;
    let long = comment(LONG).layout(at(386.0), &theme);
    let content = long.content.as_ref().unwrap();
    let line = t.content.style(&theme.typography).line_height;
    assert!(content.rect.h <= line * 2.0 + 0.5, "{:?}", content.rect);
    assert!(content.text.len() < LONG.len(), "cut");
    assert!(long.row.h > t.pad * 2.0 + t.lead);

    let l = invite().layout(at(386.0), &theme);
    assert_eq!(l.actions.len(), 2);
    assert_eq!(l.actions[0].y, l.actions[1].y, "side by side");
    let detail = l.detail.as_ref().unwrap().rect;
    assert!((l.actions[0].y - detail.bottom() - t.actions_gap).abs() < 0.5);
    assert!((l.row.bottom() - l.actions[0].bottom() - t.pad).abs() < 0.5);
    assert_eq!(l.badge.unwrap().right(), l.lead.right() + t.badge_out);
    let dot = l.dot.unwrap();
    assert!((dot.right() - (l.row.right() - t.pad_end)).abs() < 0.5);
}

#[test]
fn a_long_title_is_ellipsized_the_time_kept_and_the_detail_is_one_line() {
    let theme = core();
    let a = Alert::new(Intent::Destructive, LONG, LONG).time("12 min ago");
    let l = a.layout(at(320.0), &theme);
    let title = l.title.as_ref().unwrap();
    assert!(title.text.ends_with('\u{2026}'), "{}", title.text);
    assert_eq!(l.time.as_ref().unwrap().text, "12 min ago");
    let detail = l.detail.as_ref().unwrap();
    assert!(detail.text.ends_with('\u{2026}'));
    let line = theme
        .message
        .alert
        .detail
        .style(&theme.typography)
        .line_height;
    assert_eq!(detail.rect.h, line);
}

#[test]
fn every_part_of_an_alert_stays_in_its_row_over_the_whole_range_and_theme() {
    for name in ["hoff", "core", "plev", "nord", "gruvbox"] {
        let theme = Theme::named(name).unwrap();
        for a in [invite(), comment(SCRIPTS), comment(LONG).divider(true)] {
            for w in (160..=1600).step_by(8) {
                let bounds = at(w as f32);
                let l = a.layout(bounds, &theme);
                assert_eq!(l.row.w, bounds.w);
                for (part, r) in alert_parts(&l) {
                    assert!(finite(r), "{name}@{w}: {part}");
                    if w as f32 >= a.min_width(&theme) {
                        assert!(
                            within(r, l.row),
                            "{name}@{w}: {part} {r:?} out of {:?}",
                            l.row
                        );
                    }
                }
                if let Some(t) = &l.title {
                    let time = l.time.as_ref().map_or(t.rect.right(), |p| p.rect.right());
                    assert!(time <= l.dot.map_or(f32::MAX, |d| d.x) + 0.5, "{name}@{w}");
                }
            }
        }
    }
}

#[test]
fn the_answers_wrap_when_the_column_is_narrow() {
    let theme = core();
    let a = invite();
    let narrow = a.layout(at(a.min_width(&theme)), &theme);
    assert!(narrow.actions[1].y > narrow.actions[0].y, "wrapped");
    let wide = a.layout(at(600.0), &theme);
    assert!(narrow.row.h > wide.row.h);
}

#[test]
fn a_click_on_an_answer_chooses_it_and_a_click_on_the_row_reads_the_alert() {
    let theme = core();
    let bounds = at(386.0);
    let mut a = invite();
    let accept = a.layout(bounds, &theme).actions[1];
    let (x, y) = accept.center();
    assert!(a.handle_event(&move_to(x, y), bounds, &theme).changed);
    a.handle_event(&down(x, y), bounds, &theme);
    assert!(a.handle_event(&up(x, y), bounds, &theme).clicked);
    assert_eq!(a.chosen(), Some(1));
    assert!(!a.is_unread(), "an answer reads it");

    let mut b = invite();
    let (lx, ly) = b.layout(bounds, &theme).lead.center();
    b.handle_event(&down(lx, ly), bounds, &theme);
    assert!(b.handle_event(&up(lx, ly), bounds, &theme).clicked);
    assert!(!b.is_unread());
    assert_eq!(b.chosen(), None, "the row answers nothing");

    // pressed on an answer, released on the row: nothing is chosen or read
    let mut c = invite();
    c.handle_event(&down(x, y), bounds, &theme);
    c.handle_event(&up(lx, ly), bounds, &theme);
    assert!(c.is_unread() && c.chosen().is_none());
    assert!(!c.choose(2), "no third answer");
    assert!(c.open() && !c.open(), "read once");
}

#[test]
fn the_alert_draws_its_runs_binds_its_parts_and_rings_only_while_focused() {
    let theme = core();
    let bounds = at(386.0);
    let mut a = invite().sem(id("note"));
    let (c, drawn) = alert_nodes(&a, bounds, &theme);
    let words = texts(&drawn);
    for want in [
        "3D object is generated",
        "4h ago",
        "Minimalist Architecture Scene",
        "Accept",
    ] {
        assert!(words.iter().any(|t| t == want), "{want} in {words:?}");
    }
    let l = a.layout(bounds, &theme);
    assert_eq!(
        c.binding(&id("note")).unwrap().rect,
        [l.row.x, l.row.y, l.row.w, l.row.h]
    );
    let r = l.actions[0];
    assert_eq!(
        c.binding(&id("note.action[0]")).unwrap().rect,
        [r.x, r.y, r.w, r.h]
    );
    let before = drawn.len();
    a.set_focused(true);
    assert_eq!(
        alert_nodes(&a, bounds, &theme).1.len(),
        before + 1,
        "one ring"
    );
    a.set_focused(false);
    a.open();
    assert_eq!(
        alert_nodes(&a, bounds, &theme).1.len(),
        before - 1,
        "the dot goes"
    );
}

#[test]
fn an_alert_projects_its_intent_its_words_whole_and_its_answers() {
    let row = id("note");
    let mut a = comment(LONG)
        .action("Reply", ButtonVariant::Secondary)
        .sem(row.clone())
        .unread(true);
    let s = snap(|b| {
        a.project(b, &row, None);
    });
    let node = s.get(&row).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::Status, "Randomdash, 1h ago")
    );
    let description = node.description.as_deref().unwrap();
    assert!(description.ends_with(LONG), "read whole: {description}");
    assert_eq!(node.state.get("unread"), Some(&accord::Value::Bool(true)));
    assert!(node.focusable && node.actions.iter().any(|x| x.name.as_str() == "activate"));
    let reply = s.get(&id("note.action[0]")).unwrap();
    assert_eq!(
        (reply.role, reply.parent.as_ref()),
        (Role::Button, Some(&row))
    );

    assert!(a.choose(0));
    let s = snap(|b| {
        a.project(b, &row, None);
    });
    let reply = s.get(&id("note.action[0]")).unwrap();
    assert_eq!(
        reply.state.get("selected"),
        Some(&accord::Value::Bool(true))
    );
    assert_eq!(
        s.get(&row).unwrap().state.get("unread"),
        Some(&accord::Value::Bool(false))
    );

    let danger = Alert::new(Intent::Destructive, "Export failed", "").sem(row.clone());
    let s = snap(|b| {
        danger.project(b, &row, None);
    });
    let node = s.get(&row).unwrap();
    assert_eq!(
        (node.role, node.description.as_deref()),
        (Role::Alert, None)
    );
}

// attachment

use crate::widgets::message::attachment::{AttachmentKind, AttachmentTray, TrayFocus, TrayLayout};

const PDF: &str = "Minimalist Architecture Scene, final review.pdf";

fn tray() -> AttachmentTray {
    AttachmentTray::new()
        .with("Uploaded 1", "PNG, 1.2 MB", AttachmentKind::Image(None))
        .with("Uploaded 2", "PNG, 840 KB", AttachmentKind::Image(None))
        .with(PDF, "PDF, 3.4 MB", AttachmentKind::File)
        .with(
            "Classic Car.glb",
            "3D object, 12 MB",
            AttachmentKind::Object,
        )
        .with_add()
}

fn tray_parts(l: &TrayLayout) -> Vec<(&'static str, Rect)> {
    let mut out: Vec<_> = l.add.map(|r| ("add", r)).into_iter().collect();
    for t in &l.tiles {
        out.push(("tile", t.rect));
        out.push(("remove", t.remove));
        out.extend(t.icon.map(|r| ("icon", r)));
        for (name, p) in [("name", &t.name), ("detail", &t.detail)] {
            if let Some(p) = p {
                out.push((name, p.rect));
            }
        }
    }
    out
}

fn tray_nodes(t: &AttachmentTray, bounds: Rect, theme: &Theme) -> (Compositor, Vec<SceneNode>) {
    let mut c = Compositor::new();
    c.begin_frame();
    t.render(&mut c, bounds, theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    (c, nodes)
}

#[test]
fn the_attachment_carries_the_kit_in_core_and_derives_elsewhere() {
    let kit = MessageTokens::hoff_core().attachment;
    assert_eq!((kit.tile, kit.radius, kit.gap), (60.0, 12.0, 8.0));
    assert_eq!(
        (kit.remove, kit.remove_out, kit.remove_icon),
        (20.0, 6.0, 16.0)
    );
    assert_eq!(core().message.attachment, kit);
    let hoff = Theme::hoff();
    let derived = MessageTokens::derive(&hoff).attachment;
    assert_eq!(derived.file_border, hoff.colors.divider);
    assert_eq!(derived.name.color, hoff.colors.text);
    assert_eq!(derived.file_fill.0[3], 1.0);
    assert_eq!(derived.tile, kit.tile);
}

#[test]
fn a_picture_is_one_tile_square_and_a_file_tile_grows_with_its_name() {
    let theme = core();
    let t = &theme.message.attachment;
    let short = AttachmentTray::new()
        .with("a", "", AttachmentKind::Image(None))
        .with("notes.txt", "TXT, 2 KB", AttachmentKind::File)
        .with(PDF, "PDF", AttachmentKind::File);
    let l = short.layout(at(1200.0), &theme);
    assert_eq!((l.tiles[0].rect.w, l.tiles[0].rect.h), (t.tile, t.tile));
    assert!(
        l.tiles[2].rect.w > l.tiles[1].rect.w,
        "a longer name, a wider tile"
    );
    assert!(l.tiles.iter().all(|x| x.rect.h == t.tile));
    // a tile as wide as its words keeps them whole wherever it lands
    let off = short.layout(Rect::new(296.3, 20.0, 1200.0, 0.0), &theme);
    for tile in &off.tiles[1..] {
        for p in [&tile.name, &tile.detail] {
            let p = p.as_ref().unwrap();
            assert!(!p.text.ends_with('\u{2026}'), "{} cut", p.text);
        }
    }
    // on a narrow tray the long name is cut inside its tile
    let narrow = short.layout(at(220.0), &theme);
    let name = narrow.tiles[2].name.as_ref().unwrap();
    assert!(name.text.ends_with('\u{2026}'), "{}", name.text);
    assert!(within(name.rect, narrow.tiles[2].rect));
}

#[test]
fn the_tray_wraps_and_holds_every_part_over_the_whole_range_and_theme() {
    for name in ["hoff", "core", "plev", "nord", "gruvbox"] {
        let theme = Theme::named(name).unwrap();
        let t = tray().show_removes(true);
        let mut lines = 0;
        for w in (80..=1600).step_by(8) {
            let bounds = at(w as f32);
            let l = t.layout(bounds, &theme);
            for (part, r) in tray_parts(&l) {
                assert!(finite(r), "{name}@{w}: {part}");
                if w as f32 >= t.min_width(&theme) {
                    assert!(
                        within(r, l.rect),
                        "{name}@{w}: {part} {r:?} out of {:?}",
                        l.rect
                    );
                }
            }
            let n = 1 + l
                .tiles
                .windows(2)
                .filter(|p| p[1].rect.y > p[0].rect.y)
                .count();
            if w == 80 {
                lines = n;
            }
            assert!(n <= lines, "{name}@{w}: wider, never more lines");
            assert_eq!(l.rect.h, t.height(w as f32, &theme));
        }
    }
}

#[test]
fn the_remove_button_shows_on_hover_focus_or_touch_and_takes_the_click_hidden() {
    let theme = core();
    let bounds = at(600.0);
    let mut t = tray();
    let first = t.items()[0].key();
    assert!(!t.remove_shown(first));
    let l = t.layout(bounds, &theme);
    let (tx, ty) = l.tiles[0].rect.center();
    assert!(t.handle_event(&move_to(tx, ty), bounds, &theme).changed);
    assert!(t.remove_shown(first), "hover shows it");
    t.handle_event(&move_to(-50.0, -50.0), bounds, &theme);
    assert!(!t.remove_shown(first));
    t.set_focus(Some(TrayFocus::Tile(first)));
    assert!(t.remove_shown(first), "a reader's focus shows it");
    t.set_focus(None);
    assert!(
        tray().show_removes(true).remove_shown(first),
        "touch shows it"
    );

    // a click on the hidden button removes the tile; the others keep
    // their keys
    let (rx, ry) = l.tiles[0].remove.center();
    t.handle_event(&down(rx, ry), bounds, &theme);
    assert!(t.handle_event(&up(rx, ry), bounds, &theme).clicked);
    assert_eq!(t.items().len(), 3);
    assert_eq!(t.items()[0].key(), first + 1);
    // pressed on a remove button, released elsewhere: nothing goes
    let l = t.layout(bounds, &theme);
    let (rx, ry) = l.tiles[0].remove.center();
    t.handle_event(&down(rx, ry), bounds, &theme);
    t.handle_event(&up(-50.0, -50.0), bounds, &theme);
    assert_eq!(t.items().len(), 3);
    assert!(!t.remove(99));
}

#[test]
fn the_add_button_asks_the_owner_once_per_click() {
    let theme = core();
    let bounds = at(600.0);
    let mut t = tray();
    let (x, y) = t.layout(bounds, &theme).add.unwrap().center();
    t.handle_event(&down(x, y), bounds, &theme);
    assert!(t.handle_event(&up(x, y), bounds, &theme).clicked);
    assert!(t.take_add_request());
    assert!(!t.take_add_request(), "taken once");
    let key = t.push("Uploaded 3", "PNG", AttachmentKind::Image(None));
    assert_eq!(key, 4, "a new key, never a reused one");
    let mut bare = AttachmentTray::new();
    assert!(
        !bare.request_add() && !bare.take_add_request(),
        "no add button"
    );
    assert!(bare.layout(bounds, &theme).add.is_none());
}

#[test]
fn the_tray_binds_projects_and_maps_its_ids() {
    let theme = core();
    let bounds = at(600.0);
    let root = id("files");
    let t = tray().sem(root.clone());
    let (c, drawn) = tray_nodes(&t, bounds, &theme);
    let l = t.layout(bounds, &theme);
    let r = l.tiles[2].rect;
    assert_eq!(
        c.binding(&id("files[2]")).unwrap().rect,
        [r.x, r.y, r.w, r.h]
    );
    let rm = l.tiles[0].remove;
    assert_eq!(
        c.hit(rm.x + rm.w / 2.0, rm.y + rm.h / 2.0),
        Some(&id("files[0].remove"))
    );
    assert!(c.binding(&id("files.add")).is_some());
    assert!(texts(&drawn).iter().any(|w| w == "PDF, 3.4 MB"));

    let s = snap(|b| {
        t.project(b, &root, "attachments", None);
    });
    assert_eq!(s.get(&root).unwrap().role, Role::List);
    let tile = s.get(&id("files[2]")).unwrap();
    assert_eq!((tile.role, tile.label.as_str()), (Role::ListItem, PDF));
    assert_eq!(tile.description.as_deref(), Some("PDF, 3.4 MB"));
    let rm = s.get(&id("files[2].remove")).unwrap();
    assert_eq!(
        (rm.role, rm.label.as_str()),
        (Role::Button, format!("Remove {PDF}").as_str())
    );
    assert!(s.get(&id("files.add")).is_some());

    assert_eq!(t.target(&id("files.add")), Some(TrayFocus::Add));
    assert_eq!(t.target(&id("files[1]")), Some(TrayFocus::Tile(1)));
    assert_eq!(t.target(&id("files[3].remove")), Some(TrayFocus::Remove(3)));
    assert_eq!(t.target(&id("files[9]")), None);
}

// marker

use crate::widgets::message::marker::{CursorState, Marker};

const STATES: [CursorState; 4] = [
    CursorState::Point,
    CursorState::Pinned,
    CursorState::Uploading,
    CursorState::Typing,
];

fn marker_nodes(m: &Marker, bounds: Rect, theme: &Theme) -> (Compositor, Vec<SceneNode>) {
    let mut c = Compositor::new();
    c.begin_frame();
    m.render(&mut c, bounds, theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    (c, nodes)
}

#[test]
fn the_marker_carries_the_kit_in_core_and_derives_elsewhere() {
    let kit = MessageTokens::hoff_core().marker;
    assert_eq!(
        (kit.avatar, kit.avatar_ring, kit.avatar_overlap),
        (24.0, 1.5, 10.0)
    );
    assert_eq!((kit.point.w, kit.point.h), (20.0, 18.0));
    assert_eq!(
        (kit.pinned.w, kit.pinned.h, kit.pinned.tail_radius),
        (30.0, 28.0, 2.0)
    );
    assert_eq!(core().message.marker, kit);
    let hoff = Theme::hoff();
    let derived = MessageTokens::derive(&hoff).marker;
    // the theme's colors composed over its paper: opaque, so a shape and
    // the patch over its tail corner do not show their overlap
    for c in [derived.pin_unread, derived.point_border, derived.pin_fill]
        .into_iter()
        .chain(derived.active.fill)
        .chain(derived.pinned.fill)
    {
        assert_eq!(c.0[3], 1.0, "{c:?}");
    }
    let info = hoff.colors.info.0;
    let paper = hoff.colors.bg.0;
    let want = info[1] * info[3] + paper[1] * (1.0 - info[3]);
    assert!(
        (derived.pin_unread.0[1] - want).abs() < 1e-5,
        "the info accent over paper"
    );
    // the light theme's info accent is under 3:1 on its paper: the unread
    // mark takes its accent, a blue that reads
    let light = Theme::light();
    let lit = MessageTokens::derive(&light);
    assert_eq!(lit.marker.pin_unread, light.colors.accent);
    assert_eq!(lit.alert.unread, light.colors.accent);
    assert_eq!(derived.avatar, kit.avatar);
}

#[test]
fn a_pin_is_as_wide_as_its_avatars_overlap_and_holds_them() {
    let theme = core();
    let t = &theme.message.marker;
    let one = Marker::pin(["Luna"]);
    let two = Marker::pin(["Luna", "Sam"]);
    let (w1, h1) = one.size(&theme);
    let (w2, h2) = two.size(&theme);
    assert_eq!(h1, h2);
    let slot = t.avatar + t.avatar_ring * 2.0;
    assert!((w2 - w1 - (slot - t.avatar_overlap)).abs() < 1e-3);
    assert!((h1 - ((t.pin_pad + t.pin_border_width) * 2.0 + slot)).abs() < 1e-3);
    let l = two.layout(at(0.0), &theme);
    assert!(l.avatars[1].x > l.avatars[0].x && l.avatars[1].x < l.avatars[0].right());
    for a in &l.avatars {
        assert!(within(
            a.outset(t.avatar_ring),
            l.rect.inset(t.pin_pad + t.pin_border_width)
        ));
    }
    assert_eq!(
        Marker::pin(Vec::<String>::new()).size(&theme),
        one.size(&theme)
    );
}

#[test]
fn every_cursor_state_is_the_kit_s_box_and_draws_finite_under_every_theme() {
    for name in ["hoff", "core", "plev", "nord", "gruvbox"] {
        let theme = Theme::named(name).unwrap();
        for s in STATES {
            let m = Marker::cursor(s);
            let (w, h) = m.size(&theme);
            assert!(w > 0.0 && h > 0.0, "{name}: {s:?}");
            let (_, nodes) = marker_nodes(&m, at(0.0), &theme);
            assert!(nodes.len() >= 3, "{name}: {s:?} draws its box and mark");
        }
        let (_, nodes) = marker_nodes(&Marker::pin(["Luna", "Sam"]).unread(true), at(0.0), &theme);
        assert!(!nodes.is_empty());
    }
    let theme = core();
    assert_eq!(
        Marker::cursor(CursorState::Point).size(&theme),
        (20.0, 18.0)
    );
    assert_eq!(
        Marker::cursor(CursorState::Typing).size(&theme),
        (30.0, 28.0)
    );
}

#[test]
fn a_click_on_a_pin_opens_and_reads_it_and_a_cursor_takes_nothing() {
    let theme = core();
    let bounds = at(0.0);
    let mut pin = Marker::pin(["Luna"]).unread(true);
    let (x, y) = pin.layout(bounds, &theme).rect.center();
    assert!(pin.handle_event(&move_to(x, y), bounds, &theme).changed);
    assert!(pin.is_hovered());
    pin.handle_event(&down(x, y), bounds, &theme);
    assert!(pin.handle_event(&up(x, y), bounds, &theme).clicked);
    assert!(!pin.is_unread());
    assert!(!pin.open(), "read already");

    let mut cursor = Marker::cursor(CursorState::Pinned);
    assert_eq!(
        cursor.handle_event(&down(x, y), bounds, &theme),
        EventResult::IGNORED
    );
    cursor.set_focused(true);
    assert!(!cursor.is_focused(), "a cursor takes no focus");
    assert!(!cursor.open());
}

#[test]
fn a_pin_tells_unread_by_its_border_and_rings_while_focused() {
    let theme = core();
    let t = &theme.message.marker;
    let border = |m: &Marker| {
        marker_nodes(m, at(0.0), &theme)
            .1
            .iter()
            .find_map(|n| match n {
                SceneNode::RoundedRect { color, .. } => Some(*color),
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(border(&Marker::pin(["Luna"])), t.pin_read.0);
    assert_eq!(border(&Marker::pin(["Luna"]).unread(true)), t.pin_unread.0);
    let mut pin = Marker::pin(["Luna"]);
    let before = marker_nodes(&pin, at(0.0), &theme).1.len();
    pin.set_focused(true);
    assert_eq!(marker_nodes(&pin, at(0.0), &theme).1.len(), before + 1);
}

#[test]
fn a_marker_binds_and_projects_a_pin_as_a_button_and_a_cursor_as_an_image() {
    let theme = core();
    let pin_id = id("pin");
    let pin = Marker::pin(["Luna", "Sam"])
        .unread(true)
        .sem(pin_id.clone());
    let (c, _) = marker_nodes(&pin, at(0.0), &theme);
    let r = pin.layout(at(0.0), &theme).rect;
    assert_eq!(c.binding(&pin_id).unwrap().rect, [r.x, r.y, r.w, r.h]);
    let s = snap(|b| {
        pin.project(b, &pin_id, None);
    });
    let node = s.get(&pin_id).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::Button, "Comment by Luna, Sam")
    );
    assert_eq!(node.state.get("unread"), Some(&accord::Value::Bool(true)));
    assert!(node.actions.iter().any(|a| a.name.as_str() == "activate"));

    let cur_id = id("cursor");
    let cursor = Marker::cursor(CursorState::Typing).sem(cur_id.clone());
    let s = snap(|b| {
        cursor.project(b, &cur_id, None);
    });
    let node = s.get(&cur_id).unwrap();
    assert_eq!(
        (node.role, node.description.as_deref()),
        (Role::Image, Some("typing"))
    );
    assert!(node.actions.is_empty() && !node.focusable);
}

// ---------------------------------------------------------------------------
// Badge: the kit's chip, the count pill on its run, the notification dot
// ---------------------------------------------------------------------------

use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::hoff_core::{RED, SHADES};
use crate::widgets::buttons::{Badge, BadgeKind};

/// WCAG's ratio of two colors, each composed over black when translucent.
fn ratio(a: Color, b: Color) -> f32 {
    let lum = |c: Color| {
        let lin = |v: f32| {
            let v = v * c.0[3];
            if v <= 0.040_45 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.0[0]) + 0.7152 * lin(c.0[1]) + 0.0722 * lin(c.0[2])
    };
    let (la, lb) = (lum(a), lum(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// `c` composed over the opaque `under`, in linear light as the
/// renderer blends.
fn on(c: Color, under: Color) -> Color {
    let lin = |v: f32| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let gamma = |v: f32| {
        if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    };
    let a = c.0[3];
    let at = |i: usize| gamma(lin(c.0[i]) * a + lin(under.0[i]) * (1.0 - a));
    Color([at(0), at(1), at(2), 1.0])
}

/// What a badge draws in `bounds`.
fn drawn(b: &Badge, bounds: Rect, t: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    b.render(&mut c, bounds, t);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

const INTENTS: [Intent; 4] = [
    Intent::Neutral,
    Intent::Informational,
    Intent::Constructive,
    Intent::Destructive,
];

#[test]
fn the_badge_carries_the_kit_in_core() {
    let b = core().message.badge;
    let [paper, well, shade3, ..] = SHADES;
    assert_eq!((b.tag.pad_x, b.tag.pad_y, b.tag.radius), (12.0, 4.0, 12.0));
    assert_eq!(
        (b.tag.text.size, b.tag.text.line_height, b.tag.text.weight),
        (11.0, 16.0, 500)
    );
    assert_eq!((b.tag.fill, b.tag.raised_fill), (shade3, paper));
    // the outline shows on the paper and in the gray card's well, so the
    // chip does not vanish where its fill nearly matches what is under it
    for border in [b.tag.border, b.tag.raised_border] {
        assert!(ratio(border, paper) >= 3.0, "{border:?} on the paper");
        assert!(ratio(border, well) >= 3.0, "{border:?} in the well");
    }
    assert_eq!((b.dot, b.dot_inset), (4.0, 4.0));
    let near = |a: Color, b: Color| (0..4).all(|i| (a.0[i] - b.0[i]).abs() < 1e-4);
    assert!(
        near(b.dots.destructive, RED),
        "the kit's red reads on its paper"
    );
    let (w, h) = Badge::tag("Card").preferred_size(&core());
    assert_eq!(
        h,
        16.0 + (4.0 + 1.0) * 2.0,
        "py-1, a border and the 16 px line"
    );
    assert!(w > h);
}

/// Every theme: the chip's label at 7:1 on core and AA elsewhere, the
/// intents' discs and the dots at 3:1, a count's numeral at 7:1.
#[test]
fn every_badge_reads_on_what_it_sits() {
    let named = ["hoff", "hoff-light", "plev", "dracula", "nord", "gruvbox"];
    let themes = named
        .iter()
        .map(|n| (*n, Theme::named(n).expect("a theme")))
        .chain([
            ("core", core()),
            ("dark", Theme::dark()),
            ("light", Theme::light()),
        ]);
    for (name, t) in themes {
        let b = &t.message.badge;
        let paper = Color([t.colors.bg.0[0], t.colors.bg.0[1], t.colors.bg.0[2], 1.0]);
        let card = on(t.colors.surface, paper);
        let chip = on(b.tag.fill, card);
        let text = if name == "core" { 7.0 } else { 4.5 };
        assert!(
            ratio(b.tag.text.color, chip) >= text,
            "{name}: the chip's label"
        );
        for i in INTENTS {
            if i != Intent::Neutral {
                assert!(ratio(b.marks.of(i), chip) >= 3.0, "{name}: {i:?} disc");
            }
            assert!(ratio(b.dots.of(i), card) >= 3.0, "{name}: {i:?} dot");
            let fill = b.count_fill.of(i);
            assert!(ratio(b.count_ink.of(i), fill) >= 7.0, "{name}: {i:?} count");
        }
    }
}

#[test]
fn a_count_is_a_disc_for_one_digit_and_a_pill_for_more() {
    let t = core();
    let (w1, h1) = Badge::count("3").preferred_size(&t);
    let (w2, h2) = Badge::count("1200").preferred_size(&t);
    assert_eq!(w1, h1);
    assert_eq!(h1, h2);
    assert!(w2 > w1);
}

/// An intent puts its disc before the label: the chip widens by the disc
/// and its gap, and draws it; a neutral chip draws none.
#[test]
fn an_intent_adds_its_disc_before_the_label() {
    let t = core();
    let tag = &t.message.badge.tag;
    let plain = Badge::tag("staged");
    let lit = Badge::tag("staged").intent(Intent::Constructive);
    let (w0, _) = plain.preferred_size(&t);
    let (w1, _) = lit.preferred_size(&t);
    assert!((w1 - w0 - (tag.mark + tag.mark_gap)).abs() <= 1.0);
    let discs = |b: &Badge| {
        drawn(b, Rect::new(0.0, 0.0, 400.0, 40.0), &t)
            .iter()
            .filter(|n| matches!(n, SceneNode::RoundedRect { w, .. } if *w == tag.mark))
            .count()
    };
    assert_eq!((discs(&plain), discs(&lit)), (0, 1));
}

/// A label wider than its room is ellipsized inside it; the box never
/// leaves the bounds, and is never narrower than tall.
#[test]
fn a_long_label_ellipsizes_inside_its_bounds() {
    let t = core();
    let b = Badge::tag(LONG).intent(Intent::Destructive);
    for w in [40.0, 120.0, 280.0] {
        let bounds = Rect::new(0.0, 0.0, w, 40.0);
        let r = b.rect(bounds, &t);
        assert!(r.w <= w + 0.5, "{w}: {}", r.w);
        let shown = drawn(&b, bounds, &t)
            .into_iter()
            .find_map(|n| match n {
                SceneNode::Text { key, x, .. } => Some((key.text, x)),
                _ => None,
            })
            .expect("a label");
        assert!(shown.0.len() < LONG.len(), "{w}: ellipsized");
        let style = b.text_style(&t);
        let (tw, _) = TextMeasurer::measure_styled(&shown.0, &style, None);
        assert!(
            shown.1 + tw <= r.right() + 0.5,
            "{w}: the label stays in its box"
        );
    }
    let unbounded = b.rect(Rect::new(0.0, 0.0, 0.0, 0.0), &t);
    assert_eq!(
        unbounded.w,
        b.preferred_size(&t).0,
        "no width: the preferred size"
    );
}

#[test]
fn the_dot_sits_on_its_host_s_top_end_corner() {
    let t = core();
    let host = Rect::new(10.0, 20.0, 40.0, 40.0);
    let d = Badge::dot_on(host, &t);
    assert_eq!((d.right(), d.y, d.w), (46.0, 24.0, 4.0));
    let n = drawn(&Badge::dot("New"), d, &t);
    assert_eq!(n.len(), 1, "a disc, no text");
    assert_eq!(Badge::dot("New").kind, BadgeKind::Dot);
}

/// A reader hears a label with its words and intent, can land on it,
/// and finds no action.
#[test]
fn a_badge_projects_a_label_with_its_intent() {
    let root = SemId::parse("page").unwrap();
    let id = root.child("badge").unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Badge");
    Badge::count("3").project(&mut b, &id, Some(&root));
    let seen: Snapshot = b.finish().expect("the projection validates");
    let n = seen.get(&id).expect("the badge");
    assert_eq!((n.role, n.label.as_str()), (Role::Label, "3"));
    assert!(n.actions.is_empty());
    assert!(format!("{:?}", n.state).contains("destructive"));
}

// ---------------------------------------------------------------------------
// Avatar: the online dot
// ---------------------------------------------------------------------------

use crate::widgets::content::{Avatar, AvatarSize};

/// The dot sits on the disc's bottom end rim, its center on the circle,
/// and reads at 3:1 on the paper under every theme.
#[test]
fn the_online_dot_sits_on_the_bottom_end_rim_and_reads() {
    for t in [core(), Theme::hoff(), Theme::dark(), Theme::light()] {
        let disc = Rect::new(0.0, 0.0, 48.0, 48.0);
        let d = Avatar::online_rect(disc, &t);
        let (cx, cy) = d.center();
        let r = ((cx - 24.0).powi(2) + (cy - 24.0).powi(2)).sqrt();
        assert!((r - 24.0).abs() < 0.01, "on the rim: {r}");
        assert!(cx > 24.0 && cy > 24.0, "bottom end");
        let a = &t.message.avatar;
        assert!(ratio(a.online_fill, a.ring) >= 3.0);
    }
    assert_eq!(core().message.avatar.online, 8.0, "Notification:72 size-2");
    let mut c = Compositor::new();
    c.begin_frame();
    let t = core();
    let avatar = Avatar::new("Ann").size(AvatarSize::Md).online(true);
    avatar.render(&mut c, Rect::new(0.0, 0.0, 44.0, 44.0), &t);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    let fill = t.message.avatar.online_fill.0;
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::RoundedRect { color, .. } if *color == fill)),
        "the dot is drawn in its green"
    );
}

/// A like draws the kit's heart on its red, and only informs: the tone
/// and the glyph come from its kind, the role from its intent.
#[test]
fn a_like_is_red_with_a_heart_and_informs() {
    let t = core();
    let like = Alert::new(Intent::Informational, "Marina", "Liked Classic Car")
        .avatar("Marina")
        .kind(NotificationKind::Like);
    assert_eq!(NotificationKind::Like.glyph(), "heart");
    assert_eq!(NotificationKind::Like.tone(&t.message.alert.kinds), RED);
    let mut c = Compositor::new();
    c.begin_frame();
    like.render(&mut c, Rect::new(0.0, 0.0, 480.0, 120.0), &t);
    let drawn = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    assert!(
        drawn
            .iter()
            .any(|n| matches!(n, SceneNode::RoundedRect { color, .. } if *color == RED.0)),
        "the badge on the kit's red"
    );
    let root = SemId::parse("page").unwrap();
    let id = root.child("like").unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Alert");
    like.project(&mut b, &id, Some(&root));
    let seen = b.finish().expect("the projection validates");
    assert_eq!(seen.get(&id).unwrap().role, Role::Status, "not an alert");
}

/// The avatar's ladder, its disc without a photo and the kit's stack.
mod avatar;

/// The loading pieces: the skeleton, the progress rail, the spinner.
mod loading;

/// At rest the paper fades over the edge the thread runs past: below at
/// the start, above at the end, both between; none on a thread that fits.
#[test]
fn the_paper_fades_over_the_edge_the_thread_runs_past() {
    let theme = core();
    let fades = |s: &MessageScroller, b: Rect| {
        let mut c = Compositor::new();
        c.begin_frame();
        s.render(&mut c, b, &theme);
        let paper = theme.message.scroller.fill.0;
        c.layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .filter_map(|n| match n {
                SceneNode::GradientRect { y, h, color, .. } if color[..3] == paper[..3] => {
                    Some((*y, *h, color[3]))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let fits = thread_of(2);
    let b = surface(320.0, fits.height(320.0, &theme));
    assert_eq!(fits.cut_edges(b, &theme), (false, false));
    assert!(fades(&fits, b).is_empty());
    let mut s = thread_of(12);
    let b = surface(320.0, s.height(320.0, &theme));
    assert_eq!(s.cut_edges(b, &theme), (false, true), "at the first frame");
    let below = fades(&s, b);
    assert_eq!(below.len(), 1);
    assert_eq!(
        below[0].1, theme.message.scroller.edge,
        "two body lines deep"
    );
    let (x, y) = b.center();
    s.handle_event(&wheel(x, y, 120.0), b, &theme);
    assert_eq!(s.cut_edges(b, &theme), (true, true));
    for _ in 0..200 {
        s.handle_event(&wheel(x, y, 120.0), b, &theme);
    }
    assert_eq!(s.cut_edges(b, &theme), (true, false));
    let above = fades(&s, b);
    assert_eq!((above.len(), above[0].2), (1, 1.0), "opaque at the top");
}

/// The empty state's body, centered line by line.
mod empty;

/// The prompt input, its model menu and its line (wave 3).
mod prompt;

/// The comment popups: the thread, the new comment, the quick comment
/// (wave 3).
mod comments;

/// The notifications dropdown (wave 3).
mod notifications;
