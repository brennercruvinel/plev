//! Builder tour: the declarative engine::builder API live. The demo card is
//! a real Element tree rendered via render_interactive, so its buttons are
//! REAL engine hit regions (the old demo drew fake buttons via View::render)
//! and a counter proves reactivity: click, state changes, the tree rebuilds.
//! The source panel besides it shows the code in Inclusive Sans, the
//! embedded UI family. Named
//! builder_tour to not collide with the engine's engine::builder.

use engine::builder::{Element, button, div, text as btext};
use engine::color::Color;
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::widgets::{EventResult, Rect, WidgetEvent};
use engine::view::ViewContext;

use super::{group_label, panel};

/// Gap-class constants: card padding, card gap, label band, and the
/// minimum live-result card width (below it the cards stack).
const PAD: f32 = 24.0;
const GAP: f32 = 24.0;
const LABEL_H: f32 = 24.0;
const MIN_DEMO_W: f32 = 320.0;
/// Layout-engine upper bound; the card takes its natural height (tested).
const DEMO_MAX_H: f32 = 2048.0;
const CODE_LINE_H: f32 = 18.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterMsg {
    Decrement,
    Increment,
    Reset,
}

/// Counter backend: pure message -> state, tested before any pixel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CounterState {
    pub count: i64,
    /// Applied state changes; each one rebuilt the element tree.
    pub rebuilds: u64,
}

impl CounterState {
    /// The click handler; true = visible state changed, caller invalidates.
    pub fn apply(&mut self, msg: CounterMsg) -> bool {
        let next = match msg {
            CounterMsg::Decrement => self.count - 1,
            CounterMsg::Increment => self.count + 1,
            CounterMsg::Reset => 0,
        };
        let changed = next != self.count;
        if changed {
            self.count = next;
            self.rebuilds += 1;
        }
        changed
    }
}

/// Demo row buttons in hit-region preorder: zone index i == ACTIONS[i].
const ACTIONS: [(&str, CounterMsg); 3] = [
    ("-1", CounterMsg::Decrement),
    ("+1", CounterMsg::Increment),
    ("Reset", CounterMsg::Reset),
];

/// Source shown in the panel; mirrors `demo_tree` below.
const CODE: &[&str] = &[
    "fn counter(n: i64) -> Element {",
    "    div().col().gap(10).p(24)",
    "        .bg(glass.surface).rounded(20)",
    "        .child(text(\"Counter\")",
    "            .font_size(16).font_weight(600))",
    "        .child(div().row().wrap().gap(12)",
    "            .child(button(\"-1\").on_click(|_| send(Dec)))",
    "            .child(div().px(18).py(6)",
    "                .child(text(&n.to_string())))",
    "            .child(button(\"+1\").on_click(|_| send(Inc)))",
    "            .child(button(\"Reset\").on_click(|_| send(Reset))))",
    "}",
];

/// One TextStyle for the code runs, shared by measurement and drawing.
fn code_style() -> TextStyle {
    let mut style = TextStyle::new(12.0);
    style.line_height = CODE_LINE_H;
    style.font_family = Some("Inclusive Sans".to_string());
    style
}

/// Natural width of the source card: widest measured line plus padding.
fn code_panel_width() -> f32 {
    let style = code_style();
    let widest = CODE
        .iter()
        .map(|l| TextMeasurer::measure_styled(l, &style, None).0)
        .fold(0.0, f32::max);
    widest + PAD * 2.0
}

/// Push builder output shifted to the card origin (trees lay out at (0,0)).
fn push_offset(c: &mut Compositor, nodes: Vec<SceneNode>, dx: f32, dy: f32) {
    for mut node in nodes {
        match &mut node {
            SceneNode::Rect { x, y, .. }
            | SceneNode::RoundedRect { x, y, .. }
            | SceneNode::GradientRect { x, y, .. }
            | SceneNode::Text { x, y, .. }
            | SceneNode::Shadow { x, y, .. } => {
                *x += dx;
                *y += dy;
            }
            // Extend before the demo tree gains image/clip/blur/path nodes;
            // these would otherwise stay at the tree's local origin.
            _ => {}
        }
        c.push(node);
    }
}

#[derive(Default)]
pub struct BuilderSection {
    state: CounterState,
    /// Hovered ACTIONS index; the tree is rebuilt with the hover color.
    hover: Option<usize>,
}

impl BuilderSection {
    pub fn new() -> Self {
        Self::default()
    }

