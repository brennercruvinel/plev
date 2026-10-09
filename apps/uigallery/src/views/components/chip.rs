//! Chip page: selected, per intent, and one chip that toggles on click
//! (a filter chip, `selectable`: the widget flips itself).
//!
//! Projected (task-75 phase 2b): each chip, `chip_page.chip[<i>]`, as the
//! comps projects it: the tags are labels, the last one a toggle whose
//! `toggle` flips it as a click does.

use engine::widgets::prelude::{Chip, EventResult, Rect, WidgetEvent};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Intent, Theme};
use accord::{Request, SemId, SnapshotBuilder};

use super::{After, Component, child_id, flow, root_id, under};

pub struct ChipPage {
    root: Option<SemId>,
    chips: Vec<Chip>,
    /// The chip a screen reader or the keyboard focused (its ring shows);
    /// Escape or a click lets it go.
    focus: Option<usize>,
}

impl ChipPage {
    pub fn new() -> Self {
        let root = root_id("chip_page");
        let family = child_id(root.as_ref(), "chip");
        let chips = vec![
            Chip::new("exact"),
            Chip::new("ann").selected(true),
            Chip::new("bm25").intent(Intent::Constructive),
            Chip::new("missing").intent(Intent::Destructive),
            Chip::new("beta").intent(Intent::Informational),
            Chip::new("click me").selectable(true),
        ];
        let chips = chips
            .into_iter()
            .enumerate()
            .map(|(i, c)| match &family {
                Some(f) => c.sem(f.key(i.to_string())),
                None => c,
            })
            .collect();
        Self {
            root,
            chips,
            focus: None,
        }
    }

    /// The chip behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.chips.iter().position(|c| c.sem_id() == Some(id))
    }

    /// The interactive chip's state, for the tests.
    #[cfg(test)]
    pub fn toggled(&self) -> bool {
        self.chips[5].selected
    }

    /// The focused chip rings, the others do not.
    fn ring(&mut self) {
        for (i, chip) in self.chips.iter_mut().enumerate() {
            chip.set_focused(self.focus == Some(i));
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            self.chips.iter().map(|c| c.preferred_size(theme)),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).1
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let (rects, _) = self.layout(content, theme);
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        for (chip, rect) in self.chips.iter_mut().zip(rects) {
            let r = chip.handle_event(event, rect);
            if r.clicked {
                return EventResult::clicked();
            }
            result = result.merge(r);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (rects, _) = self.layout(content, theme);
        for (chip, rect) in self.chips.iter().zip(rects) {
            chip.render(c, rect, theme);
        }
    }
}

impl Component for ChipPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for chip in &self.chips {
            if let Some(id) = chip.sem_id() {
                chip.project(b, id, Some(root));
            }
        }
        let focused = self.focus.and_then(|i| self.chips.get(i));
        if let Some(id) = focused.and_then(Chip::sem_id) {
            b.focus(id);
        }
    }

    /// A filter chip's `toggle` flips it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(i), "toggle") if self.chips[i].selectable => {
                self.chips[i].selected = !self.chips[i].selected;
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the chip page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.focus = self.at(id).filter(|i| self.chips[*i].selectable);
        self.ring();
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.take().is_some();
        self.ring();
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_chip_toggles_on_click() {
        let theme = Theme::hoff();
        let mut page = ChipPage::new();
        let content = Rect::new(288.0, 80.0, 1272.0, 900.0);
        let (rects, _) = page.layout(content, &theme);
        let chip = rects[5];
        let (x, y) = (chip.x + 4.0, chip.y + 4.0);
        let r = page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(r.changed);
        let r = page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert!(r.clicked);
        assert!(page.toggled());
    }
}
