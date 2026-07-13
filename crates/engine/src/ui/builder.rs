use crate::layout::{Align, Direction, Justify, LayoutEngine, LayoutStyle};
use crate::text::{TextMeasurer, TextStyle};

use super::modifier::NodeRef;
use super::node::{UiHitRect, UiNode, Visual};
use super::theme::{Accent, UiTheme};

pub struct Ui {
    pub(super) nodes: Vec<UiNode>,
    pub(super) stack: Vec<usize>, // indices into nodes (parent chain)
    pub(super) theme: UiTheme,
    pub(super) hit_rects: Vec<UiHitRect>,
    pub(super) engine: LayoutEngine,
}

impl Ui {
    pub fn new(theme: UiTheme) -> Self {
        let root = UiNode {
            layout: LayoutStyle {
                direction: Direction::Column,
                flex_grow: 1.0,
                ..Default::default()
            },
            visual: Visual::None,
            children: Vec::new(),
            click_id: None,
        };
        Self {
            nodes: vec![root],
            stack: vec![0],
            theme,
            hit_rects: Vec::new(),
            engine: LayoutEngine::new(),
        }
    }

    pub fn theme(&self) -> &UiTheme {
        &self.theme
    }

    // -- Container builders --

    pub fn hstack(&mut self, f: impl FnOnce(&mut Self)) -> NodeRef {
        self.container(Direction::Row, f)
    }

    pub fn vstack(&mut self, f: impl FnOnce(&mut Self)) -> NodeRef {
        self.container(Direction::Column, f)
    }

    fn container(&mut self, dir: Direction, f: impl FnOnce(&mut Self)) -> NodeRef {
        let idx = self.push_node(UiNode {
            layout: LayoutStyle {
                direction: dir,
                ..Default::default()
            },
            visual: Visual::None,
            children: Vec::new(),
            click_id: None,
        });
        self.stack.push(idx);
        f(self);
        self.stack.pop();
        NodeRef { idx }
    }

    fn push_node(&mut self, node: UiNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        if let Some(&parent) = self.stack.last() {
            self.nodes[parent].children.push(idx);
        }
        idx
    }

    // -- Leaf builders --

    pub fn spacer(&mut self) {
        self.push_node(UiNode {
            layout: LayoutStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
            visual: Visual::None,
            children: Vec::new(),
            click_id: None,
        });
    }

    pub fn text(&mut self, content: &str) -> NodeRef {
        let color = self.theme.text[0];
        let size = 13.0;
        let lh = size * 1.4;
        let style = TextStyle::new(size).with_line_height(lh);
        let (w, _) = TextMeasurer::measure_styled(content, &style, None);
        let w = w.ceil();
        let idx = self.push_node(UiNode {
            layout: LayoutStyle {
                width: Some(w),
                height: Some(lh),
                flex_shrink: 0.0,
                ..Default::default()
            },
            visual: Visual::Text {
                content: content.to_string(),
                size,
                line_height: lh,
                weight: 400,
                color,
                family: None,
            },
            children: Vec::new(),
            click_id: None,
        });
        NodeRef { idx }
    }

    pub fn icon(&mut self, codepoint: &str) -> NodeRef {
        let color = self.theme.text[1];
        let size = 16.0;
        let idx = self.push_node(UiNode {
            layout: LayoutStyle {
                width: Some(size),
                height: Some(size),
                flex_shrink: 0.0,
                ..Default::default()
            },
            visual: Visual::Text {
                content: codepoint.to_string(),
                size,
                line_height: size,
                weight: 400,
                color,
                family: Some("codicon".to_string()),
            },
            children: Vec::new(),
            click_id: None,
        });
        NodeRef { idx }
    }

    pub fn badge(&mut self, label: &str) -> NodeRef {
        let accent = Accent::Gray;
        let bg = self.theme.accent_bg[UiTheme::accent_idx(accent)];
        let fg = self.theme.accent_fg[UiTheme::accent_idx(accent)];
        let style = TextStyle::new(11.0).with_weight(700).with_line_height(14.0);
        let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
        let text_w = text_w.ceil();
        let h = 20.0;

        let idx = self.push_node(UiNode {
            layout: LayoutStyle {
                direction: Direction::Row,
                align: Align::Center,
                justify: Justify::Center,
                padding: [2.0, 8.0, 2.0, 8.0],
                height: Some(h),
                flex_shrink: 0.0,
                ..Default::default()
            },
            visual: Visual::Box {
                bg,
                border_color: [0.0; 4],
                border_width: 0.0,
                corner_radius: h / 2.0,
            },
            children: Vec::new(),
            click_id: None,
        });

        // Text child
        self.stack.push(idx);
        self.push_node(UiNode {
            layout: LayoutStyle {
                width: Some(text_w),
                height: Some(14.0),
                flex_shrink: 0.0,
                ..Default::default()
            },
            visual: Visual::Text {
                content: label.to_string(),
                size: 11.0,
                line_height: 14.0,
                weight: 700,
                color: fg,
                family: None,
            },
            children: Vec::new(),
            click_id: None,
        });
        self.stack.pop();

        NodeRef { idx }
    }

    pub fn button(&mut self, label: &str) -> NodeRef {
        let accent = Accent::Gray;
        let bg = self.theme.accent_bg[UiTheme::accent_idx(accent)];
        let fg = self.theme.accent_fg[UiTheme::accent_idx(accent)];
        let style = TextStyle::new(12.0).with_weight(600).with_line_height(16.0);
        let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
        let text_w = text_w.ceil();

        let idx = self.push_node(UiNode {
            layout: LayoutStyle {
                direction: Direction::Row,
                align: Align::Center,
                justify: Justify::Center,
                padding: [4.0, 8.0, 4.0, 8.0],
                height: Some(28.0),
                flex_shrink: 0.0,
                ..Default::default()
            },
            visual: Visual::Box {
                bg,
                border_color: [0.0; 4],
                border_width: 0.0,
                corner_radius: 6.0,
            },
            children: Vec::new(),
            click_id: None,
        });

        self.stack.push(idx);
        self.push_node(UiNode {
            layout: LayoutStyle {
                width: Some(text_w),
                height: Some(16.0),
                flex_shrink: 0.0,
                ..Default::default()
            },
            visual: Visual::Text {
                content: label.to_string(),
                size: 12.0,
                line_height: 16.0,
                weight: 600,
                color: fg,
                family: None,
            },
            children: Vec::new(),
            click_id: None,
        });
        self.stack.pop();

        NodeRef { idx }
    }

    pub fn separator(&mut self) {
        self.push_node(UiNode {
            layout: LayoutStyle {
                height: Some(1.0),
                ..Default::default()
            },
            visual: Visual::Box {
                bg: self.theme.border,
                border_color: [0.0; 4],
                border_width: 0.0,
                corner_radius: 0.0,
            },
            children: Vec::new(),
            click_id: None,
        });
    }

    pub fn rect(&mut self) -> NodeRef {
        let idx = self.push_node(UiNode {
            layout: LayoutStyle::default(),
            visual: Visual::Box {
                bg: [0.0; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                corner_radius: 0.0,
            },
            children: Vec::new(),
            click_id: None,
        });
        NodeRef { idx }
    }
}
