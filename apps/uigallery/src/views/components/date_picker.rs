//! Date Picker page, on the showcase mold of the kit's ButtonsPage: the
//! picker (`engine::widgets::selection::DatePicker`), front A's field
//! over front B's popover with the calendar in it, closed and live,
//! disabled, and open, with a day chosen and with a range and nothing
//! chosen. The kit has no date picker; every visual value comes from the
//! theme (`theme.form`, `theme.overlay.popover`, `theme.selection`).
//!
//! A card holds its picker at the top and the room its panel opens into
//! below: the panel's container is the card's body, so it answers the
//! card, never the window, and nothing else draws over it. The live
//! cards open on a press of the box and close on a day, a second press,
//! a press outside or Escape; the "Open" cards show the panel forced.
//!
//! Projected: a group per title, `date_picker_page.group[<g>]`; in it
//! each card's picker, `date_picker_page.group[<g>].card[<c>]`, a button
//! named by its label and described by its chip, with its `value` (the
//! day's key) and `expanded`, whose `activate` opens and closes the panel
//! as a press does; while open, its panel `<card>.panel` with a `close`,
//! and in it the calendar `<card>.calendar`, whose days' `select`
//! chooses the day and whose arrows' `activate` turn the month. A day's
//! `select` keeps the panel open, as the keyboard walks the days choosing
//! each (the person's click closes it): Escape or `close` leaves it.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::overlay::popover::Popover;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::widgets::selection::{Calendar, Date, DatePicker};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, key_under, root_id, under};

/// The day the page calls today, as the Calendar page's: a fixed sample,
/// so the page draws the same frame on every day it runs.
const TODAY: Date = Date {
    year: 2026,
    month: 10,
    day: 31,
};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    picker: DatePicker,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    span: usize,
    cards: Vec<Card>,
}

/// What the keyboard or a reader focused in a picker: its box, a day of
/// its open calendar, or an arrow (`-1` the month before, `1` after).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Spot {
    Box,
    Day(Date),
    Arrow(i32),
}

pub struct DatePickerPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The spot, `(group, card, spot)`, a reader or the keyboard focused
    /// (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize, Spot)>,
}

/// `date_picker_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `date_picker_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn day(day: u32) -> Date {
    Date {
        year: 2026,
        month: 10,
        day,
    }
}

/// A due date picker on October 2026, the 14th chosen.
fn due() -> DatePicker {
    let calendar = Calendar::new(2026, 10).selected(day(14)).today(TODAY);
    DatePicker::new("Due date", "Pick a day", calendar)
}

/// A check-in picker that takes the 10th to the 20th, nothing chosen.
fn check_in() -> DatePicker {
    let calendar = Calendar::new(2026, 10)
        .today(TODAY)
        .range(Some(day(10)), Some(day(20)));
    DatePicker::new("Check-in", "Pick a day", calendar).helper("From the 10th to the 20th")
}

