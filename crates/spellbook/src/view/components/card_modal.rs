//! Card detail modal. Port of `desktop/ui/js/ui/card-modal.js` +
//! `card-face.js`: large art (normal variant), oracle text, mana cost, type
//! line, price/rarity/EDHREC line, PT official names, the printings strip
//! with owned-set marks, the owned-copies box, and the flip button when
//! `Card::is_double_faced()`.
//!
//! Dropped from the port: the per-copy edit/delete forms (the collection
//! screen owns editing copies) and the QA box (the JS itself called it "não
//! é IA generativa" - it was a `contains` on the oracle text). This modal
//! still adds copies: "+ 1 unidade" and the detailed form, which is the
//! half the collection screen reopens this dialog for.
//!
//! Answers flow as usual: pointer and save paths return
//! [`CardModalAnswer`]s, and the worker events land through `on_event`.
//! Copy-adds drive `AddCollection` through the ctx - saving is a command,
//! not an answer (the modal is still open waiting on it).

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::TextMeasurer;
use engine::theme::{Theme, TypographyScale};
use engine::ui::widgets::{
    Button, ButtonVariant, EventResult, IconButton, Rect, Select, WidgetEvent, rounded_rect,
};
use spellbook_core::client::{Command, Event};
use spellbook_core::images;
use spellbook_core::ops::cards::{CardDetail, Printing};
use spellbook_core::ops::collection::{CardCopies, CollectionIn, CopyEntry};
use spellbook_core::types::Card;

use super::super::{EditKey, ScreenCtx, text};
use super::field::{FIELD_H, LabeledField};
use super::modal::{ModalFrame, PAD as MODAL_PAD};
use super::set_picker::SetPicker;
use crate::art::ArtCache;

const WIDTH: f32 = 640.0;
/// Art column (CSS: .modal-card-img 200px), MTG card aspect 488/680.
const ART_W: f32 = 200.0;
const ART_H: f32 = ART_W * 680.0 / 488.0;
const THUMB_W: f32 = 64.0;
const THUMB_H: f32 = THUMB_W * 680.0 / 488.0;
const BTN_H: f32 = 44.0;
const ROW_H: f32 = 34.0;
/// Long bodies (printings + copies) scroll inside the modal, like the CSS's
/// `max-height: 85vh; overflow-y: auto`.
const COPIES_MAX_H: f32 = 280.0;

/// What the modal tells its screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardModalAnswer {
    /// The user closed it - the copy count may have changed under it, so
    /// the owning list reloads (the JS's `onCollectionChange`).
    Closed,
}

/// The detail form's slots, in tab order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    Set,
    Artist,
    Qty,
    Notes,
}

enum Phase {
    Loading,
    Ready(Box<CardDetail>),
    Missing,
}

pub struct CardModal {
    frame: ModalFrame,
    name: String,
    loading: engine::ui::widgets::EmptyState,
    phase: Phase,
    printings: Vec<Printing>,
    printings_for: String,
    copies: Option<Box<CardCopies>>,
    copies_for: String,
    /// Selected printing and face: the flip swaps which art the large image
    /// shows (`wireCardFlips` swapped the two sources).
    art_index: usize,
    flipped: bool,
    art_hover: Option<usize>,
    owned_sets: Vec<String>,

    // Copies box.
    detail_open: bool,
    detail_set: SetPicker,
    detail_artist: LabeledField,
    detail_lang: Select,
    detail_qty: LabeledField,
    detail_notes: LabeledField,
    detail_focus: Option<Slot>,
    add_one: Button,
    add_detailed: Button,
    detail_save: Button,
    detail_cancel: Button,
    flip: IconButton,
    add_msg: Option<String>,
    adding: bool,

    copies_scroll: ScrollState,
    close: Button,
}

