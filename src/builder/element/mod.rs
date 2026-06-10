mod builders;

use super::style::*;
use crate::view::ViewContext;

pub struct ClickEvent {
    pub x: f32,
    pub y: f32,
}

pub struct HoverEvent {
    pub x: f32,
    pub y: f32,
}

pub struct KeyEvent;
pub struct FocusEvent;
pub struct ScrollEvent;

pub type ClickHandler = Option<Box<dyn FnMut(&ClickEvent)>>;
pub type HoverHandler = Option<Box<dyn FnMut(&HoverEvent)>>;
pub type KeyHandler = Option<Box<dyn FnMut(&KeyEvent)>>;
pub type FocusHandler = Option<Box<dyn FnMut(&FocusEvent)>>;
pub type ScrollHandler = Option<Box<dyn FnMut(&ScrollEvent)>>;

#[derive(Default)]
pub struct EventHandlers {
    pub on_click: ClickHandler,
    pub on_hover: HoverHandler,
    pub on_key: KeyHandler,
    pub on_focus: FocusHandler,
    pub on_blur: FocusHandler,
    pub on_scroll: ScrollHandler,
}

pub(crate) enum ElementKind {
    Div,
    Text {
        content: String,
        font_size: f32,
        line_height: f32,
        max_width: Option<f32>,
        truncate_chars: Option<usize>,
    },
    Path {
        data: crate::path::TessellatedPath,
    },
}

pub struct Element {
    pub(crate) kind: ElementKind,
    pub(crate) style: Style,
    pub(crate) layout: LayoutConfig,
    #[allow(dead_code)]
    pub(crate) events: EventHandlers,
    pub(crate) children: Vec<Element>,
    pub(crate) intent: Option<crate::theme::Intent>,
}

pub trait IntoView {
    fn into_view(self) -> Element;
}

impl IntoView for Element {
    fn into_view(self) -> Element {
        self
    }
}

impl IntoView for &str {
    fn into_view(self) -> Element {
        text(self)
    }
}

impl IntoView for String {
    fn into_view(self) -> Element {
        text(&self)
    }
}

pub fn div() -> Element {
    Element {
        kind: ElementKind::Div,
        style: Style::default(),
        layout: LayoutConfig::default(),
        events: EventHandlers::default(),
        children: Vec::new(),
        intent: None,
    }
}

pub fn text(content: &str) -> Element {
    Element {
        kind: ElementKind::Text {
            content: content.to_string(),
            font_size: 16.0,
            line_height: 20.8,
            max_width: None,
            truncate_chars: None,
        },
        style: Style::default(),
        layout: LayoutConfig::default(),
        events: EventHandlers::default(),
        children: Vec::new(),
        intent: None,
    }
}

pub fn button(label: &str) -> Element {
    use crate::color::Color;
    div()
        .bg(Color::rgba(0.25, 0.25, 0.35, 1.0))
        .rounded(4.0)
        .p(8.0)
        .child(text(label).font_size(14.0))
}

pub fn path(data: crate::path::TessellatedPath) -> Element {
    Element {
        kind: ElementKind::Path { data },
        style: Style::default(),
        layout: LayoutConfig::default(),
        events: EventHandlers::default(),
        children: Vec::new(),
        intent: None,
    }
}

pub fn image() -> Element {
    div()
}

pub fn spacer() -> Element {
    div().grow(1.0)
}

pub struct Scope<'a> {
    #[allow(dead_code)]
    pub cx: &'a mut ViewContext,
}

impl<'a> From<&'a mut ViewContext> for Scope<'a> {
    fn from(cx: &'a mut ViewContext) -> Self {
        Scope { cx }
    }
}
