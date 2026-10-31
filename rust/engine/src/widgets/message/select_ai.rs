//! The model menu of the hoff core kit: its `components/SelectAI` (the
//! models on a floating panel, each a title over a description) and the
//! listbox its prompt input opens (`PromptInput/SelectAi`), the same rows
//! with the chosen one filled.
//!
//! Sized by its tokens: the panel's width, and as tall as its rows. A
//! title or a description wider than the row is ellipsized; the
//! projection reads both whole.
//!
//! Behavior apart from the look: a click on a row (or a reader's and an
//! agent's request) chooses it ([`SelectAi::choose`]); the pointer and
//! the keyboard light the row they are on. Nothing visual lives here: the
//! panel, the rows and the runs are `theme.message.prompt` (`panel`,
//! `choice`, `menu_shadow`).
//!
//! Two ways to be declared: on its own, a menu of its models
//! ([`SelectAi::project`], each `<id>[<i>]` a menu item with `activate`);
//! opened by a select, its rows as that select's options
//! ([`SelectAi::project_options`]).

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::message::{fit, width};
use super::tokens::{ChoiceTokens, MessageText, PanelTokens};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::styling::theme::{ShadowSpec, Theme};
use crate::widgets::recipe::{
    focus_ring_inside, rounded_rect, rounded_rect_stroke, shadow_drops, shadow_insets,
};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// One model: its name and what it is good for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelChoice {
    pub title: String,
    pub description: String,
}

impl ModelChoice {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectAi {
    choices: Vec<ModelChoice>,
    selected: Option<usize>,
    hovered: Option<usize>,
    pressed: Option<usize>,
    focused: Option<usize>,
    /// Anchored to a trigger (the prompt's listbox: the stacked drops) or
    /// standing alone (the kit's `SelectAI`: `shadow-2xl`).
    anchored: bool,
    sem: Option<SemId>,
}

impl SelectAi {
    pub fn new(choices: Vec<ModelChoice>) -> Self {
        Self {
            choices,
            selected: None,
            hovered: None,
            pressed: None,
            focused: None,
            anchored: false,
            sem: None,
        }
    }

    /// The kit's four models (`components/SelectAI/options.tsx`).
    pub fn kit() -> Self {
        Self::new(vec![
            ModelChoice::new("Hoff Research App 2.5", "Great for 3D modeling"),
            ModelChoice::new("Sora 4", "Great for video generation"),
            ModelChoice::new("Kling AI", "Great for video generation"),
            ModelChoice::new("Spline AI", "Great for pre-made scenes"),
        ])
    }

    /// The chosen model, filled.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = (index < self.choices.len()).then_some(index);
        self
    }

    /// Opened by a trigger: the listbox's drops.
    pub fn anchored(mut self, on: bool) -> Self {
        self.anchored = on;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn choices(&self) -> &[ModelChoice] {
        &self.choices
    }

    pub fn chosen(&self) -> Option<usize> {
        self.selected
    }

    pub fn chosen_title(&self) -> Option<&str> {
        self.selected
            .and_then(|i| self.choices.get(i))
            .map(|c| c.title.as_str())
    }

    /// Choose the model at `index`. `false` for one it does not have.
    pub fn choose(&mut self, index: usize) -> bool {
        if index >= self.choices.len() {
            return false;
        }
        self.selected = Some(index);
        true
    }

    /// Choose the model titled `title`. `false` for none.
    pub fn choose_title(&mut self, title: &str) -> bool {
        match self.choices.iter().position(|c| c.title == title) {
            Some(i) => self.choose(i),
            None => false,
        }
    }

    /// The keyboard's row, or none.
    pub fn set_focused(&mut self, index: Option<usize>) {
        self.focused = index.filter(|i| *i < self.choices.len());
    }

    pub fn focused(&self) -> Option<usize> {
        self.focused
    }

    /// The pointer left (the panel closed under it).
    pub fn reset_pointer(&mut self) {
        self.hovered = None;
        self.pressed = None;
    }

    fn row_h(t: &ChoiceTokens) -> f32 {
        t.pad_y * 2.0 + t.title.line_height + t.description_gap + t.description.line_height
    }

    /// The panel's size: its width, and as tall as its rows.
    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let p = &theme.message.prompt;
        let inset = p.panel.border_width + p.panel.pad;
        let rows = Self::row_h(&p.choice) * self.choices.len() as f32;
        (p.panel.w, inset * 2.0 + rows)
    }

    /// Each row's rect in `bounds`.
    pub fn rows(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let p = &theme.message.prompt;
        let inner = bounds.inset(p.panel.border_width + p.panel.pad);
        let h = Self::row_h(&p.choice);
        (0..self.choices.len())
            .map(|i| Rect::new(inner.x, inner.y + i as f32 * h, inner.w, h))
            .collect()
    }

