//! Web worker: the same command/event interface as `worker::NestWorker`,
//! but executing synchronously on the UI thread (wasm32-unknown-unknown
//! has no threads without atomics + COOP/COEP headers).
//!
//! Blocking analysis: exact search and chunk decode are O(n·dim) over an
//! already-resident `Vec<u8>` — fast enough for thousands of chunks. The
//! O(n²) graph layout is the one heavy command; it blocks the frame loop
//! for its duration (phase C accepts this; a future revision can chunk
//! layout across frames). `RefCell` stands in for the native worker's
//! channel so the public surface stays `&self` like `NestWorker`.

use std::cell::RefCell;
use std::collections::VecDeque;

use super::nestread::NestBytes;
use super::types::{NestCommand, NestEvent};

struct Inner {
    file: Option<NestBytes>,
    queue: VecDeque<NestEvent>,
}

/// Inline "worker": `send` executes immediately and queues the event;
/// `try_recv` pops it. Interface-identical to the native `NestWorker`.
pub struct WebWorker {
    inner: RefCell<Inner>,
}

impl WebWorker {
    pub fn spawn() -> Self {
        Self {
            inner: RefCell::new(Inner {
                file: None,
                queue: VecDeque::new(),
            }),
        }
    }

    pub fn send(&self, command: NestCommand) -> bool {
        self.run(command);
        true
    }

    pub fn try_recv(&self) -> Option<NestEvent> {
        self.inner.borrow_mut().queue.pop_front()
    }

    fn push(&self, event: NestEvent) {
        self.inner.borrow_mut().queue.push_back(event);
    }

    fn run(&self, command: NestCommand) {
        match command {
            NestCommand::Open(_) => self.push(NestEvent::Opened(Err(
                "path open is only supported on the desktop build".to_string(),
            ))),
            NestCommand::OpenBytes { name, bytes } => {
                let event = match NestBytes::open(name, bytes) {
                    Ok(file) => {
                        let view = Box::new(file.opened_view());
                        self.inner.borrow_mut().file = Some(file);
                        Ok(view)
                    }
                    Err(e) => Err(e),
                };
                self.push(NestEvent::Opened(event));
            }
            NestCommand::SearchByVector { query, mode, k } => {
                // Only the exact path exists on web; the other modes fall
                // back to it, mirroring the runtime's own no-section
                // fallback (and the explain panel says `route: exact`).
                let _ = mode;
                let result = {
                    let inner = self.inner.borrow();
                    match &inner.file {
                        Some(file) => file.search_exact(&query, k),
                        None => Err("no database open".to_string()),
                    }
                };
                self.push(NestEvent::SearchResults(result));
            }
            NestCommand::SearchByText { .. } => self.push(NestEvent::SearchResults(Err(
                "text search requires the desktop app (the offline embedder is a \
                 python subprocess)"
                    .to_string(),
            ))),
            NestCommand::LoadChunks => {
                let result = {
                    let inner = self.inner.borrow();
                    match &inner.file {
                        Some(file) => file.chunks_data(),
                        None => Err("no database open".to_string()),
                    }
                };
                self.push(NestEvent::ChunksLoaded(result));
            }
            NestCommand::LoadGraph => {
                let result = {
                    let inner = self.inner.borrow();
                    match &inner.file {
                        Some(file) => match file.graph_data() {
                            // The O(n²) layout runs inline here — see the
                            // module docs for the blocking note.
                            Some(data) => Ok(super::graph::compute_layout(&data, 1000.0, 1000.0)),
                            None => Err("no graph section in this file".to_string()),
                        },
                        None => Err("no database open".to_string()),
                    }
                };
                self.push(NestEvent::GraphLoaded(result));
            }
            NestCommand::Benchmark { n_queries, k } => {
                // Progress heartbeats are meaningless under inline
                // execution (nothing paints between queries); the result
                // event alone drives the UI.
                let result = {
                    let inner = self.inner.borrow();
                    match &inner.file {
                        Some(file) => file.benchmark(n_queries, k, &|_| {}),
                        None => Err("no database open".to_string()),
                    }
                };
                self.push(NestEvent::BenchmarkDone(result));
            }
            NestCommand::CheckEmbedder => self.push(NestEvent::EmbedderStatus(Err(
                "text search requires the desktop app".to_string(),
            ))),
            NestCommand::Shutdown => {}
        }
    }
}

impl Default for WebWorker {
    fn default() -> Self {
        Self::spawn()
    }
}
