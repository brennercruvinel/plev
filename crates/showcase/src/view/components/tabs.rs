//! Tabs page: the segmented strip, sized from its measured labels so
//! every one keeps its breathing room. Tab focuses the strip.

use comps::prelude::{EventResult, Rect, Tabs, WidgetEvent};
use engine::compositor::Compositor;
use engine::text::TextMeasurer;
use engine::theme::{Theme, TypographyScale};

use super::EditKey;

/// Horizontal slack around each label inside its segment: the strip's
/// container padding plus the breathing room the reference asks for.
const SEGMENT_SLACK: f32 = 48.0;

pub struct TabsPage {
    tabs: Tabs,
}

impl TabsPage {
    pub fn new() -> Self {
        Self {
            tabs: Tabs::new(["Account", "Appearance", "About"]),
        }
    }

    /// Strip width from the measured labels (base-2sm, the style
    /// `Tabs::render` uses), clamped to `max_w`.
    pub fn strip_w(&self, max_w: f32) -> f32 {
        let style = TypographyScale::hoff().base_2sm();
        let widest = self
            .tabs
            .labels
            .iter()
            .map(|l| TextMeasurer::measure_styled(l, &style, None).0)
            .fold(0.0, f32::max);
        ((widest + SEGMENT_SLACK) * self.tabs.labels.len() as f32).min(max_w)
    }

    fn rect(&self, content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            self.strip_w(content.w),
            Tabs::height(theme),
        )
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        Tabs::height(theme)
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            self.tabs.set_focused(true);
            return true;
        }
        false
    }

    pub fn handle_escape(&mut self) -> bool {
        if self.tabs.is_focused() {
            self.tabs.set_focused(false);
            true
        } else {
            false
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rect = self.rect(content, theme);
        self.tabs.handle_event(event, rect, theme)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = self.rect(content, theme);
        self.tabs.render(c, rect, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_label_keeps_its_slack_in_its_segment() {
        let theme = Theme::hoff();
        let page = TabsPage::new();
        let content = Rect::new(288.0, 80.0, 760.0, 700.0);
        let strip = page.rect(content, &theme);
        let rects = page.tabs.item_rects(strip, &theme);
        assert_eq!(rects.len(), page.tabs.labels.len());
        let style = TypographyScale::hoff().base_2sm();
        for (label, r) in page.tabs.labels.iter().zip(rects) {
            let (w, _) = TextMeasurer::measure_styled(label, &style, None);
            assert!(
                r.w - w >= 32.0,
                "{label} is cramped: {:.1}px slack",
                r.w - w
            );
        }
    }
}
