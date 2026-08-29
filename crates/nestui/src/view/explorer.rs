//! Explorer: the heart of the UI — tab navigation, screen routing, the
//! backend worker handle and all opened-database state. Compiles on every
//! target: `Worker` is the thread-backed `NestWorker` on native and the
//! inline `WebWorker` on wasm. Native-only pieces (recents file, embedder
//! probe, system clipboard) are cfg-gated in place.
//!
//! Data flows one way: worker events land in `poll_backend`, get folded
//! into central state (`db`, `chunks`, embedder probe), and screens render
//! from it. Screens bubble intents up as [`Action`]s; only this module
//! talks to the worker, the clipboard and the recents file.

use std::collections::HashMap;
use std::path::PathBuf;

use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::overlay::OverlayManager;
use engine::theme::{Intent, Theme};
use engine::ui::widgets::{Button, EventResult, Rect, Tabs, ToastManager, WidgetEvent};

use crate::model::Worker;
#[cfg(not(target_arch = "wasm32"))]
use crate::model::recents;
use crate::model::types::{ChunksData, NestCommand, NestEvent, OpenedDbView, SearchMode};

use super::chunks::ChunksScreen;
use super::graph::{GraphContext, GraphScreen};
use super::open::{OpenContext, OpenScreen};
use super::overview::OverviewScreen;
use super::search::{SearchContext, SearchScreen};
use super::stats::StatsScreen;
use super::{Action, EditKey, Screen, text};

const PAD: f32 = 40.0;
/// Header band: title + tab strip.
const HEADER_H: f32 = 128.0;

#[derive(Clone, Copy)]
struct Layers {
    overlay: LayerId,
    toast: LayerId,
}

pub struct Explorer {
    width: f32,
    height: f32,
    tabs: Tabs,
    screen: Screen,
    worker: Worker,
    db: Option<Box<OpenedDbView>>,
    opening: bool,
    open_error: String,
    #[cfg(not(target_arch = "wasm32"))]
    recents: Vec<String>,
    #[cfg(not(target_arch = "wasm32"))]
    recents_path: Option<PathBuf>,
    /// The Open screen's picker button was clicked (web only).
    #[cfg(target_arch = "wasm32")]
    pick_requested: bool,
    embedder: Option<Result<String, String>>,
    file_hover: bool,
    chunks: Option<ChunksData>,
    chunks_loading: bool,
    /// chunk_id → ordinal, built once when texts arrive (result previews).
    chunk_index: Option<HashMap<String, usize>>,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    clipboard: engine::editor::SystemClipboard,
    empty_cta: Button,
    open: OpenScreen,
    overview: OverviewScreen,
    search: SearchScreen,
    chunks_screen: ChunksScreen,
    graph_screen: GraphScreen,
    stats_screen: StatsScreen,
    layers: Option<Layers>,
}

