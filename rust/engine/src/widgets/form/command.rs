//! Command: a palette inline, the kit's Search (`components/Search/
//! index.tsx`, the box with its search icon) over the list a combobox
//! opens (the select's panel and rows, `super::select::list`), here part
//! of the widget and always shown. The commands sit in groups, each
//! under its caption (the Label's, [`super::label::Label::caption`]),
//! with an icon and a shortcut hint (the Search's `⌘ K` keycap); typing
//! filters them by label, any case, and a group with no match goes; none
//! at all says so.
//!
//! The box is an [`InputGroup`] (`command.search`): typing, the editing
//! keys, the clipboard and the input method reach its field. Up and Down
//! move the highlight over the commands shown ([`Command::nav`]), Enter
//! runs it ([`Command::enter`]), a press on a row runs that one; the
//! command run last is [`Command::ran`] for the owner to act on.
//!
//! Inline, the list stands on its panel; on a surface that already is
//! one, as the dialog's sheet ([`Command::bare`]), the rows stand on it.
//!
//! Projected ([`Command::project`]): the search box as its field is; the
//! list beside it (`<id>.list`, `Role::List`, its `count` the commands)
//! with the command run last as `ran`, and in it each command shown
//! (`<id>.list[<index>]`, `Role::ListItem`) with a `select` that runs it;
//! one of them takes Tab, the highlighted one (a roving tabindex, as a
//! list's rows).

use accord::{Role, SemId, SnapshotBuilder};

use crate::control::input::scroll::ScrollState;
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::content::kbd::Kbd;
use crate::widgets::form::input_group::{Addon, InputGroup};
use crate::widgets::form::select::list::{self, Window};
use crate::widgets::icons;
use crate::widgets::recipe::rounded_rect;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// `color` at `alpha` of its own opacity.
fn faded(color: Color, alpha: f32) -> Color {
    Color(with_alpha(color, color.0[3] * alpha))
}

/// One command: its label, a Lucide icon and a shortcut, in a group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandItem {
    pub label: String,
    pub icon: Option<&'static str>,
    pub shortcut: Option<String>,
    pub group: usize,
}

/// A row of the list: a group's caption or a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandRow {
    Heading(usize),
    Item(usize),
}

/// Where the box and the list go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CommandLayout {
    pub search: Rect,
    pub list: Rect,
}

/// An inline command palette.
#[derive(Clone, Debug)]
pub struct Command {
    pub search: InputGroup,
    groups: Vec<String>,
    items: Vec<CommandItem>,
    /// The command the arrows or the pointer moved to.
    highlighted: Option<usize>,
    scroll: ScrollState,
    /// A key moved the highlight: the list scrolls to it when next laid
    /// out.
    reveal: bool,
    ran: Option<usize>,
    /// A reader or Tab put the focus on the highlighted command.
    item_focus: bool,
    empty: String,
    /// The list on its own panel (inline); off on a surface that already
    /// is one, as the dialog's sheet.
    panel: bool,
    sem: Option<SemId>,
}

impl Command {
    pub fn new(placeholder: &str) -> Self {
        Self {
            search: InputGroup::new(placeholder).start(Addon::Icon("search")),
            groups: Vec::new(),
            items: Vec::new(),
            highlighted: None,
            scroll: ScrollState::new(),
            reveal: false,
            ran: None,
            item_focus: false,
            empty: "No results found.".into(),
            panel: true,
            sem: None,
        }
    }

    /// A group under `caption`, with its commands: `(label, icon,
    /// shortcut)`.
    pub fn group(
        mut self,
        caption: impl Into<String>,
        items: impl IntoIterator<Item = (&'static str, Option<&'static str>, Option<&'static str>)>,
    ) -> Self {
        let group = self.groups.len();
        self.groups.push(caption.into());
        for (label, icon, shortcut) in items {
            self.items.push(CommandItem {
                label: label.into(),
                icon,
                shortcut: shortcut.map(Into::into),
                group,
            });
        }
        self
    }

    /// The box's text at the start: the list filtered by it.
    pub fn with_query(mut self, query: &str) -> Self {
        self.search.input.set_text(query);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.search = self.search.disabled(disabled);
        self
    }

    /// What the list says when nothing matches.
    pub fn empty(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }

    /// The list straight on the surface under it, without a panel of its
    /// own: the palette on a sheet that already is one.
    pub fn bare(mut self) -> Self {
        self.panel = false;
        self
    }

    /// The id the box binds to and projects as; the list is `<id>.list`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.search = self.search.sem(id.clone());
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn list_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("list").ok()
    }

