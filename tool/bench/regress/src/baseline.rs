//! The baseline of one (page, theme, viewport): what its frame meant and
//! drew when it was last accepted.
//!
//! ```text
//! <dir>/<page>@<theme>@<w>x<h>[@<s>x]/   the scale only when it is not 1
//!   scene.json      the format and hash versions, the key, how the page
//!                   settled, the projection's revision, every layer
//!                   (name, z, hash, node count, bounds) and every
//!                   binding (id, layer, rect, clip)
//!   semantics.json  the projection in the lean form `showcase --render
//!                   --semantics` writes; only for a page that projects
//!   frame.png       the frame; only when asked for (`bless --png`)
//! ```
//!
//! A check reads `scene.json` and nothing else while nothing changed: the
//! revision stands in for the projection (equal revisions are equal
//! snapshots, docs/adr/semantics/0032-semantic-contract-as-pure-projection.md), and the
//! image is decoded only when a layer's hash moved.

use std::fs;
use std::path::{Path, PathBuf};

use accord::Snapshot;
use engine::drawing::headless::{Frame, Viewport};
use serde::{Deserialize, Serialize};

use crate::hash::HASH_VERSION;
use crate::scene::Scene;

/// The version of the baseline layout. Another layout is refused with
/// the advice to bless again.
pub const FORMAT: &str = "regress.baseline.v1";

pub const SCENE: &str = "scene.json";
pub const SEMANTICS: &str = "semantics.json";
pub const IMAGE: &str = "frame.png";

/// What a baseline is of.
#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    /// The page title as the sidebar shows it.
    pub page: String,
    pub theme: String,
    pub viewport: Viewport,
}

impl Key {
    /// The directory name, also the name a report gives the baseline:
    /// `todo@core@1280x800`, `data-table@hoff@390x844@3x`.
    pub fn label(&self) -> String {
        let v = self.viewport;
        let mut label = format!(
            "{}@{}@{}x{}",
            slug(&self.page),
            self.theme,
            v.width,
            v.height
        );
        if v.scale != 1.0 {
            label.push_str(&format!("@{}x", v.scale));
        }
        label
    }
}

/// A page title as a path segment: lowercase ASCII letters and digits,
/// any run of other characters one `-`.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

/// How the page came to rest before its frame was taken.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settle {
    /// Fixed animation steps taken (`showcase::app::stage_page`).
    pub steps: usize,
    /// `false`: the page still moved after the last step (a spinner, a
    /// caret) and its frame is the one at that fixed step.
    pub settled: bool,
}

/// The projection's identity, to compare without reading the file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Projection {
    pub revision: String,
    pub nodes: usize,
}

/// `scene.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub format: String,
    pub hash: String,
    pub page: String,
    pub theme: String,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub settle: Settle,
    /// `None` for a page that projects nothing.
    pub semantics: Option<Projection>,
    #[serde(flatten)]
    pub scene: Scene,
}

impl Meta {
    pub fn new(key: &Key, settle: Settle, semantics: Option<&Snapshot>, scene: Scene) -> Self {
        Self {
            format: FORMAT.to_string(),
            hash: HASH_VERSION.to_string(),
            page: key.page.clone(),
            theme: key.theme.clone(),
            width: key.viewport.width,
            height: key.viewport.height,
            scale: key.viewport.scale,
            settle,
            semantics: semantics.map(|s| Projection {
                revision: s.revision().to_string(),
                nodes: s.len(),
            }),
            scene,
        }
    }

    pub fn key(&self) -> Key {
        Key {
            page: self.page.clone(),
            theme: self.theme.clone(),
            viewport: Viewport::new(self.width, self.height, self.scale),
        }
    }
}

/// A baseline on disk.
#[derive(Debug)]
pub struct Baseline {
    pub dir: PathBuf,
    pub meta: Meta,
}

impl Baseline {
    /// `scene.json` of `dir`, refused when it was written in another
    /// format or hash encoding.
    pub fn read(dir: &Path) -> Result<Self, String> {
        let path = dir.join(SCENE);
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let meta: Meta =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if meta.format != FORMAT || meta.hash != HASH_VERSION {
            return Err(format!(
                "{}: written as {} with {}, this harness reads {FORMAT} with {HASH_VERSION}; \
                 bless it again",
                path.display(),
                meta.format,
                meta.hash
            ));
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            meta,
        })
    }

    pub fn key(&self) -> Key {
        self.meta.key()
    }

    /// The projection's revision as accepted, `None` for a page that
    /// projected nothing.
    pub fn revision(&self) -> Option<&str> {
        self.meta.semantics.as_ref().map(|p| p.revision.as_str())
    }

    /// The projection the baseline accepted, read from `semantics.json`.
    pub fn semantics(&self) -> Result<Option<Snapshot>, String> {
        if self.meta.semantics.is_none() {
            return Ok(None);
        }
        let path = self.dir.join(SEMANTICS);
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn has_image(&self) -> bool {
        self.dir.join(IMAGE).is_file()
    }

    /// `frame.png` decoded to RGBA, `None` when the baseline keeps none.
    pub fn image(&self) -> Result<Option<Frame>, String> {
        let path = self.dir.join(IMAGE);
        if !path.is_file() {
            return Ok(None);
        }
        read_png(&path).map(Some)
    }
}

