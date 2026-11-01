//! Prompt Input page, in the showcase mold, as the hoff core kit's
//! PromptInputPage: the prompt input live (its every state a click away:
//! hover, voice input, upload, the model menu), collapsed under its
//! premade prompts, and with the toolbar; then each state at rest, a card
//! apiece; then the model menu on its own (the kit's `SelectAI`). A card
//! is as tall as its widget and the room its menus open into
//! (`showcase::layout_sized`), never less than the kit's `h-50`; a prompt
//! takes the card's inside up to the kit's width.
//!
//! The keyboard: Tab walks the parts in the projection's order, the
//! arrows walk an open menu and the premade prompts, Enter and Space
//! press, Escape closes what is open and then lets the focus go; the
//! characters go to the focused line (the page is a text host).
//!
//! Projected: a group per title, `prompt_input_page.group[<g>]`; in it
//! each card as a group named by its chip, `…card[<c>]`; in a card the
//! prompt, `…card[<c>].prompt`, with its parts as the widget declares
//! them (`.text`, `.add`, `.files[<i>]`, `.setting`, `.model[<i>]`,
//! `.voice`, `.submit`, `.premade[<i>]`, `.surprise`,
//! `.categories[<i>]`), or the model menu, `…card[<c>].models[<i>]`.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::message::prompt::{Part, PromptInput};
use engine::widgets::message::select_ai::SelectAi;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::{After, Component, EditKey, child_id, key_under, root_id, showcase, under};
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;

enum Sample {
    Prompt(Box<PromptInput>),
    Models(SelectAi),
}

struct Card {
    title: &'static str,
    sample: Sample,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct PromptInputPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    let group = child_id(Some(root), "group")?.key(g.to_string());
    child_id(Some(&group), "card").map(|f| f.key(c.to_string()))
}

fn prompt(title: &'static str, p: PromptInput) -> Card {
    Card {
        title,
        sample: Sample::Prompt(Box::new(p)),
    }
}