    pub fn items(&self) -> &[CommandItem] {
        &self.items
    }

    /// The command run last.
    pub fn ran(&self) -> Option<&CommandItem> {
        self.ran.map(|i| &self.items[i])
    }

    /// The commands whose label holds the box's text, any case.
    pub fn matches(&self) -> Vec<usize> {
        let query = self.search.input.text().trim().to_lowercase();
        (0..self.items.len())
            .filter(|&i| query.is_empty() || self.items[i].label.to_lowercase().contains(&query))
            .collect()
    }

    /// The rows shown: each group with a match under its caption.
    pub fn rows(&self) -> Vec<CommandRow> {
        let m = self.matches();
        let mut rows = Vec::new();
        for g in 0..self.groups.len() {
            let mut shown = m.iter().filter(|&&i| self.items[i].group == g).peekable();
            if shown.peek().is_some() {
                rows.push(CommandRow::Heading(g));
                rows.extend(shown.map(|&i| CommandRow::Item(i)));
            }
        }
        rows
    }

    /// The highlighted command among those shown: the one moved to, else
    /// the first. None while the palette is disabled: nothing there runs.
    pub fn highlighted(&self) -> Option<usize> {
        if self.search.input.disabled {
            return None;
        }
        let m = self.matches();
        self.highlighted
            .filter(|h| m.contains(h))
            .or_else(|| m.first().copied())
    }

    /// Up (`down` false) or Down over the commands shown, stopping at the
    /// ends.
    pub fn nav(&mut self, down: bool) -> bool {
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

    /// The focus on command `index` (a reader's, or Tab), highlighting it,
    /// or off the commands. `false` when it is not shown.
    pub fn focus_item(&mut self, index: Option<usize>) -> bool {
        match index {
            Some(i) if self.matches().contains(&i) => {
                self.highlighted = Some(i);
                self.item_focus = true;
                self.reveal = true;
                true
            }
            Some(_) => false,
            None => {
                self.item_focus = false;
                true
            }
        }
    }

    /// The command that has the focus, when one has.
    pub fn focused_item(&self) -> Option<usize> {
        self.highlighted().filter(|_| self.item_focus)
    }

    /// Run command `index`: it is [`ran`](Self::ran). `false` when there
    /// is no such command or the palette is disabled.
    pub fn run(&mut self, index: usize) -> bool {
        if index >= self.items.len() || self.search.input.disabled {
            return false;
        }
        self.ran = Some(index);
        self.highlighted = Some(index);
        true
    }

    /// Enter: run the highlighted command. `false` with none shown.
    pub fn enter(&mut self) -> bool {
        self.highlighted().is_some_and(|i| self.run(i))
    }

    /// Where the box and the list go in `bounds` (its height is not read:
    /// the palette takes [`height_for`](Self::height_for)).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> CommandLayout {
        let search = Rect::new(bounds.x, bounds.y, bounds.w, self.search.height(theme));
        // The list a select opens, its rows (one at least, for the empty
        // state) scrolling in the same strip.
        let p = &theme.form.select.panel;
        let list = Rect::new(
            bounds.x,
            search.bottom() + theme.form.field.label_gap,
            bounds.w,
            p.pad * 2.0 + list::view_h(self.rows().len().max(1), theme),
        );
        CommandLayout { search, list }
    }

