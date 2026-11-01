//! The kit's HomeSidebar (`components/HomeSidebar`): the logo, the
//! navigation (a dropdown per section with its links, the collapsible of
//! front D, and plain links), the "My scenes" caption and link, then the
//! folders: the "New Folder" row, which opens the field that names a new
//! one (`components/NewFolder`), and a row per folder
//! (`components/Folder`), drawn as the folder picker of front E draws
//! its rows.
//!
//! Without its library ([`HomeSidebar::library`]) it is the navigation
//! alone, an app's own sidebar (the gallery's). Everything under the logo
//! is the body, which scrolls when it runs past the sidebar's height (the
//! kit's `overflow-y-auto scrollbar-none`: a wheel, no bar); the logo
//! stays.
//!
//! Every row is one line: a label that does not fit the width ends in an
//! ellipsis (the dropdowns and their links are truncated, as a nav link
//! and a folder are), so a narrow sidebar never breaks a word; the
//! projected names are whole.
//!
//! With a [`SemId`] the rows are `<id>.row[<r>]` (a dropdown's links
//! `<id>.row[<r>].links[<i>]`), the scenes link `<id>.scenes`, the new
//! folder `<id>.new` and the folders `<id>.folder[<i>]`.

use accord::SemId;

use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::content::{Collapsible, DisclosureList};
use crate::widgets::form::TextField;
use crate::widgets::form::tokens::Look;
use crate::widgets::icons;
use crate::widgets::nav::NavLink;
use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::widgets::selection::FolderRow;
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::{PRODUCT, caption, caption_h, folder_row, head_h, logo_rect, shell, text};
use crate::widgets::header::Logo;

mod semantics;

/// A row of the navigation: a dropdown and its links, or a link.
#[derive(Clone, Debug)]
pub enum HomeRow {
    Menu(Collapsible, DisclosureList),
    Link(NavLink),
}

/// Where the parts sit.
#[derive(Clone, Debug)]
pub struct HomeLayout {
    pub rows: Vec<Rect>,
    pub caption_y: f32,
    pub scenes: Rect,
    pub new: Rect,
    pub field: Option<Rect>,
    pub folders: Vec<Rect>,
}

#[derive(Clone, Debug)]
pub struct HomeSidebar {
    pub rows: Vec<HomeRow>,
    pub logo: Logo,
    pub caption: String,
    pub scenes: NavLink,
    pub folders: Vec<String>,
    /// The new folder's field is open, with what was typed in it.
    pub naming: bool,
    /// The new folder's name, the form's bare field (front A).
    pub name: TextField,
    hovered: Option<FolderRow>,
    focused: Option<FolderRow>,
    /// The "My scenes" caption, its link and the folders under the
    /// navigation; off, the sidebar is the navigation alone.
    library: bool,
    /// How far the body has scrolled, as asked; the layout holds it to
    /// what the body overflows by.
    scroll: f32,
    /// A strip under the logo the app fills, this high (the gallery's
    /// search); 0, none.
    slot: f32,
    sem: Option<SemId>,
}

fn row_h(row: &HomeRow, w: f32, theme: &Theme) -> f32 {
    match row {
        HomeRow::Menu(m, links) => m.height(w, links.height(w, theme), theme),
        HomeRow::Link(_) => theme.navigation.link.height,
    }
}

impl HomeSidebar {
    pub fn new(rows: Vec<HomeRow>, caption: impl Into<String>, scenes: NavLink) -> Self {
        let rows = rows
            .into_iter()
            .map(|row| match row {
                HomeRow::Menu(m, links) => HomeRow::Menu(m.truncate(true), links.truncate(true)),
                link => link,
            })
            .collect();
        Self {
            rows,
            logo: Logo::new(PRODUCT),
            caption: caption.into(),
            scenes,
            folders: Vec::new(),
            naming: false,
            name: TextField::new("Untitled Folder").look(Look::Bare),
            hovered: None,
            focused: None,
            library: true,
            scroll: 0.0,
            slot: 0.0,
            sem: None,
        }
    }

