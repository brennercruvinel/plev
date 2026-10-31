//! Menus page, the kit's MenuPage (`templates/MenuPage/index.tsx`): one
//! group of seven cards three to a row, each a menu open in its card as
//! the template shows it, in its order: the zoom view options, the file
//! menu, the folder picker, the AI model picker, the upload menu, the
//! filter menu and the profile menu.
//!
//! The file, filter and profile menus are front B's (`context_menu`'s
//! entries, its `ContextMenu` in the kit's recipes); the upload menu is
//! the same menu in the profile's recipe, with its icons; the folder
//! picker is `engine::widgets::selection::Folders`. The zoom options are
//! front G's and the AI model picker front C's: until they arrive their
//! cards name them.
//!
//! The menus operate: the pointer lights a row, a press picks it (a
//! filter moves its check within its section), and the card's menu says
//! the last pick. Projected: `menus_page.group[0]`, and in it each card's
//! menu `menus_page.group[0].card[<c>]`, a `Role::Menu` of items
//! `<card>[<id>]` (a `Role::Group` of buttons for the folders), each with
//! an `activate` that picks it as a press does, the menu's `picked` the
//! last pick.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::styling::theme::Theme;
use engine::widgets::overlay::tokens::MenuStyle;
use engine::widgets::prelude::{ContextMenu, EventResult, MenuEntry, Rect, WidgetEvent};
use engine::widgets::recipe::focus_ring;
use engine::widgets::selection::{FolderRow, Folders};

use super::context_menu::{file_menu, filter_menu, profile_menu};
use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, key_under, root_id, under};

/// The template's card: `min-h-121.5 pt-20 pb-16` (MenuPage/index.tsx:20).
const CARD_MIN_H: f32 = 486.0;
const CARD_TOP: f32 = 80.0;
const CARD_BOTTOM: f32 = 64.0;
/// `RowCards` of `span={3}`: three to a row.
const SPAN: usize = 3;

/// The kit's upload menu (`components/Upload/index.tsx:2-27`), its icons
/// the Lucide ones nearest the kit's paths.
fn upload_menu() -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(1, "Add photos or videos").icon("image-plus"),
        MenuEntry::item(2, "Add 3D objects").icon("box"),
        MenuEntry::item(3, "Add files (docs, txt...)").icon("file-plus"),
    ]
}

/// The kit's folders (`components/Folders/folders.tsx`).
const FOLDERS: [&str; 4] = [
    "Untitled Folder",
    "3D Icons",
    "Cute Characters",
    "Baby Monster",
];

enum Body {
    /// Another front's component, named until it arrives.
    Pending(&'static str),
    Menu(ContextMenu),
    Folders(Folders),
}

struct Card {
    /// The template's card title, its chip.
    title: &'static str,
    body: Body,
    /// The last item picked, by its label.
    picked: Option<String>,
}

/// What the keyboard or a reader focused: a menu's item (by id) or a
/// folder row, in card `c`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Spot {
    Item(u64),
    Row(FolderRow),
}

pub struct MenusPage {
    root: Option<SemId>,
    cards: Vec<Card>,
    focus: Option<(usize, Spot)>,
}

/// `menus_page.group[0]`.
fn group_id(root: &SemId) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key("0"))
}

