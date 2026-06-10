use std::ops::Range;

use crate::compositor::{Compositor, LayerId, SceneNode};
use crate::scroll::ScrollState;
use crate::theme::Theme;

use super::scrollbar::Scrollbar;
use super::{EventResult, Rect, WidgetEvent, with_alpha};

/// Rows tessellated beyond the viewport on each side, so partially
/// visible rows and small scroll deltas don't pop.
const OVERSCAN: usize = 2;

/// Generic virtualized list: only visible rows are rendered, whatever the
/// item count. Owns scroll state, a fading [`Scrollbar`], hover and
/// selection — the row *content* is the caller's via a render closure.
///
/// ```rust
/// # use plev::ui::widgets::{VirtualList, Rect};
/// # use plev::compositor::Compositor;
/// # use plev::theme::Theme;
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
    viewport: Rect,
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
            viewport: Rect::default(),
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
    /// selection. `bounds` must match the rect passed to `render_with`.
    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.set_viewport(bounds);

        // Scrollbar first: it sits on top of rows.
        let sb = self.scrollbar.handle_event(event, bounds, &mut self.scroll);
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
    /// and selection backgrounds plus the scrollbar.
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

        for index in self.visible_range() {
            let rect = self.item_rect(index);
            let hovered = self.hovered == Some(index);
            let selected = self.selected == Some(index);

            if selected {
                compositor.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: rect.x,
                        y: rect.y,
                        w: rect.w,
                        h: rect.h,
                        color: with_alpha(theme.colors.accent, 0.14),
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
                        color: with_alpha(theme.colors.bg_hover, 1.0),
                    },
                );
            }
            row_fn(compositor, index, rect, hovered, selected);
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
