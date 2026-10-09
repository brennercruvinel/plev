//! silences domain: a video, the silences found in it with the settings
//! they were found with, what to do with each one, the counts, "apply
//! all", and the instant the screen shows.
//!
//! pure state, no gpu, no window, no ffmpeg. whoever runs the detection
//! feeds it in (`set_detected`); `parse` reads ffmpeg's `silencedetect`
//! stderr into segments. the page opens seeded with the recorded
//! detection of the probe (`probe.stderr`, the real stderr of ffmpeg 9 on
//! the test video of task-69f), so the page, the golden projection and
//! the regression baseline are the same on every machine, with or without
//! ffmpeg; a real detection replaces the seed.
//!
//! a cut's id is minted once per detection and never reused: a request
//! aimed at a cut of a detection that has been replaced names nothing,
//! instead of landing on another silence with the same number.

pub mod parse;

#[cfg(test)]
mod tests;

pub use parse::{Detected, parse};

/// the stderr ffmpeg 9 wrote on the test video in the probe of task-69f
/// (`silencedetect=noise=-30dB:d=0.5`): the seed of the page and the
/// fixture of the parser.
pub const PROBE_STDERR: &str = include_str!("probe.stderr");

/// the file name of the test video the worker generates.
pub const TEST_VIDEO: &str = "silences-test.mp4";

/// a stretch of silence, in seconds from the start of the video.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub start: f64,
    pub end: f64,
}

impl Segment {
    pub fn duration(self) -> f64 {
        (self.end - self.start).max(0.0)
    }

    /// the instant a frame of the silence is taken at.
    pub fn middle(self) -> f64 {
        (self.start + self.end) / 2.0
    }

    pub fn contains(self, t: f64) -> bool {
        self.start <= t && t < self.end
    }
}

/// what to do with a silence: nothing decided yet, cut it, keep it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Decision {
    #[default]
    Pending,
    Apply,
    Reject,
}

impl Decision {
    /// the word the projection and the screen use.
    pub fn name(self) -> &'static str {
        match self {
            Decision::Pending => "pending",
            Decision::Apply => "apply",
            Decision::Reject => "reject",
        }
    }
}

/// a silence the detection found, with what was decided about it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cut {
    id: u64,
    segment: Segment,
    decision: Decision,
}

impl Cut {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn segment(&self) -> Segment {
        self.segment
    }

    pub fn decision(&self) -> Decision {
        self.decision
    }
}

/// the detection settings: the level under which audio counts as silence
/// (dB full scale) and the shortest silence that counts (seconds).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub threshold_db: f32,
    pub min_duration: f32,
}

impl Default for Settings {
    /// the probe's settings, `silencedetect=noise=-30dB:d=0.5`.
    fn default() -> Self {
        Self {
            threshold_db: -30.0,
            min_duration: 0.5,
        }
    }
}

impl Settings {
    /// the thresholds a screen offers: from near-silence to full scale.
    pub const THRESHOLD_RANGE: (f32, f32) = (-60.0, 0.0);
    /// the shortest silences a screen offers, in seconds.
    pub const MIN_DURATION_RANGE: (f32, f32) = (0.1, 3.0);

    pub fn clamped(self) -> Self {
        let (lo, hi) = Self::THRESHOLD_RANGE;
        let (dlo, dhi) = Self::MIN_DURATION_RANGE;
        Self {
            threshold_db: self.threshold_db.clamp(lo, hi),
            min_duration: self.min_duration.clamp(dlo, dhi),
        }
    }
}

/// the video under review: its file name and its length in seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Video {
    pub name: String,
    pub duration: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Counts {
    pub total: usize,
    pub applied: usize,
    pub rejected: usize,
    pub pending: usize,
    /// the seconds the applied cuts take out.
    pub cut_secs: f64,
}

/// what "apply all" did: how many pending cuts it applied, and the counts
/// after.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Applied {
    pub newly: usize,
    pub counts: Counts,
}

