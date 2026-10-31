//! The attachments of a message in the hoff core kit: its image preview
//! (`NewComment/ImagePreview`), square tiles with rounded corners and a
//! remove button on their top end corner that shows on hover, and, for
//! what is not a picture, a file tile in the language of its add-files
//! menu (`PromptInput/AddFiles`: the icon in its box, the name, a detail
//! under it). The tray flows the tiles and wraps them, and can end with
//! the kit's add button (the bordered icon button at its 40 step).
//!
//! Sized by its content: a picture is one tile square, a file tile as
//! wide as its name up to the tray's share (`theme.message.attachment`),
//! the name ellipsized there; the tray takes the width it is given and
//! grows a line per wrap. It keeps the room the remove buttons pass the
//! tiles by, so nothing it draws leaves its bounds.
//!
//! Behavior apart from the look: the remove button stays where a click
//! takes it while hidden, as the kit's (an `opacity-0` button still takes
//! the click), and shows on the tile's hover, on a reader's focus, and
//! always where the owner says the input has no hover (touch,
//! [`AttachmentTray::show_removes`]). A click on it removes the tile; a
//! click on the add button asks the owner for more
//! ([`AttachmentTray::take_add_request`]): what to add is the owner's
//! picker, not the tray's.
//!
//! With a [`SemId`] the render binds each tile `<id>[<key>]`, its remove
//! button `<id>[<key>].remove` and the add button `<id>.add`, keyed by
//! the key [`AttachmentTray::push`] gave, so removing one leaves the
//! others' ids alone; [`AttachmentTray::project`] declares them.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::message::{Placed, fit, placed, run, width};
use crate::drawing::compositor::{Compositor, SceneNode};
use crate::drawing::gpu::image::ImageHandle;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{IconButton, IconButtonSize, IconButtonVariant};
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, gradient_fill, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// What an attachment holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentKind {
    /// A picture: its pixels once loaded, the tile's wash until then.
    Image(Option<ImageHandle>),
    /// A document: the file tile.
    File,
    /// A 3D object: the file tile with the layers icon.
    Object,
}

#[derive(Clone, Debug)]
pub struct Attachment {
    key: u64,
    pub name: String,
    /// The size, the type: under the name of a file tile, and the
    /// description a reader hears.
    pub detail: String,
    pub kind: AttachmentKind,
}

impl Attachment {
    pub fn key(&self) -> u64 {
        self.key
    }

    fn icon(&self) -> Option<&'static str> {
        match self.kind {
            AttachmentKind::Image(_) => None,
            AttachmentKind::File => Some("file"),
            AttachmentKind::Object => Some("layers"),
        }
    }
}

/// The part of a tray a reader's focus is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayFocus {
    Tile(u64),
    Remove(u64),
    Add,
}

/// Where one tile lands.
#[derive(Clone, Debug, PartialEq)]
pub struct TileLayout {
    pub rect: Rect,
    pub remove: Rect,
    /// A file tile's icon, name and detail.
    pub icon: Option<Rect>,
    pub name: Option<Placed>,
    pub detail: Option<Placed>,
}

/// Where everything of a tray lands, from its bounds alone.
#[derive(Clone, Debug, PartialEq)]
pub struct TrayLayout {
    /// The whole tray: as wide as the bounds, as tall as its lines.
    pub rect: Rect,
    pub tiles: Vec<TileLayout>,
    pub add: Option<Rect>,
}

#[derive(Clone, Debug)]
pub struct AttachmentTray {
    items: Vec<Attachment>,
    next_key: u64,
    add: Option<IconButton>,
    add_requested: bool,
    show_removes: bool,
    hover: Option<u64>,
    hover_remove: Option<u64>,
    pressed_remove: Option<u64>,
    focus: Option<TrayFocus>,
    sem: Option<SemId>,
}

impl Default for AttachmentTray {
    fn default() -> Self {
        Self::new()
    }
}

