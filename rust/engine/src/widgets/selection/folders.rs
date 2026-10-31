//! The kit's folder picker (`components/Folders`): a raised panel with a
//! search box, a "New Folder" row and a row per folder, each folder
//! marked by its color. The kit's values are `theme.selection.folders`
//! ([`FoldersTokens`], `SelectionTokens::hoff_core`), derived for the
//! other themes in `selection/derive.rs`.
//!
//! A press on a row picks it ([`FolderRow`]); the pointer lights the row
//! under it. The search box and the new folder's name are front A's
//! [`TextField`]s drawn bare ([`Look::Bare`]) inside the kit's boxes,
//! which this draws: they take the caret, the selection, the input
//! method and the clipboard as every field does, through the app that
//! routes the keys to [`Folders::field_mut`]. The new-folder row opens
//! the name's box under it ([`Folders::toggle_naming`], as the kit's
//! `NewFolder`); [`Folders::commit_name`] adds the folder (Enter) and
//! [`Folders::cancel_naming`] closes the box (Escape).
//!
//! With a [`SemId`] ([`Folders::sem`]) the render binds the panel, the
//! search box (`<id>.search`), the new-folder row (`<id>.new`), the name
//! (`<id>.name`) and each folder (`<id>[<index>]`), and
//! [`Folders::project`] declares a `Role::Group`: the boxes text inputs
//! with their `set_value` (the name's `submit` adds the folder), the rows
//! buttons whose `activate` picks them as their press does.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::{ButtonSurface, Theme};
use crate::widgets::buttons::paint_surface;
use crate::widgets::form::TextField;
use crate::widgets::form::tokens::{Look, Slot};
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The folder picker's values.
#[derive(Clone, Debug, PartialEq)]
pub struct FoldersTokens {
    /// The panel: fill, drop and ring.
    pub panel: ButtonSurface,
    pub width: f32,
    pub pad: f32,
    pub radius: f32,
    /// The search box: its surface, height, radius, its icon slot and
    /// icon, the text's start, and the gap under it.
    pub search: ButtonSurface,
    pub search_h: f32,
    pub search_radius: f32,
    pub search_slot: f32,
    pub search_icon: f32,
    pub search_text_x: f32,
    /// The text's end inside a box (`pr-3`).
    pub text_end: f32,
    pub search_gap: f32,
    /// The new folder's name box (its surface), the gap over it, its
    /// icon's start and color.
    pub naming: ButtonSurface,
    pub naming_gap: f32,
    pub naming_icon_x: f32,
    pub naming_icon: Color,
    /// A row: height, radius, its hover, the icon's slot and size, the
    /// gap after the slot, the label.
    pub row_h: f32,
    pub row_radius: f32,
    pub row_hover: Color,
    pub slot: f32,
    pub icon: f32,
    pub gap: f32,
    pub label: Color,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    /// The new-folder icon and the search icon.
    pub mark: Color,
    /// The folders' colors, in turn.
    pub hues: [Color; 4],
}

/// A row of the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolderRow {
    New,
    Folder(usize),
}

/// Where the parts sit in the picker's bounds.
#[derive(Clone, Debug)]
pub struct FoldersLayout {
    pub panel: Rect,
    pub search: Rect,
    pub new: Rect,
    /// The name's box, while it is open.
    pub name: Option<Rect>,
    pub folders: Vec<Rect>,
}

/// The placeholder of the new folder's name, and the name it takes when
/// left empty (`NewFolder/index.tsx:46`).
const UNTITLED: &str = "Untitled Folder";

#[derive(Clone, Debug)]
pub struct Folders {
    pub folders: Vec<String>,
    /// The search box (`Search files...`).
    pub search: TextField,
    /// The new folder's name, while its box is open.
    naming: Option<TextField>,
    hovered: Option<FolderRow>,
    focused: Option<FolderRow>,
    sem: Option<SemId>,
}

