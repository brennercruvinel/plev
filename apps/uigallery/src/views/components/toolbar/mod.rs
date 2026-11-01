//! Toolbar page: the kit's ToolbarPage (`templates/ToolbarPage`), every
//! card of it on the showcase mold. The toolbar with the select tool on
//! and the zoom control and export (`Default, Zoom - View options,
//! Export: On`), the toolbar in play mode, the video player bare and with
//! repeat, the view controller across (orbit) and upright (tilt), and the
//! player with repeat on a photo (its stand-in gradient). The kit lays
//! the cards in one untitled group; here each kind is a titled group, as
//! every page of the mold names its groups.
//!
//! Projected: a group per title, `toolbar_page.group[<g>]`; in it each
//! toolbar `toolbar_page.bar[<b>]` (its tools `.tool[<i>]`, `.history[<i>]`,
//! the zoom control `.zoom` and, while open, its menu `.zoom.menu` with
//! rows `.line[<i>]`, the export button `.export`), each player
//! `toolbar_page.player[<p>]` (`.play`, `.repeat`), each view controller
//! `toolbar_page.view[<v>]` (`.back`, `.forward`).

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::nav::{Tool, Toolbar, VideoPlayer, ViewController, Zoom};
use engine::widgets::prelude::{Button, EventResult, Rect, WidgetEvent, gradient_fill};

use super::showcase::{self, GroupLayout};
use super::{After, Component, EditKey, child_id, root_id, under};
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;
use engine::widgets::form::TextField;

/// The toolbar and player cards are the kit's `h-60`; the view
/// controller and photo cards its `h-88`.
const BAR_CARD_H: f32 = 240.0;
const TALL_CARD_H: f32 = 352.0;
/// The photo player's width, the kit's `w-140`.
const PHOTO_PLAYER_W: f32 = 560.0;

/// What a card shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shows {
    Bar(usize),
    Player(usize),
    View(usize),
}

struct Card {
    title: &'static str,
    shows: Shows,
    height: f32,
}

struct Group {
    title: &'static str,
    span: usize,
    cards: Vec<Card>,
}

pub struct ToolbarPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    bars: Vec<Toolbar>,
    bar_ids: Vec<Option<SemId>>,
    players: Vec<VideoPlayer>,
    views: Vec<ViewController>,
    /// What a screen reader focused, projected back as the focus.
    focus: Option<SemId>,
    /// The page's rect and theme at the last event: where a focus that
    /// comes from the projection rings its part.
    last: Option<(Rect, Theme)>,
}

/// `toolbar_page.<name>[<i>]`.
fn part_id(root: Option<&SemId>, name: &str, i: usize) -> Option<SemId> {
    child_id(root, name).map(|family| family.key(i.to_string()))
}

fn sub(id: Option<&SemId>, name: &str) -> Option<SemId> {
    child_id(id, name)
}

/// The kit's zoom panel: the five zoom commands, the last run checked
/// (`Zoom to 100%`), and the two view toggles, on.
pub fn kit_zoom() -> Zoom {
    Zoom::new("100%")
        .command("Zoom in", "⌘ +")
        .command("Zoom out", "⌘ -")
        .command("Zoom to 100%", "⌘ 0")
        .command("Zoom to 200%", "⌘ 1")
        .command("Zoom to fit", "⌘ 2")
        .active(2)
        .toggle("Comments", "Shift C", true)
        .toggle("3D Views", "Shift", true)
}

/// A toolbar with ids under `id`: its tools, history, zoom and export.
fn with_ids(mut bar: Toolbar, id: Option<&SemId>) -> Toolbar {
    let tool = |name: &str, i: usize, t: Tool| match part_id(id, name, i) {
        Some(at) => t.sem(at),
        None => t,
    };
    bar.tools = bar
        .tools
        .into_iter()
        .enumerate()
        .map(|(i, t)| tool("tool", i, t))
        .collect();
    bar.history = bar
        .history
        .into_iter()
        .enumerate()
        .map(|(i, t)| tool("history", i, t))
        .collect();
    if let Some(zoom) = sub(id, "zoom") {
        if let Some(menu) = sub(Some(&zoom), "menu") {
            bar.zoom = bar.zoom.clone().sem(menu);
        }
        bar = bar.zoom_sem(zoom);
    }
    if let Some(export) = sub(id, "export") {
        bar.export = bar.export.clone().sem(export);
    }
    bar
}

