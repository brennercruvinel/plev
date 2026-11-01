//! Menubar page: File, Edit and View in a row, each a dropdown menu of the
//! kit (`theme.overlay.menu`), as the kit's toolbar and asset header line
//! their popovers up. The widget is `engine::widgets::overlay::menubar`.
//!
//! "Live" operates: a press opens a menu; while one is open the pointer on
//! another title opens that one; a pick, a press outside or Escape closes
//! it. "Open" shows the bar with its File menu open, forced, in its card.
//!
//! Projected: the bar `menubar_page.bar`, a toolbar; each title
//! `menubar_page.bar[<i>]`, a button with its `expanded`; the open menu
//! `<title>.panel` and its items; the last pick `menubar_page.picked`.
//! The forced bar is a group of its titles and its open menu's items.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::styling::theme::TypeRole;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::overlay::OverlayManager;
use engine::widgets::overlay::menubar::{Menubar, render_bar};
use engine::widgets::prelude::{
    Button, ButtonSize, ButtonVariant, EventResult, MenuEntry, Rect, WidgetEvent, rounded_rect,
};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, flow, key_under, root_id, under};

fn menus() -> Vec<(String, Vec<MenuEntry>)> {
    let items = |labels: &[&str]| {
        labels
            .iter()
            .enumerate()
            .map(|(i, l)| MenuEntry::item(i as u64 + 1, *l))
            .collect::<Vec<_>>()
    };
    vec![
        (
            "File".into(),
            items(&["New scene", "Open...", "Duplicate", "Export..."]),
        ),
        ("Edit".into(), items(&["Undo", "Redo", "Copy", "Paste"])),
        (
            "View".into(),
            items(&["Zoom in", "Zoom out", "Zoom to fit"]),
        ),
    ]
}

const LIVE_CHIP: &str = "Default, Hover, Open, Focus";
const GROUPS: [&str; 2] = ["Live", "States"];

pub struct MenubarPage {
    root: Option<SemId>,
    bar: Menubar,
    titles: Vec<Button>,
    /// The forced bar, its File menu open.
    shown: Menubar,
    shown_titles: Vec<Button>,
    picked: Option<String>,
    /// The title a reader focused.
    focus: Option<usize>,
}

fn titles(bar: &Menubar, id: Option<&SemId>) -> Vec<Button> {
    bar.menus
        .iter()
        .enumerate()
        .map(|(i, (title, _))| {
            let b = Button::new(title.as_str())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm);
            match id {
                Some(bar) => b.sem(bar.key(i.to_string())),
                None => b,
            }
        })
        .collect()
}

