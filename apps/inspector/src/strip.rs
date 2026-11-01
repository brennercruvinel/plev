//! the timeline strip: the events of the session by kind and by actor,
//! and the scrub. it shows `Inspector::timeline` as it is: one lane per
//! actor, in the order they first appear, and one for the state the app
//! projected (snapshots and changes, which no actor makes); each event a
//! mark at its time, in the tone of its kind; one chip per kind with its
//! count. under the lanes, the scrub (comps `Slider`) over the same span,
//! so a mark and the knob at its time line up.

use accord::{Actor, Millis};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::ControlSize;
use engine::styling::theme::TypeRole;
use engine::widgets::buttons::Chip;
use engine::widgets::form::Slider;
use engine::widgets::recipe::rounded_rect;
use engine::widgets::{Intent, Rect, Theme};
use replay::{Tally, Timeline, What};

/// what an event mark says, by its kind (and a stage by how it ended).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Observed,
    /// a snapshot or changes: the state the app projected.
    State,
    Stage,
    /// a stage that refused or failed.
    Refused,
    /// a stage that completed.
    Done,
    Input,
}

impl Tone {
    /// its color, from the theme.
    pub fn color(self, theme: &Theme) -> [f32; 4] {
        match self {
            Tone::Observed => theme.colors.info.0,
            Tone::State => theme.colors.text_dim.0,
            Tone::Stage | Tone::Input => theme.colors.text_mid.0,
            Tone::Refused => theme.colors.danger.0,
            Tone::Done => theme.colors.success.0,
        }
    }
}

/// one event on its lane: its index in the session, its time from the
/// start, and its tone.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub lane: usize,
    pub index: usize,
    pub at: Millis,
    pub tone: Tone,
}

/// where the strip's parts go in a rect.
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub chips: Vec<Rect>,
    /// each lane's row, the whole width.
    pub lanes: Vec<Rect>,
    /// where the marks go across the lanes: the knob's travel.
    pub track: Rect,
    pub scrub: Rect,
}

/// the timeline strip of one session.
#[derive(Debug)]
pub struct Strip {
    /// each lane's label: the actor with what it did, or the app's state.
    pub lanes: Vec<String>,
    pub marks: Vec<Mark>,
    pub chips: Vec<Chip>,
    /// the session's length, in ms.
    pub span: Millis,
    /// every instant an event was recorded at, from the start, in order,
    /// once each: where the keys step.
    pub instants: Vec<Millis>,
    pub scrub: Slider,
}

impl Strip {
    pub fn new(timeline: &Timeline) -> Strip {
        let span = timeline.end.saturating_sub(timeline.start);
        let mut lanes: Vec<String> = timeline.actors.iter().map(tally).collect();
        let state = timeline.actors.len();
        let lane_of = |actor: &Actor| timeline.actors.iter().position(|t| t.actor == *actor);
        let mut marks = Vec::new();
        for entry in &timeline.entries {
            let (lane, tone) = match &entry.what {
                What::Observed { actor, .. } => (lane_of(actor), Tone::Observed),
                What::Snapshot { .. } | What::Changes { .. } => (Some(state), Tone::State),
                What::Stage { actor, stage, .. } => (lane_of(actor), stage_tone(stage)),
                What::Input { .. } => (lane_of(&Actor::Human), Tone::Input),
            };
            if let Some(lane) = lane {
                marks.push(Mark {
                    lane,
                    index: entry.index,
                    at: entry.at,
                    tone,
                });
            }
        }
        if marks.iter().any(|m| m.lane == state) {
            lanes.push("the app's state".to_owned());
        }
        let chips = timeline
            .kinds
            .iter()
            .map(|k| {
                let chip = Chip::new(format!("{} {}", k.kind, k.count));
                match k.kind {
                    "observed" => chip.intent(Intent::Informational),
                    _ => chip,
                }
            })
            .collect();
        let mut instants: Vec<Millis> = timeline.entries.iter().map(|e| e.at).collect();
        instants.dedup();
        Strip {
            lanes,
            marks,
            chips,
            span,
            instants,
            scrub: Slider::new(0.0, span as f32, 0.0),
        }
    }

    /// one lane's height: a caption line with the chip's padding.
    fn lane_h(theme: &Theme) -> f32 {
        Chip::height(theme)
    }

