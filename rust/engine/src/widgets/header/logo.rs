//! The logo of the kit (`components/Logo`): the symbol, the wordmark, or
//! both side by side, inside the link home. The symbol is an icon of the
//! set and the wordmark the product's name in the logo's face, both
//! `theme.header.logo`; the kit's own vectors are a third party's brand
//! and stay in the kit. A product puts its own mark in two places: the
//! name ([`Logo::new`]) and the symbol, the theme's
//! (`theme.header.logo.symbol`) or this logo's ([`Logo::symbol`]).
//!
//! With a [`SemId`] ([`Logo::sem`]) the render binds its bounds;
//! [`Logo::project`] declares the link, named by the product.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::icons;
use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// What the logo shows (`onlyIcon`, `onlyText`, or neither).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogoKind {
    #[default]
    Full,
    Symbol,
    Wordmark,
}

#[derive(Clone, Debug)]
pub struct Logo {
    /// The product the wordmark spells and the link is named by.
    pub name: String,
    pub kind: LogoKind,
    /// The icon drawn as the symbol, over the theme's.
    pub symbol: Option<&'static str>,
    press: Press,
    sem: Option<SemId>,
}

impl Logo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: LogoKind::Full,
            symbol: None,
            press: Press::default(),
            sem: None,
        }
    }

    pub fn kind(mut self, kind: LogoKind) -> Self {
        self.kind = kind;
        self
    }

    /// Draws `icon` as the symbol instead of the theme's.
    pub fn symbol(mut self, icon: &'static str) -> Self {
        self.symbol = Some(icon);
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.press.focused = focused;
    }

    pub fn is_hovered(&self) -> bool {
        self.press.hovered
    }

    /// The wordmark's size as the face measures it.
    fn word(&self, theme: &Theme) -> (f32, f32) {
        let l = &theme.header.logo;
        let (w, h) = TextMeasurer::measure_styled(&self.name, &l.word, None);
        (w.ceil(), h.max(l.word.line_height))
    }

    /// The link's box: the padding around the symbol, the gap and the
    /// wordmark, as tall as the taller of them.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let l = &theme.header.logo;
        let (ww, wh) = self.word(theme);
        let (w, h) = match self.kind {
            LogoKind::Full => (l.symbol_size + l.gap + ww, l.symbol_size.max(wh)),
            LogoKind::Symbol => (l.symbol_size, l.symbol_size),
            LogoKind::Wordmark => (ww, wh),
        };
        (w + l.pad * 2.0, h + l.pad * 2.0)
    }

    /// A click on it is `clicked`: the owner goes home.
    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.press.handle(event, bounds, false)
    }

    /// Declares the link as `id`: named by the product, with `activate`
    /// (what it opens is the app's).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Link, self.name.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.action("activate");
        node
    }

    /// On the layer the compositor aims at ([`Compositor::target`]).
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, c.target(), bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let l = &theme.header.logo;
        let inner = Rect::new(
            bounds.x + l.pad,
            bounds.y + l.pad,
            (bounds.w - l.pad * 2.0).max(0.0),
            (bounds.h - l.pad * 2.0).max(0.0),
        );
        let mut x = inner.x;
        if self.kind != LogoKind::Wordmark {
            let s = l.symbol_size;
            let y = inner.y + (inner.h - s) / 2.0;
            if let Some(node) = icons::icon_at(self.symbol.unwrap_or(l.symbol), s, l.ink.0, x, y) {
                c.push_to_layer(layer, node);
            }
            x += s + l.gap;
        }
        if self.kind != LogoKind::Symbol {
            let (_, h) = self.word(theme);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&self.name, &l.word, None),
                    x,
                    y: inner.y + (inner.h - h) / 2.0,
                    color: l.ink.0,
                },
            );
        }
        if self.press.focused {
            c.push_to_layer(layer, focus_ring(bounds, theme.shape.nav, theme));
        }
    }
}
