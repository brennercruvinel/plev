//! Sidebar page: the kit's SidebarPage (`templates/SidebarPage`) on the
//! showcase mold, its two groups as the kit lays them, each sidebar in a
//! gray card (`isGray`), centered: the scene and assets sidebar (its
//! rows and their toggles, or its asset sections), the design and
//! animation sidebar (its groups, selects, switches, tiles and color
//! picker), the home
//! sidebar (its dropdowns,
//! links and folders) and the guideline sidebar (the kit's own pages,
//! this one current).
//!
//! Projected: a group per title, `sidebar_page.group[<g>]`; in it each
//! sidebar a region, `sidebar_page.scene`, `sidebar_page.design`,
//! `sidebar_page.home` and
//! `sidebar_page.guideline`,
//! with the parts each declares. A reader's `activate` (and
//! `toggle_expanded` on a dropdown) runs the part as a press does.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{Collapsible, DisclosureList};
use engine::widgets::nav::{
    DesignSidebar, GuidelineSidebar, HomeRow, HomeSidebar, NavLink, SceneItem, SceneSidebar,
};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect};
use engine::widgets::wake::Wake;

use super::showcase::{self, GroupLayout};
use super::{After, Component, EditKey, child_id, root_id, under};
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;
use accord::Value;
use engine::widgets::form::TextField;
use engine::widgets::form::color_picker::ColorPart;

/// What a card shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shows {
    Scene,
    Design,
    Home,
    Guideline,
}

struct Group {
    title: &'static str,
    cards: Vec<(&'static str, Shows)>,
}

pub struct SidebarPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    home: HomeSidebar,
    guideline: GuidelineSidebar,
    scene: SceneSidebar,
    design: DesignSidebar,
    focus: Option<SemId>,
}

/// HomeSidebar/navigation.ts: Explore open on Designs, Assets with its
/// counter, Likes; My Scenes; the two folders of the home.
fn kit_home() -> HomeSidebar {
    HomeSidebar::new(
        vec![
            HomeRow::Menu(
                Collapsible::new("Explore")
                    .icon("kit-explore")
                    .current(true)
                    .open(true),
                DisclosureList::new(["Designs", "Animations"]).current(0),
            ),
            HomeRow::Menu(
                Collapsible::new("Assets").icon("kit-assets").counter("112"),
                DisclosureList::new(["3D Objects", "Materials"]),
            ),
            HomeRow::Link(NavLink::new("Likes").icon("kit-likes")),
        ],
        "My scenes",
        NavLink::new("My Scenes").icon("kit-scenes"),
    )
    .folders(["Untitled Folder", "3D Icons"])
}

/// SidebarPage/content.ts: the guideline's two groups, each page's icon
/// the kit's own path (`kit-*`); the Sidebar page current,
/// as the kit's `usePathname` makes it on this page.
fn kit_guideline() -> GuidelineSidebar {
    GuidelineSidebar::new()
        .group(
            "Foundations",
            &[
                ("Branding", "kit-branding"),
                ("Colors", "kit-colors"),
                ("Depths", "kit-depths"),
                ("Icons", "kit-icons"),
                ("Typography", "kit-typography"),
            ],
        )
        .group(
            "Components",
            &[
                ("Buttons, tool-tip", "kit-buttons"),
                ("Cards", "kit-cards"),
                ("Comment", "kit-comment"),
                ("Dropdown", "kit-dropdown"),
                ("Inputs", "kit-inputs"),
                ("Menu", "kit-menu"),
                ("Notifications", "kit-notifications"),
                ("Modal, Dialogs", "kit-modal"),
                ("Sidebar", "kit-sidebar"),
                ("Prompt Input", "kit-prompt-input"),
                ("Toolbar", "kit-toolbar"),
                ("Topbar", "kit-topbar"),
            ],
        )
        .current(13)
}

