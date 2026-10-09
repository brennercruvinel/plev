//! `regress capture` (task-77): the desktop against itself. Every page
//! of the gallery, in the four themes the sidebar's T key cycles, at
//! scales 1, 2, 1.25 and 1.5, is drawn twice through the window's frame
//! path by two GPU contexts of this process and the second frame is
//! classified against the first (`classify`) under the desktop's rows
//! of the table of requested differences; with `--against <dir>`, the
//! first frame is also classified against the png another process
//! wrote for it there. Each frame's png and the report (`summary`: text
//! with one line per page and theme, and json) go under `--out`, in
//! `target/`, outside git. No baseline is written.

mod summary;

use std::path::{Path, PathBuf};
use std::time::Instant;

use engine::headless::{Frame, Viewport};
use serde::Serialize;
use showcase::view::PAGES;

use crate::baseline::{Key, read_png};
use crate::classify::{Decision, Verdict, classify};
use crate::requested::{TABLE, Table};
use crate::run::Gpu;

/// The themes the sidebar's T key cycles.
pub const THEMES: [&str; 4] = ["core", "hoff", "dark", "light"];

/// The scales that matter: a plain display, a retina one, and the
/// fractional 125% and 150% of windows and linux (and browser zoom).
pub const SCALES: [f32; 4] = [1.0, 2.0, 1.25, 1.5];

/// Where the frames and the report go unless `--out` says otherwise:
/// the repository's `target/`, which git ignores.
pub const OUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../target/determinism/desktop"
);

pub const USAGE: &str = "\
usage: regress capture [--page <title>]... [--theme <name>]... [--scale <s>]...
                       [--width <px>] [--height <px>] [--out <dir>]
                       [--against <dir>] [--reverse] [--no-png]
capture draws every page of the gallery (or the --page ones) in core,
hoff, dark and light (or the --theme ones) at scales 1, 2, 1.25 and 1.5
(or the --scale ones), 1280x800 logical unless given, twice: by two GPU
contexts of this process. the second frame is classified against the
first under the desktop's rows of the table of requested differences
(equal, requested, silent, resized). --against <dir> also classifies
each frame against the png of the same frame in <dir>, written by
another run (the check across two processes; <dir> may be --out, the
last run there). --reverse draws in the
opposite order (another history of atlases and resizes). each frame's
png (unless --no-png) and report.txt and report.json go to --out
(default target/determinism/desktop). exit 0 when every frame held, 1
when one did not, 3 on an error (no GPU adapter, a png missing in
--against).";

#[derive(Debug, PartialEq)]
pub struct Options {
    /// Page titles as the sidebar shows them; empty, every page.
    pub pages: Vec<String>,
    pub themes: Vec<String>,
    pub scales: Vec<f32>,
    pub width: f32,
    pub height: f32,
    pub out: PathBuf,
    pub against: Option<PathBuf>,
    pub reverse: bool,
    pub png: bool,
}

