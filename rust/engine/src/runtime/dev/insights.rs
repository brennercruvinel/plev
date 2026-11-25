//! What the numbers of a frame mean: the findings a person would draw
//! from the views, said in a line each. Every one is read from the frame
//! shown and the frames kept, by a rule written here, with the numbers it
//! was drawn from in its words; none is a guess about code the engine
//! cannot see.
//!
//! The rules. The first opens the list, the others follow it worst first:
//!
//! - the frame against the 16.6 ms budget, with the phase and the step
//!   that took most of it;
//! - errors and warnings in the log;
//! - layers redrawn that hold nothing (a pass each, for no pixel);
//! - the text measurer's cache missing a quarter of its questions or more;
//! - a frame that asks the heap for a thousand blocks or more, and a heap
//!   that grew over the frames kept (read once the recorder's history is
//!   full, since the reports it keeps are heap too);
//! - measurements over a frame's limits, not kept;
//! - what bounds the reading itself: a debug build, a device without GPU
//!   timestamps, a heap that is not counted, the dev mode's own cost;
//! - and what is as it should be: nothing redrawn, the frame well inside
//!   the budget.

use super::paint::{BUDGET_NS, Health, bytes, count, percent, time};
use super::{Tab, frames, gpu_span, whole};
use crate::drawing::compositor::LayerId;
use crate::runtime::perf::report::HISTORY_FRAMES;
use crate::runtime::perf::stages::Stage;
use crate::runtime::perf::{FrameReport, Level, Phase, Recorder};

/// How much a finding matters, the least first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Severity {
    /// As it should be.
    Good,
    /// What bounds the reading, or is worth knowing.
    Note,
    /// Worth a look.
    Warn,
    /// Wrong now.
    Bad,
}

/// One finding: what it is, the numbers behind it, and the view that
/// shows them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Insight {
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub tab: Tab,
}

/// Blocks asked of the heap in one frame from which it is said.
const BLOCKS_MANY: u64 = 1_000;
/// Bytes the heap grew over the frames kept from which it is said.
const GROWTH_MANY: i64 = 1 << 20;
/// Frames the growth is read over, at least.
const GROWTH_FRAMES: usize = 30;
/// The dev mode's own time in a frame from which it is said.
const DEV_COST_NS: u64 = 1_000_000;
/// GPU memory under which the layers' share of it is not worth a line.
const GPU_MEMORY_SMALL: u64 = 32 << 20;
/// Draw calls in a frame from which they are said.
const DRAWS_MANY: u32 = 200;

