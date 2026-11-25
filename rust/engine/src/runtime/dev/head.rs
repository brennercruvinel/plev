//! The panel's head: the strip of tabs with what stands beside them, and
//! the line of the frame's vital numbers under it.
//!
//! The strip is a tool's: the picker first (the pointer chooses a control
//! on the page), then one small tab a view, its icon and its name, the
//! one shown underlined; at the right, which frame the views show (the
//! live one, the slowest, or one chosen in the history), the dock to move
//! the panel to, and the close. A panel too narrow for every name keeps
//! the icons and names the tab shown only.
//!
//! The vitals stay whatever the tab: the frame's time against the budget,
//! the CPU's and the GPU's share of it, what it drew and what it holds.

use super::paint::{
    BUDGET_NS, Health, ICON, Painter, alpha, bytes, count, percent, signed_bytes, time,
};
use super::{Act, Dock, Follow, Tab, gpu_span, whole};
use crate::runtime::perf::{FrameReport, Recorder};
use crate::widgets::prelude::Rect;

/// The strip's height, and the vitals'.
const STRIP: f32 = 32.0;
const VITALS: f32 = 26.0;
/// A tab's room either side of its icon and name.
const TAB_PAD: f32 = 10.0;
/// An icon button's side.
const BUTTON: f32 = 24.0;
/// The line under the tab shown.
const UNDERLINE: f32 = 2.0;
/// The room around a view's content.
const BODY_PAD: f32 = 12.0;

/// What the head shows of the dev mode's state.
pub(super) struct State {
    pub tab: Tab,
    pub follow: Follow,
    pub dock: Dock,
    pub picking: bool,
}

/// The head at the top of `panel`; the place left for the view.
pub(super) fn draw(
    p: &mut Painter<'_>,
    panel: Rect,
    state: &State,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
) -> Rect {
    // one line when the tabs and what stands beside them fit, even as
    // icons; two when they do not: the tabs, then the rest under them
    let beside = controls_width(p, state, shown);
    let least = BUTTON + p.gap() * 1.5 + (TAB_PAD * 2.0 + ICON) * Tab::ALL.len() as f32;
    let two = least + beside > panel.w;
    let lines = if two { 2.0 } else { 1.0 };
    let strip = Rect::new(panel.x, panel.y, panel.w, STRIP * lines);
    p.fill(strip, p.theme.colors.bg_panel.0);
    let first = Rect::new(strip.x, strip.y, strip.w, STRIP);
    if two {
        let second = Rect::new(strip.x, strip.y + STRIP, strip.w, STRIP);
        controls(p, second, state, shown);
        tabs(p, first, state);
        p.rule_h(strip.x, first.bottom() - 1.0, strip.w);
    } else {
        let right = controls(p, first, state, shown);
        tabs(
            p,
            Rect::new(first.x, first.y, right - first.x, first.h),
            state,
        );
    }
    p.rule_h(strip.x, strip.bottom() - 1.0, strip.w);

    let line = Rect::new(panel.x, strip.bottom(), panel.w, VITALS);
    vitals(p, line, recorder, shown);
    p.rule_h(line.x, line.bottom() - 1.0, line.w);

    let top = line.bottom() + BODY_PAD;
    Rect::new(
        panel.x + BODY_PAD,
        top,
        (panel.w - BODY_PAD * 2.0).max(0.0),
        (panel.bottom() - top - BODY_PAD).max(0.0),
    )
}

/// An icon button in `at`: its wash while hot or `on`, its icon.
fn button(p: &mut Painter<'_>, at: Rect, icon: &str, act: Act, on: bool) {
    let hot = p.region(at, act);
    let info = p.theme.colors.info.0;
    if on {
        p.chip(at, alpha(info, 0.14));
    } else if hot {
        p.chip(at, alpha(p.ink(), 0.07));
    }
    let ink = match (on, hot) {
        (true, _) => info,
        (false, true) => p.ink(),
        (false, false) => p.mid(),
    };
    p.icon_in(icon, at, ICON, ink);
}