impl ToolbarPage {
    pub fn new() -> Self {
        let root = root_id("toolbar_page");
        let r = root.as_ref();
        // Toolbar/index.tsx: Select V, Move M, Message Q, Crop C, Play P,
        // the select tool on; Undo U (disabled) and Redo R after the zoom.
        let edit = Toolbar::new(
            vec![
                Tool::new("mouse-pointer-2", "Select", "V"),
                Tool::new("hand", "Move", "M"),
                Tool::new("message-square", "Message", "Q"),
                Tool::new("crop", "Crop", "C"),
                Tool::new("play", "Play", "P"),
            ],
            kit_zoom(),
        )
        .selecting(0)
        .history(vec![
            Tool::new("arrow-left", "Undo", "U").disabled(true),
            Tool::new("arrow-right", "Redo", "R"),
        ])
        .export(Button::new("Export"));
        // Play mode: Close X, Back B, a divider, Crop C; no history.
        let play = Toolbar::new(
            vec![
                Tool::new("x", "Close", "X"),
                Tool::new("undo-2", "Back", "B"),
                Tool::new("crop", "Crop", "C"),
            ],
            kit_zoom(),
        )
        .divider_after(2)
        .export(Button::new("Export"));
        let bar_ids = vec![part_id(r, "bar", 0), part_id(r, "bar", 1)];
        let bars = vec![
            with_ids(edit, bar_ids[0].as_ref()),
            with_ids(play, bar_ids[1].as_ref()),
        ];
        let player = |i: usize, p: VideoPlayer| match part_id(r, "player", i) {
            Some(id) => p.sem(id),
            None => p,
        };
        // VideoPlayer/index.tsx: `8s`, 40% played.
        let players = vec![
            player(0, VideoPlayer::new("8s", 0.4)),
            player(1, VideoPlayer::new("8s", 0.4).repeat(true)),
            player(2, VideoPlayer::new("8s", 0.4).repeat(true).on_image(true)),
        ];
        let view = |i: usize, v: ViewController| match part_id(r, "view", i) {
            Some(id) => v.sem(id),
            None => v,
        };
        let views = vec![
            view(0, ViewController::new(false)),
            view(1, ViewController::new(true)),
        ];
        let card = |title, shows, height| Card {
            title,
            shows,
            height,
        };
        let groups = vec![
            Group {
                title: "Toolbar",
                span: 1,
                cards: vec![
                    card(
                        "Default, Zoom - View options, Export: On",
                        Shows::Bar(0),
                        BAR_CARD_H,
                    ),
                    card("Playmode", Shows::Bar(1), BAR_CARD_H),
                ],
            },
            Group {
                title: "Video player",
                span: 2,
                cards: vec![
                    card("Default", Shows::Player(0), BAR_CARD_H),
                    card("Repeat", Shows::Player(1), BAR_CARD_H),
                ],
            },
            Group {
                title: "View controller",
                span: 2,
                cards: vec![
                    card("Orbit", Shows::View(0), TALL_CARD_H),
                    card("Tilt", Shows::View(1), TALL_CARD_H),
                ],
            },
            Group {
                title: "On an image",
                span: 1,
                cards: vec![card("Default", Shows::Player(2), TALL_CARD_H)],
            },
        ];
        Self {
            root,
            groups,
            bars,
            bar_ids,
            players,
            views,
            focus: None,
            last: None,
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        // A bar that wraps in its card makes the card as tall as the mold
        // needs to keep it clear of the chip.
        let grids = self.groups.iter().map(|g| {
            let card_w = showcase::card_width_at(g.span, showcase::CARD_MIN_W, content, theme);
            let room = card_w - theme.spacing.sm * 2.0;
            let height = |c: &Card| match c.shows {
                Shows::Bar(b) => {
                    let bar_h = self.bars[b].size_in(room, theme).1;
                    c.height.max(showcase::card_height(bar_h, theme))
                }
                _ => c.height,
            };
            showcase::Grid {
                span: g.span,
                min_w: showcase::CARD_MIN_W,
                heights: g.cards.iter().map(height).collect(),
            }
        });
        showcase::layout_grids(grids, content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// The widget's rect in its card: centered, at the card's inner
    /// start when it is wider than the card.
    fn widget_rect(&self, shows: Shows, card: Rect, theme: &Theme) -> Rect {
        let size = match shows {
            // In a card narrower than the bar, the bar wraps (`size_in`).
            Shows::Bar(b) => self.bars[b].size_in(card.w - theme.spacing.sm * 2.0, theme),
            Shows::Player(p) => {
                // As wide as it asks (the photo's player the kit's
                // `w-140`), never wider than the card's inside: the track
                // gives way first.
                let player = &self.players[p];
                let (min_w, h) = player.preferred_size(theme);
                let w = if player.on_image {
                    PHOTO_PLAYER_W.max(min_w)
                } else {
                    min_w
                };
                (w.min(card.w - theme.spacing.sm * 2.0), h)
            }
            Shows::View(v) => self.views[v].preferred_size(theme),
        };
        let at = showcase::centered(size, card);
        let inner = card.x + theme.spacing.sm;
        Rect::new(at.x.max(inner), at.y, size.0, size.1)
    }

    fn placed(&self, content: Rect, theme: &Theme) -> Vec<(Shows, Rect, Rect)> {
        let layout = self.layout(content, theme);
        let mut out = Vec::new();
        for (group, at) in self.groups.iter().zip(&layout) {
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                out.push((
                    card.shows,
                    *rect,
                    self.widget_rect(card.shows, *rect, theme),
                ));
            }
        }
        out
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        if self.last.as_ref().is_none_or(|(at, _)| *at != content) {
            self.last = Some((content, theme.clone()));
        }
        let mut out = EventResult::IGNORED;
        for (shows, _, rect) in self.placed(content, theme) {
            let r = match shows {
                Shows::Bar(b) => self.bars[b].handle_event(event, rect, theme),
                Shows::Player(p) => self.players[p].handle_event(event, rect, theme),
                Shows::View(v) => self.views[v].handle_event(event, rect, theme),
            };
            out = EventResult {
                handled: out.handled || r.handled,
                changed: out.changed || r.changed,
                clicked: out.clicked || r.clicked,
            };
        }
        out
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        let layer = LayerId::DEFAULT;
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                showcase::card_frame(c, *rect, theme);
                let w = self.widget_rect(card.shows, *rect, theme);
                c.push_clip(rect.x, rect.y, rect.w, rect.h);
                match card.shows {
                    Shows::Bar(b) => self.bars[b].render_to_layer(c, layer, w, theme),
                    Shows::Player(p) => {
                        if self.players[p].on_image {
                            photo(c, *rect, theme);
                        }
                        self.players[p].render_to_layer(c, layer, w, theme)
                    }
                    Shows::View(v) => self.views[v].render_to_layer(c, layer, w, theme),
                }
                c.pop_clip();
                showcase::chip(
                    c,
                    rect.x + theme.spacing.sm,
                    rect.y + theme.spacing.sm,
                    card.title,
                    theme,
                );
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        // The open zoom panels and the tooltips over every card.
        for (shows, card, rect) in self.placed(content, theme) {
            if let Shows::Bar(b) = shows {
                let room = Rect::new(content.x, card.y, content.w, card.h);
                self.bars[b].render_overlay(c, layer, rect, room, theme);
            }
        }
    }

    /// Which widget `id` is or is under.
    fn find(&self, id: &SemId) -> Option<(Shows, String)> {
        let rest = |owner: Option<&SemId>| {
            owner.and_then(|o| {
                let (o, t) = (o.to_string(), id.to_string());
                (t == o || t.starts_with(&format!("{o}."))).then(|| t[o.len()..].to_string())
            })
        };
        let bars = self
            .bar_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (Shows::Bar(i), id.clone()));
        let players = self
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| (Shows::Player(i), p.sem_id().cloned()));
        let views = self
            .views
            .iter()
            .enumerate()
            .map(|(i, v)| (Shows::View(i), v.sem_id().cloned()));
        bars.chain(players)
            .chain(views)
            .find_map(|(shows, owner)| rest(owner.as_ref()).map(|r| (shows, r)))
    }
}