    /// The width its content asks for: the widest command's row (its
    /// insets, its icon, its label as it measures, its key cap), in the
    /// list's padding. A host clamps it between its floor and its room.
    pub fn preferred_width(&self, theme: &Theme) -> f32 {
        let p = &theme.form.select.panel;
        let gap = theme.form.select.gap;
        let row = |item: &CommandItem| {
            let (label, _) = TextMeasurer::measure_styled(&item.label, &p.text, None);
            let icon = item.icon.map_or(0.0, |_| theme.form.group.icon + gap);
            let cap = item
                .shortcut
                .as_ref()
                .map_or(0.0, |k| gap + Kbd::new(k.as_str()).preferred_size(theme).0);
            p.text_inset * 2.0 + icon + label.ceil() + cap
        };
        let widest = self.items.iter().map(row).fold(0.0, f32::max);
        widest + p.pad * 2.0
    }

    pub fn height_for(&self, w: f32, theme: &Theme) -> f32 {
        self.layout(Rect::new(0.0, 0.0, w, 0.0), theme)
            .list
            .bottom()
    }

    /// The scroll as the list shows it, the highlight revealed after a
    /// key. Pure: the draw shows what the next event stores.
    fn scroll_for(&self, theme: &Theme) -> ScrollState {
        let rows = self.rows();
        let mut scroll = self.scroll.clone();
        list::sync(&mut scroll, rows.len().max(1), theme);
        if self.reveal
            && let Some(k) = self
                .highlighted()
                .and_then(|h| rows.iter().position(|r| *r == CommandRow::Item(h)))
        {
            list::reveal(&mut scroll, k, rows.len(), theme);
        }
        scroll
    }

    fn window(&self, bounds: Rect, theme: &Theme) -> Window {
        let l = self.layout(bounds, theme);
        let n = self.rows().len().max(1);
        Window::new(l.list, n, self.scroll_for(theme).offset(), theme)
    }

