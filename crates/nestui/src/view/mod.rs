//! nestui view: winit-free, gpu-free application state and scene building.
//!
//! `NestuiView` is the thin shell every platform compiles: theme, size,
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
use engine::text::{TextMeasurer, TextStyle};
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
    /// Open the .nest at this path (typed, pasted, recent, dropped).
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
    /// Open the browser's file picker (web only; desktop opens by path).
    PickFile,
}

pub struct NestuiView {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
    pub theme: Theme,
    pub toasts: ToastManager,
    explorer: explorer::Explorer,
}

impl NestuiView {
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

    pub fn resize(&mut self, width: f32, height: f32, scale_factor: f32) {
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
        self.explorer.resize(width, height);
    }

    /// Ask the worker to open a .nest file (desktop launch argument).
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

    /// Global shortcuts with Cmd/Ctrl held: `o` → Open, `1..=6` → tabs.
    pub fn handle_shortcut(&mut self, key: &str) -> bool {
        self.explorer.handle_shortcut(key)
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

/// Truncate `s` with an ellipsis so it fits on one line of `avail` px,
/// measured with the SAME [`TextStyle`] the caller draws with (real
/// shaping, never a per-char estimate). Port of the ide's
/// `components::hoff::truncate_to_width`.
pub(crate) fn truncate_to_width(s: &str, avail: f32, style: &TextStyle) -> String {
    if TextMeasurer::measure_styled(s, style, None).0 <= avail {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let candidate = |n: usize| -> String {
        let t: String = chars[..n].iter().collect();
        format!("{}\u{2026}", t.trim_end())
    };
    // Largest prefix whose "prefix…" really fits, by binary search on the
    // char count (each probe is a cached real measurement).
    let (mut lo, mut hi) = (0usize, chars.len().saturating_sub(1));
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if TextMeasurer::measure_styled(&candidate(mid), style, None).0 <= avail {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    candidate(lo)
}

/// A chunk id rendered short: strip the `sha256:` prefix, keep 12 hex
/// chars (fixed char count — hashes are fixed-width, no shaping needed).
pub(crate) fn short_id(chunk_id: &str) -> String {
    let hex = chunk_id.strip_prefix("sha256:").unwrap_or(chunk_id);
    let short: String = hex.chars().take(12).collect();
    format!("{short}…")
}

/// Human byte size (KiB/MiB/GiB), no dependencies.
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
            magic: "NEST".into(),
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
            blobs: serde_json::Value::Null,
            file_hash: format!("sha256:{}", "1".repeat(64)),
            content_hash: format!("sha256:{}", "2".repeat(64)),
            simd_backend: "neon".into(),
        };
        Box::new(OpenedDbView {
            path: PathBuf::from("/tmp/demo.nest"),
            inspect,
            chunk_ids: (0..3)
                .map(|i| format!("sha256:{}{}", i, "0".repeat(63)))
                .collect(),
            has_ann: false,
            has_bm25: false,
            has_graph: false,
            has_spaces: false,
            space_names: vec![],
            graph_nodes: None,
        })
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
    pub fn fake_graph_scene() -> crate::model::graph::GraphScene {
        let data = crate::model::graph::GraphData {
            n_nodes: 3,
            offsets: vec![0, 1, 2, 2],
            neighbors: vec![1, 2],
            edge_types: vec![
                crate::model::graph::EDGE_NEXT_CHUNK,
                crate::model::graph::EDGE_NEXT_CHUNK,
            ],
        };
        crate::model::graph::compute_layout(&data, 1000.0, 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_the_open_screen_with_the_hoff_theme() {
        let view = NestuiView::new(1200.0, 800.0);
        // Page frame is the HOFF #444444.
        assert_eq!(view.theme.colors.bg.0, engine::theme::hoff::PAGE_BG.0);
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(view.explorer.screen(), Screen::Open);
    }

    #[test]
    fn render_builds_a_scene_without_gpu_at_two_widths() {
        for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
            let mut view = NestuiView::new(w, h);
            let mut c = Compositor::new();
            view.render(&mut c);
        }
    }

    #[test]
    fn truncate_to_width_ellipsizes_and_keeps_short_strings() {
        let style = TextStyle::new(14.0);
        let short = "abc";
        assert_eq!(truncate_to_width(short, 500.0, &style), short);
        let long = "x".repeat(400);
        let out = truncate_to_width(&long, 100.0, &style);
        assert!(out.ends_with('\u{2026}'));
        assert!(TextMeasurer::measure_styled(&out, &style, None).0 <= 100.0);
        assert!(out.len() < long.len());
    }

    #[test]
    fn short_id_strips_prefix_and_truncates() {
        let id = format!("sha256:{}", "ab".repeat(32));
        assert_eq!(short_id(&id), "abababababab…");
    }
}
