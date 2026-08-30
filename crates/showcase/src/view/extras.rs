//! Extras section: the app-level widgets — chips, icon buttons, spinners,
//! empty state and a live split pane. Every state/intent is shown, like
//! the other gallery sections.

use engine::compositor::Compositor;
use engine::graph::{GraphEdge, GraphSpec};
use engine::theme::{Intent, Theme};
use engine::ui::widgets::{
    Button, ButtonSize, ButtonVariant, Chip, EmptyState, EventResult, GraphView, IconButton, Rect,
    Spinner, SpinnerSize, SplitDirection, SplitPane, WidgetEvent,
};

use super::{group_label, panel, text};

const GAP: f32 = 12.0;
const LABEL_H: f32 = 30.0;
const ROW_GAP: f32 = 28.0;
const PANEL_H: f32 = 220.0;

pub struct ExtrasSection {
    chips: Vec<Chip>,
    icon_buttons: Vec<IconButton>,
    spinners: [Spinner; 3],
    /// Spinners run only while this toggle is on: the gallery contract is
    /// that an idle section settles (probe_ticks_settle_with_no_input), so
    /// the demo starts stopped and the button starts the motion.
    spinning: bool,
    spin_toggle: Button,
    empty: EmptyState,
    empty_plain: EmptyState,
    split: SplitPane,
    graph: GraphView,
    /// Demo state: the interactive chip toggles on click.
    filter_on: bool,
}

/// Rects for everything the section draws, top to bottom. Shared by hit
/// testing and render so events and pixels always agree.
struct Layout {
    chips: Vec<Rect>,
    icon_buttons: Vec<Rect>,
    spinners: [Rect; 3],
    spin_toggle: Rect,
    empty: Rect,
    empty_plain: Rect,
    split: Rect,
    graph: Rect,
    labels: Vec<(&'static str, f32)>,
    total_h: f32,
}

impl ExtrasSection {
    pub fn new() -> Self {
        let chips = vec![
            Chip::new("exact"),
            Chip::new("ann").selected(true),
            Chip::new("bm25").intent(Intent::Constructive),
            Chip::new("missing").intent(Intent::Destructive),
            Chip::new("beta").intent(Intent::Informational),
            Chip::new("click me").interactive(true),
        ];
        let icon_buttons = vec![
            IconButton::new("copy"),
            IconButton::new("search").variant(ButtonVariant::Outline),
            IconButton::new("settings").variant(ButtonVariant::Ghost),
            IconButton::new("trash").variant(ButtonVariant::Danger),
            IconButton::new("save").size(ButtonSize::Sm),
            IconButton::new("play")
                .intent(Intent::Constructive)
                .size(ButtonSize::Lg),
            IconButton::new("undo").disabled(true),
        ];
        Self {
            chips,
            icon_buttons,
            spinners: [
                Spinner::new().size(SpinnerSize::Sm),
                Spinner::new(),
                Spinner::new().size(SpinnerSize::Lg),
            ],
            spinning: false,
            spin_toggle: Button::new("Start spinners")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Outline)
                .icon("play"),
            empty: EmptyState::new(
                "No results yet",
                "Run a search and the hits land here, \
                 with scores, citations and the explain panel.",
            )
            .icon("search")
            .cta(Button::new("Run a search").icon("search")),
            empty_plain: EmptyState::new("Nothing archived", "Items you archive show up here."),
            split: SplitPane::new(SplitDirection::Horizontal, 0.35),
            graph: {
                // A small three-kind demo graph: a next chain plus two
                // cross links (accent) and one citation (info).
                let mut edges: Vec<GraphEdge> = (0..11u32)
                    .map(|i| GraphEdge {
                        from: i,
                        to: i + 1,
                        kind: 0,
                    })
                    .collect();
                edges.push(GraphEdge {
                    from: 0,
                    to: 6,
                    kind: 1,
                });
                edges.push(GraphEdge {
                    from: 3,
                    to: 9,
                    kind: 1,
                });
                edges.push(GraphEdge {
                    from: 5,
                    to: 1,
                    kind: 2,
                });
                let mut view = GraphView::new();
                view.set_graph(&GraphSpec { n_nodes: 12, edges });
                view
            },
            filter_on: false,
        }
    }

