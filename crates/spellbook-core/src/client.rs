//! Threaded wrapper around the operations so the UI never blocks.
//!
//! Same shape as plev's `GitClient`: commands flow through an mpsc channel
//! into a worker thread, results come back through a caller-provided
//! callback. The callback normally forwards into a `winit::EventLoopProxy`,
//! but this crate stays UI-agnostic on purpose - nothing here knows what a
//! window is.
//!
//! Why a thread at all: under render-on-demand a frame is only drawn when
//! something changed, so a query blocking the UI thread does not show a
//! spinner, it freezes the window. Every operation here can block - SQLite
//! on a cold page cache, the image cache on disk, Scryfall over the network.
//!
//! [`Command`] and [`Event`] grow one pair of variants per operation as the
//! domains land; the machinery below does not change.

use std::sync::mpsc::{Sender, channel};
use std::thread::JoinHandle;

use crate::error::Error;

/// Work the UI asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Stop the worker. Sent by `Drop`; the thread is joined after it.
    Shutdown,
}

/// Results delivered to the event callback, one per command.
#[derive(Debug)]
pub enum Event {
    /// The worker thread is up and the data directories exist.
    Ready,
    /// A command failed in a way no specific variant covers.
    Failed(Error),
}

/// Handle to the worker thread. Dropping it shuts the worker down and waits
/// for the in-flight command to finish, so a half-written row never outlives
/// the process.
pub struct SpellbookClient {
    tx: Sender<Command>,
    handle: Option<JoinHandle<()>>,
}

impl SpellbookClient {
    /// Spawns the worker. `on_event` is called from the worker thread for
    /// every finished command.
    pub fn spawn(on_event: impl Fn(Event) + Send + 'static) -> Self {
        let (tx, rx) = channel::<Command>();
        let handle = std::thread::Builder::new()
            .name("spellbook-data".into())
            .spawn(move || {
                on_event(Event::Ready);
                while let Ok(command) = rx.recv() {
                    if command == Command::Shutdown {
                        break;
                    }
                    run(command, &on_event);
                }
            })
            .expect("spawn spellbook worker thread");
        Self {
            tx,
            handle: Some(handle),
        }
    }

    /// Queues a command. Silently ignored once the worker has stopped -
    /// which only happens during shutdown, where there is no one left to
    /// tell.
    pub fn send(&self, command: Command) {
        let _ = self.tx.send(command);
    }
}

impl Drop for SpellbookClient {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Runs one command on the worker thread and reports the result.
fn run(command: Command, on_event: &(impl Fn(Event) + ?Sized)) {
    match command {
        // Handled by the loop before it gets here.
        Command::Shutdown => {}
    }
    let _ = on_event;
}