/// The picker and the tabs, from the left of `strip`.
fn tabs(p: &mut Painter<'_>, strip: Rect, state: &State) {
    let (caption, strong) = (p.caption(), p.strong());
    let gap = p.gap();
    let mid_y = strip.y + (strip.h - BUTTON) / 2.0;
    let pick = Rect::new(strip.x + gap / 2.0, mid_y, BUTTON, BUTTON);
    button(p, pick, "mouse-pointer-2", Act::Pick, state.picking);
    let mut x = pick.right() + gap / 2.0;
    p.rule_v(x, strip.y + gap, strip.h - gap * 2.0);
    x += gap / 2.0;

    // every name when they fit, the icons and the name of the one shown
    // when they do not
    let named = Tab::ALL.map(|tab| TAB_PAD * 2.0 + ICON + gap / 2.0 + p.width(tab.name(), &strong));
    let room = strip.right() - x;
    let names = named.iter().sum::<f32>() <= room;
    let shown = Tab::ALL.iter().position(|tab| *tab == state.tab);
    let own = shown.map_or(0.0, |i| named[i]);
    let others = (Tab::ALL.len() - 1) as f32;
    let full = TAB_PAD * 2.0 + ICON;
    // the name of the tab shown goes too when even the icons are tight
    let named_one = own + full * others <= room;
    let bare = if named_one {
        full
    } else {
        (room / Tab::ALL.len() as f32).min(full)
    };
    for (tab, named) in Tab::ALL.into_iter().zip(named) {
        let on = tab == state.tab;
        let label = names || (on && named_one);
        let w = if label { named } else { bare };
        let at = Rect::new(x, strip.y, w, strip.h);
        let hot = p.region(at, Act::Tab(tab));
        if hot && !on {
            let wash = Rect::new(at.x + 2.0, at.y + 4.0, at.w - 4.0, at.h - 9.0);
            p.chip(wash, alpha(p.ink(), 0.06));
        }
        let ink = if on || hot { p.ink() } else { p.mid() };
        let icon_y = at.y + (at.h - ICON) / 2.0;
        let pad = if label { TAB_PAD } else { (at.w - ICON) / 2.0 };
        p.icon(tab.icon(), at.x + pad, icon_y, ICON, ink);
        if label {
            let style = if on { &strong } else { &caption };
            let text_x = at.x + TAB_PAD + ICON + gap / 2.0;
            let text_y = at.y + (at.h - style.line_height) / 2.0;
            p.text(tab.name(), text_x, text_y, at.right() - text_x, ink, style);
        }
        if on {
            let line = Rect::new(
                at.x + 4.0,
                at.bottom() - UNDERLINE - 1.0,
                at.w - 8.0,
                UNDERLINE,
            );
            p.fill(line, p.theme.colors.accent.0);
        }
        x += w;
    }
}

/// The choices of which frame the views show: a name, what it follows,
/// and whether it is the one chosen.
fn segments(state: &State, shown: Option<&FrameReport>) -> Vec<(String, Follow, bool)> {
    let pinned = match (state.follow, shown) {
        (Follow::Frame(index), frame) => Some(frame.map_or(index, |f| f.index)),
        _ => None,
    };
    let mut all = vec![
        (
            "Live".to_owned(),
            Follow::Latest,
            state.follow == Follow::Latest,
        ),
        (
            "Slowest".to_owned(),
            Follow::Slowest,
            state.follow == Follow::Slowest,
        ),
    ];
    if let Some(index) = pinned {
        all.push((format!("#{index}"), Follow::Frame(index), true));
    }
    all
}

/// The dot before "Live".
const DOT: f32 = 6.0;

/// How wide each choice of [`segments`] is drawn.
fn segment_widths(p: &Painter<'_>, all: &[(String, Follow, bool)]) -> Vec<f32> {
    let strong = p.num_strong();
    let gap = p.gap();
    all.iter()
        .map(|(label, follow, _)| {
            let lead = if *follow == Follow::Latest {
                DOT + gap / 2.0
            } else {
                0.0
            };
            p.width(label, &strong) + gap * 2.0 + lead
        })
        .collect()
}

/// The room [`controls`] takes at the right of the strip.
fn controls_width(p: &Painter<'_>, state: &State, shown: Option<&FrameReport>) -> f32 {
    let gap = p.gap();
    let track: f32 = segment_widths(p, &segments(state, shown))
        .iter()
        .sum::<f32>()
        + 4.0;
    gap / 2.0 + BUTTON * 2.0 + gap + track + gap
}

/// The frame the views show, the dock and the close, at the right of
/// `strip`; where they begin.
fn controls(p: &mut Painter<'_>, strip: Rect, state: &State, shown: Option<&FrameReport>) -> f32 {
    let gap = p.gap();
    let mid_y = strip.y + (strip.h - BUTTON) / 2.0;
    let close = Rect::new(strip.right() - gap / 2.0 - BUTTON, mid_y, BUTTON, BUTTON);
    button(p, close, "x", Act::Close, false);
    let dock = Rect::new(close.x - BUTTON, mid_y, BUTTON, BUTTON);
    let to = match state.dock {
        Dock::Bottom => "panel-right",
        Dock::Side => "panel-bottom",
    };
    button(p, dock, to, Act::Dock, false);
    p.rule_v(dock.x - gap / 2.0, strip.y + gap, strip.h - gap * 2.0);

    // which frame: one track, a segment a choice, the one chosen raised
    let (caption, strong) = (p.caption(), p.num_strong());
    let all = segments(state, shown);
    let widths = segment_widths(p, &all);
    let pad = gap;
    let track_h = BUTTON - 2.0;
    let track_w: f32 = widths.iter().sum::<f32>() + 4.0;
    let track = Rect::new(
        dock.x - gap - track_w,
        strip.y + (strip.h - track_h) / 2.0,
        track_w,
        track_h,
    );
    p.chip(track, alpha(p.ink(), 0.06));
    let mut x = track.x + 2.0;
    for ((label, follow, on), w) in all.into_iter().zip(widths) {
        let at = Rect::new(x, track.y + 2.0, w, track.h - 4.0);
        let live = follow == Follow::Latest;
        let hot = p.region(at, Act::Follow(follow));
        if on {
            p.chip(at, p.theme.colors.bg.0);
        }
        let ink = if on || hot { p.ink() } else { p.mid() };
        let mut text_x = at.x + pad;
        if live {
            let color = if on {
                p.theme.colors.success.0
            } else {
                p.faint()
            };
            p.pill(
                Rect::new(text_x, at.y + (at.h - DOT) / 2.0, DOT, DOT),
                color,
            );
            text_x += DOT + gap / 2.0;
        }
        let style = if on { &strong } else { &caption };
        let text_y = at.y + (at.h - style.line_height) / 2.0;
        p.text(&label, text_x, text_y, at.right() - text_x, ink, style);
        x += w;
    }
    track.x - gap
}

