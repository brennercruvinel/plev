//! Recently-opened .nest files, persisted as a plain JSON array.
//!
//! Location follows the nest CLI's data-home convention: `$XDG_DATA_HOME/
//! nestui/recents.json`, else `~/.local/share/nestui/recents.json`. No
//! `dirs` dependency — the workspace doesn't have one and two candidates
//! are easy to write by hand.

use std::path::PathBuf;

/// Cap on the recents list (the Open screen shows it verbatim).
pub const MAX_RECENTS: usize = 10;

/// Default recents file location (`None` when no home is discoverable).
pub fn default_path() -> Option<PathBuf> {
    let data_home = std::env::var("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".local").join("share"))
        })?;
    Some(data_home.join("nestui").join("recents.json"))
}

/// Load the recents list; a missing or malformed file reads as empty
/// (recents are a convenience, never worth an error dialog).
pub fn load(path: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default()
}

/// Push `path` to the front (deduped, capped) and persist. Failures to
/// write are logged and swallowed — same rationale as `load`.
pub fn record(recents_path: &PathBuf, path: &str) -> Vec<String> {
    let mut recents = load(recents_path);
    recents.retain(|p| p != path);
    recents.insert(0, path.to_string());
    recents.truncate(MAX_RECENTS);
    if let Some(parent) = recents_path.parent()
        && std::fs::create_dir_all(parent).is_ok()
        && let Ok(json) = serde_json::to_string_pretty(&recents)
        && let Err(e) = std::fs::write(recents_path, json)
    {
        log::warn!("failed to persist recents: {e}");
    }
    recents
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_loads_empty() {
        let path = std::env::temp_dir().join(format!("nestui_recents_none_{}", std::process::id()));
        assert!(load(&path).is_empty());
    }

    #[test]
    fn record_dedupes_caps_and_roundtrips() {
        let dir = std::env::temp_dir().join(format!("nestui_recents_{}", std::process::id()));
        let path = dir.join("recents.json");

        for i in 0..12 {
            record(&path, &format!("/db/{i}.nest"));
        }
        let recents = load(&path);
        assert_eq!(recents.len(), MAX_RECENTS);
        // Most recent first, oldest evicted.
        assert_eq!(recents[0], "/db/11.nest");
        assert_eq!(recents[MAX_RECENTS - 1], "/db/2.nest");

        // Re-opening an existing entry moves it to the front.
        record(&path, "/db/5.nest");
        let recents = load(&path);
        assert_eq!(recents[0], "/db/5.nest");
        assert_eq!(
            recents.iter().filter(|p| *p == "/db/5.nest").count(),
            1,
            "no duplicates"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