    fn scrub_h(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Sm)
    }

    /// the chips, flowed into rows against `width`.
    fn flow(&self, rect: Rect, theme: &Theme) -> Vec<Rect> {
        let gap = theme.spacing.sm;
        let (mut x, mut y) = (rect.x, rect.y);
        let mut out = Vec::new();
        for chip in &self.chips {
            let (w, h) = chip.preferred_size(theme);
            let w = w.min(rect.w);
            if x > rect.x && x + w > rect.x + rect.w {
                x = rect.x;
                y += h + gap;
            }
            out.push(Rect::new(x, y, w, h));
            x += w + gap;
        }
        out
    }

    /// the strip's height at `width`: the chips, the lanes, the scrub.
    pub fn height(&self, width: f32, theme: &Theme) -> f32 {
        let chips = self.flow(Rect::new(0.0, 0.0, width, 0.0), theme);
        let top = chips.iter().map(Rect::bottom).fold(0.0, f32::max);
        let gap = theme.spacing.sm;
        top + gap + self.lanes.len() as f32 * Self::lane_h(theme) + gap + Self::scrub_h(theme)
    }

    /// the label column: the widest label, at most a third of the strip.
    fn label_w(&self, width: f32, theme: &Theme) -> f32 {
        let style = theme.typography.run(TypeRole::Caption.regular());
        let widest = self
            .lanes
            .iter()
            .map(|l| TextMeasurer::measure_styled(l, &style, None).0)
            .fold(0.0, f32::max);
        (widest + theme.spacing.md).min(width / 3.0)
    }

    pub fn geometry(&self, rect: Rect, theme: &Theme) -> Geometry {
        let chips = self.flow(rect, theme);
        let gap = theme.spacing.sm;
        let top = chips.iter().map(Rect::bottom).fold(rect.y, f32::max) + gap;
        let lane_h = Self::lane_h(theme);
        let lanes: Vec<Rect> = (0..self.lanes.len())
            .map(|i| Rect::new(rect.x, top + i as f32 * lane_h, rect.w, lane_h))
            .collect();
        let label = self.label_w(rect.w, theme);
        let scrub = Rect::new(
            rect.x + label,
            top + self.lanes.len() as f32 * lane_h + gap,
            (rect.w - label).max(0.0),
            Self::scrub_h(theme),
        );
        let knob = theme.control.slider_knob;
        let track = Rect::new(
            scrub.x + knob / 2.0,
            top,
            (scrub.w - knob).max(0.0),
            self.lanes.len() as f32 * lane_h,
        );
        Geometry {
            chips,
            lanes,
            track,
            scrub,
        }
    }

    /// the x of a time from the start, on the track.
    pub fn x_of(&self, at: Millis, track: Rect) -> f32 {
        if self.span == 0 {
            return track.x;
        }
        track.x + track.w * (at.min(self.span) as f32 / self.span as f32)
    }

    /// the time from the start the scrub stands at.
    pub fn at(&self) -> Millis {
        (self.scrub.value().max(0.0).round() as Millis).min(self.span)
    }

    /// the instant an event was recorded at right before `at`, or right
    /// after it.
    pub fn step(&self, at: Millis, forward: bool) -> Millis {
        if forward {
            let i = self.instants.partition_point(|&i| i <= at);
            self.instants.get(i).copied().unwrap_or(self.span)
        } else {
            let i = self.instants.partition_point(|&i| i < at);
            i.checked_sub(1)
                .and_then(|i| self.instants.get(i))
                .copied()
                .unwrap_or(0)
        }
    }

    pub fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme, at: Millis) {
        let g = self.geometry(rect, theme);
        for (chip, r) in self.chips.iter().zip(&g.chips) {
            chip.render(c, *r, theme);
        }
        let style = theme.typography.run(TypeRole::Caption.regular());
        let label_w = g.scrub.x - rect.x;
        let edge = theme.control.edge_width;
        for (label, lane) in self.lanes.iter().zip(&g.lanes) {
            let text = TextMeasurer::truncate_to_width(label, &style, label_w - theme.spacing.sm);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&text, &style, None),
                x: lane.x,
                y: lane.y + TextMeasurer::vertical_center(&style, lane.h),
                color: theme.colors.text_mid.0,
            });
            c.push(rounded_rect(
                g.track.x,
                lane.y + lane.h / 2.0,
                g.track.w,
                edge,
                0.0,
                theme.glass.edge_soft.0,
            ));
        }
        let mark_w = theme.control.edge_width_strong;
        let inset = theme.spacing.xs;
        for mark in &self.marks {
            let Some(lane) = g.lanes.get(mark.lane) else {
                continue;
            };
            c.push(rounded_rect(
                self.x_of(mark.at, g.track) - mark_w / 2.0,
                lane.y + inset,
                mark_w,
                (lane.h - inset * 2.0).max(0.0),
                0.0,
                mark.tone.color(theme),
            ));
        }
        c.push(rounded_rect(
            self.x_of(at, g.track) - edge / 2.0,
            g.track.y,
            edge,
            g.track.h,
            0.0,
            theme.colors.accent.0,
        ));
        self.scrub.render(c, g.scrub, theme);
    }
}

/// a lane's label: the actor and what it did.
fn tally(t: &Tally) -> String {
    let mut parts = Vec::new();
    for (n, what) in [
        (t.observations, "observed"),
        (t.requests, "asked"),
        (t.inputs, "input"),
    ] {
        if n > 0 {
            parts.push(format!("{n} {what}"));
        }
    }
    if parts.is_empty() {
        t.actor.to_string()
    } else {
        format!("{}: {}", t.actor, parts.join(", "))
    }
}

fn stage_tone(stage: &str) -> Tone {
    match stage {
        "rejected" | "failed" => Tone::Refused,
        "completed" => Tone::Done,
        _ => Tone::Stage,
    }
}
