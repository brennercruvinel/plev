//! One frame, complete, for the dev mode: its stages and the phases they
//! make, its zones on every thread, what code said about it (plots and
//! messages), the compositor's counters, the memory readings, what the
//! heap did, every layer with what the frame did to it, and the GPU's own
//! time once its timestamps come back.
//!
//! A [`Recorder`] belongs to a window host and is off by default.
//! [`Recorder::set_on`] turns the sources on together: stages and the GPU
//! sync for the calling thread (the frame's), zones and marks for the
//! process. The heap's counters are not the recorder's to switch: they
//! count from the moment the app turned them on (`heap::enable`), and the
//! recorder keeps what moved between two of its frames.
//! While on, the host calls [`Recorder::end_frame`] after each frame it
//! draws: the stage, zone and mark frames are taken there, so one report is
//! exactly one frame, and the report goes into a ring of the last
//! [`HISTORY_FRAMES`]. A frame's GPU timestamps are read back by a later
//! frame (`gpu_time`); each reading is attached to the report whose index
//! it carries, so [`FrameReport::gpu`] fills in after the fact.
//!
//! The engine renders on demand: after the last frame of a change no
//! frame comes to bring its timestamps back. A tool that shows them asks
//! for a few frames of its own ([`Recorder::mark_tool`] before each),
//! while [`Recorder::settling`] says the latest frame still waits for
//! them. Tool frames stay in the history, flagged, and are left out of
//! [`Recorder::latest`] and [`Recorder::slowest`].
//!
//! Stages are per thread and zones per process: one recorder on at a time.

use std::collections::VecDeque;

use super::gpu_time::GpuReading;
use super::heap::{self, HeapFrame, HeapTotals};
use super::marks::{self, Message, PlotPoint};
use super::memory::MemoryStats;
use super::pipeline::Pipeline;
use super::stages::{self, StageFrame};
use super::zones::{self, ZoneFrame};
use crate::drawing::compositor::{Compositor, LayerId, RenderStats};
use crate::drawing::gpu::GpuContext;

/// Frames a [`Recorder`] keeps: four seconds at 60 fps.
pub const HISTORY_FRAMES: usize = 240;

/// Tool frames drawn after the latest frame before
/// [`Recorder::settling`] gives up on its timestamps.
pub const SETTLING_FRAMES: u32 = 3;

/// One frame, complete.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameReport {
    /// The host's frame counter for this frame.
    pub index: u64,
    pub stages: StageFrame,
    pub pipeline: Pipeline,
    pub zones: ZoneFrame,
    pub stats: RenderStats,
    pub memory: MemoryStats,
    /// The values code gave under a name (`plot!`), one per name.
    pub plots: Vec<PlotPoint>,
    /// The lines code said (`message!`), in order.
    pub messages: Vec<Message>,
    /// Plots and messages over the frame's limits, not kept.
    pub marks_dropped: u64,
    /// The heap from the end of the frame recorded before to this one's.
    pub heap: HeapFrame,
    /// Every layer of the compositor, in its order.
    pub layers: Vec<LayerReport>,
    /// This frame's GPU timestamps, attached when a later frame reads
    /// them back; `None` until then, and always without timestamps.
    pub gpu: Option<GpuReading>,
    /// A frame the dev mode itself asked for.
    pub tool: bool,
}

/// One layer as the frame left it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerReport {
    pub id: LayerId,
    pub z_order: i32,
    pub visible: bool,
    pub opacity: f32,
    /// The frame encoded the layer's passes again.
    pub redrawn: bool,
    /// Scene nodes in the layer.
    pub nodes: u32,
    /// Glyph quads uploaded for the layer.
    pub glyphs: u32,
    /// Draw calls of the layer's passes this frame; 0 when not redrawn.
    pub draws: u32,
    /// Layer effects (blur, shadow) applied to it.
    pub effects: u32,
}