impl Folders {
    pub fn new(folders: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            folders: folders.into_iter().map(Into::into).collect(),
            search: TextField::new("Search files...").look(Look::Bare),
            naming: None,
            hovered: None,
            focused: None,
            sem: None,
        }
    }

    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(search) = id.child("search") {
            self.search = self.search.sem(search);
        }
        self.sem = Some(id);
        self
    }

    /// The name box's id, `<id>.name`.
    pub fn name_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("name").ok()
    }

    pub fn naming(&self) -> Option<&TextField> {
        self.naming.as_ref()
    }

    pub fn naming_mut(&mut self) -> Option<&mut TextField> {
        self.naming.as_mut()
    }

    /// Open the name's box (its caret in it), or close it, as a press on
    /// the new-folder row does.
    pub fn toggle_naming(&mut self) -> bool {
        if self.naming.take().is_none() {
            let mut field = TextField::new(UNTITLED).look(Look::Bare);
            if let Some(id) = self.name_id() {
                field = field.sem(id);
            }
            field.focus();
            self.search.unfocus();
            self.naming = Some(field);
        }
        true
    }

    /// Add the folder the name's box holds (its placeholder when empty)
    /// and close the box: Enter. The name, `None` with no box open.
    pub fn commit_name(&mut self) -> Option<String> {
        let field = self.naming.take()?;
        let typed = field.text().trim();
        let name = if typed.is_empty() { UNTITLED } else { typed }.to_owned();
        self.folders.push(name.clone());
        Some(name)
    }

    /// Close the name's box, adding nothing: Escape.
    pub fn cancel_naming(&mut self) -> bool {
        self.naming.take().is_some()
    }

    /// The field that has the keys: the focused search box or name.
    pub fn field_mut(&mut self) -> Option<&mut TextField> {
        if self.search.is_focused() {
            return Some(&mut self.search);
        }
        self.naming.as_mut().filter(|f| f.is_focused())
    }

    pub fn field(&self) -> Option<&TextField> {
        if self.search.is_focused() {
            return Some(&self.search);
        }
        self.naming.as_ref().filter(|f| f.is_focused())
    }

    /// The keys to the name (`true`) or to the search box.
    pub fn focus_field(&mut self, name: bool) {
        self.search.set_focused(!name);
        if let Some(f) = &mut self.naming {
            f.set_focused(name);
        }
    }

    /// Neither box has the keys; `true` when one had.
    pub fn blur_fields(&mut self) -> bool {
        let had = self.field().is_some();
        self.search.unfocus();
        if let Some(f) = &mut self.naming {
            f.unfocus();
        }
        had
    }

    /// The ends that put a box's text where the kit puts it: after its
    /// icon (`pl-12`), before its end (`pr-3`).
    pub fn slots(theme: &Theme) -> [Slot; 2] {
        let t = &theme.selection.folders;
        [Slot::Width(t.search_text_x), Slot::Width(t.text_end)]
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// The id of `row`: `<id>.new`, or `<id>[<index>]`.
    pub fn row_id(&self, row: FolderRow) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        match row {
            FolderRow::New => id.child("new").ok(),
            FolderRow::Folder(i) => Some(id.key(i.to_string())),
        }
    }

    /// The row `id` names.
    pub fn row_at_id(&self, id: &SemId) -> Option<FolderRow> {
        (0..self.folders.len())
            .map(FolderRow::Folder)
            .chain([FolderRow::New])
            .find(|r| self.row_id(*r).as_ref() == Some(id))
    }

    /// Keyboard focus on a row, or on none: its ring.
    pub fn set_focused(&mut self, row: Option<FolderRow>) {
        self.focused = row;
    }

    pub fn focused(&self) -> Option<FolderRow> {
        self.focused
    }

    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.selection.folders;
        let rows = (self.folders.len() + 1) as f32;
        let name = if self.naming.is_some() {
            t.naming_gap + t.search_h
        } else {
            0.0
        };
        (
            t.width,
            t.pad * 2.0 + t.search_h + t.search_gap + t.row_h * rows + name,
        )
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> FoldersLayout {
        let t = &theme.selection.folders;
        let inner = bounds.inset(t.pad);
        let search = Rect::new(inner.x, inner.y, inner.w, t.search_h);
        let new = Rect::new(inner.x, search.bottom() + t.search_gap, inner.w, t.row_h);
        let name = self
            .naming
            .as_ref()
            .map(|_| Rect::new(inner.x, new.bottom() + t.naming_gap, inner.w, t.search_h));
        let top = name.map_or(new.bottom(), |n| n.bottom());
        let row = |i: usize| Rect::new(inner.x, top + t.row_h * i as f32, inner.w, t.row_h);
        FoldersLayout {
            panel: bounds,
            search,
            new,
            name,
            folders: (0..self.folders.len()).map(row).collect(),
        }
    }

    fn row_at(&self, x: f32, y: f32, l: &FoldersLayout) -> Option<FolderRow> {
        if l.new.contains(x, y) {
            return Some(FolderRow::New);
        }
        l.folders
            .iter()
            .position(|r| r.contains(x, y))
            .map(FolderRow::Folder)
    }

    /// The pointer lights the row under it; a press picks it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<FolderRow>) {
        let l = self.layout(bounds, theme);
        // the boxes take the press (a caret, or a blur) and the drag
        let slots = Self::slots(theme);
        let mut boxes = self.search.handle_event_in(event, l.search, theme, slots);
        if let (Some(f), Some(r)) = (&mut self.naming, l.name) {
            boxes = boxes.merge(f.handle_event_in(event, r, theme, slots));
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.row_at(x, y, &l);
                if hit == self.hovered {
                    return (EventResult::IGNORED, None);
                }
                self.hovered = hit;
                (boxes.merge(EventResult::changed()), None)
            }
            WidgetEvent::MouseDown { x, y } => match self.row_at(x, y, &l) {
                Some(row) => (EventResult::clicked(), Some(row)),
                None => (boxes, None),
            },
            _ => (boxes, None),
        }
    }

    fn style(theme: &Theme) -> TextStyle {
        let t = &theme.selection.folders;
        theme
            .typography
            .exact(t.font_size, t.line_height, t.font_weight)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.selection.folders;
        let l = self.layout(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        paint_surface(
            c,
            LayerId::DEFAULT,
            l.panel,
            t.radius,
            &t.panel,
            1.0,
            false,
            theme,
        );
        let style = Self::style(theme);
        // the search box, its icon and its placeholder
        let s = l.search;
        paint_surface(
            c,
            LayerId::DEFAULT,
            s,
            t.search_radius,
            &t.search,
            1.0,
            false,
            theme,
        );
        let slot_x = s.x + (s.h - t.search_slot) / 2.0;
        let ix = slot_x + (t.search_slot - t.search_icon) / 2.0;
        let iy = s.y + (s.h - t.search_icon) / 2.0;
        if let Some(n) = icons::icon_at("kit-search", t.search_icon, t.mark.0, ix, iy) {
            c.push(n);
        }
        let slots = Self::slots(theme);
        self.search.render_in(c, LayerId::DEFAULT, s, theme, slots);
        if let (Some(f), Some(r)) = (&self.naming, l.name) {
            let (w, ink) = (t.search_radius, t.naming_icon.0);
            paint_surface(c, LayerId::DEFAULT, r, w, &t.naming, 1.0, false, theme);
            let (ix, iy) = (r.x + t.naming_icon_x, r.y + (r.h - t.icon) / 2.0);
            if let Some(n) = icons::icon_at("kit-folder", t.icon, ink, ix, iy) {
                c.push(n);
            }
            f.render_in(c, LayerId::DEFAULT, r, theme, slots);
        }
        let rows = std::iter::once((FolderRow::New, l.new, "New Folder")).chain(
            l.folders
                .iter()
                .zip(&self.folders)
                .enumerate()
                .map(|(i, (r, name))| (FolderRow::Folder(i), *r, name.as_str())),
        );
        for (row, r, label) in rows {
            if let Some(id) = self.row_id(row) {
                c.bind(&id, [r.x, r.y, r.w, r.h]);
            }
            if self.hovered == Some(row) {
                c.push(rounded_rect(
                    r.x,
                    r.y,
                    r.w,
                    r.h,
                    t.row_radius,
                    t.row_hover.0,
                ));
            }
            if self.focused == Some(row) {
                c.push(focus_ring(r, t.row_radius, theme));
            }
            let (name, color) = match row {
                FolderRow::New => ("kit-folder-new", t.mark),
                FolderRow::Folder(i) => ("kit-folder", t.hues[i % t.hues.len()]),
            };
            let sx = r.x + (r.h - t.slot) / 2.0;
            let ix = sx + (t.slot - t.icon) / 2.0;
            if let Some(n) = icons::icon_at(name, t.icon, color.0, ix, r.y + (r.h - t.icon) / 2.0) {
                c.push(n);
            }
            let x = sx + t.slot + t.gap;
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, Some((r.right() - x).max(0.0))),
                x,
                y: r.y + (r.h - style.line_height) / 2.0,
                color: t.label.0,
            });
        }
    }

    /// Declares the picker as `id`: a `Role::Group` named `label`, the
    /// search box and the open name a `Role::TextInput` (the name with a
    /// `submit`), the new-folder row and each folder a `Role::Button`
    /// with `activate`.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        if let Some(at) = self.search.sem_id() {
            self.search.project(b, at, "Search files", Some(id));
        }
        if let Some(f) = &self.naming
            && let Some(at) = f.sem_id()
        {
            f.project(b, at, "New folder name", Some(id))
                .action("submit");
        }
        let rows = std::iter::once((FolderRow::New, "New Folder")).chain(
            self.folders
                .iter()
                .enumerate()
                .map(|(i, n)| (FolderRow::Folder(i), n.as_str())),
        );
        for (row, name) in rows {
            if let Some(at) = self.row_id(row) {
                let mut node = b.node(&at, Role::Button, name);
                node.parent(id);
                node.action("activate");
            }
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
    }
}