impl Default for PromptInputPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptInputPage {
    pub fn new() -> Self {
        let mut groups = vec![
            Group {
                title: "Prompt Input",
                columns: 1,
                cards: vec![
                    prompt(
                        "Default, Hover, Voice Input, Upload, Select AI Model",
                        PromptInput::new(),
                    ),
                    prompt(
                        "Collapsed, Premade Prompt",
                        PromptInput::new().collapsed(true).premade(true),
                    ),
                    // TODO(task-104 front G): the kit's ViewController over
                    // the box (`PromptInput isViewController`), once front G
                    // lands it; until then the box alone.
                    prompt("Prompt box with Toolbar", PromptInput::new()),
                ],
            },
            Group {
                title: "States",
                columns: 2,
                cards: vec![
                    prompt("Hover", PromptInput::new().held_hover(true)),
                    prompt("Voice Input", PromptInput::new().voice(true)),
                    prompt("Upload", PromptInput::new().files_open(true)),
                    prompt("Select AI Model", PromptInput::new().models_open(true)),
                    prompt(
                        "Typed, a long prompt",
                        PromptInput::new().with_text(
                            "A low-poly fox curled up asleep in a misty pine forest at dawn, soft rim light",
                        ),
                    ),
                    prompt(
                        "Mixed scripts",
                        PromptInput::new().with_text("سيارة كلاسيكية 🚗 古典的な車 Классика"),
                    ),
                ],
            },
            Group {
                title: "Select AI",
                columns: 2,
                cards: vec![
                    Card {
                        title: "Models",
                        sample: Sample::Models(SelectAi::kit()),
                    },
                    Card {
                        title: "Chosen",
                        sample: Sample::Models(SelectAi::kit().selected(1)),
                    },
                ],
            },
        ];
        let root = root_id("prompt_input_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(cid) = card_id(root, g, c) else {
                        continue;
                    };
                    card.sample = match std::mem::replace(
                        &mut card.sample,
                        Sample::Models(SelectAi::new(Vec::new())),
                    ) {
                        Sample::Prompt(p) => match child_id(Some(&cid), "prompt") {
                            Some(id) => Sample::Prompt(Box::new((*p).sem(id))),
                            None => Sample::Prompt(p),
                        },
                        Sample::Models(m) => match child_id(Some(&cid), "models") {
                            Some(id) => Sample::Models(m.sem(id)),
                            None => Sample::Models(m),
                        },
                    };
                }
            }
        }
        Self { root, groups }
    }

    fn prompts(&self) -> impl Iterator<Item = &PromptInput> {
        self.groups
            .iter()
            .flat_map(|g| &g.cards)
            .filter_map(|c| match &c.sample {
                Sample::Prompt(p) => Some(p.as_ref()),
                Sample::Models(_) => None,
            })
    }

    fn prompts_mut(&mut self) -> impl Iterator<Item = &mut PromptInput> {
        self.groups
            .iter_mut()
            .flat_map(|g| &mut g.cards)
            .filter_map(|c| match &mut c.sample {
                Sample::Prompt(p) => Some(p.as_mut()),
                Sample::Models(_) => None,
            })
    }

    /// A widget's inside in a card: the card less its `md` margins.
    fn avail(card_w: f32, theme: &Theme) -> f32 {
        (card_w - theme.spacing.md * 2.0).max(0.0)
    }

    /// A sample's block in a card `w` wide: its size, and for a prompt the
    /// room over and under its box.
    fn block(sample: &Sample, w: f32, theme: &Theme) -> ((f32, f32), (f32, f32)) {
        match sample {
            Sample::Prompt(p) => {
                let size = p.size(Self::avail(w, theme), theme);
                let room = p.room(Rect::new(0.0, 0.0, size.0, size.1), theme);
                (size, room)
            }
            Sample::Models(m) => (m.size(theme), (0.0, 0.0)),
        }
    }

    /// Per group its layout, and per card where its widget's box goes.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Rect>)> {
        let sized = self.groups.iter().map(|g| {
            let w = showcase::card_width(g.columns, content, theme);
            let cards = g.cards.iter().map(|card| {
                let ((_, h), (above, below)) = Self::block(&card.sample, w, theme);
                showcase::card_height(above + h + below, theme)
            });
            (g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let boxes = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        let ((w, h), (above, below)) = Self::block(&card.sample, rect.w, theme);
                        let block = showcase::centered((w, above + h + below), *rect);
                        Rect::new(block.x, block.y + above, w, h)
                    })
                    .collect();
                (placed, boxes)
            })
            .collect()
    }

    /// Each prompt's box, in page order.
    fn prompt_frames(&self, content: Rect, theme: &Theme) -> Vec<Rect> {
        self.layout(content, theme)
            .into_iter()
            .zip(&self.groups)
            .flat_map(|((_, boxes), group)| {
                boxes
                    .into_iter()
                    .zip(&group.cards)
                    .filter(|(_, c)| matches!(c.sample, Sample::Prompt(_)))
                    .map(|(r, _)| r)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let layout: Vec<_> = self
            .layout(content, theme)
            .into_iter()
            .map(|(g, _)| g)
            .collect();
        showcase::content_height(&layout, content)
    }

    /// The caret's blink in the focused line.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.prompts_mut()
            .fold(Wake::idle(), |w, p| w.merge(p.wake(dt)))
    }

    /// A press lets the keyboard's focus go (a line pressed takes it back
    /// with its caret); then the widget in the card under the pointer
    /// takes it, so a press in one card leaves what another holds open
    /// (each card shows its state at rest). Every widget takes a move, so
    /// a hover can leave.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur_parts() {
            result = EventResult::changed();
        }
        let (px, py) = event.pos();
        let every = matches!(event, WidgetEvent::MouseMove { .. });
        let layout = self.layout(content, theme);
        for (group, (placed, boxes)) in self.groups.iter_mut().zip(&layout) {
            for ((card, rect), at) in group.cards.iter_mut().zip(&placed.cards).zip(boxes) {
                if !every && !rect.contains(px, py) {
                    continue;
                }
                let r = match &mut card.sample {
                    Sample::Prompt(p) => p.handle_event(event, *at, theme),
                    Sample::Models(m) => m.handle_event(event, *at, theme),
                };
                result = result.merge(r);
            }
        }
        result
    }

    /// Escape: in the widget with the focus, what is open closes first
    /// (the focus back on what opened it); then the focus goes.
    fn escape(&mut self) -> bool {
        for p in self.prompts_mut() {
            if p.focused().is_some() && p.close_menus() {
                return true;
            }
        }
        self.blur_parts()
    }

    /// Lets the focus of every part go. `true` when one had it.
    fn blur_parts(&mut self) -> bool {
        let mut had = false;
        for card in self.groups.iter_mut().flat_map(|g| &mut g.cards) {
            match &mut card.sample {
                Sample::Prompt(p) => {
                    had |= p.focused().is_some();
                    p.focus(None);
                }
                Sample::Models(m) => {
                    had |= m.focused().is_some();
                    m.set_focused(None);
                }
            }
        }
        had
    }

    pub fn render(&self, c: &mut Compositor, overlay: LayerId, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, boxes))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), at) in group.cards.iter().zip(&placed.cards).zip(boxes) {
                showcase::card(c, *rect, card.title, theme);
                match &card.sample {
                    Sample::Prompt(p) => p.render(c, *at, overlay, theme),
                    Sample::Models(m) => m.render(c, LayerId::DEFAULT, *at, theme),
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }

    /// The prompt with the focused line, if any.
    fn typing(&mut self) -> Option<&mut PromptInput> {
        self.prompts_mut().find(|p| p.line().is_focused())
    }
}

