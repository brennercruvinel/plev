//! The notifications dropdown of the hoff core kit
//! (`components/Notifications`): on the kit's card (the toast's surface),
//! a header with the title and two filters, All and Unread, over a
//! hairline; under it the notifications (the inline alert, the kit's
//! `Notification`), a hairline between two, scrolling once they outgrow
//! the card.
//!
//! Sized for ranges: as wide as the kit's `w-96` or the column, as tall
//! as what it holds up to the kit's `h-200` (`notifications.max_h`), then
//! the list scrolls under the wheel, a touch drag and the keyboard (one
//! scroll state, the engine's, with its fading bar). The filter shows
//! every notification or the unread ones; the rows take their ids by the
//! place they are shown at, so a list's rows are keyed `0..count`.
//!
//! Behavior apart from the look: a filter switches on a click or a
//! reader's `select`; a notification is read by a click or `activate`,
//! and its answers chosen, as the alert does. Every visual value is
//! `theme.message.notifications` and the toast's surface; the rows are
//! the alert's.
//!
//! Its parts take `<id>.filter[<0|1>]` and `<id>.list[<i>]` (with the
//! alert's own `.action[<j>]`).

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::select_ai::line;
use crate::control::input::scroll::ScrollState;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::{Intent, Theme};
use crate::widgets::buttons::{ButtonVariant, Press};
use crate::widgets::content::EmptyState;
use crate::widgets::feedback::Scrollbar;
use crate::widgets::feedback::alert::{Alert, NotificationKind};
use crate::widgets::message::message::width;
use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke, shadow_drops};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The filters (`components/Notifications/index.tsx:13-22`).
pub const FILTERS: [&str; 2] = ["All", "Unread"];

const DRAGGABLE: &str = "These draggabale sliders look really cool. Maybe these could be \
     displayed when you hold shift, to rotate exactly on the X or Y. But by default I don't \
     think we need these controllers, you could just rotate the object by clicking and \
     dragging anywhere as expected on any 3D tool (theoretically).";

/// The kit's notifications (`components/Notifications/content.tsx`), each
/// as the inline alert of its kind.
pub fn kit_notifications() -> Vec<Alert> {
    let comment = |title: &str, time: &str, who: &str, unread: bool| {
        Alert::new(Intent::Neutral, title, "Commented on Classic Car in Studio")
            .kind(NotificationKind::Comment)
            .time(time)
            .avatar(who)
            .content(DRAGGABLE)
            .unread(unread)
    };
    let image = |unread: bool| {
        Alert::new(
            Intent::Informational,
            "Cute Turtle is generated",
            "Matte texture - Hoff Style",
        )
        .kind(NotificationKind::Image)
        .time("2h ago")
        .avatar("Hoff")
        .content("Prompt: Create 3D character dancing")
        .unread(unread)
    };
    vec![
        comment("Randomdash", "1h ago", "Randomdash", true),
        image(true),
        Alert::new(
            Intent::Constructive,
            "3D object is generated",
            "Invited you to edit Minimalist Architecture Scene",
        )
        .kind(NotificationKind::Invite)
        .time("4h ago")
        .avatar("Hoff")
        .unread(true)
        .action("Decline", ButtonVariant::Primary)
        .action("Accept", ButtonVariant::Secondary),
        Alert::new(
            Intent::Informational,
            "Marina",
            "Liked Classic Car in Studio",
        )
        .kind(NotificationKind::Like)
        .time("10h ago")
        .avatar("Marina"),
        comment("3D object is generated", "16h ago", "Ana", false),
        Alert::new(
            Intent::Informational,
            "Animation is generated",
            "12s – 720p",
        )
        .kind(NotificationKind::Video)
        .time("1d ago")
        .avatar("Hoff")
        .content("Prompt: Create 3D character dancing"),
        comment("Randomdash", "1h ago", "Randomdash", true),
        image(true),
    ]
}

/// Where the header's parts and the list land.
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationsLayout {
    pub card: Rect,
    pub title: Rect,
    pub filters: [Rect; 2],
    pub rule: Rect,
    /// The list's viewport.
    pub list: Rect,
}

