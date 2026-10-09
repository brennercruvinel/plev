//! HOFF chip: a small status/tag pill — caption-sm label in a radius-full
//! glass capsule, tinted by [`Intent`]. Filled (`selected`) reads as the
//! active filter/tag; the outline variant is the quiet default. Static by
//! default; `interactive(true)` opts into the Button press contract
//! (click fires on release inside).
//!
//! Geometry from the theme: height is the caption-sm line box plus the
//! `xs` spacing above and below, the side padding is `md` spacing.
//!
//! `selectable(true)` makes it a filter chip: a click toggles `selected`,
//! like a checkbox. With a [`SemId`] ([`Chip::sem`]) the render binds the
//! bounds it draws and hit-tests with, and [`Chip::project`] declares the
//! chip in a semantic projection. Keyboard focus ([`Chip::set_focused`],
//! driven by the owning view) rings a chip the pointer reaches.

use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{Intent, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, intent_fill, with_alpha};

/// Status/tag pill. See the module docs for the interactivity contract.
#[derive(Clone, Debug)]
pub struct Chip {
    pub label: String,
    pub intent: Intent,
    /// Filled (active) vs outline (quiet) styling.
    pub selected: bool,
    /// Opt into pointer interaction (hover/press/click).
    pub interactive: bool,
    /// A filter chip: a click toggles `selected`. Implies the pointer
    /// interaction of `interactive`.
    pub selectable: bool,
    hovered: bool,
    pressed: bool,
    focused: bool,
    sem: Option<SemId>,
}

impl Chip {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            intent: Intent::Neutral,
            selected: false,
            interactive: false,
            selectable: false,
            hovered: false,
            pressed: false,
            focused: false,
            sem: None,
        }
    }

    /// The logical identity the render binds its bounds to
    /// (`Compositor::bind`).
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this chip as `id` in a projection, named by its label. A
    /// selectable chip is a `Role::Toggle` with its `selected` state and a
    /// `toggle` action that reads it; an interactive one is a
    /// `Role::Button` with `activate`; a static tag is a `Role::Label`
    /// with no action, since nothing runs when it is clicked. It only
    /// reads the widget; the app owns it and runs the action. Pass the id
    /// given to [`Chip::sem`] so the node and the binding name the same
    /// control. Returns the node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let role = if self.selectable {
            Role::Toggle
        } else if self.interactive {
            Role::Button
        } else {
            Role::Label
        };
        let mut node = b.node(id, role, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        if self.selectable {
            node.state("selected", self.selected);
            node.action("toggle").reads(format!("{id}:selected"));
        } else if self.interactive {
            node.action("activate");
        }
        node
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Keyboard focus, driven by the owning view (plev has no global
    /// focus chain): the accent ring around the capsule. A static chip
    /// (neither `interactive` nor `selectable`) refuses it.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && self.pressable();
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Whether the pointer reaches the chip at all.
    fn pressable(&self) -> bool {
        self.interactive || self.selectable
    }

    /// Label style: HOFF caption-sm, one style for measure+draw.
    fn style(theme: &Theme) -> TextStyle {
        theme.typography.caption_sm()
    }

    /// Chip height for a theme: the caption line box plus `xs` above and
    /// below (HOFF: 16 + 2 * 4 = 24).
    pub fn height(theme: &Theme) -> f32 {
        Self::style(theme).line_height + theme.spacing.xs * 2.0
    }

    /// Intrinsic size from real text measurement.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let (tw, _) = TextMeasurer::measure_styled(&self.label, &Self::style(theme), None);
        ((tw + theme.spacing.md * 2.0).ceil(), Self::height(theme))
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        if !self.pressable() {
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let inside = bounds.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if bounds.contains(x, y) {
                    self.pressed = true;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                if !self.pressed {
                    return EventResult::IGNORED;
                }
                self.pressed = false;
                if bounds.contains(x, y) {
                    if self.selectable {
                        self.selected = !self.selected;
                    }
                    EventResult::clicked()
                } else {
                    EventResult::changed()
                }
            }
            WidgetEvent::Scroll { .. } => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            compositor.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let style = Self::style(theme);
        let accent = intent_fill(theme, self.intent);
        let text = theme.colors.text;
        let glass = &theme.glass;
        if self.focused {
            compositor.push(focus_ring(bounds, bounds.h / 2.0, theme));
        }

        // Filled (selected): intent wash + intent label. Outline: glass
        // edge stroke + dim label. Hover brightens both (interactive only).
        let (bg, edge, fg) = if self.selected {
            let hot = self.hovered && self.pressable();
            (
                with_alpha(
                    theme.colors.accent,
                    if hot {
                        glass.wash_hover_alpha
                    } else {
                        glass.wash_alpha
                    },
                ),
                [0.0; 4],
                if self.intent == Intent::Neutral {
                    text.0
                } else {
                    accent
                },
            )
        } else {
            let bg = if self.hovered && self.pressable() {
                theme.glass.surface_hover.0
            } else {
                [0.0; 4]
            };
            let fg = if self.intent == Intent::Neutral {
                theme.colors.text_mid.0
            } else {
                accent
            };
            (bg, theme.glass.edge_soft.0, fg)
        };

        if bg[3] > 0.001 {
            compositor.push(rounded_rect(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                bounds.h / 2.0,
                bg,
            ));
        }
        if edge[3] > 0.001 {
            compositor.push(rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                bounds.h / 2.0,
                edge,
                theme.control.edge_width,
            ));
        }
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(&self.label, &style, None),
            x: bounds.x + theme.spacing.md,
            y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
            color: fg,
        });
    }
}
