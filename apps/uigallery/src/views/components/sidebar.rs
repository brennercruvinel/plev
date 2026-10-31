//! Sidebar page: the kit's SidebarPage (`templates/SidebarPage`) on the
//! showcase mold, its two groups as the kit lays them, each sidebar in a
//! gray card (`isGray`), centered: the scene and assets sidebar (its
//! rows and their toggles, or its asset sections), the home sidebar (its dropdowns,
//! links and folders) and the guideline sidebar (the kit's own pages,
//! this one current).
//!
//! Projected: a group per title, `sidebar_page.group[<g>]`; in it each
//! sidebar a region, `sidebar_page.scene`, `sidebar_page.home` and
//! `sidebar_page.guideline`,
//! with the parts each declares. A reader's `activate` (and
//! `toggle_expanded` on a dropdown) runs the part as a press does.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{Collapsible, DisclosureList};
use engine::widgets::nav::{
    GuidelineSidebar, HomeRow, HomeSidebar, NavLink, SceneItem, SceneSidebar,
};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect};
use engine::widgets::wake::Wake;

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

/// What a card shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shows {
    Scene,
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
    focus: Option<SemId>,
}

/// HomeSidebar/navigation.ts: Explore open on Designs, Assets with its
/// counter, Likes; My Scenes; the two folders of the home.
fn kit_home() -> HomeSidebar {
    HomeSidebar::new(
        vec![
            HomeRow::Menu(
                Collapsible::new("Explore")
                    .icon("layout-grid")
                    .current(true)
                    .open(true),
                DisclosureList::new(["Designs", "Animations"]).current(0),
            ),
            HomeRow::Menu(
                Collapsible::new("Assets").icon("layers").counter("112"),
                DisclosureList::new(["3D Objects", "Materials"]),
            ),
            HomeRow::Link(NavLink::new("Likes").icon("heart")),
        ],
        "My scenes",
        NavLink::new("My Scenes").icon("box"),
    )
    .folders(["Untitled Folder", "3D Icons"])
}

/// SidebarPage/content.ts: the guideline's two groups, each page's icon
/// the Lucide one nearest the kit's drawing; the Sidebar page current,
/// as the kit's `usePathname` makes it on this page.
fn kit_guideline() -> GuidelineSidebar {
    GuidelineSidebar::new()
        .group(
            "Foundations",
            &[
                ("Branding", "venetian-mask"),
                ("Colors", "swatch-book"),
                ("Depths", "lightbulb"),
                ("Icons", "circle"),
                ("Typography", "type"),
            ],
        )
        .group(
            "Components",
            &[
                ("Buttons, tool-tip", "mouse-pointer-2"),
                ("Cards", "credit-card"),
                ("Comment", "message-square-more"),
                ("Dropdown", "arrow-down-wide-narrow"),
                ("Inputs", "keyboard"),
                ("Menu", "equal"),
                ("Notifications", "bell"),
                ("Modal, Dialogs", "square-mouse-pointer"),
                ("Sidebar", "panel-left"),
                ("Prompt Input", "sparkle"),
                ("Toolbar", "ruler"),
                ("Topbar", "panel-top"),
            ],
        )
        .current(13)
}

/// SceneAndAssetsSidebar: its head, the scene's eight rows (Object 2
/// selected, each kind's icon the Lucide one nearest the kit's), the
/// assets' two sections.
fn kit_scene() -> SceneSidebar {
    let item = SceneItem::new;
    SceneSidebar::new("Robot 2.0", "3D Design Project")
        .items(
            vec![
                item("Camera 1", "focus"),
                item("Dome Light", "sun"),
                item("Key Light", "lightbulb"),
                item("Area Light", "lamp-ceiling"),
                item("Object 2", "box"),
                item("Background 2", "box"),
                item("Robot", "box"),
                item("Background 1", "box"),
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
        let groups = vec![
            Group {
                title: "Canvas Sidebar",
                cards: vec![("Scene, Assets", Shows::Scene)],
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
            focus: None,
        }
    }

    fn size(&self, shows: Shows, theme: &Theme) -> (f32, f32) {
        match shows {
            Shows::Home => self.home.size(theme),
            Shows::Guideline => self.guideline.size(theme),
            Shows::Scene => self.scene.size(theme),
        }
    }

    /// Two cards a row (`span={2}`), each as tall as its sidebar asks.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let min_w = theme.navigation.sidebars.width + theme.spacing.md * 2.0;
        let grids = self.groups.iter().map(|g| showcase::Grid {
            span: 2,
            min_w,
            heights: g
                .cards
                .iter()
                .map(|(_, s)| showcase::card_height(self.size(*s, theme).1, theme))
                .collect(),
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
        self.home.wake(dt).merge(self.scene.wake(dt))
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
            }
            c.pop_clip();
            let inset = theme.spacing.sm;
            showcase::chip(c, card.x + inset, card.y + inset, title, theme);
        }
    }

    fn set_focus(&mut self, id: Option<&SemId>) {
        self.home.focus_id(id);
        let at = id.and_then(|i| self.guideline.position(i));
        self.guideline.set_focused(at);
        let part = id.and_then(|i| self.scene.part_at_id(i));
        self.scene.set_focused(part);
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
            "activate" | "toggle_expanded" | "toggle" | "select"
        ) {
            return Err(miss());
        }
        if self.home.activate(&req.target) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> SemId {
        SemId::parse(s).unwrap()
    }

    fn seen(page: &SidebarPage) -> accord::Snapshot {
        let root = id("sidebar_page");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Sidebar");
        page.project(&mut b, &root);
        b.finish().expect("the page projects a valid tree")
    }

    fn req(page: &SidebarPage, target: &str, action: &str) -> Request {
        Request {
            actor: accord::Actor::Test {
                name: "sidebar".into(),
            },
            target: id(target),
            action: accord::ActionName::parse(action).unwrap(),
            param: None,
            observed: accord::Observation {
                revision: seen(page).revision(),
                values: Vec::new(),
            },
            ask: None,
        }
    }

    /// Every sidebar fits its card, whole, on a wide page and a phone.
    #[test]
    fn every_sidebar_sits_inside_its_card() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = SidebarPage::new();
            for w in [1200.0, 330.0] {
                let content = Rect::new(0.0, 0.0, w, 800.0);
                for (_, _, card, at) in page.placed(content, &theme) {
                    assert!(
                        at.x >= card.x && at.bottom() <= card.bottom(),
                        "{at:?} {card:?}"
                    );
                }
            }
        }
    }

    /// A reader's focus rings the part and is projected back; activate
    /// runs it.
    #[test]
    fn a_readers_focus_and_activate_reach_the_sidebars() {
        let mut page = SidebarPage::new();
        let new = id("sidebar_page.home.new");
        assert!(page.focus(&new));
        assert_eq!(page.focus.as_ref(), Some(&new));
        let r = req(&page, "sidebar_page.home.new", "activate");
        page.run(&r).unwrap();
        assert!(page.home.naming);
        let r = req(&page, "sidebar_page.home.row[1]", "toggle_expanded");
        page.run(&r).unwrap();
        let r = req(&page, "sidebar_page.guideline[1]", "activate");
        page.run(&r).unwrap();
        assert_eq!(page.guideline.current_link(), Some(1));
        assert!(page.blur());
        let r = req(&page, "sidebar_page.nothing", "activate");
        assert!(page.run(&r).is_err());
    }
}
