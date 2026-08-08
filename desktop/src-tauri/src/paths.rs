//! Where the two databases and the frontend live.
//!
//! Resolution order, so the same binary works both when run from the repo during development
//! and when installed as a bundled app on any of the three targets:
//!   1. $SPELLBOOK_DATA_DIR / $SPELLBOOK_STATIC_DIR — explicit override, used by tests.
//!   2. The repo layout (../../data, ../../webapp), detected by walking up from the executable
//!      and from the current directory.
//!   3. The OS application-data directory (~/Library/Application Support/Spellbook on macOS,
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

/// Directory holding mtg.sqlite and the edhrec/ cache.
pub fn data_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SPELLBOOK_DATA_DIR") {
        return PathBuf::from(p);
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