    /// The command under `(x, y)`.
    pub fn item_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        match self.rows().get(self.window(bounds, theme).at(x, y)?) {
            Some(CommandRow::Item(i)) => Some(*i),
            _ => None,
        }
    }

    /// The pointer: the box's own, a row hovered highlights it, a press on
    /// it runs it, the wheel over the list scrolls it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.scroll = self.scroll_for(theme);
        self.reveal = false;
        let l = self.layout(bounds, theme);
        let r = self.search.handle_event(event, l.search, theme);
        if self.search.input.disabled {
            return r;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => match self.item_at(x, y, bounds, theme) {
                Some(i) if self.highlighted != Some(i) => {
                    self.highlighted = Some(i);
                    r.merge(EventResult::changed())
                }
                _ => r,
            },
            WidgetEvent::MouseDown { x, y } => match self.item_at(x, y, bounds, theme) {
                Some(i) => {
                    self.run(i);
                    r.merge(EventResult::clicked())
                }
                None => r,
            },
            WidgetEvent::Scroll { x, y, delta } if l.list.contains(x, y) && delta.is_finite() => {
                let before = self.scroll.offset();
                self.scroll.scroll_by(delta);
                if self.scroll.offset() != before {
                    EventResult::changed()
                } else {
                    r
                }
            }
            _ => r,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        // the layer its owner aims at (`Compositor::on_layer`)
        let layer = c.target();
        let l = self.layout(bounds, theme);
        self.search.render(c, l.search, theme);
        if self.panel {
            list::draw_body(c, layer, l.list, theme);
        }
        let p = &theme.form.select.panel;
        let rows = self.rows();
        if rows.is_empty() {
            let row = Rect::new(l.list.x, l.list.y + p.pad, l.list.w, p.row_h);
            self.text(c, &self.empty, row, p.text_inset, p.text_color, theme);
            return;
        }
        let window = self.window(bounds, theme);
        let v = window.view;
        c.push(SceneNode::PushClip {
            x: v.x,
            y: v.y,
            w: v.w,
            h: v.h,
        });
        let lit = self.highlighted();
        let caption = &theme.form.label;
        // disabled, the list fades as its box does (the kit's disabled
        // button, `opacity-30`): its rows read as not there to press
        let alpha = if self.search.input.disabled {
            theme.form.select.disabled_alpha
        } else {
            1.0
        };
        for k in window.rows.clone() {
            let row = window.row(k);
            match rows[k] {
                CommandRow::Heading(g) => {
                    let y = row.y + TextMeasurer::vertical_center(&caption.caption, row.h);
                    c.push(SceneNode::Text {
                        key: TextNodeKey::from_style(&self.groups[g], &caption.caption, None),
                        x: row.x + p.text_inset,
                        y,
                        color: faded(caption.caption_color, alpha).0,
                    });
                }
                CommandRow::Item(i) => self.item(c, i, row, lit == Some(i), alpha, theme),
            }
        }
        c.push(SceneNode::PopClip);
    }

    /// One line of text in `row`, `inset` from its start.
    fn text(
        &self,
        c: &mut Compositor,
        text: &str,
        row: Rect,
        inset: f32,
        color: crate::styling::color::Color,
        theme: &Theme,
    ) {
        let style = &theme.form.select.panel.text;
        let shown = TextMeasurer::truncate_to_width(text, style, (row.w - inset * 2.0).max(0.0));
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&shown, style, None),
            x: row.x + inset,
            y: row.y + TextMeasurer::vertical_center(style, row.h),
            color: color.0,
        });
    }

    /// A command's row: its fill when highlighted, its icon, its label and
    /// its shortcut's keycap at the end; bound to `<list>[<index>]`.
    fn item(&self, c: &mut Compositor, i: usize, row: Rect, lit: bool, alpha: f32, theme: &Theme) {
        let p = &theme.form.select.panel;
        let g = &theme.form.group;
        let item = &self.items[i];
        if let Some(id) = self.list_id() {
            c.bind(&id.key(i.to_string()), [row.x, row.y, row.w, row.h]);
        }
        if lit {
            let r = Rect::new(row.x + p.pad, row.y, row.w - p.pad * 2.0, row.h);
            let radius = p.row_radius.min(r.h / 2.0);
            c.push(rounded_rect(r.x, r.y, r.w, r.h, radius, p.selected_fill.0));
        }
        let color = faded(if lit { p.active_color } else { p.text_color }, alpha);
        let mut inset = p.text_inset;
        if let Some(name) = item.icon {
            let size = g.icon;
            let y = row.y + (row.h - size) / 2.0;
            if let Some(node) = icons::icon_at(name, size, color.0, row.x + inset, y) {
                c.push(node);
            }
            inset += size + theme.form.select.gap;
        }
        let mut end = row.right() - p.text_inset;
        if let Some(keys) = &item.shortcut {
            // A menu row's key cap, as the kit's menus draw a shortcut.
            let cap = Kbd::new(keys.as_str()).disabled(alpha < 1.0);
            let (w, h) = cap.preferred_size(theme);
            let r = Rect::new(end - w, row.y + (row.h - h) / 2.0, w, h);
            cap.render(c, r, theme);
            end = r.x - theme.form.select.gap;
        }
        let label = Rect::new(row.x + inset, row.y, (end - row.x - inset).max(0.0), row.h);
        self.text(c, &item.label, label, 0.0, color, theme);
    }

    /// Declares the palette as `id` (see the module docs). It only reads
    /// the widget: the owner runs `set_value` on the box and `select` on
    /// a command ([`Command::run`]).
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) {
        self.search.project(b, id, label, parent);
        let Some(list) = self.list_id() else {
            return;
        };
        let ran = format!("{list}:ran");
        let mut node = b.node(&list, Role::List, "commands");
        node.parent(parent.unwrap_or(id));
        node.state("ran", self.ran().map_or("", |c| c.label.as_str()))
            .state("count", i64::try_from(self.items.len()).unwrap_or(i64::MAX))
            .enabled(!self.search.input.disabled);
        let roving = self.highlighted();
        for row in self.rows() {
            if let CommandRow::Item(i) = row {
                let mut item = b.node(
                    list.key(i.to_string()),
                    Role::ListItem,
                    self.items[i].label.as_str(),
                );
                item.parent(&list)
                    .enabled(!self.search.input.disabled)
                    .focusable(roving == Some(i));
                if let Some(keys) = &self.items[i].shortcut {
                    item.description(keys.as_str());
                }
                item.action("select").reads(ran.as_str());
            }
        }
    }
}
