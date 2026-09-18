//! urnaui view: winit-free, gpu-free application state and scene building.
//!
//! `UrnauiView` is the thin shell every platform compiles: theme, size,
//! toasts. The explorer (tabs, screens, backend worker, opened-database
//! state) lives in [`explorer`] and compiles on every target — desktop
//! talks to the mmap worker thread, web to the inline worker over the
//! portable reader. Screens own widget state only; data arrives from the
//! worker and stays central so screens stay disposable.
//!
//! Invalidation contract (render-on-demand): every `handle_*` returns
//! `true` when visible state changed; `app.rs` requests a redraw on it.

mod chunks;
mod explorer;
mod field;
mod graph;
mod open;
mod overview;
mod search;
mod stats;

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::theme::Theme;
use engine::ui::widgets::{Rect, ToastManager, WidgetEvent, rounded_rect, rounded_rect_stroke};

/// Non-character editing keys bridged from winit by `keys.rs` (the view
/// stays winit-free). Enter submits; Tab is reserved for focus traversal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    Tab,
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
}

/// Top-level screens, all reachable from the tab strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Open,
    Overview,
    Search,
    Chunks,
    Graph,
    Stats,
}

impl Screen {
    /// Screens reachable from the tab strip, in order.
    pub const TABS: [Screen; 6] = [
        Screen::Open,
        Screen::Overview,
        Screen::Search,
        Screen::Chunks,
        Screen::Graph,
        Screen::Stats,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Screen::Open => "Open",
            Screen::Overview => "Overview",
            Screen::Search => "Search",
            Screen::Chunks => "Chunks",
            Screen::Graph => "Graph",
            Screen::Stats => "Stats",
        }
    }
}

/// Actions screens bubble up to the shell (worker commands, clipboard,
/// navigation). Keeps screens free of worker/clipboard handles.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    None,
    /// Open the .urna at this path (typed, pasted, recent, dropped).
    OpenPath(String),
    /// Run a search: the raw query string, whether it is a JSON vector,
    /// the selected mode and k. The shell parses/validates against the db.
    RunSearch {
        query: String,
        is_vector: bool,
        mode: crate::model::types::SearchMode,
        k: i32,
    },
    /// Copy `text` to the clipboard and toast about it.
    Copy {
        text: String,
        what: String,
    },
    /// Switch screens (empty-state CTAs).
    Goto(Screen),
    /// Run the latency benchmark on the worker.
    RunBenchmark {
        n_queries: usize,
        k: i32,
    },
    /// Open the platform file picker (native dialog on desktop, the
    /// browser's input on the web).
    PickFile,
    /// Re-run the file's integrity checks (`urna validate`).
    Validate,
    /// Export inlined blob `index` to a file the user picks.
    ExportBlob(usize),
    /// Decode the frame preview of chunk `ordinal` (media corpora).
    LoadFrame(usize),
}

/// Read-only view over the central chunk state every screen resolves
/// hits and selections through: chunk_id → ordinal, ordinal → text /
/// span / decoded frame. Everything is optional until the worker has
/// delivered it, and the screens degrade to ids + offsets meanwhile.
pub struct ChunkLookup<'a> {
    pub ids: &'a [String],
    pub index: Option<&'a std::collections::HashMap<String, usize>>,
    pub chunks: Option<&'a crate::model::types::ChunksData>,
    pub frames:
        &'a std::collections::HashMap<usize, Result<engine::gpu::image::ImageHandle, String>>,
}

impl ChunkLookup<'_> {
    pub fn ordinal(&self, chunk_id: &str) -> Option<usize> {
        self.index?.get(chunk_id).copied()
    }

    pub fn text(&self, ordinal: usize) -> Option<&str> {
        self.chunks?.texts.get(ordinal).map(String::as_str)
    }

    /// First line of the canonical text: the card name in a forge item
    /// corpus, the opening sentence in a text corpus.
    pub fn first_line(&self, ordinal: usize) -> Option<&str> {
        self.text(ordinal).and_then(|t| t.lines().next())
    }

    pub fn meta(&self, ordinal: usize) -> Option<&crate::model::types::ChunkMeta> {
        self.chunks?.metas.get(ordinal)
    }

    /// The decoded frame, `Some(Err)` when the decode failed, `None` while
    /// nothing was requested or the decode is in flight.
    pub fn frame(
        &self,
        ordinal: usize,
    ) -> Option<&Result<engine::gpu::image::ImageHandle, String>> {
        self.frames.get(&ordinal)
    }

    /// Whether this chunk lives in a media blob (frame preview possible).
    pub fn is_media(&self, ordinal: usize) -> bool {
        self.meta(ordinal).is_some_and(|m| m.blob.is_some())
    }
}

