//! HOFF segmented tabs: a graphite strip (`glass.tabs`, the tabs radius)
//! whose equal-width segments share one base-2sm style for measurement
//! and drawing; the active segment is a raised, edge-lit glass block.
//! Keyboard focus ([`Tabs::set_focused`]) rings the whole strip with the
//! accent ring; focus on one tab ([`Tabs::set_focused_tab`], what the arrow
//! keys move) rings that segment instead.
//!
//! With a [`SemId`] ([`Tabs::sem`]) the render binds the strip and each
//! segment to the rects it draws and hit-tests them with, and
//! [`Tabs::project`] declares the strip in a semantic projection; the app
//! still owns `active`.

use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::{ControlSize, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::recipe::{focus_ring, focus_ring_inside, glass_pill, shadow_node};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Segmented tab strip. Tabs share the width equally (CSS `flex: 1`);
/// give the widget [`Tabs::height`] (one `Md` control) for the canonical
/// `Xs` segments inside the tabs padding.
#[derive(Clone, Debug)]
pub struct Tabs {
    pub labels: Vec<String>,
    pub active: usize,
    hovered: Option<usize>,
    focused: bool,
    focused_tab: Option<usize>,
    sem: Option<SemId>,
}

impl Tabs {
    pub fn new(labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            active: 0,
            hovered: None,
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
        self.hovered
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

    /// Canonical strip height: the `Md` control (segments are `Xs` inside
    /// the tabs padding).
    pub fn height(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    /// Hit rects for each tab within `bounds`: equal-width segments
    /// inside the tabs container padding.
    pub fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let pad = theme.control.tabs_pad;
        let n = self.labels.len().max(1) as f32;
        let w = (bounds.w - pad * 2.0) / n;
        let h = bounds.h - pad * 2.0;
        (0..self.labels.len())
            .map(|i| Rect::new(bounds.x + pad + i as f32 * w, bounds.y + pad, w, h))
            .collect()
    }

    fn tab_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        self.item_rects(bounds, theme)
            .iter()
            .position(|r| r.contains(x, y))
    }

    /// Segments are laid out from the theme's tabs padding, so the event
    /// path takes the same `theme` the render path draws with.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.tab_at(x, y, bounds, theme);
                if hit != self.hovered {
                    self.hovered = hit;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = self.tab_at(x, y, bounds, theme) {
                    if i != self.active {
                        self.active = i;
                        EventResult::clicked()
                    } else {
                        EventResult {
                            handled: true,
                            ..EventResult::IGNORED
                        }
                    }
                } else {
                    EventResult::IGNORED
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let rects = self.item_rects(bounds, theme);
        if let Some(id) = &self.sem {
            compositor.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
            for (i, r) in rects.iter().enumerate() {
                compositor.bind(&id.key(i.to_string()), [r.x, r.y, r.w, r.h]);
            }
        }

        let radius = theme.shape.tabs.min(bounds.h / 2.0);
        if self.focused {
            compositor.push(focus_ring(bounds, radius, theme));
        }

        compositor.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: bounds.y,
            w: bounds.w,
            h: bounds.h,
            color: glass.tabs.0,
            corner_radius: radius,
            border_width: 0.0,
            border_color: [0.0; 4],
        });

        // Labels: base-2sm (Tabs.module.sass), one style for measure+render.
        let style = theme.typography.base_2sm();

        // Active block: raised shadow + edge-light + surface-hover fill at
        // the block radius.
        if let Some(rect) = rects.get(self.active) {
            let block_radius = theme.shape.block.min(rect.h / 2.0);
            if let Some(shadow) = shadow_node(*rect, block_radius, &theme.shadows.raised) {
                compositor.push(shadow);
            }
            for node in glass_pill(
                *rect,
                block_radius,
                glass.edge.0,
                theme.control.edge_width_strong,
                glass.surface_hover.0,
                theme,
            ) {
                compositor.push(node);
            }
        }
        for (i, (label, rect)) in self.labels.iter().zip(&rects).enumerate() {
            let is_active = i == self.active;
            let is_hovered = self.hovered == Some(i);

            // base-2sm $text-secondary -> $text-primary on hover/active.
            let color = if is_active || is_hovered {
                theme.colors.text
            } else {
                theme.colors.text_mid
            };
            let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: rect.x + (rect.w - text_w) / 2.0,
                y: rect.y + TextMeasurer::vertical_center(&style, rect.h),
                color: color.0,
            });
        }
        // The focused tab's ring last, over the block and the labels.
        if let Some(rect) = self.focused_tab.and_then(|i| rects.get(i)) {
            let block_radius = theme.shape.block.min(rect.h / 2.0);
            compositor.push(focus_ring_inside(*rect, block_radius, theme));
        }
    }
}
