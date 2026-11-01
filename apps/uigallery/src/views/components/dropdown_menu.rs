//! Dropdown Menu page, the hoff core kit's menus (the header's
//! `UserDropdown`, the MenuPage template): a trigger opens a menu, placed
//! below its end inside the visible page; an item picked, a press
//! outside, the trigger again or Escape closes it. The widget is
//! `engine::widgets::overlay::dropdown_menu` over the kit's menu
//! (`theme.overlay.menu`).
//!
//! "Live" operates, and says the last item picked. "Menus" shows the
//! kit's MenuPage menus, open, forced, as the Context Menu page defines
//! them: the profile menu, the file menu, the filter menu, each in its
//! own recipe and in a card as tall as it.
//!
//! Projected: the trigger `dropdown_menu_page.group[0].card[0]`, a button
//! with its `expanded`; while open, the menu `<trigger>.panel` and its
//! items `<trigger>.panel[<id>]`; the last pick, `dropdown_menu_page.picked`.
//! A forced menu is a group of its items as labels.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::styling::theme::TypeRole;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::overlay::OverlayManager;
use engine::widgets::overlay::dropdown_menu::DropdownMenu;
use engine::widgets::prelude::{
    ContextMenu, EventResult, IconButton, IconButtonVariant, MenuEntry, Rect, WidgetEvent,
};

use super::showcase::{self, CARD_H, GroupLayout};
use super::{After, Component, child_id, context_menu, key_under, root_id, under};

/// One of the kit's menus, shown open in its card.
struct Shown {
    chip: &'static str,
    menu: ContextMenu,
}

pub struct DropdownMenuPage {
    root: Option<SemId>,
    trigger: IconButton,
    dropdown: DropdownMenu,
    shown: Vec<Shown>,
    /// The label of the last item picked.
    picked: Option<String>,
    /// The trigger (`None`) or an item (`Some(id)`) a reader focused.
    focus: Option<Option<u64>>,
}

const LIVE_CHIP: &str = "Default, Hover, Open, Focus";
const GROUPS: [&str; 2] = ["Live", "Menus"];

fn trigger_id(root: &SemId) -> Option<SemId> {
    let group = child_id(Some(root), "group")?.key("0");
    child_id(Some(&group), "card").map(|family| family.key("0"))
}