    /// The demo Element tree, rebuilt from state on every render. Width
    /// derives from the content rect; only the row wraps below MIN_DEMO_W.
    fn demo_tree(&self, w: f32, theme: &Theme) -> Element {
        let (g, tc, r) = (&theme.glass, &theme.colors, theme.radius.md);
        let t = |s: &str, fs: f32, fw: u16, c: Color| {
            btext(s).font_size(fs).font_weight(fw).text_color(c)
        };
        let mut row = div().row().wrap().gap(12.0).align_center().pt(4.0);
        for (i, &(label, _)) in ACTIONS.iter().enumerate() {
            let hov = self.hover == Some(i);
            let bg = if hov { g.button_hover } else { g.button };
            // on_click makes render_interactive emit the hit region; the engine
            // has no handler dispatch yet, so handle_event routes zone i.
            row = row.child(button(label).bg(bg).rounded(r).px(14.0).on_click(|_| {}));
            if i == 0 {
                let pill = div().px(18.0).py(6.0).bg(g.surface_active).rounded(r);
                row = row.child(pill.child(t(&self.state.count.to_string(), 18.0, 700, tc.text)));
            }
        }
        let caption = format!("tree rebuilt from state {} times", self.state.rebuilds);
        let card = div().col().gap(10.0).p(PAD).w(w).bg(g.surface);
        card.rounded(theme.radius.lg)
            .border(1.0)
            .border_color(g.edge_soft)
            .child(t("Counter", 16.0, 600, tc.text))
            .child(t("State drives the tree.", 13.0, 400, tc.text_dim))
            .child(row)
            .child(t(&caption, 12.0, 400, g.text_faint))
    }

    /// Engine geometry for the demo card at origin (ox, oy), width `w`:
    /// natural height (first node = the root's RoundedRect surface), hit
    /// zones (preorder: zone i == ACTIONS[i]). Colors never affect taffy
    /// geometry, so hoff stands in for any active theme.
    fn demo_geometry(&self, ox: f32, oy: f32, w: f32) -> (f32, Vec<Rect>) {
        let (mut cx, hoff) = (ViewContext::new(w, DEMO_MAX_H), Theme::hoff());
        let result = self.demo_tree(w, &hoff).render_interactive(&mut cx);
        let height = match result.nodes.first() {
            Some(SceneNode::RoundedRect { h, .. }) => *h,
            _ => 0.0,
        };
        let mut zones = Vec::new();
        for r in &result.hit_regions {
            let b = &r.bounds;
            zones.push(Rect::new(ox + b.x, oy + b.y, b.width, b.height));
        }
        (height, zones)
    }

    /// Pure split, side by side when both cards fit, stacked otherwise:
    /// (demo card, code card, total content height for page scroll).
    fn layout(&self, content: Rect) -> (Rect, Rect, f32) {
        let code_w = code_panel_width();
        let side = content.w >= MIN_DEMO_W + GAP + code_w;
        let demo_w = content.w - if side { GAP + code_w } else { 0.0 };
        let demo_y = content.y + LABEL_H;
        let (demo_h, _) = self.demo_geometry(content.x, demo_y, demo_w);
        let demo = Rect::new(content.x, demo_y, demo_w, demo_h);
        let code_h = CODE.len() as f32 * CODE_LINE_H + PAD * 2.0;
        let stack_y = demo_y + demo_h + GAP + LABEL_H;
        let code = if side {
            Rect::new(content.x + demo_w + GAP, demo_y, code_w, code_h)
        } else {
            Rect::new(content.x, stack_y, content.w, code_h)
        };
        let total = (demo.y + demo.h).max(code.y + code.h) - content.y + GAP;
        (demo, code, total)
    }

