//! Worker thread owning the [`NestBackend`] so the UI never blocks on
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

use super::backend::NestBackend;
pub use super::types::{NestCommand, NestEvent};
use super::types::{SearchMode, SearchResultsView};

/// Handle to the nest worker thread. Dropping it shuts the worker down.
pub struct NestWorker {
    tx: Sender<NestCommand>,
    rx: Receiver<NestEvent>,
    handle: Option<JoinHandle<()>>,
}

impl NestWorker {
    /// Spawn the worker with no database open.
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = channel::<NestCommand>();
        let (event_tx, event_rx) = channel::<NestEvent>();
        let handle = std::thread::Builder::new()
            .name("nest-backend".into())
            .spawn(move || {
                let mut backend: Option<NestBackend> = None;
                while let Ok(command) = cmd_rx.recv() {
                    if matches!(command, NestCommand::Shutdown) {
                        break;
                    }
                    run_command(&mut backend, command, &event_tx);
                }
            })
            .expect("spawn nest worker thread");
        Self {
            tx: cmd_tx,
            rx: event_rx,
            handle: Some(handle),
        }
    }

    /// Queue a command; never blocks. Returns `false` if the worker is
    /// gone (only possible after shutdown).
    pub fn send(&self, command: NestCommand) -> bool {
        self.tx.send(command).is_ok()
    }

    /// Non-blocking drain of finished results, one event per call.
    pub fn try_recv(&self) -> Option<NestEvent> {
        self.rx.try_recv().ok()
    }
}

impl Drop for NestWorker {
    fn drop(&mut self) {
        let _ = self.tx.send(NestCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_command(backend: &mut Option<NestBackend>, command: NestCommand, tx: &Sender<NestEvent>) {
    match command {
        NestCommand::Open(path) => {
            let event = match NestBackend::open(&path) {
                Ok(db) => {
                    let view = db.opened_view().map(Box::new).map_err(|e| e.to_string());
                    *backend = Some(db);
                    view
                }
                Err(e) => Err(e.to_string()),
            };
            let _ = tx.send(NestEvent::Opened(event));
        }
        NestCommand::OpenBytes { .. } => {
            // In-memory open is the web path; on desktop files come from
            // the filesystem (dropped/picked paths).
            let _ = tx.send(NestEvent::Opened(Err(
                "in-memory open is only supported on the web build".to_string(),
            )));
        }
        NestCommand::SearchByVector { query, mode, k } => {
            let result = match backend.as_ref() {
                Some(db) => db
                    .search(&mode, &query, k)
                    .map(SearchResultsView::from)
                    .map_err(|e| e.to_string()),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(NestEvent::SearchResults(result));
        }
        NestCommand::SearchByText { query, mode, k } => {
            let result = match backend.as_ref() {
                Some(db) => search_by_text(db, &query, &mode, k),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(NestEvent::SearchResults(result));
        }
        NestCommand::LoadChunks => {
            let result = match backend.as_mut() {
                Some(db) => db.load_chunks().map_err(|e| e.to_string()),
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(NestEvent::ChunksLoaded(result));
        }
        NestCommand::LoadGraph => {
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
            let _ = tx.send(NestEvent::GraphLoaded(result));
        }
        NestCommand::Benchmark { n_queries, k } => {
            let total = n_queries;
            let result = match backend.as_ref() {
                Some(db) => {
                    // The CLI's default ANN width, from the search screen's
                    // candidate budget convention.
                    let ef = ((k as usize) * 4).max(64);
                    let progress = |done: usize| {
                        let _ = tx.send(NestEvent::BenchmarkProgress { done, total });
                    };
                    db.benchmark(n_queries, k, ef, &progress)
                        .map_err(|e| e.to_string())
                }
                None => Err("no database open".to_string()),
            };
            let _ = tx.send(NestEvent::BenchmarkDone(result));
        }
        NestCommand::CheckEmbedder => {
            let _ = tx.send(NestEvent::EmbedderStatus(
                crate::model::embed::check_embedder(),
            ));
        }
        NestCommand::Shutdown => unreachable!("handled by the worker loop"),
    }
}

/// Embed-then-search: the offline potion bridge produces the query vector
/// (gated against the manifest identity), then the requested path runs.
/// Hybrid feeds the raw text to BM25; other modes ignore it.
fn search_by_text(
    db: &NestBackend,
    query: &str,
    mode: &SearchMode,
    k: i32,
) -> Result<SearchResultsView, String> {
    let (model, dim, model_hash) = db.embed_identity().map_err(|e| e.to_string())?;
    let vector = crate::model::embed::embed_query(&model, dim, &model_hash, query)
        .map_err(|e| e.to_string())?;
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

    fn recv(worker: &NestWorker) -> NestEvent {
        worker
            .rx
            .recv_timeout(Duration::from_secs(5))
            .expect("worker replies within 5s")
    }

    #[test]
    fn search_without_open_reports_no_database() {
        let worker = NestWorker::spawn();
        assert!(worker.send(NestCommand::SearchByVector {
            query: vec![1.0, 0.0],
            mode: SearchMode::Exact,
            k: 1,
        }));
        match recv(&worker) {
            NestEvent::SearchResults(Err(e)) => assert_eq!(e, "no database open"),
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn opening_a_missing_file_reports_the_error() {
        let worker = NestWorker::spawn();
        assert!(worker.send(NestCommand::Open(PathBuf::from(
            "/definitely/not/a/real/file.nest"
        ))));
        match recv(&worker) {
            NestEvent::Opened(Err(e)) => assert!(!e.is_empty()),
            other => panic!("unexpected event: {other:?}"),
        }
    }
}
