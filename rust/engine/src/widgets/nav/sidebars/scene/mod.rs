//! The kit's SceneAndAssetsSidebar (`components/SceneAndAssetsSidebar`):
//! a rounded canvas panel. The head (the logo's mark, the panel button,
//! the project's title with its chevron, its kind), the Scene and Assets
//! tabs, then the scene's rows or the asset sections, and the search at
//! the foot (the Search of front A, `InputGroup`).
//!
//! A scene row (`Scene/Item`) is its kind's icon on a tile and its name;
//! the selected row is filled and its tile raised. Hovering a row shows
//! its three toggles (lock, hide, highlight); a toggle that is on stays
//! shown. An asset section (`Assets/Accordion`) is the collapsible of
//! front D in its accordion look, over a grid of tiles whose pictures
//! are their stand-in gradient.
//!
//! With a [`SemId`]: `<id>.panel`, `<id>.project`, the tabs `<id>.tabs`,
//! the search `<id>.search`, the rows `<id>.item[<i>]` and their toggles `<id>.item[<i>].lock`,
//! `.hide`, `.mark`, the sections `<id>.section[<s>]`.

use accord::{Role, SemId, SnapshotBuilder};

use crate::styling::theme::Theme;
use crate::widgets::content::{Collapsible, DisclosureLook};
use crate::widgets::form::input_group::{Addon, InputGroup};
use crate::widgets::header::{Logo, LogoKind};
use crate::widgets::nav::Tabs;
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

mod paint;

use super::{PRODUCT, logo_rect};

/// The scene's tab or the assets'.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneTab {
    #[default]
    Scene,
    Assets,
}

/// The toggles of a row, in the kit's order: (name, icon off, icon on).
const TOGGLES: [(&str, &str, &str); 3] = [
    ("lock", "lock-open", "lock"),
    ("hide", "eye", "eye-off"),
    ("mark", "sparkles", "sparkles"),
];

/// A row of the scene: its kind's icon, its name, its toggles.
#[derive(Clone, Debug)]
pub struct SceneItem {
    pub title: String,
    pub icon: &'static str,
    pub on: [bool; 3],
}

impl SceneItem {
    pub fn new(title: impl Into<String>, icon: &'static str) -> Self {
        Self {
            title: title.into(),
            icon,
            on: [false; 3],
        }
    }
}

/// An asset section: its collapsible, how many tiles, large pictures.
#[derive(Clone, Debug)]
pub struct Section {
    pub head: Collapsible,
    pub tiles: usize,
    pub large: bool,
}

/// What the pointer or the focus is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenePart {
    Panel,
    Project,
    Item(usize),
    Toggle(usize, usize),
}

/// Where the parts sit.
#[derive(Clone, Debug)]
pub struct SceneLayout {
    pub panel: Rect,
    pub project: Rect,
    pub title_y: f32,
    pub subtitle_y: f32,
    pub head_rule: f32,
    pub tabs: Rect,
    pub tabs_rule: f32,
    pub body: Rect,
    pub items: Vec<Rect>,
    pub sections: Vec<Rect>,
    pub search: Rect,
    pub search_rule: f32,
}

#[derive(Clone, Debug)]
pub struct SceneSidebar {
    pub title: String,
    /// The logo's symbol (`header::Logo`, front F).
    pub logo: Logo,
    pub subtitle: String,
    pub tabs: Tabs,
    pub items: Vec<SceneItem>,
    pub selected: Option<usize>,
    pub sections: Vec<Section>,
    pub search: InputGroup,
    hovered: Option<ScenePart>,
    focused: Option<ScenePart>,
    sem: Option<SemId>,
}

