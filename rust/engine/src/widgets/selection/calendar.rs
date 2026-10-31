//! The calendar: one month as a grid of days, a title and the arrows to
//! the month before and after, weeks from Monday (ISO 8601). The kit has
//! no calendar, so it is derived from it: a day presses like the
//! toolbar actions (`theme.button.toolbar`, through the buttons' own
//! [`Press`] and `paint_surface`), the chosen day is the dark call to
//! action (`theme.button.secondary`), today is ringed as the outline
//! button, and the arrows are ghost [`IconButton`]s. The weeks keep their
//! six rows, and a day of another month is a blank cell: neither drawn
//! nor pressed, as the projection holds only the month shown. Its own values (cell
//! range, gap, text) are `theme.selection.calendar`.
//!
//! The grid fits the `Rect` it gets: a day is a seventh of the width,
//! clamped between the token's minimum and maximum, never a fixed size.
//!
//! [`Date`] is the civil date the calendar works in, with no time zone:
//! the app says which day is today.
//!
//! With a [`SemId`] ([`Calendar::sem`]) the render binds the calendar,
//! its arrows (`<id>.prev`, `<id>.next`) and each day of the month shown
//! (`<id>[<yyyy-mm-dd>]`), and [`Calendar::project`] declares a
//! `Role::Group` whose text `selected` names the chosen day, each day a
//! `Role::Radio` with its `checked` and a `select`.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::{
    IconButton, IconButtonSize, IconButtonVariant, Press, paint_surface,
};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// A civil date, proleptic Gregorian, no time zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
    /// 1 to the month's length.
    pub day: u32,
}

