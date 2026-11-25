//! The frames the views read, and a frame's time as they all count it.

use crate::runtime::perf::{FrameReport, Recorder};

/// The frames a chart covers, at most.
const FRAMES_MOST: usize = 120;

/// The frames a chart covers, oldest first: the latest the app drew for
/// itself, the dev mode's own left out.
pub(super) fn frames(recorder: &Recorder) -> Vec<&FrameReport> {
    let mut kept: Vec<&FrameReport> = recorder
        .history()
        .rev()
        .filter(|frame| !frame.tool)
        .take(FRAMES_MOST)
        .collect();
    kept.reverse();
    kept
}

/// The GPU's time over the frame, from the frame's own timestamps.
pub(super) fn gpu_span(frame: &FrameReport) -> Option<u64> {
    frame.gpu.as_ref().and_then(|gpu| gpu.span)
}

/// A frame's whole time: its CPU phases and the GPU's span.
pub(super) fn whole(frame: &FrameReport) -> u64 {
    frame.pipeline.cpu_nanos + gpu_span(frame).unwrap_or(0)
}
