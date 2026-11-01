//! Context Menu page, the hoff core kit's MenuPage (`templates/MenuPage`):
//! "Live" is an area a right click opens the kit's file menu in, at the
//! pointer and inside the window; "Menus" shows the kit's three menus
//! open, each in its own recipe ([`MenuStyle`]): the file menu with its
//! shortcuts in key caps, the filter menu with its checks, the profile
//! menu with its icons. The open menu is the view's
//! (`ActiveOverlay::Menu`, `text_input/input.rs`), as a text field's is.
//!
//! Projected: the area `context_menu_page.group[0].card[0]`, a button
//! whose one action, `open`, opens the menu at its middle: a reader's
//! press, Enter or Space on it, an agent's ask, what a right click does
//! there. The open menu is the view's `context_menu`. A shown menu is a
//! group of its items as labels, a filter's with its `checked`, a file's
//! described by its keys.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::TypeRole;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::overlay::tokens::MenuStyle;
use engine::widgets::prelude::{ContextMenu, EventResult, MenuEntry, Rect, WidgetEvent};
use engine::widgets::recipe::focus_ring_inside;

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

/// The kit's file menu (`FileMenu/index.tsx:2-31`): each title, its keys,
/// and whether it destroys; a rule after the third (`nth-3`).
const FILE: [(&str, Option<&str>, bool); 6] = [
    ("Export...", Some("⇧⌘E"), false),
    ("Create new scene...", Some("⌘N"), false),
    ("Duplicate", Some("⌘D"), false),
    ("Rename", None, false),
    ("Move to folder...", None, false),
    ("Delete", None, true),
];
const FILE_RULE: usize = 3;

/// The kit's file menu, its items numbered from 1.
pub fn file_menu() -> Vec<MenuEntry> {
    let mut out = Vec::with_capacity(FILE.len() + 1);
    for (i, (title, keys, destroys)) in FILE.into_iter().enumerate() {
        if i == FILE_RULE {
            out.push(MenuEntry::Separator);
        }
        let mut item = MenuEntry::item(i as u64 + 1, title);
        if let Some(keys) = keys {
            item = item.shortcut(keys);
        }
        if destroys {
            item = item.intent(Intent::Destructive);
        }
        out.push(item);
    }
    out
}

/// The kit's profile menu (`ProfileMenu/index.tsx:2-27`, the header's
/// `UserDropdown`): four items, a rule (`nth-4`), two; the icons are the
/// kit's own inline paths (`kit-*`).
pub fn profile_menu() -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(1, "Profile").icon("kit-profile"),
        MenuEntry::item(2, "Subscription").icon("kit-subscription"),
        MenuEntry::item(3, "Join Discord").icon("kit-discord"),
        MenuEntry::item(4, "Settings").icon("kit-settings"),
        MenuEntry::Separator,
        MenuEntry::item(5, "Updates").icon("kit-updates"),
        MenuEntry::item(6, "Sign out").icon("kit-sign-out"),
    ]
}

/// The kit's filter menu (`FiltersMenu/items.ts`): the scene types, a
/// rule, the orders; the first of each checked, as the kit loads.
pub fn filter_menu() -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(1, "All scenes").checked(true),
        MenuEntry::item(2, "Designs").checked(false),
        MenuEntry::item(3, "Animation").checked(false),
        MenuEntry::Separator,
        MenuEntry::item(4, "Newest first").checked(true),
        MenuEntry::item(5, "Oldest first").checked(false),
    ]
}

/// The kit's MenuPage menus in its order, each named by its card.
pub fn kit_menus() -> Vec<(&'static str, ContextMenu)> {
    vec![
        (
            "File menu",
            ContextMenu::new(file_menu()).style(MenuStyle::File),
        ),
        (
            "Filter menu",
            ContextMenu::new(filter_menu()).style(MenuStyle::Filter),
        ),
        ("Profile menu", ContextMenu::new(profile_menu())),
    ]
}

/// The menu the area opens: the kit's file menu.
pub fn demo_menu() -> ContextMenu {
    ContextMenu::new(file_menu()).style(MenuStyle::File)
}

