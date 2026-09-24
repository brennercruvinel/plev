//! The commands over a set of baselines: which (page, theme, viewport)
//! a request names, staging each one, comparing it with its baseline,
//! and writing what `bless` accepts.

use std::path::Path;
use std::time::{Duration, Instant};

use engine::gpu::RenderConfig;
use engine::headless::{Frame, HeadlessError, Renderer, Viewport};
use sem::Snapshot;
use showcase::app::semantics_json;
use showcase::view::PAGES;

use crate::baseline::{self, Baseline, Image, Key, Meta, Settle};
use crate::cli::{self, Command, Request};
use crate::compare::{self, Status};
use crate::pixels::{self, Pixels};
use crate::report;
use crate::scene::{self, Scene};
use crate::state::{self, Page};

/// `Ok(true)` when every baseline held (a bless always holds).
pub fn run(r: &Request) -> Result<bool, String> {
    match r.command {
        Command::Check => check(r),
        Command::Bless => bless(r).map(|()| true),
    }
}

/// A page staged for a baseline (a gallery page at launch, or a page in
/// a state, `state`): its frame, its projection, its scene.
pub struct Staged {
    pub key: Key,
    pub page: Page,
    pub semantics: Option<Snapshot>,
    pub scene: Scene,
    /// Staging the page and projecting it.
    pub staged: Duration,
    /// Naming and hashing its layers, reading its bindings.
    pub captured: Duration,
}

impl Staged {
    pub fn settle(&self) -> Settle {
        Settle {
            steps: self.page.steps(),
            settled: self.page.settled(),
        }
    }
}

/// Stage `key` and read what it means and draws. `with_bounds` measures
/// every layer's extent (a bless; a check measures only what changed).
pub fn stage(key: &Key, with_bounds: bool) -> Result<Staged, String> {
    let start = Instant::now();
    let page = state::stage(key.viewport, &key.page, &key.theme)?;
    let semantics = page.semantics()?;
    let staged = start.elapsed();
    let start = Instant::now();
    let scene = scene::capture(page.compositor(), key.viewport, with_bounds);
    let captured = start.elapsed();
    let key = Key {
        page: page.title(),
        ..key.clone()
    };
    Ok(Staged {
        key,
        page,
        semantics,
        scene,
        staged,
        captured,
    })
}

/// The baselines under `dir` a request selects: every one, narrowed by
/// `--page`, `--theme` and the viewport flags that were given.
pub fn existing(r: &Request) -> Result<Vec<Baseline>, String> {
    if !r.dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for dir in baseline::list(&r.dir)? {
        let b = Baseline::read(&dir)?;
        let k = b.key();
        let page = r.pages.is_empty() || r.pages.iter().any(|p| p.eq_ignore_ascii_case(&k.page));
        let theme = r.theme.as_ref().is_none_or(|t| *t == k.theme);
        let view = r.width.is_none_or(|w| w == k.viewport.width)
            && r.height.is_none_or(|h| h == k.viewport.height)
            && r.scale.is_none_or(|s| s == k.viewport.scale);
        if page && theme && view {
            out.push(b);
        }
    }
    Ok(out)
}

/// The keys `--page` or `--all` name, at the theme and viewport given or
/// the defaults. `--all` is every page of the gallery; a page in a state
/// (`state::STATES`) is named by `--page`.
fn named(r: &Request) -> Vec<Key> {
    let viewport = Viewport::new(
        r.width.unwrap_or(cli::DEFAULT_WIDTH),
        r.height.unwrap_or(cli::DEFAULT_HEIGHT),
        r.scale.unwrap_or(1.0),
    );
    let theme = r.theme.clone().unwrap_or(cli::DEFAULT_THEME.to_string());
    let pages: Vec<String> = if r.all {
        PAGES.iter().map(|p| p.title.to_string()).collect()
    } else {
        r.pages.clone()
    };
    pages
        .into_iter()
        .map(|page| Key {
            page,
            theme: theme.clone(),
            viewport,
        })
        .collect()
}

/// The headless renderer, opened on the first frame asked of it and
/// kept for the next (a renderer that drew other pages draws a page the
/// same, `showcase::app` tests it). It counts what it draws.
#[derive(Default)]
pub struct Gpu {
    renderer: Option<Renderer>,
    absent: bool,
    /// Frames drawn.
    pub frames: usize,
    /// What opening the renderer took, once it was opened.
    pub open: Option<Duration>,
}

impl Gpu {
    /// Whether a renderer was ever opened.
    pub fn opened(&self) -> bool {
        self.renderer.is_some() || self.absent
    }