impl DatePickerPage {
    pub fn new() -> Self {
        let mut groups = vec![
            Group {
                title: "Date picker",
                span: 2,
                cards: vec![
                    Card {
                        title: "Default, Hover, Press, Focus",
                        picker: due(),
                    },
                    Card {
                        title: "Disabled",
                        picker: due().disabled(true),
                    },
                ],
            },
            Group {
                title: "Date picker, open",
                span: 2,
                cards: vec![
                    Card {
                        title: "Open, a day chosen, today ringed",
                        picker: due().pinned(),
                    },
                    Card {
                        title: "Open, a range, nothing chosen",
                        picker: check_in().pinned(),
                    },
                ],
            },
        ];
        let root = root_id("date_picker_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.picker = card.picker.clone().sem(id);
                    }
                }
            }
        }
        Self {
            root,
            groups,
            focus: None,
        }
    }

    fn cards(&self) -> impl Iterator<Item = ((usize, usize), &Card)> {
        self.groups.iter().enumerate().flat_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .map(move |(c, card)| ((g, c), card))
        })
    }

    /// The card whose picker, panel or calendar `id` names.
    fn at(&self, id: &SemId) -> Option<(usize, usize)> {
        self.cards()
            .find(|(_, card)| {
                card.picker
                    .sem_id()
                    .is_some_and(|p| p == id || under(p, id))
            })
            .map(|(at, _)| at)
    }

    /// What a card asks: its picker's field and the room its panel
    /// opens into, a gap between; a disabled picker, which never opens,
    /// its field alone.
    fn picker_h(picker: &DatePicker, w: f32, theme: &Theme) -> f32 {
        let field = picker.field_height(w, theme);
        if picker.is_disabled() {
            return field;
        }
        field + theme.spacing.sm + picker.panel_size(w, theme).1
    }

    /// The picker's width in a card: the panel's at the card's body.
    fn picker_w(picker: &DatePicker, body_w: f32, theme: &Theme) -> f32 {
        picker.panel_size(body_w, theme).0.min(body_w)
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| {
            let card_w = showcase::card_width(g.span, content, theme);
            let body_w = showcase::room(card_w, theme);
            let heights = g
                .cards
                .iter()
                .map(|c| {
                    let w = Self::picker_w(&c.picker, body_w, theme);
                    showcase::card_height(Self::picker_h(&c.picker, w, theme), theme)
                })
                .collect();
            (g.title, g.span, heights)
        });
        showcase::layout_sized(groups, content, theme)
    }

    /// The picker's field bounds and the container its panel opens in:
    /// the field at the top of the room the card keeps, centered across,
    /// the room below it the container.
    fn places(picker: &DatePicker, card: Rect, theme: &Theme) -> (Rect, Rect) {
        let body = showcase::body(card, theme);
        let w = Self::picker_w(picker, body.w, theme);
        let h = Self::picker_h(picker, w, theme);
        let at = showcase::centered((w, h), card);
        let field = Rect::new(at.x, at.y, w, picker.field_height(w, theme));
        (field, Rect::new(at.x, at.y, w, h))
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// The spot `id` names in card `(g, c)`'s picker.
    fn spot(&self, g: usize, c: usize, id: &SemId) -> Option<Spot> {
        let picker = &self.groups[g].cards[c].picker;
        let cal = picker.calendar.sem_id()?;
        if picker.sem_id() == Some(id) {
            Some(Spot::Box)
        } else if cal.child("prev").ok().as_ref() == Some(id) {
            Some(Spot::Arrow(-1))
        } else if cal.child("next").ok().as_ref() == Some(id) {
            Some(Spot::Arrow(1))
        } else {
            let key = key_under(cal, id)?;
            Date::parse(key).map(Spot::Day)
        }
    }

    /// The focus on one spot, or on none; a disabled picker refuses it.
    fn set_focus(&mut self, at: Option<(usize, usize, Spot)>) {
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let spot = at.filter(|a| (a.0, a.1) == (g, c)).map(|a| a.2);
                let p = &mut card.picker;
                p.set_focused(spot == Some(Spot::Box));
                match spot {
                    Some(Spot::Day(d)) => p.calendar.set_focused(Some(d)),
                    Some(Spot::Arrow(by)) if !p.is_disabled() => p.calendar.set_arrow_focused(by),
                    _ => p.calendar.set_focused(None),
                }
            }
        }
        self.focus = at.filter(|&(g, c, spot)| {
            let p = &self.groups[g].cards[c].picker;
            match spot {
                Spot::Box => p.is_focused(),
                Spot::Day(d) => p.calendar.focused() == Some(d),
                Spot::Arrow(_) => !p.is_disabled(),
            }
        });
    }

    /// Escape: an open live panel closes first (a pinned one stays),
    /// then the focus goes.
    pub fn handle_escape(&mut self) -> bool {
        let mut closed = false;
        for group in &mut self.groups {
            for card in &mut group.cards {
                closed |= card.picker.close();
            }
        }
        if closed {
            return true;
        }
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.focus.is_some() {
            self.set_focus(None);
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (group, at) in self.groups.iter_mut().zip(&layout) {
            for (card, rect) in group.cards.iter_mut().zip(&at.cards) {
                let (field, room) = Self::places(&card.picker, *rect, theme);
                result = result.merge(card.picker.handle_event(event, field, room, theme));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                showcase::card(c, *rect, card.title, theme);
                let (field, room) = Self::places(&card.picker, *rect, theme);
                card.picker.render(c, field, room, theme);
            }
            if i != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for DatePickerPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                if let Some(id) = card.picker.sem_id() {
                    card.picker.project(b, id, Some(&at));
                    if let Some(mut node) = b.reopen(id) {
                        node.description(card.title);
                    }
                }
            }
        }
        let focused = self.focus.and_then(|(g, c, spot)| {
            let p = &self.groups.get(g)?.cards.get(c)?.picker;
            let cal = p.calendar.sem_id()?;
            match spot {
                Spot::Box => p.sem_id().cloned(),
                Spot::Day(d) => Some(cal.key(d.key())),
                Spot::Arrow(by) => cal.child(if by < 0 { "prev" } else { "next" }).ok(),
            }
        });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// The box's `activate` opens and closes the panel, the panel's
    /// `close` closes it, a day's `select` chooses it and an arrow's
    /// `activate` turns the month, as their presses do.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let refuse = || format!("the page has no {} on {}", req.action, req.target);
        let (g, c) = self.at(&req.target).ok_or_else(refuse)?;
        let picker = &mut self.groups[g].cards[c].picker;
        let id = picker.sem_id().cloned().ok_or_else(refuse)?;
        let cal = picker.calendar.sem_id().cloned().ok_or_else(refuse)?;
        let ran = match req.action.as_str() {
            "activate" if req.target == id => picker.toggle(),
            "close" if Some(&req.target) == Popover::panel_id(&id).as_ref() => picker.close(),
            "select" => key_under(&cal, &req.target)
                .and_then(Date::parse)
                .is_some_and(|d| picker.is_open() && picker.pick(d)),
            "activate" => {
                let by = if cal.child("prev").ok().as_ref() == Some(&req.target) {
                    -1
                } else if cal.child("next").ok().as_ref() == Some(&req.target) {
                    1
                } else {
                    0
                };
                picker.is_open() && picker.calendar.turn(by)
            }
            _ => false,
        };
        if ran {
            Ok(After::Nothing)
        } else {
            Err(refuse())
        }
    }

    /// The box takes the focus (its ring); another node of the page takes
    /// it from the pickers.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self
            .at(id)
            .and_then(|(g, c)| Some((g, c, self.spot(g, c, id)?)));
        self.set_focus(at);
        true
    }

    /// Escape: an open live panel closes, then the focus goes.
    fn blur(&mut self) -> bool {
        self.handle_escape()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The width sweep of the epic, 280 to 1600 every 8 px under the core
    /// and the hoff themes: every card in the content and apart from its
    /// neighbor, every picker inside its card and under its chip, and an
    /// open panel inside its card, below the box.
    #[test]
    fn every_picker_and_panel_fits_its_card_from_narrow_to_wide() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = DatePickerPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (n, (card, r)) in group.cards.iter().zip(&at.cards).enumerate() {
                        assert!(r.x >= content.x && r.right() <= content.right() + 0.5);
                        if let Some(next) = at.cards.get(n + 1)
                            && next.y == r.y
                        {
                            assert!(next.x >= r.right(), "{w}: cards overlap");
                        }
                        let inset = showcase::inset(&theme);
                        let chip =
                            showcase::chip_rect(r.x + inset, r.y + inset, card.title, &theme);
                        let (field, room) = DatePickerPage::places(&card.picker, *r, &theme);
                        assert!(field.x >= r.x && field.right() <= r.right() + 0.5, "{w}");
                        assert!(field.y >= chip.bottom(), "{w}: under the chip");
                        assert!(room.bottom() <= r.bottom() + 0.5, "{w}: the room");
                        if card.picker.is_open() {
                            let l = card.picker.layout(field, room, &theme);
                            assert!(l.panel.y >= l.field.bottom(), "{w}: below the message");
                            assert!(l.panel.x >= r.x && l.panel.right() <= r.right() + 0.5);
                            assert!(l.panel.bottom() <= r.bottom() + 0.5, "{w}: in the card");
                        }
                    }
                }
            }
        }
    }

    /// A press on the live box opens its panel; a day pressed fills the
    /// box and closes it; Escape closes an open panel before it lets the
    /// focus go; the disabled picker takes neither press nor focus.
    #[test]
    fn the_live_picker_opens_chooses_and_closes() {
        let theme = Theme::hoff_core();
        let mut page = DatePickerPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let card = page.layout(content, &theme)[0].cards[0];
        let (field, room) = DatePickerPage::places(&page.groups[0].cards[0].picker, card, &theme);
        let press = |page: &mut DatePickerPage, (x, y): (f32, f32)| {
            page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
            page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme)
        };
        let input = page.groups[0].cards[0]
            .picker
            .layout(field, room, &theme)
            .field
            .input;
        press(&mut page, input.center());
        assert!(page.groups[0].cards[0].picker.is_open());
        let p = &page.groups[0].cards[0].picker;
        let l = p.layout(field, room, &theme);
        let at = p
            .calendar
            .grid()
            .iter()
            .position(|d| *d == day(21))
            .unwrap();
        let cell = p.calendar.layout(l.calendar, &theme).days[at];
        press(&mut page, cell.center());
        let p = &page.groups[0].cards[0].picker;
        assert_eq!((p.chosen(), p.is_open()), (Some(day(21)), false));

        press(&mut page, input.center());
        assert!(page.handle_escape(), "Escape closes the panel");
        assert!(!page.groups[0].cards[0].picker.is_open());

        let off = page.groups[0].cards[1].picker.sem_id().unwrap().clone();
        assert!(page.focus(&off));
        assert_eq!(page.focus, None, "a disabled picker takes no focus");
        let open = &page.groups[1].cards[0].picker;
        let day14 = open.calendar.sem_id().unwrap().key(day(14).key());
        assert!(page.focus(&day14));
        assert_eq!(page.focus, Some((1, 0, Spot::Day(day(14)))));
    }
}