/// The photo a player lies on in the kit (`bg-toolbar.jpg`): its stand-in
/// gradient, filling the card inside its rim.
fn photo(c: &mut Compositor, card: Rect, theme: &Theme) {
    let [top, bottom] = theme.navigation.tool.player.image;
    let edge = theme.control.edge_width;
    let r = card.inset(edge);
    c.push(gradient_fill(r, theme.radius.lg, top.0, bottom.0));
}

impl Component for ToolbarPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = part_id(Some(root), "group", g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                match card.shows {
                    Shows::Bar(i) => {
                        if let Some(id) = &self.bar_ids[i] {
                            self.bars[i].project(b, id, card.title, Some(&at));
                        }
                    }
                    Shows::Player(i) => {
                        if let Some(id) = self.players[i].sem_id() {
                            self.players[i].project(b, id, card.title, Some(&at));
                        }
                    }
                    Shows::View(i) => {
                        if let Some(id) = self.views[i].sem_id() {
                            self.views[i].project(b, id, card.title, Some(&at));
                        }
                    }
                }
            }
        }
        if let Some(id) = &self.focus {
            b.focus(id);
        }
    }

    /// `activate`: a tool selects (on a bar that selects), the zoom
    /// control opens or closes its panel, a panel row runs, play flips;
    /// the other buttons have nothing to do in a showcase.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let miss = || format!("the toolbar page has no {} on {}", req.action, req.target);
        if req.action.as_str() != "activate" {
            return Err(miss());
        }
        let (shows, rest) = self.find(&req.target).ok_or_else(miss)?;
        let index = |prefix: &str| {
            rest.strip_prefix(prefix)
                .and_then(|r| r.strip_suffix(']'))
                .and_then(|n| n.parse::<usize>().ok())
        };
        match shows {
            Shows::Bar(i) => {
                let bar = &mut self.bars[i];
                if let Some(t) = index(".tool[") {
                    if bar.selectable && t < bar.tools.len() && !bar.tools[t].disabled {
                        bar.active = Some(t);
                    }
                } else if rest == ".zoom" {
                    bar.open = !bar.open;
                } else if let Some(l) = index(".zoom.menu.line[") {
                    bar.zoom.choose(l);
                }
            }
            Shows::Player(i) => {
                if rest == ".play" {
                    self.players[i].playing = !self.players[i].playing;
                }
            }
            Shows::View(_) => {}
        }
        Ok(After::Nothing)
    }

    /// A reader's focus on a widget of the page: kept and projected
    /// back, so the tree says where it is.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.focus = Some(id.clone());
        self.focus_bars(Some(id));
        true
    }

    fn blur(&mut self) -> bool {
        self.focus_bars(None);
        self.focus.take().is_some()
    }
}

