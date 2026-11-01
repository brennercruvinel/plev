//! The prompt input of the hoff core kit (`components/PromptInput`): a
//! raised box with the line a person writes in, and under it a row of
//! controls: the add-files button and its menu (`AddFiles`), the
//! setting's select, the model menu's trigger and its listbox
//! (`SelectAi`), the voice toggle and the submit button. Collapsed, the
//! line hides until voice input shows it; with premade prompts, a panel
//! of suggestions sits over the box (`PremadePrompt`); with the toolbar,
//! the view controller (front G's) sits centered over it.
//!
//! Sized for ranges, never for one screen: the box is as wide as its
//! column up to the kit's `w-135.5` (`prompt.max_w`), and as tall as its
//! parts; where the column is too narrow for the two selects beside the
//! square buttons, they take a row of their own under them, side by side.
//! A label wider than its control is ellipsized. The menus open outside
//! the box, the add-files one above its button and the model's under its
//! trigger, and [`PromptInput::room`] says how much room they and the
//! premade panel need, so the owner keeps it clear.
//!
//! Behavior apart from the look: the line edits as a text field does
//! ([`ComposeLine`]); the add button opens its menu, whose rows add a file
//! (the gallery stands in for the picker: it counts them); the trigger
//! opens the model listbox, whose rows choose a model; voice input dims
//! and leaves out what it does not use, asks something else, and marks
//! the box's corner; submit sends the line (it keeps what was sent and
//! clears). Every visual value is `theme.message.prompt`; the buttons are
//! the kit's buttons (`theme.button`), the setting's select the kit's
//! Select (`theme.form.select`).
//!
//! The parts take `<id>.text`, `.add` (its menu `.files[<i>]`),
//! `.setting`, `.model` (its options `.model[<i>]`), `.voice`, `.submit`,
//! and over the box `.premade[<i>]`, `.surprise`, `.categories[<i>]`,
//! `.view` (its arrows `.view.back` and `.view.forward`);
//! [`PromptInput::project`] declares them and [`PromptInput::act`] runs
//! what a reader or an agent asks of them.

mod events;
mod premade;
mod project;
mod render;

use accord::SemId;

pub use premade::PremadeLayout;
pub use project::Part;

use super::compose::{ComposeLine, LineLook};
use super::select_ai::SelectAi;
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{IconButton, IconButtonSize, IconButtonVariant, Press, SubmitButton};
use crate::widgets::form::Select;
use crate::widgets::form::tokens::SelectLook;
use crate::widgets::nav::ViewController;
use crate::widgets::wake::Wake;
use crate::widgets::widget::Rect;
use premade::Premade;

/// The add-files menu's rows, their icon and their label
/// (`PromptInput/AddFiles/index.tsx:8-26`).
pub const FILES: [(&str, &str); 3] = [
    ("image-plus", "Add photos or videos"),
    ("box", "Add 3D objects"),
    ("file-plus", "Add files (docs, txt...)"),
];

/// The setting's options (`PromptInput/index.tsx:10-13`).
pub const SETTINGS: [&str; 2] = ["Inspiration", "Describe"];

/// What the line asks, at rest and under voice input (`:50-54`).
pub const PLACEHOLDER: &str = "Describe your 3D object or scene...";
pub const VOICE_PLACEHOLDER: &str = "Ask me anything...";

/// Where every part lands, from the box's rect alone.
#[derive(Clone, Debug, PartialEq)]
pub struct PromptLayout {
    pub frame: Rect,
    /// The line, unless collapsed.
    pub text: Option<Rect>,
    pub add: Rect,
    pub setting: Rect,
    pub model: Rect,
    pub voice: Rect,
    pub submit: Rect,
    /// The recording mark, under voice input.
    pub mark: Option<Rect>,
    /// The add-files menu and its rows, open.
    pub files: Option<(Rect, Vec<Rect>)>,
    /// The model listbox, open.
    pub models: Option<Rect>,
    pub premade: Option<PremadeLayout>,
    /// The view controller over the box.
    pub view: Option<Rect>,
}