/// The vital numbers of the frame shown, on one line.
fn vitals(p: &mut Painter<'_>, line: Rect, recorder: &Recorder, shown: Option<&FrameReport>) {
    let (caption, num, strong) = (p.caption(), p.num(), p.num_strong());
    let gap = p.gap();
    let pad = gap * 1.5;
    let y = line.y + (line.h - p.line()) / 2.0;
    let Some(shown) = shown else {
        let waiting = "Waiting for a frame: move the pointer over the page.";
        p.text(
            waiting,
            line.x + pad,
            y,
            line.w - pad * 2.0,
            p.mid(),
            &caption,
        );
        return;
    };

    // at the right: the frame's number, and what kind of build measures it
    let mut right = line.right() - pad;
    let frame = format!("frame {}", shown.index);
    p.text_right(&frame, right, y, p.mid(), &num);
    right -= p.width(&frame, &num) + gap;
    if cfg!(debug_assertions) {
        let warning = p.theme.colors.warning.0;
        let w = p.width("debug build", &p.small()) + (p.theme.spacing.xs + 2.0) * 2.0;
        p.badge("debug build", right - w, y, warning);
        right -= w + gap;
    }

    let total = whole(shown);
    let health = Health::of(total);
    let color = p.health_color(health);
    let mut x = line.x + pad;
    let dot = 8.0;
    p.pill(Rect::new(x, line.y + (line.h - dot) / 2.0, dot, dot), color);
    x += dot + gap / 2.0;
    let whole_text = time(total);
    p.text(&whole_text, x, y, right - x, p.ink(), &strong);
    x += p.width(&whole_text, &strong) + gap / 2.0;
    x += p.badge(&format!("{}%", percent(total, BUDGET_NS)), x, y, color) + pad;

    let redrawn = shown.layers.iter().filter(|l| l.redrawn).count();
    let nodes: u64 = shown.layers.iter().map(|l| u64::from(l.nodes)).sum();
    let mut items = vec![
        ("CPU", time(shown.pipeline.cpu_nanos)),
        (
            "GPU",
            match (gpu_span(shown), recorder.gpu_available()) {
                (Some(span), _) => time(span),
                (None, true) => "...".to_owned(),
                (None, false) => "n/a".to_owned(),
            },
        ),
        ("Draws", count(u64::from(shown.stats.draw_calls))),
        ("Layers", format!("{redrawn}/{}", shown.layers.len())),
        ("Nodes", count(nodes)),
        ("Glyphs", count(u64::from(shown.stats.glyphs))),
    ];
    if shown.heap.counted {
        items.push(("Heap", signed_bytes(shown.heap.net_bytes())));
    }
    items.push(("GPU mem", bytes(shown.memory.gpu_total_bytes())));
    if let Some(rss) = shown.memory.process_rss_bytes {
        items.push(("RSS", bytes(rss)));
    }
    for (label, value) in items {
        let label_w = p.width(label, &caption);
        let value_w = p.width(&value, &num);
        if x + label_w + gap / 2.0 + value_w > right {
            break;
        }
        p.text(label, x, y, label_w + 1.0, p.faint(), &caption);
        x += label_w + gap / 2.0;
        p.text(&value, x, y, value_w + 1.0, p.ink(), &num);
        x += value_w + pad;
    }
}

/// The gear that opens the panel: a small round button on the page.
pub(super) fn gear(p: &mut Painter<'_>, at: Rect) {
    let hot = p.region(at, Act::Open);
    let colors = &p.theme.colors;
    let (ground, ink) = if hot {
        (colors.bg_hover.0, colors.text.0)
    } else {
        (colors.bg.0, colors.text_mid.0)
    };
    p.pill(at.outset(1.0), colors.divider.0);
    p.pill(at, ground);
    p.icon_in("settings", at, ICON, ink);
}