    /// The frame of `page` through the window's frame path and what
    /// drawing and reading it back took (opening the renderer apart),
    /// `None` on a machine without an adapter.
    pub fn frame(
        &mut self,
        page: &mut Page,
        viewport: Viewport,
    ) -> Result<Option<(Frame, Duration)>, String> {
        if self.absent {
            return Ok(None);
        }
        let start = Instant::now();
        let renderer = match self.renderer.take() {
            Some(mut r) => {
                if r.viewport() != viewport {
                    r.resize(viewport).map_err(|e| e.to_string())?;
                }
                r
            }
            None => match Renderer::new(viewport, RenderConfig::default()) {
                Ok(r) => {
                    self.open = Some(start.elapsed());
                    r
                }
                Err(HeadlessError::NoAdapter) => {
                    self.absent = true;
                    return Ok(None);
                }
                Err(e) => return Err(e.to_string()),
            },
        };
        let renderer = self.renderer.insert(renderer);
        let start = Instant::now();
        let frame = page.render(renderer)?;
        self.frames += 1;
        Ok(Some((frame, start.elapsed())))
    }
}

/// What checking one baseline found.
pub struct Outcome {
    pub label: String,
    pub delta: compare::Delta,
    /// The page's projection now.
    pub semantics: Option<Snapshot>,
    /// The page's scene; a layer that moved carries its bounds.
    pub now: Scene,
    pub pixels: Pixels,
    /// Staging the page and projecting it.
    pub staged: Duration,
    /// Naming and hashing its layers, reading its bindings.
    pub captured: Duration,
    /// Comparing it with the baseline.
    pub compared: Duration,
    /// Drawing the frame and reading it back, when pixels were asked
    /// (opening the renderer apart, `Gpu::open`).
    pub rendered: Option<Duration>,
    /// Decoding the baseline image.
    pub decoded: Option<Duration>,
    /// Comparing the pixels inside the regions that moved.
    pub diffed: Option<Duration>,
}

/// Stage the page of `b` and compare it with `b`: the meaning, then the
/// layers and bindings, and, only when a layer's hash moved and the
/// baseline keeps an image, the pixels inside the regions of the layers
/// that moved.
pub fn check_one(b: &Baseline, gpu: &mut Gpu) -> Result<Outcome, String> {
    let key = b.key();
    let mut staged = stage(&key, false)?;
    let start = Instant::now();
    let mut delta = compare::delta(
        &b.meta.scene,
        b.revision(),
        || b.semantics(),
        &staged.scene,
        staged.semantics.as_ref(),
    )?;
    let compared = start.elapsed();
    let mut regions = Vec::new();
    for layer in delta
        .layers
        .iter_mut()
        .filter(|l| l.status != Status::Unchanged)
    {
        let now = scene::bounds_of(staged.page.compositor(), key.viewport, &layer.name);
        if let Some(after) = layer.after.as_mut() {
            after.bounds = now;
        }
        if let Some(l) = staged
            .scene
            .layers
            .iter_mut()
            .find(|l| l.name == layer.name)
        {
            l.bounds = now;
        }
        let before = layer.before.as_ref().and_then(|l| l.bounds);
        match (before, now) {
            (Some(a), Some(b)) => regions.push(scene::union(a, b)),
            (Some(r), None) | (None, Some(r)) => regions.push(r),
            (None, None) => {}
        }
    }
    let mut out = Outcome {
        label: key.label(),
        delta,
        semantics: staged.semantics.clone(),
        now: staged.scene.clone(),
        pixels: Pixels::NotNeeded,
        staged: staged.staged,
        captured: staged.captured,
        compared,
        rendered: None,
        decoded: None,
        diffed: None,
    };
    if !out.delta.drawn_changed() {
        return Ok(out);
    }
    if !b.has_image() {
        out.pixels = Pixels::NoBaselineImage;
        return Ok(out);
    }
    let Some((frame, drawn)) = gpu.frame(&mut staged.page, key.viewport)? else {
        out.pixels = Pixels::NoAdapter;
        return Ok(out);
    };
    out.rendered = Some(drawn);
    let start = Instant::now();
    let Some(base) = b.image()? else {
        out.pixels = Pixels::NoBaselineImage;
        return Ok(out);
    };
    out.decoded = Some(start.elapsed());
    let start = Instant::now();
    let changed = pixels::diff(&base, &frame, &regions, key.viewport.scale)?;
    out.diffed = Some(start.elapsed());
    out.pixels = Pixels::Diffed(changed);
    Ok(out)
}

