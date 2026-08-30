//! The card database itself: how fresh it is, and kicking off a refresh.
//!
//! This is the app's only scheduled network use. Everything else reads what the update left
//! behind, so the app keeps working with the plug pulled.

use serde::{Deserialize, Serialize};

use crate::db::open_cards_db;
use crate::error::{Error, Result};
use crate::images;
use crate::paths;

/// Freshness of the card index, as the sidebar reports it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataInfo {
    /// `false` before the first download: the sidebar offers "baixar" instead
    /// of "atualizar", and every other screen is expected to be empty.
    pub exists: bool,
    pub cards: i64,
    pub pt_names: i64,
    /// Unix seconds, from the index file's mtime. `None` when it can't be read.
    pub built_at: Option<f64>,
}

/// Progress of the background data update, polled by the sidebar panel.
///
/// Mirrors `update::STATUS`, which the worker thread writes as it goes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateStatus {
    /// `idle` | `running` | `done` | `error`. The poller keeps asking while
    /// this is `running` and reports the error text when it is `error`.
    pub state: String,
    /// What the worker is doing right now, shown beside the bar.
    pub task: Option<String>,
    pub percent: f64,
    pub error: Option<String>,
    /// Whatever the finished run summarised about itself. Shaped by
    /// `update::run`, which is free to change it, so it stays untyped here.
    pub result: Option<serde_json::Value>,
}

/// How much art is on disk, so the sidebar can say whether the app is genuinely offline-ready.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImagesInfo {
    pub bytes: u64,
    /// The same figure in GB, rounded to two decimals for display.
    pub gb: f64,
    pub cards_with_art: i64,
}

pub fn info() -> DataInfo {
    let Some(cdb) = open_cards_db() else {
        return DataInfo {
            exists: false,
            cards: 0,
            pt_names: 0,
            built_at: None,
        };
    };
    let cards: i64 = cdb
        .query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))
        .unwrap_or(0);
    let pt: i64 = cdb
        .query_row(
            "SELECT COUNT(DISTINCT printed_name) FROM names_localized",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let built_at = std::fs::metadata(paths::cards_db())
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs_f64());
    DataInfo {
        exists: true,
        cards,
        pt_names: pt,
        built_at,
    }
}

pub fn update_status() -> UpdateStatus {
    let s = crate::update::STATUS.lock().unwrap();
    UpdateStatus {
        state: s.state.to_string(),
        task: s.task.clone(),
        percent: s.percent,
        error: s.error.clone(),
        result: s.result.clone(),
    }
}

pub fn update_start() -> Result<()> {
    if crate::update::start() {
        Ok(())
    } else {
        Err(Error::Conflict(
            "Uma atualização já está em andamento.".into(),
        ))
    }
}

/// Stops an update in flight. The already-downloaded art stays: the cache is resumable, so
/// "cancel" means "pause", and re-running picks up exactly where this left off.
pub fn update_cancel() {
    crate::update::cancel();
}

/// How much art is cached, so the sidebar can say whether the app is genuinely offline-ready.
pub fn images_info() -> ImagesInfo {
    let bytes = images::cache_size_bytes();
    let cached: i64 = crate::db::open_cards_db()
        .and_then(|c| {
            c.query_row(
                "SELECT COUNT(*) FROM cards WHERE image_uri IS NOT NULL",
                [],
                |r| r.get(0),
            )
            .ok()
        })
        .unwrap_or(0);
    ImagesInfo {
        bytes,
        gb: (bytes as f64 / 1_073_741_824.0 * 100.0).round() / 100.0,
        cards_with_art: cached,
    }
}