#[derive(Clone, Debug)]
pub struct Notifications {
    alerts: Vec<Alert>,
    filter: usize,
    presses: [Press; 2],
    scroll: ScrollState,
    bar: Scrollbar,
    sem: Option<SemId>,
}

impl Notifications {
    pub fn new(alerts: Vec<Alert>) -> Self {
        Self {
            alerts,
            filter: 0,
            presses: [Press::default(); 2],
            scroll: ScrollState::new(),
            bar: Scrollbar::new(),
            sem: None,
        }
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self.reid();
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn filter(&self) -> usize {
        self.filter
    }

    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }

    /// The indices of the notifications the filter shows, in order.
    pub fn shown(&self) -> Vec<usize> {
        (0..self.alerts.len())
            .filter(|i| self.filter == 0 || self.alerts[*i].is_unread())
            .collect()
    }

    /// The alert shown at `pos`.
    pub fn shown_mut(&mut self, pos: usize) -> Option<&mut Alert> {
        let i = *self.shown().get(pos)?;
        self.alerts.get_mut(i)
    }

    /// Each shown row takes `<id>.list[<pos>]`, the first without the
    /// hairline above it.
    fn reid(&mut self) {
        let list = self.sem.as_ref().and_then(|id| id.child("list").ok());
        for (pos, i) in self.shown().into_iter().enumerate() {
            let a = self.alerts[i].clone().divider(pos > 0).listed(true);
            self.alerts[i] = match &list {
                Some(list) => a.sem(list.key(pos.to_string())),
                None => a,
            };
        }
    }

    /// Shows filter `i` (`0` all, `1` unread), the list back at its top.
    /// `false` for one it does not have.
    pub fn set_filter(&mut self, i: usize) -> bool {
        if i >= FILTERS.len() {
            return false;
        }
        for a in &mut self.alerts {
            a.set_focused(false);
            a.set_action_focused(None);
        }
        self.filter = i;
        self.scroll = ScrollState::new();
        self.reid();
        true
    }

    /// Reads what changed the unread set: under the Unread filter a row
    /// read leaves the list, and the others take its place's ids.
    pub fn settle(&mut self) {
        self.reid();
    }

    fn header_h(theme: &Theme) -> f32 {
        let t = &theme.message.notifications;
        let tab = t.tab.line_height(&theme.typography) + t.tab_pad_y * 2.0;
        t.header_pad_y * 2.0
            + t.title.line_height(&theme.typography).max(tab)
            + theme.message.divider_width
    }

    /// The shown rows' height, all of them, `w` wide; with none, the
    /// empty state's with the theme's `lg` clear around it.
    pub fn content_h(&self, w: f32, theme: &Theme) -> f32 {
        let shown = self.shown();
        if shown.is_empty() {
            let state = self
                .empty_state()
                .height(Rect::new(0.0, 0.0, w, 0.0), theme);
            return state + theme.spacing.lg * 2.0;
        }
        shown
            .into_iter()
            .map(|i| self.alerts[i].height(w, theme))
            .sum()
    }

    /// What the list says while the filter shows nothing: none yet under
    /// All, all read under Unread.
    pub fn empty_state(&self) -> EmptyState {
        let (title, message) = if self.filter == 0 {
            (
                "No notifications yet",
                "Comments, invites and likes land here.",
            )
        } else {
            ("You're all caught up", "Nothing waits to be read.")
        };
        EmptyState::new(title, message).icon("kit-notifications")
    }

