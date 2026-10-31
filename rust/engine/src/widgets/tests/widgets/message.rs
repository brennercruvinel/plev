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
    out.extend(l.reaction.map(|r| ("reaction", r)));
    out.extend(l.divider.map(|r| ("divider", r)));
    out
}

// tokens

#[test]
fn the_core_theme_carries_the_kit_and_the_others_derive_it() {
    let kit = MessageTokens::hoff_core();
    assert_eq!(core().message, kit);
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
    assert_eq!(hoff.message, derived);
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
        thread(LUNA).divider(true),
        thread(LONG),
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
