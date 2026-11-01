//! Notifications page, in the showcase mold, as the hoff core kit's
//! NotificationsPage: the notification in each of its variations (a
//! comment, an image generated, an invite with its answers, a like, a
//! video generated), each as wide as the kit's or its card, then the
//! notifications dropdown on its well, its filters over a list that
//! scrolls. A card is as tall as what it holds (`showcase::layout_sized`),
//! never less than the kit's `h-50`.
//!
//! A notification is read by a click on it, and an invite answered by a
//! click on one of its buttons, which reads it too; a filter shows every
//! notification or the unread ones. The keyboard: Tab walks the
//! projection, the arrows walk the filters and the list's rows (the list
//! scrolls the row in), Enter and Space press, Escape lets the focus go.
//!
//! Projected: a group per title, `notifications_page.group[<g>]`; in it
//! each card as a group named by its chip, `…card[<c>]`; a variation's
//! notification, `…card[<c>].alert`, as the alert projects it; the
//! dropdown, `…card[<c>].dropdown`, with its filters
//! (`.filter[<i>]`) and its rows (`.list[<i>]`, their answers
//! `.action[<j>]`).

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::feedback::alert::Alert;
use engine::widgets::message::notifications::{Notifications, kit_notifications};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect};

use super::{After, Component, child_id, key_under, root_id, showcase, under};

enum Sample {
    Alert(Box<Alert>),
    Dropdown(Box<Notifications>),
}

struct Card {
    title: &'static str,
    sample: Sample,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct NotificationsPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The dropdown's row a reader focused, scrolled in at the next draw.
    reveal: Option<usize>,
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    let group = child_id(Some(root), "group")?.key(g.to_string());
    child_id(Some(&group), "card").map(|f| f.key(c.to_string()))
}

/// Where a part a reader names is.
enum At {
    Alert,
    Action(usize),
    Filter(usize),
    Row(usize),
    RowAction(usize, usize),
}

