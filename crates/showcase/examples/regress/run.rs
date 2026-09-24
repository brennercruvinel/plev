//! The commands over a set of baselines: which (page, theme, viewport)
//! a request names, staging each one, and writing what `bless` accepts.

use std::path::Path;

use engine::headless::Viewport;
use sem::Snapshot;
use showcase::app::{RenderError, StagedPage, semantics_json, stage_page};
use showcase::view::PAGES;

use crate::baseline::{self, Baseline, Image, Key, Meta, Settle};
use crate::cli::{self, Command, Request};
use crate::scene::{self, Scene};

pub fn run(r: &Request) -> Result<(), String> {
    match r.command {
        Command::Bless => bless(r),
    }
}

/// A page staged for a baseline: its frame, its projection, its scene.
pub struct Staged {
    pub key: Key,
    pub page: StagedPage,
    pub semantics: Option<Snapshot>,
    pub scene: Scene,
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
    let page = stage_page(key.viewport, &key.page, &key.theme, None).map_err(|e| e.to_string())?;
    let semantics = match page.semantics() {
        Ok(s) => Some(s),
        Err(RenderError::NoSemantics(_)) => None,
        Err(e) => return Err(e.to_string()),
    };
    let scene = scene::capture(page.compositor(), key.viewport, with_bounds);
    let key = Key {
        page: page.title().to_string(),
        ..key.clone()
    };
    Ok(Staged {
        key,
        page,
        semantics,
        scene,
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
/// the defaults.
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
    for key in &keys {
        let staged = stage(key, true)?;
        let dir = accept(&r.dir, &staged)?;
        println!("{}", summary(&staged, &dir));
    }
    Ok(())
}

/// Write the baseline of `staged` under `root`.
fn accept(root: &Path, staged: &Staged) -> Result<std::path::PathBuf, String> {
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
    baseline::write(root, &meta, json.as_deref(), Image::Drop)
}

/// One line on what a baseline holds.
fn summary(staged: &Staged, dir: &Path) -> String {
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
    format!(
        "wrote {}: {} layers ({}), {} bound controls, {semantics}, {settle}",
        dir.display(),
        layers.len(),
        layers.join(", "),
        staged.scene.bindings.len(),
    )
}