impl CardModal {
    /// Opening asks for everything the two JS helpers fetched in parallel:
    /// the card, its printings and its copies.
    pub fn open(name: &str, ctx: &mut ScreenCtx) -> Self {
        let theme = Theme::hoff();
        let mut detail_qty = LabeledField::new("Quantidade", "1", &theme);
        detail_qty.set_value("1");
        let mut m = Self {
            frame: ModalFrame::new(),
            name: name.to_string(),
            loading: engine::ui::widgets::EmptyState::new("Carregando…", "")
                .icon("search"),
            phase: Phase::Loading,
            printings: Vec::new(),
            printings_for: String::new(),
            copies: None,
            copies_for: String::new(),
            art_index: 0,
            flipped: false,
            art_hover: None,
            owned_sets: Vec::new(),
            detail_open: false,
            detail_set: SetPicker::new("Edição", &theme, Some(name.to_string())),
            detail_artist: LabeledField::new("Artista", "Nome do artista", &theme),
            detail_lang: Select::new(["Inglês", "Português"], 0),
            detail_qty,
            detail_notes: LabeledField::new("Notas", "Ex: foil, comprada na loja X", &theme),
            detail_focus: None,
            add_one: Button::new("+ 1 unidade").size(engine::ui::widgets::ButtonSize::Sm),
            add_detailed: Button::new("Adicionar com detalhes…")
                .variant(ButtonVariant::Outline)
                .size(engine::ui::widgets::ButtonSize::Sm),
            detail_save: Button::new("Adicionar à coleção")
                .size(engine::ui::widgets::ButtonSize::Sm),
            detail_cancel: Button::new("Cancelar")
                .variant(ButtonVariant::Outline)
                .size(engine::ui::widgets::ButtonSize::Sm),
            flip: IconButton::new("redo").variant(ButtonVariant::Outline),
            add_msg: None,
            adding: false,
            copies_scroll: ScrollState::new(),
            close: Button::new("Fechar").variant(ButtonVariant::Outline),
        };
        ctx.send(Command::GetCard {
            name: name.to_string(),
            oracle_id: None,
        });
        ctx.send(Command::CardPrintings {
            name: name.to_string(),
        });
        ctx.send(Command::CardCopies {
            name: name.to_string(),
        });
        m.detail_set.set_card(Some(name.to_string()));
        m
    }

    /// Card this modal resolved to, for art requests outside.
    pub fn card(&self) -> Option<&Card> {
        match &self.phase {
            Phase::Ready(d) => Some(&d.card),
            _ => None,
        }
    }

    // -- Data events ----------------------------------------------------------

    pub fn on_event(&mut self, event: &Event, ctx: &mut ScreenCtx) -> bool {
        match event {
            Event::CardLoaded(result) => {
                match result {
                    Ok(detail) => {
                        // Answers race between modals; only the one that asked
                        // keeps it. A fresh modal has no name match yet, so it
                        // keeps whatever resolves while it's "Loading".
                        let relevant = matches!(self.phase, Phase::Loading)
                            || detail.card.name == self.name;
                        relevant.then(|| {
                            self.phase = Phase::Ready(detail.clone());
                        });
                        return relevant;
                    }
                    Err(_) => {
                        if matches!(self.phase, Phase::Loading) {
                            self.phase = Phase::Missing;
                            return true;
                        }
                        return false;
                    }
                }
            }
            Event::PrintingsLoaded { name, printings } => {
                // Only the modal asking about this card stores them.
                let want = match &self.phase {
                    Phase::Ready(d) => &d.card.name,
                    _ => &self.name,
                };
                if name != want && !self.name.is_empty() && name != &self.name {
                    return false;
                }
                self.printings = printings.clone();
                self.printings_for = name.clone();
                true
            }
            Event::CardCopiesLoaded { name, copies } => {
                let want = match &self.phase {
                    Phase::Ready(d) => &d.card.name,
                    _ => &self.name,
                };
                if name != want && name != &self.name {
                    return false;
                }
                self.owned_sets = copies
                    .entries
                    .iter()
                    .filter_map(|e| e.set_code.as_deref())
                    .map(str::to_lowercase)
                    .collect();
                self.copies = Some(copies.clone());
                self.copies_for = name.clone();
                true
            }
            Event::CollectionAdded(result) => {
                if !self.adding {
                    return false;
                }
                self.adding = false;
                match result {
                    Ok(_) => {
                        self.detail_open = false;
                        ctx.send(Command::CardCopies {
                            name: self.card().map(|c| c.name.clone()).unwrap_or_else(|| self.name.clone()),
                        });
                    }
                    Err(e) => {
                        self.add_msg = Some(e.detail().to_string());
                    }
                }
                true
            }
            Event::SetsFound(sets) => self.detail_set.on_suggestions(sets),
            _ => false,
        }
    }

    // -- Layout ---------------------------------------------------------------