impl SceneSidebar {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            logo: Logo::new(PRODUCT).kind(LogoKind::Symbol),
            subtitle: subtitle.into(),
            tabs: Tabs::new(["Scene", "Assets"]),
            items: Vec::new(),
            selected: None,
            sections: Vec::new(),
            search: InputGroup::new("Search")
                .start(Addon::Icon("search"))
                .end(Addon::Hint("⌘ K".into())),
            hovered: None,
            focused: None,
            sem: None,
        }
    }

    pub fn items(mut self, items: Vec<SceneItem>, selected: Option<usize>) -> Self {
        self.items = items;
        self.selected = selected;
        self
    }

    /// An asset section of `tiles` tiles, open as the kit's start.
    pub fn section(mut self, title: impl Into<String>, tiles: usize, large: bool) -> Self {
        let first = self.sections.is_empty();
        let head = Collapsible::new(title)
            .look(DisclosureLook::Accordion)
            .first(first)
            .open(true);
        self.sections.push(Section { head, tiles, large });
        self
    }

    pub fn tab(mut self, tab: SceneTab) -> Self {
        self.tabs.active = tab as usize;
        self
    }

    pub fn current_tab(&self) -> SceneTab {
        if self.tabs.active == 1 {
            SceneTab::Assets
        } else {
            SceneTab::Scene
        }
    }

    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(l) = id.child("logo") {
            self.logo = self.logo.clone().sem(l);
        }
        if let Ok(f) = id.child("search") {
            self.search = self.search.clone().sem(f);
        }
        if let Ok(t) = id.child("tabs") {
            self.tabs = self.tabs.clone().sem(t);
        }
        if let Ok(f) = id.child("section") {
            for (s, section) in self.sections.iter_mut().enumerate() {
                section.head = section.head.clone().sem(f.key(s.to_string()));
            }
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

    /// The id of a part.
    pub fn part_id(&self, part: ScenePart) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        let item = |i: usize| id.child("item").ok().map(|f| f.key(i.to_string()));
        match part {
            ScenePart::Panel => id.child("panel").ok(),
            ScenePart::Project => id.child("project").ok(),
            ScenePart::Item(i) => item(i),
            ScenePart::Toggle(i, t) => item(i)?.child(TOGGLES[t].0).ok(),
        }
    }

    fn parts(&self) -> impl Iterator<Item = ScenePart> + '_ {
        let items = (0..self.items.len()).flat_map(|i| {
            std::iter::once(ScenePart::Item(i)).chain((0..3).map(move |t| ScenePart::Toggle(i, t)))
        });
        [ScenePart::Panel, ScenePart::Project]
            .into_iter()
            .chain(items)
    }

    /// The part `id` names.
    pub fn part_at_id(&self, id: &SemId) -> Option<ScenePart> {
        self.parts().find(|p| self.part_id(*p).as_ref() == Some(id))
    }

    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.navigation.sidebars;
        let l = self.layout(Rect::new(0.0, 0.0, t.width, 0.0), theme);
        let content = match self.current_tab() {
            SceneTab::Scene => l
                .items
                .last()
                .map_or(l.body.y, |r| r.bottom() + t.canvas.list_pad),
            SceneTab::Assets => l.sections.last().map_or(l.body.y, |r| r.bottom()),
        };
        let foot = t.edge_width + t.canvas.search_pad * 2.0 + self.search.height(theme);
        (t.width, (content + foot).max(t.min_h))
    }

    /// The grid's height in a section `w` wide: the accordion's content
    /// width (its `p-4 pt-0` is the collapsible's), two square tiles a row.
    fn grid_h(&self, s: &Section, w: f32, theme: &Theme) -> f32 {
        let c = &theme.navigation.sidebars.canvas;
        let inner = s
            .head
            .content_rect(Rect::new(0.0, 0.0, w, 0.0), 0.0, theme)
            .w;
        let cell = (inner - c.grid_gap) / 2.0;
        let rows = s.tiles.div_ceil(2) as f32;
        rows * cell + (rows - 1.0).max(0.0) * c.grid_gap
    }

    /// The tiles of a section's grid, in its content rect.
    pub fn cells(&self, s: &Section, at: Rect, theme: &Theme) -> Vec<Rect> {
        let c = &theme.navigation.sidebars.canvas;
        let cell = (at.w - c.grid_gap) / 2.0;
        (0..s.tiles)
            .map(|i| {
                let (col, row) = ((i % 2) as f32, (i / 2) as f32);
                let x = at.x + col * (cell + c.grid_gap);
                Rect::new(x, at.y + row * (cell + c.grid_gap), cell, cell)
            })
            .collect()
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> SceneLayout {
        let t = &theme.navigation.sidebars;
        let c = &t.canvas;
        let (x, w) = (bounds.x + c.pad, bounds.w - c.pad * 2.0);
        let top = bounds.y + c.pad;
        let panel = Rect::new(x + w - c.button, top, c.button, c.button);
        let head_row = c.button.max(self.logo.preferred_size(theme).1);
        let title_y = top + head_row + c.head_gap;
        let subtitle_y = title_y + c.title.line_height + c.subtitle_pad;
        let head_rule = subtitle_y + c.subtitle.line_height + c.subtitle_pad + c.head_pad_b;
        let tabs_y = head_rule + t.edge_width + c.band_pad_y;
        let tabs = Rect::new(x, tabs_y, w, Tabs::height(theme));
        let tabs_rule = tabs.bottom() + c.band_pad_y;
        let search_h = self.search.height(theme);
        let foot = bounds.y + bounds.h.max(t.min_h);
        let search = Rect::new(
            bounds.x + c.search_pad,
            foot - c.search_pad - search_h,
            bounds.w - c.search_pad * 2.0,
            search_h,
        );
        let search_rule = search.y - c.search_pad - t.edge_width;
        let body_y = tabs_rule + t.edge_width;
        let body = Rect::new(bounds.x, body_y, bounds.w, (search_rule - body_y).max(0.0));
        let (lx, lw) = (bounds.x + c.list_pad, bounds.w - c.list_pad * 2.0);
        let items = (0..self.items.len())
            .map(|i| {
                Rect::new(
                    lx,
                    body_y + c.list_pad + (c.row_h + c.row_gap) * i as f32,
                    lw,
                    c.row_h,
                )
            })
            .collect();
        let mut y = body_y;
        let sections = self
            .sections
            .iter()
            .map(|s| {
                let h = s
                    .head
                    .height(bounds.w, self.grid_h(s, bounds.w, theme), theme);
                let r = Rect::new(bounds.x, y, bounds.w, h);
                y += h;
                r
            })
            .collect();
        let style = c.title.style(&theme.typography);
        let tw = crate::drawing::text::TextMeasurer::measure_styled(&self.title, &style, None).0;
        let project = Rect::new(
            x,
            title_y,
            tw + c.title_gap + c.chevron_tile,
            c.title.line_height,
        );
        SceneLayout {
            panel,
            project,
            title_y,
            subtitle_y,
            head_rule,
            tabs,
            tabs_rule,
            body,
            items,
            sections,
            search,
            search_rule,
        }
    }

    /// The toggles' boxes in a row, from its end (`right-3`, `gap-3`).
    pub fn toggle_rects(&self, row: Rect, theme: &Theme) -> [Rect; 3] {
        let c = &theme.navigation.sidebars.canvas;
        let y = row.y + (row.h - c.toggle) / 2.0;
        let x =
            |k: f32| row.right() - c.label_gap - c.toggle * (3.0 - k) - c.toggle_gap * (2.0 - k);
        [0.0, 1.0, 2.0].map(|k| Rect::new(x(k), y, c.toggle, c.toggle))
    }

    fn part_at(&self, x: f32, y: f32, l: &SceneLayout, theme: &Theme) -> Option<ScenePart> {
        if l.panel.contains(x, y) {
            return Some(ScenePart::Panel);
        }
        if l.project.contains(x, y) {
            return Some(ScenePart::Project);
        }
        if self.current_tab() != SceneTab::Scene {
            return None;
        }
        let i = l.items.iter().position(|r| r.contains(x, y))?;
        let t = self
            .toggle_rects(l.items[i], theme)
            .iter()
            .position(|r| r.contains(x, y));
        Some(t.map_or(ScenePart::Item(i), |t| ScenePart::Toggle(i, t)))
    }

    /// The text fields: the search's.
    pub fn text_fields(&mut self) -> Vec<&mut crate::widgets::form::TextField> {
        vec![&mut self.search.input]
    }

    /// Their rects, in the same order: where the search's text goes.
    pub fn field_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let l = self.layout(bounds, theme);
        vec![self.search.layout(l.search, theme).input]
    }

    /// Keyboard focus on a part, or none.
    pub fn set_focused(&mut self, part: Option<ScenePart>) {
        self.focused = part;
    }

    /// What a press does to `part`: a row is selected, a toggle flips.
    pub fn activate(&mut self, part: ScenePart) -> bool {
        match part {
            ScenePart::Item(i) => self.selected = Some(i),
            ScenePart::Toggle(i, t) => self.items[i].on[t] = !self.items[i].on[t],
            ScenePart::Panel | ScenePart::Project => return false,
        }
        true
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        let mut wake = Wake::idle();
        for s in &mut self.sections {
            wake = wake.merge(s.head.wake(dt));
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
        let pad = theme.navigation.sidebars.canvas.pad;
        let logo = logo_rect(&self.logo, bounds, pad, theme);
        let mut out = self.logo.handle_event(event, logo);
        out = out.merge(self.tabs.handle_event(event, l.tabs, theme));
        if self.current_tab() == SceneTab::Assets {
            for (s, r) in self.sections.iter_mut().zip(&l.sections) {
                out = out.merge(s.head.handle_event(event, *r, theme));
            }
        }
        out = out.merge(self.search.handle_event(event, l.search, theme));
        let (x, y) = event.pos();
        let hit = self.part_at(x, y, &l, theme);
        match event {
            WidgetEvent::MouseMove { .. } if hit != self.hovered => {
                self.hovered = hit;
                out = out.merge(EventResult::changed());
            }
            WidgetEvent::MouseDown { .. } => {
                if let Some(part) = hit {
                    self.activate(part);
                    out = out.merge(EventResult::clicked());
                }
            }
            _ => {}
        }
        out
    }

    /// A `Role::Region` named `label`: the head's buttons, the tabs, the
    /// rows (selected or not) and their toggles, or the sections.
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
        if let Some(l) = self.logo.sem_id() {
            self.logo.project(b, l, Some(id));
        }
        for (part, name) in [
            (ScenePart::Panel, "Toggle the panel"),
            (ScenePart::Project, self.title.as_str()),
        ] {
            if let Some(at) = self.part_id(part) {
                b.node(&at, Role::Button, name).parent(id);
            }
        }
        if let Some(at) = self.search.sem_id() {
            self.search.project(b, at, "Search", Some(id));
        }
        if let Some(at) = self.tabs.sem_id() {
            self.tabs.project(b, at, "Scene or assets", Some(id));
        }
        match self.current_tab() {
            SceneTab::Scene => {
                for (i, item) in self.items.iter().enumerate() {
                    let Some(at) = self.part_id(ScenePart::Item(i)) else {
                        continue;
                    };
                    let mut node = b.node(&at, Role::Button, item.title.as_str());
                    node.parent(id).state("selected", self.selected == Some(i));
                    node.action("activate");
                    for (t, (name, _, _)) in TOGGLES.iter().enumerate() {
                        let Some(tid) = self.part_id(ScenePart::Toggle(i, t)) else {
                            continue;
                        };
                        let mut node = b.node(&tid, Role::Toggle, *name);
                        node.parent(&at).state("on", item.on[t]);
                        node.action("toggle").reads(format!("{tid}:on"));
                    }
                }
            }
            SceneTab::Assets => {
                for s in &self.sections {
                    if let Some(at) = s.head.sem_id() {
                        s.head.project(b, at, Some(id));
                    }
                }
            }
        }
    }
}
