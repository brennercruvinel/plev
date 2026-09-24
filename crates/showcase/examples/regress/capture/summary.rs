//! The capture's report: a line per scale and theme as the run goes,
//! then the totals, a table with one line per page and theme and a
//! column per scale, and every frame that did not hold with its
//! numbers; `report.json` holds the same, frame by frame.

use std::path::Path;
use std::time::Duration;

use serde::Serialize;

use super::{Cell, Options};
use crate::baseline::Key;
use crate::classify::Verdict;

/// A finished capture.
pub struct Run<'a> {
    pub options: &'a Options,
    pub cells: &'a [Cell],
    pub total: Duration,
    /// What opening each context's renderer took.
    pub open: [Option<Duration>; 2],
}

/// Verdicts by class.
#[derive(Debug, Default, PartialEq, Serialize)]
pub struct Counts {
    pub equal: usize,
    pub requested: usize,
    pub silent: usize,
    pub resized: usize,
}

impl Counts {
    pub fn of<'a>(verdicts: impl Iterator<Item = &'a Verdict>) -> Self {
        let mut c = Self::default();
        for v in verdicts {
            match v {
                Verdict::Equal => c.equal += 1,
                Verdict::Requested { .. } => c.requested += 1,
                Verdict::Silent(_) => c.silent += 1,
                Verdict::Resized { .. } => c.resized += 1,
            }
        }
        c
    }

    fn line(&self) -> String {
        format!(
            "{} equal, {} requested, {} silent, {} resized",
            self.equal, self.requested, self.silent, self.resized
        )
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// The line printed when the frames of `last`'s scale and theme are done
/// (they are the last cells, together).
pub fn group_line(cells: &[Cell], last: &Key) -> String {
    let same = |c: &&Cell| c.scale == last.viewport.scale && c.theme == last.theme;
    let group: Vec<&Cell> = cells.iter().rev().take_while(same).collect();
    let held = group.iter().filter(|c| c.held()).count();
    let secs: f64 = group.iter().map(|c| c.ms).sum::<f64>() / 1000.0;
    format!(
        "{:>5}x {:<6} {} frames, {held} held, {secs:.1} s",
        last.viewport.scale,
        last.theme,
        group.len()
    )
}

/// The report as text.
pub fn text(run: &Run<'_>) -> String {
    let o = run.options;
    let cells = run.cells;
    let mut pages: Vec<&str> = Vec::new();
    let mut themes: Vec<&str> = Vec::new();
    for c in cells {
        if !pages.contains(&c.page.as_str()) {
            pages.push(&c.page);
        }
        if !themes.contains(&c.theme.as_str()) {
            themes.push(&c.theme);
        }
    }
    if o.reverse {
        pages.reverse();
        themes.reverse();
    }
    let opened: Vec<String> = run
        .open
        .iter()
        .map(|d| d.map_or("-".to_string(), |d| format!("{:.1} ms", ms(d))))
        .collect();
    let mut out = format!(
        "desktop capture: {} pages x {} themes x {} scales = {} frames at {}x{} logical, \
         in {:.1} s (GPU contexts opened in {})\n",
        pages.len(),
        themes.len(),
        o.scales.len(),
        cells.len(),
        o.width,
        o.height,
        run.total.as_secs_f64(),
        opened.join(" and "),
    );
    out.push_str(&format!(
        "two GPU contexts of this process: {}\n",
        Counts::of(cells.iter().map(|c| &c.contexts)).line()
    ));
    match &o.against {
        Some(dir) => out.push_str(&format!(
            "this process against {}: {}\n",
            dir.display(),
            Counts::of(cells.iter().filter_map(|c| c.processes.as_ref())).line()
        )),
        None => out.push_str("another process: not asked (--against <dir>)\n"),
    }
    let cell = |c: &Cell| match &c.processes {
        Some(p) if p != &c.contexts => format!("{} / {}", c.contexts.short(), p.short()),
        _ => c.contexts.short(),
    };
    let width = pages.iter().map(|p| p.len()).max().unwrap_or(4).max(4);
    let mut rows = vec![{
        let mut head = vec![format!("{:width$}", "page"), format!("{:6}", "theme")];
        head.extend(o.scales.iter().map(|s| format!("{:8}", format!("{s}x"))));
        head.join(" ").trim_end().to_string()
    }];
    for page in &pages {
        for theme in &themes {
            let mut line = vec![format!("{page:width$}"), format!("{theme:6}")];
            for scale in &o.scales {
                let found = cells
                    .iter()
                    .find(|c| c.page == *page && c.theme == *theme && c.scale == *scale);
                line.push(format!("{:8}", found.map_or("-".to_string(), cell)));
            }
            rows.push(line.join(" ").trim_end().to_string());
        }
    }
    out.push('\n');
    out.push_str(&rows.join("\n"));
    out.push_str("\n(a cell reads \"contexts / processes\" when the two comparisons differ)\n");
    let failed: Vec<&Cell> = cells.iter().filter(|c| !c.held()).collect();
    if failed.is_empty() {
        out.push_str("\nevery frame held\n");
    } else {
        out.push_str(&format!("\n{} frames did not hold:\n", failed.len()));
        for c in failed {
            out.push_str(&format!("  {}: contexts: {}", c.label, c.contexts.line()));
            if let Some(p) = &c.processes {
                out.push_str(&format!("; processes: {}", p.line()));
            }
            out.push('\n');
        }
    }
    out
}

#[derive(Serialize)]
struct Json<'a> {
    viewport: [f32; 2],
    pages: usize,
    themes: &'a [String],
    scales: &'a [f32],
    frames: usize,
    total_ms: f64,
    gpu_open_ms: [Option<f64>; 2],
    against: Option<String>,
    contexts: Counts,
    processes: Option<Counts>,
    cells: &'a [Cell],
}

/// `report.txt` (`text`) and `report.json` under `--out`.
pub fn write(run: &Run<'_>, text: &str) -> Result<(), String> {
    let o = run.options;
    let themes = o.themes.len().max(1);
    let json = Json {
        viewport: [o.width, o.height],
        pages: run.cells.len() / (themes * o.scales.len().max(1)),
        themes: &o.themes,
        scales: &o.scales,
        frames: run.cells.len(),
        total_ms: ms(run.total),
        gpu_open_ms: run.open.map(|d| d.map(ms)),
        against: o.against.as_ref().map(|d| d.display().to_string()),
        contexts: Counts::of(run.cells.iter().map(|c| &c.contexts)),
        processes: o
            .against
            .as_ref()
            .map(|_| Counts::of(run.cells.iter().filter_map(|c| c.processes.as_ref()))),
        cells: run.cells,
    };
    let body = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    let io = |path: &Path, e: std::io::Error| format!("{}: {e}", path.display());
    let txt = o.out.join("report.txt");
    std::fs::write(&txt, text).map_err(|e| io(&txt, e))?;
    let js = o.out.join("report.json");
    std::fs::write(&js, body + "\n").map_err(|e| io(&js, e))
}
