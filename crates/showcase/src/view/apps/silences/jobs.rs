//! The worker's answers settled into the page (task-69 phases 4a and 4b):
//! a detection replaces the cuts and ends its request, a frame goes into
//! the cache and ends the previews waiting for it, and the frames the
//! screen and the generated surface show are asked for, one job at a
//! time. The frame of an instant is named by a frame ref, the text the
//! projection carries for every frame it offers (`silences.video:original`,
//! `silences.video:result`, `silences.cut[<id>]:frame`): the media node's
//! id and the instant in seconds, `silences.video@4.017052`. A generated
//! surface binds an `image` to one of those and the host asks the page
//! for it through `surface::Frames`; the page decodes what it does not
//! have in `wake`, never in a frame.
//!
//! While the app sleeps (`engine::window::Background`, task-76 item 7)
//! `settle` reads nothing: an answer that arrives stays on the worker's
//! channel and is applied on the first wake after the app wakes, when the
//! shell draws the frame that shows it. A job asked while asleep still
//! starts today: the detect and the preview submit from the page's
//! `execute` (`semantics.rs`), outside this file.

use std::collections::HashMap;

use engine::gpu::image::ImageHandle;
use engine::window::Background;
use sem::{SemId, Value};
use showcase::model::silences::{Decision, TEST_VIDEO, Video};
use surface::Frames;

use super::semantics::{count, micros, ms};
use super::worker::{Done, Job};
use super::{Said, SilencesPage};
use crate::view::apps::app::TodoPage;

/// The frame ref of the instant `t` of the media node `video`.
pub(super) fn frame_ref(video: &SemId, t: f64) -> String {
    format!("{video}@{}", micros(t))
}

/// The instant (ms) a frame ref of `video` names; `None` for a text that
/// is not one, or that names another node.
pub(super) fn instant_of(video: &SemId, source: &str) -> Option<u64> {
    let (node, at) = source.rsplit_once('@')?;
    if node != video.as_str() {
        return None;
    }
    let secs: f64 = at.parse().ok()?;
    secs.is_finite().then(|| ms(secs))
}

/// The page's frames, as a generated surface reads them: the cache,
/// looked up by the instant a frame ref names. Nothing is decoded here.
pub(super) struct PageFrames<'a> {
    pub video: &'a SemId,
    pub frames: &'a HashMap<u64, Result<ImageHandle, String>>,
}

impl Frames for PageFrames<'_> {
    fn frame(&self, source: &str) -> Option<ImageHandle> {
        let at = instant_of(self.video, source)?;
        match self.frames.get(&at) {
            Some(Ok(handle)) => Some(*handle),
            _ => None,
        }
    }
}