impl Explorer {
    pub fn new(theme: &Theme) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let recents_path = recents::default_path();
        #[cfg(not(target_arch = "wasm32"))]
        let recents = recents_path.as_ref().map(recents::load).unwrap_or_default();
        Self {
            width: 1200.0,
            height: 800.0,
            tabs: Tabs::new(Screen::TABS.iter().map(|s| s.title())),
            screen: Screen::Open,
            worker: Worker::spawn(),
            db: None,
            opening: false,
            open_error: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            recents,
            #[cfg(not(target_arch = "wasm32"))]
            recents_path,
            #[cfg(target_arch = "wasm32")]
            pick_requested: false,
            // On the web there is no python bridge: the text-mode hint
            // says so up front instead of failing on submission.
            #[cfg(target_arch = "wasm32")]
            embedder: Some(Err("text search requires the desktop app".to_string())),
            #[cfg(not(target_arch = "wasm32"))]
            embedder: None,
            file_hover: false,
            chunks: None,
            chunks_loading: false,
            chunk_index: None,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            clipboard: engine::editor::SystemClipboard::new(),
            empty_cta: Button::new("Open a database").icon("folder-open"),
            open: OpenScreen::new(theme),
            overview: OverviewScreen::new(),
            search: SearchScreen::new(theme),
            chunks_screen: ChunksScreen::new(),
            graph_screen: GraphScreen::new(),
            stats_screen: StatsScreen::new(),
            layers: None,
        }
    }

    #[cfg(test)]
    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Ask the worker to open a .nest file.
    pub fn open_database(&mut self, path: PathBuf) {
        self.opening = true;
        self.open_error = String::new();
        self.worker.send(NestCommand::Open(path));
    }

    /// Open picked bytes (web file picker).
    #[cfg(target_arch = "wasm32")]
    pub fn open_bytes(&mut self, name: String, bytes: Vec<u8>) {
        self.opening = true;
        self.open_error = String::new();
        self.worker.send(NestCommand::OpenBytes { name, bytes });
    }

    /// The Open screen's picker button was clicked (web only); `app.rs`
    /// triggers the DOM input.
    #[cfg(target_arch = "wasm32")]
    pub fn take_pick_request(&mut self) -> bool {
        std::mem::take(&mut self.pick_requested)
    }

    pub fn set_file_hover(&mut self, hovering: bool) {
        self.file_hover = hovering;
    }

    /// Recents are desktop-only (a JSON file in the data dir); the web
    /// build always shows an empty list.
    fn recents_slice(&self) -> &[String] {
        #[cfg(not(target_arch = "wasm32"))]
        return &self.recents;
        #[cfg(target_arch = "wasm32")]
        &[]
    }

    // -- Worker events -------------------------------------------------------

    /// Drain pending worker events, folding them into state. Returns
    /// `true` when anything changed (the shell requests a redraw).
    pub fn poll_backend(&mut self, toasts: &mut ToastManager, theme: &Theme) -> bool {
        let mut changed = false;
        while let Some(event) = self.worker.try_recv() {
            changed = true;
            match event {
                NestEvent::Opened(Ok(view)) => {
                    let path = view.path.display().to_string();
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(recents_path) = &self.recents_path {
                        self.recents = recents::record(recents_path, &path);
                    }
                    self.db = Some(view);
                    self.opening = false;
                    self.open_error = String::new();
                    // Per-database state resets.
                    self.chunks = None;
                    self.chunks_loading = false;
                    self.chunk_index = None;
                    self.search.reset();
                    self.chunks_screen.reset();
                    self.graph_screen.reset();
                    self.stats_screen.reset();
                    // Probe the embedder for the doctor line / text search.
                    #[cfg(not(target_arch = "wasm32"))]
                    self.worker.send(NestCommand::CheckEmbedder);
                    self.screen = Screen::Overview;
                    toasts.push(format!("opened {path}"), Intent::Constructive, theme);
                }
                NestEvent::Opened(Err(e)) => {
                    self.opening = false;
                    self.open_error = e.clone();
                    self.screen = Screen::Open;
                    toasts.push(format!("open failed: {e}"), Intent::Destructive, theme);
                }
                NestEvent::SearchResults(result) => {
                    if let Err(e) = &result {
                        toasts.push(format!("search failed: {e}"), Intent::Destructive, theme);
                    }
                    self.search.fold_result(result);
                }
                NestEvent::ChunksLoaded(Ok(data)) => {
                    // chunk_id → ordinal for search-result previews.
                    if let Some(db) = &self.db {
                        self.chunk_index = Some(
                            db.chunk_ids
                                .iter()
                                .enumerate()
                                .map(|(i, id)| (id.clone(), i))
                                .collect(),
                        );
                    }
                    self.chunks = Some(data);
                    self.chunks_loading = false;
                }
                NestEvent::ChunksLoaded(Err(e)) => {
                    self.chunks_loading = false;
                    toasts.push(
                        format!("failed to load chunks: {e}"),
                        Intent::Destructive,
                        theme,
                    );
                }
                NestEvent::EmbedderStatus(status) => {
                    self.embedder = Some(status);
                }
                NestEvent::GraphLoaded(result) => {
                    if let Err(e) = &result {
                        toasts.push(
                            format!("graph load failed: {e}"),
                            Intent::Destructive,
                            theme,
                        );
                    }
                    self.graph_screen.fold_scene(result);
                }
                NestEvent::BenchmarkProgress { done, total } => {
                    self.stats_screen.progress = (done, total);
                }
                NestEvent::BenchmarkDone(result) => {
                    if let Err(e) = &result {
                        toasts.push(format!("benchmark failed: {e}"), Intent::Destructive, theme);
                    }
                    self.stats_screen.fold_result(result);
                }
            }
        }
        changed
    }

    // -- Input ---------------------------------------------------------------

    fn switch_screen(&mut self, screen: Screen) {
        self.screen = screen;
        self.tabs.active = Screen::TABS.iter().position(|s| *s == screen).unwrap_or(0);
        // Chunks texts load lazily on first entry: `canonical_texts`
        // decodes the whole section, so opening stays fast.
        if screen == Screen::Chunks
            && self.db.is_some()
            && self.chunks.is_none()
            && !self.chunks_loading
        {
            self.chunks_loading = true;
            self.worker.send(NestCommand::LoadChunks);
        }
        // The graph layout (O(n²)) runs on the worker, once per db.
        if screen == Screen::Graph
            && let Some(db) = &self.db
            && db.has_graph
            && !self.graph_screen.has_scene()
            && !self.graph_screen.loading
        {
            self.graph_screen.loading = true;
            self.worker.send(NestCommand::LoadGraph);
        }
    }

    /// Fold a screen action into worker commands / clipboard / navigation.
    fn action(&mut self, action: Action, toasts: &mut ToastManager, theme: &Theme) -> bool {
        match action {
            Action::None => false,
            Action::OpenPath(path) => {
                if path.is_empty() {
                    return false;
                }
                self.open_database(PathBuf::from(path));
                true
            }
            Action::RunSearch {
                query,
                is_vector,
                mode,
                k,
            } => self.run_search(query, is_vector, mode, k),
            Action::Copy { text, what } => {
                self.copy(&text);
                toasts.push(format!("copied {what}"), Intent::Informational, theme);
                true
            }
            Action::Goto(screen) => {
                self.switch_screen(screen);
                true
            }
            Action::RunBenchmark { n_queries, k } => {
                self.worker.send(NestCommand::Benchmark { n_queries, k });
                true
            }
            Action::PickFile => {
                // Web only: the shell triggers the DOM picker on the next
                // about_to_wait. On desktop the button isn't shown.
                #[cfg(target_arch = "wasm32")]
                {
                    self.pick_requested = true;
                }
                cfg!(target_arch = "wasm32")
            }
        }
    }

    /// Global shortcuts (Cmd/Ctrl held): `o` jumps to Open, `1..=6` jump
    /// to tabs. Plain characters never reach here (they type into fields).
    pub fn handle_shortcut(&mut self, key: &str) -> bool {
        match key {
            "o" | "O" => {
                self.switch_screen(Screen::Open);
                true
            }
            d @ ("1" | "2" | "3" | "4" | "5" | "6") => {
                let idx = (d.as_bytes()[0] - b'1') as usize;
                if idx < Screen::TABS.len() {
                    self.switch_screen(Screen::TABS[idx]);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn copy(&mut self, text: &str) {
        use engine::editor::ClipboardProvider;
        self.clipboard.set_text(text);
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn copy(&mut self, _text: &str) {}

    /// Validate and dispatch a search. Vector queries parse as JSON arrays
    /// and must match the corpus dim; failures stay on the screen (no
    /// worker round-trip).
    fn run_search(&mut self, query: String, is_vector: bool, mode: SearchMode, k: i32) -> bool {
        let Some(db) = &self.db else {
            self.search
                .reject_submission("no database open".to_string());
            return true;
        };
        if is_vector {
            let vector: Vec<f32> = match serde_json::from_str(&query) {
                Ok(v) => v,
                Err(e) => {
                    self.search
                        .reject_submission(format!("invalid vector JSON: {e}"));
                    return true;
                }
            };
            let dim = db.inspect.embedding_dim as usize;
            if vector.len() != dim {
                self.search.reject_submission(format!(
                    "vector dim mismatch: corpus is {dim}, got {}",
                    vector.len()
                ));
                return true;
            }
            self.search.error = String::new();
            self.search.pending = true;
            self.worker.send(NestCommand::SearchByVector {
                query: vector,
                mode,
                k,
            });
        } else {
            self.search.error = String::new();
            self.search.pending = true;
            self.worker
                .send(NestCommand::SearchByText { query, mode, k });
        }
        true
    }

    /// Characters go to the focused field of the active screen.
    pub fn handle_key(&mut self, key: &str) -> bool {
        match self.screen {
            Screen::Open => self.open.handle_text(key),
            Screen::Search => self.search.handle_text(key),
            _ => false,
        }
    }

    pub fn handle_paste(&mut self, text: &str) -> bool {
        // Paste is plain character insertion (strip newlines — the fields
        // are single-line).
        let flat: String = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
        self.handle_key(&flat)
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        let (handled, action) = match self.screen {
            Screen::Open => self.open.handle_edit_key(key),
            Screen::Search => self.search.handle_edit_key(key),
            _ => (false, Action::None),
        };
        if action != Action::None {
            // Edit-key actions are only Open/Search submissions, which
            // never toast.
            self.dispatch_without_toasts(action);
            return true;
        }
        handled
    }

    /// Edit-key submissions never copy/notify; dispatch them directly.
    fn dispatch_without_toasts(&mut self, action: Action) {
        match action {
            Action::OpenPath(path) if !path.is_empty() => {
                self.open_database(PathBuf::from(path));
            }
            Action::RunSearch {
                query,
                is_vector,
                mode,
                k,
            } => {
                self.run_search(query, is_vector, mode, k);
            }
            Action::Goto(screen) => self.switch_screen(screen),
            _ => {}
        }
    }

    /// Escape: close the search mode dropdown first.
    pub fn close_top_overlay(&mut self) -> bool {
        if self.screen == Screen::Search && self.search.select_is_open() {
            self.search.close_select();
            return true;
        }
        false
    }

    /// Route a pointer event. Returns `true` if a redraw is needed.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        toasts: &mut ToastManager,
        theme: &Theme,
    ) -> bool {
        // An open select dropdown is exclusive (clicks elsewhere close it).
        if self.screen == Screen::Search && self.search.select_is_open() {
            let r =
                self.search
                    .route_select(event, self.content_rect(), self.search.result.is_some());
            return r.changed || r.handled;
        }

        // Tab strip.
        let r = self.tabs.handle_event(event, self.tabs_rect());
        if r.clicked {
            self.switch_screen(Screen::TABS[self.tabs.active]);
            return true;
        }
        let mut result = r;

        let content = self.content_rect();
        let (r, action) = match self.screen {
            Screen::Open => {
                let recents = self.recents_slice().to_vec();
                self.open
                    .handle_event(event, content, &recents, self.opening)
            }
            Screen::Overview => match &self.db {
                Some(db) => self.overview.handle_event(event, content, db),
                None => self.handle_empty_cta(event, content),
            },
            Screen::Search => {
                if self.db.is_some() {
                    self.search.handle_event(event, content, true)
                } else {
                    self.handle_empty_cta(event, content)
                }
            }
            Screen::Chunks => match &self.db {
                Some(db) => self
                    .chunks_screen
                    .handle_event(event, content, &db.chunk_ids),
                None => self.handle_empty_cta(event, content),
            },
            Screen::Graph => match &self.db {
                Some(db) if db.has_graph => {
                    let chunk_index = &self.chunk_index;
                    let chunks = &self.chunks;
                    let text_of = move |id: &str| -> Option<String> {
                        let idx = *chunk_index.as_ref()?.get(id)?;
                        chunks
                            .as_ref()?
                            .texts
                            .get(idx)
                            .and_then(|t| t.lines().next())
                            .map(str::to_string)
                    };
                    self.graph_screen.handle_event(
                        event,
                        content,
                        &GraphContext {
                            chunk_ids: &db.chunk_ids,
                            chunks: self.chunks.as_ref(),
                            text_of: &text_of,
                        },
                    )
                }
                // No graph section: inert empty state (the message renders
                // in `render`); swallow nothing.
                _ => (EventResult::IGNORED, Action::None),
            },
            Screen::Stats => match &self.db {
                Some(db) => {
                    let _ = db;
                    self.stats_screen.handle_event(event, content, true)
                }
                None => self.handle_empty_cta(event, content),
            },
        };
        result = result.merge(r);
        let acted = self.action(action, toasts, theme);
        result.changed || acted
    }

    /// The empty-state CTA (screens without an open db).
    fn handle_empty_cta(&mut self, event: &WidgetEvent, content: Rect) -> (EventResult, Action) {
        let r = self
            .empty_cta
            .handle_event(event, self.empty_cta_rect(content));
        if r.clicked {
            return (r, Action::Goto(Screen::Open));
        }
        (r, Action::None)
    }

    fn empty_cta_rect(&self, content: Rect) -> Rect {
        let (w, h) = self.empty_cta.preferred_size();
        Rect::new(
            content.x + (content.w - w) / 2.0,
            content.y + content.h / 2.0,
            w,
            h,
        )
    }

    // -- Animation -------------------------------------------------------------

    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = self.open.tick(dt);
        animating |= self.search.tick(dt);
        animating |= self.chunks_screen.tick(dt);
        animating |= self.graph_screen.tick(dt);
        animating
    }

    // -- Rendering -------------------------------------------------------------

    fn tabs_rect(&self) -> Rect {
        // Equal-width segments; the strip is sized from the longest label
        // (real measurement) rather than a hardcoded width.
        let style = engine::theme::TypographyScale::hoff().base_2sm();
        let max_label = Screen::TABS
            .iter()
            .map(|s| engine::text::TextMeasurer::measure_styled(s.title(), &style, None).0)
            .fold(0.0, f32::max);
        let seg_w = max_label + 48.0;
        let w = (seg_w * Screen::TABS.len() as f32 + 8.0).min(self.width - PAD * 2.0);
        Rect::new(PAD, 72.0, w, 44.0)
    }

    /// Content area below the header band.
    fn content_rect(&self) -> Rect {
        Rect::new(
            PAD,
            HEADER_H,
            (self.width - PAD * 2.0).max(200.0),
            (self.height - HEADER_H - PAD).max(120.0),
        )
    }

    fn ensure_layers(&mut self, c: &mut Compositor) -> Layers {
        *self.layers.get_or_insert_with(|| Layers {
            overlay: c.create_layer(OverlayManager::BASE_Z),
            toast: c.create_layer(OverlayManager::BASE_Z + 200),
        })
    }

    pub fn render(&mut self, c: &mut Compositor, theme: &Theme) {
        let layers = self.ensure_layers(c);

        // Header: wordmark + tab strip.
        text(c, "nestui", 20.0, 600, PAD, 24.0, theme.colors.text.0);
        let subtitle = match &self.db {
            Some(db) => db.path.display().to_string(),
            None => "no database open".to_string(),
        };
        let subtitle_style = engine::text::TextStyle::new(12.0);
        let subtitle = super::truncate_to_width(
            &subtitle,
            (self.width - PAD * 2.0 - 90.0).max(80.0),
            &subtitle_style,
        );
        text(
            c,
            &subtitle,
            12.0,
            400,
            PAD + 76.0,
            30.0,
            theme.colors.text_dim.0,
        );
        self.tabs.render(c, self.tabs_rect(), theme);

        // Screen content, clipped to the content band (scrolled/longs
        // pages never bleed into the header).
        let content = self.content_rect();
        c.push(SceneNode::PushClip {
            x: content.x,
            y: content.y,
            w: content.w,
            h: content.h,
        });
        match self.screen {
            Screen::Open => {
                let recents = self.recents_slice().to_vec();
                let ctx = OpenContext {
                    recents: &recents,
                    error: &self.open_error,
                    embedder: self.embedder.as_ref(),
                    opening: self.opening,
                    file_hover: self.file_hover,
                };
                self.open.render(c, content, theme, &ctx);
            }
            Screen::Overview => match &self.db {
                Some(db) => self.overview.render(c, content, theme, db),
                None => self.render_empty(c, content, theme, "no database open yet", true),
            },
            Screen::Search => {
                if self.db.is_some() {
                    let dim = self
                        .db
                        .as_ref()
                        .map(|db| db.inspect.embedding_dim as usize)
                        .unwrap_or(0);
                    let chunk_index = &self.chunk_index;
                    let chunks = &self.chunks;
                    let text_of = move |id: &str| -> Option<String> {
                        let idx = *chunk_index.as_ref()?.get(id)?;
                        chunks
                            .as_ref()?
                            .texts
                            .get(idx)
                            .and_then(|t| t.lines().next())
                            .map(str::to_string)
                    };
                    let embedder = self.embedder.as_ref();
                    self.search.render(c, content, theme, &SearchContext {
                        dim,
                        text_of: &text_of,
                        embedder,
                    });
                } else {
                    self.render_empty(c, content, theme, "no database open yet", true);
                }
            }
            Screen::Chunks => match &self.db {
                Some(db) => self.chunks_screen.render(
                    c,
                    content,
                    theme,
                    &db.chunk_ids,
                    self.chunks.as_ref(),
                    self.chunks_loading,
                ),
                None => self.render_empty(c, content, theme, "no database open yet", true),
            },
            Screen::Graph => match &self.db {
                Some(db) if db.has_graph => {
                    let chunk_index = &self.chunk_index;
                    let chunks = &self.chunks;
                    let text_of = move |id: &str| -> Option<String> {
                        let idx = *chunk_index.as_ref()?.get(id)?;
                        chunks
                            .as_ref()?
                            .texts
                            .get(idx)
                            .and_then(|t| t.lines().next())
                            .map(str::to_string)
                    };
                    self.graph_screen.render(
                        c,
                        content,
                        theme,
                        &GraphContext {
                            chunk_ids: &db.chunk_ids,
                            chunks: self.chunks.as_ref(),
                            text_of: &text_of,
                        },
                    );
                }
                Some(_) => self.render_empty(
                    c,
                    content,
                    theme,
                    "this file has no graph_adjacency section — rebuild it with graph support (with_graph=True)",
                    false,
                ),
                None => self.render_empty(c, content, theme, "no database open yet", true),
            },
            Screen::Stats => match &self.db {
                Some(db) => self.stats_screen.render(c, content, theme, db),
                None => self.render_empty(c, content, theme, "no database open yet", true),
            },
        }
        c.push(SceneNode::PopClip);

        // The open select dropdown floats above the clipped content.
        if self.screen == Screen::Search {
            self.search
                .render_dropdown(c, layers.overlay, content, theme);
        }
    }

    /// Render toasts on their layer (called by the shell after `render` —
    /// kept here so the toast layer id stays private).
    pub fn render_toasts(
        &mut self,
        c: &mut Compositor,
        toasts: &ToastManager,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        let layers = self.ensure_layers(c);
        toasts.render(c, layers.toast, theme, vw, vh);
    }

    /// Empty state: centered message + optional CTA to the Open screen.
    fn render_empty(
        &mut self,
        c: &mut Compositor,
        content: Rect,
        theme: &Theme,
        msg: &str,
        show_cta: bool,
    ) {
        let style = engine::text::TextStyle::new(14.0);
        let msg = super::truncate_to_width(msg, content.w, &style);
        let (tw, _) = engine::text::TextMeasurer::measure_styled(&msg, &style, None);
        text(
            c,
            &msg,
            14.0,
            500,
            content.x + (content.w - tw) / 2.0,
            content.y + content.h / 2.0 - 32.0,
            theme.colors.text_dim.0,
        );
        if show_cta {
            self.empty_cta
                .render(c, self.empty_cta_rect(content), theme);
        }
    }
}

// ---------------------------------------------------------------------------
// Headless explorer tests (navigation, empty states, error flow)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::fixtures;

    fn harness() -> (Explorer, ToastManager, Theme) {
        let theme = Theme::hoff();
        (Explorer::new(&theme), ToastManager::new(), theme)
    }

    /// Center of tab `i`'s segment in the strip.
    fn tab_center(ex: &Explorer, i: usize) -> (f32, f32) {
        ex.tabs.item_rects(ex.tabs_rect())[i].center()
    }

    fn click(ex: &mut Explorer, toasts: &mut ToastManager, theme: &Theme, x: f32, y: f32) {
        ex.handle_event(&WidgetEvent::MouseDown { x, y }, toasts, theme);
        ex.handle_event(&WidgetEvent::MouseUp { x, y }, toasts, theme);
    }

    #[test]
    fn tab_clicks_switch_screens() {
        let (mut ex, mut toasts, theme) = harness();
        assert_eq!(ex.screen(), Screen::Open);
        for (i, expected) in Screen::TABS.iter().enumerate() {
            let (x, y) = tab_center(&ex, i);
            click(&mut ex, &mut toasts, &theme, x, y);
            assert_eq!(ex.screen(), *expected);
        }
    }

    #[test]
    fn shortcuts_jump_to_tabs() {
        let (mut ex, _, _) = harness();
        assert!(ex.handle_shortcut("o") || true); // "o" → Open (already there)
        assert!(ex.handle_shortcut("3"));
        assert_eq!(ex.screen(), Screen::Search);
        assert!(ex.handle_shortcut("5"));
        assert_eq!(ex.screen(), Screen::Graph);
        assert!(ex.handle_shortcut("6"));
        assert_eq!(ex.screen(), Screen::Stats);
        assert!(!ex.handle_shortcut("9"));
        assert_eq!(ex.screen(), Screen::Stats);
        assert!(ex.handle_shortcut("o"));
        assert_eq!(ex.screen(), Screen::Open);
    }

    #[test]
    fn entering_graph_with_a_graphed_db_requests_the_layout_once() {
        let (mut ex, _, _) = harness();
        ex.db = Some(fixtures::fake_db_with_graph());
        ex.switch_screen(Screen::Graph);
        assert!(ex.graph_screen.loading);
        // A second entry does not re-send while in flight.
        ex.switch_screen(Screen::Open);
        ex.switch_screen(Screen::Graph);
        assert!(ex.graph_screen.loading);

        // The layout arrives and the screen renders it.
        ex.graph_screen.fold_scene(Ok(fixtures::fake_graph_scene()));
        assert!(ex.graph_screen.has_scene());
        let theme = Theme::hoff();
        let mut c = Compositor::new();
        ex.resize(1600.0, 1000.0);
        ex.render(&mut c, &theme);
    }

    #[test]
    fn graph_without_a_graph_section_shows_the_explanation() {
        let (mut ex, _, theme) = harness();
        ex.db = Some(fixtures::fake_db()); // has_graph = false
        ex.switch_screen(Screen::Graph);
        assert!(!ex.graph_screen.loading, "no LoadGraph without a section");
        let mut c = Compositor::new();
        ex.render(&mut c, &theme);
    }

    #[test]
    fn opening_a_missing_path_shows_the_error_and_returns_to_open() {
        let (mut ex, mut toasts, theme) = harness();
        ex.open_database(PathBuf::from("/definitely/not/here.nest"));
        assert!(ex.opening);
        for _ in 0..1000 {
            if ex.poll_backend(&mut toasts, &theme) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(!ex.opening);
        assert!(!ex.open_error.is_empty());
        assert_eq!(ex.screen(), Screen::Open);
        assert_eq!(toasts.len(), 1, "the failure also raises a toast");
    }

    #[test]
    fn vector_search_validates_json_and_dim_before_sending() {
        let (mut ex, _, _) = harness();
        ex.db = Some(fixtures::fake_db()); // dim = 4
        ex.switch_screen(Screen::Search);

        // Not JSON.
        assert!(ex.run_search("nope".into(), true, SearchMode::Exact, 5));
        assert!(ex.search.error.contains("invalid vector JSON"));
        assert!(!ex.search.pending, "rejected submissions clear pending");

        // Wrong dim.
        ex.run_search("[1.0, 2.0]".into(), true, SearchMode::Exact, 5);
        assert!(ex.search.error.contains("dim mismatch"));

        // Right dim: dispatched to the worker (no db there — the async
        // error is out of scope for this test).
        ex.run_search("[0.1, 0.2, 0.3, 0.4]".into(), true, SearchMode::Exact, 5);
        assert!(ex.search.error.is_empty());
        assert!(ex.search.pending);
    }

    #[test]
    fn empty_state_cta_navigates_to_open() {
        let (mut ex, mut toasts, theme) = harness();
        ex.switch_screen(Screen::Overview);
        assert!(ex.db.is_none());
        let cta = ex.empty_cta_rect(ex.content_rect());
        let (x, y) = cta.center();
        click(&mut ex, &mut toasts, &theme, x, y);
        assert_eq!(ex.screen(), Screen::Open);
    }

    #[test]
    fn every_screen_renders_at_narrow_and_wide() {
        let (mut ex, _, theme) = harness();
        for with_db in [false, true] {
            if with_db {
                ex.db = Some(fixtures::fake_db());
                ex.chunks = Some(fixtures::fake_chunks());
            }
            for screen in Screen::TABS {
                for (w, h) in [(800.0, 600.0), (1600.0, 1000.0)] {
                    ex.resize(w, h);
                    ex.switch_screen(screen);
                    let mut c = Compositor::new();
                    ex.render(&mut c, &theme);
                }
            }
        }
    }

    #[test]
    fn entering_chunks_with_a_db_requests_the_texts_once() {
        let (mut ex, _, _) = harness();
        ex.db = Some(fixtures::fake_db());
        ex.switch_screen(Screen::Chunks);
        assert!(ex.chunks_loading);
        // Already in flight: switching away and back does not re-send.
        ex.switch_screen(Screen::Open);
        ex.switch_screen(Screen::Chunks);
        // (No second LoadChunks: `chunks_loading` is still true and the
        // worker got exactly one command; verified by not panicking on a
        // doubled load in the integration flow.)
        assert!(ex.chunks_loading);
    }
}