impl AttachmentTray {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_key: 0,
            add: None,
            add_requested: false,
            show_removes: false,
            hover: None,
            hover_remove: None,
            pressed_remove: None,
            focus: None,
            sem: None,
        }
    }

    /// The kit's add button after the tiles.
    pub fn with_add(mut self) -> Self {
        let mut add = IconButton::new("plus")
            .variant(IconButtonVariant::Outline)
            .size(IconButtonSize::Lg);
        if let Some(id) = self.sem.as_ref().and_then(add_id) {
            add = add.sem(id);
        }
        self.add = Some(add);
        self
    }

    /// The input has no hover (touch): the remove buttons always show.
    pub fn show_removes(mut self, on: bool) -> Self {
        self.show_removes = on;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        if let Some(aid) = add_id(&id) {
            self.add = self.add.map(|b| b.sem(aid));
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Adds an attachment at the end; returns the key its ids carry.
    pub fn push(
        &mut self,
        name: impl Into<String>,
        detail: impl Into<String>,
        kind: AttachmentKind,
    ) -> u64 {
        let key = self.next_key;
        self.next_key += 1;
        self.items.push(Attachment {
            key,
            name: name.into(),
            detail: detail.into(),
            kind,
        });
        key
    }

    /// [`AttachmentTray::push`], chained.
    pub fn with(mut self, name: &str, detail: &str, kind: AttachmentKind) -> Self {
        self.push(name, detail, kind);
        self
    }

    /// Removes the attachment `key`, as its remove button does. `false`
    /// when there is none.
    pub fn remove(&mut self, key: u64) -> bool {
        let Some(i) = self.items.iter().position(|a| a.key == key) else {
            return false;
        };
        self.items.remove(i);
        for slot in [
            &mut self.hover,
            &mut self.hover_remove,
            &mut self.pressed_remove,
        ] {
            if *slot == Some(key) {
                *slot = None;
            }
        }
        if matches!(self.focus, Some(TrayFocus::Tile(k) | TrayFocus::Remove(k)) if k == key) {
            self.focus = None;
        }
        true
    }

    pub fn items(&self) -> &[Attachment] {
        &self.items
    }

    pub fn has_add(&self) -> bool {
        self.add.is_some()
    }

    /// Asks for more, as a click on the add button does. `false` without
    /// one.
    pub fn request_add(&mut self) -> bool {
        self.add_requested |= self.add.is_some();
        self.add.is_some()
    }

    /// Whether the add button was clicked since the last call.
    pub fn take_add_request(&mut self) -> bool {
        std::mem::take(&mut self.add_requested)
    }

    /// The part of this tray `id` names.
    pub fn target(&self, id: &SemId) -> Option<TrayFocus> {
        let root = self.sem.as_ref()?;
        if add_id(root).as_ref() == Some(id) {
            return Some(TrayFocus::Add);
        }
        self.items.iter().find_map(|a| {
            let tile = root.key(a.key.to_string());
            if &tile == id {
                Some(TrayFocus::Tile(a.key))
            } else {
                (remove_id(&tile).as_ref() == Some(id)).then_some(TrayFocus::Remove(a.key))
            }
        })
    }

    /// A reader's focus on a part, or on none.
    pub fn set_focus(&mut self, focus: Option<TrayFocus>) {
        self.focus = focus;
        if let Some(add) = &mut self.add {
            add.set_focused(focus == Some(TrayFocus::Add));
        }
    }

    pub fn focus(&self) -> Option<TrayFocus> {
        self.focus
    }

    /// Whether the remove button of `key` shows.
    pub fn remove_shown(&self, key: u64) -> bool {
        self.show_removes
            || self.hover == Some(key)
            || self.hover_remove == Some(key)
            || matches!(self.focus, Some(TrayFocus::Tile(k) | TrayFocus::Remove(k)) if k == key)
    }

    /// The narrowest tray that holds a picture tile and its remove button.
    pub fn min_width(&self, theme: &Theme) -> f32 {
        let t = &theme.message.attachment;
        t.tile + t.remove_out
    }

    pub fn height(&self, width: f32, theme: &Theme) -> f32 {
        self.layout(Rect::new(0.0, 0.0, width, 0.0), theme).rect.h
    }

    /// A file tile's width in a tray whose tiles get `room`.
    fn file_w(a: &Attachment, room: f32, theme: &Theme) -> f32 {
        let t = &theme.message.attachment;
        let ty = &theme.typography;
        let text = width(&a.name, &t.name.style(ty)).max(width(&a.detail, &t.detail.style(ty)));
        let frame = t.file_pad * 2.0 + t.file_icon_box + t.file_gap;
        // on the whole pixel: the column the layout subtracts back out of
        // the tile's place must not come short of the text by a rounding
        (frame + text.ceil())
            .min(room * t.file_max_fraction)
            .max(t.tile)
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> TrayLayout {
        let t = &theme.message.attachment;
        let ty = &theme.typography;
        // the room the remove buttons pass the tiles by, at the top and
        // the end
        let x0 = bounds.x;
        let right = (bounds.right() - t.remove_out).max(x0);
        let top = bounds.y + t.remove_out;
        let room = right - x0;
        let widths = self.items.iter().map(|a| match a.kind {
            AttachmentKind::Image(_) => t.tile.min(room),
            _ => Self::file_w(a, room, theme).min(room),
        });
        let add_w = self.add.as_ref().map(|b| b.preferred_size(theme).0);
        let mut rects = Vec::new();
        let (mut x, mut y) = (x0, top);
        for w in widths.chain(add_w) {
            if x > x0 && x + w > right {
                x = x0;
                y += t.tile + t.gap;
            }
            rects.push(Rect::new(x, y, w, t.tile));
            x += w + t.gap;
        }
        let add = add_w.and_then(|_| rects.pop()).map(|r| {
            let (w, h) = self
                .add
                .as_ref()
                .map_or((0.0, 0.0), |b| b.preferred_size(theme));
            Rect::new(r.x, r.y + (r.h - h) / 2.0, w, h)
        });
        let tiles = self
            .items
            .iter()
            .zip(rects)
            .map(|(a, rect)| {
                let remove = Rect::new(
                    rect.right() - t.remove + t.remove_out,
                    rect.y - t.remove_out,
                    t.remove,
                    t.remove,
                );
                let Some(_) = a.icon() else {
                    return TileLayout {
                        rect,
                        remove,
                        icon: None,
                        name: None,
                        detail: None,
                    };
                };
                let icon_box = Rect::new(
                    rect.x + t.file_pad,
                    rect.y + (rect.h - t.file_icon_box) / 2.0,
                    t.file_icon_box,
                    t.file_icon_box,
                );
                let icon = Rect::new(
                    icon_box.x + (t.file_icon_box - t.file_icon) / 2.0,
                    icon_box.y + (t.file_icon_box - t.file_icon) / 2.0,
                    t.file_icon,
                    t.file_icon,
                );
                let col_x = icon_box.right() + t.file_gap;
                let col_w = (rect.right() - t.file_pad - col_x).max(0.0);
                let (ns, ds) = (t.name.style(ty), t.detail.style(ty));
                let name = fit(&a.name, &ns, col_w);
                let detail = fit(&a.detail, &ds, col_w);
                let lines = [
                    (!name.is_empty(), ns.line_height),
                    (!detail.is_empty(), ds.line_height),
                ];
                let text_h: f32 = lines.iter().filter(|(on, _)| *on).map(|(_, h)| h).sum();
                let ty0 = rect.y + (rect.h - text_h) / 2.0;
                let (nw, dw) = (width(&name, &ns), width(&detail, &ds));
                let name_h = if name.is_empty() { 0.0 } else { ns.line_height };
                TileLayout {
                    rect,
                    remove,
                    icon: Some(icon),
                    name: placed(name, col_x, ty0, nw, &ns),
                    detail: placed(detail, col_x, ty0 + name_h, dw, &ds),
                }
            })
            .collect::<Vec<_>>();
        let bottom = tiles
            .iter()
            .map(|l| l.rect.bottom())
            .chain(add.map(|r| r.bottom()))
            .fold(top, f32::max);
        TrayLayout {
            rect: Rect::new(bounds.x, bounds.y, bounds.w, bottom - bounds.y),
            tiles,
            add,
        }
    }

    /// Hover on a tile or its remove button; a click on the remove button
    /// removes the tile, on the add button asks for more.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        let mut result = EventResult::IGNORED;
        if let (Some(add), Some(r)) = (&mut self.add, l.add) {
            let r = add.handle_event(event, r);
            if r.clicked {
                self.add_requested = true;
            }
            result = result.merge(r);
        }
        let keys: Vec<u64> = self.items.iter().map(|a| a.key).collect();
        let on_remove = |x: f32, y: f32| {
            keys.iter()
                .zip(&l.tiles)
                .find(|(_, t)| t.remove.contains(x, y))
                .map(|(k, _)| *k)
        };
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let remove = on_remove(x, y);
                let tile = remove.or_else(|| {
                    keys.iter()
                        .zip(&l.tiles)
                        .find(|(_, t)| t.rect.contains(x, y))
                        .map(|(k, _)| *k)
                });
                if (tile, remove) != (self.hover, self.hover_remove) {
                    self.hover = tile;
                    self.hover_remove = remove;
                    result = result.merge(EventResult::changed());
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(k) = on_remove(x, y) {
                    self.pressed_remove = Some(k);
                    result = result.merge(EventResult::changed());
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                if let Some(k) = self.pressed_remove.take() {
                    if on_remove(x, y) == Some(k) {
                        self.remove(k);
                        result = result.merge(EventResult::clicked());
                    } else {
                        result = result.merge(EventResult::changed());
                    }
                }
            }
            WidgetEvent::Scroll { .. } => {}
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.message.attachment;
        let ty = &theme.typography;
        let l = self.layout(bounds, theme);
        for (a, tl) in self.items.iter().zip(&l.tiles) {
            let r = tl.rect;
            let tile = self.sem.as_ref().map(|id| id.key(a.key.to_string()));
            if let Some(id) = &tile {
                c.bind(id, [r.x, r.y, r.w, r.h]);
            }
            match a.kind {
                AttachmentKind::Image(Some(image)) => c.push(SceneNode::Image {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                    image,
                    corner_radius: t.radius,
                }),
                AttachmentKind::Image(None) => {
                    c.push(rounded_rect(r.x, r.y, r.w, r.h, t.radius, t.placeholder.0));
                }
                _ => {
                    c.push(SceneNode::RoundedRect {
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: r.h,
                        color: t.file_fill.0,
                        corner_radius: t.radius,
                        border_width: t.file_border_width,
                        border_color: t.file_border.0,
                    });
                }
            }
            if let (Some(name), Some(at)) = (a.icon(), tl.icon)
                && let Some(node) = icons::icon_at(name, at.w, t.file_icon_color.0, at.x, at.y)
            {
                c.push(node);
            }
            run(c, tl.name.as_ref(), &t.name, ty, None);
            run(c, tl.detail.as_ref(), &t.detail, ty, None);
            if self.focus == Some(TrayFocus::Tile(a.key)) {
                c.push(focus_ring(r, t.radius, theme));
            }
            // bound while hidden: the kit's button takes the click at
            // opacity 0, and the hover that follows shows it
            let rm = tl.remove;
            if let Some(id) = tile.as_ref().and_then(remove_id) {
                c.bind(&id, [rm.x, rm.y, rm.w, rm.h]);
            }
            if self.remove_shown(a.key) {
                let [top, bottom] = t.remove_fill;
                c.push(gradient_fill(rm, rm.w / 2.0, top.0, bottom.0));
                let s = t.remove_icon;
                let (cx, cy) = rm.center();
                if let Some(node) =
                    icons::icon_at("x", s, t.remove_ink.0, cx - s / 2.0, cy - s / 2.0)
                {
                    c.push(node);
                }
                if self.focus == Some(TrayFocus::Remove(a.key)) {
                    c.push(focus_ring(rm, rm.w / 2.0, theme));
                }
            }
        }
        if let (Some(add), Some(r)) = (&self.add, l.add) {
            add.render(c, r, theme);
        }
    }

    /// Declares the tray as `id`: a `Role::List` named `label`; in it each
    /// attachment `<id>[<key>]`, a `Role::ListItem` named by its name and
    /// described by its detail, focusable, and its remove button
    /// `<id>[<key>].remove` ("Remove <name>", `activate`); the add button
    /// `<id>.add` ("Add files", `activate`). Pass the id given to
    /// [`AttachmentTray::sem`]. Returns the list's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for a in &self.items {
            let tile = id.key(a.key.to_string());
            let mut node = b.node(&tile, Role::ListItem, a.name.as_str());
            node.parent(id);
            if !a.detail.is_empty() {
                node.description(a.detail.as_str());
            }
            node.focusable(true);
            if let Some(rid) = remove_id(&tile) {
                let mut rm = b.node(&rid, Role::Button, format!("Remove {}", a.name));
                rm.parent(&tile);
                rm.action("activate");
            }
        }
        if let (Some(add), Some(aid)) = (&self.add, add_id(id)) {
            add.project(b, &aid, "Add files", Some(id));
        }
        let mut node = b.node(id, Role::List, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }
}

fn add_id(id: &SemId) -> Option<SemId> {
    id.child("add").ok()
}

fn remove_id(tile: &SemId) -> Option<SemId> {
    tile.child("remove").ok()
}
