//! Empty state: the standard "nothing here yet" block — optional icon,
//! title, wrapped message, optional CTA button — centered in the given
//! bounds. Content-driven: every line is really measured, the stack is
//! centered as a unit, and the message wraps against the available width
//! (clamped by a readability maximum, never a viewport constant).

use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Theme, TypographyScale};
use crate::ui::icons;

use super::{Button, EventResult, Rect, WidgetEvent};

/// Vertical gap between the icon/title/message/CTA blocks.
const GAP: f32 = 16.0;
/// The message column never exceeds this share of the bounds (long lines
/// wrap instead of running edge to edge).
const MESSAGE_WIDTH_FRAC: f32 = 0.72;
/// Icon glyph size.
const ICON: f32 = 28.0;

/// Centered empty-state block. The CTA is a retained [`Button`]; events
/// delegate to it (click fires on release inside, like every button).
#[derive(Clone, Debug)]
pub struct EmptyState {
    pub icon: Option<&'static str>,
    pub title: String,
    pub message: String,
    pub cta: Option<Button>,
}

impl EmptyState {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            icon: None,
            title: title.into(),
            message: message.into(),
            cta: None,
        }
    }

    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(name);
        self
    }

    pub fn cta(mut self, button: Button) -> Self {
        self.cta = Some(button);
        self
    }

    fn title_style() -> TextStyle {
        TypographyScale::hoff().base_m()
    }

    fn message_style() -> TextStyle {
        TypographyScale::hoff().base_2r()
    }

    /// Column width for the wrapped message.
    fn message_width(&self, bounds: Rect) -> f32 {
        bounds.w * MESSAGE_WIDTH_FRAC
    }

    /// Total stack height for the current content and bounds.
    fn content_height(&self, bounds: Rect) -> f32 {
        let mut h = 0.0;
        if self.icon.is_some() {
            h += ICON + GAP;
        }
        h += Self::title_style().line_height + GAP;
        if !self.message.is_empty() {
            let (_, mh) = TextMeasurer::measure_styled(
                &self.message,
                &Self::message_style(),
                Some(self.message_width(bounds)),
            );
            h += mh + GAP;
        }
        if let Some(cta) = &self.cta {
            h += cta.preferred_size().1;
        }
        h
    }

    /// CTA rect, centered in the stack. Shared by hit testing and render;
    /// public so owners/tests can hit-test or anchor tooltips to it.
    pub fn cta_rect(&self, bounds: Rect) -> Option<Rect> {
        let cta = self.cta.as_ref()?;
        let (w, h) = cta.preferred_size();
        let stack_h = self.content_height(bounds);
        let top = bounds.y + (bounds.h - stack_h) / 2.0;
        Some(Rect::new(
            bounds.x + (bounds.w - w) / 2.0,
            top + stack_h - h,
            w,
            h,
        ))
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        let rect = self.cta_rect(bounds);
        match (self.cta.as_mut(), rect) {
            (Some(cta), Some(rect)) => cta.handle_event(event, rect),
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let stack_h = self.content_height(bounds);
        let mut y = bounds.y + (bounds.h - stack_h) / 2.0;

        if let Some(name) = self.icon
            && let Some(node) = icons::icon_at(
                name,
                ICON,
                theme.glass.text_placeholder.0,
                bounds.x + (bounds.w - ICON) / 2.0,
                y,
            )
        {
            compositor.push(node);
            y += ICON + GAP;
        }

        let title_style = Self::title_style();
        let (tw, _) = TextMeasurer::measure_styled(&self.title, &title_style, None);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(&self.title, &title_style, None),
            x: bounds.x + (bounds.w - tw) / 2.0,
            y,
            color: theme.colors.text.0,
        });
        y += title_style.line_height + GAP;

        if !self.message.is_empty() {
            let style = Self::message_style();
            let mw = self.message_width(bounds);
            let (w, mh) = TextMeasurer::measure_styled(&self.message, &style, Some(mw));
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(&self.message, &style, Some(mw)),
                x: bounds.x + (bounds.w - w.min(mw)) / 2.0,
                y,
                color: theme.colors.text_dim.0,
            });
            let _ = mh;
        }

        if let Some(rect) = self.cta_rect(bounds)
            && let Some(cta) = &self.cta
        {
            cta.render(compositor, rect, theme);
        }
    }
}