/// The title of the file menu's item `id`.
pub fn menu_label(id: u64) -> &'static str {
    usize::try_from(id)
        .ok()
        .and_then(|i| FILE.get(i.wrapping_sub(1)))
        .map_or("?", |(title, ..)| title)
}

/// One of the kit's menus, shown open in its card.
struct Shown {
    chip: &'static str,
    menu: ContextMenu,
}

pub struct ContextMenuPage {
    root: Option<SemId>,
    area: Option<SemId>,
    shown: Vec<Shown>,
    /// A reader focused the area (its ring shows).
    focus: bool,
}

const LIVE_CHIP: &str = "Default, Open, Focus";
const HINT: &str = "Right-click anywhere in this area";
const GROUPS: [&str; 2] = ["Live", "Menus"];

impl ContextMenuPage {
    pub fn new() -> Self {
        let root = root_id("context_menu_page");
        let area = child_id(root.as_ref(), "group")
            .and_then(|g| child_id(Some(&g.key("0")), "card"))
            .map(|cards| cards.key("0"));
        let shown = kit_menus()
            .into_iter()
            .map(|(chip, menu)| Shown { chip, menu })
            .collect();
        Self {
            root,
            area,
            shown,
            focus: false,
        }
    }

    /// The live card, the theme's least card height (the area a right
    /// click opens the menu in, `h-50`), then the kit's menus, each card
    /// as tall as the tallest menu below its chip.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let frame = showcase::body_height(0.0, theme);
        let tallest = self
            .shown
            .iter()
            .map(|s| s.menu.size(theme).1)
            .fold(0.0, f32::max);
        let h = frame + tallest;
        showcase::layout_sized(
            [
                (1, vec![showcase::card_min_h(theme)]),
                (3, vec![h; self.shown.len()]),
            ],
            content,
            theme,
        )
    }

    /// The area a right click opens the menu in: the live card.
    pub fn area(&self, content: Rect, theme: &Theme) -> Rect {
        self.layout(content, theme)[0].cards[0]
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// A press anywhere lets a reader's focus go.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            return EventResult::changed();
        }
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        for (g, at) in layout.iter().enumerate() {
            showcase::title(c, GROUPS[g], content, at.title_y, theme);
            if g + 1 != layout.len() {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        let area = layout[0].cards[0];
        showcase::card(c, area, LIVE_CHIP, theme);
        if let Some(id) = &self.area {
            c.bind(id, [area.x, area.y, area.w, area.h]);
        }
        let body = showcase::body(area, theme);
        let style = theme.typography.role(TypeRole::Body);
        let (w, h) = TextMeasurer::measure_styled(HINT, &style, Some(body.w));
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(HINT, &style, Some(body.w)),
            x: (body.x + (body.w - w) / 2.0).round(),
            y: (area.y + (area.h - h) / 2.0).round(),
            color: theme.colors.text.0,
        });
        if self.focus {
            c.push(focus_ring_inside(area, theme.shape.card, theme));
        }
        for (shown, card) in self.shown.iter().zip(&layout[1].cards) {
            showcase::card(c, *card, shown.chip, theme);
            let body = showcase::body(*card, theme);
            // as wide as it asks, never wider than the card's body (its
            // labels cut to it)
            let (w, h) = shown.menu.size(theme);
            let w = w.min(body.w);
            let x = (body.x + (body.w - w) / 2.0).max(body.x).round();
            let panel = Rect::new(x, body.y, w, h);
            shown.menu.render_in(c, LayerId::DEFAULT, theme, panel, 0.0);
        }
    }
}

