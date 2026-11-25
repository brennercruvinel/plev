//! Calendar page, on the showcase mold of the kit's ButtonsPage: the
//! calendar (`engine::widgets::selection::Calendar`) with a chosen day and
//! today, disabled, with a range of days that can be chosen, and on a leap
//! February with nothing chosen. The kit has no calendar: its days press
//! as the toolbar actions and the chosen one is the call to action; every
//! visual value comes from the theme.
//!
//! A card is as tall as its calendar asks at the card's width
//! (`showcase::card_height`): a day is a seventh of the width, so the
//! grid grows with the card between the token's range.
//!
//! Projected: a group per title, `calendar_page.group[<g>]`; in it each
//! card's calendar, `calendar_page.group[<g>].card[<c>]`, a `Role::Group`
//! named by the card and described by its chip, whose `selected` is the
//! chosen day and `month` the month shown; its arrows (`<card>.prev`,
//! `<card>.next`), whose `activate` turns the month as a click does; and
//! the days of the month shown, `<card>[<yyyy-mm-dd>]`, `Role::Radio`
//! with a `select` that chooses the day as a click does.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::widgets::selection::{Calendar, Date};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, key_under, root_id, under};

/// The day the page calls today: a fixed sample, so the page draws the
/// same frame on every day it runs (the regress baselines hold).
const TODAY: Date = Date {
    year: 2026,
    month: 10,
    day: 31,
};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    /// The calendar's accessible name.
    name: &'static str,
    calendar: Calendar,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    span: usize,
    cards: Vec<Card>,
}

/// What the keyboard or a reader focused in a calendar: a day, or an
/// arrow (`-1` the previous month, `1` the next).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Spot {
    Day(Date),
    Arrow(i32),
}

pub struct CalendarPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The spot, `(group, card, spot)`, a reader or the keyboard focused
    /// (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize, Spot)>,
}

/// `calendar_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `calendar_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn day(year: i32, month: u32, day: u32) -> Date {
    Date { year, month, day }
}

impl CalendarPage {
    pub fn new() -> Self {
        let chosen = || {
            Calendar::new(2026, 10)
                .selected(day(2026, 10, 14))
                .today(TODAY)
        };
        let mut groups = vec![
            Group {
                title: "Calendar",
                span: 2,
                cards: vec![
                    Card {
                        title: "Default, Hover, Press, Focus, Today",
                        name: "due date",
                        calendar: chosen(),
                    },
                    Card {
                        title: "Disabled",
                        name: "due date",
                        calendar: chosen().disabled(true),
                    },
                ],
            },
            Group {
                title: "Calendar, limits",
                span: 2,
                cards: vec![
                    Card {
                        title: "Range: the 10th to the 20th",
                        name: "check-in",
                        calendar: Calendar::new(2026, 10)
                            .today(TODAY)
                            .range(Some(day(2026, 10, 10)), Some(day(2026, 10, 20)))
                            .selected(day(2026, 10, 15)),
                    },
                    Card {
                        title: "Nothing chosen, a leap February",
                        name: "start",
                        calendar: Calendar::new(2028, 2),
                    },
                ],
            },
        ];
        let root = root_id("calendar_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.calendar = card.calendar.clone().sem(id);
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

    /// The spot behind `id`: `(group, card, spot)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, Spot)> {
        self.cards().find_map(|((g, c), card)| {
            let cal = card.calendar.sem_id()?;
            let arrow = |name: &str| cal.child(name).is_ok_and(|a| a == *id);
            let spot = if arrow("prev") {
                Spot::Arrow(-1)
            } else if arrow("next") {
                Spot::Arrow(1)
            } else {
                Spot::Day(Date::parse(key_under(cal, id)?)?)
            };
            Some((g, c, spot))
        })
    }

    fn calendar_mut(&mut self, g: usize, c: usize) -> Option<&mut Calendar> {
        Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.calendar)
    }

