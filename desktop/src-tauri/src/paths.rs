//! Where the two databases and the frontend live.
//!
//! Resolution order, so the same binary works both when run from the repo during development
//! and when installed as a bundled app:
//!   1. $SPELLBOOK_DATA_DIR / $SPELLBOOK_APP_DB_DIR / $SPELLBOOK_STATIC_DIR — explicit
//!      override, used by tests.
//!   2. config.json in the app-data directory — how an installed .app (which sits outside the
//!      repo, so step 3 can't work) is pointed at an existing data/app.db.
//!   3. The repo layout (../../data, ../../webapp), detected by walking up from the executable
//!      and from the current directory.
//!   4. The OS application-data directory (~/Library/Application Support/Spellbook on macOS,
//!      %APPDATA%\Spellbook on Windows, ~/.local/share/spellbook on Linux).
//!
//! The card index (mtg.sqlite, ~31MB) and the bulk downloads are deliberately NOT bundled into
//! the app: they're rebuilt from Scryfall by the in-app updater, exactly as the Python version
//! does, so the installer stays small and the data stays user-owned.

use std::path::{Path, PathBuf};

fn repo_root_from(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        // The repo root is the directory that has both of these.
        if dir.join("data").is_dir() && dir.join("webapp").join("static").is_dir() {
            return Some(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    None
}

fn detect_repo_root() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(root) = repo_root_from(&exe) {
            return Some(root);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(root) = repo_root_from(&cwd) {
            return Some(root);
        }
    }
    None
}

fn app_data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Spellbook")
}

/// Optional `config.json` in the app-data dir, e.g.
///     { "data_dir": "/path/to/repo/data", "app_db_dir": "/path/to/repo/webapp" }
///
/// Needed because an installed .app lives outside the repo, so walking up from the executable
/// can't find `data/` or `app.db` any more. Rather than hardcoding a machine-specific path into
/// the bundle (which would break on anyone else's Mac), the bundle stays generic and the pointer
/// lives beside the user's own data. Absent or unreadable, the normal resolution order applies.
fn config_value(key: &str) -> Option<PathBuf> {
    let raw = std::fs::read_to_string(app_data_dir().join("config.json")).ok()?;
    // Deliberately a hand-rolled scan rather than pulling serde_json into this module: the file
    // has at most a couple of string keys, and a malformed one must never stop the app booting.
    let needle = format!("\"{key}\"");
    let after = raw.split(&needle).nth(1)?;
    let after = after.split(':').nth(1)?;
    let start = after.find('"')? + 1;
    let rest = &after[start..];
    let end = rest.find('"')?;
    let path = PathBuf::from(&rest[..end]);
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

/// Directory holding mtg.sqlite and the edhrec/ cache.
pub fn data_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SPELLBOOK_DATA_DIR") {
        return PathBuf::from(p);
    }
    if let Some(p) = config_value("data_dir") {
        return p;
    }
    if let Some(root) = detect_repo_root() {
        return root.join("data");
    }
    app_data_dir().join("data")
}

/// Directory holding app.db (the user's own decks/collection).
pub fn app_db_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SPELLBOOK_APP_DB_DIR") {
        return PathBuf::from(p);
    }
    if let Some(p) = config_value("app_db_dir") {
        return p;
    }
    if let Some(root) = detect_repo_root() {
        return root.join("webapp");
    }
    app_data_dir()
}

/// Directory holding index.html + assets.
pub fn static_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SPELLBOOK_STATIC_DIR") {
        return PathBuf::from(p);
    }
    if let Some(p) = config_value("static_dir") {
        return p;
    }
    if let Some(root) = detect_repo_root() {
        return root.join("webapp").join("static");
    }
    // Bundled: Tauri copies the frontend in next to the executable as ../Resources/static.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("..").join("Resources").join("static");
            if bundled.is_dir() {
                return bundled;
            }
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