#[derive(Clone, Debug)]
pub struct PromptInput {
    line: ComposeLine,
    add: IconButton,
    files_open: bool,
    files_hover: Option<usize>,
    files_pressed: Option<usize>,
    files_focus: Option<usize>,
    /// Files added through the menu.
    added: usize,
    setting: Select,
    model: SelectAi,
    model_open: bool,
    model_press: Press,
    voice_button: IconButton,
    voice: bool,
    /// The kit's SubmitButton at its default step.
    submit: SubmitButton,
    /// The submit button has the keyboard (it keeps no flag to read).
    submit_focused: bool,
    collapsed: bool,
    premade: Option<Premade>,
    view: Option<ViewController>,
    hovered: bool,
    /// Shows the hover at rest (a showcase card of the state).
    held_hover: bool,
    /// What the last submit sent.
    sent: Option<String>,
    sem: Option<SemId>,
}

impl Default for PromptInput {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptInput {
    pub fn new() -> Self {
        Self {
            line: ComposeLine::new(PLACEHOLDER),
            add: IconButton::new("plus")
                .variant(IconButtonVariant::Outline)
                .size(IconButtonSize::Lg),
            files_open: false,
            files_hover: None,
            files_pressed: None,
            files_focus: None,
            added: 0,
            setting: Select::new(SETTINGS, 0).look(SelectLook::Ghost).icon("zap"),
            model: SelectAi::kit().selected(0).anchored(true),
            model_open: false,
            model_press: Press::default(),
            voice_button: IconButton::new("mic")
                .variant(IconButtonVariant::Ghost)
                .size(IconButtonSize::Lg),
            voice: false,
            submit: SubmitButton::new(),
            submit_focused: false,
            collapsed: false,
            premade: None,
            view: None,
            hovered: false,
            held_hover: false,
            sent: None,
            sem: None,
        }
    }

    /// The line hidden until voice input shows it (`collapsed`).
    pub fn collapsed(mut self, on: bool) -> Self {
        self.collapsed = on;
        self
    }

    /// The premade prompts over the box (`premadePrompt`).
    pub fn premade(mut self, on: bool) -> Self {
        self.premade = on.then(Premade::kit);
        self
    }

    /// The view controller over the box (`isViewController`).
    pub fn view_controller(mut self, on: bool) -> Self {
        self.view = on.then(|| ViewController::new(false));
        self
    }

    /// The hover shown at rest.
    pub fn held_hover(mut self, on: bool) -> Self {
        self.held_hover = on;
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.line = self.line.with_text(text);
        self
    }

    /// Voice input on.
    pub fn voice(mut self, on: bool) -> Self {
        self.set_voice(on);
        self
    }

    /// The add-files menu open.
    pub fn files_open(mut self, on: bool) -> Self {
        self.files_open = on;
        self
    }

    /// The model listbox open.
    pub fn models_open(mut self, on: bool) -> Self {
        self.model_open = on;
        self
    }