    fn row_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        self.rows(bounds, theme)
            .iter()
            .position(|r| r.contains(x, y))
    }

    /// The pointer lights the row it is over; a press and a release on
    /// the same row choose it (`clicked`).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let at = self.row_at(x, y, bounds, theme);
                if at == self.hovered {
                    return EventResult::IGNORED;
                }
                self.hovered = at;
                EventResult::changed()
            }
            WidgetEvent::MouseDown { x, y } => {
                self.pressed = self.row_at(x, y, bounds, theme);
                if self.pressed.is_some() || bounds.contains(x, y) {
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                let Some(down) = self.pressed.take() else {
                    return EventResult::IGNORED;
                };
                if self.row_at(x, y, bounds, theme) == Some(down) {
                    self.choose(down);
                    EventResult::clicked()
                } else {
                    EventResult::changed()
                }
            }
            WidgetEvent::Scroll { .. } => EventResult::IGNORED,
        }
    }

    pub fn render(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let p = &theme.message.prompt;
        let shadow = if self.anchored {
            &p.panel.shadow
        } else {
            &p.menu_shadow
        };
        paint_panel(c, layer, bounds, &p.panel, shadow);
        let t = &p.choice;
        let rows = self.rows(bounds, theme);
        for (i, (choice, row)) in self.choices.iter().zip(&rows).enumerate() {
            let fill = if self.selected == Some(i) {
                Some(t.selected)
            } else if self.hovered == Some(i) || self.focused == Some(i) {
                Some(t.hover)
            } else {
                None
            };
            if let Some(fill) = fill {
                c.push_to_layer(
                    layer,
                    rounded_rect(row.x, row.y, row.w, row.h, t.radius, fill.0),
                );
            }
            if self.focused == Some(i) {
                c.push_to_layer(layer, focus_ring_inside(*row, t.radius, theme));
            }
            if let Some(id) = self.option_id(i) {
                c.bind_to_layer(layer, &id, [row.x, row.y, row.w, row.h]);
            }
            let x = row.x + t.pad_x;
            let room = (row.w - t.pad_x * 2.0).max(0.0);
            let y = row.y + t.pad_y;
            line(c, layer, &choice.title, &t.title, x, y, room, theme);
            let y = y + t.title.line_height + t.description_gap;
            line(
                c,
                layer,
                &choice.description,
                &t.description,
                x,
                y,
                room,
                theme,
            );
        }
        // opened by a trigger, the trigger is what its id names
        if let Some(id) = self.sem.as_ref().filter(|_| !self.anchored) {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
    }

    /// The id of row `index`: `<id>[<index>]`.
    pub fn option_id(&self, index: usize) -> Option<SemId> {
        self.sem.as_ref().map(|id| id.key(index.to_string()))
    }

    /// Declares the panel on its own as `id`: a `Role::Menu` named
    /// `label` whose `selected` is the chosen title, and each model a
    /// `Role::MenuItem` `<id>[<i>]` named by its title, described by its
    /// description, `selected` when chosen, with `activate` (choose it),
    /// which reads the menu's `selected`.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let reads = format!("{id}:selected");
        for (i, choice) in self.choices.iter().enumerate() {
            let mut row = b.node(id.key(i.to_string()), Role::MenuItem, choice.title.as_str());
            row.parent(id)
                .description(choice.description.as_str())
                .state("selected", self.selected == Some(i));
            row.action("activate").reads(reads.as_str());
        }
        let mut node = b.node(id, Role::Menu, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("selected", self.chosen_title().unwrap_or_default());
        node
    }

    /// Declares the rows as the options of the select `select`: each
    /// `<select>[<i>]` a `Role::Option` named by its title, described by
    /// its description, with `select`, which reads the select's
    /// `selected`.
    pub fn project_options(&self, b: &mut SnapshotBuilder, select: &SemId, enabled: bool) {
        let reads = format!("{select}:selected");
        for (i, choice) in self.choices.iter().enumerate() {
            let mut row = b.node(
                select.key(i.to_string()),
                Role::Option,
                choice.title.as_str(),
            );
            row.parent(select)
                .description(choice.description.as_str())
                .enabled(enabled);
            row.action("select").reads(reads.as_str());
        }
    }
}

/// A floating panel of the kit: its drops, its fill and border, its
/// insets.
pub(crate) fn paint_panel(
    c: &mut Compositor,
    layer: LayerId,
    r: Rect,
    t: &PanelTokens,
    shadow: &[ShadowSpec],
) {
    shadow_drops(c, layer, r, t.radius, shadow);
    c.push_to_layer(layer, rounded_rect(r.x, r.y, r.w, r.h, t.radius, t.fill.0));
    c.push_to_layer(
        layer,
        rounded_rect_stroke(r.x, r.y, r.w, r.h, t.radius, t.border.0, t.border_width),
    );
    shadow_insets(c, layer, r, t.radius, shadow);
}

/// One run on one line of `room` at `(x, y)`, ellipsized.
#[allow(clippy::too_many_arguments)]
pub(crate) fn line(
    c: &mut Compositor,
    layer: LayerId,
    text: &str,
    run: &MessageText,
    x: f32,
    y: f32,
    room: f32,
    theme: &Theme,
) -> f32 {
    let style = run.style(&theme.typography);
    let shown = fit(text, &style, room);
    if shown.is_empty() {
        return 0.0;
    }
    let w = width(&shown, &style);
    c.push_to_layer(
        layer,
        SceneNode::Text {
            key: TextNodeKey::from_style(&shown, &style, None),
            x,
            y,
            color: run.color.0,
        },
    );
    w
}