    /// The focus on one spot, or on none; a disabled calendar refuses it.
    fn set_focus(&mut self, at: Option<(usize, usize, Spot)>) {
        let at = at.filter(|&(g, c, _)| {
            self.groups
                .get(g)
                .and_then(|group| group.cards.get(c))
                .is_some_and(|card| !card.calendar.disabled)
        });
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                match at.filter(|a| (a.0, a.1) == (g, c)).map(|a| a.2) {
                    Some(Spot::Arrow(by)) => card.calendar.set_arrow_focused(by),
                    Some(Spot::Day(date)) => card.calendar.set_focused(Some(date)),
                    None => card.calendar.set_focused(None),
                }
            }
        }
        self.focus = at;
    }

    /// The groups down `content`, each card as tall as its calendar asks
    /// at the card's width.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|group| {
            let room = Self::room(showcase::card_width(group.span, content, theme), theme);
            let heights = group
                .cards
                .iter()
                .map(|card| showcase::card_height(card.calendar.size_within(room, theme).1, theme))
                .collect();
            (group.title, group.span, heights)
        });
        showcase::layout_sized(groups, content, theme)
    }

    /// The width a calendar gets in a card `card_w` wide: the card less
    /// its inset on both sides.
    fn room(card_w: f32, theme: &Theme) -> f32 {
        showcase::room(card_w, theme)
    }

    /// The calendar's rect in its card, centered.
    fn calendar_rect(calendar: &Calendar, card: Rect, theme: &Theme) -> Rect {
        showcase::centered(calendar.size_within(Self::room(card.w, theme), theme), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (group, at) in self.groups.iter_mut().zip(&layout) {
            for (card, rect) in group.cards.iter_mut().zip(&at.cards) {
                let bounds = Self::calendar_rect(&card.calendar, *rect, theme);
                result = result.merge(card.calendar.handle_event(event, bounds, theme));
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
                let bounds = Self::calendar_rect(&card.calendar, *rect, theme);
                card.calendar.render(c, bounds, theme);
            }
            if i != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Default for CalendarPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for CalendarPage {
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
                if let Some(id) = card.calendar.sem_id() {
                    card.calendar
                        .project(b, id, card.name, Some(&at))
                        .description(card.title);
                }
            }
        }
        let focused = self.focus.and_then(|(g, c, spot)| {
            let cal = self.groups.get(g)?.cards.get(c)?.calendar.sem_id()?;
            match spot {
                Spot::Day(date) => Some(cal.key(date.key())),
                Spot::Arrow(by) => cal.child(if by < 0 { "prev" } else { "next" }).ok(),
            }
        });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// A day's `select` chooses it and an arrow's `activate` turns the
    /// month, as their clicks do; a disabled calendar and a day out of the
    /// range refuse.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let done = match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, Spot::Day(date))), "select") => {
                self.calendar_mut(g, c).is_some_and(|cal| cal.select(date))
            }
            (Some((g, c, Spot::Arrow(by))), "activate") => {
                self.calendar_mut(g, c).is_some_and(|cal| cal.turn(by))
            }
            _ => false,
        };
        if done {
            Ok(After::Nothing)
        } else {
            Err(format!("the page has no {} on {}", req.action, req.target))
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.at(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

#[cfg(test)]
mod tests {
    use accord::ActionName;

    use super::*;

    const CONTENT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1200.0,
        h: 900.0,
    };

    /// Card `(g, c)`'s calendar rect in `content`.
    fn bounds(page: &CalendarPage, g: usize, c: usize, content: Rect, theme: &Theme) -> Rect {
        let card = page.layout(content, theme)[g].cards[c];
        CalendarPage::calendar_rect(&page.groups[g].cards[c].calendar, card, theme)
    }

    /// A click on `date` in card `(g, c)`, as the person's.
    fn click(page: &mut CalendarPage, g: usize, c: usize, date: Date) -> EventResult {
        let theme = Theme::hoff_core();
        let cal = &page.groups[g].cards[c].calendar;
        let b = bounds(page, g, c, CONTENT, &theme);
        let i = cal.grid().iter().position(|d| *d == date).unwrap();
        let (x, y) = cal.layout(b, &theme).days[i].center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, CONTENT, &theme);
        page.handle_event(&WidgetEvent::MouseUp { x, y }, CONTENT, &theme)
    }

    fn request(target: &SemId, action: &str) -> Request {
        Request {
            actor: accord::Actor::Test {
                name: "calendar".into(),
            },
            target: target.clone(),
            action: ActionName::parse(action).unwrap(),
            param: None,
            observed: accord::Observation {
                revision: accord::Snapshot::default().revision(),
                values: Vec::new(),
            },
            ask: None,
        }
    }

    fn cal(page: &CalendarPage, g: usize, c: usize) -> &Calendar {
        &page.groups[g].cards[c].calendar
    }

    #[test]
    fn every_calendar_has_an_id_and_its_days_and_arrows_map_back() {
        let page = CalendarPage::new();
        assert!(page.root.is_some());
        for ((g, c), card) in page.cards() {
            let id = card.calendar.sem_id().expect("an id").clone();
            let first = card.calendar.grid()[7];
            assert_eq!(
                page.at(&id.key(first.key())),
                Some((g, c, Spot::Day(first)))
            );
            let prev = id.child("prev").unwrap();
            assert_eq!(page.at(&prev), Some((g, c, Spot::Arrow(-1))));
            assert_eq!(page.at(&id.key("not-a-day")), None);
        }
    }

    #[test]
    fn a_click_chooses_a_day_and_a_disabled_calendar_stays() {
        let mut page = CalendarPage::new();
        assert!(click(&mut page, 0, 0, day(2026, 10, 22)).clicked);
        assert_eq!(cal(&page, 0, 0).selected, Some(day(2026, 10, 22)));
        click(&mut page, 0, 1, day(2026, 10, 22));
        assert_eq!(
            cal(&page, 0, 1).selected,
            Some(day(2026, 10, 14)),
            "disabled"
        );
        click(&mut page, 1, 0, day(2026, 10, 25));
        assert_eq!(
            cal(&page, 1, 0).selected,
            Some(day(2026, 10, 15)),
            "out of range"
        );
    }

    #[test]
    fn select_and_activate_run_as_the_clicks_and_refuse_the_same() {
        let mut page = CalendarPage::new();
        let id = cal(&page, 0, 0).sem_id().unwrap().clone();
        assert!(page.run(&request(&id.key("2026-10-03"), "select")).is_ok());
        assert_eq!(cal(&page, 0, 0).selected, Some(day(2026, 10, 3)));
        assert!(
            page.run(&request(&id.key("2026-10-03"), "select")).is_err(),
            "chosen"
        );
        let next = id.child("next").unwrap();
        assert!(page.run(&request(&next, "activate")).is_ok());
        assert_eq!(cal(&page, 0, 0).shown, (2026, 11));
        assert!(page.run(&request(&next, "select")).is_err());

        let off = cal(&page, 0, 1).sem_id().unwrap().clone();
        assert!(
            page.run(&request(&off.key("2026-10-03"), "select"))
                .is_err()
        );
        assert!(
            page.run(&request(&off.child("prev").unwrap(), "activate"))
                .is_err()
        );
        let ranged = cal(&page, 1, 0).sem_id().unwrap().clone();
        assert!(
            page.run(&request(&ranged.key("2026-10-21"), "select"))
                .is_err()
        );
        assert!(
            page.run(&request(&ranged.key("2026-10-20"), "select"))
                .is_ok()
        );
    }

    #[test]
    fn the_focus_lands_on_a_live_calendar_and_a_click_lets_it_go() {
        let mut page = CalendarPage::new();
        let id = cal(&page, 0, 0).sem_id().unwrap().clone();
        assert!(page.focus(&id.key("2026-10-05")));
        assert_eq!(page.focus, Some((0, 0, Spot::Day(day(2026, 10, 5)))));
        assert_eq!(cal(&page, 0, 0).focused(), Some(day(2026, 10, 5)));
        assert!(page.focus(&id.child("next").unwrap()));
        assert_eq!(page.focus, Some((0, 0, Spot::Arrow(1))));
        assert_eq!(cal(&page, 0, 0).focused(), None);
        let off = cal(&page, 0, 1).sem_id().unwrap().clone();
        assert!(page.focus(&off.key("2026-10-05")));
        assert_eq!(page.focus, None, "a disabled calendar takes no focus");
        assert!(page.focus(&id.key("2026-10-05")));
        assert!(click(&mut page, 0, 0, day(2026, 10, 6)).changed);
        assert_eq!(page.focus, None);
    }

    /// The width sweep of the epic, 280 to 1600 every 8 px under the core
    /// and the hoff themes: the groups in order, every card in the
    /// content and apart from its neighbor, every calendar inside its card
    /// and under its chip, every day cell inside its calendar and within
    /// the token's range, and the title of every month inside its rect.
    #[test]
    fn the_width_sweep_holds_every_calendar_in_its_card() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let t = &theme.selection.calendar;
            let page = CalendarPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let mut prev_rule = content.y;
                for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
                    assert!(at.title_y >= prev_rule, "{w}: groups in order");
                    prev_rule = at.rule_y;
                    for (n, (card, r)) in group.cards.iter().zip(&at.cards).enumerate() {
                        assert!(
                            r.x >= content.x && r.right() <= content.right() + 0.5,
                            "{w}"
                        );
                        if let Some(next) = at.cards.get(n + 1)
                            && next.y == r.y
                        {
                            assert!(next.x >= r.right(), "{w}: cards overlap");
                        }
                        let inset = showcase::inset(&theme);
                        let chip =
                            showcase::chip_rect(r.x + inset, r.y + inset, card.title, &theme);
                        let b = CalendarPage::calendar_rect(&card.calendar, *r, &theme);
                        assert!(
                            b.x >= r.x && b.right() <= r.right() + 0.5,
                            "{w}: in its card"
                        );
                        assert!(
                            b.y >= chip.bottom() && b.bottom() <= r.bottom(),
                            "{w}: chip"
                        );
                        let l = card.calendar.layout(b, &theme);
                        for d in &l.days {
                            assert!(d.w >= t.cell_min && d.w <= t.cell_max, "{w}: {}", d.w);
                            assert!(d.x >= b.x - 0.5 && d.right() <= b.right() + 0.5, "{w}");
                            assert!(d.bottom() <= b.bottom() + 0.5, "{w}");
                        }
                        let style = theme.typography.run(t.title);
                        for m in 1..=12 {
                            let title = Calendar::new(2026, m).title();
                            let (tw, _) = engine::drawing::text::TextMeasurer::measure_styled(
                                &title, &style, None,
                            );
                            assert!(tw <= l.title.w, "{w}: {title} {tw} in {}", l.title.w);
                        }
                    }
                }
            }
        }
    }

    /// The card grows with its calendar: a wide content gives the
    /// largest days and a taller card than the narrowest.
    #[test]
    fn the_card_is_as_tall_as_its_calendar_asks() {
        let theme = Theme::hoff_core();
        let page = CalendarPage::new();
        let narrow = page.layout(Rect::new(0.0, 0.0, 280.0, 800.0), &theme)[0].cards[0];
        let wide = page.layout(Rect::new(0.0, 0.0, 1600.0, 800.0), &theme)[0].cards[0];
        let max = cal(&page, 0, 0).size_within(f32::INFINITY, &theme).1;
        assert_eq!(wide.h, showcase::card_height(max, &theme));
        assert!(narrow.h <= wide.h);
    }
}
