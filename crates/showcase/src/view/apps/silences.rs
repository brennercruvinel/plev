//! Apps Demo, Silences: the app of the video editor scenario (task-69
//! phase 4a, the thesis' section 4), the fixed screen the checkpoint b
//! compares the generated one against. A person, an agent through the
//! port and a screen reader review the silences ffmpeg finds in a test
//! video: detect them at a threshold and a minimum length, see a frame of
//! each, apply or reject each cut, apply them all.
//!
//! The page is a capability (`semantics.rs`): it projects what it means,
//! runs what it offers in `execute`, and every actor goes through the
//! gallery's one dispatcher (the todo page lends it, as to the Surface
//! page). The model (`showcase::model::silences`) changes only inside
//! `execute`. The sliders' drag stays with the widget, as typing stays
//! with a field: `set_value`, `increment` and `decrement` are how an actor
//! without a pointer moves them.
//!
//! ffmpeg runs off the app's thread (`worker.rs`), one child at a time,
//! and only after an actor asks: `detect_silence` (the test video is
//! generated first when it is missing) or `preview_segment`. The page
//! opens seeded with the recorded detection of the probe, so at rest it is
//! the same on every machine; a real detection replaces the seed, and the
//! frames of the instant shown and of every cut follow. Off the desktop
//! the seed is there and every job says it needs the desktop.
//!
//! Wiring contract for the chrome (view/dispatch.rs, view/input.rs): every
//! entry takes the todo page (the dispatcher's owner) and the inert flag,
//! `wake` settles the worker's answers, Tab and the arrows reach the
//! sliders through `handle_edit_key`, Escape through `handle_escape`.

mod draw;
mod jobs;
mod layout;
mod semantics;
#[cfg(test)]
mod tests;
mod worker;

use std::collections::HashMap;
use std::time::Duration;

use comps::prelude::{Button, ButtonSize, EventResult, Rect, Slider, Wake, WidgetEvent};
use engine::gpu::image::ImageHandle;
use engine::theme::Theme;
use sem::RequestId;
use showcase::model::silences::{Decision, Settings, SilencesModel};

use super::app::TodoPage;
use super::{EditKey, panel};
use layout::{compute, row_parts};
use semantics::{Ids, Target, Which};
use worker::{POLL, Worker};

/// A cut's row as the pointer meets it: its two buttons, retained so
/// hover and press survive frames.
struct Row {
    id: u64,
    apply: Button,
    reject: Button,
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
    /// Keyboard focus, on a slider.
    focus: Option<Which>,
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
    /// What each frame took the worker, in the order they came (the
    /// measurement of the ignored test).
    #[cfg(test)]
    frame_took: Vec<Duration>,
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
            #[cfg(test)]
            frame_took: Vec::new(),
        }
    }

    /// The view says whether an overlay holds the pointer. While it does,
    /// `silences` is projected inert, so any request comes back `Inert`,
    /// and keys fall through to the chrome.
    pub fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    /// Natural height: the three cards, one under the other.
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        compute(self, content, theme).height
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
        self.inert = inert;
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
            let badge_w = draw::badge(cut.decision()).preferred_size(theme).0;
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
            let mark = layout::mark_rect(l.timeline, cut.segment(), duration);
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

    /// Tab moves the keyboard focus over the two sliders; the arrows nudge
    /// the focused one by a step, as a request (`increment`, `decrement`).
    pub fn handle_edit_key(&mut self, key: EditKey, todo: &mut TodoPage) -> bool {
        if self.inert {
            return false;
        }
        match key {
            EditKey::Tab => {
                let next = match self.focus {
                    None => Which::Threshold,
                    Some(Which::Threshold) => Which::MinDuration,
                    Some(Which::MinDuration) => Which::Threshold,
                };
                self.set_focus(Some(next));
                true
            }
            EditKey::Left | EditKey::Right => {
                let Some(which) = self.focus else {
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

    /// Escape takes the focus off the sliders; `false` when none had it.
    pub fn handle_escape(&mut self) -> bool {
        if self.inert || self.focus.is_none() {
            return false;
        }
        self.set_focus(None);
        true
    }

    /// The worker's answers: while a job runs the page looks at the
    /// channel every `POLL` and asks for a frame when something arrived.
    pub fn wake(&mut self, _dt: f32, todo: &mut TodoPage) -> Wake {
        let moved = self.settle(todo);
        let now = if moved { Wake::now() } else { Wake::idle() };
        if self.worker.busy() {
            now.merge(Wake::after(POLL))
        } else {
            now
        }
    }
}
