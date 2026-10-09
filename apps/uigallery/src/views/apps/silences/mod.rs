//! Apps Demo, Silences: the app of the video editor scenario (task-69
//! phase 4a, the thesis' section 4), the fixed screen the checkpoint b
//! compares the generated one against, and, since phase 4b, the generated
//! one beside it. A person, an agent through the port and a screen reader
//! review the silences ffmpeg finds in a test video: detect them at a
//! threshold and a minimum length, see a frame of each, apply or reject
//! each cut, apply them all.
//!
//! The page is a capability (`semantics.rs`): it projects what it means,
//! runs what it offers in `execute`, and every actor goes through the
//! gallery's one dispatcher (the todo page lends it, as to the Surface
//! page). The model (`uigallery::domain::silences`) changes only inside
//! `execute`. The sliders' drag stays with the widget, as typing stays
//! with a field: `set_value`, `increment` and `decrement` are how an actor
//! without a pointer moves them.
//!
//! ffmpeg runs off the app's thread (`worker.rs`), one child at a time,
//! and only after an actor asks: `detect_silence` (the test video is
//! generated first when it is missing), `preview_segment`, or a generated
//! surface showing a frame the page does not have (`jobs.rs`). The page
//! opens seeded with the recorded detection of the probe, so at rest it
//! is the same on every machine; a real detection replaces the seed, and
//! the frames of the instant shown and of every cut follow. Off the
//! desktop the seed is there and every job says it needs the desktop.
//!
//! Over the fixed screen sits the proposing row of the Surface page
//! (`generated.rs`): an intent, a proposer (the rules, or the Claude off
//! the app's thread), propose, cancel, close. A proposal opens a surface
//! materialized from verified IR over this app's registry, beside the
//! fixed screen, on the same dispatcher: a cut applied on either shows
//! on both. The surface's images draw the frames the page holds
//! (`screen::Frames`); the page decodes what they lack from its wake.
//!
//! Wiring contract for the chrome (view/dispatch.rs, view/input.rs): every
//! entry takes the todo page (the dispatcher's owner) and the inert flag,
//! `wake` settles the worker's answers and the Claude's, typing reaches
//! the intent field and the surface's fields through `handle_text`, Tab
//! and the arrows reach the sliders through `handle_edit_key`, Enter
//! proposes through `handle_enter`, Escape through `handle_escape`.

pub mod fixed_page;
mod generative;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::time::Duration;

use crate::domain::silences::{Decision, Settings, SilencesModel};
use accord::{Receipt, Request, RequestId, Snapshot};
use engine::drawing::gpu::image::ImageHandle;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Button, ButtonSize, EventResult, Rect, Slider, Wake, WidgetEvent};
use screen::{Dispatch, SurfaceHost};

use super::todo::TodoPage;
use super::{EditKey, panel};
use crate::views::text_input::clipboard::Keys;
use fixed_page::layout::{compute, row_parts};
use fixed_page::semantics::{Ids, Target, Which};
use fixed_page::worker::{POLL, Worker};
use generative::{Generated, Halves, page::Page};

/// A cut's row as the pointer meets it: its two buttons, retained so
/// hover and press survive frames.
struct Row {
    id: u64,
    apply: Button,
    reject: Button,
}

/// Where the fixed screen's keyboard focus is (the fields and the
/// surface keep their own): a slider, the detect button, the apply all.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Focus {
    Slider(Which),
    Detect,
    ApplyAll,
}

/// What the last run came to, for the status line.
enum Said {
    /// The seed: the recorded run of the probe.
    Seed,
    Detected {
        took: Duration,
    },
    Failed(String),
}