impl ToolbarPage {
    /// The keyboard's focus on a bar's part: its ring, a tool's tooltip.
    /// The bars' rects are the last laid out (`placed_at`).
    fn focus_bars(&mut self, id: Option<&SemId>) {
        let (content, theme) = self
            .last
            .clone()
            .unwrap_or_else(|| (Rect::new(0.0, 0.0, 1200.0, 800.0), Theme::hoff_core()));
        for (shows, _, rect) in self.placed(content, &theme) {
            if let Shows::Bar(b) = shows {
                self.bars[b].focus_id(id, rect, &theme);
            }
        }
    }

    /// The tooltips' delay; `true` while a frame is needed.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for bar in &mut self.bars {
            moving |= bar.tick(dt);
        }
        moving
    }

    /// The open bar's zoom field that has the keys.
    fn level_field(&mut self) -> Option<&mut TextField> {
        self.bars.iter_mut().find_map(|b| b.level_field())
    }

    fn sync_levels(&mut self) {
        for bar in &mut self.bars {
            bar.zoom.sync_level();
        }
    }
}

/// The zoom level's field takes the keys while it is focused.
impl TextHost for ToolbarPage {
    fn handle_text(&mut self, s: &str) -> bool {
        let typed = self.level_field().is_some_and(|f| f.insert(s));
        self.sync_levels();
        typed
    }

    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        let edited = self.level_field().is_some_and(|f| f.edit(key.into()));
        self.sync_levels();
        edited
    }

    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let Some(field) = self.level_field() else {
            return false;
        };
        let changed = match ime {
            ImeEvent::Disabled => field.end_compose(),
            ImeEvent::Preedit(text, cursor) => field.compose("", text, cursor),
            ImeEvent::Commit(text) => field.compose(text, "", None),
        };
        self.sync_levels();
        changed
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        self.placed(content, theme)
            .into_iter()
            .find_map(|(shows, _, rect)| match shows {
                Shows::Bar(b) if self.bars[b].open && self.bars[b].zoom.field.is_focused() => {
                    let bar = &self.bars[b];
                    let at = bar.zoom.text_rect(bar.panel_rect(rect, theme), theme);
                    Some(bar.zoom.field.ime_area(at, theme))
                }
                _ => None,
            })
    }

    fn right_click(&mut self, _x: f32, _y: f32, _content: Rect, _theme: &Theme) -> bool {
        false
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.level_field() {
            Some(field) => Keys::Field(field),
            None => Keys::Nobody,
        }
    }

    /// Escape leaves the field, the level as typed.
    fn handle_escape(&mut self) -> bool {
        let Some(field) = self.level_field() else {
            return false;
        };
        field.unfocus();
        self.sync_levels();
        true
    }
}

#[cfg(test)]
mod tests;