impl Default for NotificationsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationsPage {
    pub fn new() -> Self {
        let kit = kit_notifications();
        let variation = |i: usize| Sample::Alert(Box::new(kit[i].clone()));
        let mut groups = vec![
            Group {
                title: "Variations",
                columns: 2,
                cards: vec![
                    Card {
                        title: "Comment",
                        sample: variation(0),
                    },
                    Card {
                        title: "Image generated",
                        sample: variation(1),
                    },
                    Card {
                        title: "Invite",
                        sample: variation(2),
                    },
                    Card {
                        title: "Like",
                        sample: variation(3),
                    },
                    Card {
                        title: "Video generated",
                        sample: variation(5),
                    },
                ],
            },
            Group {
                title: "Notification Dropdown",
                columns: 1,
                cards: vec![
                    Card {
                        title: "Notifications",
                        sample: Sample::Dropdown(Box::new(Notifications::new(kit_notifications()))),
                    },
                    Card {
                        title: "Empty: nothing yet",
                        sample: Sample::Dropdown(Box::new(Notifications::new(Vec::new()))),
                    },
                ],
            },
        ];
        let root = root_id("notifications_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(cid) = card_id(root, g, c) else {
                        continue;
                    };
                    match &mut card.sample {
                        Sample::Alert(a) => {
                            if let Some(id) = child_id(Some(&cid), "alert") {
                                **a = (**a).clone().sem(id);
                            }
                        }
                        Sample::Dropdown(d) => {
                            if let Some(id) = child_id(Some(&cid), "dropdown") {
                                **d = (**d).clone().sem(id);
                            }
                        }
                    }
                }
            }
        }
        Self {
            root,
            groups,
            reveal: None,
        }
    }

    fn size(sample: &Sample, w: f32, theme: &Theme) -> (f32, f32) {
        let avail = (w - theme.spacing.md * 2.0).max(0.0);
        match sample {
            Sample::Alert(a) => {
                let w = theme.message.notifications.row_w.min(avail);
                (w, a.height(w, theme))
            }
            Sample::Dropdown(d) => d.size(avail, theme),
        }
    }

    /// Per group its layout, and per card where its widget goes.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Rect>)> {
        let sized = self.groups.iter().map(|g| {
            let w = showcase::card_width(g.columns, content, theme);
            let cards = g
                .cards
                .iter()
                .map(|c| showcase::card_height(Self::size(&c.sample, w, theme).1, theme));
            (g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let boxes = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(r, c)| showcase::centered(Self::size(&c.sample, r.w, theme), *r))
                    .collect();
                (placed, boxes)
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let layout: Vec<_> = self
            .layout(content, theme)
            .into_iter()
            .map(|(g, _)| g)
            .collect();
        showcase::content_height(&layout, content)
    }

    /// The dropdown's bar fade. `true` while it animates.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for card in self.groups.iter_mut().flat_map(|g| &mut g.cards) {
            if let Sample::Dropdown(d) = &mut card.sample {
                moving |= d.tick(dt);
            }
        }
        moving
    }

    /// A press lets the reader's focus go; then the widget in the card
    /// under the pointer takes it. Every widget takes a move, so a hover
    /// can leave.
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
        let (px, py) = event.pos();
        let every = matches!(event, WidgetEvent::MouseMove { .. });
        let layout = self.layout(content, theme);
        for (group, (placed, boxes)) in self.groups.iter_mut().zip(&layout) {
            for ((card, rect), at) in group.cards.iter_mut().zip(&placed.cards).zip(boxes) {
                if !every && !rect.contains(px, py) {
                    continue;
                }
                let r = match &mut card.sample {
                    Sample::Alert(a) => a.handle_event(event, *at, theme),
                    Sample::Dropdown(d) => d.handle_event(event, *at, theme),
                };
                result = result.merge(r);
            }
        }
        result
    }

    /// Scrolls the row a reader focused into the dropdown's view, once it
    /// is laid out.
    fn apply_reveal(&mut self, content: Rect, theme: &Theme) -> bool {
        let Some(row) = self.reveal.take() else {
            return false;
        };
        let layout = self.layout(content, theme);
        let mut moved = false;
        for (group, (_, boxes)) in self.groups.iter_mut().zip(&layout) {
            for (card, at) in group.cards.iter_mut().zip(boxes) {
                if let Sample::Dropdown(d) = &mut card.sample {
                    moved |= d.reveal(row, *at, theme);
                }
            }
        }
        moved
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        self.apply_reveal(content, theme);
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, boxes))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), at) in group.cards.iter().zip(&placed.cards).zip(boxes) {
                if let Sample::Dropdown(_) = card.sample {
                    // the kit's gray card the dropdown sits on
                    let well = theme.message.notifications.well.0;
                    let r = *rect;
                    c.push(rounded_rect(r.x, r.y, r.w, r.h, theme.shape.card, well));
                }
                showcase::card(c, *rect, card.title, theme);
                match &card.sample {
                    Sample::Alert(a) => a.render(c, *at, theme),
                    Sample::Dropdown(d) => d.render(c, *at, theme),
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }

    /// The card whose widget `id` names a part of, and which part.
    fn find(&mut self, id: &SemId) -> Option<(&mut Sample, At)> {
        for card in self.groups.iter_mut().flat_map(|g| &mut g.cards) {
            let at = match &card.sample {
                Sample::Alert(a) => {
                    let own = a.sem_id()?;
                    if own == id {
                        Some(At::Alert)
                    } else {
                        let family = own.child("action").ok()?;
                        key_under(&family, id)
                            .and_then(|k| k.parse().ok())
                            .map(At::Action)
                    }
                }
                Sample::Dropdown(d) => {
                    let own = d.sem_id()?;
                    let filters = own.child("filter").ok()?;
                    let list = own.child("list").ok()?;
                    let rest = id.as_str().strip_prefix(list.as_str());
                    if let Some(i) = key_under(&filters, id).and_then(|k| k.parse().ok()) {
                        Some(At::Filter(i))
                    } else if let Some(rest) = rest.and_then(|r| r.strip_prefix('[')) {
                        let (row, tail) = rest.split_once(']')?;
                        let row = row.parse().ok()?;
                        match tail
                            .strip_prefix(".action[")
                            .and_then(|t| t.strip_suffix(']'))
                        {
                            Some(j) => Some(At::RowAction(row, j.parse().ok()?)),
                            None if tail.is_empty() => Some(At::Row(row)),
                            None => None,
                        }
                    } else {
                        None
                    }
                }
            };
            if let Some(at) = at {
                return Some((&mut card.sample, at));
            }
        }
        None
    }
}

