//! Graph page: a small three-kind demo graph on the pan/zoom canvas.
//! Drag, wheel, click a node.

use comps::prelude::{EventResult, GraphView, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::graph::{GraphEdge, GraphSpec};
use engine::theme::Theme;

use super::{panel, text};

pub struct GraphPage {
    graph: GraphView,
}

impl GraphPage {
    pub fn new() -> Self {
        // A next chain plus two cross links (accent) and one citation
        // (info).
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
        let mut graph = GraphView::new();
        graph.set_graph(&GraphSpec { n_nodes: 12, edges });
        Self { graph }
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        Rect::new(content.x, content.y, content.w, theme.size.card_min_w * 1.5)
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.size.card_min_w * 1.5
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        // Selection changes re-render (incident edges light up).
        self.graph.handle_event(event, Self::rect(content, theme))
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        panel(c, rect, theme);
        self.graph.render(c, rect, theme);
        if let Some(sel) = self.graph.selected() {
            let style = theme.typography.caption_r();
            text(
                c,
                &format!("node {sel} selected"),
                style.font_size,
                style.font_weight,
                rect.x + theme.spacing.lg,
                rect.bottom() - theme.spacing.lg - style.line_height,
                theme.colors.text_mid.0,
            );
        }
    }
}