impl DropdownMenuPage {
    pub fn new() -> Self {
        let root = root_id("dropdown_menu_page");
        let mut trigger = IconButton::new("settings").variant(IconButtonVariant::Toolbar);
        let mut dropdown = DropdownMenu::new(context_menu::profile_menu());
        if let Some(id) = root.as_ref().and_then(trigger_id) {
            dropdown = dropdown.sem(&id);
            trigger = trigger.sem(id);
        }
        // The profile menu first: the one the trigger opens.
        let mut menus = context_menu::kit_menus();
        menus.rotate_right(1);
        let shown = menus
            .into_iter()
            .map(|(chip, menu)| Shown { chip, menu })
            .collect();
        Self {
            root,
            trigger,
            dropdown,
            shown,
            picked: None,
            focus: None,
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let frame = CARD_H - showcase::body(Rect::new(0.0, 0.0, 0.0, CARD_H), theme).h;
        let tallest = self
            .shown
            .iter()
            .map(|s| s.menu.size(theme).1)
            .fold(0.0, f32::max);
        let groups = [
            (
                1,
                1,
                showcase::card_height(self.trigger.preferred_size(theme).1, theme),
            ),
            (3, self.shown.len(), CARD_H.max(frame + tallest)),
        ];
        showcase::layout_sized(
            groups.map(|(span, n, h)| (span, vec![h; n])),
            content,
            theme,
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// The trigger, and where the live menu goes inside `viewport`.
    fn live(&self, content: Rect, viewport: Rect, theme: &Theme) -> (Rect, Rect) {
        let layout = self.layout(content, theme);
        let card = layout[0].cards[0];
        let trigger = showcase::centered(self.trigger.preferred_size(theme), card);
        let panel = self.dropdown.placement(trigger, viewport, theme).rect;
        (trigger, panel)
    }

    /// The stack in line with the live menu (the page's wake).
    pub fn sync(
        &mut self,
        mgr: &mut OverlayManager,
        content: Rect,
        viewport: Rect,
        theme: &Theme,
    ) -> bool {
        let (_, panel) = self.live(content, viewport, theme);
        let motion = theme.intent_motion(Intent::Neutral);
        self.dropdown.popover.sync(mgr, panel, &motion)
    }

    fn set_focus(&mut self, at: Option<Option<u64>>) {
        self.focus = at;
        self.trigger.set_focused(at == Some(None));
        self.dropdown.highlight(at.flatten());
    }

    fn pick(&mut self, id: u64) {
        let label = self.dropdown.menu.entries.iter().find_map(|e| match *e {
            MenuEntry::Item {
                id: i, ref label, ..
            } if i == id => Some(label.clone()),
            _ => None,
        });
        self.picked = label;
        self.dropdown.close();
        if self.focus.is_some() {
            self.set_focus(Some(None));
        }
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
        let (trigger, panel) = self.live(content, viewport, theme);
        let (r, picked) = self.dropdown.handle_event(event, trigger, panel, theme);
        result = result.merge(r);
        if let Some(id) = picked {
            self.pick(id);
            return result;
        }
        let (px, py) = event.pos();
        if self.dropdown.is_open() && panel.contains(px, py) {
            return result;
        }
        let r = self.trigger.handle_event(event, trigger);
        if r.clicked {
            self.dropdown.toggle();
        }
        result.merge(r)
    }

    /// The groups, the trigger, the kit's menus in their cards; the live
    /// menu, while it shows, on `anchored` at its fade.
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
        let live = layout[0].cards[0];
        showcase::card(c, live, LIVE_CHIP, theme);
        if let Some(picked) = &self.picked {
            let style = theme.typography.run(TypeRole::Caption.regular());
            let body = showcase::body(live, theme);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(picked, &style, Some(body.w)),
                x: body.x,
                y: body.bottom() - style.line_height,
                color: theme.colors.text_mid.0,
            });
        }
        for (shown, card) in self.shown.iter().zip(&layout[1].cards) {
            showcase::card(c, *card, shown.chip, theme);
            let body = showcase::body(*card, theme);
            let (w, _) = shown.menu.size(theme);
            let x = (body.x + (body.w - w) / 2.0).max(body.x).round();
            shown.menu.render(c, LayerId::DEFAULT, theme, x, body.y);
        }
        let (trigger, panel) = self.live(content, viewport, theme);
        self.trigger.active = self.dropdown.is_open();
        self.trigger.render(c, trigger, theme);
        if let Some(alpha) = self.dropdown.popover.shown(mgr) {
            c.set_layer_opacity(anchored, alpha);
            self.dropdown.render(c, anchored, panel, theme);
        }
    }

    /// The item `id` names under the open menu, if it is one.
    fn item(&self, id: &SemId) -> Option<u64> {
        let menu = self.dropdown.menu.sem_id()?;
        key_under(menu, id)?.parse().ok()
    }
}

