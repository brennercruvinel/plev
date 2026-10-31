//! Input group: a text field with addons at the ends of its box, the
//! kit's Search (`components/Search/index.tsx`): an icon button at its
//! start (the search), a shortcut hint at its end (`⌘ K`), or a text
//! addon (a prefix such as `https://`, a suffix such as `@hoff.dev`).
//! Every value is `theme.form`'s: the box is the `search` recipe (or any
//! other [`Look`]) and the addons are `theme.form.group`.
//!
//! The typed text keeps clear of the addons: each end's inset is the
//! recipe's padding or what its addon takes, whichever is wider, and the
//! addons are measured, never estimated. A hovered group darkens its
//! icon, as the kit's `group-hover` does. The editing is the inner
//! [`TextField`]'s (`group.input`).
//!
//! A select can take the end of the box (the kit's Invite,
//! `components/Invite/index.tsx`: who to invite, and what they can do),
//! the select's inline look, as wide as its options measure (never less
//! than `theme.form.group.select_w`); its open list floats on the
//! owner's overlay layer ([`InputGroup::render_dropdown`]) and has the
//! pointer while open.
//!
//! Projected ([`InputGroup::project`]): the text field, described by its
//! hint; beside it the icon button (`<id>.action`, `activate`) and the
//! select (`<id>.choice`, as a select projects).

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::form::select::Select;
use crate::widgets::form::text_field::TextField;
use crate::widgets::form::tokens::{Look, SelectLook, Slot};
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring_inside, rounded_rect, shadow_drops, shadow_insets};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// What an end of the box holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Addon {
    /// A Lucide icon on a button the owner runs (the search).
    Icon(&'static str),
    /// A shortcut hint, `⌘ K`.
    Hint(String),
    /// A text addon, a prefix or a suffix.
    Text(String),
}

/// Where the parts of a group go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupLayout {
    pub input: Rect,
    /// The start and end addons' boxes.
    pub start: Option<Rect>,
    pub end: Option<Rect>,
    /// The select at the end.
    pub choice: Option<Rect>,
}

/// A text field with addons.
#[derive(Clone, Debug)]
pub struct InputGroup {
    pub input: TextField,
    start: Option<Addon>,
    end: Option<Addon>,
    /// The keyboard focus on the icon button (a reader's, or Tab).
    action_focus: bool,
    /// A select at the end of the box.
    pub choice: Option<Select>,
    sem: Option<SemId>,
}

impl InputGroup {
    pub fn new(placeholder: impl AsRef<str>) -> Self {
        Self {
            input: TextField::new(placeholder).look(Look::Search),
            start: None,
            end: None,
            action_focus: false,
            choice: None,
            sem: None,
        }
    }

    /// A select at the end of the box, in its inline look.
    pub fn end_select(mut self, select: Select) -> Self {
        let select = select.look(SelectLook::Inline);
        self.choice = Some(match self.choice_id() {
            Some(id) => select.sem(id),
            None => select,
        });
        self
    }

    pub fn start(mut self, addon: Addon) -> Self {
        self.start = Some(addon);
        self
    }

    pub fn end(mut self, addon: Addon) -> Self {
        self.end = Some(addon);
        self
    }

    /// Draw `look`'s box instead of the search box.
    pub fn look(mut self, look: Look) -> Self {
        self.input = self.input.look(look);
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.input.set_text(text);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.input.disabled = disabled;
        if let Some(choice) = &mut self.choice {
            choice.disabled = disabled;
        }
        self
    }