/// `source · frame N` for a media span, `source · start–end` otherwise:
/// the one spelling of a chunk's location every screen uses.
pub(crate) fn span_label(source_uri: &str, start: u64, end: u64, media: bool) -> String {
    let source = source_uri.trim_start_matches("media://");
    if media && end == start + 1 {
        format!("{source} · frame {start}")
    } else {
        format!("{source} · {start}–{end}")
    }
}

/// Parse a `urna://<content_hash>/<chunk_id>` citation into its parts.
pub(crate) fn parse_citation(s: &str) -> Option<(&str, &str)> {
    let rest = s.trim().strip_prefix("urna://")?;
    let (content_hash, chunk_id) = rest.split_once('/')?;
    (!content_hash.is_empty() && !chunk_id.is_empty()).then_some((content_hash, chunk_id))
}

pub struct UrnauiView {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
    pub theme: Theme,
    pub toasts: ToastManager,
    explorer: explorer::Explorer,
}

impl UrnauiView {
    pub fn new(width: f32, height: f32) -> Self {
        let theme = Theme::hoff();
        Self {
            width,
            height,
            scale_factor: 1.0,
            toasts: ToastManager::new(),
            explorer: explorer::Explorer::new(&theme),
            theme,
        }
    }

    /// Install the event-loop wake the worker fires after each result
    /// (see `app.rs`); without it results wait for the next input event.
    pub fn set_wake(&self, wake: Box<dyn Fn() + Send + 'static>) {
        self.explorer.set_wake(wake);
    }

    pub fn resize(&mut self, width: f32, height: f32, scale_factor: f32) {
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
        self.explorer.resize(width, height);
    }

    /// Ask the worker to open a .urna file (desktop launch argument).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_database(&mut self, path: std::path::PathBuf) {
        self.explorer.open_database(path);
    }

    /// A file was dropped on the window.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn file_dropped(&mut self, path: std::path::PathBuf) {
        self.explorer.set_file_hover(false);
        self.explorer.open_database(path);
        self.toasts.push(
            "opening dropped file…",
            engine::theme::Intent::Informational,
            &self.theme,
        );
    }

    /// Drag-and-drop hover feedback on the Open screen.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_file_hover(&mut self, hovering: bool) {
        self.explorer.set_file_hover(hovering);
    }

    /// Drain pending worker events. Returns `true` when state changed and
    /// a redraw is needed.
    pub fn poll_backend(&mut self) -> bool {
        let theme = self.theme.clone();
        self.explorer.poll_backend(&mut self.toasts, &theme)
    }

    /// Character input (winit `Key::Character`), routed to the focused
    /// field of the active screen.
    pub fn handle_key(&mut self, key: &str) -> bool {
        self.explorer.handle_key(key)
    }

    /// Clipboard paste into the focused field.
    pub fn handle_paste(&mut self, text: &str) -> bool {
        self.explorer.handle_paste(text)
    }

    /// Global shortcut dispatch (Cmd/Ctrl+O, Cmd/Ctrl+1..=6): the shell
    /// turns winit keys into engine `Keystroke`s; the view never sees
    /// winit types.
    pub fn handle_keystroke(&mut self, keystroke: &engine::actions::Keystroke) -> bool {
        self.explorer.handle_keystroke(keystroke)
    }

    /// Non-character editing keys forwarded by the platform shell.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        self.explorer.handle_edit_key(key)
    }

    /// Escape semantics: close the open select dropdown first; `false`
    /// when nothing was open (the shell may quit).
    pub fn close_top_overlay(&mut self) -> bool {
        self.explorer.close_top_overlay()
    }

    /// Route a pointer event. Returns `true` if a redraw is needed.
    pub fn handle_event(&mut self, event: &WidgetEvent) -> bool {
        // Toasts float above everything (click-to-dismiss).
        if self
            .toasts
            .handle_event(event, self.width, self.height)
            .clicked
        {
            return true;
        }
        let theme = self.theme.clone();
        self.explorer.handle_event(event, &mut self.toasts, &theme)
    }

    /// Advance animations. Returns `true` while anything is moving
    /// (toasts, cursor blink, scrollbar fades) so frames keep coming.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = self.toasts.tick(dt);
        animating |= self.explorer.tick(dt);
        animating
    }

    pub fn render(&mut self, c: &mut Compositor) {
        c.begin_frame();
        self.explorer.render(c, &self.theme);
        self.explorer
            .render_toasts(c, &self.toasts, &self.theme, self.width, self.height);
    }

    /// Bytes from the web file picker (wasm only; the desktop opens by
    /// path).
    #[cfg(target_arch = "wasm32")]
    pub fn file_picked(&mut self, name: String, bytes: Vec<u8>) {
        self.explorer.open_bytes(name, bytes);
    }

    /// The Open screen's file-picker button was clicked (wasm only).
    #[cfg(target_arch = "wasm32")]
    pub fn take_pick_request(&mut self) -> bool {
        self.explorer.take_pick_request()
    }
}

