//! Where the app keeps its data and finds its frontend.
//!
//! Spellbook is a native macOS app, so it stores everything the way one is expected to:
//! under ~/Library/Application Support/Spellbook, which the app owns outright. Nothing lives
//! in the source repo any more, and nothing is read from ~/Documents — that folder is
//! TCC-protected, and reading from it forced a permission prompt on every fresh install (and,
//! before the startup order was fixed, deadlocked the launch entirely).
//!
//!   ~/Library/Application Support/Spellbook/
//!     app.db            the user's decks / collection / games
//!     data/mtg.sqlite   the Scryfall card index
//!     data/edhrec/      cached per-commander synergy
//!     data/images/      locally cached card art (for offline use)
//!     config.json       optional overrides, see below
//!
//! The frontend ships inside the .app bundle (Contents/Resources/static), so the installed app
//! is self-contained and doesn't depend on a checkout being present.
//!
//! Resolution order:
//!   1. $SPELLBOOK_* environment overrides — used by the test suite to run against a scratch dir.
//!   2. config.json in the app-data dir — escape hatch for pointing at data kept elsewhere.
//!   3. The app-data dir itself (the normal case).
//! The card index is deliberately NOT bundled: it's ~31MB, rebuilt from Scryfall by the in-app
//! updater, and belongs to the user rather than the installer.

use std::path::PathBuf;

/// ~/Library/Application Support/Spellbook (and the OS equivalent elsewhere).
pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Spellbook")
}

/// Reads a path out of the optional config.json, e.g.
///     { "data_dir": "/somewhere/else/data", "app_db_dir": "/somewhere/else" }
///
/// Hand-rolled rather than pulling in a JSON parse here: the file has at most a few string keys,
/// and a malformed one must never stop the app from booting — it just falls through to the
/// default location.
fn config_value(key: &str) -> Option<PathBuf> {
    let raw = std::fs::read_to_string(app_data_dir().join("config.json")).ok()?;
    let needle = format!("\"{key}\"");
    let after = raw.split(&needle).nth(1)?;
    let after = after.split(':').nth(1)?;
    let start = after.find('"')? + 1;
    let rest = &after[start..];
    let end = rest.find('"')?;
    let path = PathBuf::from(&rest[..end]);
    path.is_dir().then_some(path)
}

fn resolve(env_key: &str, config_key: &str, default: PathBuf) -> PathBuf {
    if let Ok(p) = std::env::var(env_key) {
        return PathBuf::from(p);
    }
    if let Some(p) = config_value(config_key) {
        return p;
    }
    default
}

/// Directory holding mtg.sqlite, the edhrec/ cache and cached images.
pub fn data_dir() -> PathBuf {
    resolve("SPELLBOOK_DATA_DIR", "data_dir", app_data_dir().join("data"))
}

/// Directory holding app.db (the user's own decks/collection).
pub fn app_db_dir() -> PathBuf {
    resolve("SPELLBOOK_APP_DB_DIR", "app_db_dir", app_data_dir())
}

/// Directory holding index.html + assets, normally shipped inside the bundle.
pub fn static_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SPELLBOOK_STATIC_DIR") {
        return PathBuf::from(p);
    }
    if let Some(p) = config_value("static_dir") {
        return p;
    }
    // Installed: Contents/MacOS/spellbook -> Contents/Resources/static
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("..").join("Resources").join("static");
            if bundled.is_dir() {
                return bundled;
            }
        }
    }
    // Running under `cargo run` from the repo, where there is no bundle yet.
    if let Ok(cwd) = std::env::current_dir() {
        let mut cur = Some(cwd.as_path());
        while let Some(dir) = cur {
            let candidate = dir.join("webapp").join("static");
            if candidate.is_dir() {
                return candidate;
            }
            cur = dir.parent();
        }
    }
    app_data_dir().join("static")
}

pub fn cards_db() -> PathBuf {
    data_dir().join("mtg.sqlite")
}

pub fn app_db() -> PathBuf {
    app_db_dir().join("app.db")
}

/// Locally cached card art, for true offline use (the index stores remote scryfall.io URLs).
pub fn images_dir() -> PathBuf {
    data_dir().join("images")
}

/// Creates the app-data layout on first run. Safe to call every launch.
pub fn ensure_dirs() {
    let _ = std::fs::create_dir_all(app_db_dir());
    let _ = std::fs::create_dir_all(data_dir());
}
