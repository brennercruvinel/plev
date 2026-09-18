//! Worker thread owning the [`UrnaBackend`] so the UI never blocks on
//! search or section decode.
//!
//! Commands flow in through an mpsc channel, events flow out through a
//! second channel the UI drains with `try_recv` (from the event loop's
//! `about_to_wait`, which then requests a redraw). Same shape as
//! `crates/git`'s `GitClient`, except results come back over a channel
//! instead of a callback — winit apps already poll every frame.
//!
//! The command/event vocabulary lives in `model::types` (shared with the
//! web inline worker).

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;

use super::backend::UrnaBackend;
pub use super::types::{UrnaCommand, UrnaEvent};
use super::types::{SearchMode, SearchResultsView};

/// Handle to the urna worker thread. Dropping it shuts the worker down.
pub struct UrnaWorker {
    tx: Sender<UrnaCommand>,
    rx: Receiver<UrnaEvent>,
    handle: Option<JoinHandle<()>>,
}

impl UrnaWorker {
    /// Spawn the worker with no database open.
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = channel::<UrnaCommand>();
        let (event_tx, event_rx) = channel::<UrnaEvent>();
        let handle = std::thread::Builder::new()
            .name("urna-backend".into())
            .spawn(move || {
                let mut backend: Option<UrnaBackend> = None;
                while let Ok(command) = cmd_rx.recv() {
                    if matches!(command, UrnaCommand::Shutdown) {
                        break;
                    }
                    run_command(&mut backend, command, &event_tx);
                }
            })
            .expect("spawn urna worker thread");
        Self {
            tx: cmd_tx,
            rx: event_rx,
            handle: Some(handle),
        }
    }

    /// Queue a command; never blocks. Returns `false` if the worker is
    /// gone (only possible after shutdown).
    pub fn send(&self, command: UrnaCommand) -> bool {
        self.tx.send(command).is_ok()
    }

    /// Non-blocking drain of finished results, one event per call.
    pub fn try_recv(&self) -> Option<UrnaEvent> {
        self.rx.try_recv().ok()
    }
}