// ---------------------------------------------------------------------------
// Shared drawing helpers for the screen modules
// ---------------------------------------------------------------------------

/// Push a single-line text node to the default layer.
pub(crate) fn text(
    c: &mut Compositor,
    s: &str,
    size: f32,
    weight: u16,
    x: f32,
    y: f32,
    color: [f32; 4],
) {
    c.push(SceneNode::Text {
        key: TextNodeKey::new(s, size, size * 1.4, None).with_weight(weight),
        x,
        y,
        color,
    });
}

/// Uppercase group label — the HOFF accordion head (12/600).
pub(crate) fn group_label(c: &mut Compositor, s: &str, x: f32, y: f32, theme: &Theme) {
    text(c, s, 12.0, 600, x, y, theme.glass.text_placeholder.0);
}

/// Soft panel container — HOFF list card: radius 20, faint glass fill,
/// soft edge.
pub(crate) fn panel(c: &mut Compositor, rect: Rect, theme: &Theme) {
    c.push(rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        theme.glass.surface.0,
    ));
    c.push(rounded_rect_stroke(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        theme.glass.edge_soft.0,
        1.0,
    ));
}

/// A chunk id rendered short: strip the `sha256:` prefix, keep 12 hex
/// chars (fixed char count — hashes are fixed-width, no shaping needed).
pub(crate) fn short_id(chunk_id: &str) -> String {
    let hex = chunk_id.strip_prefix("sha256:").unwrap_or(chunk_id);
    let short: String = hex.chars().take(12).collect();
    format!("{short}…")
}

pub(crate) fn fmt_bytes(n: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut v = n as f64;
    let mut unit = 0;
    while v >= 1024.0 && unit < UNITS.len() - 1 {
        v /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} B")
    } else {
        format!("{v:.1} {}", UNITS[unit])
    }
}

// ---------------------------------------------------------------------------
// Headless view tests (no GPU: scenes build into a plain compositor)
// ---------------------------------------------------------------------------

/// Synthetic `OpenedDbView` for screen tests (3 chunks, 4 dims, no
/// optional sections). Kept crate-visible so every screen module shares it.
#[cfg(test)]
pub(crate) mod fixtures {
    use std::path::PathBuf;

    use crate::model::types::{
        ChunkMeta, ChunksData, InspectView, ManifestView, OpenedDbView, SectionInfo,
    };

    pub fn fake_db() -> Box<OpenedDbView> {
        let manifest = ManifestView {
            embedding_model: "demo-model".into(),
            embedding_dim: 4,
            n_chunks: 3,
            chunker_version: "demo-chunker/1".into(),
            model_hash: format!("sha256:{}", "0".repeat(64)),
            dtype: "float32".into(),
            metric: "cosine".into(),
            ..Default::default()
        };
        let inspect = InspectView {
            magic: "URNA".into(),
            version_major: 1,
            version_minor: 0,
            format_version: 1,
            schema_version: 1,
            embedding_dim: 4,
            n_chunks: 3,
            n_embeddings: 3,
            file_size: 4096,
            manifest,
            sections: vec![
                SectionInfo {
                    section_id: 1,
                    name: "chunk_ids".into(),
                    encoding: 0,
                    offset: 128,
                    size: 256,
                    checksum: "ab".into(),
                },
                SectionInfo {
                    section_id: 4,
                    name: "embeddings".into(),
                    encoding: 0,
                    offset: 448,
                    size: 48,
                    checksum: "cd".into(),
                },
            ],
            blobs: Vec::new(),
            spaces: Vec::new(),
            file_hash: format!("sha256:{}", "1".repeat(64)),
            content_hash: format!("sha256:{}", "2".repeat(64)),
            simd_backend: "neon".into(),
        };
        Box::new(OpenedDbView {
            path: PathBuf::from("/tmp/demo.urna"),
            inspect,
            chunk_ids: (0..3)
                .map(|i| format!("sha256:{}{}", i, "0".repeat(63)))
                .collect(),
            has_ann: false,
            has_bm25: false,
            has_graph: false,
            has_spaces: false,
            space_names: vec![],
            has_blob_data: false,
            graph_nodes: None,
        })
    }