impl SilencesPage {
    /// The frame refs `sources` name that the page does not have yet are
    /// asked for: what the generated surface's images show, looked at
    /// from `wake`.
    pub(super) fn want_sources<'a>(&mut self, sources: impl IntoIterator<Item = &'a str>) {
        let Some(video) = self.ids.as_ref().map(|ids| ids.video.clone()) else {
            return;
        };
        let wanted: Vec<u64> = sources
            .into_iter()
            .filter_map(|s| instant_of(&video, s))
            .collect();
        self.want(wanted);
    }

    /// The instant of the result (the video with the applied cuts) at the
    /// instant shown: inside an applied cut, the cut's end (the first
    /// frame that stays); elsewhere, the instant shown itself.
    pub(super) fn result_instant(&self) -> f64 {
        let shown = self.model.shown();
        match self.model.cut_at(shown) {
            Some(cut) if cut.decision() == Decision::Apply => {
                cut.segment().end.min(self.model.duration())
            }
            _ => shown,
        }
    }

    /// What the worker sent since the page last looked: a detection
    /// replaces the cuts and ends its request; a frame goes into the cache,
    /// ends the previews waiting for it and asks for the frames still
    /// missing. True when what the page shows moved. Nothing while the
    /// app sleeps: the channel keeps it for the wake.
    pub(super) fn settle(&mut self, todo: &mut TodoPage) -> bool {
        if Background::asleep() {
            return false;
        }
        let done = self.worker.poll();
        let moved = !done.is_empty();
        for d in done {
            match d {
                Done::Detected { with, result, took } => {
                    let id = self.detecting.take();
                    let ended = match result {
                        Ok(detected) => {
                            let video = Video {
                                name: TEST_VIDEO.to_owned(),
                                duration: detected.duration.unwrap_or(0.0),
                            };
                            self.model.set_detected(video, with, &detected.segments);
                            let cuts = self.model.cuts().len();
                            self.said = Said::Detected { took };
                            self.want_frames();
                            let mut outcome = sem::Outcome::default();
                            outcome.result.insert("cuts".to_owned(), count(cuts));
                            outcome
                                .result
                                .insert("seconds".to_owned(), Value::Num(took.as_secs_f64()));
                            id.map(|id| {
                                todo.with_dispatcher(|d, _, now| d.complete(id, outcome, now))
                            })
                        }
                        Err(why) => {
                            self.said = Said::Failed(why.clone());
                            id.map(|id| todo.with_dispatcher(|d, _, now| d.fail(id, why, now)))
                        }
                    };
                    if let Some(Err(e)) = ended {
                        log::warn!("silences: the detection's request did not end: {e}");
                    }
                }
                Done::Frame { ms, result, took } => {
                    log::debug!("silences: the frame at {ms} ms took {took:?}");
                    #[cfg(test)]
                    self.frame_took.push(took);
                    if let Err(why) = &result {
                        self.said = Said::Failed(why.clone());
                    }
                    let (ready, rest): (Vec<_>, Vec<_>) =
                        self.previews.drain(..).partition(|(_, at)| *at == ms);
                    self.previews = rest;
                    for (id, _) in ready {
                        let ended = todo.with_dispatcher(|d, _, now| match &result {
                            Ok(_) => {
                                let mut outcome = sem::Outcome::default();
                                outcome
                                    .result
                                    .insert("shown".to_owned(), Value::Num(ms as f64 / 1000.0));
                                d.complete(id, outcome, now)
                            }
                            Err(why) => d.fail(id, why.clone(), now),
                        });
                        if let Err(e) = ended {
                            log::warn!("silences: the preview's request did not end: {e}");
                        }
                    }
                    let ok = result.is_ok();
                    self.frames.insert(ms, result);
                    if ok {
                        self.want_frames();
                    }
                }
            }
        }
        moved
    }

    /// The frames the screen shows and does not have yet: the instant
    /// shown, the result's instant when it differs, and the middle of each
    /// cut, one job each, in that order.
    pub(super) fn want_frames(&mut self) {
        let mut wanted = vec![ms(self.model.shown()), ms(self.result_instant())];
        wanted.extend(self.model.cuts().iter().map(|c| ms(c.segment().middle())));
        self.want(wanted);
    }

    /// Queues a decode for each instant of `wanted` the cache lacks (a
    /// failure is kept, so it is never asked again in a loop).
    pub(super) fn want(&mut self, wanted: impl IntoIterator<Item = u64>) {
        for at in wanted {
            if !self.frames.contains_key(&at) {
                self.worker.submit(Job::Frame { ms: at });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sem::{Actor, Request, Stage};
    use showcase::model::silences::{PROBE_STDERR, parse};

    use super::super::worker::Worker;
    use super::*;

    /// One turn of a 60 Hz display.
    const TURN: f32 = 1.0 / 60.0;

    /// A detection's answer that arrives while the app sleeps stays on
    /// the channel: the wake reads nothing and asks for no frame, the
    /// request stays open; the first wake after the app wakes applies it
    /// and asks for the frame that shows it.
    #[test]
    fn an_answer_that_arrives_while_the_app_sleeps_is_applied_on_the_first_awake_wake() {
        let mut page = SilencesPage::with(Worker::fake(parse(PROBE_STDERR), true));
        let mut todo = TodoPage::new();
        let seen = page.snapshot().expect("the page projects");
        let detect = SemId::parse("silences.detect").unwrap();
        let req = Request::observe(&seen, Actor::Human, &detect, "detect_silence", None).unwrap();
        let id = page.dispatch(req, &mut todo);
        let stage = |todo: &TodoPage| todo.receipt(id).unwrap().stage().unwrap().clone();
        assert_eq!(stage(&todo), Stage::Executing);

        Background::sleeping(true);
        page.worker.release();
        let wake = page.wake(TURN, &mut todo);
        assert!(!wake.now, "asleep: nothing applied, no frame");
        assert_eq!(stage(&todo), Stage::Executing, "the request stays open");
        assert!(page.worker.busy(), "the answer is still on the channel");

        Background::sleeping(false);
        let wake = page.wake(TURN, &mut todo);
        assert!(wake.now, "awake: applied, a frame");
        let Stage::Completed(outcome) = stage(&todo) else {
            panic!("the detection completes: {:?}", stage(&todo));
        };
        assert_eq!(outcome.result.get("cuts"), Some(&Value::Int(2)));
    }

    #[test]
    fn a_frame_ref_names_the_media_node_and_the_instant() {
        let video = SemId::parse("silences.video").unwrap();
        assert_eq!(frame_ref(&video, 4.017052), "silences.video@4.017052");
        assert_eq!(frame_ref(&video, 0.0), "silences.video@0");
        assert_eq!(instant_of(&video, "silences.video@4.017052"), Some(4017));
        assert_eq!(instant_of(&video, "silences.video@0"), Some(0));
        assert_eq!(instant_of(&video, "silences.other@1"), None);
        assert_eq!(instant_of(&video, "silences.video@x"), None);
        assert_eq!(instant_of(&video, "silences-test.mp4"), None);
    }
}