    /// The prompt's id; its parts take theirs under it.
    pub fn sem(mut self, id: SemId) -> Self {
        let child = |name: &str| id.child(name).ok();
        if let Some(c) = child("text") {
            self.line = self.line.sem(c);
        }
        if let Some(c) = child("add") {
            self.add = self.add.sem(c);
        }
        if let Some(c) = child("setting") {
            self.setting = self.setting.sem(c);
        }
        if let Some(c) = child("model") {
            self.model = self.model.sem(c);
        }
        if let Some(c) = child("voice") {
            self.voice_button = self.voice_button.sem(c);
        }
        if let Some(c) = child("submit") {
            self.submit = self.submit.sem(c);
        }
        if let (Some(v), Some(c)) = (&mut self.view, child("view")) {
            *v = v.clone().sem(c);
        }
        if let Some(p) = &mut self.premade {
            if let Some(c) = child("surprise") {
                p.surprise = p.surprise.clone().sem(c);
            }
            for (i, b) in p.categories.iter_mut().enumerate() {
                if let Some(c) = child("categories").map(|f| f.key(i.to_string())) {
                    *b = b.clone().sem(c);
                }
            }
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn text(&self) -> &str {
        self.line.text()
    }

    pub fn sent(&self) -> Option<&str> {
        self.sent.as_deref()
    }

    pub fn added(&self) -> usize {
        self.added
    }

    pub fn is_voice(&self) -> bool {
        self.voice
    }

    pub fn is_files_open(&self) -> bool {
        self.files_open
    }

    pub fn is_models_open(&self) -> bool {
        self.model_open
    }

    pub fn model(&self) -> Option<&str> {
        self.model.chosen_title()
    }

    pub fn setting(&self) -> Option<&str> {
        self.setting.selected_label()
    }

    /// The line shows: not collapsed, or under voice input, or holding
    /// text (a premade prompt picked opens a collapsed box).
    fn shows_line(&self) -> bool {
        self.voice || !self.collapsed || !self.line.is_empty()
    }

    /// Voice input on or off: the line, the add button and both menus
    /// left out while on, and the line asks something else.
    pub fn set_voice(&mut self, on: bool) {
        self.voice = on;
        self.voice_button.icon = if on { "x" } else { "mic" };
        self.line.set_disabled(on);
        self.line
            .set_placeholder(if on { VOICE_PLACEHOLDER } else { PLACEHOLDER });
        self.add.disabled = on;
        self.setting.disabled = on;
        if on {
            self.setting.close();
            self.files_open = false;
            self.model_open = false;
        }
    }

    pub fn toggle_voice(&mut self) {
        self.set_voice(!self.voice);
    }

    /// Sends the line: kept as sent, and cleared. `false` with nothing to
    /// send.
    pub fn submit(&mut self) -> bool {
        if self.line.is_empty() {
            return false;
        }
        self.sent = Some(self.line.text().to_owned());
        self.line.clear();
        true
    }

    /// Opens or closes the add-files menu (the model's closes).
    pub fn toggle_files(&mut self) -> bool {
        if self.voice {
            return false;
        }
        self.files_open = !self.files_open;
        self.model_open = false;
        self.files_hover = None;
        self.files_focus = None;
        true
    }

    /// Picks the add-files row `i`: a file added, the menu closed.
    pub fn add_file(&mut self, i: usize) -> bool {
        if i >= FILES.len() || self.voice {
            return false;
        }
        self.added += 1;
        self.files_open = false;
        if self.files_focus.take().is_some() {
            self.add.set_focused(true);
        }
        true
    }

    pub fn set_models_open(&mut self, on: bool) -> bool {
        if self.voice && on {
            return false;
        }
        self.model_open = on;
        self.files_open = false;
        self.model.reset_pointer();
        if !on {
            self.model.set_focused(None);
        }
        true
    }

    /// Chooses the model at `i` and closes the listbox.
    pub fn choose_model(&mut self, i: usize) -> bool {
        if self.voice || !self.model.choose(i) {
            return false;
        }
        let had = self.model.focused().is_some();
        self.set_models_open(false);
        self.model_press.focused |= had;
        true
    }

    /// Puts a premade suggestion into the line.
    pub fn use_prompt(&mut self, i: usize) -> bool {
        let Some(p) = self
            .premade
            .as_ref()
            .and_then(|p| p.prompts.get(i))
            .cloned()
        else {
            return false;
        };
        self.line.set_value(&p)
    }

    /// `Surprise me`: the next suggestion into the line.
    pub fn surprise(&mut self) -> bool {
        match self.premade.as_mut().and_then(Premade::surprise_me) {
            Some(p) => self.line.set_value(&p),
            None => false,
        }
    }

    /// Closes what is open, the topmost first; the keyboard on one of its
    /// rows goes back to what opened it. `true` when something was open.
    pub fn close_menus(&mut self) -> bool {
        if self.setting.is_open() {
            self.setting.close();
            return true;
        }
        if self.model_open {
            let had = self.model.focused().is_some();
            self.set_models_open(false);
            self.model_press.focused |= had;
            return true;
        }
        if self.files_open {
            self.files_open = false;
            if self.files_focus.take().is_some() {
                self.add.set_focused(true);
            }
            return true;
        }
        false
    }

    /// The line, for the keys a page routes to it while it is focused.
    pub fn line(&self) -> &ComposeLine {
        &self.line
    }

    pub fn line_mut(&mut self) -> &mut ComposeLine {
        &mut self.line
    }

    /// Where an input method puts its candidate window: the line's caret,
    /// in a box at `frame`. `None` when the line does not show.
    pub fn ime_area(&self, frame: Rect, theme: &Theme) -> Option<Rect> {
        let text = self.layout(frame, theme).text?;
        Some(self.line.ime_area(text, &self.line_look(theme), theme))
    }

    /// The caret's blink while the line is focused.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.line.wake(dt)
    }

    fn line_look(&self, theme: &Theme) -> LineLook {
        let t = &theme.message.prompt;
        LineLook {
            text: t.text,
            placeholder: t.placeholder,
            alpha: 1.0,
        }
    }

    /// The trigger's natural width: its padding, the model's name, the
    /// chevron.
    fn trigger_w(&self, theme: &Theme) -> f32 {
        let t = &theme.message.prompt.trigger;
        let style = t.text.style(&theme.typography);
        let name = self.model.chosen_title().unwrap_or_default();
        let (w, _) = TextMeasurer::measure_styled(name, &style, None);
        (t.pad_x * 2.0 + w.ceil() + t.gap + t.chevron).max(t.min_w)
    }

    /// The narrowest box whose square buttons still fit in a row.
    pub fn min_width(&self, theme: &Theme) -> f32 {
        let t = &theme.message.prompt;
        (t.border_width + t.pad) * 2.0 + t.control * 3.0 + t.row_gap * 2.0
    }

    /// Whether the selects share the square buttons' row in a box `w`
    /// wide.
    fn one_row(&self, w: f32, theme: &Theme) -> bool {
        let t = &theme.message.prompt;
        let inner = w - (t.border_width + t.pad) * 2.0;
        let free = inner - t.control * 3.0 - t.row_gap * 4.0;
        free >= t.setting_min + t.trigger.min_w
    }

    /// The box's size in a column `avail` wide.
    pub fn size(&self, avail: f32, theme: &Theme) -> (f32, f32) {
        let t = &theme.message.prompt;
        let w = avail.min(t.max_w).max(0.0);
        let line = if self.shows_line() {
            t.text_top + t.text.line_height + t.text_bottom
        } else {
            0.0
        };
        let rows = if self.one_row(w, theme) {
            t.control
        } else {
            t.control * 2.0 + t.row_gap
        };
        (w, (t.border_width + t.pad) * 2.0 + line + rows)
    }

    /// The room over and under a box `frame` the menus and the premade
    /// panel take, `(above, below)`, open or not, so opening one moves
    /// nothing.
    pub fn room(&self, frame: Rect, theme: &Theme) -> (f32, f32) {
        let t = &theme.message.prompt;
        let files = self.files_panel_h(theme) + t.panel.gap;
        let premade = self.premade.as_ref().map_or(0.0, |p| p.room(frame, theme));
        let l = self.layout(frame, theme);
        let below_row = frame.bottom() - l.model.bottom();
        let models = (self.model.size(theme).1 + t.panel.gap - below_row).max(0.0);
        let setting = self.setting.dropdown_rect(l.setting, theme);
        let setting = (setting.bottom() - frame.bottom()).max(0.0);
        let view = self
            .view
            .as_ref()
            .map_or(0.0, |v| v.preferred_size(theme).1 + t.view_gap);
        (files.max(premade).max(view), models.max(setting))
    }

    fn files_panel_h(&self, theme: &Theme) -> f32 {
        let p = &theme.message.prompt;
        (p.panel.border_width + p.panel.pad) * 2.0 + p.files.h * FILES.len() as f32
    }

    /// Where every part lands in `frame` (the box's rect).
    pub fn layout(&self, frame: Rect, theme: &Theme) -> PromptLayout {
        let t = &theme.message.prompt;
        let inner = frame.inset(t.border_width + t.pad);
        let (c, gap) = (t.control, t.row_gap);
        let text = self.shows_line().then(|| {
            let room = if self.voice { t.voice.text_room } else { 0.0 };
            Rect::new(
                inner.x + t.text_inset,
                inner.y + t.text_top,
                (inner.w - t.text_inset - room).max(0.0),
                t.text.line_height,
            )
        });
        let row_y = text.map_or(inner.y, |r| r.bottom() + t.text_bottom);
        let add = Rect::new(inner.x, row_y, c, c);
        let submit = Rect::new(inner.right() - c, row_y, c, c);
        let voice = Rect::new(submit.x - gap - c, row_y, c, c);
        let (setting, model) = if self.one_row(frame.w, theme) {
            let free = voice.x - gap - (add.right() + gap) - gap;
            let natural = self.setting.width_in(theme).max(t.setting_min);
            let model_w = self.trigger_w(theme).min(free - t.setting_min);
            let setting_w = natural.min(free - model_w);
            (
                Rect::new(add.right() + gap, row_y, setting_w, c),
                Rect::new(voice.x - gap - model_w, row_y, model_w, c),
            )
        } else {
            let y = row_y + c + gap;
            let w = ((inner.w - gap) / 2.0).max(0.0);
            (
                Rect::new(inner.x, y, w, c),
                Rect::new(inner.right() - w, y, w, c),
            )
        };
        let mark = self.voice.then(|| {
            let v = &t.voice;
            Rect::new(
                frame.right() - v.inset - v.ring,
                frame.y + v.inset,
                v.ring,
                v.ring,
            )
        });
        let files = self.files_open.then(|| {
            // as wide as the kit's, or the box where the box is narrower
            let (w, h) = (t.panel.w.min(frame.w), self.files_panel_h(theme));
            let x = add.x.min(frame.right() - w).max(frame.x);
            let panel = Rect::new(x, add.y - t.panel.gap - h, w, h);
            let rows_in = panel.inset(t.panel.border_width + t.panel.pad);
            let rows = (0..FILES.len())
                .map(|i| {
                    Rect::new(
                        rows_in.x,
                        rows_in.y + i as f32 * t.files.h,
                        rows_in.w,
                        t.files.h,
                    )
                })
                .collect();
            (panel, rows)
        });
        let models = self.model_open.then(|| {
            let (w, h) = self.model.size(theme);
            let w = w.min(frame.w);
            let x = (model.x + (model.w - w) / 2.0)
                .min(frame.right() - w)
                .max(frame.x);
            Rect::new(x, model.bottom() + t.panel.gap, w, h)
        });
        let premade = self
            .premade
            .as_ref()
            .filter(|_| !self.voice)
            .map(|p| p.layout(frame, theme));
        let view = self.view.as_ref().map(|v| {
            let (w, h) = v.preferred_size(theme);
            let x = (frame.x + (frame.w - w) / 2.0).max(frame.x);
            Rect::new(x, frame.y - t.view_gap - h, w, h)
        });
        PromptLayout {
            view,
            frame,
            text,
            add,
            setting,
            model,
            voice,
            submit,
            mark,
            files,
            models,
            premade,
        }
    }
}
