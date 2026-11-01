//! Combobox: a select whose control is a text box that filters its
//! options as one types, the kit's Select (`components/Select/
//! index.tsx`) with a filter. Closed it shows the chosen option, as a
//! select does; focused, its text is selected whole, and the options
//! list (the select's, `super::select::list`) opens under it on a press,
//! on Up or Down, or as soon as the text is not the chosen option's:
//! every option while it is, else those whose label holds the text, any
//! case; none says so in a row of its own.
//!
//! The filter is a [`TextField`] (`combo.input`, the select's control as
//! a text box, [`Look::Combo`]): typing, the editing keys, the clipboard
//! and the input method reach it as any field. Up and Down move the
//! highlight over the matches ([`Combobox::nav`]), Enter picks it
//! ([`Combobox::enter`]), a press on a row picks that one; picking puts
//! the option's label in the box and blurs it, which closes the list.
//!
//! Projected ([`Combobox::project`]): a `Role::Select` as a select is,
//! named `label`, its `selected` option, `expanded` while open, the
//! options as its accepted choices, with `select`, `open` and `close`,
//! and the filter's text as `query`; while open, the matching options as
//! `<id>[<index>]` with a `select` of their own, as an open select's.

use accord::{Constraints, ParamSpec, Role, SemId, SnapshotBuilder, ValueKind};

use crate::control::input::scroll::ScrollState;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::form::select::list::{self, Row, Window};
use crate::widgets::form::text_field::TextField;
use crate::widgets::form::tokens::{Look, Slot};
use crate::widgets::hit::hit_rect;
use crate::widgets::icons;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// A select with a filter.
#[derive(Clone, Debug)]
pub struct Combobox {
    pub input: TextField,
    options: Vec<String>,
    selected: Option<usize>,
    /// The option the arrow keys or the pointer moved to.
    highlighted: Option<usize>,
    scroll: ScrollState,
    /// The list was opened (a press, a key); it is open too while the
    /// text is not the chosen option's.
    open: bool,
    /// A reader or Tab put the focus on the highlighted option.
    option_focus: bool,
    /// The highlight moved by a key and the list has not been laid out
    /// since: it scrolls the least that shows it, the next time.
    reveal: bool,
    /// What the list says when nothing matches.
    empty: String,
    sem: Option<SemId>,
}

impl Combobox {
    pub fn new(options: impl IntoIterator<Item = impl Into<String>>, placeholder: &str) -> Self {
        Self {
            input: TextField::new(placeholder).look(Look::Combo),
            options: options.into_iter().map(Into::into).collect(),
            selected: None,
            highlighted: None,
            scroll: ScrollState::new(),
            reveal: false,
            open: false,
            option_focus: false,
            empty: "No results.".into(),
            sem: None,
        }
    }