impl Date {
    /// The date, when it exists.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        let ok = (1..=12).contains(&month) && day >= 1 && day <= Self::days_in(year, month);
        ok.then_some(Self { year, month, day })
    }

    pub fn is_leap(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// The length of `month` in `year`.
    pub fn days_in(year: i32, month: u32) -> u32 {
        match month {
            2 if Self::is_leap(year) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    /// Days since 1970-01-01 (H. Hinnant's `days_from_civil`).
    pub fn to_days(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` after 1970-01-01 (`civil_from_days`).
    pub fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Self { year, month, day }
    }

    pub fn add_days(self, days: i64) -> Self {
        Self::from_days(self.to_days() + days)
    }

    /// 0 for Monday to 6 for Sunday.
    pub fn weekday(self) -> u32 {
        // 1970-01-01 was a Thursday.
        (self.to_days() + 3).rem_euclid(7) as u32
    }

    /// `yyyy-mm-dd`.
    pub fn key(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// A `yyyy-mm-dd` text, when it is a date.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.splitn(3, '-');
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        Self::new(year, month, day)
    }

    /// The day as a person reads it: `31 October 2026`.
    pub fn spoken(self) -> String {
        format!(
            "{} {} {}",
            self.day,
            MONTHS[self.month as usize - 1],
            self.year
        )
    }
}

/// One month's place: `(year, month)` moved by `by` months.
fn shift((year, month): (i32, u32), by: i32) -> (i32, u32) {
    let at = year * 12 + month as i32 - 1 + by;
    (at.div_euclid(12), at.rem_euclid(12) as u32 + 1)
}

/// Where everything sits in a calendar's `Rect`.
#[derive(Clone, Debug)]
pub struct CalendarLayout {
    pub title: Rect,
    pub prev: Rect,
    pub next: Rect,
    /// One per weekday column.
    pub weekdays: [Rect; 7],
    /// The 42 day cells, week by week.
    pub days: Vec<Rect>,
}

#[derive(Clone, Debug)]
pub struct Calendar {
    /// The month shown, `(year, 1..=12)`.
    pub shown: (i32, u32),
    pub selected: Option<Date>,
    pub today: Option<Date>,
    /// The first and the last day that can be chosen.
    pub min: Option<Date>,
    pub max: Option<Date>,
    pub disabled: bool,
    prev: IconButton,
    next: IconButton,
    presses: Vec<Press>,
    focused: Option<Date>,
    sem: Option<SemId>,
}

impl Calendar {
    /// The calendar on `month` of `year`, nothing chosen.
    pub fn new(year: i32, month: u32) -> Self {
        let arrow = |icon| {
            IconButton::new(icon)
                .variant(IconButtonVariant::Ghost)
                .size(IconButtonSize::Sm)
        };
        Self {
            shown: (year, month.clamp(1, 12)),
            selected: None,
            today: None,
            min: None,
            max: None,
            disabled: false,
            prev: arrow("chevron-left"),
            next: arrow("chevron-right"),
            presses: vec![Press::default(); 42],
            focused: None,
            sem: None,
        }
    }

    /// `date` chosen, its month shown.
    pub fn selected(mut self, date: Date) -> Self {
        self.selected = Some(date);
        self.shown = (date.year, date.month);
        self
    }

    pub fn today(mut self, date: Date) -> Self {
        self.today = Some(date);
        self
    }

    /// Only days from `min` to `max` can be chosen.
    pub fn range(mut self, min: Option<Date>, max: Option<Date>) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.prev.disabled = disabled;
        self.next.disabled = disabled;
        self
    }

    /// The identity the calendar binds to; the arrows are `<id>.prev`
    /// and `<id>.next`, each day `<id>[<yyyy-mm-dd>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(prev) = id.child("prev") {
            self.prev = self.prev.sem(prev);
        }
        if let Ok(next) = id.child("next") {
            self.next = self.next.sem(next);
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Whether `date` can be chosen.
    pub fn allows(&self, date: Date) -> bool {
        !self.disabled && self.min.is_none_or(|m| date >= m) && self.max.is_none_or(|m| date <= m)
    }

    /// Choose `date`, as a click on it does, and show its month. A date
    /// out of the range, a disabled calendar or the chosen day change
    /// nothing; returns whether the choice moved.
    pub fn select(&mut self, date: Date) -> bool {
        if !self.allows(date) || self.selected == Some(date) {
            return false;
        }
        self.selected = Some(date);
        self.shown = (date.year, date.month);
        true
    }

    /// Show the month `by` months away, as the arrows do.
    pub fn turn(&mut self, by: i32) -> bool {
        if self.disabled || by == 0 {
            return false;
        }
        self.shown = shift(self.shown, by);
        true
    }

    /// Keyboard focus on `date` (its month shown), or on none.
    pub fn set_focused(&mut self, date: Option<Date>) {
        self.focused = date.filter(|_| !self.disabled);
        if let Some(d) = self.focused {
            self.shown = (d.year, d.month);
        }
        self.prev.set_focused(false);
        self.next.set_focused(false);
    }

    /// Keyboard focus on an arrow: `-1` the previous month, `1` the next.
    pub fn set_arrow_focused(&mut self, by: i32) {
        self.focused = None;
        self.prev.set_focused(by < 0);
        self.next.set_focused(by > 0);
    }

    pub fn focused(&self) -> Option<Date> {
        self.focused
    }

    /// The 42 days of the grid: the weeks that hold the month shown, from
    /// the Monday on or before its first day.
    pub fn grid(&self) -> Vec<Date> {
        let (year, month) = self.shown;
        let first = Date {
            year,
            month,
            day: 1,
        };
        let start = first.add_days(-i64::from(first.weekday()));
        (0..42).map(|i| start.add_days(i)).collect()
    }

    /// The month's title: `October 2026`.
    pub fn title(&self) -> String {
        format!("{} {}", MONTHS[self.shown.1 as usize - 1], self.shown.0)
    }

    fn style(theme: &Theme, size: f32, line: f32, weight: u16) -> TextStyle {
        theme.typography.exact(size, line, weight)
    }

    /// A day cell's edge in `width`: a seventh of it on whole pixels,
    /// clamped.
    fn cell(width: f32, theme: &Theme) -> f32 {
        let t = &theme.selection.calendar;
        ((width - t.gap * 6.0) / 7.0)
            .floor()
            .clamp(t.cell_min, t.cell_max)
    }

    /// The calendar's size within `max_w`.
    pub fn size_within(&self, max_w: f32, theme: &Theme) -> (f32, f32) {
        let t = &theme.selection.calendar;
        let cell = Self::cell(max_w, theme);
        (
            cell * 7.0 + t.gap * 6.0,
            cell + t.gap + t.weekday_line + t.gap + cell * 6.0 + t.gap * 5.0,
        )
    }

    /// The narrowest the calendar gets.
    pub fn min_width(theme: &Theme) -> f32 {
        let t = &theme.selection.calendar;
        (t.cell_min * 7.0 + t.gap * 6.0).ceil()
    }

    /// Where the title, the arrows, the weekdays and the days sit in
    /// `bounds`, the grid centered across it.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> CalendarLayout {
        let t = &theme.selection.calendar;
        let cell = Self::cell(bounds.w, theme);
        let grid_w = cell * 7.0 + t.gap * 6.0;
        let x0 = bounds.x + ((bounds.w - grid_w) / 2.0).max(0.0);
        let arrow = self.prev.preferred_size(theme).0;
        let mid = bounds.y + (cell - arrow) / 2.0;
        let next = Rect::new(x0 + grid_w - arrow, mid, arrow, arrow);
        let prev = Rect::new(next.x - t.gap - arrow, mid, arrow, arrow);
        let title = Rect::new(x0, bounds.y, (prev.x - t.gap - x0).max(0.0), cell);
        let wd_y = bounds.y + cell + t.gap;
        let col = |i: usize| x0 + i as f32 * (cell + t.gap);
        let weekdays = std::array::from_fn(|i| Rect::new(col(i), wd_y, cell, t.weekday_line));
        let top = wd_y + t.weekday_line + t.gap;
        let days = (0..42)
            .map(|i| {
                Rect::new(
                    col(i % 7),
                    top + (i / 7) as f32 * (cell + t.gap),
                    cell,
                    cell,
                )
            })
            .collect();
        CalendarLayout {
            title,
            prev,
            next,
            weekdays,
            days,
        }
    }

    /// Pointer events: the arrows turn the month, a day presses on its
    /// own and a click chooses it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        let mut result = self.prev.handle_event(event, l.prev);
        if result.clicked {
            self.turn(-1);
        }
        let r = self.next.handle_event(event, l.next);
        if r.clicked {
            self.turn(1);
        }
        result = result.merge(r);
        // a day of another month is not drawn, so it does not press
        let grid = self.grid();
        let live: Vec<bool> = grid
            .iter()
            .map(|d| d.month == self.shown.1 && self.allows(*d))
            .collect();
        let mut chosen = None;
        for (i, (press, rect)) in self.presses.iter_mut().zip(&l.days).enumerate() {
            let blocked = !live[i];
            let r = press.handle(event, *rect, blocked);
            if r.clicked {
                chosen = Some(grid[i]);
            }
            result = result.merge(r);
        }
        if let Some(date) = chosen {
            self.select(date);
        }
        result
    }

    /// Declares the calendar as `id`: a `Role::Group` named `label` whose
    /// text `selected` is the chosen day's key (empty for none) and
    /// `month` the month shown (`yyyy-mm`); its two arrows as buttons;
    /// and each day of the month shown a `Role::Radio` `<id>[<key>]`,
    /// named as a person reads it, with its `checked`, enabled when it can
    /// be chosen, and a `select` that reads the calendar's `selected`.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for (arrow, name) in [(&self.prev, "previous month"), (&self.next, "next month")] {
            if let Some(at) = arrow.sem_id() {
                arrow.project(b, at, name, Some(id));
            }
        }
        let reads = format!("{id}:selected");
        for date in self.grid().into_iter().filter(|d| d.month == self.shown.1) {
            let mut node = b.node(id.key(date.key()), Role::Radio, date.spoken());
            node.parent(id);
            node.state("checked", self.selected == Some(date))
                .enabled(self.allows(date));
            node.action("select").reads(reads.as_str());
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        let (year, month) = self.shown;
        node.state("selected", self.selected.map(Date::key).unwrap_or_default())
            .state("month", format!("{year:04}-{month:02}"))
            .enabled(!self.disabled);
        node
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.selection.calendar;
        let b = &theme.button;
        let l = self.layout(bounds, theme);
        let grid = self.grid();
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
            for (date, r) in grid.iter().zip(&l.days) {
                if date.month == self.shown.1 {
                    c.bind(&id.key(date.key()), [r.x, r.y, r.w, r.h]);
                }
            }
        }
        let fade = if self.disabled {
            b.toolbar.disabled_alpha
        } else {
            1.0
        };
        let title = Self::style(theme, t.title_size, t.title_line, t.title_weight);
        text(
            c,
            &self.title(),
            &title,
            l.title,
            false,
            b.toolbar.rest.text,
            fade,
        );
        self.prev.render(c, l.prev, theme);
        self.next.render(c, l.next, theme);
        let wd = Self::style(theme, t.weekday_size, t.weekday_line, t.weekday_weight);
        for (name, r) in WEEKDAYS.iter().zip(&l.weekdays) {
            text(c, name, &wd, *r, true, t.weekday, fade);
        }
        let day = Self::style(theme, t.day_size, t.day_line, t.day_weight);
        // Today's ring: the outline button's, with no fill of its own.
        let mut ring = b.outline.rest.clone();
        ring.fill = [Color::rgba(0.0, 0.0, 0.0, 0.0); 2];
        for (i, (date, r)) in grid.iter().zip(&l.days).enumerate() {
            if date.month != self.shown.1 {
                continue;
            }
            let chosen = self.selected == Some(*date);
            let recipe = if chosen { &b.secondary } else { &b.toolbar };
            let alpha = if self.allows(*date) {
                1.0
            } else {
                recipe.disabled_alpha
            };
            let radius = t.cell_radius.min(r.h / 2.0);
            let surface = self.presses[i].surface(recipe);
            let focused = self.focused == Some(*date);
            paint_surface(
                c,
                LayerId::DEFAULT,
                *r,
                radius,
                surface,
                alpha,
                focused,
                theme,
            );
            if self.today == Some(*date) && !chosen {
                paint_surface(c, LayerId::DEFAULT, *r, radius, &ring, alpha, false, theme);
            }
            text(
                c,
                &date.day.to_string(),
                &day,
                *r,
                true,
                surface.text,
                alpha,
            );
        }
    }
}

/// One run of text in `r`, centered across or from its left, centered
/// down, faded by `alpha`.
fn text(
    c: &mut Compositor,
    s: &str,
    style: &TextStyle,
    r: Rect,
    centered: bool,
    color: Color,
    alpha: f32,
) {
    let (w, _) = TextMeasurer::measure_styled(s, style, None);
    let [cr, cg, cb, ca] = color.0;
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(s, style, None),
        x: if centered { r.x + (r.w - w) / 2.0 } else { r.x },
        y: r.y + TextMeasurer::vertical_center(style, r.h),
        color: [cr, cg, cb, ca * alpha],
    });
}
