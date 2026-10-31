//! The kit's chip (`theme.selection.chip`): the card's state tag and the
//! notifications' filter pill. Filled (`selected`) reads as the active
//! filter or tag, the bordered tag is the quiet default; an [`Intent`]
//! tints the tag, its label whatever reads on it. Static by
//! default; `interactive(true)` opts into the Button press contract
//! (click fires on release inside).
//!
//! Geometry from the tokens: the line box, the kit's `px-3 py-1` and the
//! border the box holds; painted by the buttons' `paint_surface`.
//!
//! `selectable(true)` makes it a filter chip: a click toggles `selected`,
//! like a checkbox. With a [`SemId`] ([`Chip::sem`]) the render binds the
//! bounds it draws and hit-tests with, and [`Chip::project`] declares the
//! chip in a semantic projection. Keyboard focus ([`Chip::set_focused`],
//! driven by the owning view) rings a chip the pointer reaches.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::{ButtonSurface, Intent, Theme};
use crate::widgets::buttons::paint_surface;
use crate::widgets::selection::tokens::legible;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, intent_fill};

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

    /// Label style, one for measure and draw: `theme.selection.chip`.
    fn style(theme: &Theme) -> TextStyle {
        let t = &theme.selection.chip;
        theme
            .typography
            .exact(t.font_size, t.line_height, t.font_weight)
    }

    /// Chip height for a theme: the line box, the `py` above and below,
    /// and the border the box holds (the kit: 16 + 2 * 4 + 2 = 26).
    pub fn height(theme: &Theme) -> f32 {
        let t = &theme.selection.chip;
        Self::style(theme).line_height + (t.pad_y + t.edge) * 2.0
    }

    /// Intrinsic size from real text measurement.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.selection.chip;
        let (tw, _) = TextMeasurer::measure_styled(&self.label, &Self::style(theme), None);
        ((tw + (t.pad_x + t.edge) * 2.0).ceil(), Self::height(theme))
    }

    /// The surface this chip shows: the theme's quiet or chosen tag, and
    /// under an intent the tag in the intent's color, its label whatever
    /// reads on it.
    fn surface(&self, theme: &Theme) -> ButtonSurface {
        let t = &theme.selection.chip;
        let hot = self.hovered && self.pressable();
        let base = match (self.selected, hot) {
            (true, false) => &t.on,
            (true, true) => &t.on_hover,
            (false, false) => &t.rest,
            (false, true) => &t.rest_hover,
        };
        if self.intent == Intent::Neutral {
            return base.clone();
        }
        let c = &theme.colors;
        let hue = match self.intent {
            Intent::Informational => t.info,
            intent => Color(intent_fill(theme, intent)),
        };
        if self.selected {
            ButtonSurface {
                fill: [hue; 2],
                text: legible(c.bg, c.text, hue, c.bg),
                ..base.clone()
            }
        } else {
            let mut shadow = base.shadow.clone();
            for s in &mut shadow {
                s.color = hue;
            }
            ButtonSurface {
                shadow,
                text: legible(hue, c.text, base.fill[0], c.bg),
                ..base.clone()
            }
        }
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
        let t = &theme.selection.chip;
        let radius = t.radius.min(bounds.h / 2.0);
        if self.focused {
            compositor.push(focus_ring(bounds, radius, theme));
        }
        let surface = self.surface(theme);
        paint_surface(
            compositor,
            LayerId::DEFAULT,
            bounds,
            radius,
            &surface,
            1.0,
            false,
            theme,
        );
        let style = Self::style(theme);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(&self.label, &style, None),
            x: bounds.x + t.pad_x + t.edge,
            y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
            color: surface.text.0,
        });
    }
}