impl MenubarPage {
    pub fn new() -> Self {
        let root = root_id("menubar_page");
        let id = child_id(root.as_ref(), "bar");
        let mut bar = Menubar::new(menus());
        if let Some(id) = &id {
            bar = bar.sem(id);
        }
        let titles = titles(&bar, id.as_ref());
        let mut shown = Menubar::new(menus());
        shown.open(0);
        let shown_titles = titles_plain(&shown);
        Self {
            root,
            bar,
            titles,
            shown,
            shown_titles,
            picked: None,
            focus: None,
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let frame = showcase::body_height(0.0, theme);
        let menu_h = self.shown.menus[0].1.menu.size(theme).1;
        let bar_h = ButtonSize::Sm.height(theme) + theme.overlay.menubar.pad * 2.0;
        let open_h = frame + bar_h + theme.overlay.popover.clearance.gap + menu_h;
        let groups = [
            (1, vec![showcase::card_height(bar_h, theme)]),
            (1, vec![open_h]),
        ];
        showcase::layout_sized(groups, content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// The titles laid out in a row at the top of `body`, inside the bar's
    /// padding, wrapping.
    fn row(buttons: &[Button], body: Rect, theme: &Theme) -> Vec<Rect> {
        let t = &theme.overlay.menubar;
        let inner = body.inset(t.pad);
        let sizes = buttons.iter().map(|b| b.preferred_size(theme));
        flow(inner, inner.y, t.gap, sizes).0
    }

    /// The bar around `rects`, its padding outside them.
    fn bar(rects: &[Rect], theme: &Theme) -> Rect {
        let pad = theme.overlay.menubar.pad;
        let (x0, y0) = rects
            .iter()
            .fold((f32::MAX, f32::MAX), |(x, y), r| (x.min(r.x), y.min(r.y)));
        let (x1, y1) = rects.iter().fold((f32::MIN, f32::MIN), |(x, y), r| {
            (x.max(r.right()), y.max(r.bottom()))
        });
        if rects.is_empty() {
            return Rect::default();
        }
        Rect::new(x0 - pad, y0 - pad, x1 - x0 + pad * 2.0, y1 - y0 + pad * 2.0)
    }

    /// The live titles and, for each, where its menu goes in `viewport`.
    fn live(&self, content: Rect, viewport: Rect, theme: &Theme) -> Vec<(Rect, Rect)> {
        let card = self.layout(content, theme)[0].cards[0];
        let rects = Self::row(&self.titles, showcase::body(card, theme), theme);
        rects
            .into_iter()
            .zip(&self.bar.menus)
            .map(|(r, (_, m))| (r, m.placement(r, viewport, theme).rect))
            .collect()
    }

    pub fn sync(
        &mut self,
        mgr: &mut OverlayManager,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> bool {
        let motion = theme.intent_motion(Intent::Neutral);
        let live = self.live(content, viewport, theme);
        let mut changed = false;
        for ((_, panel), (_, menu)) in live.into_iter().zip(&mut self.bar.menus) {
            changed |= menu.popover.sync(mgr, panel, &motion);
        }
        changed
    }

    fn set_focus(&mut self, at: Option<usize>) {
        self.focus = at;
        for (i, t) in self.titles.iter_mut().enumerate() {
            t.set_focused(at == Some(i));
        }
    }

    fn pick(&mut self, menu: usize, id: u64) {
        let entries = &self.bar.menus[menu].1.menu.entries;
        self.picked = entries.iter().find_map(|e| match *e {
            MenuEntry::Item {
                id: i, ref label, ..
            } if i == id => Some(label.clone()),
            _ => None,
        });
        self.bar.close();
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.focus.is_some() {
            self.set_focus(None);
            result = EventResult::changed();
        }
        let live = self.live(content, viewport, theme);
        if let Some(open) = self.bar.open_index() {
            let (trigger, panel) = live[open];
            let (r, picked) = self.bar.menus[open]
                .1
                .handle_event(event, trigger, panel, theme);
            result = result.merge(r);
            if let Some(id) = picked {
                self.pick(open, id);
                return result;
            }
            let (px, py) = event.pos();
            let on_title = live.iter().any(|(t, _)| t.contains(px, py));
            if panel.contains(px, py) || (!self.bar.menus[open].1.is_open() && !on_title) {
                return result;
            }
        }
        for (i, (title, (rect, _))) in self.titles.iter_mut().zip(&live).enumerate() {
            let r = title.handle_event(event, *rect);
            if title.is_hovered()
                && matches!(event, WidgetEvent::MouseMove { .. })
                && self.bar.hover(i)
            {
                result = result.merge(EventResult::changed());
            }
            if r.clicked {
                self.bar.press(i);
            }
            result = result.merge(r);
        }
        result
    }

    /// A title's highlight while its menu is open: the menu's row wash.
    fn highlight(c: &mut Compositor, rect: Rect, theme: &Theme) {
        let m = &theme.overlay.menu;
        c.push(rounded_rect(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            m.item_radius,
            m.hover_fill.0,
        ));
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        anchored: LayerId,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
        mgr: &OverlayManager,
    ) {
        let layout = self.layout(content, theme);
        for (g, at) in layout.iter().enumerate() {
            showcase::title(c, GROUPS[g], content, at.title_y, theme);
            if g + 1 != layout.len() {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        showcase::card(c, layout[0].cards[0], LIVE_CHIP, theme);
        let live = self.live(content, viewport, theme);
        let rects: Vec<Rect> = live.iter().map(|(r, _)| *r).collect();
        render_bar(c, LayerId::DEFAULT, Self::bar(&rects, theme), theme);
        for (i, (title, (rect, panel))) in self.titles.iter().zip(live).enumerate() {
            if self.bar.menus[i].1.is_open() {
                Self::highlight(c, rect, theme);
                self.bar.menus[i].1.fit(panel, theme);
            }
            title.render(c, rect, theme);
            if let Some(alpha) = self.bar.menus[i].1.popover.shown(mgr) {
                c.set_layer_opacity(anchored, alpha);
                self.bar.menus[i].1.render(c, anchored, panel, theme);
            }
        }
        if let Some(picked) = &self.picked {
            let style = theme.typography.run(TypeRole::Caption.regular());
            let body = showcase::body(layout[0].cards[0], theme);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(picked, &style, Some(body.w)),
                x: body.x,
                y: body.bottom() - style.line_height,
                color: theme.colors.text_mid.0,
            });
        }
        let card = layout[1].cards[0];
        showcase::card(c, card, "Open", theme);
        let body = showcase::body(card, theme);
        let rects = Self::row(&self.shown_titles, body, theme);
        render_bar(c, LayerId::DEFAULT, Self::bar(&rects, theme), theme);
        for (i, (title, rect)) in self.shown_titles.iter().zip(&rects).enumerate() {
            let menu = &self.shown.menus[i].1;
            if menu.is_open() {
                Self::highlight(c, *rect, theme);
                let panel = menu.placement(*rect, body, theme).rect;
                menu.render(c, LayerId::DEFAULT, panel, theme);
            }
            title.render(c, *rect, theme);
        }
    }

    /// The live menu `id` is under, and the item, if it is one.
    fn item(&self, id: &SemId) -> Option<(usize, u64)> {
        self.bar.menus.iter().enumerate().find_map(|(i, (_, m))| {
            let menu = m.menu.sem_id()?;
            Some((i, key_under(menu, id)?.parse().ok()?))
        })
    }
}

fn titles_plain(bar: &Menubar) -> Vec<Button> {
    titles(bar, None)
}

impl Component for MenubarPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(groups) = child_id(Some(root), "group") else {
            return;
        };
        let live = groups.key("0");
        b.node(&live, Role::Group, GROUPS[0]).parent(root);
        if let Some(bar) = child_id(Some(root), "bar") {
            b.node(&bar, Role::Toolbar, "menubar")
                .parent(&live)
                .description(LIVE_CHIP);
            for (title, (name, menu)) in self.titles.iter().zip(&self.bar.menus) {
                if let Some(id) = title.sem_id() {
                    title.project(b, id, Some(&bar));
                    menu.project(b, id, name, Some(&bar));
                }
            }
        }
        if let Some(picked) = child_id(Some(root), "picked") {
            b.node(
                &picked,
                Role::Label,
                self.picked.as_deref().unwrap_or("nothing picked"),
            )
            .parent(&live);
        }
        let states = groups.key("1");
        b.node(&states, Role::Group, GROUPS[1]).parent(root);
        let open = states.key("0");
        b.node(&open, Role::Group, "Open").parent(&states);
        for (i, (name, menu)) in self.shown.menus.iter().enumerate() {
            let title = open.key(i.to_string());
            b.node(&title, Role::Label, name.as_str()).parent(&open);
            if menu.is_open() {
                for e in &menu.menu.entries {
                    if let MenuEntry::Item { id, label, .. } = e {
                        b.node(title.key(id.to_string()), Role::Label, label.as_str())
                            .parent(&title);
                    }
                }
            }
        }
        let focused = match self.focus {
            Some(i) => self.titles.get(i).and_then(Button::sem_id).cloned(),
            None => None,
        };
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// A title's `activate` opens or closes its menu; an item's picks it;
    /// a menu's `close` closes it.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let action = req.action.as_str();
        if let Some(i) = self
            .titles
            .iter()
            .position(|t| t.sem_id() == Some(&req.target))
        {
            if action == "activate" {
                self.bar.press(i);
                return Ok(After::Nothing);
            }
        }
        if let Some(open) = self.bar.open_index() {
            let menu = &self.bar.menus[open].1;
            if menu.menu.sem_id() == Some(&req.target) && action == "close" {
                self.bar.close();
                return Ok(After::Nothing);
            }
            if let Some((i, id)) = self.item(&req.target)
                && i == open
                && action == "activate"
            {
                self.pick(i, id);
                return Ok(After::Nothing);
            }
        }
        Err(format!(
            "the menubar page has no {action} on {}",
            req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self.titles.iter().position(|t| t.sem_id() == Some(id));
        self.set_focus(at);
        true
    }

    /// Escape: the open menu closes, the focus stays on its title; with
    /// none open, the focus goes.
    fn blur(&mut self) -> bool {
        if let Some(open) = self.bar.open_index() {
            self.bar.close();
            if self.focus.is_some() {
                self.set_focus(Some(open));
            }
            return true;
        }
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(inner: Rect, outer: Rect) -> bool {
        const E: f32 = 0.01;
        inner.x >= outer.x - E
            && inner.y >= outer.y - E
            && inner.right() <= outer.right() + E
            && inner.bottom() <= outer.bottom() + E
    }

    /// From 280 to 1600 every 8 px, core and hoff: every live menu stays in
    /// the page below its title; the forced one fits its card.
    #[test]
    fn every_menu_stays_in_its_container_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = MenubarPage::new();
            let mut w = 280.0;
            while w <= 1600.0 {
                let content = Rect::new(0.0, 0.0, w, 10_000.0);
                let viewport = Rect::new(0.0, 0.0, w, 800.0);
                for (title, panel) in page.live(content, viewport, &theme) {
                    assert!(inside(panel, viewport), "{w}: a menu leaves the page");
                    assert!(panel.y >= title.bottom(), "{w}: below its title");
                }
                let body = showcase::body(page.layout(content, &theme)[1].cards[0], &theme);
                let title = MenubarPage::row(&page.shown_titles, body, &theme)[0];
                let panel = page.shown.menus[0].1.placement(title, body, &theme).rect;
                assert!(inside(panel, body), "{w}: the forced menu leaves its card");
                w += 8.0;
            }
        }
    }

    #[test]
    fn the_pointer_switches_menus_while_one_is_open_and_escape_closes() {
        let mut page = MenubarPage::new();
        assert!(!page.bar.hover(1), "none open: a hover opens nothing");
        page.bar.press(0);
        assert!(page.bar.hover(1));
        assert_eq!(page.bar.open_index(), Some(1));
        assert!(page.bar.step(1));
        assert_eq!(page.bar.open_index(), Some(2));
        assert!(page.bar.step(1));
        assert_eq!(page.bar.open_index(), Some(0), "wraps");
        page.set_focus(Some(0));
        assert!(page.blur());
        assert_eq!(page.bar.open_index(), None);
        assert_eq!(page.focus, Some(0));
        page.bar.press(1);
        page.pick(1, 3);
        assert_eq!(page.picked.as_deref(), Some("Copy"));
        assert_eq!(page.bar.open_index(), None);
    }
}