impl LayerReport {
    /// Every layer of `compositor` after a frame (`window::draw_frame`
    /// left what it redrew in `Compositor::drawn_layers`).
    pub fn of(compositor: &Compositor) -> Vec<LayerReport> {
        let drawn = compositor.drawn_layers();
        compositor
            .layers()
            .iter()
            .map(|layer| {
                let draws = drawn.iter().find(|(id, _)| *id == layer.id);
                LayerReport {
                    id: layer.id,
                    z_order: layer.z_order,
                    visible: layer.visible,
                    opacity: layer.opacity,
                    redrawn: draws.is_some(),
                    nodes: count(layer.nodes().len()),
                    glyphs: layer.glyph_count(),
                    draws: draws.map_or(0, |(_, n)| *n),
                    effects: count(layer.effects().len()),
                }
            })
            .collect()
    }
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl FrameReport {
    /// The report of frame `index` from what it recorded and what it left
    /// in `compositor`; no GPU reading yet, nothing said and no heap count
    /// (`Recorder::end_frame` fills those).
    pub fn new(
        index: u64,
        stages: StageFrame,
        zones: ZoneFrame,
        compositor: &Compositor,
        memory: MemoryStats,
    ) -> Self {
        Self {
            index,
            pipeline: Pipeline::of(&stages),
            stages,
            zones,
            stats: compositor.stats(),
            memory,
            plots: Vec::new(),
            messages: Vec::new(),
            marks_dropped: 0,
            heap: HeapFrame::default(),
            layers: LayerReport::of(compositor),
            gpu: None,
            tool: false,
        }
    }
}

/// The last [`HISTORY_FRAMES`] reports of one window. Off by default.
#[derive(Debug, Default)]
pub struct Recorder {
    on: bool,
    frames: VecDeque<FrameReport>,
    /// The next frame is the tool's.
    tool_next: bool,
    /// Tool frames since the latest frame that was not one.
    tool_frames: u32,
    gpu_available: bool,
    /// The heap's counters at the end of the last frame recorded.
    heap_last: HeapTotals,
}

impl Recorder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Turn recording on or off: stages and the GPU sync on the calling
    /// thread, zones and marks in the process. Turning on starts from an
    /// empty stage, zone and mark frame; turning off drops what they
    /// held. The history stays until [`clear`](Self::clear).
    pub fn set_on(&mut self, on: bool) {
        if on == self.on {
            return;
        }
        self.on = on;
        self.tool_next = false;
        stages::enable(on);
        stages::set_gpu_sync(on);
        zones::enable(on);
        marks::enable(on);
        if on {
            let _ = stages::take();
            let _ = zones::take();
            let _ = marks::take();
            self.heap_last = heap::totals();
        }
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    /// Flag the next frame recorded as the tool's own.
    pub fn mark_tool(&mut self) {
        self.tool_next = true;
    }

    /// The frame `index` was drawn: take its stages, zones and marks, read
    /// the compositor, `memory` and the heap, keep the report, then attach the GPU
    /// readings `gpu` collected to the frames they belong to. Nothing
    /// while off.
    pub fn end_frame(
        &mut self,
        index: u64,
        compositor: &Compositor,
        memory: MemoryStats,
        gpu: &GpuContext,
    ) {
        if !self.on {
            return;
        }
        let mut report = FrameReport::new(index, stages::take(), zones::take(), compositor, memory);
        let said = marks::take();
        report.plots = said.plots;
        report.messages = said.messages;
        report.marks_dropped = said.dropped;
        let heap = heap::totals();
        report.heap = heap.since(&self.heap_last);
        self.heap_last = heap;
        let readings = gpu
            .timestamps
            .as_ref()
            .map(|t| t.take_readings())
            .unwrap_or_default();
        self.push(report, readings, gpu.timestamps.is_some());
    }

    /// Keep `report` (flagged when [`mark_tool`](Self::mark_tool) asked)
    /// and attach `readings` by frame index; a reading whose frame left
    /// the history is dropped. `gpu_available` says whether readings can
    /// come at all.
    pub(crate) fn push(
        &mut self,
        mut report: FrameReport,
        readings: Vec<GpuReading>,
        gpu_available: bool,
    ) {
        self.gpu_available = gpu_available;
        report.tool = std::mem::take(&mut self.tool_next);
        if report.tool {
            self.tool_frames = self.tool_frames.saturating_add(1);
        } else {
            self.tool_frames = 0;
        }
        if self.frames.len() == HISTORY_FRAMES {
            self.frames.pop_front();
        }
        self.frames.push_back(report);
        for reading in readings {
            if let Some(frame) = self
                .frames
                .iter_mut()
                .rev()
                .find(|f| f.index == reading.frame)
            {
                frame.gpu = Some(reading);
            }
        }
    }

    /// The latest frame still waits for its GPU timestamps and fewer than
    /// [`SETTLING_FRAMES`] tool frames were drawn since: a tool that shows
    /// them asks for one more frame. Never without timestamps.
    pub fn settling(&self) -> bool {
        self.on
            && self.gpu_available
            && self.tool_frames < SETTLING_FRAMES
            && self.latest().is_some_and(|f| f.gpu.is_none())
    }

    /// The context draws GPU timestamps (`RenderConfig::gpu_timestamps` on
    /// an adapter that has them), as of the last frame recorded. Without
    /// them every [`FrameReport::gpu`] stays `None`.
    pub fn gpu_available(&self) -> bool {
        self.gpu_available
    }

    /// Every report kept, oldest first, tool frames included.
    pub fn history(&self) -> impl DoubleEndedIterator<Item = &FrameReport> {
        self.frames.iter()
    }

    /// The latest frame that was not the tool's.
    pub fn latest(&self) -> Option<&FrameReport> {
        self.frames.iter().rev().find(|f| !f.tool)
    }

    /// The report of frame `index`, while kept.
    pub fn get(&self, index: u64) -> Option<&FrameReport> {
        self.frames.iter().rev().find(|f| f.index == index)
    }

    /// The frame that took longest (`Pipeline::frame_nanos`), tool frames
    /// left out; the latest of equals.
    pub fn slowest(&self) -> Option<&FrameReport> {
        self.frames
            .iter()
            .filter(|f| !f.tool)
            .max_by_key(|f| f.pipeline.frame_nanos)
    }

    /// How long every run of the zone `name` took, over the frames kept
    /// that were not the tool's, oldest first: what a histogram of the
    /// zone is made of.
    pub fn zone_runs(&self, name: &str) -> Vec<u64> {
        self.frames
            .iter()
            .filter(|f| !f.tool)
            .flat_map(|f| f.zones.threads.iter())
            .flat_map(|t| t.zones.iter())
            .filter(|z| z.name == name)
            .map(|z| z.nanos)
            .collect()
    }

    /// Reports kept.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Forget every report.
    pub fn clear(&mut self) {
        self.frames.clear();
        self.tool_frames = 0;
    }
}
