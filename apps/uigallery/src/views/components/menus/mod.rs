//! Menus page, the kit's MenuPage (`templates/MenuPage/index.tsx`): one
//! group of seven cards three to a row, each a menu open in its card as
//! the template shows it, in its order: the zoom view options, the file
//! menu, the folder picker, the AI model picker, the upload menu, the
//! filter menu and the profile menu.
//!
//! The file, filter and profile menus are front B's (`context_menu`'s
//! entries, its `ContextMenu` in the kit's recipes); the upload menu is
//! the same menu in the profile's recipe, with its icons; the folder
//! picker is `engine::widgets::selection::Folders`; the zoom options are
//! the toolbar's zoom panel (`engine::widgets::nav::Zoom`, the Toolbar
//! page's `kit_zoom`) and the AI model picker the prompt's
//! (`engine::widgets::message::select_ai::SelectAi`, the kit's four models).
//!
//! The menus operate: the pointer lights a row, a press picks it (a
//! filter moves its check within its section, a zoom command takes the
//! check, a zoom toggle flips, a model is chosen), and the card's menu
//! says the last pick. Projected: `menus_page.group[0]`, and in it each
//! card's menu `menus_page.group[0].card[<c>]`, a `Role::Menu` of items
//! `<card>[<id>]` (the zoom's rows `<card>.line[<i>]`, the models
//! `<card>[<i>]`; a `Role::Group` of buttons for the folders), each with
//! an `activate` that picks it as a press does, the menu's `picked` the
//! last pick.

use accord::SemId;
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::Theme;
use engine::styling::theme::TypeRole;
use engine::widgets::form::TextField;
use engine::widgets::message::select_ai::SelectAi;
use engine::widgets::nav::Zoom;
use engine::widgets::overlay::tokens::MenuStyle;
use engine::widgets::prelude::{ContextMenu, EventResult, MenuEntry, Rect, WidgetEvent};
use engine::widgets::recipe::{focus_ring, focus_ring_inside};
use engine::widgets::selection::{FolderRow, Folders};

use super::context_menu::{file_menu, filter_menu, profile_menu};
use super::showcase::{self, GroupLayout};
use super::toolbar::kit_zoom;
use super::{child_id, key_under, root_id};

/// The template's card: `min-h-121.5` (MenuPage/index.tsx:20), the
/// least height of a row of more than one; a card alone in its row is
/// only as tall as it holds.
const CARD_MIN_H: f32 = 486.0;
/// `RowCards` of `span={3}`: three to a row.
const SPAN: usize = 3;

