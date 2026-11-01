//! The button group of the hoff core kit: the toolbar shell
//! (`components/Toolbar`) holding actions and toggles in a row, sections
//! split by a hairline. It owns no press of its own: an action is the
//! kit's toolbar [`IconButton`], a selectable item is a [`Toggle`] (the
//! toolbar action with `active`), and each presses as it does alone. The
//! group lays them out, draws the shell and, when `exclusive`, keeps one
//! toggle on (the kit's tool picker). Every value is
//! `theme.selection.button_group`.
//!
//! The layout is the composition's: the items flow in a row and wrap
//! onto the next one when the width runs out (a separator never starts a
//! row), so the group fits any width down to its widest item, and its
//! height follows ([`ButtonGroup::size_within`]).
//!
//! With a [`SemId`] ([`ButtonGroup::sem`]) the render binds the shell,
//! and [`ButtonGroup::project`] declares a `Role::Toolbar` with each item
//! under it as it projects itself.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::toggle::Toggle;
use crate::widgets::buttons::{IconButton, paint_surface};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// One thing in the group.
#[derive(Clone, Debug)]
pub enum GroupItem {
    /// A plain action, named for a reader (an icon names nothing).
    Action { button: IconButton, label: String },
    /// A selectable item.
    Toggle(Toggle),
    /// The hairline between two sections.
    Separator,
}

impl GroupItem {
    /// The item's size in a row `room` wide: its own, except a labeled
    /// toggle wider than the row, which truncates to it (down to its
    /// ellipsis).
    fn size(&self, theme: &Theme, room: f32) -> (f32, f32) {
        match self {
            GroupItem::Action { button, .. } => button.preferred_size(theme),
            GroupItem::Toggle(t) => {
                let (w, h) = t.preferred_size(theme);
                (w.min(room).max(t.min_width(theme)), h)
            }
            GroupItem::Separator => (theme.selection.button_group.separator_w, 0.0),
        }
    }

