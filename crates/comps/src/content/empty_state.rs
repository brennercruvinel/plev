//! Empty state: the standard "nothing here yet" block — optional icon,
//! title, wrapped message, optional CTA button — centered in the given
//! bounds. Content-driven: every line is really measured, the stack is
//! centered as a unit, and the message wraps against the available width
//! (clamped by a readability maximum, never a viewport constant).

use crate::icons;
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{IconSize, Theme};

use crate::action::Button;
use crate::core::{EventResult, Rect, WidgetEvent};

/// The message column never exceeds this share of the bounds (long lines
/// wrap instead of running edge to edge), and never the readable maximum.
const MESSAGE_WIDTH_FRAC: f32 = 0.72;

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

    fn title_style(theme: &Theme) -> TextStyle {
        theme.typography.base_m()
    }

    fn message_style(theme: &Theme) -> TextStyle {
        theme.typography.base_2r()
    }

    /// Gap between the icon/title/message/CTA blocks: the `lg` step.
    fn gap(theme: &Theme) -> f32 {
        theme.spacing.lg
    }

    /// Icon glyph edge: the large icon size.
    fn icon_size(theme: &Theme) -> f32 {
        theme.control.icon(IconSize::Lg)
    }

    /// Column width for the wrapped message.
    fn message_width(&self, bounds: Rect, theme: &Theme) -> f32 {
        (bounds.w * MESSAGE_WIDTH_FRAC).min(theme.size.readable_max_w)
    }

    /// Total stack height for the current content and bounds.
    fn content_height(&self, bounds: Rect, theme: &Theme) -> f32 {
        let gap = Self::gap(theme);
        let mut h = 0.0;
        if self.icon.is_some() {
            h += Self::icon_size(theme) + gap;
        }
        h += Self::title_style(theme).line_height + gap;
        if !self.message.is_empty() {
            let (_, mh) = TextMeasurer::measure_styled(
                &self.message,
                &Self::message_style(theme),
                Some(self.message_width(bounds, theme)),
            );
            h += mh + gap;
        }
        if let Some(cta) = &self.cta {
            h += cta.preferred_size(theme).1;
        }
        h
    }

    /// CTA rect, centered in the stack. Shared by hit testing and render;
    /// public so owners/tests can hit-test or anchor tooltips to it.
    pub fn cta_rect(&self, bounds: Rect, theme: &Theme) -> Option<Rect> {
        let cta = self.cta.as_ref()?;
        let (w, h) = cta.preferred_size(theme);
        let stack_h = self.content_height(bounds, theme);
        let top = bounds.y + (bounds.h - stack_h) / 2.0;
        Some(Rect::new(
            bounds.x + (bounds.w - w) / 2.0,
            top + stack_h - h,
            w,
            h,
        ))
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rect = self.cta_rect(bounds, theme);
        match (self.cta.as_mut(), rect) {
            (Some(cta), Some(rect)) => cta.handle_event(event, rect),
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let gap = Self::gap(theme);
        let icon_size = Self::icon_size(theme);
        let stack_h = self.content_height(bounds, theme);
        let mut y = bounds.y + (bounds.h - stack_h) / 2.0;

        if let Some(name) = self.icon
            && let Some(node) = icons::icon_at(
                name,
                icon_size,
                theme.glass.text_placeholder.0,
                bounds.x + (bounds.w - icon_size) / 2.0,
                y,
            )
        {
            compositor.push(node);
            y += icon_size + gap;
        }

        let title_style = Self::title_style(theme);
        let (tw, _) = TextMeasurer::measure_styled(&self.title, &title_style, None);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(&self.title, &title_style, None),
            x: bounds.x + (bounds.w - tw) / 2.0,
            y,
            color: theme.colors.text.0,
        });
        y += title_style.line_height + gap;

        if !self.message.is_empty() {
            let style = Self::message_style(theme);
            let mw = self.message_width(bounds, theme);
            let (w, mh) = TextMeasurer::measure_styled(&self.message, &style, Some(mw));
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(&self.message, &style, Some(mw)),
                x: bounds.x + (bounds.w - w.min(mw)) / 2.0,
                y,
                color: theme.colors.text_dim.0,
            });
            let _ = mh;
        }

        if let Some(rect) = self.cta_rect(bounds, theme)
            && let Some(cta) = &self.cta
        {
            cta.render(compositor, rect, theme);
        }
    }
}