/// The kit's upload menu (`components/Upload/index.tsx:2-27`), its icons
/// the kit's own paths (`kit-upload-*`).
fn upload_menu() -> Vec<MenuEntry> {
    vec![
        MenuEntry::item(1, "Add photos or videos").icon("kit-upload-media"),
        MenuEntry::item(2, "Add 3D objects").icon("kit-upload-object"),
        MenuEntry::item(3, "Add files (docs, txt...)").icon("kit-upload-file"),
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
    Zoom(Box<Zoom>),
    Menu(ContextMenu),
    Folders(Box<Folders>),
    Models(Box<SelectAi>),
}

struct Card {
    /// The template's card title, its chip.
    title: &'static str,
    body: Body,
    /// The last item picked, by its label.
    picked: Option<String>,
}

/// What the keyboard or a reader focused: a menu's item (by id), a
/// folder row, or a zoom row or a model (by index), in card `c`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Spot {
    Item(u64),
    Row(FolderRow),
    Line(usize),
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
            card("Zoom view options", Body::Zoom(Box::new(kit_zoom()))),
            card("File menu", menu(file_menu(), MenuStyle::File)),
            card("File menu", Body::Folders(Box::new(Folders::new(FOLDERS)))),
            card("Select AI model", Body::Models(Box::new(SelectAi::kit()))),
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
                let body = std::mem::replace(&mut card.body, Body::Menu(ContextMenu::new(vec![])));
                card.body = match body {
                    Body::Zoom(z) => Body::Zoom(Box::new((*z).sem(id))),
                    Body::Menu(m) => Body::Menu(m.sem(id)),
                    Body::Folders(f) => Body::Folders(Box::new((*f).sem(id))),
                    Body::Models(m) => Body::Models(Box::new((*m).sem(id))),
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
            Body::Zoom(z) => z.size(theme),
            Body::Menu(m) => m.size(theme),
            Body::Folders(f) => f.size(theme),
            Body::Models(m) => m.size(theme),
        }
    }

    /// The line under a menu that says its last pick: one caption line
    /// and the gap above it.
    fn pick_line(theme: &Theme) -> f32 {
        theme.spacing.sm + Self::caption(theme).line_height
    }

    fn caption(theme: &Theme) -> TextStyle {
        theme.typography.run(TypeRole::Caption.regular())
    }

    /// Each card as tall as its menu and the pick's line under it, in the
    /// showcase's frame (the chip's band above, as much below); a row of
    /// more than one at least the template's `min-h-121.5`, a card alone
    /// in its row only as tall as it holds.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let pick = Self::pick_line(theme);
        let heights = self
            .cards
            .iter()
            .map(|c| showcase::card_height(Self::size(&c.body, theme).1 + pick, theme))
            .collect();
        let grid = showcase::Grid::new(SPAN, showcase::card_min_w(theme), heights)
            .floor(CARD_MIN_H)
            .untitled();
        showcase::layout_grids([grid], content, theme)
    }

    /// The card's widget: centered across, at the top of the card's body
    /// (under the chip), never wider than the body. The widgets that lay
    /// out in a rect (the zoom, the folders, the models) take that width;
    /// a context menu keeps its recipe's, which the narrowest card holds.
    fn widget_rect(body: &Body, card: Rect, theme: &Theme) -> Rect {
        let (w, h) = Self::size(body, theme);
        let room = showcase::body(card, theme);
        let w = w.min(room.w);
        Rect::new((room.x + (room.w - w) / 2.0).round(), room.y, w, h)
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
            (Body::Folders(f), Spot::Row(FolderRow::New)) => {
                f.toggle_naming();
                return true;
            }
            (Body::Folders(f), Spot::Row(FolderRow::Folder(i))) => f.folders.get(i).cloned(),
            (Body::Zoom(z), Spot::Line(i)) => {
                let label = z.line_label(i).map(str::to_owned);
                if label.is_some() {
                    z.choose(i);
                }
                label
            }
            (Body::Models(m), Spot::Line(i)) => m
                .choose(i)
                .then(|| m.chosen_title().map(str::to_owned))
                .flatten(),
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
                Body::Models(m) => m.set_focused(match spot {
                    Some(Spot::Line(i)) => Some(i),
                    _ => None,
                }),
                Body::Zoom(_) => {}
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
                Body::Zoom(z) => {
                    let family = z.sem_id()?.child("line").ok()?;
                    let i = key_under(&family, id)?.parse::<usize>().ok()?;
                    z.line_label(i).map(|_| (c, Spot::Line(i)))
                }
                Body::Models(m) => {
                    let i = key_under(m.sem_id()?, id)?.parse::<usize>().ok()?;
                    (i < m.choices().len()).then_some((c, Spot::Line(i)))
                }
            })
    }

    /// The folder picker, where it is.
    fn folders(&self) -> Option<&Folders> {
        self.cards.iter().find_map(|c| match &c.body {
            Body::Folders(f) => Some(f.as_ref()),
            _ => None,
        })
    }

    /// The zoom panel, where it is.
    fn zoom(&self) -> Option<&Zoom> {
        self.cards.iter().find_map(|c| match &c.body {
            Body::Zoom(z) => Some(z.as_ref()),
            _ => None,
        })
    }

    fn zoom_mut(&mut self) -> Option<&mut Zoom> {
        self.cards.iter_mut().find_map(|c| match &mut c.body {
            Body::Zoom(z) => Some(z.as_mut()),
            _ => None,
        })
    }

    /// The box that has the keys: the folder picker's search or name, or
    /// the zoom's level.
    fn text_field(&self) -> Option<&TextField> {
        let level = self.zoom().map(|z| &z.field).filter(|f| f.is_focused());
        self.folders().and_then(Folders::field).or(level)
    }

    fn text_field_mut(&mut self) -> Option<&mut TextField> {
        if self.zoom().is_some_and(|z| z.field.is_focused()) {
            return self.zoom_mut().map(|z| &mut z.field);
        }
        self.folders_mut().and_then(Folders::field_mut)
    }

    /// The zoom's level is what its box holds.
    fn sync_level(&mut self) {
        if let Some(z) = self.zoom_mut() {
            z.sync_level();
        }
    }

    fn folders_card(&self) -> Option<usize> {
        self.cards
            .iter()
            .position(|c| matches!(c.body, Body::Folders(_)))
    }

    fn folders_mut(&mut self) -> Option<&mut Folders> {
        self.cards.iter_mut().find_map(|c| match &mut c.body {
            Body::Folders(f) => Some(f.as_mut()),
            _ => None,
        })
    }

    /// The new folder's name added (Enter, the name's `submit`): the
    /// card says it.
    fn commit_name(&mut self) -> bool {
        let Some(c) = self.folders_card() else {
            return false;
        };
        let name = self.folders_mut().and_then(Folders::commit_name);
        let made = name.is_some();
        if made {
            self.cards[c].picked = name;
        }
        made
    }

    /// The search box's and the name's rects in `content`.
    fn field_rects(&self, content: Rect, theme: &Theme) -> (Option<Rect>, Option<Rect>) {
        let (Some(c), Some(f)) = (self.folders_card(), self.folders()) else {
            return (None, None);
        };
        let card = self.layout(content, theme)[0].cards[c];
        let l = f.layout(Self::widget_rect(&self.cards[c].body, card, theme), theme);
        (Some(l.search), l.name)
    }

    /// The zoom panel's rect in `content`.
    fn zoom_rect(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let c = self
            .cards
            .iter()
            .position(|c| matches!(c.body, Body::Zoom(_)))?;
        let card = self.layout(content, theme)[0].cards[c];
        Some(Self::widget_rect(&self.cards[c].body, card, theme))
    }

    /// Escape: an open name's box closes, then the field or the row that
    /// has the focus lets it go.
    pub fn handle_escape(&mut self) -> bool {
        if self.folders_mut().is_some_and(Folders::cancel_naming) {
            return true;
        }
        let mut field = self.folders_mut().is_some_and(Folders::blur_fields);
        if let Some(z) = self.zoom_mut().filter(|z| z.field.is_focused()) {
            z.field.unfocus();
            z.sync_level();
            field = true;
        }
        let had = self.focus.is_some();
        self.set_focus(None);
        field || had
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.focus.is_some() {
            self.set_focus(None);
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        let mut picks = Vec::new();
        for (c, (card, rect)) in self.cards.iter_mut().zip(&layout[0].cards).enumerate() {
            let at = Self::widget_rect(&card.body, *rect, theme);
            let (r, spot) = match &mut card.body {
                Body::Menu(m) => {
                    let (r, id) = m.handle_event_in(event, at, 0.0, theme);
                    (r, id.map(Spot::Item))
                }
                Body::Folders(f) => {
                    let (r, row) = f.handle_event(event, at, theme);
                    (r, row.map(Spot::Row))
                }
                Body::Zoom(z) => {
                    let (x, y) = event.pos();
                    let row = z.line_at(x, y, at, theme);
                    let r = z.handle_event(event, at, theme);
                    if r.changed && z.field.is_focused() {
                        z.sync_level();
                    }
                    // the panel ran the row itself: the card only says it
                    let label = row.filter(|_| r.clicked).and_then(|i| z.line_label(i));
                    if let Some(label) = label {
                        card.picked = Some(label.to_owned());
                    }
                    (r, None)
                }
                Body::Models(m) => {
                    let r = m.handle_event(event, at, theme);
                    if r.clicked {
                        card.picked = m.chosen_title().map(str::to_owned);
                    }
                    (r, None)
                }
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
        let caption = Self::caption(theme);
        for (i, (card, rect)) in self.cards.iter().zip(&layout[0].cards).enumerate() {
            showcase::card(c, *rect, card.title, theme);
            let at = Self::widget_rect(&card.body, *rect, theme);
            let ring = self.focus.filter(|f| f.0 == i).map(|f| f.1);
            match &card.body {
                Body::Zoom(z) => {
                    z.render_to_layer(c, LayerId::DEFAULT, at, theme);
                    if let Some(Spot::Line(l)) = ring
                        && let Some(r) = z.line_rects(at, theme).get(l)
                    {
                        let radius = theme.navigation.tool.zoom.line_radius;
                        c.push(focus_ring_inside(*r, radius, theme));
                    }
                }
                Body::Models(m) => m.render(c, LayerId::DEFAULT, at, theme),
                Body::Menu(m) => {
                    m.render_in(c, LayerId::DEFAULT, theme, at, 0.0);
                    if let Some(Spot::Item(id)) = ring {
                        let rects = m.entry_rects(at.x, at.y, theme);
                        let row = m
                            .entries
                            .iter()
                            .position(|e| matches!(e, MenuEntry::Item { id: x, .. } if *x == id));
                        if let Some(r) = row.and_then(|r| rects.get(r)) {
                            let t = m.tokens(theme);
                            let row = Rect::new(r.x + t.pad, r.y, at.w - t.pad * 2.0, r.h);
                            c.push(focus_ring(row, t.item_radius, theme));
                        }
                    }
                }
                Body::Folders(f) => f.render(c, at, theme),
            }
            if let Some(picked) = &card.picked {
                // one line under the menu, cut to the body's width
                let body = showcase::body(*rect, theme);
                let line = TextMeasurer::truncate_to_width(picked, &caption, body.w);
                c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(&line, &caption, None),
                    x: body.x,
                    y: at.bottom() + theme.spacing.sm,
                    color: theme.colors.text_mid.0,
                });
            }
        }
    }
}

mod component;
mod host;
#[cfg(test)]
mod tests;
