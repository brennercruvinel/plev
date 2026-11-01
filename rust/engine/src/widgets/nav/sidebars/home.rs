//! The kit's HomeSidebar (`components/HomeSidebar`): the logo, the
//! navigation (a dropdown per section with its links, the collapsible of
//! front D, and plain links), the "My scenes" caption and link, then the
//! folders: the "New Folder" row, which opens the field that names a new
//! one (`components/NewFolder`), and a row per folder
//! (`components/Folder`), drawn as the folder picker of front E draws
//! its rows.
//!
//! With a [`SemId`] the rows are `<id>.row[<r>]` (a dropdown's links
//! `<id>.row[<r>].links[<i>]`), the scenes link `<id>.scenes`, the new
//! folder `<id>.new` and the folders `<id>.folder[<i>]`.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId};
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

use super::{caption, caption_h, folder_row, head_h, logo, shell, text};

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
    pub caption: String,
    pub scenes: NavLink,
    pub folders: Vec<String>,
    /// The new folder's field is open, with what was typed in it.
    pub naming: bool,
    /// The new folder's name, the form's bare field (front A).
    pub name: TextField,
    hovered: Option<FolderRow>,
    focused: Option<FolderRow>,
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
        Self {
            rows,
            caption: caption.into(),
            scenes,
            folders: Vec::new(),
            naming: false,
            name: TextField::new("Untitled Folder").look(Look::Bare),
            hovered: None,
            focused: None,
            sem: None,
        }
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

    /// `<id>.new` or `<id>.folder[<i>]`.
    pub fn folder_id(&self, row: FolderRow) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        match row {
            FolderRow::New => id.child("new").ok(),
            FolderRow::Folder(i) => id.child("folder").ok().map(|f| f.key(i.to_string())),
        }
    }

    fn folder_rows(&self) -> impl Iterator<Item = FolderRow> {
        std::iter::once(FolderRow::New).chain((0..self.folders.len()).map(FolderRow::Folder))
    }

    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.navigation.sidebars;
        let l = self.layout(Rect::new(0.0, 0.0, t.width, 0.0), theme);
        let end = l.folders.last().copied().unwrap_or(l.new);
        (t.width, (end.bottom() + t.body_pad).max(t.min_h))
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> HomeLayout {
        let t = &theme.navigation.sidebars;
        let link = &theme.navigation.link;
        let folder_h = theme.selection.folders.row_h;
        let (x, w) = (bounds.x + t.body_pad, bounds.w - t.body_pad * 2.0);
        let mut y = bounds.y + head_h(theme);
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

    /// Keyboard focus on the part `id` names (none when `None` or not
    /// one of them); `true` when it names one.
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
        let on = id.is_some() && self.scenes.sem_id() == id;
        self.scenes.set_focused(on);
        self.focused = id.and_then(|i| {
            self.folder_rows()
                .find(|r| self.folder_id(*r).as_ref() == Some(i))
        });
        found || on || self.focused.is_some()
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
        let l = self.layout(bounds, theme);
        let mut out = EventResult::IGNORED;
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
        out = out.merge(self.scenes.handle_event(event, l.scenes));
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
        let ft = &theme.selection.folders;
        shell(c, bounds, false, theme);
        logo(c, bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let l = self.layout(bounds, theme);
        let recipe = &theme.navigation.link;
        for (row, r) in self.rows.iter().zip(&l.rows) {
            match row {
                HomeRow::Menu(m, links) => {
                    let h = links.height(r.w, theme);
                    m.render(c, *r, h, theme, |c, at| links.render(c, at, theme));
                }
                HomeRow::Link(link) => link.render_with(c, LayerId::DEFAULT, *r, theme, recipe),
            }
        }
        caption(c, &self.caption, l.scenes.x, l.caption_y, l.scenes.w, theme);
        self.scenes
            .render_with(c, LayerId::DEFAULT, l.scenes, theme, recipe);
        for row in self.folder_rows() {
            let r = Self::folder_rect(row, &l);
            if let Some(id) = self.folder_id(row) {
                c.bind(&id, [r.x, r.y, r.w, r.h]);
            }
            let (label, icon) = match row {
                FolderRow::New => ("New Folder", ("folder-plus", ft.mark.0)),
                FolderRow::Folder(i) => (
                    self.folders[i].as_str(),
                    ("folder", ft.hues[i % ft.hues.len()].0),
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
            if let Some(n) =
                icons::icon_at("folder", ft.icon, t.field_mark.0, f.x + t.field_icon_x, iy)
            {
                c.push(n);
            }
            let at = Self::name_rect(f, theme);
            if self.name.is_empty() && !self.name.is_focused() {
                let style = theme
                    .typography
                    .exact(ft.font_size, ft.line_height, ft.font_weight);
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

    /// A `Role::Region` named `label`: the dropdowns (their links while
    /// open), the links, the scenes link, the new folder (`expanded`
    /// while its field is open) and the folders, each a button.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::Region, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        for row in &self.rows {
            match row {
                HomeRow::Menu(m, links) => {
                    let Some(at) = m.sem_id() else { continue };
                    m.project(b, at, Some(id));
                    let open = m.project_panel(b, at, Some(id)).is_some();
                    if open && let Ok(panel) = at.child("panel") {
                        links.project(b, Some(&panel));
                    }
                }
                HomeRow::Link(link) => {
                    if let Some(at) = link.sem_id() {
                        link.project(b, at, Some(id));
                    }
                }
            }
        }
        if let Some(at) = self.scenes.sem_id() {
            self.scenes
                .project(b, at, Some(id))
                .description(self.caption.as_str());
        }
        for row in self.folder_rows() {
            let Some(at) = self.folder_id(row) else {
                continue;
            };
            let name = match row {
                FolderRow::New => "New Folder",
                FolderRow::Folder(i) => self.folders[i].as_str(),
            };
            let mut node = b.node(&at, Role::Button, name);
            node.parent(id);
            if row == FolderRow::New {
                node.state("expanded", self.naming);
            }
            node.action("activate");
        }
        if self.naming
            && let Some(at) = self.name.sem_id()
        {
            self.name.project(b, at, "Folder name", Some(id));
        }
    }
}

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
        let away = WidgetEvent::MouseMove {
            x: f32::NEG_INFINITY,
            y: f32::NEG_INFINITY,
        };
        res = res.merge(links.handle_event(&away, at, theme));
    }
    res
}
