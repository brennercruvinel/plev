use std::ops::Range;

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::control::input::scroll::ScrollState;
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::widget::{EventResult, Rect, WidgetEvent};
use crate::widgets::feedback::Scrollbar;
use crate::widgets::recipe::focus_ring_inside;

/// Rows tessellated beyond the viewport on each side, so partially
/// visible rows and small scroll deltas don't pop.
const OVERSCAN: usize = 2;

/// How far a row edge may miss the viewport's edge, in rows, and still
/// count as inside it (the float error of an offset that lines a row up
/// with the edge).
const EDGE: f32 = 1e-3;

/// Generic virtualized list: only visible rows are rendered, whatever the
/// item count. Owns scroll state, a fading [`Scrollbar`], hover,
/// selection and the keyboard focus on a row; the row *content* is the
/// caller's via a render closure.
///
/// With a [`SemId`] ([`VirtualList::sem`]) the render binds the list and
/// each row it holds to the rects it hit-tests them with, and
/// [`VirtualList::project`] declares the list in a semantic projection.
///
/// ```rust
/// # use comps::content::VirtualList;
/// # use comps::core::Rect;
/// # use engine::compositor::Compositor;
/// # use engine::theme::Theme;
/// let mut list = VirtualList::new(24.0);
/// list.set_item_count(10_000);
/// let bounds = Rect::new(0.0, 0.0, 300.0, 240.0);
/// let mut compositor = Compositor::new();
/// let theme = Theme::dark();
/// list.render_with(&mut compositor, bounds, &theme, |c, index, row, hovered, selected| {
///     // draw row `index` into `row` bounds
/// });
/// ```
#[derive(Clone, Debug)]
pub struct VirtualList {
    pub item_height: f32,
    pub scroll: ScrollState,
    pub scrollbar: Scrollbar,
    pub selected: Option<usize>,
    item_count: usize,
    hovered: Option<usize>,
    focused: Option<usize>,
    viewport: Rect,
    sem: Option<SemId>,
}

/// The part of a full-width `row` inside `bounds`, the part a pointer
/// reaches (rows overscan past the bounds, the hit test does not); `None`
/// when none of it is inside.
pub(super) fn reachable(row: Rect, bounds: Rect) -> Option<Rect> {
    let top = row.y.max(bounds.y);
    let bottom = (row.y + row.h).min(bounds.y + bounds.h);
    (bottom >= top).then(|| Rect::new(row.x, top, row.w, bottom - top))
}

/// The key a list names its selected row by: its index, or empty.
fn selected_key(selected: Option<usize>) -> String {
    selected.map(|i| i.to_string()).unwrap_or_default()
}