impl Component for DropdownMenuPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(groups) = child_id(Some(root), "group") else {
            return;
        };
        let live = groups.key("0");
        b.node(&live, Role::Group, GROUPS[0]).parent(root);
        if let Some(id) = self.trigger.sem_id() {
            self.trigger
                .project(b, id, "profile menu", Some(&live))
                .description(LIVE_CHIP);
            self.dropdown.project(b, id, "profile menu", Some(&live));
        }
        if let Some(picked) = child_id(Some(root), "picked") {
            let said = self.picked.as_deref().unwrap_or("nothing picked");
            b.node(&picked, Role::Label, said).parent(&live);
        }
        let menus = groups.key("1");
        b.node(&menus, Role::Group, GROUPS[1]).parent(root);
        let Some(cards) = child_id(Some(&menus), "card") else {
            return;
        };
        for (c, shown) in self.shown.iter().enumerate() {
            let card = cards.key(c.to_string());
            b.node(&card, Role::Group, shown.chip).parent(&menus);
            for entry in &shown.menu.entries {
                if let MenuEntry::Item { id, label, .. } = entry {
                    let at = card.key(id.to_string());
                    b.node(&at, Role::Label, label.as_str()).parent(&card);
                }
            }
        }
        let focused = match self.focus {
            Some(None) => self.trigger.sem_id().cloned(),
            Some(Some(item)) if self.dropdown.is_open() => {
                self.dropdown.menu.sem_id().map(|m| m.key(item.to_string()))
            }
            _ => None,
        };
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// The trigger's `activate` opens or closes the menu; an item's picks
    /// it; the menu's `close` closes it.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let action = req.action.as_str();
        if self.trigger.sem_id() == Some(&req.target) && action == "activate" {
            self.dropdown.toggle();
            return Ok(After::Nothing);
        }
        if self.dropdown.is_open() {
            if self.dropdown.menu.sem_id() == Some(&req.target) && action == "close" {
                self.dropdown.close();
                return Ok(After::Nothing);
            }
            if let Some(id) = self.item(&req.target)
                && action == "activate"
            {
                self.pick(id);
                return Ok(After::Nothing);
            }
        }
        Err(format!(
            "the dropdown menu page has no {action} on {}",
            req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = if self.trigger.sem_id() == Some(id) {
            Some(None)
        } else {
            self.item(id).filter(|_| self.dropdown.is_open()).map(Some)
        };
        self.set_focus(at);
        true
    }

    /// Escape: the open menu closes and the focus goes back to the
    /// trigger; with none open, the focus goes.
    fn blur(&mut self) -> bool {
        if self.dropdown.is_open() {
            self.dropdown.close();
            if self.focus.is_some() {
                self.set_focus(Some(None));
            }
            return true;
        }
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

impl DropdownMenuPage {
    /// The panel's id beside the trigger (for the tests).
    #[cfg(test)]
    fn panel(&self) -> Option<SemId> {
        engine::widgets::overlay::popover::Popover::panel_id(self.trigger.sem_id()?)
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

    /// From 280 to 1600 every 8 px, core and hoff: the live menu stays in
    /// the visible page clear of its trigger, and every kit menu fits its
    /// card's body.
    #[test]
    fn every_menu_stays_in_its_container_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = DropdownMenuPage::new();
            let mut w = 280.0;
            while w <= 1600.0 {
                let content = Rect::new(0.0, 0.0, w, 10_000.0);
                let viewport = Rect::new(0.0, 0.0, w, 800.0);
                let (trigger, panel) = page.live(content, viewport, &theme);
                assert!(
                    inside(panel, viewport),
                    "{w}: the live menu leaves the page"
                );
                assert!(panel.y >= trigger.bottom(), "{w}: below its trigger");
                for (shown, card) in page
                    .shown
                    .iter()
                    .zip(&page.layout(content, &theme)[1].cards)
                {
                    let body = showcase::body(*card, &theme);
                    let (mw, mh) = shown.menu.size(&theme);
                    assert!(
                        mh <= body.h + 0.01,
                        "{w}: {} is taller than its card",
                        shown.chip
                    );
                    assert!(
                        mw <= body.w + 0.01 || w < 300.0,
                        "{w}: {} is wider",
                        shown.chip
                    );
                }
                w += 8.0;
            }
        }
    }

    #[test]
    fn an_item_picked_closes_the_menu_and_says_what_was_picked() {
        let mut page = DropdownMenuPage::new();
        assert!(page.panel().is_some());
        page.set_focus(Some(None));
        page.dropdown.toggle();
        page.pick(4);
        assert_eq!(page.picked.as_deref(), Some("Settings"));
        assert!(!page.dropdown.is_open());
        assert_eq!(page.focus, Some(None), "back on the trigger");
    }

    #[test]
    fn escape_closes_the_menu_then_lets_the_focus_go() {
        let mut page = DropdownMenuPage::new();
        page.set_focus(Some(Some(2)));
        page.dropdown.open();
        assert!(page.blur());
        assert!(!page.dropdown.is_open());
        assert_eq!(page.focus, Some(None));
        assert!(page.blur());
        assert_eq!(page.focus, None);
    }
}
