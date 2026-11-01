//! Builder tour: the declarative engine::compose::builder API live. The demo card is
//! a real Element tree rendered via render_interactive, so its buttons are
//! REAL engine hit regions (the old demo drew fake buttons via View::render)
//! and a counter proves reactivity: click, state changes, the tree rebuilds.
//! The source beside it is a `CodeBlock` (the theme's code recipe), one
//! row per line, cut at the card's width. Named builder_tour to not
//! collide with the engine's engine::compose::builder.

use engine::compose::builder::{Element, div, text as btext};
use engine::compose::view::ViewContext;
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::drawing::text::TextMeasurer;
use engine::styling::color::Color;
use engine::styling::theme::{Theme, TypeRole, TypeRun};
use engine::widgets::content::{CodeBlock, CodeLine, LineKind};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::group_label;

/// Gap-class constants: card padding, card gap, label band, and the
/// minimum live-result card width (below it the cards stack).
const PAD: f32 = 24.0;
const GAP: f32 = 24.0;
const LABEL_H: f32 = 24.0;
const MIN_DEMO_W: f32 = 320.0;
/// Layout-engine upper bound; the card takes its natural height (tested).
const DEMO_MAX_H: f32 = 2048.0;

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
    "fn action(label: &str, ty: &TypographyScale) -> Element {",
    "    let style = ty.role(TypeRole::Heading);",
    "    div().px(14).py(8).rounded(8)",
    "        .child(text(label).text_style(&style))",
    "}",
    "fn counter(n: i64, ty: &TypographyScale) -> Element {",
    "    let title = ty.run(TypeRole::Title.strong());",
    "    let count = ty.run(TypeRole::TitleLg.strong());",
    "    div().col().gap(10).p(24)",
    "        .bg(glass.surface).rounded(20)",
    "        .child(text(\"Counter\").text_style(&title))",
    "        .child(div().row().wrap().gap(12)",
    "            .child(action(\"-1\", ty).on_click(|_| send(Dec)))",
    "            .child(div().px(18).py(6)",
    "                .child(text(&n.to_string()).text_style(&count)))",
    "            .child(action(\"+1\", ty).on_click(|_| send(Inc)))",
    "            .child(action(\"Reset\", ty).on_click(|_| send(Reset))))",
    "}",
];

/// The source as a code block: a row per line, captioned by its language.
fn code_block() -> CodeBlock {
    let rows = CODE.iter().map(|l| CodeLine::new(*l, LineKind::Context));
    CodeBlock::lines(rows.collect()).caption("rust")
}