/// SceneAndAssetsSidebar: its head, the scene's eight rows (Object 2
/// selected, each kind's icon the kit's own path), the
/// assets' two sections.
fn kit_scene() -> SceneSidebar {
    let item = SceneItem::new;
    SceneSidebar::new("Robot 2.0", "3D Design Project")
        .items(
            vec![
                item("Camera 1", "kit-camera"),
                item("Dome Light", "kit-dome-light"),
                item("Key Light", "kit-key-light"),
                item("Area Light", "kit-area-light"),
                item("Object 2", "kit-object"),
                item("Background 2", "kit-object"),
                item("Robot", "kit-object"),
                item("Background 1", "kit-object"),
            ],
            Some(4),
        )
        .section("3D Objects", 4, true)
        .section("Materials", 6, false)
}

impl SidebarPage {
    pub fn new() -> Self {
        let root = root_id("sidebar_page");
        let r = root.as_ref();
        let mut home = kit_home();
        if let Some(id) = child_id(r, "home") {
            home = home.sem(id);
        }
        let mut guideline = kit_guideline();
        if let Some(id) = child_id(r, "guideline") {
            guideline = guideline.sem(id);
        }
        let mut scene = kit_scene();
        if let Some(id) = child_id(r, "scene") {
            scene = scene.sem(id);
        }
        let mut design = DesignSidebar::kit();
        if let Some(id) = child_id(r, "design") {
            design = design.sem(id);
        }
        let groups = vec![
            Group {
                title: "Canvas Sidebar",
                cards: vec![
                    ("Scene, Assets", Shows::Scene),
                    ("Design, Animation", Shows::Design),
                ],
            },
            Group {
                title: "Home Sidebar",
                cards: vec![("Home", Shows::Home), ("Guideline", Shows::Guideline)],
            },
        ];
        Self {
            root,
            groups,
            home,
            guideline,
            scene,
            design,
            focus: None,
        }
    }

    fn size(&self, shows: Shows, theme: &Theme) -> (f32, f32) {
        match shows {
            Shows::Home => self.home.size(theme),
            Shows::Guideline => self.guideline.size(theme),
            Shows::Scene => self.scene.size(theme),
            Shows::Design => self.design.size(theme),
        }
    }