pub struct SilencesPage {
    model: SilencesModel,
    threshold: Slider,
    min_duration: Slider,
    detect: Button,
    apply_all: Button,
    rows: Vec<Row>,
    hover_cut: Option<u64>,
    hover_mark: Option<u64>,
    /// Keyboard focus on the fixed screen: a slider, or one of its two
    /// buttons.
    focus: Option<Focus>,
    /// The ids the page projects and binds; `None` only if the constant
    /// ids failed to parse (a test proves they do not), which leaves the
    /// page without semantics: every request is then refused.
    ids: Option<Ids>,
    /// Set by the view while an overlay holds the pointer: the page
    /// projects itself inert and takes no input.
    inert: bool,
    worker: Worker,
    /// The `detect_silence` request left open, ended when the worker
    /// answers.
    detecting: Option<RequestId>,
    /// The `preview_segment` requests left open, each with the instant
    /// (ms) whose frame it waits for.
    previews: Vec<(RequestId, u64)>,
    /// The frames decoded, by instant in ms; a failure is kept too, so it
    /// is never asked again in a loop.
    frames: HashMap<u64, Result<ImageHandle, String>>,
    said: Said,
    /// The generated surface: the proposing row, the close, the host.
    generated: Generated,
    /// What each frame took the worker, in the order they came (the
    /// measurement of the ignored test).
    #[cfg(test)]
    frame_took: Vec<Duration>,
}

/// The app as its generated surface reaches it: the fixed screen's
/// projection and the gallery's dispatcher.
struct App<'a> {
    page: &'a mut SilencesPage,
    todo: &'a mut TodoPage,
}

impl Dispatch for App<'_> {
    fn dispatch(&mut self, req: Request) -> Option<Receipt> {
        let id = self.page.dispatch(req, self.todo);
        self.todo.receipt(id).cloned()
    }

    fn project(&self) -> Option<Snapshot> {
        self.page.snapshot().ok()
    }
}

impl SilencesPage {
    pub fn new() -> Self {
        Self::with(Worker::new())
    }

    /// Another worker (the fake, or the real program) for a page in a
    /// test.
    #[cfg(test)]
    fn set_worker(&mut self, worker: Worker) {
        self.worker = worker;
    }

    fn with(worker: Worker) -> Self {
        let ids = Ids::new()
            .inspect_err(|e| log::error!("silences: the page ids do not parse: {e}"))
            .ok();
        let settings = Settings::default();
        let (lo, hi) = Settings::THRESHOLD_RANGE;
        let (dlo, dhi) = Settings::MIN_DURATION_RANGE;
        let mut threshold = Slider::new(lo, hi, settings.threshold_db).step(1.0);
        let mut min_duration = Slider::new(dlo, dhi, settings.min_duration).step(0.1);
        let mut detect = Button::new("detect silences").icon("play");
        let mut apply_all = Button::new("apply all").size(ButtonSize::Sm);
        if let Some(ids) = &ids {
            threshold = threshold.sem(ids.threshold.clone());
            min_duration = min_duration.sem(ids.min_duration.clone());
            detect = detect.sem(ids.detect.clone());
            apply_all = apply_all.sem(ids.apply_all.clone());
        }
        Self {
            model: SilencesModel::seeded(),
            threshold,
            min_duration,
            detect,
            apply_all,
            rows: Vec::new(),
            hover_cut: None,
            hover_mark: None,
            focus: None,
            ids,
            inert: false,
            worker,
            detecting: None,
            previews: Vec::new(),
            frames: HashMap::new(),
            said: Said::Seed,
            generated: Generated::new(),
            #[cfg(test)]
            frame_took: Vec::new(),
        }
    }