    pub fn content_height(&self, content: Rect) -> f32 {
        self.layout(content).2
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let demo = self.layout(content).0;
        let (_, zones) = self.demo_geometry(demo.x, demo.y, demo.w);
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = zones.iter().position(|z| z.contains(x, y));
                if hit == self.hover {
                    return EventResult::IGNORED;
                }
                self.hover = hit;
                EventResult::changed()
            }
            WidgetEvent::MouseDown { x, y } => {
                let Some(i) = zones.iter().position(|z| z.contains(x, y)) else {
                    return EventResult::IGNORED;
                };
                let changed = self.state.apply(ACTIONS[i].1);
                EventResult {
                    handled: true,
                    changed,
                    clicked: changed,
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (demo, code, _) = self.layout(content);

        group_label(c, "LIVE RESULT", demo.x, demo.y - LABEL_H, theme);
        let mut cx = ViewContext::new(demo.w, DEMO_MAX_H).with_theme(theme.clone());
        let result = self.demo_tree(demo.w, theme).render_interactive(&mut cx);
        push_offset(c, result.nodes, demo.x, demo.y);

        group_label(c, "SOURCE", code.x, code.y - LABEL_H, theme);
        panel(c, code, theme);
        let style = code_style();
        let (x, y, w, h) = (code.x, code.y, code.w, code.h);
        c.push(SceneNode::PushClip { x, y, w, h });
        for (i, line) in CODE.iter().enumerate() {
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(line, &style, None),
                x: code.x + PAD,
                y: code.y + PAD + i as f32 * CODE_LINE_H,
                color: theme.colors.text_dim.0,
            });
        }
        c.push(SceneNode::PopClip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::compositor::LayerId;

    fn content(w: f32) -> Rect {
        Rect::new(288.0, 118.0, w, 682.0)
    }
    fn down(x: f32, y: f32) -> WidgetEvent {
        WidgetEvent::MouseDown { x, y }
    }
    fn mv(x: f32, y: f32) -> WidgetEvent {
        WidgetEvent::MouseMove { x, y }
    }
    fn zones(s: &BuilderSection, c: Rect) -> Vec<Rect> {
        let demo = s.layout(c).0;
        s.demo_geometry(demo.x, demo.y, demo.w).1
    }
    fn has_text(c: &Compositor, t: &str) -> bool {
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Text { key, .. } if key.text == t))
    }

    #[test]
    fn counter_handler_applies_messages_and_counts_rebuilds() {
        use CounterMsg::*;
        let mut st = CounterState::default();
        assert!(st.apply(Increment) && st.apply(Decrement) && st.apply(Decrement));
        assert_eq!(st.count, -1);
        assert!(st.apply(Reset));
        assert_eq!((st.count, st.rebuilds), (0, 4));
        assert!(!st.apply(Reset) && st.rebuilds == 4, "reset at 0: no-op");
    }

    #[test]
    fn layout_splits_wide_and_stacks_narrow() {
        let s = BuilderSection::new();
        let wide = content(1132.0);
        let (demo, code, _) = s.layout(wide);
        assert!(demo.w >= MIN_DEMO_W);
        assert!(code.x >= demo.x + demo.w, "code sits beside the demo");
        assert!(code.x + code.w <= wide.x + wide.w + 0.5);
        assert!(demo.h > 100.0 && demo.h < 500.0, "natural h={}", demo.h);

        let narrow = content(232.0);
        let (demo, code, total) = s.layout(narrow);
        assert_eq!((demo.w, code.w), (narrow.w, narrow.w), "stacked cards span");
        assert!(code.y >= demo.y + demo.h, "code stacks under demo");
        assert!(total >= code.y + code.h - narrow.y && s.content_height(narrow) == total);
    }

    #[test]
    fn clicks_and_hover_on_real_hit_regions_drive_state_and_redraw() {
        let mut s = BuilderSection::new();
        let wide = content(1132.0);
        let demo = s.layout(wide).0;
        let z = zones(&s, wide);
        assert_eq!(z.len(), ACTIONS.len(), "engine zones in ACTIONS order");
        for z in &z {
            assert!(z.x >= demo.x && z.x + z.w <= demo.x + demo.w + 0.5);
            assert!(z.y >= demo.y && z.y + z.h <= demo.y + demo.h + 0.5);
        }
        assert!(z[0].x < z[1].x && z[1].x < z[2].x);

        let (x, y) = z[1].center(); // "+1"
        let r = s.handle_event(&down(x, y), wide);
        assert!(r.clicked && r.changed, "click must request a redraw");
        assert_eq!(s.state.count, 1);

        let (x, y) = z[0].center(); // "-1"
        s.handle_event(&down(x, y), wide);
        assert_eq!(s.state.count, 0);
        assert!(s.handle_event(&mv(x, y), wide).changed, "hover in redraws");
        assert!(!s.handle_event(&mv(x, y), wide).changed, "no hover change");
        assert!(s.handle_event(&mv(5.0, 5.0), wide).changed, "hover out");

        let (x, y) = z[2].center(); // "Reset" while already 0
        let r = s.handle_event(&down(x, y), wide);
        assert!(r.handled && !r.changed, "no-op click must not redraw");
        assert_eq!(s.handle_event(&down(5.0, 5.0), wide), EventResult::IGNORED);
    }

    #[test]
    fn render_shows_count_updates_after_click_and_mono_code() {
        let mut s = BuilderSection::new();
        let (wide, theme) = (content(1132.0), Theme::hoff());
        let mut c = Compositor::new();
        s.render(&mut c, wide, &theme);
        assert!(has_text(&c, "0"), "counter starts at 0");

        let code_lines = c
            .layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .filter(|n| {
                matches!(n, SceneNode::Text { key, .. }
                    if key.font_family.as_deref() == Some("Inclusive Sans"))
            })
            .count();
        assert_eq!(
            code_lines,
            CODE.len(),
            "every code line draws in Inclusive Sans"
        );

        let (x, y) = zones(&s, wide)[1].center();
        assert!(s.handle_event(&down(x, y), wide).changed);
        let mut c = Compositor::new();
        s.render(&mut c, wide, &theme);
        assert!(has_text(&c, "1"), "rebuilt tree must show the new count");
        assert!(!has_text(&c, "0"), "stale count must be gone");
    }
}