/// `menus_page.group[0].card[<c>]`.
fn card_id(root: &SemId, c: usize) -> Option<SemId> {
    child_id(group_id(root).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// The label of item `id` in `menu`.
fn label(menu: &ContextMenu, id: u64) -> Option<String> {
    menu.entries.iter().find_map(|e| match e {
        MenuEntry::Item { id: i, label, .. } if *i == id => Some(label.clone()),
        _ => None,
    })
}

/// Item `id` picked: a row with a check takes it, and the other checks
/// of its section (between separators) let it go.
fn check(menu: &mut ContextMenu, id: u64) {
    let Some(at) = menu
        .entries
        .iter()
        .position(|e| matches!(e, MenuEntry::Item { id: i, checked: Some(_), .. } if *i == id))
    else {
        return;
    };
    let section = |i: usize, entries: &[MenuEntry]| {
        entries[..i]
            .iter()
            .filter(|e| matches!(e, MenuEntry::Separator))
            .count()
    };
    let mine = section(at, &menu.entries);
    for i in 0..menu.entries.len() {
        let same = section(i, &menu.entries) == mine;
        if let MenuEntry::Item {
            id: item,
            checked: Some(on),
            ..
        } = &mut menu.entries[i]
            && same
        {
            *on = *item == id;
        }
    }
}

impl MenusPage {
    pub fn new() -> Self {
        let menu = |entries, style| Body::Menu(ContextMenu::new(entries).style(style));
        let card = |title, body| Card {
            title,
            body,
            picked: None,
        };
        let mut cards = vec![
            card("Zoom view options", Body::Pending("Zoom, from front G")),
            card("File menu", menu(file_menu(), MenuStyle::File)),
            card("File menu", Body::Folders(Folders::new(FOLDERS))),
            card("Select AI model", Body::Pending("SelectAI, from front C")),
            card("Upload", menu(upload_menu(), MenuStyle::Upload)),
            card("Filter menu", menu(filter_menu(), MenuStyle::Filter)),
            card("Profile menu", menu(profile_menu(), MenuStyle::Profile)),
        ];
        let root = root_id("menus_page");
        if let Some(root) = &root {
            for (c, card) in cards.iter_mut().enumerate() {
                let Some(id) = card_id(root, c) else {
                    continue;
                };
                card.body = match std::mem::replace(&mut card.body, Body::Pending("")) {
                    Body::Menu(m) => Body::Menu(m.sem(id)),
                    Body::Folders(f) => Body::Folders(f.sem(id)),
                    pending => pending,
                };
            }
        }
        Self {
            root,
            cards,
            focus: None,
        }
    }

    fn size(body: &Body, theme: &Theme) -> (f32, f32) {
        match body {
            Body::Pending(_) => (0.0, 0.0),
            Body::Menu(m) => m.size(theme),
            Body::Folders(f) => f.size(theme),
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let heights = self
            .cards
            .iter()
            .map(|c| (CARD_TOP + Self::size(&c.body, theme).1 + CARD_BOTTOM).max(CARD_MIN_H))
            .collect();
        showcase::layout_sized([(SPAN, heights)], content, theme)
    }

    /// The card's widget: centered across, `pt-20` from the card's top.
    fn widget_rect(body: &Body, card: Rect, theme: &Theme) -> Rect {
        let (w, h) = Self::size(body, theme);
        let w = w.min(card.w - theme.spacing.sm * 2.0);
        Rect::new(
            (card.x + (card.w - w) / 2.0).round(),
            card.y + CARD_TOP,
            w,
            h,
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// Card `c` picks `spot`, as a press on it does.
    fn pick(&mut self, c: usize, spot: Spot) -> bool {
        let card = &mut self.cards[c];
        let picked = match (&mut card.body, spot) {
            (Body::Menu(m), Spot::Item(id)) => {
                check(m, id);
                label(m, id)
            }
            (Body::Folders(f), Spot::Row(FolderRow::New)) if f.sem_id().is_some() => {
                Some("New Folder".to_owned())
            }
            (Body::Folders(f), Spot::Row(FolderRow::Folder(i))) => f.folders.get(i).cloned(),
            _ => None,
        };
        let moved = picked.is_some();
        if moved {
            card.picked = picked;
        }
        moved
    }

    fn set_focus(&mut self, at: Option<(usize, Spot)>) {
        for (c, card) in self.cards.iter_mut().enumerate() {
            let spot = at.filter(|a| a.0 == c).map(|a| a.1);
            match &mut card.body {
                Body::Menu(m) => m.set_highlighted(match spot {
                    Some(Spot::Item(id)) => Some(id),
                    _ => None,
                }),
                Body::Folders(f) => f.set_focused(match spot {
                    Some(Spot::Row(r)) => Some(r),
                    _ => None,
                }),
                Body::Pending(_) => {}
            }
        }
        self.focus = at;
    }

    /// The spot `id` names: `(card, spot)`.
    fn at(&self, id: &SemId) -> Option<(usize, Spot)> {
        self.cards
            .iter()
            .enumerate()
            .find_map(|(c, card)| match &card.body {
                Body::Menu(m) => {
                    let key = key_under(m.sem_id()?, id)?;
                    let item = key.parse::<u64>().ok()?;
                    label(m, item).map(|_| (c, Spot::Item(item)))
                }
                Body::Folders(f) => f.row_at_id(id).map(|r| (c, Spot::Row(r))),
                Body::Pending(_) => None,
            })
    }

    pub fn handle_escape(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.handle_escape() {
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        let mut picks = Vec::new();
        for (c, (card, rect)) in self.cards.iter_mut().zip(&layout[0].cards).enumerate() {
            let at = Self::widget_rect(&card.body, *rect, theme);
            let (r, spot) = match &mut card.body {
                Body::Menu(m) => {
                    let (r, id) = m.handle_event(event, at.x, at.y, theme);
                    (r, id.map(Spot::Item))
                }
                Body::Folders(f) => {
                    let (r, row) = f.handle_event(event, at, theme);
                    (r, row.map(Spot::Row))
                }
                Body::Pending(_) => (EventResult::IGNORED, None),
            };
            result = result.merge(r);
            if let Some(spot) = spot {
                picks.push((c, spot));
            }
        }
        for (c, spot) in picks {
            self.pick(c, spot);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let caption = theme.typography.caption_r();
        for (i, (card, rect)) in self.cards.iter().zip(&layout[0].cards).enumerate() {
            showcase::card(c, *rect, card.title, theme);
            let at = Self::widget_rect(&card.body, *rect, theme);
            match &card.body {
                Body::Pending(name) => {
                    let body = showcase::body(*rect, theme);
                    c.push(SceneNode::Text {
                        key: TextNodeKey::from_style(name, &caption, Some(body.w)),
                        x: body.x,
                        y: rect.y + CARD_TOP,
                        color: theme.colors.text_mid.0,
                    });
                }
                Body::Menu(m) => {
                    m.render(c, LayerId::DEFAULT, theme, at.x, at.y);
                    if let Some((_, Spot::Item(id))) = self.focus.filter(|f| f.0 == i) {
                        let rects = m.entry_rects(at.x, at.y, theme);
                        let row = m
                            .entries
                            .iter()
                            .position(|e| matches!(e, MenuEntry::Item { id: x, .. } if *x == id));
                        if let Some(r) = row.and_then(|r| rects.get(r)) {
                            let t = m.tokens(theme);
                            let row = Rect::new(r.x + t.pad, r.y, r.w - t.pad * 2.0, r.h);
                            c.push(focus_ring(row, t.item_radius, theme));
                        }
                    }
                }
                Body::Folders(f) => f.render(c, at, theme),
            }
            if let Some(picked) = &card.picked {
                let body = showcase::body(*rect, theme);
                c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(picked, &caption, Some(body.w)),
                    x: body.x,
                    y: body.bottom() - caption.line_height,
                    color: theme.colors.text_mid.0,
                });
            }
        }
    }
}

impl Component for MenusPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(group) = group_id(root) else {
            return;
        };
        b.node(&group, Role::Group, "Menu").parent(root);
        for card in &self.cards {
            match &card.body {
                Body::Menu(m) => {
                    let Some(id) = m.sem_id() else {
                        continue;
                    };
                    for e in &m.entries {
                        if let MenuEntry::Item {
                            id: item,
                            label,
                            shortcut,
                            checked,
                            ..
                        } = e
                        {
                            let mut node = b.node(id.key(item.to_string()), Role::MenuItem, label);
                            node.parent(id);
                            if let Some(keys) = shortcut {
                                node.description(keys.as_str());
                            }
                            if let Some(on) = checked {
                                node.state("checked", *on);
                            }
                            node.action("activate");
                        }
                    }
                    let mut node = b.node(id, Role::Menu, card.title);
                    node.parent(&group);
                    node.state("picked", card.picked.clone().unwrap_or_default());
                }
                Body::Folders(f) => {
                    if let Some(id) = f.sem_id() {
                        f.project(b, id, card.title, Some(&group));
                        if let Some(mut node) = b.reopen(id) {
                            node.state("picked", card.picked.clone().unwrap_or_default());
                        }
                    }
                }
                Body::Pending(_) => {}
            }
        }
        let focused = self
            .focus
            .and_then(|(c, spot)| match (&self.cards[c].body, spot) {
                (Body::Menu(m), Spot::Item(id)) => Some(m.sem_id()?.key(id.to_string())),
                (Body::Folders(f), Spot::Row(r)) => f.row_id(r),
                _ => None,
            });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// An item's or a folder's `activate` picks it, as its press does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some((c, spot)), "activate") if self.pick(c, spot) => Ok(After::Nothing),
            _ => Err(format!("the page has no {} on {}", req.action, req.target)),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.at(id));
        true
    }

    fn blur(&mut self) -> bool {
        self.handle_escape()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The width sweep, 280 to 1600 every 8 px under the core and the
    /// hoff themes: every card in the content and apart, every menu inside
    /// its card and under its chip.
    #[test]
    fn every_menu_fits_its_card_from_narrow_to_wide() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = MenusPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let at = &page.layout(content, &theme)[0];
                for (n, (card, r)) in page.cards.iter().zip(&at.cards).enumerate() {
                    assert!(
                        r.x >= content.x && r.right() <= content.right() + 0.5,
                        "{w}"
                    );
                    if let Some(next) = at.cards.get(n + 1)
                        && next.y == r.y
                    {
                        assert!(next.x >= r.right(), "{w}: cards overlap");
                    }
                    let inset = theme.spacing.sm;
                    let chip = showcase::chip_rect(r.x + inset, r.y + inset, card.title, &theme);
                    let b = MenusPage::widget_rect(&card.body, *r, &theme);
                    assert!(
                        b.x >= r.x && b.right() <= r.right() + 0.5,
                        "{w}: {}",
                        card.title
                    );
                    assert!(b.y >= chip.bottom() && b.bottom() <= r.bottom(), "{w}");
                }
            }
        }
    }

    /// A press on a filter moves its check within its section and the
    /// menu says it; a press on a folder picks it; the focus rings an
    /// item and Escape lets it go.
    #[test]
    fn a_press_picks_and_a_filter_moves_its_check() {
        let theme = Theme::hoff_core();
        let mut page = MenusPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let layout = page.layout(content, &theme);
        let press = |page: &mut MenusPage, (x, y): (f32, f32)| {
            page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme)
        };
        let Body::Menu(m) = &page.cards[5].body else {
            panic!("the filter menu")
        };
        let at = MenusPage::widget_rect(&page.cards[5].body, layout[0].cards[5], &theme);
        let rows = m.entry_rects(at.x, at.y, &theme);
        assert!(press(&mut page, rows[2].center()).clicked);
        let Body::Menu(m) = &page.cards[5].body else {
            unreachable!()
        };
        let checks: Vec<_> = m
            .entries
            .iter()
            .filter_map(|e| match e {
                MenuEntry::Item { checked, .. } => *checked,
                _ => None,
            })
            .collect();
        assert_eq!(checks, [false, false, true, true, false]);
        assert_eq!(page.cards[5].picked.as_deref(), Some("Animation"));

        let Body::Folders(f) = &page.cards[2].body else {
            panic!("the folders")
        };
        let at = MenusPage::widget_rect(&page.cards[2].body, layout[0].cards[2], &theme);
        let row = f.layout(at, &theme).folders[1];
        press(&mut page, row.center());
        assert_eq!(page.cards[2].picked.as_deref(), Some("3D Icons"));

        let id = card_id(page.root.as_ref().unwrap(), 1).unwrap().key("2");
        assert!(page.focus(&id));
        assert_eq!(page.focus, Some((1, Spot::Item(2))));
        assert!(page.handle_escape());
        assert_eq!(page.focus, None);
    }
}