    /// A strip `h` high under the logo, the body's width, that the app
    /// fills ([`HomeSidebar::slot_rect`]): it stays with the logo while
    /// the body scrolls, a section's gap above the body. 0 takes it away.
    pub fn set_slot(&mut self, h: f32) {
        self.slot = h.max(0.0);
    }

    /// The logo's strip at the top of `bounds`.
    pub fn head_rect(bounds: Rect, theme: &Theme) -> Rect {
        Rect::new(
            bounds.x,
            bounds.y,
            bounds.w,
            head_h(theme).min(bounds.h.max(0.0)),
        )
    }

    /// The strip under the logo in `bounds`, as wide as the body's rows;
    /// `None` without one.
    pub fn slot_rect(&self, bounds: Rect, theme: &Theme) -> Option<Rect> {
        let t = &theme.navigation.sidebars;
        (self.slot > 0.0).then(|| {
            Rect::new(
                bounds.x + t.body_pad,
                bounds.y + head_h(theme),
                bounds.w - t.body_pad * 2.0,
                self.slot,
            )
        })
    }

    /// What stands above the body: the logo's strip, and the slot under
    /// it with a section's gap.
    fn top(&self, theme: &Theme) -> f32 {
        let gap = theme.navigation.sidebars.group_gap;
        head_h(theme)
            + if self.slot > 0.0 {
                self.slot + gap
            } else {
                0.0
            }
    }

    /// With the "My scenes" caption, its link and the folders (the kit's
    /// home, the default), or the navigation alone.
    pub fn library(mut self, on: bool) -> Self {
        self.library = on;
        self
    }

    pub fn folders(mut self, folders: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.folders = folders.into_iter().map(Into::into).collect();
        self
    }

    /// Shown with the new folder's field open, as a state.
    pub fn naming(mut self, on: bool) -> Self {
        self.naming = on;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(l) = id.child("logo") {
            self.logo = self.logo.clone().sem(l);
        }
        let rows = id.child("row").ok();
        for (r, row) in self.rows.iter_mut().enumerate() {
            let Some(at) = rows.as_ref().map(|f| f.key(r.to_string())) else {
                continue;
            };
            match row {
                HomeRow::Menu(m, links) => {
                    if let Ok(l) = at.child("links") {
                        *links = links.clone().sem(l);
                    }
                    *m = m.clone().sem(at);
                }
                HomeRow::Link(link) => *link = link.clone().sem(at),
            }
        }
        if let Ok(n) = id.child("name") {
            self.name = self.name.clone().sem(n);
        }
        if let Ok(s) = id.child("scenes") {
            self.scenes = self.scenes.clone().sem(s);
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// `<id>.logo`, the link home.
    pub fn logo_id(&self) -> Option<&SemId> {
        self.logo.sem_id()
    }

    /// `<id>.new` or `<id>.folder[<i>]`.
    pub fn folder_id(&self, row: FolderRow) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        match row {
            FolderRow::New => id.child("new").ok(),
            FolderRow::Folder(i) => id.child("folder").ok().map(|f| f.key(i.to_string())),
        }
    }

    /// The new folder and the folders: none without the library.
    fn folder_rows(&self) -> impl Iterator<Item = FolderRow> {
        let n = if self.library { self.folders.len() } else { 0 };
        let new = self.library.then_some(FolderRow::New);
        new.into_iter().chain((0..n).map(FolderRow::Folder))
    }

    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.navigation.sidebars;
        let bounds = Rect::new(0.0, 0.0, t.width, 0.0);
        let end = self.end(&self.place(bounds, 0.0, theme), bounds, theme);
        (t.width, end.max(t.min_h))
    }

    /// Where the body's last part ends, its padding below it.
    fn end(&self, l: &HomeLayout, bounds: Rect, theme: &Theme) -> f32 {
        let t = &theme.navigation.sidebars;
        let last = if self.library {
            l.folders.last().copied().unwrap_or(l.new).bottom()
        } else {
            l.rows
                .last()
                .map_or(bounds.y + self.top(theme), |r| r.bottom())
        };
        last + t.body_pad
    }