/// A png decoded to RGBA rows, as a frame.
pub fn read_png(path: &Path) -> Result<Frame, String> {
    let img = image::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let rgba = img.to_rgba8();
    Ok(Frame {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}

/// Every baseline directory under `root` (one holding `scene.json`), in
/// name order.
pub fn list(root: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?;
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join(SCENE).is_file())
        .collect();
    dirs.sort();
    Ok(dirs)
}

/// What a write does with `frame.png`.
pub enum Image<'a> {
    /// Write this frame.
    Write(&'a Frame),
    /// Remove the file if there is one.
    Drop,
}

/// Write the baseline of `meta` under `root`: `scene.json`, then
/// `semantics.json` when the page projects (removed when it no longer
/// does), then the image as asked. Returns the directory.
pub fn write(
    root: &Path,
    meta: &Meta,
    semantics: Option<&str>,
    image: Image<'_>,
) -> Result<PathBuf, String> {
    let dir = root.join(meta.key().label());
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let io = |path: &Path, e: std::io::Error| format!("{}: {e}", path.display());
    let mut text = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    text.push('\n');
    let scene = dir.join(SCENE);
    fs::write(&scene, text).map_err(|e| io(&scene, e))?;
    let sem = dir.join(SEMANTICS);
    match semantics {
        Some(json) => fs::write(&sem, format!("{json}\n")).map_err(|e| io(&sem, e))?,
        None if sem.is_file() => fs::remove_file(&sem).map_err(|e| io(&sem, e))?,
        None => {}
    }
    let png = dir.join(IMAGE);
    match image {
        Image::Write(frame) => frame
            .write_png(&png)
            .map_err(|e| format!("{}: {e}", png.display()))?,
        Image::Drop if png.is_file() => fs::remove_file(&png).map_err(|e| io(&png, e))?,
        Image::Drop => {}
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{BindRecord, LayerRecord};

    fn key(page: &str, scale: f32) -> Key {
        Key {
            page: page.to_string(),
            theme: "core".to_string(),
            viewport: Viewport::new(1280.0, 800.0, scale),
        }
    }

    #[test]
    fn a_label_names_page_theme_and_viewport_and_the_scale_when_not_one() {
        assert_eq!(key("Todo", 1.0).label(), "todo@core@1280x800");
        assert_eq!(
            key("Data Table", 2.0).label(),
            "data-table@core@1280x800@2x"
        );
        assert_eq!(
            key("Tabs / Pills", 1.5).label(),
            "tabs-pills@core@1280x800@1.5x"
        );
    }

    fn meta() -> Meta {
        let scene = Scene {
            layers: vec![LayerRecord {
                name: "todo".to_string(),
                z: 0,
                hash: 0x0123_4567_89ab_cdef,
                nodes: 3,
                bounds: Some([0.0, 0.0, 10.0, 10.0]),
            }],
            bindings: vec![BindRecord {
                id: "todo".to_string(),
                layer: "todo".to_string(),
                rect: [1.5, 2.0, 3.0, 4.0],
                clip: None,
            }],
        };
        Meta::new(
            &key("Todo", 1.0),
            Settle {
                steps: 2,
                settled: true,
            },
            None,
            scene,
        )
    }

    #[test]
    fn a_written_baseline_reads_back_and_the_image_follows_the_request() {
        let root = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => panic!("{e}"),
        };
        let meta = meta();
        let dir = match write(root.path(), &meta, None, Image::Drop) {
            Ok(d) => d,
            Err(e) => panic!("{e}"),
        };
        assert!(dir.ends_with("todo@core@1280x800"));
        let back = match Baseline::read(&dir) {
            Ok(b) => b,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(back.meta, meta);
        assert!(matches!(back.semantics(), Ok(None)));
        assert_eq!(list(root.path()).ok(), Some(vec![dir.clone()]));
        assert!(!dir.join(SEMANTICS).exists() && !back.has_image());
        assert!(matches!(back.image(), Ok(None)));
        let frame = Frame {
            width: 2,
            height: 1,
            rgba: vec![1, 2, 3, 255, 4, 5, 6, 255],
        };
        if let Err(e) = write(root.path(), &meta, None, Image::Write(&frame)) {
            panic!("{e}");
        }
        assert_eq!(back.image().ok().flatten(), Some(frame));
        if let Err(e) = write(root.path(), &meta, None, Image::Drop) {
            panic!("{e}");
        }
        assert!(!back.has_image());
    }

    #[test]
    fn another_format_or_hash_encoding_is_refused_with_the_advice() {
        let root = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => panic!("{e}"),
        };
        let mut meta = meta();
        meta.hash = "regress.layer.v0".to_string();
        let dir = match write(root.path(), &meta, None, Image::Drop) {
            Ok(d) => d,
            Err(e) => panic!("{e}"),
        };
        match Baseline::read(&dir) {
            Err(e) => assert!(e.contains("bless it again"), "{e}"),
            Ok(_) => panic!("an old hash encoding was read"),
        }
    }
}
