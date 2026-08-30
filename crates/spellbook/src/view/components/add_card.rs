//! "Adicionar carta" modal - single card and bulk list. Port of
//! `desktop/ui/js/ui/add-card.js`, minus the wishlist tab (the wishlist
//! screen owns its add flow, and the JS duplicated the same form for it).
//!
//! Single mode: card name with autocomplete (PT localized names match -
//! the index search goes through `names_localized`), printing via the
//! shared [`SetPicker`], artist, language, quantity, optional deck
//! allocation and notes. List mode: pasted lines ("2 Lightning Bolt", one
//! per row) resolved one add at a time; lines that miss the index stay in
//! the field to be fixed and resent, exactly as the JS kept the failed
//! lines in the textarea. Unlike the JS (which forced one copy per line),
//! a leading quantity is kept.
//!
//! Focus is a single chain: one [`Focusable`] slot is focused at a time -
//! the JS relied on DOM focus, and Tab cycled. Keyboard: Enter saves,
//! Escape unwinds (suggestion list, then the modal).
//!
//! This is the second component (with [`super::set_picker`]) whose methods
//! take `&mut ScreenCtx`: typing drives `SearchCards`/`SearchSets`, saving
//! drives `AddCollection`. The final "it saved" answer leaves through
//! [`AddCardAnswer`]; the screen turns it into the toast and the refresh,
//! like the JS's `onSaved` callback.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::theme::{Theme, TypographyScale};
use engine::ui::widgets::{
    Button, ButtonVariant, EventResult, Rect, Select, Tabs, WidgetEvent, rounded_rect,
};
use spellbook_core::client::{Command, Event};
use spellbook_core::ops::collection::CollectionIn;
use spellbook_core::ops::decks::DeckSummary;
use spellbook_core::types::Card;

use super::super::{EditKey, ScreenCtx, text, with_alpha};
use super::field::{FIELD_H, FIELD_FONT, LabeledField};
use super::modal::{ModalFrame, PAD as MODAL_PAD};
use super::set_picker::SetPicker;

const WIDTH: f32 = 480.0;
const TABS_H: f32 = 40.0;
const SUGGEST_H: f32 = 30.0;
const SUGGEST_ROWS: usize = 6;
const BTN_H: f32 = 44.0;
/// The hint paragraph under the list label (the JS's small text).
const HINT_H: f32 = 34.0;
/// Paste area: 8 lines, as the JS's `rows="8"`.
const LIST_H: f32 = FIELD_FONT * 1.4 * 8.0 + 16.0;
/// Name-search debounce, the JS's 250ms.
const NAME_DEBOUNCE: f32 = 0.250;

/// What the modal tells the screen when it closes for good.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddCardAnswer {
    /// At least one copy was recorded; the screen toasts and reloads.
    Saved,
    /// Dismissed without saving.
    Cancelled,
}

/// One slot of the focus chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    Name,
    Set,
    Artist,
    Qty,
    Notes,
    List,
}

/// A save in flight. The worker answers one `AddCollection` per command;
/// the bulk list replays one command per line, so a failure surfaces in
/// paste order and the rest keep going.
#[derive(Default)]
enum Save {
    #[default]
    Idle,
    Single {
        name: String,
    },
    /// Remaining (qty, name) lines; `deck_id` applies to all of them.
    Bulk {
        deck_id: Option<i64>,
        remaining: Vec<(i64, String)>,
        added: usize,
        missing: Vec<String>,
    },
}

/// Bulk-list parse: "2 Lightning Bolt", "1x Sol Ring", bare "Cultivate",
/// blank lines, comments and section headers skipped. Exported for tests
/// and for the deck import screens, which accept the same shapes.
pub fn parse_bulk_list(text: &str) -> Vec<(i64, String)> {
    spellbook_core::decklist::parse_text(text)
}

pub struct AddCardModal {
    frame: ModalFrame,
    tabs: Tabs,
    decks: Vec<DeckSummary>,

    // Single mode.
    name: LabeledField,
    suggestions: Vec<Card>,
    name_in_flight: bool,
    name_dirty: bool,
    since_name_edit: f32,
    pub set: SetPicker,
    artist: LabeledField,
    lang: Select,
    qty: LabeledField,
    deck: Select,
    notes: LabeledField,