    /// The option chosen at the start.
    pub fn selected(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected = Some(index);
            self.input.set_text(&self.options[index]);
        }
        self
    }

    /// What the list says when nothing matches.
    pub fn empty(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.input.disabled = disabled;
        self
    }

    /// The id the control binds to and projects as; the open rows bind as
    /// `<id>[<index>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.input = self.input.sem(id.clone());
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn options(&self) -> &[String] {
        &self.options
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.selected.map(|i| self.options[i].as_str())
    }

    /// The list is open: the box is focused, and the list was opened or
    /// the text typed.
    pub fn is_open(&self) -> bool {
        let typed = self.input.text() != self.selected_label().unwrap_or_default();
        self.input.is_focused() && !self.input.disabled && (self.open || typed)
    }

    /// Focus the box and open the list (a press on it, `open`).
    pub fn open(&mut self) {
        if !self.input.is_focused() {
            self.set_focused(true);
        }
        self.open = !self.input.disabled && self.input.is_focused();
        self.reveal = true;
    }

    /// The option that has the focus (a reader's, or Tab), when one has.
    pub fn focused_option(&self) -> Option<usize> {
        self.highlighted()
            .filter(|_| self.option_focus && self.is_open())
    }

    /// The options that match the box's text, in order: all of them while
    /// it is empty or the chosen option's label.
    pub fn matches(&self) -> Vec<usize> {
        let query = self.input.text().trim();
        if query.is_empty() || Some(self.input.text()) == self.selected_label() {
            return (0..self.options.len()).collect();
        }
        let query = query.to_lowercase();
        (0..self.options.len())
            .filter(|&i| self.options[i].to_lowercase().contains(&query))
            .collect()
    }

    /// The highlighted option among the matches: the one moved to, else
    /// the chosen one, else the first.
    pub fn highlighted(&self) -> Option<usize> {
        let m = self.matches();
        [self.highlighted, self.selected]
            .into_iter()
            .flatten()
            .find(|i| m.contains(i))
            .or_else(|| m.first().copied())
    }

    /// Focus the box (its text selected whole, so typing replaces it) or
    /// blur it, back to the chosen option's label.
    pub fn set_focused(&mut self, focused: bool) {
        let label = self.selected_label().unwrap_or_default().to_owned();
        self.input.set_text(&label);
        self.highlighted = None;
        self.scroll = ScrollState::new();
        self.reveal = focused;
        self.open = false;
        self.option_focus = false;
        self.input.set_focused(focused);
        if focused {
            self.input.select_all();
        }
    }

    /// Choose option `index`: its label in the box, the list closed.
    /// `false` when there is no such option or it is disabled.
    pub fn pick(&mut self, index: usize) -> bool {
        if index >= self.options.len() || self.input.disabled {
            return false;
        }
        self.selected = Some(index);
        self.set_focused(false);
        true
    }

    /// Choose the option labeled `label`.
    pub fn pick_label(&mut self, label: &str) -> bool {
        match self.options.iter().position(|o| o == label) {
            Some(i) => self.pick(i),
            None => false,
        }
    }

    /// Up (`down` false) or Down over the matches, the list scrolled to
    /// show the highlight when it is next laid out. `false` while closed.
    pub fn nav(&mut self, down: bool) -> bool {
        if !self.input.is_focused() || self.input.disabled {
            return false;
        }
        self.option_focus = false;
        if !self.is_open() {
            self.open = true;
            self.reveal = true;
            return true;
        }
        let m = self.matches();
        let Some(last) = m.len().checked_sub(1) else {
            return true;
        };
        let at = self
            .highlighted()
            .and_then(|h| m.iter().position(|&i| i == h));
        let to = match (at, down) {
            (None, _) => 0,
            (Some(k), true) => (k + 1).min(last),
            (Some(k), false) => k.saturating_sub(1),
        };
        self.highlighted = Some(m[to]);
        self.reveal = true;
        true
    }

    /// Highlight option `index` (a reader moved to it), opening the list;
    /// `false` when it is not a match.
    pub fn highlight(&mut self, index: usize) -> bool {
        if !self.is_open() {
            self.open();
        }
        if !self.is_open() || !self.matches().contains(&index) {
            return false;
        }
        self.highlighted = Some(index);
        self.option_focus = true;
        self.reveal = true;
        true
    }

    /// Enter: pick the highlighted match. `false` while closed; an open
    /// list with no match stays open.
    pub fn enter(&mut self) -> bool {
        if !self.is_open() {
            return false;
        }
        if let Some(i) = self.highlighted() {
            self.pick(i);
        }
        true
    }

    /// The text's insets: the select's padding at the start, the chevron
    /// and its gap at the end.
    fn slots(theme: &Theme) -> [Slot; 2] {
        let r = &theme.form.select;
        [Slot::Pad, Slot::Width(r.gap + r.chevron + r.pad_end)]
    }

    /// The control's height in `theme`.
    pub fn height(theme: &Theme) -> f32 {
        theme.form.combo.height
    }

    /// The control at the top of `bounds`.
    fn control(bounds: Rect, theme: &Theme) -> Rect {
        Rect::new(bounds.x, bounds.y, bounds.w, Self::height(theme))
    }

    /// The open list below the control, and its rows: the matches, or one
    /// row saying there are none.
    fn panel(&self, bounds: Rect, theme: &Theme) -> (Rect, Window) {
        let n = self.matches().len().max(1);
        let dd = list::panel(
            Self::control(bounds, theme),
            n,
            theme.form.select.panel.gap,
            theme,
        );
        (
            dd,
            Window::new(dd, n, self.scroll_for(theme).offset(), theme),
        )
    }

    /// The scroll as the list shows it: synced to the matches and, after
    /// a key moved the highlight, moved the least that shows it. Pure, so
    /// the draw shows what the next event stores.
    fn scroll_for(&self, theme: &Theme) -> ScrollState {
        let m = self.matches();
        let n = m.len().max(1);
        let mut scroll = self.scroll.clone();
        list::sync(&mut scroll, n, theme);
        if self.reveal
            && let Some(k) = self
                .highlighted()
                .and_then(|h| m.iter().position(|&i| i == h))
        {
            list::reveal(&mut scroll, k, n, theme);
        }
        scroll
    }

    /// The list's rect below the control, open or not: where a picture
    /// of the open state (a showcase card) draws it.
    pub fn list_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        self.panel(bounds, theme).0
    }

    /// The list's rect while open.
    pub fn dropdown_rect(&self, bounds: Rect, theme: &Theme) -> Option<Rect> {
        self.is_open().then(|| self.panel(bounds, theme).0)
    }

    /// The match under `(x, y)` in the open list.
    fn option_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        if !self.is_open() {
            return None;
        }
        let (_, w) = self.panel(bounds, theme);
        let k = w.at(x, y)?;
        self.matches().get(k).copied()
    }

    /// The pointer: a press on the control focuses it (the list opens), on
    /// a row picks it, anywhere else blurs it; the pointer over a row
    /// highlights it; the wheel over the list scrolls it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let control = Self::control(bounds, theme);
        let slots = Self::slots(theme);
        self.scroll = self.scroll_for(theme);
        self.reveal = false;
        if self.input.disabled {
            return self.input.handle_event_in(event, control, theme, slots);
        }
        match *event {
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = self.option_at(x, y, bounds, theme) {
                    self.pick(i);
                    return EventResult::clicked();
                }
                if self
                    .dropdown_rect(bounds, theme)
                    .is_some_and(|r| r.contains(x, y))
                {
                    return EventResult::changed();
                }
                if hit_rect(control, theme).contains(x, y) && !self.is_open() {
                    self.open();
                    return EventResult::clicked();
                }
                let was = self.input.is_focused();
                let r = self.input.handle_event_in(event, control, theme, slots);
                if was && !self.input.is_focused() {
                    self.set_focused(false);
                }
                r
            }
            WidgetEvent::MouseMove { x, y } => {
                let mut r = self.input.handle_event_in(event, control, theme, slots);
                if let Some(i) = self.option_at(x, y, bounds, theme)
                    && self.highlighted != Some(i)
                {
                    self.highlighted = Some(i);
                    r = r.merge(EventResult::changed());
                }
                r
            }
            WidgetEvent::Scroll { x, y, delta } => {
                match self.dropdown_rect(bounds, theme) {
                    Some(dd) if dd.contains(x, y) && delta.is_finite() => {
                        let before = self.scroll.offset();
                        self.scroll.scroll_by(delta);
                        if self.scroll.offset() != before {
                            EventResult::changed()
                        } else {
                            // The wheel stops at the list, at its end too.
                            EventResult {
                                handled: true,
                                changed: false,
                                clicked: false,
                            }
                        }
                    }
                    _ => EventResult::IGNORED,
                }
            }
            WidgetEvent::MouseUp { .. } => EventResult::IGNORED,
        }
    }

    /// Where the input method's window goes: the caret in the box.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        self.input
            .ime_area_in(Self::control(bounds, theme), theme, Self::slots(theme))
    }

    /// Draws the control: the box with the text (the chosen option, the
    /// filter while open) and the chevron.
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let control = Self::control(bounds, theme);
        let layer = LayerId::DEFAULT;
        self.input
            .render_in(c, layer, control, theme, Self::slots(theme));
        let r = &theme.form.select;
        let alpha = if self.input.disabled {
            r.disabled_alpha
        } else {
            1.0
        };
        let name = if self.is_open() {
            "chevron-up"
        } else {
            "chevron-down"
        };
        let color = with_alpha(r.chevron_color, r.chevron_color.0[3] * alpha);
        if let Some(node) = icons::icon_at(
            name,
            r.chevron,
            color,
            control.right() - r.pad_end - r.chevron,
            control.y + (control.h - r.chevron) / 2.0,
        ) {
            c.push(node);
        }
    }

    /// Draws the open list on `layer` (an overlay, so it floats over what
    /// is below the control); nothing while closed.
    pub fn render_dropdown(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        if self.is_open() {
            self.render_list(c, layer, bounds, theme);
        }
    }

    /// The list on `layer` below the control at `bounds`, open or not:
    /// the matches for the text in the box, or the row saying there are
    /// none. What the open dropdown draws, and a picture of that state.
    pub fn render_list(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let scale = &theme.typography;
        let (dd, window) = self.panel(bounds, theme);
        list::draw_body(c, layer, dd, theme);
        let m = self.matches();
        if m.is_empty() {
            let p = &theme.form.select.panel;
            let row = window.row(0);
            // one row, as an option's: cut with an ellipsis where it is
            // wider (a larger text, a longer language)
            let style = scale.run(p.text);
            let said = TextMeasurer::truncate_to_width(
                &self.empty,
                &style,
                (row.right() - dd.x - p.text_inset * 2.0).max(0.0),
            );
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&said, &style, None),
                    x: dd.x + p.text_inset,
                    y: row.y + TextMeasurer::vertical_center(&scale.run(p.text), p.row_h),
                    color: p.text_color.0,
                },
            );
            return;
        }
        let lit = self.highlighted();
        list::draw_rows(
            c,
            layer,
            dd,
            &window,
            |k| Row {
                label: &self.options[m[k]],
                id: self.sem.as_ref().map(|id| id.key(m[k].to_string())),
                selected: self.selected == Some(m[k]),
                highlighted: lit == Some(m[k]),
            },
            &scale.run(theme.form.select.panel.text),
            theme,
        );
    }

    /// Declares the combobox as `id` (see the module docs). It only reads
    /// the widget: the owner runs the actions ([`Combobox::pick_label`],
    /// [`Combobox::pick`], [`Combobox::set_focused`]).
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) {
        let selected = format!("{id}:selected");
        let expanded = format!("{id}:expanded");
        let open = self.is_open();
        let mut node = b.node(id, Role::Select, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        let mut choices: Vec<String> = Vec::with_capacity(self.options.len());
        for option in &self.options {
            if !choices.contains(option) {
                choices.push(option.clone());
            }
        }
        node.state("selected", self.selected_label().unwrap_or_default())
            .state("expanded", open)
            .state("query", self.input.text())
            .constraints(Constraints {
                editable: true,
                options: Some(choices),
                ..Constraints::default()
            })
            .enabled(!self.input.disabled);
        node.action("select")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            })
            .reads(selected.as_str());
        node.action("open").reads(expanded.as_str());
        node.action("close").reads(expanded.as_str());
        if open {
            for i in self.matches() {
                let mut row = b.node(
                    id.key(i.to_string()),
                    Role::Option,
                    self.options[i].as_str(),
                );
                row.parent(id).enabled(true);
                row.action("select").reads(selected.as_str());
            }
        }
    }
}