    /// The id the box binds to and projects as; its icon button is
    /// `<id>.action`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.input = self.input.sem(id.clone());
        self.sem = Some(id);
        if let (Some(choice), Some(id)) = (self.choice.take(), self.choice_id()) {
            self.choice = Some(choice.sem(id));
        }
        self
    }

    /// The end select's id, `<id>.choice`.
    pub fn choice_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("choice").ok()
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn action_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("action").ok()
    }

    /// The icon button the group has, if any (its icon's name).
    pub fn action(&self) -> Option<&'static str> {
        [&self.start, &self.end].into_iter().find_map(|a| match a {
            Some(Addon::Icon(name)) => Some(*name),
            _ => None,
        })
    }

    pub fn set_action_focus(&mut self, on: bool) {
        self.action_focus = on && self.action().is_some();
    }

    pub fn has_action_focus(&self) -> bool {
        self.action_focus
    }

    /// An addon's size: an icon button the box's height less its edges,
    /// a hint and a text as they measure.
    fn addon_size(&self, addon: &Addon, h: f32, theme: &Theme) -> (f32, f32) {
        let g = &theme.form.group;
        match addon {
            Addon::Icon(_) => (g.icon_slot, (h - g.icon_edge * 2.0).max(0.0)),
            Addon::Hint(text) => {
                let (w, th) = TextMeasurer::measure_styled(text, &g.hint_text, None);
                (w.ceil() + g.hint_pad[0] * 2.0, th + g.hint_pad[1] * 2.0)
            }
            Addon::Text(text) => {
                let style = self.input.style(theme);
                let (w, th) = TextMeasurer::measure_styled(text, &style, None);
                (w.ceil(), th)
            }
        }
    }

    /// An addon's distance from its edge of the box.
    fn edge(addon: &Addon, theme: &Theme) -> f32 {
        let g = &theme.form.group;
        match addon {
            Addon::Icon(_) => g.icon_edge,
            Addon::Hint(_) => g.hint_edge,
            Addon::Text(_) => g.text_edge,
        }
    }

    /// Where the box and its addons go in `bounds` (its height is the
    /// box's), and the text's insets set to clear them.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> GroupLayout {
        let h = self.input.box_height(theme);
        let input = Rect::new(bounds.x, bounds.y, bounds.w, h);
        let place = |addon: &Option<Addon>, at_start: bool| {
            let addon = addon.as_ref()?;
            let (w, ah) = self.addon_size(addon, h, theme);
            let edge = Self::edge(addon, theme);
            let x = if at_start {
                input.x + edge
            } else {
                input.right() - edge - w
            };
            Some(Rect::new(x, input.y + (h - ah) / 2.0, w, ah))
        };
        let choice = self.choice.as_ref().map(|c| {
            let g = &theme.form.group;
            let h = c.height_in(theme).min(input.h - g.icon_edge * 2.0);
            let w = c.width_in(theme).max(g.select_w);
            Rect::new(
                input.right() - g.icon_edge - w,
                input.y + (input.h - h) / 2.0,
                w,
                h,
            )
        });
        GroupLayout {
            input,
            start: place(&self.start, true),
            end: place(&self.end, false),
            choice,
        }
    }

    /// The text's inset on an end: the recipe's padding or what the
    /// addon takes there (its edge, its width and, for a text, the same
    /// clearance again before the typed text), whichever is wider.
    fn inset(&self, addon: &Option<Addon>, rect: Option<Rect>, theme: &Theme) -> Slot {
        let pad = theme.form.recipe(self.look_of()).pad_x;
        match (addon, rect) {
            (Some(a), Some(r)) => {
                let gap = match a {
                    Addon::Text(_) => theme.form.group.text_edge,
                    _ => 0.0,
                };
                Slot::Width(pad.max(Self::edge(a, theme) + r.w + gap))
            }
            _ => Slot::Pad,
        }
    }

    fn look_of(&self) -> Look {
        self.input.look_of()
    }

    /// The text's insets for `l`, as the addons measure in `theme`.
    fn slots(&self, l: &GroupLayout, theme: &Theme) -> [Slot; 2] {
        let end = match l.choice {
            // Invite :22 `!pr-28`: the select, its edge and a gap.
            Some(r) => {
                let pad = theme.form.recipe(self.look_of()).pad_x;
                let clear = l.input.right() - r.x + theme.form.select.gap;
                Slot::Width(pad.max(clear))
            }
            None => self.inset(&self.end, l.end, theme),
        };
        [self.inset(&self.start, l.start, theme), end]
    }

    /// The end select's open list, when it is open.
    pub fn dropdown_rect(&self, bounds: Rect, theme: &Theme) -> Option<Rect> {
        let c = self.choice.as_ref().filter(|c| c.is_open())?;
        Some(c.dropdown_rect(self.layout(bounds, theme).choice?, theme))
    }

    /// Draws the end select's open list on `layer` (an overlay).
    pub fn render_dropdown(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        if let (Some(choice), Some(r)) = (&self.choice, self.layout(bounds, theme).choice) {
            choice.render_dropdown(c, layer, r, theme);
        }
    }

    /// Where the input method's candidate window goes: the caret.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        let l = self.layout(bounds, theme);
        self.input
            .ime_area_in(l.input, theme, self.slots(&l, theme))
    }

    /// The group's height: its box's.
    pub fn height(&self, theme: &Theme) -> f32 {
        self.input.box_height(theme)
    }

    /// Whether `(x, y)` is on the icon button.
    pub fn on_action(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> bool {
        let l = self.layout(bounds, theme);
        let icon_at = |a: &Option<Addon>, r: Option<Rect>| {
            matches!(a, Some(Addon::Icon(_))) && r.is_some_and(|r| r.contains(x, y))
        };
        icon_at(&self.start, l.start) || icon_at(&self.end, l.end)
    }

    /// The pointer: a press on the icon button is `clicked` (the owner
    /// runs it, the box keeps its focus); anything else is the box's.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        if let WidgetEvent::MouseDown { x, y } = *event
            && !self.input.disabled
            && self.on_action(x, y, bounds, theme)
        {
            return EventResult::clicked();
        }
        // The end select: open, it has every event; closed, a press on it.
        if let (Some(choice), Some(r)) = (&mut self.choice, l.choice) {
            let (x, y) = event.pos();
            let pressed = matches!(event, WidgetEvent::MouseDown { .. }) && r.contains(x, y);
            if choice.is_open() || pressed {
                return choice.handle_event(event, r, theme);
            }
            if matches!(event, WidgetEvent::MouseMove { .. }) {
                let hover = choice.handle_event(event, r, theme);
                let slots = self.slots(&l, theme);
                return hover.merge(self.input.handle_event_in(event, l.input, theme, slots));
            }
        }
        let slots = self.slots(&l, theme);
        self.input.handle_event_in(event, l.input, theme, slots)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    /// Draws the box, then its addons over it.
    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let l = self.layout(bounds, theme);
        let slots = self.slots(&l, theme);
        self.input.render_in(c, layer, l.input, theme, slots);
        if let (Some(choice), Some(r)) = (&self.choice, l.choice) {
            choice.render(c, r, theme);
        }
        let g = &theme.form.group;
        let alpha = if self.input.disabled {
            theme.form.recipe(self.look_of()).disabled_alpha
        } else {
            1.0
        };
        let fade = |color: Color| with_alpha(color, color.0[3] * alpha);
        for (addon, rect) in [(&self.start, l.start), (&self.end, l.end)] {
            let (Some(addon), Some(r)) = (addon, rect) else {
                continue;
            };
            match addon {
                Addon::Icon(name) => {
                    let hot = self.input.is_hovered() || self.input.is_focused();
                    let tint = if hot { g.icon_hover } else { g.icon_color };
                    let x = r.x + (r.w - g.icon) / 2.0;
                    let y = r.y + (r.h - g.icon) / 2.0;
                    if let Some(node) = icons::icon_at(name, g.icon, fade(tint), x, y) {
                        c.push_to_layer(layer, node);
                    }
                    if let Some(id) = self.action_id() {
                        c.bind_to_layer(layer, &id, [r.x, r.y, r.w, r.h]);
                    }
                    if self.action_focus {
                        let radius = theme.form.recipe(self.look_of()).radius;
                        let radius = (radius - g.icon_edge).max(0.0);
                        c.push_to_layer(layer, focus_ring_inside(r, radius, theme));
                    }
                }
                Addon::Hint(text) => {
                    shadow_drops(c, layer, r, g.hint_radius, &g.hint_shadow);
                    c.push_to_layer(
                        layer,
                        rounded_rect(r.x, r.y, r.w, r.h, g.hint_radius, fade(g.hint_fill)),
                    );
                    shadow_insets(c, layer, r, g.hint_radius, &g.hint_shadow);
                    c.push_to_layer(
                        layer,
                        SceneNode::Text {
                            key: TextNodeKey::from_style(text, &g.hint_text, None),
                            x: r.x + g.hint_pad[0],
                            y: r.y + g.hint_pad[1],
                            color: fade(g.hint_color),
                        },
                    );
                }
                Addon::Text(text) => {
                    let style = self.input.style(theme);
                    c.push_to_layer(
                        layer,
                        SceneNode::Text {
                            key: TextNodeKey::from_style(text, &style, None),
                            x: r.x,
                            y: r.y,
                            color: fade(g.text_color),
                        },
                    );
                }
            }
        }
    }

    /// Declares the group as `id` (the id given to [`InputGroup::sem`]):
    /// the text field named `label`, described by its hint; beside it,
    /// under `parent`, the icon button (`<id>.action`, `activate`) named by
    /// its icon. It only reads the widget.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) {
        let mut node = self.input.project(b, id, label, parent);
        let hint = [&self.start, &self.end].into_iter().find_map(|a| match a {
            Some(Addon::Hint(t)) => Some(t.clone()),
            _ => None,
        });
        if let Some(hint) = hint {
            node.description(hint);
        }
        if let (Some(choice), Some(cid)) = (&self.choice, self.choice_id()) {
            choice.project(b, &cid, "access", Some(parent.unwrap_or(id)));
        }
        if let (Some(action), Some(name)) = (self.action_id(), self.action()) {
            let mut node = b.node(&action, Role::Button, name);
            node.parent(parent.unwrap_or(id));
            node.enabled(!self.input.disabled);
            node.action("activate");
        }
    }
}
