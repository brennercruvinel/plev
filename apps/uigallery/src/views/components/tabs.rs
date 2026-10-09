//! Tabs page: the segmented strip, sized from its measured labels so
//! every one keeps its breathing room.
//!
//! Projected (task-75 phase 2b): the strip, `tabs_page.tabs`, named
//! "sections", with a tab per label, as the comps projects them; a tab's
//! `select` makes it the active one as a click does. The focus is on the
//! tab a screen reader or the keyboard moved it to (its ring, task-75
//! phase 2c), which the arrow keys move along the strip.

use accord::{Request, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::{Theme, TypographyScale};
use engine::widgets::prelude::{EventResult, Rect, Tabs, WidgetEvent};

use super::{After, Component, child_id, key_under, root_id, under};

/// Horizontal slack around each label inside its segment: the strip's
/// container padding plus the breathing room the reference asks for.
const SEGMENT_SLACK: f32 = 48.0;

pub struct TabsPage {
    root: Option<SemId>,
    tabs: Tabs,
    /// The tab a screen reader or the keyboard moved the focus to.
    tab: Option<usize>,
}

impl TabsPage {
    pub fn new() -> Self {
        let root = root_id("tabs_page");
        let tabs = Tabs::new(["Account", "Appearance", "About"]);
        let tabs = match child_id(root.as_ref(), "tabs") {
            Some(id) => tabs.sem(id),
            None => tabs,
        };
        Self {
            root,
            tabs,
            tab: None,
        }
    }

    /// The tab behind `id`, `tabs_page.tabs[<i>]`.
    fn tab_at(&self, id: &SemId) -> Option<usize> {
        let key = key_under(self.tabs.sem_id()?, id)?;
        key.parse().ok().filter(|i| *i < self.tabs.labels.len())
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

    pub fn handle_escape(&mut self) -> bool {
        self.tabs.set_focused_tab(None);
        self.tab.take().is_some()
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

impl Component for TabsPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(id) = self.tabs.sem_id() else {
            return;
        };
        self.tabs.project(b, id, "sections", Some(root));
        if let Some(at) = self.tab {
            b.focus(id.key(at.to_string()));
        }
    }

    /// A tab's `select` makes it the active one, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.tab_at(&req.target), req.action.as_str()) {
            (Some(i), "select") => {
                self.tabs.active = i;
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the tabs page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// A tab takes the focus (its ring); another node of the page takes
    /// it from the strip.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.tab = self.tab_at(id);
        self.tabs.set_focused_tab(self.tab);
        true
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