    /// The view says whether an overlay holds the pointer. While it does,
    /// the page and its surface are projected inert, so any request comes
    /// back `Inert`, and keys fall through to the chrome.
    pub fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
        self.generated.set_inert(inert);
    }

    /// Where the row, the fixed screen and the surface go in `content`.
    fn halves(&self, content: Rect, theme: &Theme) -> Halves {
        self.generated
            .halves(content, theme, |fixed| compute(self, fixed, theme).height)
    }

    /// Natural height: the row, then the fixed screen alone, or the
    /// taller of the fixed screen and the surface beside it, or both
    /// stacked.
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let h = self.halves(content, theme);
        let fixed = h.fixed.y - content.y + compute(self, h.fixed, theme).height;
        let surface = h.surface.map_or(0.0, |r| {
            r.y - content.y + self.generated.surface_h(r.w, theme)
        });
        fixed.max(surface)
    }

    /// Lends the host to `f` with the page, so a surface can reach the
    /// page's dispatcher and projection while the page holds the host
    /// (the todo page lends its dispatcher the same way).
    fn with_host<R>(&mut self, f: impl FnOnce(&mut SurfaceHost, &mut SilencesPage) -> R) -> R {
        let mut host = std::mem::take(&mut self.generated.host);
        let out = f(&mut host, self);
        self.generated.host = host;
        out
    }

    /// The surface reads the app's state again when it moved.
    fn sync_surface(&mut self) {
        if let Ok(now) = self.snapshot() {
            self.generated.host.sync(&now);
        }
    }

    /// Mirror the cuts into the retained rows: a row per cut, the buttons
    /// reading the decision (the one taken reads dark; the buttons are
    /// the item's affordance, so they bind nothing themselves).
    fn sync_rows(&mut self) {
        let mut rows = Vec::with_capacity(self.model.cuts().len());
        for cut in self.model.cuts() {
            let mut row = match self.rows.iter().position(|r| r.id == cut.id()) {
                Some(at) => self.rows.swap_remove(at),
                None => Row {
                    id: cut.id(),
                    apply: Button::new("apply").size(ButtonSize::Sm),
                    reject: Button::new("reject").size(ButtonSize::Sm),
                },
            };
            row.apply.variant = Self::variant(cut.decision() == Decision::Apply);
            row.reject.variant = Self::variant(cut.decision() == Decision::Reject);
            rows.push(row);
        }
        self.rows = rows;
        let pending = self.model.counts().pending > 0;
        self.apply_all.disabled = !pending;
        self.detect.disabled = self.worker.busy();
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
        todo: &mut TodoPage,
        inert: bool,
    ) -> EventResult {
        self.set_inert(inert);
        let h = self.halves(content, theme);
        let (mut r, row) = self.generated.handle_event(event, &h, theme);
        if row.asked.propose {
            self.propose(todo);
        }
        if row.asked.cancel {
            self.cancel(todo);
        }
        if row.close {
            self.close(todo);
        }
        if let Some(answer) = row.answer {
            self.answer(todo, answer);
        }
        if let Some(pressed) = row.pressed {
            self.press(todo, pressed);
        }
        if row.asked.taken {
            return r;
        }
        r = r.merge(self.handle_fixed_event(event, h.fixed, theme, todo));
        if h.surface.is_some() {
            let surface = self.with_host(|host, page| {
                let mut app = App { page, todo };
                host.iter_mut().fold(EventResult::IGNORED, |r, s| {
                    r.merge(s.handle_event(event, theme, &mut app))
                })
            });
            r = r.merge(surface);
        }
        r
    }

    /// The fixed screen's share of a pointer event, in its rect.
    fn handle_fixed_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
        todo: &mut TodoPage,
    ) -> EventResult {
        self.sync_rows();
        let l = compute(self, content, theme);
        let mut r = EventResult::IGNORED;
        if let WidgetEvent::MouseDown { .. } = event {
            self.set_focus(None);
        }

        // the sliders are the widget's: a drag moves the value with no
        // request, and the status says the settings drifted
        r = r.merge(
            self.threshold
                .handle_event(event, l.threshold.slider, theme),
        );
        r = r.merge(
            self.min_duration
                .handle_event(event, l.min_duration.slider, theme),
        );

        // the buttons of the page
        let detect = self.detect.handle_event(event, l.detect);
        let apply_all = self.apply_all.handle_event(event, l.apply_all);
        r = r.merge(detect).merge(apply_all);

        // the rows: hover, the two buttons, a click elsewhere previews
        let mut asked: Vec<(Target, &'static str)> = Vec::new();
        if detect.clicked {
            asked.push((Target::Detect, "detect_silence"));
        }
        if apply_all.clicked {
            asked.push((Target::ApplyAll, "apply_all"));
        }
        let (mut hover_cut, mut hover_mark) = (self.hover_cut, self.hover_mark);
        if matches!(event, WidgetEvent::MouseMove { .. }) {
            (hover_cut, hover_mark) = (None, None);
        }
        for ((cut, row), at) in self.model.cuts().iter().zip(&mut self.rows).zip(&l.rows) {
            let id = cut.id();
            let badge_w = fixed_page::draw::badge(cut.decision())
                .preferred_size(theme)
                .0;
            let parts = row_parts(*at, theme, &row.apply, &row.reject, badge_w, l.stacked);
            let apply = row.apply.handle_event(event, parts.apply);
            let reject = row.reject.handle_event(event, parts.reject);
            r = r.merge(apply).merge(reject);
            match *event {
                WidgetEvent::MouseMove { x, y } if at.contains(x, y) => hover_cut = Some(id),
                WidgetEvent::MouseDown { x, y }
                    if at.contains(x, y)
                        && !parts.apply.contains(x, y)
                        && !parts.reject.contains(x, y) =>
                {
                    asked.push((Target::Cut(id), "preview_segment"));
                }
                _ => {}
            }
            if apply.clicked {
                asked.push((Target::Cut(id), "apply_cut"));
            }
            if reject.clicked {
                asked.push((Target::Cut(id), "reject_cut"));
            }
        }

        // the band: a mark is the cut's preview
        let duration = self.model.duration();
        for cut in self.model.cuts() {
            let mark = fixed_page::layout::mark_rect(l.timeline, cut.segment(), duration);
            match *event {
                WidgetEvent::MouseMove { x, y } if mark.contains(x, y) => {
                    hover_mark = Some(cut.id())
                }
                WidgetEvent::MouseDown { x, y } if mark.contains(x, y) => {
                    asked.push((Target::Cut(cut.id()), "preview_segment"));
                }
                _ => {}
            }
        }
        if (hover_cut, hover_mark) != (self.hover_cut, self.hover_mark) {
            (self.hover_cut, self.hover_mark) = (hover_cut, hover_mark);
            r.changed = true;
        }
        for (target, action) in asked {
            r = r.merge(self.ask(todo, target, action, None));
        }
        r
    }

    /// A printable key: the intent field or a field of the surface when
    /// it has the keyboard; the fixed screen types nothing.
    pub fn handle_text(&mut self, key: &str, todo: &mut TodoPage) -> bool {
        if self.inert {
            return false;
        }
        if self.generated.proposing.intent.insert(key) {
            return true;
        }
        self.with_host(|host, page| {
            let mut app = App { page, todo };
            host.iter_mut().any(|s| s.handle_text(key, &mut app))
        })
    }

    /// Who has the keys (a chord, the field's menu): the intent field or
    /// a field of the surface; nobody while an overlay holds the page.
    pub(in crate::views) fn keys(&mut self) -> Keys<'_> {
        if self.inert {
            return Keys::Nobody;
        }
        if self.generated.proposing.intent.is_focused() {
            return Keys::Field(&mut self.generated.proposing.intent);
        }
        if self.generated.host.iter().any(|s| s.focused().is_some()) {
            return Keys::Surface;
        }
        Keys::Nobody
    }

    /// Select all in the field of the surface that has the keys.
    pub(in crate::views) fn select_all_in(&mut self, todo: &mut TodoPage) -> bool {
        let all = engine::widgets::form::EditKey::SelectAll;
        self.with_host(|host, page| {
            let mut app = App { page, todo };
            host.iter_mut().any(|s| s.handle_edit_key(all, &mut app))
        })
    }

    /// Editing keys: the intent field or a field of the surface when it
    /// has the keyboard; else Tab moves the keyboard focus over the two
    /// sliders and the arrows nudge the focused one by a step, as a
    /// request (`increment`, `decrement`).
    pub fn handle_edit_key(&mut self, key: EditKey, todo: &mut TodoPage) -> bool {
        if self.inert {
            return false;
        }
        if key != EditKey::Tab {
            if self.generated.proposing.intent.edit(key.into()) {
                return true;
            }
            let typed = self.with_host(|host, page| {
                let mut app = App { page, todo };
                host.iter_mut()
                    .any(|s| s.handle_edit_key(key.into(), &mut app))
            });
            if typed {
                return true;
            }
        }
        match key {
            EditKey::Tab => {
                let next = match self.focus {
                    Some(Focus::Slider(Which::Threshold)) => Which::MinDuration,
                    _ => Which::Threshold,
                };
                self.set_focus(Some(Focus::Slider(next)));
                true
            }
            EditKey::Left | EditKey::Right => {
                let Some(Focus::Slider(which)) = self.focus else {
                    return false;
                };
                let target = match which {
                    Which::Threshold => Target::Threshold,
                    Which::MinDuration => Target::MinDuration,
                };
                let action = if key == EditKey::Left {
                    "decrement"
                } else {
                    "increment"
                };
                self.ask(todo, target, action, None);
                true
            }
            _ => false,
        }
    }

    /// Enter: in the intent field, the person proposes; in a field of
    /// the surface, what that field submits.
    pub fn handle_enter(&mut self, todo: &mut TodoPage, inert: bool) -> bool {
        self.set_inert(inert);
        if inert {
            return false;
        }
        if self.generated.proposing.intent.is_focused() {
            self.propose(todo);
            return true;
        }
        self.with_host(|host, page| {
            let mut app = App { page, todo };
            host.iter_mut().any(|s| s.handle_enter(&mut app))
        })
    }

    /// Escape closes the proposer's panel, blurs the intent field, a
    /// field of the surface, or takes the focus off the sliders; `false`
    /// when nothing had it.
    pub fn handle_escape(&mut self) -> bool {
        if self.inert {
            return false;
        }
        if self.generated.proposing.escape() || self.generated.proposing.blur() {
            return true;
        }
        if self.generated.focus.take().is_some() {
            return true;
        }
        if self.generated.host.iter_mut().any(|s| s.handle_escape()) {
            return true;
        }
        if self.focus.is_none() {
            return false;
        }
        self.set_focus(None);
        true
    }

    /// Whether the page tells a screen reader something of its own: the
    /// accessibility band is open (`reader_states`).
    pub(crate) fn has_reader_states(&self) -> bool {
        self.generated.access.is_open()
    }

    /// What only a screen reader is told (task-75 phase 2a), into `b`,
    /// which holds the page's projection: which preference is on, on the
    /// band's buttons. Never in `projection` or `snapshot`, which the port
    /// and a model's prompt read.
    pub(crate) fn reader_states(&self, b: &mut accord::SnapshotBuilder) {
        let on = self.generated.host.preferences();
        self.generated.access.reader_states(b, on);
    }

    /// The fixed screen's focus goes to `focus`: the sliders and the two
    /// buttons draw their ring by it.
    fn set_focus(&mut self, focus: Option<Focus>) {
        self.focus = focus;
        let on = |f: Focus| focus == Some(f);
        self.threshold
            .set_focused(on(Focus::Slider(Which::Threshold)));
        self.min_duration
            .set_focused(on(Focus::Slider(Which::MinDuration)));
        self.detect.set_focused(on(Focus::Detect));
        self.apply_all.set_focused(on(Focus::ApplyAll));
    }

    /// The worker's answers and the Claude's: while a job runs the page
    /// looks at the channel every `POLL` and asks for a frame when
    /// something arrived; the frames the surface shows and the page lacks
    /// are asked for here, never in a frame; the surface's transitions and
    /// the row's caret ask for their own frames.
    pub fn wake(&mut self, dt: f32, todo: &mut TodoPage) -> Wake {
        let moved = self.settle(todo);
        let answered = Page { app: self }.settle(todo);
        let chained = moved && todo.with_dispatcher(|d, _, now| self.run_chains(d, now));
        if moved || answered || chained {
            self.sync_surface();
            self.refresh_offer();
        }
        let sources = self.generated.host.sources();
        self.want_sources(sources.iter().map(String::as_str));
        let now = if moved || answered {
            Wake::now()
        } else {
            Wake::idle()
        };
        let poll = if self.worker.busy() {
            Wake::after(POLL)
        } else {
            Wake::idle()
        };
        now.merge(poll)
            .merge(self.generated.host.wake(dt))
            .merge(self.generated.proposing.wake(dt))
    }
}
