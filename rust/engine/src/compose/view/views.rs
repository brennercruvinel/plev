//! Built-in View implementations: ContainerView, RectView, TextView.

use crate::compositor::{SceneNode, TextNodeKey};
use crate::layout::LayoutStyle;

use super::context::ViewContext;
use super::trait_def::View;

// ---------------------------------------------------------------------------
// ContainerView -- applies layout to children, optional background
// ---------------------------------------------------------------------------

pub struct ContainerView {
    pub style: LayoutStyle,
    pub children: Vec<Box<dyn View>>,
    pub background: Option<[f32; 4]>,
}

impl View for ContainerView {
    fn layout(&self) -> LayoutStyle {
        self.style.clone()
    }

    fn children(&self) -> &[Box<dyn View>] {
        &self.children
    }

    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode> {
        if let Some(color) = self.background {
            vec![SceneNode::Rect {
                x: cx.bounds.x,
                y: cx.bounds.y,
                w: cx.bounds.width,
                h: cx.bounds.height,
                color,
            }]
        } else {
            vec![]
        }
    }
}

// ---------------------------------------------------------------------------
// RectView -- a single colored rectangle
// ---------------------------------------------------------------------------

pub struct RectView {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
}

impl View for RectView {
    fn layout(&self) -> LayoutStyle {
        LayoutStyle {
            width: Some(self.w),
            height: Some(self.h),
            ..Default::default()
        }
    }

    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode> {
        vec![SceneNode::Rect {
            x: cx.bounds.x,
            y: cx.bounds.y,
            w: cx.bounds.width,
            h: cx.bounds.height,
            color: self.color,
        }]
    }
}

// ---------------------------------------------------------------------------
// TextView -- a single text node
// ---------------------------------------------------------------------------

pub struct TextView {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub max_width: Option<f32>,
    pub x: f32,
    pub y: f32,
    pub color: [f32; 4],
}

impl View for TextView {
    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode> {
        vec![SceneNode::Text {
            key: TextNodeKey::new(
                &self.text,
                self.font_size,
                self.line_height,
                self.max_width.or(Some(cx.bounds.width)),
            ),
            x: cx.bounds.x,
            y: cx.bounds.y,
            color: self.color,
        }]
    }
}