    // List mode.
    list: LabeledField,
    list_deck: Select,
    /// (added, missing names) of the last save pass.
    list_status: Option<(usize, Vec<String>)>,

    focus: Option<Slot>,
    error: Option<String>,
    saving: Save,
    save: Button,
    cancel: Button,
    hover_suggest: Option<usize>,
}

/// Every rect one frame of the dialog needs - one layout pass used by
/// hit-testing and render, like the screens do.
#[derive(Clone, Copy, Debug, Default)]
struct Layout {
    tabs: Rect,
    name: Rect,
    set: Rect,
    artist: Rect,
    lang: Rect,
    qty: Rect,
    deck_label: Rect,
    deck: Rect,
    list_deck: Rect,
    notes: Rect,
    hint: Rect,
    list: Rect,
    status: Rect,
    error: Rect,
    save: Rect,
    cancel: Rect,
    height: f32,
}

impl AddCardModal {
    pub fn new(theme: &Theme) -> Self {
        let mut qty = LabeledField::new("Quantidade", "1", theme);
        qty.set_value("1");
        Self {
            frame: ModalFrame::new(),
            tabs: Tabs::new(["Uma carta", "Adicionar por lista"]),
            decks: Vec::new(),
            name: LabeledField::new("Nome da carta *", "Nome (PT ou EN)…", theme),
            suggestions: Vec::new(),
            name_in_flight: false,
            name_dirty: false,
            since_name_edit: 0.0,
            set: SetPicker::new("Edição", theme, None),
            artist: LabeledField::new("Artista", "Nome do artista", theme),
            lang: Select::new(["Inglês", "Português"], 0),
            qty,
            deck: Select::new(["Nenhum (fica livre na coleção)"], 0),
            notes: LabeledField::new("Notas", "Ex: foil, assinada", theme),
            list: LabeledField::new(
                "Nomes das cartas — um por linha *",
                "Sol Ring\nCultivate\nDemonic Tutor…",
                theme,
            ),
            list_deck: Select::new(["Nenhum (fica livre na coleção)"], 0),
            list_status: None,
            focus: None,
            error: None,
            saving: Save::Idle,
            save: Button::new("Salvar"),
            cancel: Button::new("Cancelar").variant(ButtonVariant::Outline),
            hover_suggest: None,
        }
    }

    /// Opening asks for the decks the allocators list (the JS's `api.decks()`
    /// before building the markup).
    pub fn open(&mut self, ctx: &mut ScreenCtx) {
        ctx.send(Command::ListDecks);
        self.set_focus(Some(Slot::Name), ctx);
    }

    pub fn is_saving(&self) -> bool {
        !matches!(self.saving, Save::Idle)
    }

    fn mode_list(&self) -> bool {
        self.tabs.active == 1
    }