    /// The dropdown's size in a column `avail` wide.
    pub fn size(&self, avail: f32, theme: &Theme) -> (f32, f32) {
        let t = &theme.message.notifications;
        let edge = theme.message.toast.border_width;
        let w = t.w.min(avail).max(0.0);
        let inner = (w - edge * 2.0).max(0.0);
        let h = edge * 2.0 + Self::header_h(theme) + self.content_h(inner, theme);
        (w, h.min(t.max_h))
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> NotificationsLayout {
        let t = &theme.message.notifications;
        let edge = theme.message.toast.border_width;
        let (w, h) = self.size(bounds.w, theme);
        let card = Rect::new(bounds.x, bounds.y, w, h);
        let inner = card.inset(edge);
        let tab_h = t.tab.line_height(&theme.typography) + t.tab_pad_y * 2.0;
        let row_h = t.title.line_height(&theme.typography).max(tab_h);
        let top = inner.y + t.header_pad_y;
        let style = t.tab.style(&theme.typography);
        let mut x = inner.right() - t.header_pad_x;
        let mut filters = [Rect::default(); 2];
        for (i, label) in FILTERS.iter().enumerate().rev() {
            let fw = width(label, &style).ceil() + t.tab_pad_x * 2.0;
            x -= fw;
            filters[i] = Rect::new(x, top + (row_h - tab_h) / 2.0, fw, tab_h);
            x -= t.tab_gap;
        }
        let title_x = inner.x + t.header_pad_x;
        let title = Rect::new(
            title_x,
            top + (row_h - t.title.line_height(&theme.typography)) / 2.0,
            (filters[0].x - t.tab_gap - title_x).max(0.0),
            t.title.line_height(&theme.typography),
        );
        let rule_y = top + row_h + t.header_pad_y;
        let rule = Rect::new(inner.x, rule_y, inner.w, theme.message.divider_width);
        let list_y = rule.bottom();
        let list = Rect::new(inner.x, list_y, inner.w, (inner.bottom() - list_y).max(0.0));
        NotificationsLayout {
            card,
            title,
            filters,
            rule,
            list,
        }
    }

    /// Each shown row's rect at the current scroll, with its index.
    pub fn rows(&self, list: Rect, theme: &Theme) -> Vec<(usize, Rect)> {
        let mut y = list.y - self.scroll.offset();
        self.shown()
            .into_iter()
            .map(|i| {
                let r = Rect::new(list.x, y, list.w, self.alerts[i].height(list.w, theme));
                y = r.bottom();
                (i, r)
            })
            .collect()
    }

    fn sync(&mut self, list: Rect, theme: &Theme) {
        self.scroll.set_viewport(list.h);
        self.scroll.set_content(self.content_h(list.w, theme));
    }

    /// Scrolls the least that shows the row at `pos`. `true` when it
    /// moved.
    pub fn reveal(&mut self, pos: usize, bounds: Rect, theme: &Theme) -> bool {
        let list = self.layout(bounds, theme).list;
        self.sync(list, theme);
        let Some((_, row)) = self.rows(list, theme).get(pos).copied() else {
            return false;
        };
        let old = self.scroll.offset();
        if row.y < list.y || row.h > list.h {
            self.scroll.scroll_by(row.y - list.y);
        } else if row.bottom() > list.bottom() {
            self.scroll.scroll_by(row.bottom() - list.bottom());
        }
        let moved = self.scroll.offset() != old;
        if moved {
            self.bar.notify_scroll();
        }
        moved
    }

    /// The filters take their clicks; the wheel scrolls the list while it
    /// can; the rows shown take the pointer inside the list.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        self.sync(l.list, theme);
        let mut result = self
            .bar
            .handle_event(event, l.list, &mut self.scroll, theme);
        if result.handled {
            return result;
        }
        for i in 0..FILTERS.len() {
            let r = self.presses[i].handle(event, l.filters[i], false);
            if r.clicked {
                self.set_filter(i);
            }
            result = result.merge(r);
        }
        if let WidgetEvent::Scroll { x, y, delta } = *event {
            if !l.list.contains(x, y) {
                return result;
            }
            let old = self.scroll.offset();
            self.scroll.scroll_by(delta);
            if self.scroll.offset() == old {
                return result;
            }
            self.bar.notify_scroll();
            return EventResult::changed();
        }
        let (px, py) = event.pos();
        let inside = l.list.contains(px, py);
        let rows = self.rows(l.list, theme);
        let unread_before = self.shown();
        for (i, r) in rows {
            if inside || matches!(event, WidgetEvent::MouseMove { .. }) {
                let ev = if inside {
                    *event
                } else {
                    WidgetEvent::MouseMove {
                        x: f32::MIN,
                        y: f32::MIN,
                    }
                };
                result = result.merge(self.alerts[i].handle_event(&ev, r, theme));
            }
        }
        if self.shown() != unread_before {
            self.settle();
        }
        result
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.bar.tick(dt)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let layer = LayerId::DEFAULT;
        let t = &theme.message.notifications;
        let s = &theme.message.toast;
        let l = self.layout(bounds, theme);
        let k = l.card;
        if let Some(id) = &self.sem {
            c.bind(id, [k.x, k.y, k.w, k.h]);
        }
        shadow_drops(c, layer, k, s.radius, std::slice::from_ref(&s.shadow));
        c.push(rounded_rect(k.x, k.y, k.w, k.h, s.radius, s.fill.0));
        c.push(rounded_rect_stroke(
            k.x,
            k.y,
            k.w,
            k.h,
            s.radius,
            s.border.0,
            s.border_width,
        ));
        let ti = l.title;
        line(c, layer, "Notifications", &t.title, ti.x, ti.y, ti.w, theme);
        for (i, (label, r)) in FILTERS.iter().zip(l.filters).enumerate() {
            if let Some(id) = self.filter_id(i) {
                c.bind(&id, [r.x, r.y, r.w, r.h]);
            }
            let on = self.filter == i;
            if on {
                c.push(rounded_rect(r.x, r.y, r.w, r.h, t.tab_radius, t.tab_fill.0));
            }
            if self.presses[i].focused {
                c.push(focus_ring(r, t.tab_radius, theme));
            }
            let run = if on || self.presses[i].hovered {
                super::tokens::MessageText {
                    color: t.tab_active,
                    ..t.tab
                }
            } else {
                t.tab
            };
            let x = r.x + t.tab_pad_x;
            line(c, layer, label, &run, x, r.y + t.tab_pad_y, r.w, theme);
        }
        let rule = l.rule;
        c.push(SceneNode::Rect {
            x: rule.x,
            y: rule.y,
            w: rule.w,
            h: rule.h,
            color: theme.message.divider.0,
        });
        let list = l.list;
        c.push_clip(list.x, list.y, list.w, list.h);
        if self.shown().is_empty() {
            self.empty_state().render(c, list, theme);
        }
        for (i, r) in self.rows(list, theme) {
            if r.bottom() > list.y && r.y < list.bottom() {
                self.alerts[i].render(c, r, theme);
            }
        }
        c.pop_clip();
        self.bar.render(c, list, &self.scroll, theme);
    }