#[derive(Clone, Debug)]
pub struct SilencesModel {
    video: Option<Video>,
    /// the settings the cuts were detected with.
    detected_with: Settings,
    cuts: Vec<Cut>,
    next_id: u64,
    /// the instant the screen shows, seconds, within the video.
    shown: f64,
}

impl Default for SilencesModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SilencesModel {
    /// no video, no cuts.
    pub fn new() -> Self {
        Self {
            video: None,
            detected_with: Settings::default(),
            cuts: Vec::new(),
            next_id: 1,
            shown: 0.0,
        }
    }

    /// the probe's detection on the test video: two silences, every one
    /// pending, the instant shown at the start.
    pub fn seeded() -> Self {
        let mut model = Self::new();
        let probe = parse(PROBE_STDERR);
        let video = Video {
            name: TEST_VIDEO.to_owned(),
            duration: probe.duration.unwrap_or(0.0),
        };
        model.set_detected(video, Settings::default(), &probe.segments);
        model
    }

    pub fn video(&self) -> Option<&Video> {
        self.video.as_ref()
    }

    /// the video's length; zero without a video.
    pub fn duration(&self) -> f64 {
        self.video.as_ref().map_or(0.0, |v| v.duration)
    }

    pub fn shown(&self) -> f64 {
        self.shown
    }

    /// shows the instant `t`, kept inside the video. true when it moved.
    pub fn show(&mut self, t: f64) -> bool {
        let t = if t.is_finite() {
            t.clamp(0.0, self.duration())
        } else {
            0.0
        };
        let moved = t != self.shown;
        self.shown = t;
        moved
    }

    /// a detection replaces the cuts: fresh ids, every decision pending,
    /// zero-length segments dropped, the shown instant kept inside the
    /// new video.
    pub fn set_detected(&mut self, video: Video, with: Settings, segments: &[Segment]) {
        self.cuts = segments
            .iter()
            .filter(|s| s.end > s.start)
            .map(|s| {
                let id = self.next_id;
                self.next_id += 1;
                Cut {
                    id,
                    segment: *s,
                    decision: Decision::Pending,
                }
            })
            .collect();
        self.video = Some(video);
        self.detected_with = with.clamped();
        self.show(self.shown);
    }

    pub fn detected_with(&self) -> Settings {
        self.detected_with
    }

    pub fn cuts(&self) -> &[Cut] {
        &self.cuts
    }

    pub fn cut(&self, id: u64) -> Option<&Cut> {
        self.cuts.iter().find(|c| c.id == id)
    }

    /// the cut whose silence holds the instant `t`, when one does.
    pub fn cut_at(&self, t: f64) -> Option<&Cut> {
        self.cuts.iter().find(|c| c.segment.contains(t))
    }

    /// decides `id`. `None` when there is no such cut; otherwise whether
    /// the decision changed.
    pub fn decide(&mut self, id: u64, decision: Decision) -> Option<bool> {
        let cut = self.cuts.iter_mut().find(|c| c.id == id)?;
        let changed = cut.decision != decision;
        cut.decision = decision;
        Some(changed)
    }

    /// every pending cut is applied; what was rejected stays rejected
    /// (someone said so).
    pub fn apply_all(&mut self) -> Applied {
        let mut newly = 0;
        for cut in &mut self.cuts {
            if cut.decision == Decision::Pending {
                cut.decision = Decision::Apply;
                newly += 1;
            }
        }
        Applied {
            newly,
            counts: self.counts(),
        }
    }

    pub fn counts(&self) -> Counts {
        let mut counts = Counts {
            total: self.cuts.len(),
            ..Counts::default()
        };
        for cut in &self.cuts {
            match cut.decision {
                Decision::Pending => counts.pending += 1,
                Decision::Apply => {
                    counts.applied += 1;
                    counts.cut_secs += cut.segment.duration();
                }
                Decision::Reject => counts.rejected += 1,
            }
        }
        counts
    }
}
