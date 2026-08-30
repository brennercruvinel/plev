//! Spellbook's data layer: the card index, the user's collection, decks,
//! games and the Scryfall/EDHREC updater. Knows nothing about the screen.
//!
//! This crate is to `spellbook` what plev's `git` crate is to `ide`: every
//! operation is a plain function over SQLite, and [`client::SpellbookClient`]
//! runs them on a worker thread so the UI thread never blocks on a query, a
//! file read or the network. Callers hand it a callback - normally one that
//! forwards into a `winit::EventLoopProxy` - and it stays UI-agnostic.
//!
//! It was previously reachable only through an embedded axum server that a
//! Tauri webview talked to over `fetch()`. The HTTP layer is gone; the SQL,
//! the rules and the Portuguese error strings below are the same ones, moved
//! out from under it.

pub mod client;
pub mod db;
pub mod decklist;
pub mod edhrec;
pub mod error;
pub mod images;
pub mod ops;
pub mod paths;
pub mod repair;
pub mod types;
pub mod update;
pub mod wizard;

pub use error::{Error, Result};
pub use types::{Activity, Card, ImageRef};

/// First-run setup: create the data directories and migrate app.db.
///
/// The Tauri build had to defer this until after the NSApplication event loop
/// was up, because app.db could sit in TCC-protected ~/Documents and the
/// permission prompt needed a running UI app. Data now lives under
/// ~/Library/Application Support, which the app owns outright, so this is
/// safe to call before the window exists.
pub fn init() {
    paths::ensure_dirs();
    if let Err(e) = db::init_app_db() {
        log::error!("failed to initialise app.db: {e}");
    }
    repair::canonicalise_card_names();
}