    /// `<id>.filter[<i>]`.
    pub fn filter_id(&self, i: usize) -> Option<SemId> {
        let id = self.sem.as_ref()?.child("filter").ok()?;
        Some(id.key(i.to_string()))
    }

    pub fn set_filter_focused(&mut self, i: Option<usize>) {
        for (j, p) in self.presses.iter_mut().enumerate() {
            p.focused = i == Some(j);
        }
    }

    pub fn focused_filter(&self) -> Option<usize> {
        self.presses.iter().position(|p| p.focused)
    }

    /// Declares the dropdown as `id`: a `Role::Group` named
    /// "Notifications"; its filters (a tab list, `selected` the chosen
    /// one's index, each tab with `select`); the shown notifications (a
    /// list with its `count`, each the alert's node). Returns the group's
    /// node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        if let Ok(tabs) = id.child("filter") {
            let reads = format!("{tabs}:selected");
            for (i, label) in FILTERS.iter().enumerate() {
                let mut tab = b.node(tabs.key(i.to_string()), Role::Tab, *label);
                tab.parent(&tabs);
                tab.action("select").reads(reads.as_str());
            }
            b.node(&tabs, Role::TabList, "Show")
                .parent(id)
                .state("selected", self.filter.to_string());
        }
        if let Ok(list) = id.child("list") {
            let shown = self.shown();
            b.node(&list, Role::List, "Notifications")
                .parent(id)
                .state("count", i64::try_from(shown.len()).unwrap_or(i64::MAX));
            for i in shown {
                let a = &self.alerts[i];
                if let Some(aid) = a.sem_id() {
                    a.project(b, aid, Some(&list));
                }
            }
        }
        let mut node = b.node(id, Role::Group, "Notifications");
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }
}
