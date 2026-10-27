//! Graph page: a small three-kind demo graph on the pan/zoom canvas.
//! Drag, wheel, click a node.

use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::graph::{Edge, Graph, Node};
use engine::widgets::prelude::{EventResult, GraphView, Rect, WidgetEvent};

use super::{panel, text};

pub struct GraphPage {
    graph: GraphView,
}

impl GraphPage {
    pub fn new() -> Self {
        // A next chain plus two cross links (accent) and one citation
        // (info).
        let edge = |from, to, kind| Edge {
            kind,
            ..Edge::new(from, to)
        };
        let mut edges: Vec<Edge> = (0..11).map(|i| edge(i, i + 1, 0)).collect();
        edges.extend([edge(0, 6, 1), edge(3, 9, 1), edge(5, 1, 2)]);
        let data = Graph {
            nodes: (0..12).map(|i| Node::new(i.to_string())).collect(),
            edges,
            groups: Vec::new(),
        };
        let mut graph = GraphView::new();
        graph.set_graph(&data);
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