    /// The body: everything under the logo (and the slot), what
    /// scrolls.
    pub fn body_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let head = self.top(theme).min(bounds.h.max(0.0));
        Rect::new(bounds.x, bounds.y + head, bounds.w, bounds.h - head)
    }

    /// What the body overflows `bounds` by: how far it can scroll.
    pub fn max_scroll(&self, bounds: Rect, theme: &Theme) -> f32 {
        let end = self.end(&self.place(bounds, 0.0, theme), bounds, theme);
        (end - bounds.bottom()).max(0.0)
    }

    /// How far the body has scrolled in `bounds`.
    pub fn scroll_offset(&self, bounds: Rect, theme: &Theme) -> f32 {
        self.scroll.clamp(0.0, self.max_scroll(bounds, theme))
    }

    /// Scrolls the body by `delta` (down when positive); `true` when it
    /// moved.
    pub fn scroll_by(&mut self, delta: f32, bounds: Rect, theme: &Theme) -> bool {
        let max = self.max_scroll(bounds, theme);
        let before = self.scroll.clamp(0.0, max);
        self.scroll = (before + delta).clamp(0.0, max);
        self.scroll != before
    }

    /// The parts in `bounds`, the body scrolled as far as it can be.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> HomeLayout {
        self.place(bounds, self.scroll_offset(bounds, theme), theme)
    }

    /// The parts with the body moved up by `shift`.
    fn place(&self, bounds: Rect, shift: f32, theme: &Theme) -> HomeLayout {
        let t = &theme.navigation.sidebars;
        let link = &theme.navigation.link;
        let folder_h = theme.selection.folders.row_h;
        let (x, w) = (bounds.x + t.body_pad, bounds.w - t.body_pad * 2.0);
        let mut y = bounds.y + self.top(theme) - shift;
        let mut rows = Vec::new();
        for row in &self.rows {
            let h = row_h(row, w, theme);
            rows.push(Rect::new(x, y, w, h));
            y += h + link.link_gap;
        }
        y += t.group_gap - link.link_gap;
        // The home's caption has no `mb-0.5`.
        let caption_y = y;
        y += caption_h(theme) - t.caption_gap;
        let scenes = Rect::new(x, y, w, link.height);
        y = scenes.bottom() + t.row_gap;
        let new = Rect::new(x, y, w, folder_h);
        y = new.bottom() + t.row_gap;
        let field = self.naming.then(|| {
            let r = Rect::new(x, y, w, t.field_h);
            y = r.bottom() + t.row_gap;
            r
        });
        let folders = (0..self.folders.len())
            .map(|i| Rect::new(x, y + (folder_h + t.row_gap) * i as f32, w, folder_h))
            .collect();
        HomeLayout {
            rows,
            caption_y,
            scenes,
            new,
            field,
            folders,
        }
    }

    fn folder_rect(row: FolderRow, l: &HomeLayout) -> Rect {
        match row {
            FolderRow::New => l.new,
            FolderRow::Folder(i) => l.folders[i],
        }
    }

    /// Keyboard focus on the part `id` names, the logo among them (none
    /// when `None` or not one of them); `true` when it names one.
    pub fn focus_id(&mut self, id: Option<&SemId>) -> bool {
        let mut found = false;
        for row in &mut self.rows {
            match row {
                HomeRow::Menu(m, links) => {
                    let on = id.is_some() && m.sem_id() == id;
                    m.set_focused(on);
                    let at = id.and_then(|i| links.position(i));
                    links.set_focused(at);
                    found |= on || at.is_some();
                }
                HomeRow::Link(link) => {
                    let on = id.is_some() && link.sem_id() == id;
                    link.set_focused(on);
                    found |= on;
                }
            }
        }
        let logo = id.is_some() && self.logo.sem_id() == id;
        self.logo.set_focused(logo);
        let on = id.is_some() && self.scenes.sem_id() == id;
        self.scenes.set_focused(on);
        self.focused = id.and_then(|i| {
            self.folder_rows()
                .find(|r| self.folder_id(*r).as_ref() == Some(i))
        });
        found || logo || on || self.focused.is_some()
    }

    /// What a press does to the part `id` names: a dropdown opens or
    /// closes, a link becomes current, the new folder opens its field.
    pub fn activate(&mut self, id: &SemId) -> bool {
        for row in &mut self.rows {
            match row {
                HomeRow::Menu(m, links) => {
                    if m.sem_id() == Some(id) {
                        return m.toggle();
                    }
                    if let Some(i) = links.position(id) {
                        return links.activate(i);
                    }
                }
                HomeRow::Link(link) if link.sem_id() == Some(id) => {
                    link.active = true;
                    return true;
                }
                HomeRow::Link(_) => {}
            }
        }
        if self.scenes.sem_id() == Some(id) {
            self.scenes.active = true;
            return true;
        }
        if self.folder_id(FolderRow::New).as_ref() == Some(id) {
            self.naming = !self.naming;
            return true;
        }
        self.folder_rows()
            .any(|r| self.folder_id(r).as_ref() == Some(id))
    }

    /// Where the name is typed: the field after its folder mark.
    pub fn name_rect(f: Rect, theme: &Theme) -> Rect {
        let t = &theme.navigation.sidebars;
        let x = f.x + t.field_text_x;
        Rect::new(x, f.y, (f.right() - x - t.field_icon_x).max(0.0), f.h)
    }

    /// The text fields shown now: the name, while it is being given.
    pub fn text_fields(&mut self) -> Vec<&mut TextField> {
        if self.naming {
            vec![&mut self.name]
        } else {
            Vec::new()
        }
    }

    /// Their rects, in the same order.
    pub fn field_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let l = self.layout(bounds, theme);
        l.field
            .map(|f| Self::name_rect(f, theme))
            .into_iter()
            .collect()
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        let mut wake = Wake::idle();
        for row in &mut self.rows {
            if let HomeRow::Menu(m, _) = row {
                wake = wake.merge(m.wake(dt));
            }
        }
        wake
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if let WidgetEvent::Scroll { x, y, delta } = *event {
            if !bounds.contains(x, y) {
                return EventResult::IGNORED;
            }
            return EventResult {
                handled: true,
                changed: self.scroll_by(delta, bounds, theme),
                clicked: false,
            };
        }
        let l = self.layout(bounds, theme);
        let pad = theme.navigation.sidebars.head_pad;
        let at = logo_rect(&self.logo, bounds, pad, theme);
        let mut out = self.logo.handle_event(event, at);
        // the body's parts scrolled under the logo are not there: a
        // pointer off the body is away from them (a release still ends a
        // press)
        let (x, y) = event.pos();
        let event = if self.body_rect(bounds, theme).contains(x, y)
            || matches!(event, WidgetEvent::MouseUp { .. })
        {
            event
        } else {
            &AWAY
        };
        for (row, r) in self.rows.iter_mut().zip(&l.rows) {
            let res = match row {
                HomeRow::Menu(m, links) => menu_event(m, links, event, *r, theme),
                HomeRow::Link(link) => {
                    let res = link.handle_event(event, *r);
                    if res.clicked {
                        link.active = true;
                    }
                    res
                }
            };
            out = out.merge(res);
        }
        if self.library {
            out = out.merge(self.scenes.handle_event(event, l.scenes));
        }
        let (x, y) = event.pos();
        if let Some(f) = l.field {
            let at = Self::name_rect(f, theme);
            let press = matches!(event, WidgetEvent::MouseDown { .. });
            if (press && f.contains(x, y)) || (!press && self.name.is_focused()) {
                out = out.merge(self.name.handle_event(event, at, theme));
            } else if press && self.name.is_focused() {
                self.name.unfocus();
                out = out.merge(EventResult::changed());
            }
        }
        let hit = self
            .folder_rows()
            .find(|r| Self::folder_rect(*r, &l).contains(x, y));
        match event {
            WidgetEvent::MouseMove { .. } if hit != self.hovered => {
                self.hovered = hit;
                out = out.merge(EventResult::changed());
            }
            WidgetEvent::MouseDown { .. } if hit == Some(FolderRow::New) => {
                self.naming = !self.naming;
                out = out.merge(EventResult::clicked());
            }
            WidgetEvent::MouseDown { .. } if hit.is_some() => {
                out = out.merge(EventResult::clicked());
            }
            _ => {}
        }
        out
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.navigation.sidebars;
        shell(c, bounds, false, theme);
        // bound before its parts: a hit on one of them is the part's
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let at = logo_rect(&self.logo, bounds, t.head_pad, theme);
        self.logo.render(c, at, theme);
        let l = self.layout(bounds, theme);
        let body = self.body_rect(bounds, theme);
        c.push_clip(body.x, body.y, body.w, body.h);
        self.render_body(c, &l, theme);
        c.pop_clip();
    }

    /// The rows, the library and the new folder's field, as laid out.
    fn render_body(&self, c: &mut Compositor, l: &HomeLayout, theme: &Theme) {
        let t = &theme.navigation.sidebars;
        let ft = &theme.selection.folders;
        let recipe = &theme.navigation.link;
        for (row, r) in self.rows.iter().zip(&l.rows) {
            match row {
                HomeRow::Menu(m, links) => {
                    let h = links.height(r.w, theme);
                    m.render(c, *r, h, theme, |c, at| links.render(c, at, theme));
                }
                HomeRow::Link(link) => link.render_with(c, c.target(), *r, theme, recipe),
            }
        }
        if self.library {
            caption(c, &self.caption, l.scenes.x, l.caption_y, l.scenes.w, theme);
            self.scenes
                .render_with(c, c.target(), l.scenes, theme, recipe);
        }
        for row in self.folder_rows() {
            let r = Self::folder_rect(row, l);
            if let Some(id) = self.folder_id(row) {
                c.bind(&id, [r.x, r.y, r.w, r.h]);
            }
            let (label, icon) = match row {
                FolderRow::New => ("New Folder", ("kit-folder-new", ft.mark.0)),
                FolderRow::Folder(i) => (
                    self.folders[i].as_str(),
                    ("kit-folder", ft.hues[i % ft.hues.len()].0),
                ),
            };
            let (hovered, focused) = (self.hovered == Some(row), self.focused == Some(row));
            folder_row(c, r, label, icon, hovered, focused, theme);
        }
        if let Some(f) = l.field {
            c.push(rounded_rect(
                f.x,
                f.y,
                f.w,
                f.h,
                t.field_radius,
                t.field_fill.0,
            ));
            c.push(rounded_rect_stroke(
                f.x,
                f.y,
                f.w,
                f.h,
                t.field_radius,
                t.field_edge.0,
                t.edge_width,
            ));
            let iy = f.y + (f.h - ft.icon) / 2.0;
            if let Some(n) = icons::icon_at(
                "kit-folder",
                ft.icon,
                t.field_mark.0,
                f.x + t.field_icon_x,
                iy,
            ) {
                c.push(n);
            }
            let at = Self::name_rect(f, theme);
            if self.name.is_empty() && !self.name.is_focused() {
                let style = theme.typography.run(ft.text);
                let y = f.y + (f.h - style.line_height) / 2.0;
                let shown = "Untitled Folder";
                c.push(text(
                    shown,
                    &style,
                    at.x,
                    y,
                    t.field_placeholder.0,
                    Some(at.w),
                ));
            } else {
                self.name.render(c, at, theme);
            }
            if self.focused == Some(FolderRow::New) {
                c.push(focus_ring(f, t.field_radius, theme));
            }
        }
    }
}

/// A pointer nowhere: what a part hears when the pointer is away from it.
const AWAY: WidgetEvent = WidgetEvent::MouseMove {
    x: f32::NEG_INFINITY,
    y: f32::NEG_INFINITY,
};

/// A dropdown takes the pointer on its trigger, and its links while it
/// is open; a pointer away from them leaves their hover.
fn menu_event(
    m: &mut Collapsible,
    links: &mut DisclosureList,
    event: &WidgetEvent,
    r: Rect,
    theme: &Theme,
) -> EventResult {
    let links_h = links.height(r.w, theme);
    let shown = m.shown_rect(r, theme);
    let at = m.content_rect(r, links_h, theme);
    let (x, y) = event.pos();
    let reaches =
        m.is_open() && (shown.contains(x, y) || matches!(event, WidgetEvent::MouseUp { .. }));
    let mut res = m.handle_event(event, r, theme);
    if reaches {
        res = res.merge(links.handle_event(event, at, theme));
    } else if matches!(event, WidgetEvent::MouseMove { .. }) {
        res = res.merge(links.handle_event(&AWAY, at, theme));
    }
    res
}