/// Check every baseline the request selects. A baseline where nothing
/// moved ends at the comparison, with no frame drawn.
fn check(r: &Request) -> Result<bool, String> {
    let baselines = existing(r)?;
    if baselines.is_empty() {
        return Err(format!("no baseline under {} to check", r.dir.display()));
    }
    let start = Instant::now();
    let mut gpu = Gpu::default();
    let mut reports = Vec::new();
    for b in &baselines {
        let o = check_one(b, &mut gpu)?;
        let report = report::build(&o, &b.meta.scene);
        print!("{}", report::text(&report));
        reports.push(report);
    }
    let total = start.elapsed();
    let changed = reports.iter().filter(|r| r.changed).count();
    let gpu_line = if let Some(open) = gpu.open {
        format!("{} frames drawn, GPU opened in {}", gpu.frames, ms(open))
    } else if gpu.opened() {
        "no GPU adapter here, no frame drawn".to_string()
    } else {
        "no frame drawn, no GPU opened".to_string()
    };
    println!(
        "checked {} baselines in {}: {} unchanged, {changed} changed; {gpu_line}",
        reports.len(),
        ms(total),
        reports.len() - changed,
    );
    if let Some(path) = &r.json {
        report::write_json(path, &reports, gpu.frames, gpu.open, total)?;
    }
    Ok(changed == 0)
}

/// A duration in milliseconds with one decimal.
pub fn ms(d: Duration) -> String {
    format!("{:.1} ms", d.as_secs_f64() * 1000.0)
}

fn bless(r: &Request) -> Result<(), String> {
    let keys: Vec<Key> = if r.all || !r.pages.is_empty() {
        named(r)
    } else {
        existing(r)?.iter().map(Baseline::key).collect()
    };
    if keys.is_empty() {
        return Err(format!(
            "no baseline under {} to bless; name pages with --page or --all",
            r.dir.display()
        ));
    }
    let mut gpu = Gpu::default();
    for key in &keys {
        // what is being accepted: the check of the baseline it replaces
        let before = Baseline::read(&r.dir.join(key.label())).ok();
        match &before {
            Some(b) => {
                let o = check_one(b, &mut gpu)?;
                let report = report::build(&o, &b.meta.scene);
                if report.changed {
                    print!("accept   {}\n{}", report.baseline, report::body(&report));
                } else {
                    println!("same     {}", report.baseline);
                }
            }
            None => println!("new      {}", key.label()),
        }
        let mut staged = stage(key, true)?;
        // an image stays an image: a baseline that keeps one gets a new
        // one unless --no-png drops it
        let png = r
            .png
            .unwrap_or_else(|| before.as_ref().is_some_and(Baseline::has_image));
        let frame = if png {
            let Some((frame, _)) = gpu.frame(&mut staged.page, key.viewport)? else {
                return Err(format!(
                    "{}: the baseline keeps an image and this machine has no GPU \
                     adapter to draw it; --no-png drops the image",
                    staged.key.label()
                ));
            };
            Some(frame)
        } else {
            None
        };
        let dir = accept(&r.dir, &staged, frame.as_ref())?;
        println!("{}", summary(&staged, &dir, frame.is_some()));
    }
    Ok(())
}

/// Write the baseline of `staged` under `root`, with `image` as its
/// `frame.png` or none.
pub fn accept(
    root: &Path,
    staged: &Staged,
    image: Option<&Frame>,
) -> Result<std::path::PathBuf, String> {
    let meta = Meta::new(
        &staged.key,
        staged.settle(),
        staged.semantics.as_ref(),
        staged.scene.clone(),
    );
    let json = match &staged.semantics {
        Some(s) => Some(semantics_json(s, false).map_err(|e| e.to_string())?),
        None => None,
    };
    let image = match image {
        Some(frame) => Image::Write(frame),
        None => Image::Drop,
    };
    baseline::write(root, &meta, json.as_deref(), image)
}

/// One line on what a baseline holds.
fn summary(staged: &Staged, dir: &Path, image: bool) -> String {
    let layers: Vec<String> = staged
        .scene
        .layers
        .iter()
        .map(|l| format!("{} {:016x}", l.name, l.hash))
        .collect();
    let semantics = match &staged.semantics {
        Some(s) => format!("{} semantic nodes (revision {})", s.len(), s.revision()),
        None => "no semantic projection".to_string(),
    };
    let settle = if staged.page.settled() {
        format!("settled after {} steps", staged.page.steps())
    } else {
        format!("still moving, frozen at step {}", staged.page.steps())
    };
    let image = if image { ", with its image" } else { "" };
    format!(
        "wrote {}: {} layers ({}), {} bound controls, {semantics}, {settle}{image}",
        dir.display(),
        layers.len(),
        layers.join(", "),
        staged.scene.bindings.len(),
    )
}