impl Component for ContextMenuPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(groups) = child_id(Some(root), "group") else {
            return;
        };
        let live = groups.key("0");
        b.node(&live, Role::Group, GROUPS[0]).parent(root);
        if let Some(area) = &self.area {
            b.node(area, Role::Button, "scene file")
                .parent(&live)
                .description(HINT)
                .action("open");
            if self.focus {
                b.focus(area);
            }
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
                let MenuEntry::Item {
                    id,
                    label,
                    shortcut,
                    checked,
                    ..
                } = entry
                else {
                    continue;
                };
                let mut node = b.node(card.key(id.to_string()), Role::Label, label.as_str());
                node.parent(&card);
                if let Some(keys) = shortcut {
                    node.description(keys.as_str());
                }
                if let Some(on) = checked {
                    node.state("checked", *on);
                }
            }
        }
    }

    /// The area's `open`: the view opens the file menu at its middle.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        if self.area.as_ref() == Some(&req.target) && req.action.as_str() == "open" {
            return Ok(After::ContextMenu);
        }
        Err(format!(
            "the context menu page has no {} on {}",
            req.action, req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.focus = self.area.as_ref() == Some(id);
        true
    }

    fn blur(&mut self) -> bool {
        std::mem::replace(&mut self.focus, false)
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

    /// From 280 to 1600 every 8 px, core and hoff: the area spans the
    /// content, and every kit menu fits its card's body (its width from
    /// the card's narrowest column, 300).
    #[test]
    fn the_area_and_every_menu_stay_in_their_container_across_the_width_range() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = ContextMenuPage::new();
            let mut w = 280.0;
            while w <= 1600.0 {
                let content = Rect::new(0.0, 0.0, w, 10_000.0);
                let area = page.area(content, &theme);
                assert!(inside(area, content), "{w}: the area leaves the page");
                assert!((area.w - w).abs() < 0.01, "{w}: the area spans the row");
                let layout = page.layout(content, &theme);
                for (shown, card) in page.shown.iter().zip(&layout[1].cards) {
                    let body = showcase::body(*card, &theme);
                    let (mw, mh) = shown.menu.size(&theme);
                    assert!(mh <= body.h + 0.01, "{w}: {} is taller", shown.chip);
                    assert!(
                        mw <= body.w + 0.01 || w < 300.0,
                        "{w}: {} is wider than its card",
                        shown.chip
                    );
                }
                w += 8.0;
            }
        }
    }

    #[test]
    fn the_kit_s_file_menu_numbers_its_items_and_rules_after_the_third() {
        let entries = file_menu();
        assert!(matches!(entries[FILE_RULE], MenuEntry::Separator));
        assert_eq!(menu_label(1), "Export...");
        assert_eq!(menu_label(6), "Delete");
        assert_eq!(menu_label(0), "?");
        assert_eq!(menu_label(7), "?");
        let keyed = entries
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    MenuEntry::Item {
                        shortcut: Some(_),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(keyed, 3, "Export, Create and Duplicate carry keys");
    }

    #[test]
    fn the_area_opens_the_menu_and_refuses_the_rest() {
        let mut page = ContextMenuPage::new();
        let Some(area) = page.area.clone() else {
            panic!("the area's id parses");
        };
        let revision = {
            let mut b = SnapshotBuilder::new();
            let Some(root) = page.root.clone() else {
                panic!("the root parses");
            };
            b.node(&root, Role::Region, "Context Menu");
            page.project(&mut b, &root);
            b.finish().unwrap_or_else(|e| panic!("{e}")).revision()
        };
        let observe = |action: &str| Request {
            actor: accord::Actor::Test {
                name: "context_menu".into(),
            },
            target: area.clone(),
            action: accord::ActionName::parse(action).unwrap_or_else(|e| panic!("{e}")),
            param: None,
            observed: accord::Observation {
                revision,
                values: Vec::new(),
            },
            ask: None,
        };
        assert_eq!(page.run(&observe("open")), Ok(After::ContextMenu));
        assert!(page.run(&observe("activate")).is_err());
    }

    #[test]
    fn escape_lets_the_area_s_focus_go() {
        let mut page = ContextMenuPage::new();
        let Some(area) = page.area.clone() else {
            panic!("the area's id parses");
        };
        assert!(page.focus(&area));
        assert!(page.focus);
        assert!(page.blur());
        assert!(!page.blur(), "nothing left to let go");
    }
}