    /// Two cards a row (`span={2}`), each as tall as its sidebar asks.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let min_w = theme.navigation.sidebars.width + theme.spacing.md * 2.0;
        let grids = self.groups.iter().map(|g| {
            showcase::Grid::new(
                2,
                min_w,
                g.cards
                    .iter()
                    .map(|(_, s)| showcase::card_height(self.size(*s, theme).1, theme))
                    .collect(),
            )
        });
        showcase::layout_grids(grids, content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// Each card's rect and its sidebar's, centered in the card's body.
    fn placed(&self, content: Rect, theme: &Theme) -> Vec<(&'static str, Shows, Rect, Rect)> {
        let layout = self.layout(content, theme);
        let mut out = Vec::new();
        for (group, at) in self.groups.iter().zip(&layout) {
            for ((title, shows), card) in group.cards.iter().zip(&at.cards) {
                let body = showcase::body(*card, theme);
                let at = showcase::centered(self.size(*shows, theme), body);
                let at = Rect::new(at.x.max(body.x), at.y.max(body.y), at.w, at.h);
                out.push((*title, *shows, *card, at));
            }
        }
        out
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        let design = self.design.wake(dt);
        self.home.wake(dt).merge(self.scene.wake(dt)).merge(design)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut out = EventResult::IGNORED;
        for (_, shows, _, at) in self.placed(content, theme) {
            let r = match shows {
                Shows::Home => self.home.handle_event(event, at, theme),
                Shows::Guideline => self.guideline.handle_event(event, at, theme),
                Shows::Scene => self.scene.handle_event(event, at, theme),
                Shows::Design => self.design.handle_event(event, at, theme),
            };
            out = out.merge(r);
        }
        out
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        for (title, shows, card, at) in self.placed(content, theme) {
            c.push(rounded_rect(
                card.x,
                card.y,
                card.w,
                card.h,
                theme.shape.card,
                theme.colors.bg_panel.0,
            ));
            showcase::card_frame(c, card, theme);
            c.push_clip(card.x, card.y, card.w, card.h);
            match shows {
                Shows::Home => self.home.render(c, at, theme),
                Shows::Guideline => self.guideline.render(c, at, theme),
                Shows::Scene => self.scene.render(c, at, theme),
                Shows::Design => self.design.render(c, at, theme),
            }
            c.pop_clip();
            let inset = showcase::inset(theme);
            showcase::chip(c, card.x + inset, card.y + inset, title, theme);
        }
        // An open select's list, over every card.
        for (_, shows, _, at) in self.placed(content, theme) {
            if shows == Shows::Design {
                self.design.render_overlay(c, at, theme);
            }
        }
    }

    /// The text fields shown now, in every sidebar.
    fn fields(&mut self) -> Vec<&mut TextField> {
        let mut out = self.home.text_fields();
        out.extend(self.scene.text_fields());
        out.extend(self.design.text_fields());
        out
    }

    fn focused_field(&mut self) -> Option<&mut TextField> {
        self.fields().into_iter().find(|f| f.is_focused())
    }

    fn set_focus(&mut self, id: Option<&SemId>) {
        for f in self.fields() {
            if id.is_some() && f.sem_id() == id {
                f.focus();
            } else {
                f.unfocus();
            }
        }
        let ring = |own: Option<&SemId>| id.is_some() && own == id;
        let on = [
            ring(self.home.logo_id()),
            ring(self.guideline.logo_id()),
            ring(self.scene.logo_id()),
        ];
        self.home.logo.set_focused(on[0]);
        self.guideline.logo.set_focused(on[1]);
        self.scene.logo.set_focused(on[2]);
        self.home.focus_id(id);
        let at = id.and_then(|i| self.guideline.position(i));
        self.guideline.set_focused(at);
        let part = id.and_then(|i| self.scene.part_at_id(i));
        self.scene.set_focused(part);
        self.design.focus_id(id);
        self.focus = id.cloned();
    }
}

impl Component for SidebarPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = child_id(Some(root), "group").map(|f| f.key(g.to_string())) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for (title, shows) in &group.cards {
                match shows {
                    Shows::Home => {
                        if let Some(id) = self.home.sem_id() {
                            self.home.project(b, id, title, Some(&at));
                        }
                    }
                    Shows::Guideline => {
                        if let Some(id) = self.guideline.sem_id() {
                            self.guideline.project(b, id, title, Some(&at));
                        }
                    }
                    Shows::Scene => {
                        if let Some(id) = self.scene.sem_id() {
                            self.scene.project(b, id, title, Some(&at));
                        }
                    }
                    Shows::Design => {
                        if let Some(id) = self.design.sem_id() {
                            self.design.project(b, id, title, Some(&at));
                        }
                    }
                }
            }
        }
        if let Some(id) = &self.focus {
            b.focus(id);
        }
    }

    /// `activate` (`toggle_expanded` on a dropdown): a dropdown opens or
    /// closes, a link becomes current, the new folder opens its field.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let miss = || format!("the sidebar page has no {} on {}", req.action, req.target);
        if !matches!(
            req.action.as_str(),
            "activate" | "toggle_expanded" | "toggle" | "select" | "set_value"
        ) {
            return Err(miss());
        }
        if self.home.activate(&req.target) {
            return Ok(After::Nothing);
        }
        if req.action.as_str() == "set_value" {
            let Some(Value::Text(text)) = &req.param else {
                return Err(miss());
            };
            if let Some(p) = self.design.picker_mut() {
                let target = Some(&req.target);
                if p.text_id(ColorPart::Hex) == target {
                    return if p.set_hex(text) {
                        Ok(After::Nothing)
                    } else {
                        Err(miss())
                    };
                }
                if p.text_id(ColorPart::Alpha) == target {
                    return if p.set_opacity(text) {
                        Ok(After::Nothing)
                    } else {
                        Err(miss())
                    };
                }
            }
            let field = self
                .fields()
                .into_iter()
                .find(|f| f.sem_id() == Some(&req.target));
            return match field {
                Some(f) => {
                    f.set_text(text);
                    Ok(After::Nothing)
                }
                None => Err(miss()),
            };
        }
        // A logo is the link home: nothing to follow in a showcase.
        let logos = [
            self.home.logo_id(),
            self.guideline.logo_id(),
            self.scene.logo_id(),
        ];
        if logos.contains(&Some(&req.target)) {
            return Ok(After::Nothing);
        }
        if self.design.activate(&req.target) {
            return Ok(After::Nothing);
        }
        if let Some(part) = self.scene.part_at_id(&req.target)
            && self.scene.activate(part)
        {
            return Ok(After::Nothing);
        }
        if let Some(i) = self.scene.tabs.sem_id().and_then(|t| {
            let rest = req.target.as_str().strip_prefix(t.as_str())?;
            rest.strip_prefix('[')?
                .strip_suffix(']')?
                .parse::<usize>()
                .ok()
        }) {
            self.scene.tabs.active = i;
            return Ok(After::Nothing);
        }
        for s in &mut self.scene.sections {
            if s.head.sem_id() == Some(&req.target) {
                s.head.toggle();
                return Ok(After::Nothing);
            }
        }
        if let Some(n) = self.guideline.position(&req.target) {
            self.guideline.set_current(n);
            return Ok(After::Nothing);
        }
        Err(miss())
    }

    /// A reader's focus on a part of a sidebar: its ring, and projected
    /// back as the focus.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(Some(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

