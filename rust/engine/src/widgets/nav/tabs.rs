//! The tab strip: the kit's segmented control (`components/Tabs`), the
//! same one the toggle group is. Layout, painting and the hover are the
//! segmented control's ([`ToggleGroup`], `theme.selection.segmented`);
//! the strip keeps its own contract: a press on a tab switches to it at
//! once (the tabs of an app switch on the press), the app owns `active`,
//! and it projects as a tab list. Keyboard focus
//! ([`Tabs::set_focused`]) rings the whole strip; focus on one tab
//! ([`Tabs::set_focused_tab`], what the arrow keys move) rings that
//! segment.
//!
//! With a [`SemId`] ([`Tabs::sem`]) the render binds the strip and each
//! segment to the rects it draws and hit-tests them with, and
//! [`Tabs::project`] declares the strip in a semantic projection; the app
//! still owns `active`.

use crate::drawing::compositor::Compositor;
use crate::styling::theme::{ControlSize, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::recipe::focus_ring;
use crate::widgets::selection::ToggleGroup;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Segmented tab strip. Tabs share the width equally (CSS `flex: 1`);
/// give the widget [`Tabs::height`] (one `Md` control).
#[derive(Clone, Debug)]
pub struct Tabs {
    pub labels: Vec<String>,
    pub active: usize,
    /// The segmented control that lays the strip out, paints it and
    /// tracks the hover; its choice follows `active`.
    segments: ToggleGroup,
    focused: bool,
    focused_tab: Option<usize>,
    sem: Option<SemId>,
}

impl Tabs {
    pub fn new(labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        Self {
            segments: ToggleGroup::new(labels.clone()),
            labels,
            active: 0,
            focused: false,
            focused_tab: None,
            sem: None,
        }
    }

    /// The logical identity the render binds the strip to
    /// (`Compositor::bind`); each segment binds as `<id>[<index>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this strip as `id` in a projection: a `Role::TabList`
    /// named `label` whose `selected` state is the key of the active tab
    /// (its index; empty without tabs), and one `Role::Tab` child
    /// `<id>[<index>]` per label with a `select` action that reads the
    /// list's `selected`, so a switch decided on a tab another actor left
    /// goes stale. The hovered segment is never projected. It only reads
    /// the widget; the app owns it and runs `select` (sets `active`). Pass
    /// the id given to [`Tabs::sem`] so the nodes and the bindings name the
    /// same controls. Returns the list's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let selected = format!("{id}:selected");
        for (i, tab) in self.labels.iter().enumerate() {
            let mut node = b.node(id.key(i.to_string()), Role::Tab, tab.as_str());
            node.parent(id);
            node.action("select").reads(selected.as_str());
        }
        let active = if self.active < self.labels.len() {
            self.active.to_string()
        } else {
            String::new()
        };
        let mut node = b.node(id, Role::TabList, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("selected", active);
        node
    }

    pub fn hovered(&self) -> Option<usize> {
        self.segments.hovered()
    }

    /// Keyboard focus, driven by the owning view (plev has no global
    /// focus chain). The ring wraps the whole strip.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Keyboard focus on the tab at `index`, or on none, driven by the
    /// owning view: the ring wraps that segment, not the strip, so the
    /// focus can sit on a tab that is not the active one.
    pub fn set_focused_tab(&mut self, index: Option<usize>) {
        self.focused_tab = index.filter(|i| *i < self.labels.len());
    }

    pub fn focused_tab(&self) -> Option<usize> {
        self.focused_tab
    }

    /// Canonical strip height: the `Md` control.
    pub fn height(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    /// The segmented control as the strip stands now: its labels, the
    /// active tab under the thumb (the app sets `active` freely, so the
    /// thumb sits on it with no slide), the focused tab, the hover kept.
    fn segmented(&self) -> ToggleGroup {
        let mut g = if self.segments.items == self.labels {
            self.segments.clone()
        } else {
            ToggleGroup::new(self.labels.clone())
        };
        if g.selected != self.active {
            g = g.selected(self.active);
        }
        g.set_focused(self.focused_tab);
        g
    }

    /// Hit rects for each tab within `bounds`: the segmented control's
    /// equal segments inside its well.
    pub fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        self.segmented().segment_rects(bounds, theme)
    }

    fn tab_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        self.item_rects(bounds, theme)
            .iter()
            .position(|r| r.contains(x, y))
    }

    /// A press on a tab switches to it; the pointer's moves are the
    /// segmented control's hover. Segments are laid out from the theme,
    /// so the event path takes the same `theme` the render path draws
    /// with.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.segments = self.segmented();
        match *event {
            WidgetEvent::MouseMove { .. } => self.segments.handle_event(event, bounds, theme),
            WidgetEvent::MouseDown { x, y } => match self.tab_at(x, y, bounds, theme) {
                Some(i) if i != self.active => {
                    self.active = i;
                    EventResult::clicked()
                }
                Some(_) => EventResult {
                    handled: true,
                    ..EventResult::IGNORED
                },
                None => EventResult::IGNORED,
            },
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let segments = self.segmented();
        if let Some(id) = &self.sem {
            compositor.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
            for (i, r) in segments.segment_rects(bounds, theme).iter().enumerate() {
                compositor.bind(&id.key(i.to_string()), [r.x, r.y, r.w, r.h]);
            }
        }
        if self.focused {
            let radius = theme.selection.segmented.track_radius.min(bounds.h / 2.0);
            compositor.push(focus_ring(bounds, radius, theme));
        }
        segments.render(compositor, bounds, theme);
    }
}