    /// Flow layout for a row of measured widgets (wraps at content.w).
    fn flow(content: Rect, y: f32, sizes: impl Iterator<Item = (f32, f32)>) -> (Vec<Rect>, f32) {
        let mut rects = Vec::new();
        let mut x = content.x;
        let mut line_h: f32 = 0.0;
        let mut y = y;
        for (w, h) in sizes {
            if x > content.x && x + w > content.x + content.w {
                x = content.x;
                y += line_h + GAP;
                line_h = 0.0;
            }
            rects.push(Rect::new(x, y, w, h));
            x += w + GAP;
            line_h = line_h.max(h);
        }
        (rects, line_h)
    }

    fn layout(&self, content: Rect) -> Layout {
        let mut labels = Vec::new();
        let mut y = content.y;

        labels.push(("CHIPS", y));
        let (chips, line_h) = Self::flow(
            content,
            y + LABEL_H,
            self.chips.iter().map(|c| c.preferred_size()),
        );
        y += LABEL_H + line_h + ROW_GAP;

        labels.push(("ICON BUTTONS", y));
        let (icon_buttons, line_h) = Self::flow(
            content,
            y + LABEL_H,
            self.icon_buttons.iter().map(|b| b.preferred_size()),
        );
        y += LABEL_H + line_h + ROW_GAP;

        labels.push(("SPINNERS", y));
        let spinners = std::array::from_fn(|i| {
            let px = self.spinners[i].size.px();
            Rect::new(content.x + i as f32 * 56.0, y + LABEL_H, 32.0, 32.0).with_square(px)
        });
        let (tw, th) = self.spin_toggle.preferred_size();
        let spin_toggle = Rect::new(
            content.x + 3.0 * 56.0,
            y + LABEL_H - (th - 32.0) / 2.0 - 4.0,
            tw,
            th,
        );
        y += LABEL_H + 32.0 + ROW_GAP;

        labels.push(("EMPTY STATES", y));
        let two_col = content.w >= 800.0;
        let (empty, empty_plain) = if two_col {
            let w = (content.w - GAP) / 2.0;
            (
                Rect::new(content.x, y + LABEL_H, w, PANEL_H),
                Rect::new(content.x + w + GAP, y + LABEL_H, w, PANEL_H),
            )
        } else {
            (
                Rect::new(content.x, y + LABEL_H, content.w, PANEL_H),
                Rect::new(content.x, y + LABEL_H + PANEL_H + GAP, content.w, PANEL_H),
            )
        };
        y += LABEL_H
            + if two_col {
                PANEL_H
            } else {
                PANEL_H * 2.0 + GAP
            }
            + ROW_GAP;

        labels.push(("SPLIT PANE (DRAG THE DIVIDER)", y));
        let split = Rect::new(content.x, y + LABEL_H, content.w, 200.0);
        y += LABEL_H + 200.0 + ROW_GAP;

        labels.push(("GRAPH CANVAS (DRAG / WHEEL / CLICK A NODE)", y));
        let graph = Rect::new(content.x, y + LABEL_H, content.w, 300.0);
        y += LABEL_H + 300.0;

        Layout {
            chips,
            icon_buttons,
            spinners,
            spin_toggle,
            empty,
            empty_plain,
            split,
            graph,
            labels,
            total_h: y - content.y,
        }
    }

    pub fn content_height(&self, content: Rect) -> f32 {
        self.layout(content).total_h + GAP
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let l = self.layout(content);
        let mut result = EventResult::IGNORED;
        for (chip, rect) in self.chips.iter_mut().zip(&l.chips) {
            let r = chip.handle_event(event, *rect);
            if r.clicked {
                chip.selected = !chip.selected;
                self.filter_on = chip.selected;
                return EventResult::clicked();
            }
            result = result.merge(r);
        }
        for (button, rect) in self.icon_buttons.iter_mut().zip(&l.icon_buttons) {
            result = result.merge(button.handle_event(event, *rect));
        }
        let r = self.spin_toggle.handle_event(event, l.spin_toggle);
        if r.clicked {
            self.spinning = !self.spinning;
            return EventResult::clicked();
        }
        result = result.merge(r);
        result = result.merge(self.empty.handle_event(event, l.empty));
        result = result.merge(self.empty_plain.handle_event(event, l.empty_plain));
        result = result.merge(self.split.handle_event(event, l.split));
        // Selection changes re-render (incident edges light up).
        result.merge(self.graph.handle_event(event, l.graph))
    }