impl Drop for UrnaWorker {
    fn drop(&mut self) {
        let _ = self.tx.send(UrnaCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_command(backend: &mut Option<UrnaBackend>, command: UrnaCommand, tx: &Sender<UrnaEvent>) {
    match command {
        UrnaCommand::Open(path) => {
            let event = match UrnaBackend::open(&path) {
                Ok(db) => {
                    let view = db.opened_view().map(Box::new).map_err(|e| e.to_string());
                    *backend = Some(db);
                    view
                }
                Err(e) => Err(e.to_string()),
            };
            let opened = event.is_ok();
            let _ = tx.send(UrnaEvent::Opened(event));
            // Texts + spans right behind the snapshot: one decode pass, and
            // every screen (result previews, chunk list, frame lookup) has
            // them before the user can ask. Never a second full-file read
            // (the backend maps the file once).
            if opened && let Some(db) = backend.as_mut() {
                let _ = tx.send(UrnaEvent::ChunksLoaded(
                    db.load_chunks().map_err(|e| e.to_string()),
                ));
            }
        }
        UrnaCommand::OpenBytes { .. } => {
            // In-memory open is the web path; on desktop files come from
            // the filesystem (dropped/picked paths).
            let _ = tx.send(UrnaEvent::Opened(Err(
                "in-memory open is only supported on the web build".to_string(),
            )));
        }
        UrnaCommand::SearchByVector { query, mode, k } => {
            let result = match backend.as_ref() {
                Some(db) => db
                    .search(&mode, &query, k)
                    .map(SearchResultsView::from)
                    .map_err(|e| e.to_string()),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(UrnaEvent::SearchResults(result));
        }
        UrnaCommand::SearchByText { query, mode, k } => {
            let result = match backend.as_ref() {
                Some(db) => search_by_text(db, &query, &mode, k),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(UrnaEvent::SearchResults(result));
        }
        UrnaCommand::LoadChunks => {
            let result = match backend.as_mut() {
                Some(db) => db.load_chunks().map_err(|e| e.to_string()),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(UrnaEvent::ChunksLoaded(result));
        }
        UrnaCommand::LoadGraph => {
            let result = match backend.as_ref() {
                Some(db) => match db.graph_data() {
                    Some(data) => {
                        // Layout in a fixed 1000×1000 world box; the view
                        // fits it to the viewport with a ViewTransform.
                        Ok(engine::graph::compute_layout(&data, 1000.0, 1000.0))
                    }
                    None => Err("no graph section in this file".to_string()),
                },
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(UrnaEvent::GraphLoaded(result));
        }
        UrnaCommand::Benchmark { n_queries, k } => {
            let total = n_queries;
            let result = match backend.as_ref() {
                Some(db) => {
                    // The CLI's default ANN width, from the search screen's
                    // candidate budget convention.
                    let ef = ((k as usize) * 4).max(64);
                    let progress = |done: usize| {
                        let _ = tx.send(UrnaEvent::BenchmarkProgress { done, total });
                    };
                    db.benchmark(n_queries, k, ef, &progress)
                        .map_err(|e| e.to_string())
                }
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(UrnaEvent::BenchmarkDone(result));
        }
        UrnaCommand::CheckEmbedder => {
            let _ = tx.send(UrnaEvent::EmbedderStatus(
                crate::model::embed::check_embedder(),
            ));
        }
        UrnaCommand::Validate => {
            let result = match backend.as_ref() {
                Some(db) => db.validate().map_err(|e| e.to_string()),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(UrnaEvent::Validated(result));
        }
        UrnaCommand::ExportBlob { index, dest } => {
            let result = match backend.as_ref() {
                Some(db) => db
                    .export_blob(index, &dest)
                    .map(|()| dest)
                    .map_err(|e| e.to_string()),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(UrnaEvent::BlobExported(result));
        }
        UrnaCommand::LoadFrame { ordinal, max_side } => {
            let result = match backend.as_ref() {
                Some(db) => load_frame(db, ordinal, max_side),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(UrnaEvent::FrameLoaded { ordinal, result });
        }
        UrnaCommand::Shutdown => unreachable!("handled by the worker loop"),
    }
}

/// Resolve chunk `ordinal` through the blob overlay to (blob range, frame)
/// and decode that frame to PNG bytes.
fn load_frame(db: &UrnaBackend, ordinal: usize, max_side: u32) -> Result<Vec<u8>, String> {
    let span = db
        .blob_span(ordinal)
        .ok_or_else(|| "this chunk does not map into a media blob".to_string())?;
    let frame = span
        .frame()
        .ok_or_else(|| format!("span {}–{} is not a single frame", span.start, span.end))?;
    let range = db.blob_range(span.blob_index as usize).ok_or_else(|| {
        "the media bytes are not inlined in this file (sidecar blob)".to_string()
    })?;
    let path = db.path().to_string_lossy().into_owned();
    crate::model::frames::decode_frame(&path, range, frame, max_side).map_err(|e| e.to_string())
}

/// Embed-then-search: the offline embedder produces the query vector
/// (gated against the manifest identity, or the space's for
/// `SearchMode::Space`), then the requested path runs. Hybrid feeds the
/// raw text to BM25; other modes ignore it.
fn search_by_text(
    db: &UrnaBackend,
    query: &str,
    mode: &SearchMode,
    k: i32,
) -> Result<SearchResultsView, String> {
    let vector = match mode {
        SearchMode::Space { name } => {
            let (dim, model_hash) = db.space_identity(name).map_err(|e| e.to_string())?;
            crate::model::embed::embed_query_space(name, dim, &model_hash, query)
                .map_err(|e| e.to_string())?
        }
        _ => {
            let (model, dim, model_hash) = db.embed_identity().map_err(|e| e.to_string())?;
            crate::model::embed::embed_query(&model, dim, &model_hash, query)
                .map_err(|e| e.to_string())?
        }
    };
    // Hybrid needs the text for its lexical leg regardless of what the
    // caller's mode struct carries.
    let mode = match mode {
        SearchMode::Hybrid {
            candidates_per_path,
            ..
        } => SearchMode::Hybrid {
            query_text: query.to_string(),
            candidates_per_path: *candidates_per_path,
        },
        other => other.clone(),
    };
    db.search(&mode, &vector, k)
        .map(SearchResultsView::from)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    fn recv(worker: &UrnaWorker) -> UrnaEvent {
        worker
            .rx
            .recv_timeout(Duration::from_secs(5))
            .expect("worker replies within 5s")
    }

    #[test]
    fn search_without_open_reports_no_database() {
        let worker = UrnaWorker::spawn();
        assert!(worker.send(UrnaCommand::SearchByVector {
            query: vec![1.0, 0.0],
            mode: SearchMode::Exact,
            k: 1,
        }));
        match recv(&worker) {
            UrnaEvent::SearchResults(Err(e)) => assert_eq!(e, "no database open"),
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn opening_a_missing_file_reports_the_error() {
        let worker = UrnaWorker::spawn();
        assert!(worker.send(UrnaCommand::Open(PathBuf::from(
            "/definitely/not/a/real/file.urna"
        ))));
        match recv(&worker) {
            UrnaEvent::Opened(Err(e)) => assert!(!e.is_empty()),
            other => panic!("unexpected event: {other:?}"),
        }
    }
}