    fn lang_code(&self) -> &'static str {
        ["en", "pt"][self.lang.selected.min(1)]
    }

    fn deck_id_of(&self, sel: &Select) -> Option<i64> {
        (sel.selected > 0)
            .then(|| self.decks.get(sel.selected - 1).map(|d| d.id))
            .flatten()
    }

    fn field_mut(&mut self, slot: Slot) -> &mut LabeledField {
        match slot {
            Slot::Name => &mut self.name,
            Slot::Artist => &mut self.artist,
            Slot::Qty => &mut self.qty,
            Slot::Notes => &mut self.notes,
            Slot::List => &mut self.list,
            // The picker owns its field.
            Slot::Set => &mut self.name,
        }
    }

    fn set_focus(&mut self, slot: Option<Slot>, ctx: &mut ScreenCtx) {
        if self.focus == Some(Slot::Set) && slot != Some(Slot::Set) {
            self.set.set_focused(false, ctx);
        }
        self.focus = slot;
        for s in [Slot::Name, Slot::Artist, Slot::Qty, Slot::Notes, Slot::List] {
            if slot == Some(s) {
                self.field_mut(s).input.focus();
            } else {
                self.field_mut(s).input.unfocus();
            }
        }
        if slot == Some(Slot::Set) && !self.set.field.input.focused {
            self.set.set_focused(true, ctx);
        }
    }

    // -- Layout ---------------------------------------------------------------

    fn layout(&self, content: Rect) -> Layout {
        let mut l = Layout::default();
        let w = content.w;
        l.tabs = Rect::new(content.x, content.y, w, TABS_H);
        let mut y = content.y + TABS_H + 14.0;

        if !self.mode_list() {
            l.name = Rect::new(content.x, y, w, LabeledField::height());
            y += l.name.h + 12.0;
            // The 2x2 form grid of the JS: edição/artista, idioma/quantidade.
            let col = (w - 10.0) / 2.0;
            l.set = Rect::new(content.x, y, col, LabeledField::height());
            l.artist = Rect::new(content.x + col + 10.0, y, col, LabeledField::height());
            y += l.set.h + 10.0;
            l.lang = Rect::new(content.x, y + 18.0, col, FIELD_H);
            l.qty = Rect::new(content.x + col + 10.0, y, col, LabeledField::height());
            y += l.qty.h + 12.0;
            l.deck_label = Rect::new(content.x, y, w, 16.0);
            l.deck = Rect::new(content.x, y + 18.0, w, BTN_H);
            y += BTN_H + 30.0;
            l.notes = Rect::new(content.x, y, w, LabeledField::height());
            y += l.notes.h + 8.0;
        } else {
            l.hint = Rect::new(content.x, y, w, LabeledField::height() + HINT_H);
            y += l.hint.h + 10.0;
            l.list = Rect::new(content.x, y, w, LIST_H);
            y += LIST_H + 12.0;
            l.deck_label = Rect::new(content.x, y, w, 16.0);
            l.list_deck = Rect::new(content.x, y + 18.0, w, BTN_H);
            y += BTN_H + 30.0;
            if self.list_status.is_some() {
                l.status = Rect::new(content.x, y, w, 22.0);
                y += 28.0;
            }
        }

        y += 4.0;
        if self.error.is_some() {
            l.error = Rect::new(content.x, y, w, 18.0);
            y += 24.0;
        }
        let (save_w, _) = self.save.preferred_size();
        let (cancel_w, _) = self.cancel.preferred_size();
        l.save = Rect::new(content.x + w - save_w.max(90.0), y, save_w.max(90.0), BTN_H);
        l.cancel = Rect::new(l.save.x - 8.0 - cancel_w.max(100.0), y, cancel_w.max(100.0), BTN_H);
        l.height = y - content.y + BTN_H;
        l
    }

    /// Panel + layout for this window, in one call so hit-testing and
    /// render compute the same geometry.
    fn geometry(&mut self, window: Rect) -> (Rect, Layout) {
        // Height derives from the layout alone (mode + status + error), not
        // from the window: compute against a free-standing content rect of
        // the dialog's content width.
        let inner_w = WIDTH - MODAL_PAD * 2.0;
        let probe = self.layout(Rect::new(0.0, 0.0, inner_w, 0.0));
        let panel = self.frame.rect(window, WIDTH, probe.height);
        let content = self.frame.content_rect(panel);
        (panel, self.layout(content))
    }

    fn suggest_rect(&self, name_field: Rect) -> Rect {
        let rows = self.suggestions.len().min(SUGGEST_ROWS) as f32;
        Rect::new(
            name_field.x,
            name_field.y + name_field.h + 4.0,
            name_field.w,
            rows * (SUGGEST_H + 4.0) + 8.0,
        )
    }

    // -- Data events ----------------------------------------------------------

    /// Worker answers the modal cares about. Feed every `Event` here.
    pub fn on_event(&mut self, event: &Event, _ctx: &mut ScreenCtx) -> bool {
        match event {
            Event::DecksListed(decks) => {
                self.decks = decks.clone();
                let options: Vec<String> = std::iter::once("Nenhum (fica livre na coleção)".into())
                    .chain(self.decks.iter().map(|d| d.name.clone()))
                    .collect();
                self.deck.options = options.clone();
                self.list_deck.options = options;
                true
            }
            Event::CardsFound(cards) => {
                if !self.name_in_flight {
                    return false;
                }
                self.name_in_flight = false;
                if !self.name.input.focused {
                    return false;
                }
                self.suggestions = cards.iter().take(SUGGEST_ROWS).cloned().collect();
                true
            }
            Event::SetsFound(sets) => self.set.on_suggestions(sets),
            _ => false,
        }
    }

    /// The `CollectionAdded` half of a save. Returns the closing answer
    /// when the save that resolved was the last thing the modal waited on.
    pub fn on_added(
        &mut self,
        result: &spellbook_core::error::Result<i64>,
        ctx: &mut ScreenCtx,
    ) -> (Option<AddCardAnswer>, bool) {
        match std::mem::take(&mut self.saving) {
            Save::Idle => (None, false),
            Save::Single { name } => match result {
                Ok(_) => (Some(AddCardAnswer::Saved), true),
                Err(e) => {
                    self.error = Some(e.detail().to_string());
                    self.field_mut(Slot::Name).set_value(&name);
                    (None, true)
                }
            },
            Save::Bulk {
                deck_id,
                mut remaining,
                mut added,
                mut missing,
            } => {
                match result {
                    Ok(_) => added += 1,
                    Err(_) => {
                        // The name of the failed line is the head we just sent.
                        if let Some((_, name)) = remaining.first().cloned() {
                            missing.push(name);
                        }
                    }
                }
                remaining.remove(0);
                if let Some((qty, name)) = remaining.first().cloned() {
                    self.saving = Save::Bulk {
                        deck_id,
                        remaining,
                        added,
                        missing,
                    };
                    ctx.send(add_command(name, qty, None, deck_id));
                    return (None, true);
                }
                // Done. Missed names stay in the field to be fixed and
                // resent - the JS's partial-success path.
                self.list_status = Some((added, missing.clone()));
                if missing.is_empty() {
                    (Some(AddCardAnswer::Saved), true)
                } else {
                    self.list.set_value(&missing.join("\n"));
                    // Count them as saved when at least one went through:
                    // the toast is the screen's, the modal stays open.
                    if added > 0 {
                        ctx.toast(format!("{added} adicionada(s)."), engine::theme::Intent::Neutral);
                    }
                    (None, true)
                }
            }
        }
    }

    // -- Save -----------------------------------------------------------------

    fn save(&mut self, ctx: &mut ScreenCtx) {
        if self.is_saving() {
            return;
        }
        self.error = None;
        if !self.mode_list() {
            let card_name = self.name.value().trim().to_string();
            if card_name.is_empty() {
                self.error = Some("Preencha o nome da carta.".into());
                self.set_focus(Some(Slot::Name), ctx);
                return;
            }
            let qty = self.qty.value().trim().parse::<i64>().unwrap_or(1).max(1);
            self.saving = Save::Single {
                name: card_name.clone(),
            };
            ctx.send(Command::AddCollection(Box::new(CollectionIn {
                card_name,
                set_code: self.set.code(),
                artist: self.artist.value_opt(),
                lang: self.lang_code().into(),
                quantity: qty,
                notes: self.notes.value_opt(),
                deck_id: self.deck_id_of(&self.deck),
                oracle_id: None,
            })));
        } else {
            let lines = parse_bulk_list(self.list.value());
            if lines.is_empty() {
                self.error = Some("Digite ao menos um nome de carta.".into());
                self.set_focus(Some(Slot::List), ctx);
                return;
            }
            self.list_status = None;
            let deck_id = self.deck_id_of(&self.list_deck);
            let (qty, name) = lines[0].clone();
            self.saving = Save::Bulk {
                deck_id,
                remaining: lines[1..].to_vec(),
                added: 0,
                missing: Vec::new(),
            };
            // `remaining` in state excludes the head only after it's sent;
            // keep the invariant "head of remaining = in flight" instead.
            if let Save::Bulk { remaining, .. } = &mut self.saving {
                remaining.insert(0, (qty, name.clone()));
            }
            ctx.send(add_command(name, qty, None, deck_id));
        }
    }

    // -- Pointer input --------------------------------------------------------

    /// `Some` closes the modal (x, backdrop, cancel, or a finished save).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        window: Rect,
        ctx: &mut ScreenCtx,
    ) -> (Option<AddCardAnswer>, EventResult) {
        let (panel, l) = self.geometry(window);
        let (close, mut result) = self.frame.handle_event(event, panel);
        if close {
            return (Some(AddCardAnswer::Cancelled), result);
        }

        // The selects and the set picker overlay the form while open: they
        // eat the event first.
        if self.deck.is_open() {
            return (None, result.merge(self.deck.handle_event(event, l.deck)));
        }
        if self.list_deck.is_open() {
            return (
                None,
                result.merge(self.list_deck.handle_event(event, l.list_deck)),
            );
        }
        if self.lang.is_open() {
            return (None, result.merge(self.lang.handle_event(event, l.lang)));
        }
        if self.set.is_open() {
            return (
                None,
                result.merge(self.set.handle_event(event, self.set.field_rect(l.set))),
            );
        }

        // Mode tabs.
        let tabs_r = self.tabs.handle_event(event, l.tabs);
        if tabs_r.clicked {
            self.error = None;
            let slot = if self.mode_list() { Slot::List } else { Slot::Name };
            self.set_focus(Some(slot), ctx);
        }
        result = result.merge(tabs_r);

        if !self.mode_list() {
            result = result.merge(self.single_pointer(event, &l, ctx));
        } else if let WidgetEvent::MouseDown { x, y } = *event {
            // List mode has one field and one select.
            if l.list.contains(x, y) {
                self.set_focus(Some(Slot::List), ctx);
                result = result.merge(EventResult::changed());
            } else {
                result = result.merge(self.list_deck.handle_event(event, l.list_deck));
                if !self.list_deck.is_open() {
                    result = result.merge(EventResult::IGNORED);
                }
            }
        }

        // Save / cancel.
        let cancel_r = self.cancel.handle_event(event, l.cancel);
        if cancel_r.clicked {
            return (Some(AddCardAnswer::Cancelled), EventResult::clicked());
        }
        result = result.merge(cancel_r);
        let save_r = self.save.handle_event(event, l.save);
        if save_r.clicked {
            self.save(ctx);
            result = result.merge(EventResult::clicked());
        } else {
            result = result.merge(save_r);
        }
        (None, result)
    }

    fn single_pointer(
        &mut self,
        event: &WidgetEvent,
        l: &Layout,
        ctx: &mut ScreenCtx,
    ) -> EventResult {
        // Name suggestion list (hangs off the name field).
        let name_field = self.name.field_rect(l.name);
        let suggest = self.suggest_rect(name_field);
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hovered = (!self.suggestions.is_empty())
                    .then(|| suggest.contains(x, y))
                    .unwrap_or(false)
                    .then(|| ((y - suggest.y - 4.0) / (SUGGEST_H + 4.0)).floor() as usize)
                    .filter(|i| *i < self.suggestions.len());
                if hovered != self.hover_suggest {
                    self.hover_suggest = hovered;
                    return EventResult::changed();
                }
            }
            WidgetEvent::MouseDown { x, y }
                if !self.suggestions.is_empty() && suggest.contains(x, y) =>
            {
                let i = ((y - suggest.y - 4.0) / (SUGGEST_H + 4.0)).floor() as usize;
                if let Some(card) = self.suggestions.get(i).cloned() {
                    self.name.set_value(&card.name);
                    self.suggestions.clear();
                    self.name_dirty = false;
                    // A picked card scopes the set picker to its printings,
                    // the JS's wireSetPicker(backdrop, "ac-set", "#ac-name").
                    self.set.set_card(Some(card.name));
                    return EventResult::clicked();
                }
            }
            _ => {}
        }

        let WidgetEvent::MouseDown { x, y } = *event else {
            return EventResult::IGNORED;
        };

        // Field clicks: focus + caret.
        for (slot, rect) in [
            (Slot::Name, l.name),
            (Slot::Artist, l.artist),
            (Slot::Qty, l.qty),
            (Slot::Notes, l.notes),
        ] {
            let fr = self.field_mut(slot).field_rect(rect);
            if fr.contains(x, y) {
                self.set_focus(Some(slot), ctx);
                self.field_mut(slot).click(x - fr.x);
                return EventResult::changed();
            }
        }
        let set_field = self.set.field_rect(l.set);
        if set_field.contains(x, y) {
            self.set_focus(Some(Slot::Set), ctx);
            self.set.field.click(x - set_field.x);
            return EventResult::changed();
        }
        // Selects.
        if l.lang.contains(x, y) {
            self.set_focus(None, ctx);
            return self.lang.handle_event(event, l.lang);
        }
        if l.deck.contains(x, y) {
            self.set_focus(None, ctx);
            return self.deck.handle_event(event, l.deck);
        }
        EventResult::IGNORED
    }

    // -- Text input -----------------------------------------------------------

    pub fn handle_text(&mut self, s: &str) -> bool {
        match self.focus {
            Some(Slot::Set) => self.set.handle_text(s).changed,
            Some(slot) => {
                let changed = self.field_mut(slot).handle_text(s);
                if !changed {
                    return false;
                }
                if slot == Slot::Qty {
                    // Digits only, the type="number" of the JS.
                    let clean: String = self
                        .qty
                        .value()
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect();
                    if clean != self.qty.value() {
                        self.qty.set_value(&clean);
                    }
                }
                if slot == Slot::Name {
                    self.name_dirty = true;
                    self.since_name_edit = 0.0;
                    // A name typed by hand drops the picked card's printings
                    // scope, like the JS's rewire on input.
                    self.set.set_card(None);
                }
                true
            }
            None => false,
        }
    }

    pub fn handle_edit_key(&mut self, key: EditKey, ctx: &mut ScreenCtx) -> EventResult {
        match key {
            EditKey::Tab => {
                // The DOM's tab order, kept: single mode walks its five
                // fields, list mode the textarea. Never moves into Selects -
                // arrows suffice for two/three-option dropdowns.
                let order: &[Slot] = if self.mode_list() {
                    &[Slot::List]
                } else {
                    &[Slot::Name, Slot::Set, Slot::Artist, Slot::Qty, Slot::Notes]
                };
                let next = match self.focus {
                    None => order.first().copied(),
                    Some(slot) => {
                        let i = order.iter().position(|s| *s == slot).unwrap_or(0);
                        order.get(i + 1).copied().or(Some(order[0]))
                    }
                };
                self.set_focus(next, ctx);
                EventResult::changed()
            }
            EditKey::Enter => {
                // Enter on the open set list picks a suggestion; elsewhere it
                // saves, the form's submit.
                if self.set.is_open() && self.focus == Some(Slot::Set) {
                    let (consumed, changed) = self.set.handle_edit_key(key);
                    return if consumed && changed {
                        EventResult::clicked()
                    } else {
                        EventResult::changed()
                    };
                }
                self.save(ctx);
                EventResult::clicked()
            }
            _ => match self.focus {
                Some(Slot::Set) => {
                    let (consumed, changed) = self.set.handle_edit_key(key);
                    if consumed {
                        if changed {
                            EventResult::changed()
                        } else {
                            EventResult {
                                handled: true,
                                ..EventResult::IGNORED
                            }
                        }
                    } else {
                        EventResult::IGNORED
                    }
                }
                Some(slot) => {
                    let (consumed, changed) = self.field_mut(slot).handle_edit_key(key);
                    if consumed {
                        if changed {
                            EventResult::changed()
                        } else {
                            EventResult {
                                handled: true,
                                ..EventResult::IGNORED
                            }
                        }
                    } else {
                        EventResult::IGNORED
                    }
                }
                None => EventResult::IGNORED,
            },
        }
    }

    /// Escape unwinds one layer at a time; `false` tells the screen to
    /// close the modal itself.
    pub fn handle_escape(&mut self) -> bool {
        if self.set.handle_escape() {
            return true;
        }
        if !self.suggestions.is_empty() {
            self.suggestions.clear();
            return true;
        }
        if self.deck.is_open() || self.list_deck.is_open() || self.lang.is_open() {
            self.deck.close();
            self.list_deck.close();
            self.lang.close();
            return true;
        }
        false
    }

    /// Blink + debounced searches.
    pub fn tick(&mut self, dt: f32, ctx: &mut ScreenCtx) -> bool {
        self.since_name_edit += dt;
        if self.name_dirty && !self.name_in_flight && self.since_name_edit >= NAME_DEBOUNCE {
            self.name_dirty = false;
            let q = self.name.value().trim().to_string();
            if q.len() >= 2 {
                self.name_in_flight = true;
                ctx.send(Command::SearchCards { q, limit: 6 });
            }
        }
        self.set.tick(dt, ctx);
        self.focus.is_some()
    }

    // -- Render ---------------------------------------------------------------

    pub fn render(&mut self, c: &mut Compositor, layer: LayerId, window: Rect, theme: &Theme) {
        let (panel, l) = self.geometry(window);
        self.frame.render(
            c,
            layer,
            window,
            panel,
            "Adicionar carta",
            theme,
        );
        self.tabs.render(c, l.tabs, theme);

        if !self.mode_list() {
            self.name.render(c, l.name, theme);
            self.render_suggestions(c, panel, l, theme);
            self.set.render(c, l.set, theme);
            self.artist.render(c, l.artist, theme);
            self.render_field_label(c, "Idioma", l.lang, theme);
            self.lang.render(c, l.lang, theme);
            self.qty.render(c, l.qty, theme);
            self.render_deck_row(c, l.deck_label, l.deck, false, theme);
            self.notes.render(c, l.notes, theme);
        } else {
            text(
                c,
                "Nomes das cartas — um por linha *",
                12.0,
                400,
                l.hint.x,
                l.hint.y,
                theme.colors.text_dim.0,
            );
            text(
                c,
                "Cada linha vira tantas unidades quanto o número indicar. Ajuste os detalhes \
                 depois na Coleção.",
                11.0,
                400,
                l.hint.x,
                l.hint.y + 20.0,
                theme.glass.text_placeholder.0,
            );
            self.render_list_area(c, l.list, theme);
            self.render_deck_row(c, l.deck_label, l.list_deck, true, theme);
            if let Some((added, missing)) = &self.list_status {
                let msg = if missing.is_empty() {
                    format!("{added} carta(s) adicionada(s).")
                } else {
                    format!(
                        "{added} adicionada(s). {} não encontrada(s) — corrija e salve de novo:",
                        missing.len()
                    )
                };
                text(
                    c,
                    &msg,
                    12.0,
                    400,
                    l.status.x,
                    l.status.y,
                    if missing.is_empty() {
                        theme.colors.success.0
                    } else {
                        theme.colors.danger.0
                    },
                );
            }
        }

        if let Some(error) = &self.error {
            text(
                c,
                error,
                12.0,
                400,
                l.error.x,
                l.error.y,
                theme.colors.danger.0,
            );
        }
        self.cancel.render(c, l.cancel, theme);
        self.save.render(c, l.save, theme);

        // Floating layers of the controls, above the form.
        self.lang.render_dropdown(c, layer, l.lang, theme);
        self.deck.render_dropdown(c, layer, l.deck, theme);
        self.list_deck.render_dropdown(c, layer, l.list_deck, theme);
        self.set.render_overlay(c, layer, l.set, theme);
    }

    fn render_field_label(&self, c: &mut Compositor, label: &str, rect: Rect, theme: &Theme) {
        text(
            c,
            label,
            12.0,
            400,
            rect.x,
            rect.y - 18.0,
            theme.colors.text_dim.0,
        );
    }

    fn render_deck_row(
        &self,
        c: &mut Compositor,
        label_rect: Rect,
        sel_rect: Rect,
        list: bool,
        theme: &Theme,
    ) {
        text(
            c,
            if list {
                "Alocar todas a um deck (opcional)"
            } else {
                "Alocar a um deck (opcional)"
            },
            12.0,
            400,
            label_rect.x,
            label_rect.y,
            theme.colors.text_dim.0,
        );
        let sel = if list { &self.list_deck } else { &self.deck };
        sel.render(c, sel_rect, theme);
    }

    fn render_list_area(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        if self.list.input.focused {
            c.push(engine::ui::widgets::focus_ring(rect, theme.radius.sm, theme));
        }
        c.push(rounded_rect(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            theme.radius.sm,
            theme.glass.field.0,
        ));
        // Single-line TextInput draws the first line only; the list field
        // trades the caret's pixel mapping for a plain multiline render of
        // its buffer. Typing is unchanged - only the drawing is local.
        let style = TypographyScale::hoff().base_2r();
        let value = self.list.value();
        let shown = if value.is_empty() && !self.list.input.focused {
            self.list.input.placeholder.as_str()
        } else {
            value
        };
        let color = if value.is_empty() && !self.list.input.focused {
            self.list.input.placeholder_color
        } else {
            self.list.input.text_color
        };
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(shown, &style, Some(rect.w - 16.0)),
            x: rect.x + 8.0,
            y: rect.y + 8.0,
            color,
        });
    }

    fn render_suggestions(&self, c: &mut Compositor, panel: Rect, l: Layout, theme: &Theme) {
        if self.suggestions.is_empty() {
            return;
        }
        let _ = panel;
        let field = self.name.field_rect(l.name);
        let list = self.suggest_rect(field);
        let radius = theme.radius.lg;
        c.push(engine::ui::widgets::menu_shadow(list, radius));
        for node in engine::ui::widgets::glass_pill(
            list,
            radius,
            theme.glass.edge_soft.0,
            1.5,
            theme.glass.popover.0,
        ) {
            c.push(node);
        }
        let style = TypographyScale::hoff().base_2sm();
        for (i, card) in self.suggestions.iter().enumerate() {
            let row = Rect::new(
                list.x + 4.0,
                list.y + 4.0 + i as f32 * (SUGGEST_H + 4.0),
                list.w - 8.0,
                SUGGEST_H,
            );
            if self.hover_suggest == Some(i) {
                c.push(rounded_rect(
                    row.x,
                    row.y,
                    row.w,
                    row.h,
                    theme.radius.md,
                    theme.glass.surface_hover.0,
                ));
            }
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&card.name, &style, Some(row.w - 20.0)),
                x: row.x + 10.0,
                y: row.y + (SUGGEST_H - style.line_height) / 2.0,
                    color: with_alpha(
                        theme.colors.text.0,
                        theme.colors.text.0[3] * 0.8,
                    ),
            });
        }
    }
}