fn count(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

impl VirtualList {
    pub fn new(item_height: f32) -> Self {
        Self {
            item_height: item_height.max(1.0),
            scroll: ScrollState::new(),
            scrollbar: Scrollbar::new(),
            selected: None,
            item_count: 0,
            hovered: None,
            focused: None,
            viewport: Rect::default(),
            sem: None,
        }
    }

    /// The logical identity the render binds the list to
    /// (`Compositor::bind_to_layer`); each row it holds binds as
    /// `<id>[<index>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this list as `id` in a projection: a `Role::List` named
    /// `label` with its item `count` and the key of its `selected` row
    /// (the index, empty when none), and one `Role::ListItem` child
    /// `<id>[<index>]` per row it holds ([`VirtualList::visible_range`]),
    /// never the whole dataset, each named by `row_label` (the rows are
    /// the app's, so is their text) and with a `select` that reads the
    /// list's `selected`. One row takes the focus, the list's one stop in
    /// the Tab sequence ([`VirtualList::tab_stop`]); the arrow keys move
    /// the focus along the others. Scroll and hover are never projected;
    /// which rows the list holds follows the scroll. It only reads the
    /// widget; the
    /// app owns it and runs `select` (sets `selected`). Pass the id given
    /// to [`VirtualList::sem`] so the nodes and the bindings name the same
    /// controls. Returns the list's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
        mut row_label: impl FnMut(usize) -> String,
    ) -> NodeBuilder<'b> {
        let selected = format!("{id}:selected");
        let stop = self.tab_stop();
        for index in self.visible_range() {
            let mut row = b.node(id.key(index.to_string()), Role::ListItem, row_label(index));
            row.parent(id).focusable(stop == Some(index));
            row.action("select").reads(selected.as_str());
        }
        let mut node = b.node(id, Role::List, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("count", count(self.item_count))
            .state("selected", selected_key(self.selected));
        node
    }

    /// Binds each row the list holds as `<id>[<index>]` on `layer`, to the
    /// part of it inside the viewport, which is where the hit test finds
    /// it. Rows wholly outside (the overscan) bind nothing.
    pub(crate) fn bind_rows(&self, c: &mut Compositor, layer: LayerId, id: &SemId) {
        for index in self.visible_range() {
            if let Some(r) = reachable(self.item_rect(index), self.viewport) {
                c.bind_to_layer(layer, &id.key(index.to_string()), [r.x, r.y, r.w, r.h]);
            }
        }
    }

    pub fn item_count(&self) -> usize {
        self.item_count
    }

    pub fn set_item_count(&mut self, count: usize) {
        self.item_count = count;
        self.scroll.set_content(count as f32 * self.item_height);
    }

    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    /// Keyboard focus on row `index`, or on none, driven by the owning
    /// view (the keyboard, a screen reader): the row draws the focus ring
    /// and the list scrolls it into view. A row past the end is none.
    pub fn set_focused(&mut self, index: Option<usize>) {
        self.focused = index.filter(|i| *i < self.item_count);
        if let Some(i) = self.focused {
            self.reveal(i);
        }
    }

    /// The row with the keyboard focus. It follows the scroll: once the
    /// rows scroll past it, the focus is on the nearest row in view
    /// ([`VirtualList::shown`]), so it never leaves the rows the list
    /// holds.
    pub fn focused(&self) -> Option<usize> {
        let at = self.focused?;
        let shown = self.shown();
        match shown.end.checked_sub(1) {
            Some(last) if !shown.is_empty() => Some(at.clamp(shown.start, last)),
            _ => Some(at),
        }
    }

    /// The row that takes the focus when Tab comes to the list, the
    /// list's one stop in the Tab sequence: the focused row, else the
    /// selected one while it is in view, else the first in view.
    pub fn tab_stop(&self) -> Option<usize> {
        let shown = self.shown();
        self.focused()
            .or(self.selected.filter(|s| shown.contains(s)))
            .or((!shown.is_empty()).then_some(shown.start))
    }

    /// The rows wholly inside the viewport; the ones that touch it when
    /// none fits wholly (a viewport shorter than a row).
    pub fn shown(&self) -> Range<usize> {
        if self.item_count == 0 || self.viewport.h <= 0.0 {
            return 0..0;
        }
        let top = self.scroll.offset() / self.item_height;
        let bottom = (self.scroll.offset() + self.viewport.h) / self.item_height;
        let whole = (top - EDGE).ceil() as usize..((bottom + EDGE).floor() as usize);
        let range = if whole.is_empty() {
            top.floor() as usize..(bottom.ceil() as usize)
        } else {
            whole
        };
        range.start.min(self.item_count)..range.end.min(self.item_count)
    }

    /// Scrolls the least that brings row `index` wholly into view.
    fn reveal(&mut self, index: usize) {
        if self.viewport.h <= 0.0 {
            return;
        }
        let top = index as f32 * self.item_height;
        let bottom = top + self.item_height;
        let now = self.scroll.offset();
        let to = if top < now {
            top
        } else if bottom > now + self.viewport.h {
            bottom - self.viewport.h
        } else {
            return;
        };
        self.scroll.scroll_to(to);
        if self.scroll.offset() != now {
            self.scrollbar.notify_scroll();
        }
    }

    /// Sync the scroll viewport with the rect the list is rendered into.
    pub fn set_viewport(&mut self, bounds: Rect) {
        self.viewport = bounds;
        self.scroll.set_viewport(bounds.h);
    }

    /// Range of item indices that intersect the viewport (plus overscan).
    pub fn visible_range(&self) -> Range<usize> {
        if self.item_count == 0 || self.viewport.h <= 0.0 {
            return 0..0;
        }
        let first = (self.scroll.offset() / self.item_height).floor() as usize;
        let visible = (self.viewport.h / self.item_height).ceil() as usize + 1;
        let start = first.saturating_sub(OVERSCAN);
        let end = (first + visible + OVERSCAN).min(self.item_count);
        start..end
    }

    /// Rect of item `index` in screen space (independent of visibility).
    pub fn item_rect(&self, index: usize) -> Rect {
        Rect::new(
            self.viewport.x,
            self.viewport.y + index as f32 * self.item_height - self.scroll.offset(),
            self.viewport.w,
            self.item_height,
        )
    }

    fn item_at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.viewport.contains(x, y) {
            return None;
        }
        let i = ((y - self.viewport.y + self.scroll.offset()) / self.item_height).floor();
        if i < 0.0 {
            return None;
        }
        let i = i as usize;
        (i < self.item_count).then_some(i)
    }

    /// Advance scrollbar fade. Returns `true` while animating.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.scrollbar.tick(dt)
    }

    pub fn is_animating(&self) -> bool {
        self.scrollbar.is_animating()
    }

    /// Handle wheel scrolling, scrollbar interaction, hover and click
    /// selection. `bounds` must match the rect passed to `render_with`;
    /// the scrollbar's hit band comes from `theme`. The keyboard focus
    /// follows the scroll ([`VirtualList::focused`]).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let result = self.event(event, bounds, theme);
        self.focused = self.focused();
        result
    }

    fn event(&mut self, event: &WidgetEvent, bounds: Rect, theme: &Theme) -> EventResult {
        self.set_viewport(bounds);

        // Scrollbar first: it sits on top of rows.
        let sb = self
            .scrollbar
            .handle_event(event, bounds, &mut self.scroll, theme);
        if sb.handled {
            return sb;
        }

        match *event {
            WidgetEvent::Scroll { x, y, delta } => {
                if !bounds.contains(x, y) {
                    return EventResult::IGNORED;
                }
                let old = self.scroll.offset();
                self.scroll.scroll_by(delta);
                self.scrollbar.notify_scroll();
                if self.scroll.offset() != old {
                    EventResult::changed()
                } else {
                    EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    }
                }
            }
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.item_at(x, y);
                if hit != self.hovered {
                    self.hovered = hit;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = self.item_at(x, y) {
                    self.selected = Some(i);
                    EventResult::clicked()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { .. } => EventResult::IGNORED,
        }
    }

    /// Render visible rows. `row_fn(compositor, index, rect, hovered,
    /// selected)` draws each row's content; the list itself draws hover
    /// and selection backgrounds, the focus ring on the focused row
    /// (inside it, so it never reaches the next), plus the scrollbar.
    ///
    /// Rows at the edges overflow `bounds` by up to one row; pair the
    /// list with a layer clip rect (`Compositor::set_layer_clip_rect`)
    /// when the overflow would be visible.
    pub fn render_with(
        &mut self,
        compositor: &mut Compositor,
        bounds: Rect,
        theme: &Theme,
        row_fn: impl FnMut(&mut Compositor, usize, Rect, bool, bool),
    ) {
        self.render_with_to_layer(compositor, LayerId::DEFAULT, bounds, theme, row_fn);
    }

    /// [`render_with`](VirtualList::render_with) targeting a specific layer
    /// (pair with that layer's clip rect to hide overscan rows).
    pub fn render_with_to_layer(
        &mut self,
        compositor: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        theme: &Theme,
        mut row_fn: impl FnMut(&mut Compositor, usize, Rect, bool, bool),
    ) {
        self.set_viewport(bounds);
        if let Some(id) = &self.sem {
            compositor.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
            self.bind_rows(compositor, layer, id);
        }

        let focused = self.focused();
        for index in self.visible_range() {
            let rect = self.item_rect(index);
            let hovered = self.hovered == Some(index);
            let selected = self.selected == Some(index);

            // HOFF rows: rest .02 / hover .05 / selected .10 white glass.
            if selected {
                compositor.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: rect.x,
                        y: rect.y,
                        w: rect.w,
                        h: rect.h,
                        color: theme.glass.surface_active.0,
                    },
                );
            } else if hovered {
                compositor.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: rect.x,
                        y: rect.y,
                        w: rect.w,
                        h: rect.h,
                        color: theme.glass.surface_hover.0,
                    },
                );
            }
            row_fn(compositor, index, rect, hovered, selected);
            if focused == Some(index) {
                let radius = theme.shape.nav.min(rect.h / 2.0);
                compositor.push_to_layer(layer, focus_ring_inside(rect, radius, theme));
            }
        }

        // The scrollbar stays on the same layer so it is clipped (and
        // composited) together with the rows.
        let mut scratch = Vec::new();
        self.scrollbar
            .render_nodes(&mut scratch, bounds, &self.scroll, theme);
        for node in scratch {
            compositor.push_to_layer(layer, node);
        }
    }
}
