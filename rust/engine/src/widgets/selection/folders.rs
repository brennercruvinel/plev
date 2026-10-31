//! The kit's folder picker (`components/Folders`): a raised panel with a
//! search box, a "New Folder" row and a row per folder, each folder
//! marked by its color. The kit's values are `theme.selection.folders`
//! ([`FoldersTokens`], `SelectionTokens::hoff_core`), derived for the
//! other themes in `selection/derive.rs`.
//!
//! A press on a row picks it ([`FolderRow`]); the pointer lights the row
//! under it. The search box is drawn as the kit draws it, with its
//! placeholder: the gallery has no text routing for it yet, so it does
//! not take typing and is not projected as a field.
//!
//! With a [`SemId`] ([`Folders::sem`]) the render binds the panel, the
//! new-folder row (`<id>.new`) and each folder (`<id>[<index>]`), and
//! [`Folders::project`] declares a `Role::Group` of buttons, each with an
//! `activate` that picks it as its press does.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::{ButtonSurface, Theme};
use crate::widgets::buttons::paint_surface;
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
    pub search_gap: f32,
    pub placeholder: Color,
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
    pub folders: Vec<Rect>,
}

#[derive(Clone, Debug)]
pub struct Folders {
    pub folders: Vec<String>,
    hovered: Option<FolderRow>,
    focused: Option<FolderRow>,
    sem: Option<SemId>,
}

impl Folders {
    pub fn new(folders: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            folders: folders.into_iter().map(Into::into).collect(),
            hovered: None,
            focused: None,
            sem: None,
        }
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
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
        (
            t.width,
            t.pad * 2.0 + t.search_h + t.search_gap + t.row_h * rows,
        )
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> FoldersLayout {
        let t = &theme.selection.folders;
        let inner = bounds.inset(t.pad);
        let search = Rect::new(inner.x, inner.y, inner.w, t.search_h);
        let row = |i: usize| {
            let y = search.bottom() + t.search_gap + t.row_h * i as f32;
            Rect::new(inner.x, y, inner.w, t.row_h)
        };
        FoldersLayout {
            panel: bounds,
            search,
            new: row(0),
            folders: (0..self.folders.len()).map(|i| row(i + 1)).collect(),
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
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.row_at(x, y, &l);
                if hit == self.hovered {
                    return (EventResult::IGNORED, None);
                }
                self.hovered = hit;
                (EventResult::changed(), None)
            }
            WidgetEvent::MouseDown { x, y } => match self.row_at(x, y, &l) {
                Some(row) => (EventResult::clicked(), Some(row)),
                None => (EventResult::IGNORED, None),
            },
            _ => (EventResult::IGNORED, None),
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
        let ty = s.y + (s.h - style.line_height) / 2.0;
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style("Search files...", &style, None),
            x: s.x + t.search_text_x,
            y: ty,
            color: t.placeholder.0,
        });
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
    /// new-folder row and each folder a `Role::Button` with `activate`.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
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
