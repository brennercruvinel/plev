//! The JSON `check --json` writes: the totals of the run, then every
//! baseline's [`Report`] as serde gives it.

use std::path::Path;
use std::time::Duration;

use serde::Serialize;

use super::Report;

#[derive(Serialize)]
struct Checked<'a> {
    checked: usize,
    unchanged: usize,
    changed: usize,
    frames_drawn: usize,
    gpu_open_ms: Option<f64>,
    total_ms: f64,
    baselines: &'a [Report],
}

/// Write `reports` to `path`, with the frames drawn, what opening the
/// renderer took (if it was opened) and the whole run's time.
pub fn write_json(
    path: &Path,
    reports: &[Report],
    frames: usize,
    open: Option<Duration>,
    total: Duration,
) -> Result<(), String> {
    let changed = reports.iter().filter(|r| r.changed).count();
    let checked = Checked {
        checked: reports.len(),
        unchanged: reports.len() - changed,
        changed,
        frames_drawn: frames,
        gpu_open_ms: open.map(|d| d.as_secs_f64() * 1000.0),
        total_ms: total.as_secs_f64() * 1000.0,
        baselines: reports,
    };
    let json = serde_json::to_string_pretty(&checked).map_err(|e| e.to_string())?;
    std::fs::write(path, json + "\n").map_err(|e| format!("{}: {e}", path.display()))
}
