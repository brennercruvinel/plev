//! The frame at a high level: the 22 stages of a [`StageFrame`] grouped
//! into the phases a frame runs, in order (input, tick, scene, resolve,
//! encode, GPU), each with its total, the time not in any of its timed
//! parts, and the parts that ran.
//!
//! [`Pipeline::of`] is a pure function of the stage frame. Stages nest
//! (see `stages`): a phase's parts run inside it, so its own time is its
//! total minus its timed parts, saturating at zero where a part is also
//! timed outside the phase (a layer hash asked by `needs_render` before
//! the frame) or two parts overlap (an upload inside the text resolve).
//!
//! The GPU phase is what the GPU's clock read during this frame. A
//! frame's timestamps are read back by a later frame (`gpu_time`), so
//! with timestamps it is an earlier frame's GPU time; without them it is
//! the GPU wait, an upper bound, when the GPU sync ran.

use super::stages::{Stage, StageCost, StageFrame};

/// A phase of the frame, in the order a frame runs them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    /// The app's event handlers, the hit tests they ask included.
    Input,
    /// The animation tick.
    Tick,
    /// Building the scene: layout, text measuring, binds.
    Scene,
    /// The compositor's resolve and the text resolve: hashes, geometry,
    /// glyphs, uploads.
    Resolve,
    /// The layer and composite passes encoded and submitted.
    Encode,
    /// The GPU running a frame.
    Gpu,
}

impl Phase {
    /// How many phases there are.
    pub const COUNT: usize = 6;

    /// Every phase, in frame order (`ALL[p as usize] == p`).
    pub const ALL: [Phase; Phase::COUNT] = [
        Phase::Input,
        Phase::Tick,
        Phase::Scene,
        Phase::Resolve,
        Phase::Encode,
        Phase::Gpu,
    ];

    /// Short lowercase name for a table row.
    pub fn name(self) -> &'static str {
        match self {
            Phase::Input => "input",
            Phase::Tick => "tick",
            Phase::Scene => "scene",
            Phase::Resolve => "resolve",
            Phase::Encode => "encode",
            Phase::Gpu => "gpu",
        }
    }

    /// The phase runs on the CPU (every one but [`Phase::Gpu`]).
    pub fn is_cpu(self) -> bool {
        self != Phase::Gpu
    }
}

/// One phase's cost in one frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseCost {
    pub phase: Phase,
    /// The phase's total.
    pub nanos: u64,
    /// The total minus the timed parts, saturating at zero.
    pub own_nanos: u64,
    /// What the phase handled: events, scene nodes, glyphs, draw calls,
    /// GPU layer passes (see [`Stage::items`] of its main stage).
    pub items: u64,
    /// The parts that ran, in stage order.
    pub parts: Vec<(Stage, StageCost)>,
}

/// A frame's phases and its totals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pipeline {
    /// Every phase, in [`Phase::ALL`] order, run or not.
    pub phases: [PhaseCost; Phase::COUNT],
    /// The CPU phases summed.
    pub cpu_nanos: u64,
    /// The GPU phase's time.
    pub gpu_nanos: u64,
    /// The GPU span minus its layer passes and its composite pass: the
    /// passes not timed (a backdrop blur) and the gaps between passes.
    /// Zero without a timestamp reading.
    pub gpu_other_nanos: u64,
    /// [`Stage::Frame`] when the app's loop recorded it, else
    /// `cpu_nanos`.
    pub frame_nanos: u64,
    /// `frame_nanos` minus the CPU phases, saturating: the time the frame
    /// spent outside every phase (the image upload, the app's own work).
    pub other_nanos: u64,
}

/// The parts of the scene phase that its own time leaves out. Shaping is
/// listed but not subtracted: a measure miss already includes its own.
const SCENE_TIMED: [Stage; 5] = [
    Stage::Projection,
    Stage::Layout,
    Stage::MeasureHit,
    Stage::MeasureMiss,
    Stage::Bind,
];