/// The sidebars' fields take the keys: the new folder's name, the
/// scene's search, the artboard's width and height.
impl TextHost for SidebarPage {
    fn handle_text(&mut self, s: &str) -> bool {
        self.focused_field().is_some_and(|f| f.insert(s))
    }

    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        key != EditKey::Tab && self.focused_field().is_some_and(|f| f.edit(key.into()))
    }

    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        if let ImeEvent::Disabled = ime {
            // every field ends its composition: no short circuit
            let mut ended = false;
            for f in self.fields() {
                ended |= f.end_compose();
            }
            return ended;
        }
        let Some(field) = self.focused_field() else {
            return false;
        };
        match ime {
            ImeEvent::Preedit(text, cursor) => field.compose("", text, cursor),
            ImeEvent::Commit(text) => field.compose(text, "", None),
            ImeEvent::Disabled => false,
        }
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        for (_, shows, _, at) in self.placed(content, theme) {
            let (rects, focused): (Vec<Rect>, Vec<bool>) = match shows {
                Shows::Home => (
                    self.home.field_rects(at, theme),
                    vec![self.home.naming && self.home.name.is_focused()],
                ),
                Shows::Scene => (
                    self.scene.field_rects(at, theme),
                    vec![self.scene.search.input.is_focused()],
                ),
                Shows::Design => (
                    self.design.field_rects(at, theme),
                    self.design.focused_fields(),
                ),
                Shows::Guideline => continue,
            };
            if let Some(i) = focused.iter().position(|f| *f) {
                return rects.get(i).copied();
            }
        }
        None
    }

    fn right_click(&mut self, _x: f32, _y: f32, _content: Rect, _theme: &Theme) -> bool {
        false
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.focused_field() {
            Some(f) => Keys::Field(f),
            None => Keys::Nobody,
        }
    }

    /// Enter keeps the color typed in the picker.
    fn handle_enter(&mut self) -> bool {
        self.design
            .picker_mut()
            .is_some_and(|p| p.focused().is_some() && p.enter())
    }

    /// Escape puts back the picker's color, or leaves a field.
    fn handle_escape(&mut self) -> bool {
        if let Some(p) = self.design.picker_mut()
            && p.focused().is_some()
        {
            return p.escape();
        }
        let Some(field) = self.focused_field() else {
            return false;
        };
        field.unfocus();
        true
    }
}

#[cfg(test)]
mod tests;