/// One bulk-line add: no set, no artist, the deck all lines share.
fn add_command(card_name: String, quantity: i64, oracle_id: Option<String>, deck_id: Option<i64>) -> Command {
    Command::AddCollection(Box::new(CollectionIn {
        card_name,
        set_code: None,
        artist: None,
        lang: "en".into(),
        quantity,
        notes: None,
        deck_id,
        oracle_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bulk grammar: quantities, "1x", bare names, comment lines and
    /// section headers - the shapes Moxfield/Archidekt exports produce and
    /// the JS sent raw to the bulk endpoint.
    #[test]
    fn parses_pasted_list_shapes() {
        let text = "\
// meus comandantes
Commander (1)
1 Kenrith, the Returned King

2 Lightning Bolt
1x Sol Ring (C21) 289
Cultivate
Creatures (2)
2 Zulaport Cutthroat [ZNR] *F*
4x Anel Solar
";
        assert_eq!(
            parse_bulk_list(text),
            vec![
                (1, "Kenrith, the Returned King".to_string()),
                (2, "Lightning Bolt".to_string()),
                (1, "Sol Ring".to_string()),
                (1, "Cultivate".to_string()),
                (2, "Zulaport Cutthroat".to_string()),
                (4, "Anel Solar".to_string()),
            ]
        );
    }

    /// Empty or header-only pastes parse to nothing, which the save path
    /// reports as "Digite ao menos um nome de carta."
    #[test]
    fn junk_only_pastes_are_empty() {
        assert!(parse_bulk_list("").is_empty());
        assert!(parse_bulk_list("\n\n  \n").is_empty());
        assert!(parse_bulk_list("Commander\nCreatures (23)\n# notas\n").is_empty());
    }

    /// A name that starts with a section word ("Deckhand") is still a card.
    #[test]
    fn section_words_inside_real_lines_survive() {
        assert_eq!(
            parse_bulk_list("1 Deckhand\nLand Tax"),
            vec![(1, "Deckhand".to_string()), (1, "Land Tax".to_string())]
        );
    }
}