/// The findings about `shown`: the budget's, then the worst first. `dev` is the layer the
/// dev mode draws on: its own redraw is not the page's.
pub(super) fn of(recorder: &Recorder, shown: &FrameReport, dev: LayerId) -> Vec<Insight> {
    let mut all = Vec::new();
    let mut say = |severity, tab, title: String, detail: String| {
        all.push(Insight {
            severity,
            title,
            detail,
            tab,
        });
    };

    // the frame against the budget
    let total = whole(shown);
    let nanos = |phase: Phase| match phase {
        Phase::Gpu => gpu_span(shown).unwrap_or(0),
        _ => shown.pipeline.phase(phase).nanos,
    };
    let top = Phase::ALL
        .into_iter()
        .max_by_key(|phase| nanos(*phase))
        .unwrap_or(Phase::Scene);
    let step = shown
        .pipeline
        .phases
        .iter()
        .flat_map(|phase| phase.parts.iter())
        .max_by_key(|(_, cost)| cost.nanos)
        .filter(|(_, cost)| cost.nanos > 0);
    let mut where_to = format!(
        "{} is {}% of the frame",
        top.name(),
        percent(nanos(top), total)
    );
    if let Some((stage, cost)) = step {
        where_to = format!(
            "{where_to}; the heaviest step on the CPU is {} at {}",
            stage.name(),
            time(cost.nanos)
        );
    }
    let share = percent(total, BUDGET_NS);
    match Health::of(total) {
        Health::Over => say(
            Severity::Bad,
            Tab::Zones,
            format!("Over budget: {} is {share}% of 16.6 ms", time(total)),
            format!("{where_to}."),
        ),
        Health::Tight => say(
            Severity::Warn,
            Tab::Zones,
            format!("Close to the budget: {}, {share}% of 16.6 ms", time(total)),
            format!("{where_to}."),
        ),
        Health::Good => say(
            Severity::Good,
            Tab::Frame,
            format!("Inside the budget: {}, {share}% of 16.6 ms", time(total)),
            format!("{where_to}."),
        ),
    }

    // the log
    let lines = |level: Level| recorder.log().filter(move |l| l.message.level == level);
    let (errors, warnings) = (lines(Level::Error).count(), lines(Level::Warn).count());
    let last = |level: Level| {
        lines(level)
            .next_back()
            .map(|l| l.message.text.clone())
            .unwrap_or_default()
    };
    if errors > 0 {
        say(
            Severity::Bad,
            Tab::Log,
            format!("{errors} {} in the log", plural(errors, "error")),
            last(Level::Error),
        );
    }
    if warnings > 0 {
        say(
            Severity::Warn,
            Tab::Log,
            format!("{warnings} {} in the log", plural(warnings, "warning")),
            last(Level::Warn),
        );
    }

    // the page's layers
    let page: Vec<_> = shown.layers.iter().filter(|l| l.id != dev).collect();
    let redrawn = page.iter().filter(|l| l.redrawn).count();
    let hollow: Vec<_> = page
        .iter()
        .filter(|l| l.redrawn && l.visible && l.nodes == 0)
        .collect();
    if !hollow.is_empty() {
        let gpu: u64 = shown
            .gpu
            .iter()
            .flat_map(|g| g.passes.iter())
            .filter(|pass| hollow.iter().any(|l| l.id == pass.layer))
            .filter_map(|pass| pass.nanos)
            .sum();
        let cost = if gpu > 0 {
            format!(", {} on the GPU", time(gpu))
        } else {
            String::new()
        };
        say(
            Severity::Warn,
            Tab::Gpu,
            format!(
                "{} redrawn {} nothing",
                hollow.len(),
                if hollow.len() == 1 {
                    "layer holds"
                } else {
                    "layers hold"
                }
            ),
            format!("Each is still a render pass{cost}. A layer with no node need not be dirty."),
        );
    }
    if redrawn == 0 && !page.is_empty() {
        say(
            Severity::Good,
            Tab::Gpu,
            "Nothing redrawn".to_owned(),
            format!(
                "The page reused all {} of its layers: render on demand at work.",
                page.len()
            ),
        );
    } else if redrawn == page.len() && page.len() > 1 && hollow.is_empty() {
        say(
            Severity::Note,
            Tab::Gpu,
            format!("Every layer redrawn ({redrawn} of {redrawn})"),
            "A change that touches one layer redraws that one alone.".to_owned(),
        );
    }
    if shown.stats.draw_calls >= DRAWS_MANY {
        say(
            Severity::Note,
            Tab::Gpu,
            format!("{} draw calls", count(u64::from(shown.stats.draw_calls))),
            "Many small batches: nodes that share a pipeline and a clip draw together.".to_owned(),
        );
    }

    // text
    let (hit, miss) = (
        shown.stages.get(Stage::MeasureHit),
        shown.stages.get(Stage::MeasureMiss),
    );
    let asked = hit.calls + miss.calls;
    if miss.calls > 0 && miss.calls * 4 >= asked {
        say(
            Severity::Warn,
            Tab::Frame,
            format!(
                "Text cache missed {} of {} measures",
                count(miss.calls),
                count(asked)
            ),
            format!(
                "Shaping them took {}. Text that changes every frame is shaped every frame.",
                time(miss.nanos)
            ),
        );
    }

    // the heap
    let heap = shown.heap;
    if !heap.counted {
        say(
            Severity::Note,
            Tab::Memory,
            "The heap is not counted".to_owned(),
            "Install perf::heap::Counting as the global allocator for blocks and bytes a frame."
                .to_owned(),
        );
    } else {
        if heap.allocs >= BLOCKS_MANY {
            say(
                Severity::Warn,
                Tab::Memory,
                format!("{} heap blocks asked for in one frame", count(heap.allocs)),
                format!(
                    "{} in, {} back. A frame that allocates pays for it every time it runs.",
                    bytes(heap.bytes_in),
                    bytes(heap.bytes_out)
                ),
            );
        }
        // read only once the recorder's own history is full: until then
        // the reports it keeps are themselves heap that grows every frame
        let kept = frames(recorder);
        let steady = recorder.len() >= HISTORY_FRAMES && kept.len() >= GROWTH_FRAMES;
        if let (true, Some(first), Some(last)) = (steady, kept.first(), kept.last()) {
            let grew = last.heap.live_bytes - first.heap.live_bytes;
            if grew >= GROWTH_MANY {
                say(
                    Severity::Warn,
                    Tab::Memory,
                    format!(
                        "The heap grew {} over {} frames",
                        bytes(grew.unsigned_abs()),
                        kept.len()
                    ),
                    "Live bytes rise frame after frame: something is kept and never given back."
                        .to_owned(),
                );
            }
        }
    }

    // what was not kept
    let dropped = shown.marks_dropped + shown.zones.dropped;
    if dropped > 0 {
        say(
            Severity::Warn,
            Tab::Zones,
            format!("{} measurements were not kept", count(dropped)),
            "Zones, plots or messages over a frame's limit: what shows is not all there was."
                .to_owned(),
        );
    }

    // what bounds the reading
    let memory = shown.memory;
    let on_gpu = memory.gpu_total_bytes();
    if on_gpu >= GPU_MEMORY_SMALL && memory.layer_bytes * 5 >= on_gpu * 4 {
        let empty = page.iter().filter(|l| l.nodes == 0).count();
        say(
            Severity::Note,
            Tab::Memory,
            format!(
                "Layers hold {} of {} on the GPU",
                bytes(memory.layer_bytes),
                bytes(on_gpu)
            ),
            format!(
                "{} layers the size of the window; {empty} of them hold no node.",
                shown.layers.len()
            ),
        );
    }
    if !recorder.gpu_available() {
        say(
            Severity::Note,
            Tab::Gpu,
            "No GPU timestamps".to_owned(),
            "This device offers no TIMESTAMP_QUERY: the GPU's own time is not measured.".to_owned(),
        );
    } else if let Some(gpu) = shown.gpu.as_ref().filter(|gpu| gpu.incomplete > 0) {
        say(
            Severity::Note,
            Tab::Gpu,
            format!(
                "{} {} without both timestamps",
                gpu.incomplete,
                plural(gpu.incomplete as usize, "pass")
            ),
            "A pass that draws nothing writes no end on Metal: its time is not counted.".to_owned(),
        );
    }
    let own: u64 = shown
        .zones
        .threads
        .iter()
        .flat_map(|t| t.zones.iter())
        .filter(|z| z.name == "dev mode")
        .map(|z| z.nanos)
        .sum();
    if own >= DEV_COST_NS {
        say(
            Severity::Note,
            Tab::Zones,
            format!("The dev mode took {} of this frame", time(own)),
            "Its panel is a layer of the same frame: close it to time the page alone.".to_owned(),
        );
    }
    if cfg!(debug_assertions) {
        say(
            Severity::Note,
            Tab::Frame,
            "This is a debug build".to_owned(),
            "Its times are several times a release build's: measure with --release.".to_owned(),
        );
    }

    // the frame against the budget opens the list whatever it says: it
    // is what the others are read against
    all[1..].sort_by_key(|insight| std::cmp::Reverse(insight.severity));
    all
}

/// `word`, or its plural for a count that is not one.
fn plural(n: usize, word: &str) -> String {
    match (n, word) {
        (1, _) => word.to_owned(),
        (_, "pass") => "passes".to_owned(),
        _ => format!("{word}s"),
    }
}