impl TextHost for PromptInputPage {
    fn handle_text(&mut self, s: &str) -> bool {
        self.typing().is_some_and(|p| p.line_mut().insert(s))
    }

    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        self.typing().is_some_and(|p| p.line_mut().edit(key.into()))
    }

    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = false;
                for p in self.prompts_mut() {
                    dropped |= p.line_mut().end_compose();
                }
                return dropped;
            }
            ImeEvent::Preedit(text, cursor) => ("", text, cursor),
            ImeEvent::Commit(text) => (text, "", None),
        };
        self.typing()
            .is_some_and(|p| p.line_mut().compose(committed, preedit, cursor))
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let frames = self.prompt_frames(content, theme);
        self.prompts()
            .zip(frames)
            .find(|(p, _)| p.line().is_focused())
            .and_then(|(p, frame)| p.ime_area(frame, theme))
    }

    /// A right-click on a line: it takes the keys for its menu, keeping a
    /// selection it has, or taking a caret at the click.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let frames = self.prompt_frames(content, theme);
        let hit = self.prompts().zip(&frames).position(|(p, f)| {
            p.layout(*f, theme).text.is_some_and(|t| t.contains(x, y)) && !p.is_voice()
        });
        let Some(i) = hit else {
            return false;
        };
        let frame = frames[i];
        let Some(p) = self.prompts_mut().nth(i) else {
            return false;
        };
        if p.line().is_focused() && p.line().has_selection() {
            return true;
        }
        p.focus(Some(Part::Text));
        p.handle_event(&WidgetEvent::MouseDown { x, y }, frame, theme);
        p.handle_event(&WidgetEvent::MouseUp { x, y }, frame, theme);
        true
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.typing() {
            Some(p) => Keys::Field(p.line_mut().field_mut()),
            None => Keys::Nobody,
        }
    }

    /// Escape closes what is open, then lets the focus go.
    fn handle_escape(&mut self) -> bool {
        self.escape()
    }
}

impl Component for PromptInputPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let mut focused = None;
        for (g, group) in self.groups.iter().enumerate() {
            let Some(gid) = child_id(Some(root), "group").map(|f| f.key(g.to_string())) else {
                continue;
            };
            b.node(&gid, Role::Group, group.title).parent(root);
            for (c, card) in group.cards.iter().enumerate() {
                let Some(cid) = card_id(root, g, c) else {
                    continue;
                };
                b.node(&cid, Role::Group, card.title).parent(&gid);
                match &card.sample {
                    Sample::Prompt(p) => {
                        if let Some(id) = p.sem_id() {
                            p.project(b, id, "Prompt input", Some(&cid));
                        }
                        focused = focused.or_else(|| p.focused().and_then(|f| p.part_id(f)));
                    }
                    Sample::Models(m) => {
                        if let Some(id) = m.sem_id() {
                            m.project(b, id, "AI models", Some(&cid));
                            focused = focused.or_else(|| m.focused().and_then(|i| m.option_id(i)));
                        }
                    }
                }
            }
        }
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    fn run(&mut self, req: &Request) -> Result<After, String> {
        let action = req.action.as_str();
        for card in self.groups.iter_mut().flat_map(|g| &mut g.cards) {
            match &mut card.sample {
                Sample::Prompt(p) => {
                    if let Some(part) = p.part_of(&req.target) {
                        return p
                            .act(part, action, req.param.as_ref())
                            .map(|()| After::Nothing);
                    }
                }
                Sample::Models(m) => {
                    let Some(id) = m.sem_id().cloned() else {
                        continue;
                    };
                    let row = key_under(&id, &req.target).and_then(|k| k.parse::<usize>().ok());
                    if let (Some(i), "activate") = (row, action) {
                        return if m.choose(i) {
                            Ok(After::Nothing)
                        } else {
                            Err(format!("no model {i}"))
                        };
                    }
                }
            }
        }
        Err(format!(
            "the prompt input page has no {action} on {}",
            req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.blur_parts();
        for card in self.groups.iter_mut().flat_map(|g| &mut g.cards) {
            match &mut card.sample {
                Sample::Prompt(p) => {
                    if let Some(part) = p.part_of(id) {
                        p.focus(Some(part));
                    }
                }
                Sample::Models(m) => {
                    let row = m
                        .sem_id()
                        .and_then(|own| key_under(own, id))
                        .and_then(|k| k.parse::<usize>().ok());
                    if row.is_some() {
                        m.set_focused(row);
                    }
                }
            }
        }
        true
    }

    /// Escape: what is open closes first, then the focus goes.
    fn blur(&mut self) -> bool {
        self.escape()
    }
}

#[cfg(test)]
mod tests;