impl Pipeline {
    /// The phases of `frame`.
    pub fn of(frame: &StageFrame) -> Pipeline {
        let input = phase(frame, Phase::Input, &[Stage::Input], &[Stage::HitTest], &[]);
        let tick = phase(frame, Phase::Tick, &[Stage::Tick], &[], &[]);
        let scene = phase(
            frame,
            Phase::Scene,
            &[Stage::SceneBuild],
            &SCENE_TIMED,
            &[Stage::Shape],
        );
        let resolve_parts = [
            Stage::LayerHash,
            Stage::Geometry,
            Stage::TextResolve,
            Stage::Upload,
        ];
        let mut resolve = phase(
            frame,
            Phase::Resolve,
            &[Stage::Resolve, Stage::TextResolve],
            &resolve_parts,
            &[],
        );
        resolve.items = frame.get(Stage::TextResolve).items;
        let encode = phase(frame, Phase::Encode, &[Stage::Encode], &[], &[]);
        let (gpu, gpu_other_nanos) = gpu_phase(frame);

        let cpu = [&input, &tick, &scene, &resolve, &encode];
        let cpu_nanos = cpu.iter().map(|p| p.nanos).sum();
        let whole = frame.get(Stage::Frame);
        let frame_nanos = if whole.calls > 0 {
            whole.nanos
        } else {
            cpu_nanos
        };
        Pipeline {
            cpu_nanos,
            gpu_nanos: gpu.nanos,
            gpu_other_nanos,
            frame_nanos,
            other_nanos: frame_nanos.saturating_sub(cpu_nanos),
            phases: [input, tick, scene, resolve, encode, gpu],
        }
    }

    /// The cost of `phase`.
    pub fn phase(&self, phase: Phase) -> &PhaseCost {
        &self.phases[phase as usize]
    }

    /// The share of `budget_nanos` (a refresh interval) the frame took:
    /// 1.0 is the whole budget; 0.0 for a zero budget.
    pub fn budget_share(&self, budget_nanos: u64) -> f32 {
        if budget_nanos == 0 {
            return 0.0;
        }
        (self.frame_nanos as f64 / budget_nanos as f64) as f32
    }
}

/// A CPU phase: `main` summed, minus the `timed` parts, with every part
/// that ran (`timed` and `listed`) in stage order.
fn phase(
    frame: &StageFrame,
    phase: Phase,
    main: &[Stage],
    timed: &[Stage],
    listed: &[Stage],
) -> PhaseCost {
    let nanos = main.iter().map(|&s| frame.get(s).nanos).sum();
    let inside: u64 = timed.iter().map(|&s| frame.get(s).nanos).sum();
    let mut parts: Vec<(Stage, StageCost)> = timed
        .iter()
        .chain(listed)
        .map(|&s| (s, frame.get(s)))
        .filter(|(_, cost)| cost.calls > 0)
        .collect();
    parts.sort_by_key(|(s, _)| *s as usize);
    PhaseCost {
        phase,
        nanos,
        own_nanos: nanos.saturating_sub(inside),
        items: main.first().map_or(0, |&s| frame.get(s).items),
        parts,
    }
}

/// The mean time per run of `stage` (more than one reading can land in a
/// frame without the GPU sync).
fn mean(cost: StageCost) -> u64 {
    cost.nanos.checked_div(cost.calls).unwrap_or(0)
}

/// The GPU phase and the span's time outside the timed passes. Its parts
/// are the stage costs as recorded (sums, when readings piled up); its
/// times are per reading.
fn gpu_phase(frame: &StageFrame) -> (PhaseCost, u64) {
    let span = frame.get(Stage::GpuSpan);
    let wait = frame.get(Stage::GpuWait);
    let layers = frame.get(Stage::GpuLayers);
    let composite = frame.get(Stage::GpuComposite);
    let parts: Vec<(Stage, StageCost)> =
        [(Stage::GpuLayers, layers), (Stage::GpuComposite, composite)]
            .into_iter()
            .filter(|(_, cost)| cost.calls > 0)
            .collect();
    let (nanos, other) = if span.calls > 0 {
        let timed = mean(layers) + mean(composite);
        (mean(span), mean(span).saturating_sub(timed))
    } else if wait.calls > 0 {
        (wait.nanos, 0)
    } else {
        (0, 0)
    };
    let own_nanos = if span.calls > 0 { other } else { nanos };
    let cost = PhaseCost {
        phase: Phase::Gpu,
        nanos,
        own_nanos,
        items: layers.items.checked_div(span.calls).unwrap_or(0),
        parts: if span.calls > 0 { parts } else { Vec::new() },
    };
    (cost, other)
}
