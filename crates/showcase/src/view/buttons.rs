//! Buttons section: every variant x size x intent x state.

use plev::compositor::Compositor;
use plev::theme::{Intent, Theme};
use plev::ui::widgets::{Button, ButtonSize, ButtonVariant, EventResult, Rect, WidgetEvent};

use super::group_label;

const GAP: f32 = 12.0;
const ROW_GAP: f32 = 30.0;
const LABEL_H: f32 = 24.0;

pub struct ButtonsSection {
    groups: Vec<(&'static str, Vec<Button>)>,
}

impl ButtonsSection {
    pub fn new() -> Self {
        let variants = vec![
            Button::new("Solid"),
            Button::new("Outline").variant(ButtonVariant::Outline),
            Button::new("Ghost").variant(ButtonVariant::Ghost),
            Button::new("Danger").variant(ButtonVariant::Danger),
        ];
        let sizes = vec![
            Button::new("Small").size(ButtonSize::Sm),
            Button::new("Medium").size(ButtonSize::Md),
            Button::new("Large").size(ButtonSize::Lg),
        ];
        let intents = vec![
            Button::new("Neutral"),
            Button::new("Constructive").intent(Intent::Constructive),
            Button::new("Destructive").intent(Intent::Destructive),
            Button::new("Informational").intent(Intent::Informational),
        ];
        let disabled = vec![
            Button::new("Solid").disabled(true),
            Button::new("Outline")
                .variant(ButtonVariant::Outline)
                .disabled(true),
            Button::new("Ghost")
                .variant(ButtonVariant::Ghost)
                .disabled(true),
            Button::new("Danger")
                .variant(ButtonVariant::Danger)
                .disabled(true),
        ];
        let iconic = vec![
            Button::new("Save").icon("save"),
            Button::new("Run").icon("play").intent(Intent::Constructive),
            Button::new("Search")
                .variant(ButtonVariant::Outline)
                .icon("search"),
            Button::new("Delete")
                .variant(ButtonVariant::Danger)
                .icon("trash"),
        ];
        Self {
            groups: vec![
                ("VARIANTS", variants),
                ("SIZES", sizes),
                ("INTENTS (SOLID)", intents),
                ("DISABLED", disabled),
                ("WITH ICONS", iconic),
            ],
        }
    }

    /// Per-button rects, parallel to `groups`.
    fn layout(&self, content: Rect) -> Vec<Vec<Rect>> {
        let mut all = Vec::with_capacity(self.groups.len());
        let mut y = content.y;
        for (_, group) in &self.groups {
            y += LABEL_H;
            let mut row = Vec::with_capacity(group.len());
            let mut x = content.x;
            let mut max_h: f32 = 0.0;
            for button in group {
                let (w, h) = button.preferred_size();
                row.push(Rect::new(x, y, w, h));
                x += w + GAP;
                max_h = max_h.max(h);
            }
            // Center buttons of differing heights on the row baseline.
            for r in &mut row {
                r.y += (max_h - r.h) / 2.0;
            }
            all.push(row);
            y += max_h + ROW_GAP;
        }
        all
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let rects = self.layout(content);
        let mut result = EventResult::IGNORED;
        for (group, row) in self.groups.iter_mut().zip(&rects) {
            for (button, rect) in group.1.iter_mut().zip(row) {
                result = result.merge(button.handle_event(event, *rect));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rects = self.layout(content);
        let mut y = content.y;
        for ((label, group), row) in self.groups.iter().zip(&rects) {
            group_label(c, label, content.x, y, theme);
            let mut max_h: f32 = 0.0;
            for (button, rect) in group.iter().zip(row) {
                button.render(c, *rect, theme);
                max_h = max_h.max(rect.h);
            }
            y += LABEL_H + max_h + ROW_GAP;
        }
    }
}