    /// Spinners rotate only while toggled on; an idle section must settle
    /// (render-on-demand), so the default state returns `false`.
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.spinning {
            return false;
        }
        for s in &mut self.spinners {
            s.tick(dt);
        }
        true
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content);
        for (label, y) in &l.labels {
            group_label(c, label, content.x, *y, theme);
        }
        for (chip, rect) in self.chips.iter().zip(&l.chips) {
            chip.render(c, *rect, theme);
        }
        for (button, rect) in self.icon_buttons.iter().zip(&l.icon_buttons) {
            button.render(c, *rect, theme);
        }
        for (spinner, rect) in self.spinners.iter().zip(&l.spinners) {
            spinner.render(c, *rect, theme);
        }
        self.spin_toggle.label = if self.spinning {
            "Stop spinners".to_string()
        } else {
            "Start spinners".to_string()
        };
        self.spin_toggle.render(c, l.spin_toggle, theme);

        panel(c, l.empty, theme);
        self.empty.render(c, l.empty, theme);
        panel(c, l.empty_plain, theme);
        self.empty_plain.render(c, l.empty_plain, theme);

        // Split pane demo: two labeled panes + the draggable divider.
        panel(c, l.split, theme);
        let first = self.split.first_rect(l.split);
        let second = self.split.second_rect(l.split);
        text(
            c,
            &format!("first pane — {:.0}px (35% desired)", first.w),
            13.0,
            500,
            first.x + 16.0,
            first.y + 16.0,
            theme.colors.text_mid.0,
        );
        text(
            c,
            "second pane",
            13.0,
            500,
            second.x + 16.0,
            second.y + 16.0,
            theme.colors.text_mid.0,
        );
        self.split.render(c, l.split, theme);

        panel(c, l.graph, theme);
        self.graph.render(c, l.graph, theme);
        if let Some(sel) = self.graph.selected() {
            text(
                c,
                &format!("node {sel} selected"),
                12.0,
                500,
                l.graph.x + 16.0,
                l.graph.y + l.graph.h - 24.0,
                theme.colors.text_mid.0,
            );
        }
        let _ = self.filter_on;
    }
}

/// Rect helper: center a `px`-sized square inside the rect.
trait SquareExt {
    fn with_square(self, px: f32) -> Rect;
}

impl SquareExt for Rect {
    fn with_square(self, px: f32) -> Rect {
        Rect::new(
            self.x + (self.w - px) / 2.0,
            self.y + (self.h - px) / 2.0,
            px,
            px,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extras_render_and_layout_at_narrow_and_wide() {
        let theme = Theme::hoff();
        let mut section = ExtrasSection::new();
        for w in [500.0, 1272.0] {
            let content = Rect::new(288.0, 80.0, w, 900.0);
            let mut c = Compositor::new();
            section.render(&mut c, content, &theme);
            // Nothing overflows the content's right edge.
            let l = section.layout(content);
            for r in l.chips.iter().chain(l.icon_buttons.iter()) {
                assert!(r.x + r.w <= content.x + content.w + 0.5);
            }
            assert!(section.content_height(content) > 0.0);
        }
    }

    #[test]
    fn interactive_chip_toggles_on_click() {
        let mut section = ExtrasSection::new();
        let content = Rect::new(288.0, 80.0, 1272.0, 900.0);
        let l = section.layout(content);
        let chip = l.chips[5]; // "click me"
        let (x, y) = (chip.x + 4.0, chip.y + 4.0);
        let r = section.handle_event(&WidgetEvent::MouseDown { x, y }, content);
        assert!(r.changed);
        let r = section.handle_event(&WidgetEvent::MouseUp { x, y }, content);
        assert!(r.clicked);
        assert!(section.filter_on, "click toggled the demo filter chip");
    }

    #[test]
    fn split_divider_drags() {
        let mut section = ExtrasSection::new();
        let content = Rect::new(288.0, 80.0, 1272.0, 900.0);
        let l = section.layout(content);
        let d = section.split.divider_rect(l.split);
        let (x, y) = (d.x + 1.0, d.y + 20.0);
        section.handle_event(&WidgetEvent::MouseDown { x, y }, content);
        assert!(section.split.is_dragging());
        section.handle_event(&WidgetEvent::MouseMove { x: x + 100.0, y }, content);
        assert!(section.split.ratio() > 0.35);
    }
}