/// `args` after the command; `Ok(None)` asks for the usage.
pub fn parse(args: &[String]) -> Result<Option<Options>, String> {
    let mut o = Options {
        pages: Vec::new(),
        themes: Vec::new(),
        scales: Vec::new(),
        width: 1280.0,
        height: 800.0,
        out: PathBuf::from(OUT),
        against: None,
        reverse: false,
        png: true,
    };
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        let positive = |raw: &String| match raw.parse::<f32>() {
            Ok(v) if v.is_finite() && v > 0.0 => Ok(v),
            _ => Err(format!("{flag} takes a positive number, got '{raw}'")),
        };
        match flag.as_str() {
            "-h" | "--help" => return Ok(None),
            "--page" => o.pages.push(value()?.clone()),
            "--theme" => o.themes.push(value()?.clone()),
            "--scale" => o.scales.push(positive(value()?)?),
            "--width" => o.width = positive(value()?)?,
            "--height" => o.height = positive(value()?)?,
            "--out" => o.out = PathBuf::from(value()?),
            "--against" => o.against = Some(PathBuf::from(value()?)),
            "--reverse" => o.reverse = true,
            "--no-png" => o.png = false,
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    if o.themes.is_empty() {
        o.themes = THEMES.iter().map(|t| t.to_string()).collect();
    }
    if o.scales.is_empty() {
        o.scales = SCALES.to_vec();
    }
    Ok(Some(o))
}

/// One frame of the capture and what its comparisons found.
#[derive(Debug, Serialize)]
pub struct Cell {
    pub label: String,
    pub page: String,
    pub theme: String,
    pub scale: f32,
    /// Physical px.
    pub size: [u32; 2],
    /// The second context's frame against the first's.
    pub contexts: Verdict,
    /// The first context's frame against the png of `--against`.
    pub processes: Option<Verdict>,
    /// Staging and drawing both frames.
    pub ms: f64,
}

impl Cell {
    pub fn held(&self) -> bool {
        self.contexts.held() && self.processes.as_ref().is_none_or(Verdict::held)
    }
}

/// The frames `o` asks for, scale by scale, theme by theme, page by
/// page (the other way round with `--reverse`). A title matches as the
/// sidebar shows it, whatever the case.
fn keys(o: &Options) -> Result<Vec<Key>, String> {
    let pages: Vec<&str> = if o.pages.is_empty() {
        PAGES.iter().map(|p| p.title).collect()
    } else {
        let find = |t: &String| PAGES.iter().find(|p| p.title.eq_ignore_ascii_case(t));
        o.pages
            .iter()
            .map(|t| {
                find(t)
                    .map(|p| p.title)
                    .ok_or(format!("no page titled '{t}'"))
            })
            .collect::<Result<_, _>>()?
    };
    let mut keys = Vec::new();
    for &scale in &o.scales {
        for theme in &o.themes {
            for page in &pages {
                keys.push(Key {
                    page: page.to_string(),
                    theme: theme.clone(),
                    viewport: Viewport::new(o.width, o.height, scale),
                });
            }
        }
    }
    if o.reverse {
        keys.reverse();
    }
    Ok(keys)
}

/// Stage `key` and draw it through `gpu`.
fn draw(gpu: &mut Gpu, key: &Key) -> Result<Frame, String> {
    let mut page = crate::state::stage(key.viewport, &key.page, &key.theme)?;
    match gpu.frame(&mut page, key.viewport)? {
        Some((frame, _)) => Ok(frame),
        None => Err("no GPU adapter here: nothing to capture".to_string()),
    }
}

/// One frame: drawn by both contexts, written, classified.
fn capture_one(
    key: &Key,
    (a, b): (&mut Gpu, &mut Gpu),
    o: &Options,
    decisions: &[Decision<'_>],
) -> Result<Cell, String> {
    let start = Instant::now();
    let first = draw(a, key)?;
    let second = draw(b, key)?;
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    let scale = key.viewport.scale;
    let label = key.label();
    let png = format!("{label}.png");
    // read before writing: `--against` may name `--out`, the last run
    let processes = match &o.against {
        Some(dir) => Some(classify(
            &read_png(&dir.join(&png))?,
            &first,
            scale,
            decisions,
        )),
        None => None,
    };
    if o.png {
        let path = o.out.join(&png);
        first
            .write_png(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(Cell {
        contexts: classify(&first, &second, scale, decisions),
        processes,
        label,
        page: key.page.clone(),
        theme: key.theme.clone(),
        scale,
        size: [first.width, first.height],
        ms,
    })
}

/// Capture every frame `o` asks for, write the report, print it.
/// `Ok(true)` when every frame held.
pub fn run(o: &Options) -> Result<bool, String> {
    let table = Table::read(Path::new(TABLE))?;
    let decisions = table.decisions("desktop")?;
    let keys = keys(o)?;
    std::fs::create_dir_all(&o.out).map_err(|e| format!("{}: {e}", o.out.display()))?;
    let (mut a, mut b) = (Gpu::default(), Gpu::default());
    let start = Instant::now();
    let mut cells: Vec<Cell> = Vec::with_capacity(keys.len());
    for (i, key) in keys.iter().enumerate() {
        cells.push(capture_one(key, (&mut a, &mut b), o, &decisions)?);
        let next = keys.get(i + 1);
        let group = |k: &Key| (k.viewport.scale.to_bits(), k.theme.clone());
        if next.is_none_or(|n| group(n) != group(key)) {
            println!("{}", summary::group_line(&cells, key));
        }
    }
    let run = summary::Run {
        options: o,
        cells: &cells,
        total: start.elapsed(),
        open: [a.open, b.open],
    };
    let text = summary::text(&run);
    print!("{text}");
    summary::write(&run, &text)?;
    Ok(cells.iter().all(Cell::held))
}

#[cfg(test)]
mod tests;