impl Component for NotificationsPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let mut focused = None;
        for (g, group) in self.groups.iter().enumerate() {
            let Some(gid) = child_id(Some(root), "group").map(|f| f.key(g.to_string())) else {
                continue;
            };
            b.node(&gid, Role::Group, group.title).parent(root);
            for (c, card) in group.cards.iter().enumerate() {
                let Some(cid) = card_id(root, g, c) else {
                    continue;
                };
                b.node(&cid, Role::Group, card.title).parent(&gid);
                match &card.sample {
                    Sample::Alert(a) => {
                        if let Some(id) = a.sem_id() {
                            a.project(b, id, Some(&cid));
                        }
                        focused = focused.or_else(|| alert_focus(a));
                    }
                    Sample::Dropdown(d) => {
                        if let Some(id) = d.sem_id() {
                            d.project(b, id, Some(&cid));
                        }
                        let filter = d.focused_filter().and_then(|i| d.filter_id(i));
                        focused = focused.or(filter).or_else(|| {
                            d.shown()
                                .into_iter()
                                .find_map(|i| alert_focus(&d.alerts()[i]))
                        });
                    }
                }
            }
        }
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    fn run(&mut self, req: &Request) -> Result<After, String> {
        let action = req.action.as_str();
        let refuse = || format!("the notifications page has no {action} on {}", req.target);
        let Some((sample, at)) = self.find(&req.target) else {
            return Err(refuse());
        };
        let done = match (sample, at, action) {
            (Sample::Alert(a), At::Alert, "activate") => {
                a.open();
                true
            }
            (Sample::Alert(a), At::Action(j), "activate") => a.choose(j),
            (Sample::Dropdown(d), At::Filter(i), "select") => d.set_filter(i),
            (Sample::Dropdown(d), At::Row(i), "activate") => {
                let read = d.shown_mut(i).map(|a| a.open()).is_some();
                d.settle();
                read
            }
            (Sample::Dropdown(d), At::RowAction(i, j), "activate") => {
                let chose = d.shown_mut(i).is_some_and(|a| a.choose(j));
                d.settle();
                chose
            }
            _ => false,
        };
        if done {
            Ok(After::Nothing)
        } else {
            Err(refuse())
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.blur();
        let mut reveal = None;
        if let Some((sample, at)) = self.find(id) {
            match (sample, at) {
                (Sample::Alert(a), At::Alert) => a.set_focused(true),
                (Sample::Alert(a), At::Action(j)) => a.set_action_focused(Some(j)),
                (Sample::Dropdown(d), At::Filter(i)) => d.set_filter_focused(Some(i)),
                (Sample::Dropdown(d), At::Row(i)) => {
                    if let Some(a) = d.shown_mut(i) {
                        a.set_focused(true);
                    }
                    reveal = Some(i);
                }
                (Sample::Dropdown(d), At::RowAction(i, j)) => {
                    if let Some(a) = d.shown_mut(i) {
                        a.set_action_focused(Some(j));
                    }
                    reveal = Some(i);
                }
                _ => {}
            }
        }
        self.reveal = reveal;
        true
    }

    fn blur(&mut self) -> bool {
        // lets an alert's focus go; `true` when it had it
        fn drop(a: &mut Alert) -> bool {
            let had = a.is_focused() || a.focused_action().is_some();
            a.set_focused(false);
            a.set_action_focused(None);
            had
        }
        let mut had = false;
        for card in self.groups.iter_mut().flat_map(|g| &mut g.cards) {
            match &mut card.sample {
                Sample::Alert(a) => had |= drop(a),
                Sample::Dropdown(d) => {
                    had |= d.focused_filter().is_some();
                    d.set_filter_focused(None);
                    for i in 0..d.shown().len() {
                        if let Some(a) = d.shown_mut(i) {
                            had |= drop(a);
                        }
                    }
                }
            }
        }
        had
    }
}

/// The id of what a reader focused in `a`: the alert or one of its
/// answers.
fn alert_focus(a: &Alert) -> Option<SemId> {
    if a.is_focused() {
        return a.sem_id().cloned();
    }
    let j = a.focused_action()?;
    a.action_ids().nth(j).flatten().cloned()
}

#[cfg(test)]
mod tests;