    /// The narrowest the item gets.
    fn min_width(&self, theme: &Theme) -> f32 {
        match self {
            GroupItem::Toggle(t) => t.min_width(theme),
            other => other.size(theme, f32::INFINITY).0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ButtonGroup {
    pub items: Vec<GroupItem>,
    /// One toggle on at a time: pressing one turns the others off, and
    /// pressing the one on keeps it on.
    pub exclusive: bool,
    sem: Option<SemId>,
}

impl ButtonGroup {
    pub fn new(items: impl IntoIterator<Item = GroupItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            exclusive: false,
            sem: None,
        }
    }

    pub fn exclusive(mut self, exclusive: bool) -> Self {
        self.exclusive = exclusive;
        self
    }

    /// The logical identity the render binds the shell to.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Press item `index` as a click on it does: an action presses, a
    /// toggle flips, or, in an exclusive group, turns on and the others
    /// off. Returns whether anything changed state (an action changes
    /// none); a disabled item or an index out of range changes nothing.
    pub fn press(&mut self, index: usize) -> bool {
        let exclusive = self.exclusive;
        let Some(GroupItem::Toggle(t)) = self.items.get_mut(index) else {
            return false;
        };
        if t.disabled {
            return false;
        }
        if !exclusive {
            return t.toggle();
        }
        if t.on {
            return false;
        }
        t.on = true;
        for (i, item) in self.items.iter_mut().enumerate() {
            if let GroupItem::Toggle(other) = item
                && i != index
            {
                other.on = false;
            }
        }
        true
    }

    /// The item rows within `width`: each row its items' indices and the
    /// row's height. A separator that would start a row is left out.
    fn rows(&self, width: f32, theme: &Theme) -> Vec<(Vec<usize>, f32)> {
        let t = &theme.selection.button_group;
        let room = (width - t.pad * 2.0).max(0.0);
        let mut rows: Vec<(Vec<usize>, f32)> = Vec::new();
        let mut x = 0.0;
        for (i, item) in self.items.iter().enumerate() {
            let (w, h) = item.size(theme, room);
            let fits = rows
                .last()
                .is_some_and(|(r, _)| !r.is_empty() && x + t.gap + w <= room);
            if fits {
                x += t.gap + w;
            } else if matches!(item, GroupItem::Separator) {
                continue;
            } else {
                rows.push((Vec::new(), 0.0));
                x = w;
            }
            if let Some((row, row_h)) = rows.last_mut() {
                row.push(i);
                *row_h = row_h.max(h);
            }
        }
        // A separator left at a row's end separates nothing there.
        for (row, _) in &mut rows {
            while row
                .last()
                .is_some_and(|&i| matches!(self.items[i], GroupItem::Separator))
            {
                row.pop();
            }
        }
        rows
    }

    /// The group's size within `max_w`: one row when it fits, more as the
    /// width runs out; never narrower than its widest item.
    pub fn size_within(&self, max_w: f32, theme: &Theme) -> (f32, f32) {
        let t = &theme.selection.button_group;
        let width = max_w.max(self.min_width(theme));
        let room = width - t.pad * 2.0;
        let rows = self.rows(width, theme);
        let mut w: f32 = 0.0;
        let mut h = t.pad * 2.0;
        for (n, (row, row_h)) in rows.iter().enumerate() {
            let row_w: f32 = row
                .iter()
                .map(|&i| self.items[i].size(theme, room).0)
                .sum::<f32>()
                + t.gap * row.len().saturating_sub(1) as f32;
            w = w.max(row_w);
            h += row_h + if n > 0 { t.gap } else { 0.0 };
        }
        ((w + t.pad * 2.0).ceil(), h)
    }

    /// One row of everything.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        self.size_within(f32::INFINITY, theme)
    }

    /// The widest item at its narrowest, in the shell's padding: the
    /// narrowest the group gets.
    pub fn min_width(&self, theme: &Theme) -> f32 {
        let t = &theme.selection.button_group;
        let widest = self
            .items
            .iter()
            .map(|i| i.min_width(theme))
            .fold(0.0, f32::max);
        (widest + t.pad * 2.0).ceil()
    }

    /// Each item's rect in `bounds`, `None` for a separator the layout
    /// left out. A separator spans its row and the padding (or half the
    /// gap) above and below.
    pub fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Option<Rect>> {
        let t = &theme.selection.button_group;
        let rows = self.rows(bounds.w, theme);
        let room = (bounds.w - t.pad * 2.0).max(0.0);
        let mut out = vec![None; self.items.len()];
        let mut y = bounds.y + t.pad;
        let last = rows.len().saturating_sub(1);
        for (n, (row, row_h)) in rows.iter().enumerate() {
            let mut x = bounds.x + t.pad;
            for &i in row {
                let (w, h) = self.items[i].size(theme, room);
                out[i] = Some(if matches!(self.items[i], GroupItem::Separator) {
                    let above = if n == 0 { t.pad } else { t.gap / 2.0 };
                    let below = if n == last { t.pad } else { t.gap / 2.0 };
                    Rect::new(x, y - above, w, row_h + above + below)
                } else {
                    Rect::new(x, y + (row_h - h) / 2.0, w, h)
                });
                x += w + t.gap;
            }
            y += row_h + t.gap;
        }
        out
    }

    /// Pointer events: each item presses on its own; a click on a toggle
    /// in an exclusive group turns it on and the others off.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rects = self.item_rects(bounds, theme);
        let mut result = EventResult::IGNORED;
        let mut clicked = None;
        for (i, (item, rect)) in self.items.iter_mut().zip(&rects).enumerate() {
            let Some(rect) = rect else { continue };
            let r = match item {
                GroupItem::Action { button, .. } => button.handle_event(event, *rect),
                GroupItem::Toggle(t) if self.exclusive => {
                    // The group decides what a click does to the set.
                    let was = t.on;
                    let r = t.handle_event(event, *rect);
                    t.on = was;
                    r
                }
                GroupItem::Toggle(t) => t.handle_event(event, *rect),
                GroupItem::Separator => EventResult::IGNORED,
            };
            if r.clicked {
                clicked = Some(i);
            }
            result = result.merge(r);
        }
        if self.exclusive
            && let Some(i) = clicked
        {
            self.press(i);
        }
        result
    }

    /// Declares the group as `id`: a `Role::Toolbar` named `label`, with
    /// each item that has an id projected under it as it projects itself
    /// (an action as a button named by its label, a toggle as a toggle).
    /// Returns the group's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for item in &self.items {
            match item {
                GroupItem::Action { button, label } => {
                    if let Some(at) = button.sem_id() {
                        button.project(b, at, label.as_str(), Some(id));
                    }
                }
                GroupItem::Toggle(t) => {
                    if let Some(at) = t.sem_id() {
                        t.project(b, at, Some(id));
                    }
                }
                GroupItem::Separator => {}
            }
        }
        let mut node = b.node(id, Role::Toolbar, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.selection.button_group;
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let radius = t.radius.min(bounds.h / 2.0);
        paint_surface(
            c,
            LayerId::DEFAULT,
            bounds,
            radius,
            &t.shell,
            1.0,
            false,
            theme,
        );
        for (item, rect) in self.items.iter().zip(self.item_rects(bounds, theme)) {
            let Some(rect) = rect else { continue };
            match item {
                GroupItem::Action { button, .. } => button.render(c, rect, theme),
                GroupItem::Toggle(toggle) => toggle.render(c, rect, theme),
                GroupItem::Separator => c.push(SceneNode::Rect {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                    color: t.separator.0,
                }),
            }
        }
    }
}