    /// The fake db as a media corpus: one inlined av1 blob, a vision
    /// space, and every chunk mapped to one frame of the blob.
    pub fn fake_media_db() -> Box<OpenedDbView> {
        use crate::model::types::{BlobInfo, SpaceInfo};
        let mut db = fake_db();
        db.inspect.blobs = vec![BlobInfo {
            content_hash: format!("sha256:{}", "3".repeat(64)),
            original_uri: "media://cards-av1.mp4".into(),
            byte_len: 4096,
            inlined: true,
        }];
        db.inspect.spaces = vec![SpaceInfo {
            name: "clip-vit-b32".into(),
            space_index: 1,
            dim: 512,
            dtype: "int8".into(),
            model_hash: format!("sha256:{}", "4".repeat(64)),
            n_vectors: 3,
            band_bytes: 1536,
        }];
        db.inspect.manifest.capabilities_ext = Some(crate::model::types::CapabilitiesExtView {
            supports_multimodal: Some(true),
            graph_present: None,
            blobs_present: Some(true),
        });
        db.has_spaces = true;
        db.space_names = vec!["clip-vit-b32".into()];
        db.has_blob_data = true;
        db
    }

    /// Chunks of [`fake_media_db`]: each text maps to frame `i` of blob 0.
    pub fn fake_media_chunks() -> ChunksData {
        use crate::model::types::BlobSpan;
        let mut data = fake_chunks();
        for (i, m) in data.metas.iter_mut().enumerate() {
            m.source_uri = "media://cards-av1.mp4".into();
            m.offset_start = i as u64;
            m.offset_end = i as u64 + 1;
            m.blob = Some(BlobSpan {
                blob_index: 0,
                start: i as u64,
                end: i as u64 + 1,
            });
        }
        data
    }

    pub fn fake_chunks() -> ChunksData {
        ChunksData {
            texts: vec![
                "alpha chunk text".into(),
                "beta chunk text".into(),
                "gamma chunk text".into(),
            ],
            metas: (0..3)
                .map(|i| ChunkMeta {
                    source_uri: "corpus.txt".into(),
                    offset_start: i * 8,
                    offset_end: i * 8 + 5,
                    blob: None,
                })
                .collect(),
        }
    }

    /// Same fake db but advertising a graph section (for the Graph tab).
    pub fn fake_db_with_graph() -> Box<OpenedDbView> {
        let mut db = fake_db();
        db.has_graph = true;
        db.graph_nodes = Some(3);
        db
    }

    /// A small laid-out graph over the fake db's 3 chunks (0→1→2 chain).
    pub fn fake_graph_scene() -> engine::graph::GraphScene {
        let spec = engine::graph::GraphSpec {
            n_nodes: 3,
            edges: vec![
                engine::graph::GraphEdge {
                    from: 0,
                    to: 1,
                    kind: 0,
                },
                engine::graph::GraphEdge {
                    from: 1,
                    to: 2,
                    kind: 0,
                },
            ],
        };
        engine::graph::compute_layout(&engine::graph::GraphData::from_spec(&spec), 1000.0, 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_the_open_screen_with_the_hoff_theme() {
        let view = UrnauiView::new(1200.0, 800.0);
        // Page frame is the HOFF #444444.
        assert_eq!(view.theme.colors.bg.0, engine::theme::hoff::PAGE_BG.0);
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(view.explorer.screen(), Screen::Open);
    }

    #[test]
    fn render_builds_a_scene_without_gpu_at_two_widths() {
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut view = UrnauiView::new(w, h);
            let mut c = Compositor::new();
            view.render(&mut c);
        }
    }

    #[test]
    fn span_label_spells_frames_and_byte_ranges() {
        assert_eq!(
            span_label("media://cards-av1.mp4", 27, 28, true),
            "cards-av1.mp4 · frame 27"
        );
        assert_eq!(span_label("doc.txt", 0, 120, false), "doc.txt · 0–120");
        // a multi-frame media span is still a range.
        assert_eq!(span_label("media://a.mp4", 0, 4096, true), "a.mp4 · 0–4096");
    }

    #[test]
    fn parse_citation_splits_hash_and_chunk() {
        assert_eq!(
            parse_citation("urna://sha256:abc/sha256:def"),
            Some(("sha256:abc", "sha256:def"))
        );
        assert_eq!(parse_citation("sha256:def"), None);
        assert_eq!(parse_citation("urna://sha256:abc/"), None);
    }

    #[test]
    fn short_id_strips_prefix_and_truncates() {
        let id = format!("sha256:{}", "ab".repeat(32));
        assert_eq!(short_id(&id), "abababababab…");
    }
}
