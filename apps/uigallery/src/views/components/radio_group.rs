//! Radio Group page, on the showcase mold of the kit's ButtonsPage: the
//! radio group (`engine::widgets::selection::RadioGroup`), its rows
//! stacked and flowing along a line, disabled, and with long labels that
//! truncate. The kit has no radio: the rows are its single-choice list
//! line and the discs its switch. Every visual value comes from the theme.
//!
//! The page is a choice page (`toggle_group::ChoicePage`): each card's
//! group projects as a `Role::Group` of `Role::Radio` rows,
//! `radio_group_page.group[<g>].card[<c>][<i>]`, and a row's `select`
//! chooses it as a click does.

use accord::{NodeBuilder, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::widgets::selection::{RadioGroup, RadioLayout};

use super::toggle_group::{Card, Choice, ChoicePage, Group};

impl Choice for RadioGroup {
    fn count(&self) -> usize {
        self.items.len()
    }
    #[cfg(test)]
    fn label(&self, i: usize) -> &str {
        &self.items[i]
    }
    #[cfg(test)]
    fn chosen(&self) -> usize {
        self.selected
    }
    fn choose(&mut self, i: usize) -> bool {
        self.select(i)
    }
    fn set_focus(&mut self, i: Option<usize>) {
        self.set_focused(i);
    }
    fn focus_at(&self) -> Option<usize> {
        self.focused()
    }
    fn id(&self) -> Option<&SemId> {
        self.sem_id()
    }
    fn with_id(self, id: SemId) -> Self {
        self.sem(id)
    }
    fn size_in(&self, room: f32, theme: &Theme) -> (f32, f32) {
        self.size_within(room, theme)
    }
    #[cfg(test)]
    fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        self.row_rects(bounds, theme)
    }
    #[cfg(test)]
    fn shown(&self, i: usize, item: Rect, theme: &Theme) -> String {
        self.shown_label(i, item, theme)
    }
    fn pointer(&mut self, event: &WidgetEvent, bounds: Rect, theme: &Theme) -> EventResult {
        self.handle_event(event, bounds, theme)
    }
    fn draw(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render(c, bounds, theme);
    }
    fn declare<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        name: &str,
        parent: &SemId,
    ) -> NodeBuilder<'b> {
        self.project(b, id, name, Some(parent))
    }
}

pub type RadioGroupPage = ChoicePage<RadioGroup>;

const PLANS: [&str; 3] = ["Free", "Pro", "Team"];

fn pair(title: &'static str, layout: RadioLayout) -> Group<RadioGroup> {
    let make = || RadioGroup::new(PLANS).selected(1).layout(layout);
    Group {
        title,
        span: 2,
        cards: vec![
            Card {
                title: "Default, Hover, Press, Focus",
                name: "plan",
                widget: make(),
            },
            Card {
                title: "Disabled",
                name: "plan",
                widget: make().disabled(true),
            },
        ],
    }
}

impl ChoicePage<RadioGroup> {
    pub fn new() -> Self {
        let groups = vec![
            pair("Radio group", RadioLayout::Stack),
            pair("Radio group, in a line", RadioLayout::Flow),
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    Card {
                        title: "Long labels",
                        name: "sync",
                        widget: RadioGroup::new([
                            "Only this device",
                            "Every device signed in to this account, at home and at work",
                        ]),
                    },
                    Card {
                        title: "Translated labels, in a line",
                        name: "Plan",
                        widget: RadioGroup::new(["Kostenlos", "Professionell", "Mannschaft"])
                            .layout(RadioLayout::Flow),
                    },
                ],
            },
        ];
        Self::with("radio_group_page", groups)
    }
}

#[cfg(test)]
mod tests {
    use super::super::toggle_group::tests::{check_choice_page, check_truncates_only_when_narrow};
    use super::*;

    #[test]
    fn the_radio_group_page_passes_the_choice_checks() {
        check_choice_page(|_| RadioGroupPage::new());
    }

    #[test]
    fn the_long_label_truncates_only_when_the_card_is_narrow() {
        check_truncates_only_when_narrow(&RadioGroupPage::new(), (2, 0, 1));
    }
}