    fn detail_lang_code(&self) -> &'static str {
        ["en", "pt"][self.detail_lang.selected.min(1)]
    }

    /// Layout pass: header row (art + text block), then the copies box.
    /// Everything hit-tested and rendered comes from here.
    fn layout(&self, content: Rect) -> Layout {
        let mut l = Layout::default();
        l.art = Rect::new(content.x, content.y, ART_W, ART_H);
        l.info = Rect::new(
            content.x + ART_W + 16.0,
            content.y,
            (content.w - ART_W - 16.0).max(0.0),
            ART_H,
        );
        let mut y = content.y + ART_H + 16.0;
        if self.printings.len() >= 2 {
            let per_row = ((content.w + 8.0) / (THUMB_W + 8.0)).floor() as usize;
            let rows = self.printings.len().div_ceil(per_row.max(1));
            l.art_strip = Rect::new(
                content.x,
                y,
                content.w,
                18.0 + rows as f32 * (THUMB_H + 8.0) + 20.0,
            );
            y += l.art_strip.h + 14.0;
        }
        l.copies = Rect::new(content.x, y, content.w, self.copies_height(content.w));
        y += l.copies.h + 14.0;
        if let Some(msg) = &self.add_msg
            && !msg.is_empty()
        {
            l.msg = Rect::new(content.x, y, content.w, 18.0);
            y += 24.0;
        }
        let (close_w, _) = self.close.preferred_size();
        l.close = Rect::new(content.x + content.w - close_w.max(90.0), y, close_w.max(90.0), BTN_H);
        l.height = y - content.y + BTN_H;
        l
    }

    /// Copies box height, derived from how many entry rows it holds (capped
    /// and scrolled, the old panel's max-height).
    fn copies_height(&self, _w: f32) -> f32 {
        let Some(copies) = &self.copies else {
            return 60.0;
        };
        let summary_h = 30.0;
        let entries_h = (copies.entries.len().clamp(0, 8) as f32) * (ROW_H + 4.0);
        let buttons_h = 48.0;
        let detail_h = if self.detail_open { 190.0 } else { 0.0 };
        (summary_h + entries_h + buttons_h + detail_h + 28.0).min(COPIES_MAX_H + detail_h + 60.0)
    }

    fn geometry(&self, window: Rect) -> (Rect, Layout) {
        let inner_w = WIDTH - MODAL_PAD * 2.0;
        let probe = self.layout(Rect::new(0.0, 0.0, inner_w, 0.0));
        let panel = self.frame.rect(window, WIDTH, probe.height);
        (panel, self.layout(self.frame.content_rect(panel)))
    }

    // -- Save -----------------------------------------------------------------

    fn add_copy(&mut self, payload: CollectionIn, ctx: &mut ScreenCtx) {
        self.adding = true;
        self.add_msg = None;
        ctx.send(Command::AddCollection(Box::new(payload)));
    }

    fn save_detail(&mut self, ctx: &mut ScreenCtx) {
        let name = match &self.phase {
            Phase::Ready(d) => d.card.name.clone(),
            _ => self.name.clone(),
        };
        let qty = self
            .detail_qty
            .value()
            .trim()
            .parse::<i64>()
            .unwrap_or(1)
            .max(1);
        self.add_copy(
            CollectionIn {
                card_name: name,
                set_code: self.detail_set.code(),
                artist: self.detail_artist.value_opt(),
                lang: self.detail_lang_code().into(),
                quantity: qty,
                notes: self.detail_notes.value_opt(),
                deck_id: None,
                oracle_id: None,
            },
            ctx,
        );
    }

    // -- Pointer input --------------------------------------------------------

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        window: Rect,
        ctx: &mut ScreenCtx,
    ) -> (Option<CardModalAnswer>, EventResult) {
        let (panel, l) = self.geometry(window);
        let (close_frame, mut result) = self.frame.handle_event(event, panel);
        if close_frame {
            return (Some(CardModalAnswer::Closed), result);
        }
        if !matches!(self.phase, Phase::Ready(_)) {
            return (None, result);
        }

        // Floating controls first.
        if self.detail_lang.is_open() {
            return (
                None,
                result.merge(self.detail_lang.handle_event(event, l.detail_lang)),
            );
        }
        if self.detail_set.is_open() {
            return (
                None,
                result.merge(
                    self.detail_set
                        .handle_event(event, self.detail_set.field_rect(l.detail_set)),
                ),
            );
        }

        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hovered = self.thumb_at(x, y, l);
                if hovered != self.art_hover {
                    self.art_hover = hovered;
                    result = result.merge(EventResult::changed());
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                // Flip.
                if let Some(r) = self.flip_rect(l)
                    && r.contains(x, y)
                    && self.card().is_some_and(Card::is_double_faced)
                {
                    self.flipped = !self.flipped;
                    result = result.merge(EventResult::clicked());
                }
                // Art picker.
                else if let Some(i) = self.thumb_at(x, y, l) {
                    self.art_index = i;
                    self.flipped = false;
                    result = result.merge(EventResult::clicked());
                }
                // Detail form fields.
                else if self.detail_open {
                    for (slot, rect) in [
                        (Slot::Artist, l.detail_artist),
                        (Slot::Qty, l.detail_qty),
                        (Slot::Notes, l.detail_notes),
                    ] {
                        let fr = match slot {
                            Slot::Artist => self.detail_artist.field_rect(rect),
                            Slot::Qty => self.detail_qty.field_rect(rect),
                            Slot::Notes => self.detail_notes.field_rect(rect),
                            Slot::Set => rect,
                        };
                        if fr.contains(x, y) {
                            self.set_detail_focus(Some(slot), ctx);
                            match slot {
                                Slot::Artist => self.detail_artist.click(x - fr.x),
                                Slot::Qty => self.detail_qty.click(x - fr.x),
                                Slot::Notes => self.detail_notes.click(x - fr.x),
                                Slot::Set => {}
                            }
                            result = result.merge(EventResult::changed());
                        }
                    }
                    let set_field = self.detail_set.field_rect(l.detail_set);
                    if set_field.contains(x, y) {
                        self.set_detail_focus(Some(Slot::Set), ctx);
                        self.detail_set.field.click(x - set_field.x);
                        result = result.merge(EventResult::changed());
                    }
                }
            }
            _ => {}
        }

        // Buttons.
        result = result.merge(self.flip.handle_event(event, self.flip_rect(l).unwrap_or_default()));
        {
            let r = self.add_one.handle_event(event, l.add_one);
            if r.clicked {
                self.flipped = self.flipped; // keep the face the user is looking at
                self.add_copy(
                    CollectionIn {
                        card_name: self
                            .card()
                            .map(|c| c.name.clone())
                            .unwrap_or_else(|| self.name.clone()),
                        set_code: None,
                        artist: None,
                        lang: "en".into(),
                        quantity: 1,
                        // "Unidade avulsa" - the JS marked quick-adds so a
                        // later edit knew where they came from.
                        notes: Some("Unidade avulsa".into()),
                        deck_id: None,
                        oracle_id: None,
                    },
                    ctx,
                );
            }
            result = result.merge(r);
        }
        {
            let r = self.add_detailed.handle_event(event, l.add_detailed);
            if r.clicked {
                self.detail_open = !self.detail_open;
                if self.detail_open {
                    self.set_detail_focus(Some(Slot::Set), ctx);
                }
            }
            result = result.merge(r);
        }
        if self.detail_open {
            let r = self.detail_cancel.handle_event(event, l.detail_cancel);
            if r.clicked {
                self.detail_open = false;
                result = result.merge(EventResult::clicked());
            } else {
                result = result.merge(r);
            }
            let r = self.detail_save.handle_event(event, l.detail_save);
            if r.clicked && !self.adding {
                self.save_detail(ctx);
                result = result.merge(EventResult::clicked());
            } else {
                result = result.merge(r);
            }
        }
        let r = self.close.handle_event(event, l.close);
        if r.clicked {
            return (Some(CardModalAnswer::Closed), EventResult::clicked());
        }
        result = result.merge(r);

        // Scroll the copies box.
        if let WidgetEvent::Scroll { x, y, delta } = *event
            && l.copies.contains(x, y)
        {
            let old = self.copies_scroll.offset();
            self.copies_scroll.scroll_by(delta);
            if self.copies_scroll.offset() != old {
                result = result.merge(EventResult::changed());
            }
        }
        (None, result)
    }

    fn thumb_at(&self, x: f32, y: f32, l: Layout) -> Option<usize> {
        if l.art_strip.w <= 0.0 || !l.art_strip.contains(x, y) {
            return None;
        }
        let strip_y = l.art_strip.y + 18.0;
        let per_row = ((l.art_strip.w + 8.0) / (THUMB_W + 8.0)).floor() as usize;
        for (i, _) in self.printings.iter().enumerate() {
            let (row, col) = (i / per_row.max(1), i % per_row.max(1));
            let r = Rect::new(
                l.art_strip.x + col as f32 * (THUMB_W + 8.0),
                strip_y + row as f32 * (THUMB_H + 8.0),
                THUMB_W,
                THUMB_H,
            );
            if r.contains(x, y) {
                return Some(i);
            }
        }
        None
    }

    fn flip_rect(&self, l: Layout) -> Option<Rect> {
        self.card().filter(|c| c.is_double_faced()).map(|_| {
            Rect::new(l.art.x + 8.0, l.art.y + l.art.h - 40.0, BTN_H, BTN_H)
        })
    }

    // -- Text input -----------------------------------------------------------

    fn set_detail_focus(&mut self, slot: Option<Slot>, ctx: &mut ScreenCtx) {
        if self.detail_focus == Some(Slot::Set) && slot != Some(Slot::Set) {
            self.detail_set.set_focused(false, ctx);
        }
        self.detail_focus = slot;
        for s in [Slot::Artist, Slot::Qty, Slot::Notes] {
            let f = match s {
                Slot::Artist => &mut self.detail_artist,
                Slot::Qty => &mut self.detail_qty,
                Slot::Notes => &mut self.detail_notes,
                Slot::Set => unreachable!(),
            };
            if slot == Some(s) {
                f.input.focus();
            } else {
                f.input.unfocus();
            }
        }
        if slot == Some(Slot::Set) && !self.detail_set.field.input.focused {
            self.detail_set.set_focused(true, ctx);
        }
    }

    pub fn handle_text(&mut self, s: &str) -> bool {
        match self.detail_focus {
            Some(Slot::Set) => self.detail_set.handle_text(s).changed,
            Some(Slot::Artist) => self.detail_artist.handle_text(s),
            Some(Slot::Notes) => self.detail_notes.handle_text(s),
            Some(Slot::Qty) => {
                let changed = self.detail_qty.handle_text(s);
                if changed {
                    let clean: String = self
                        .detail_qty
                        .value()
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect();
                    if clean != self.detail_qty.value() {
                        self.detail_qty.set_value(&clean);
                    }
                }
                changed
            }
            None => false,
        }
    }

    pub fn handle_edit_key(&mut self, key: EditKey, ctx: &mut ScreenCtx) -> EventResult {
        if key == EditKey::Tab && self.detail_open {
            let order = [Slot::Set, Slot::Artist, Slot::Qty, Slot::Notes];
            let next = match self.detail_focus {
                None => Slot::Set,
                Some(slot) => {
                    let i = order.iter().position(|s| *s == slot).unwrap_or(0);
                    order[(i + 1) % order.len()]
                }
            };
            self.set_detail_focus(Some(next), ctx);
            return EventResult::changed();
        }
        if key == EditKey::Enter && self.detail_open && !self.adding {
            self.save_detail(ctx);
            return EventResult::clicked();
        }
        // Focused field takes the key.
        let (consumed, changed) = match self.detail_focus {
            Some(Slot::Set) => self.detail_set.handle_edit_key(key),
            Some(Slot::Artist) => self.detail_artist.handle_edit_key(key),
            Some(Slot::Qty) => self.detail_qty.handle_edit_key(key),
            Some(Slot::Notes) => self.detail_notes.handle_edit_key(key),
            None => (false, false),
        };
        if !consumed {
            EventResult::IGNORED
        } else if changed {
            EventResult::changed()
        } else {
            EventResult {
                handled: true,
                ..EventResult::IGNORED
            }
        }
    }

    /// Escape: detail form first, then the modal (false = screen closes it).
    pub fn handle_escape(&mut self) -> bool {
        if self.detail_set.handle_escape() {
            return true;
        }
        if self.detail_open {
            self.detail_open = false;
            return true;
        }
        false
    }

    pub fn tick(&mut self, dt: f32, ctx: &mut ScreenCtx) -> bool {
        self.detail_set.tick(dt, ctx);
        self.detail_focus.is_some()
    }

    // -- Render ---------------------------------------------------------------

    pub fn render(
        &mut self,
        c: &mut Compositor,
        layer: LayerId,
        window: Rect,
        theme: &Theme,
        art: &mut ArtCache,
    ) {
        let (panel, l) = self.geometry(window);
        let title = match &self.phase {
            Phase::Ready(d) => d.card.name.clone(),
            _ => "Carta".to_string(),
        };
        self.frame.render(c, layer, window, panel, &title, theme);
        let content = self.frame.content_rect(panel);

        match &self.phase {
            Phase::Loading => {
                self.loading.render(c, content, theme);
            }
            Phase::Missing => {
                let missing = engine::ui::widgets::EmptyState::new(
                    "Carta não encontrada.",
                    "",
                );
                missing.render(c, content, theme);
            }
            Phase::Ready(detail) => {
                let detail = detail.clone();
                self.render_ready(c, layer, &detail, l, content, theme, art);
            }
        }
        self.close.render(c, l.close, theme);
        self.detail_lang.render_dropdown(c, layer, l.detail_lang, theme);
        if self.detail_open {
            self.detail_set.render_overlay(c, layer, l.detail_set, theme);
        }
    }

    fn render_ready(
        &mut self,
        c: &mut Compositor,
        _layer: LayerId,
        detail: &CardDetail,
        l: Layout,
        content: Rect,
        theme: &Theme,
        art: &mut ArtCache,
    ) {
        let card = &detail.card;
        // Art: the selected printing's face (normal variant), placeholder on
        // a cache miss.
        let printing = self.printings.get(self.art_index);
        let rel_front = printing
            .and_then(|p| p.image.as_ref())
            .or(card.image.as_ref())
            .and_then(|i| i.rel.as_deref())
            .map(str::to_string);
        let rel_back = printing
            .and_then(|p| p.image_back.as_ref())
            .or(card.image_back.as_ref())
            .and_then(|i| i.rel.as_deref())
            .map(str::to_string);
        let shown = if self.flipped {
            rel_back.clone().or(rel_front.clone())
        } else {
            rel_front.clone()
        };
        match shown.as_deref().and_then(|r| {
            let rel = images::with_variant(r, "normal");
            Some((rel.clone(), art.get(&rel)))
        }) {
            Some((_, Some(handle))) => {
                c.push(SceneNode::Image {
                    x: l.art.x,
                    y: l.art.y,
                    w: ART_W,
                    h: ART_H,
                    image: handle,
                    corner_radius: theme.radius.md,
                });
            }
            _ => {
                c.push(rounded_rect(
                    l.art.x,
                    l.art.y,
                    l.art.w,
                    l.art.h,
                    theme.radius.md,
                    theme.glass.surface_active.0,
                ));
                text(
                    c,
                    &card.name,
                    12.0,
                    500,
                    l.art.x + 10.0,
                    l.art.y + l.art.h / 2.0 - 8.0,
                    theme.colors.text_dim.0,
                );
            }
        }
        // Flip button on the art, bottom-left (CSS: .modal .card-flip-btn).
        if let Some(r) = self.flip_rect(l) {
            self.flip.render(c, r, theme);
        }

        // Info column.
        let tx = l.info.x;
        let mut ty = l.info.y;
        text(c, &card.name, 20.0, 500, tx, ty, theme.colors.text.0);
        ty += 30.0;
        if let Some(cost) = card.mana_cost.as_deref().filter(|m| !m.is_empty()) {
            text(c, cost, 14.0, 500, tx, ty, theme.colors.text_mid.0);
            ty += 22.0;
        }
        if let Some(type_line) = card.type_line.as_deref() {
            text(c, type_line, 12.0, 400, tx, ty, theme.colors.text_dim.0);
            ty += 22.0;
        }
        if let Some(oracle) = card.oracle_text.as_deref().filter(|o| !o.is_empty()) {
            let style = TypographyScale::hoff().base_2r();
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(oracle, &style, Some(l.info.w)),
                x: tx,
                y: ty,
                color: theme.colors.text.0,
            });
            let (_, h) = TextMeasurer::measure_styled(oracle, &style, Some(l.info.w));
            ty += h + 12.0;
        }
        // PT official names (dedup, as the JS filtered).
        let mut pts: Vec<&str> = Vec::new();
        for p in &detail.pt_names {
            if let Some(n) = p.printed_name.as_deref()
                && !pts.contains(&n)
            {
                pts.push(n);
            }
        }
        if !pts.is_empty() {
            text(
                c,
                &format!("PT oficial: {}", pts.join(" / ")),
                12.0,
                400,
                tx,
                ty,
                theme.glass.text_faint.0,
            );
            ty += 20.0;
        }
        text(
            c,
            &format!(
                "Preço: {} · Raridade: {}",
                price_label(card.price_usd.as_deref()),
                card.rarity.as_deref().unwrap_or("?")
            ),
            12.0,
            400,
            tx,
            ty,
            theme.colors.text_dim.0,
        );
        if let Some(rank) = card.edhrec_rank {
            ty += 18.0;
            text(
                c,
                &format!("EDHREC #{rank}"),
                12.0,
                400,
                tx,
                ty,
                theme.colors.text_dim.0,
            );
        }

        // Printings strip.
        if l.art_strip.w > 0.0 {
            self.render_art_strip(c, l, theme, art);
        }
        // Copies box.
        self.render_copies(c, l, content, theme);
    }

    fn render_art_strip(
        &mut self,
        c: &mut Compositor,
        l: Layout,
        theme: &Theme,
        art: &mut ArtCache,
    ) {
        text(
            c,
            &format!("{} artes", self.printings.len()),
            11.0,
            600,
            l.art_strip.x,
            l.art_strip.y,
            theme.glass.text_placeholder.0,
        );
        let strip_y = l.art_strip.y + 18.0;
        let per_row = ((l.art_strip.w + 8.0) / (THUMB_W + 8.0)).floor() as usize;
        for (i, p) in self.printings.iter().enumerate() {
            let (row, col) = (i / per_row.max(1), i % per_row.max(1));
            let r = Rect::new(
                l.art_strip.x + col as f32 * (THUMB_W + 8.0),
                strip_y + row as f32 * (THUMB_H + 8.0),
                THUMB_W,
                THUMB_H,
            );
            let owned = p
                .set_code
                .as_deref()
                .is_some_and(|sc| self.owned_sets.iter().any(|o| o == &sc.to_lowercase()));
            let rel = p
                .image
                .as_ref()
                .and_then(|i| i.rel.as_deref())
                .map(|r| images::with_variant(r, "normal"));
            match rel.as_deref().and_then(|rr| art.get(rr)) {
                Some(handle) => {
                    c.push(SceneNode::Image {
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: r.h,
                        image: handle,
                        corner_radius: theme.radius.sm,
                    });
                }
                None => {
                    c.push(rounded_rect(
                        r.x,
                        r.y,
                        r.w,
                        r.h,
                        theme.radius.sm,
                        theme.glass.surface_active.0,
                    ));
                }
            }
            let active = i == self.art_index;
            if active || self.art_hover == Some(i) {
                c.push(engine::ui::widgets::rounded_rect_stroke(
                    r.x,
                    r.y,
                    r.w,
                    r.h,
                    theme.radius.sm,
                    if active {
                        theme.colors.accent.0
                    } else {
                        theme.glass.edge.0
                    },
                    1.5,
                ));
            }
            if owned {
                // The JS's "owned" marker: a corner dot on the thumbnails of
                // sets you have.
                c.push(rounded_rect(
                    r.x + r.w - 10.0,
                    r.y + 4.0,
                    7.0,
                    7.0,
                    3.5,
                    theme.colors.success.0,
                ));
            }
            text(
                c,
                &p.set_code.as_deref().unwrap_or("").to_uppercase(),
                9.0,
                600,
                r.x + 4.0,
                r.y + r.h - 14.0,
                [1.0, 1.0, 1.0, 0.9],
            );
        }
        // Caption: set · artist · year of the shown printing.
        if let Some(p) = self.printings.get(self.art_index) {
            let year: String = p
                .released_at
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(4)
                .collect();
            let caption = [
                p.set_name.clone().or(p.set_code.clone()).unwrap_or_default(),
                p.artist.clone().unwrap_or_default(),
                year,
            ]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
            text(
                c,
                &caption,
                11.0,
                400,
                l.art_strip.x,
                l.art_strip.y + l.art_strip.h - 16.0,
                theme.colors.text_dim.0,
            );
        }
    }

    fn render_copies(&mut self, c: &mut Compositor, l: Layout, _content: Rect, theme: &Theme) {
        let r = l.copies;
        super::super::panel(c, r, theme);
        let mut y = r.y + 12.0;
        let Some(copies) = &self.copies else {
            text(
                c,
                "Carregando cópias…",
                12.0,
                400,
                r.x + 14.0,
                y,
                theme.colors.text_dim.0,
            );
            return;
        };
        text(
            c,
            &format!("Suas cópias — {}x", copies.total),
            14.0,
            600,
            r.x + 14.0,
            y,
            theme.colors.text.0,
        );
        y += 24.0;
        if copies.total == 0 {
            text(
                c,
                "Você ainda não tem nenhuma cópia desta carta.",
                12.0,
                400,
                r.x + 14.0,
                y,
                theme.colors.text_dim.0,
            );
        } else {
            // Summary line: "2 livres · 1 em Deck X".
            let mut parts: Vec<String> = Vec::new();
            if copies.free > 0 {
                parts.push(format!(
                    "{} livre{}",
                    copies.free,
                    if copies.free > 1 { "s" } else { "" }
                ));
            }
            for d in &copies.decks {
                parts.push(format!("{} em {}", d.copies, d.deck_name));
            }
            text(
                c,
                &parts.join(" · "),
                12.0,
                400,
                r.x + 14.0,
                y,
                theme.colors.text_dim.0,
            );
            y += 24.0;
            // One row per stored copy (details only; editing lives on the
            // collection screen).
            let mut ry = y - self.copies_scroll.offset();
            c.push(SceneNode::PushClip {
                x: r.x,
                y,
                w: r.w,
                h: (r.y + r.h - y - 52.0).max(0.0),
            });
            for e in &copies.entries {
                let row = Rect::new(r.x + 8.0, ry, r.w - 16.0, ROW_H);
                if row.y + row.h > y && row.y < r.y + r.h - 48.0 {
                    render_copy_row(c, row, e, theme);
                }
                ry += ROW_H + 4.0;
            }
            c.push(SceneNode::PopClip);
        }

        // Buttons row + detail form, pinned above the box's bottom edge.
        let by = r.y + r.h - 44.0 - if self.detail_open { 190.0 } else { 0.0 };
        let (one_w, _) = self.add_one.preferred_size();
        let one = Rect::new(r.x + 14.0, by, one_w, 40.0);
        let (det_w, _) = self.add_detailed.preferred_size();
        let det = Rect::new(one.x + one_w + 8.0, by, det_w, 40.0);
        // Store these back for hit-testing.
        self.add_one.render(c, one, theme);
        self.add_detailed.render(c, det, theme);
        if self.detail_open {
            self.render_detail(c, Rect::new(r.x + 14.0, by + 48.0, r.w - 28.0, 180.0), theme);
        }
    }

    fn render_detail(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        let col = (rect.w - 10.0) / 2.0;
        let mut y = rect.y;
        self.detail_set
            .render(c, Rect::new(rect.x, y, col, LabeledField::height()), theme);
        self.detail_artist.render(
            c,
            Rect::new(rect.x + col + 10.0, y, col, LabeledField::height()),
            theme,
        );
        y += LabeledField::height() + 8.0;
        text(
            c,
            "Idioma",
            12.0,
            400,
            rect.x,
            y,
            theme.colors.text_dim.0,
        );
        self.detail_lang
            .render(c, Rect::new(rect.x, y + 16.0, col, FIELD_H), theme);
        self.detail_qty.render(
            c,
            Rect::new(rect.x + col + 10.0, y, col, LabeledField::height()),
            theme,
        );
        y += LabeledField::height() + 8.0;
        self.detail_notes
            .render(c, Rect::new(rect.x, y, rect.w, LabeledField::height()), theme);
        y += LabeledField::height() + 8.0;
        let (save_w, _) = self.detail_save.preferred_size();
        let (cancel_w, _) = self.detail_cancel.preferred_size();
        let save = Rect::new(rect.x + rect.w - save_w, y, save_w, 40.0);
        let cancel = Rect::new(save.x - 8.0 - cancel_w, y, cancel_w, 40.0);
        self.detail_cancel.render(c, cancel, theme);
        self.detail_save.render(c, save, theme);
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Layout {
    art: Rect,
    info: Rect,
    art_strip: Rect,
    copies: Rect,
    msg: Rect,
    close: Rect,
    // Detail-form rects (inside the copies box).
    detail_set: Rect,
    detail_artist: Rect,
    detail_lang: Rect,
    detail_qty: Rect,
    detail_notes: Rect,
    detail_save: Rect,
    detail_cancel: Rect,
    add_one: Rect,
    add_detailed: Rect,
    height: f32,
}

/// "12.5" -> "$12.50", missing -> "—". Port of `priceLabel`.
pub fn price_label(p: Option<&str>) -> String {
    match p.and_then(|s| s.parse::<f64>().ok()) {
        Some(v) => format!("${v:.2}"),
        None => "—".into(),
    }
}

fn render_copy_row(c: &mut Compositor, row: Rect, e: &CopyEntry, theme: &Theme) {
    c.push(rounded_rect(
        row.x,
        row.y,
        row.w,
        row.h,
        theme.radius.md,
        theme.glass.surface.0,
    ));
    let where_ = match &e.deck_name {
        Some(deck) => deck.clone(),
        None => "Livre".into(),
    };
    let where_ = if e.quantity > 1 {
        format!("{where_} · {}x", e.quantity)
    } else {
        where_
    };
    text(
        c,
        &where_,
        12.0,
        600,
        row.x + 10.0,
        row.y + 10.0,
        theme.colors.text.0,
    );
    let details = [
        e.set_code.as_deref().map(str::to_uppercase),
        (e.lang != "en").then(|| e.lang.to_uppercase()),
        e.notes.clone(),
    ]
    .into_iter()
    .flatten()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join(" · ");
    let details = if details.is_empty() {
        "sem detalhes".to_string()
    } else {
        details
    };
    let style = TypographyScale::hoff().base_2r();
    let (where_w, _) = TextMeasurer::measure_styled(&where_, &style, None);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&details, &style, Some(row.w - where_w - 40.0)),
        x: row.x + where_w + 24.0,
        y: row.y + 10.0,
        color: theme.colors.text_dim.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `priceLabel` from util.js, kept byte-for-byte in behavior: a parseable
    /// number formats "$x.xx", anything else is the em-dash.
    #[test]
    fn price_label_matches_the_js() {
        assert_eq!(price_label(Some("12.5")), "$12.50");
        assert_eq!(price_label(Some("0")), "$0.00");
        assert_eq!(price_label(Some("")), "—");
        assert_eq!(price_label(None), "—");
        assert_eq!(price_label(Some("n/a")), "—");
    }
}