/// Natural width of the source card: the widest line in the block's
/// style, and the block's padding.
fn code_panel_width(theme: &Theme) -> f32 {
    let style = CodeBlock::text_style(theme);
    let widest = CODE
        .iter()
        .map(|l| TextMeasurer::measure_styled(l, &style, None).0)
        .fold(0.0, f32::max);
    (widest + theme.navigation.code.pad * 2.0).ceil()
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

pub struct BuilderPage {
    state: CounterState,
    /// Hovered ACTIONS index; the tree is rebuilt with the hover color.
    hover: Option<usize>,
    code: CodeBlock,
}

impl Default for BuilderPage {
    fn default() -> Self {
        Self {
            state: CounterState::default(),
            hover: None,
            code: code_block(),
        }
    }
}

impl BuilderPage {
    pub fn new() -> Self {
        Self::default()
    }

    /// The demo Element tree, rebuilt from state on every render. Width
    /// derives from the content rect; only the row wraps below MIN_DEMO_W.
    fn demo_tree(&self, w: f32, theme: &Theme) -> Element {
        let (g, tc, r) = (&theme.glass, &theme.colors, theme.radius.md);
        let ty = &theme.typography;
        let t = |s: &str, run: TypeRun, c: Color| btext(s).text_style(&ty.run(run)).text_color(c);
        let mut row = div().row().wrap().gap(12.0).align_center().pt(4.0);
        for (i, &(label, _)) in ACTIONS.iter().enumerate() {
            let hov = self.hover == Some(i);
            let bg = if hov { g.button_hover } else { g.button };
            // on_click makes render_interactive emit the hit region; the engine
            // has no handler dispatch yet, so handle_event routes zone i.
            // The builder's `button` writes its label in a fixed light
            // ink, unreadable on a light theme's button: the same box, its
            // label in the theme's text.
            let label = t(label, TypeRole::Heading.run(), tc.text);
            let action = div().bg(bg).rounded(r).px(14.0).py(8.0).child(label);
            row = row.child(action.on_click(|_| {}));
            if i == 0 {
                let pill = div().px(18.0).py(6.0).bg(g.surface_active).rounded(r);
                row = row.child(pill.child(t(
                    &self.state.count.to_string(),
                    TypeRole::TitleLg.strong(),
                    tc.text,
                )));
            }
        }
        let caption = format!("tree rebuilt from state {} times", self.state.rebuilds);
        let card = div().col().gap(10.0).p(PAD).w(w).bg(g.surface);
        card.rounded(theme.radius.lg)
            .border(1.0)
            .border_color(g.edge_soft)
            .child(t("Counter", TypeRole::Title.strong(), tc.text))
            .child(t(
                "State drives the tree.",
                TypeRole::BodyLg.run(),
                tc.text_dim,
            ))
            .child(row)
            .child(t(&caption, TypeRole::Body.run(), g.text_faint))
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
    fn layout(&self, content: Rect, theme: &Theme) -> (Rect, Rect, f32) {
        let code_w = code_panel_width(theme);
        let side = content.w >= MIN_DEMO_W + GAP + code_w;
        let demo_w = content.w - if side { GAP + code_w } else { 0.0 };
        let demo_y = content.y + LABEL_H;
        let (demo_h, _) = self.demo_geometry(content.x, demo_y, demo_w);
        let demo = Rect::new(content.x, demo_y, demo_w, demo_h);
        let code_h = self.code.height_for(code_w, theme);
        let stack_y = demo_y + demo_h + GAP + LABEL_H;
        let code = if side {
            Rect::new(content.x + demo_w + GAP, demo_y, code_w, code_h)
        } else {
            Rect::new(content.x, stack_y, content.w, code_h)
        };
        let total = (demo.y + demo.h).max(code.y + code.h) - content.y + GAP;
        (demo, code, total)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).2
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let demo = self.layout(content, theme).0;
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
        let (demo, code, _) = self.layout(content, theme);

        group_label(c, "LIVE RESULT", demo.x, demo.y - LABEL_H, theme);
        let mut cx = ViewContext::new(demo.w, DEMO_MAX_H).with_theme(theme.clone());
        let result = self.demo_tree(demo.w, theme).render_interactive(&mut cx);
        push_offset(c, result.nodes, demo.x, demo.y);

        group_label(c, "SOURCE", code.x, code.y - LABEL_H, theme);
        self.code.render(c, code, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::drawing::compositor::LayerId;

    fn content(w: f32) -> Rect {
        Rect::new(288.0, 118.0, w, 682.0)
    }
    fn down(x: f32, y: f32) -> WidgetEvent {
        WidgetEvent::MouseDown { x, y }
    }
    fn mv(x: f32, y: f32) -> WidgetEvent {
        WidgetEvent::MouseMove { x, y }
    }
    fn zones(s: &BuilderPage, c: Rect) -> Vec<Rect> {
        let demo = s.layout(c, &Theme::hoff()).0;
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
        let s = BuilderPage::new();
        let wide = content(1132.0);
        let (demo, code, _) = s.layout(wide, &Theme::hoff());
        assert!(demo.w >= MIN_DEMO_W);
        assert!(code.x >= demo.x + demo.w, "code sits beside the demo");
        assert!(code.x + code.w <= wide.x + wide.w + 0.5);
        assert!(demo.h > 100.0 && demo.h < 500.0, "natural h={}", demo.h);

        let narrow = content(232.0);
        let (demo, code, total) = s.layout(narrow, &Theme::hoff());
        assert_eq!((demo.w, code.w), (narrow.w, narrow.w), "stacked cards span");
        assert!(code.y >= demo.y + demo.h, "code stacks under demo");
        assert!(
            total >= code.y + code.h - narrow.y
                && s.content_height(narrow, &Theme::hoff()) == total
        );
    }

    #[test]
    fn clicks_and_hover_on_real_hit_regions_drive_state_and_redraw() {
        let mut s = BuilderPage::new();
        let wide = content(1132.0);
        let demo = s.layout(wide, &Theme::hoff()).0;
        let z = zones(&s, wide);
        assert_eq!(z.len(), ACTIONS.len(), "engine zones in ACTIONS order");
        for z in &z {
            assert!(z.x >= demo.x && z.x + z.w <= demo.x + demo.w + 0.5);
            assert!(z.y >= demo.y && z.y + z.h <= demo.y + demo.h + 0.5);
        }
        assert!(z[0].x < z[1].x && z[1].x < z[2].x);

        let (x, y) = z[1].center(); // "+1"
        let r = s.handle_event(&down(x, y), wide, &Theme::hoff());
        assert!(r.clicked && r.changed, "click must request a redraw");
        assert_eq!(s.state.count, 1);

        let (x, y) = z[0].center(); // "-1"
        s.handle_event(&down(x, y), wide, &Theme::hoff());
        assert_eq!(s.state.count, 0);
        assert!(
            s.handle_event(&mv(x, y), wide, &Theme::hoff()).changed,
            "hover in redraws"
        );
        assert!(
            !s.handle_event(&mv(x, y), wide, &Theme::hoff()).changed,
            "no hover change"
        );
        assert!(
            s.handle_event(&mv(5.0, 5.0), wide, &Theme::hoff()).changed,
            "hover out"
        );

        let (x, y) = z[2].center(); // "Reset" while already 0
        let r = s.handle_event(&down(x, y), wide, &Theme::hoff());
        assert!(r.handled && !r.changed, "no-op click must not redraw");
        assert_eq!(
            s.handle_event(&down(5.0, 5.0), wide, &Theme::hoff()),
            EventResult::IGNORED
        );
    }

    #[test]
    fn render_shows_count_updates_after_click_and_the_code_block() {
        let mut s = BuilderPage::new();
        let (wide, theme) = (content(1132.0), Theme::hoff());
        let mut c = Compositor::new();
        s.render(&mut c, wide, &theme);
        assert!(has_text(&c, "0"), "counter starts at 0");

        let style = CodeBlock::text_style(&theme);
        let code_lines = c
            .layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .filter(|n| {
                matches!(n, SceneNode::Text { key, .. }
                    if CODE.contains(&key.text.as_str())
                        && key.font_family == style.font_family)
            })
            .count();
        assert_eq!(
            code_lines,
            CODE.len(),
            "every code line draws whole, in the block's style"
        );

        let (x, y) = zones(&s, wide)[1].center();
        assert!(s.handle_event(&down(x, y), wide, &Theme::hoff()).changed);
        let mut c = Compositor::new();
        s.render(&mut c, wide, &theme);
        assert!(has_text(&c, "1"), "rebuilt tree must show the new count");
        assert!(!has_text(&c, "0"), "stale count must be gone");
    }

    /// The width sweep, 280 to 1600 every 8 px, in core and hoff: both
    /// cards inside the content and apart, every hit zone inside the demo.
    #[test]
    fn nothing_cuts_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let s = BuilderPage::new();
            for w in (280..=1600).step_by(8) {
                let c = Rect::new(0.0, 0.0, w as f32, 800.0);
                let (demo, code, total) = s.layout(c, &theme);
                for r in [demo, code] {
                    assert!(r.x >= c.x && r.right() <= c.right() + 0.5, "{w}: {r:?}");
                }
                let apart = code.x >= demo.right() || code.y >= demo.bottom();
                assert!(apart, "{w}: the cards overlap");
                assert!(total >= code.bottom() - c.y, "{w}");
                for z in s.demo_geometry(demo.x, demo.y, demo.w).1 {
                    assert!(z.right() <= demo.right() + 0.5, "{w}: zone out");
                }
            }
        }
    }

    /// Every action's label is written in the theme's text over its
    /// button, in a light theme as in a dark one: readable.
    #[test]
    fn the_actions_read_in_every_theme() {
        for theme in [Theme::hoff_core(), Theme::hoff(), Theme::light()] {
            let s = BuilderPage::new();
            let mut c = Compositor::new();
            s.render(&mut c, content(1132.0), &theme);
            let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
            for (label, _) in ACTIONS {
                let ink = nodes.iter().find_map(|n| match n {
                    SceneNode::Text { key, color, .. } if key.text == label => Some(*color),
                    _ => None,
                });
                assert_eq!(ink, Some(theme.colors.text.0), "{label}");
            }
        }
    }
}
