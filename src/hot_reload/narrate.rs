//! Narrate DSL hot-reload: watcher + override map.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, mpsc};
use std::time::Duration;

use notify::EventKind;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;

use crate::builder::Element;
use crate::narrate_runtime;

// ── Narrate DSL Override Map ──

/// Global override map: `(file, line) -> DSL text`.
///
/// When a .rs file with plev_narrate! blocks changes, the watcher extracts
/// the blocks and stores them here. `narrate_override()` re-parses on each
/// call (microseconds) to avoid requiring Element to be Clone.
static NARRATE_OVERRIDES: LazyLock<Mutex<HashMap<(String, u32), String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Project root for converting absolute paths to relative.
pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Check if an override exists for a narrate block at the given source location.
///
/// Called from `narrate_resolve()` in the hot path. Returns a freshly parsed
/// Element or None if no override exists.
pub fn narrate_override(file: &str, line: u32) -> Option<Element> {
    let map = NARRATE_OVERRIDES.lock().ok()?;
    let dsl = map.get(&(file.to_string(), line))?;
    narrate_runtime::parse_narrate(dsl)
}

/// Update the override map with new blocks extracted from a changed file.
///
/// `file_rel` is the path relative to the project root (matching `file!()`).
pub fn update_narrate_overrides(file_rel: &str, blocks: Vec<(u32, String)>) {
    if let Ok(mut map) = NARRATE_OVERRIDES.lock() {
        // Remove old entries for this file
        map.retain(|(f, _), _| f != file_rel);
        // Insert new entries
        for (line, content) in blocks {
            map.insert((file_rel.to_string(), line), content);
        }
        if !map.is_empty() {
            log::info!(
                "Narrate hot reload: {} override(s) active for {}",
                map.len(),
                file_rel
            );
        }
    }
}

// ── Narrate Watcher ──

/// Watches `src/` and `examples/` for `.rs` file changes.
///
/// On change, extracts plev_narrate! blocks and updates the override map.
pub struct NarrateWatcher {
    _debouncer: notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::RecommendedCache,
    >,
    rx: mpsc::Receiver<Vec<PathBuf>>,
}

impl NarrateWatcher {
    /// Start watching directories for .rs file changes.
    pub fn new(dirs: &[&Path]) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel::<Vec<PathBuf>>();

        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: notify_debouncer_full::DebounceEventResult| match result {
                Ok(events) => {
                    let paths: Vec<PathBuf> = events
                        .into_iter()
                        .filter(|ev| matches!(ev.kind, EventKind::Create(_) | EventKind::Modify(_)))
                        .flat_map(|ev| ev.event.paths)
                        .filter(|p| {
                            p.extension()
                                .and_then(|e| e.to_str())
                                .map_or(false, |e| e == "rs")
                        })
                        .collect();
                    if !paths.is_empty() {
                        if tx.send(paths).is_err() {
                            log::warn!(
                                "Narrate hot-reload channel closed \
                                 — changes will not be applied"
                            );
                        }
                    }
                }
                Err(errors) => {
                    for e in errors {
                        log::error!("Narrate watcher error: {:?}", e);
                    }
                }
            },
        )?;

        for dir in dirs {
            if dir.exists() {
                debouncer.watch(dir, RecursiveMode::Recursive)?;
            }
        }

        Ok(Self {
            _debouncer: debouncer,
            rx,
        })
    }

    /// Non-blocking poll for changed .rs file paths.
    pub fn poll_changes(&self) -> Option<Vec<PathBuf>> {
        self.rx.try_recv().ok()
    }
}

/// Process a changed .rs file: extract narrate blocks and update overrides.
pub fn process_narrate_file(path: &Path) {
    let root = project_root();
    let rel = path
        .strip_prefix(&root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    match std::fs::read_to_string(path) {
        Ok(source) => {
            let blocks = narrate_runtime::extract_narrate_blocks(&source);
            if !blocks.is_empty() {
                log::info!(
                    "Narrate hot reload: found {} block(s) in {}",
                    blocks.len(),
                    rel
                );
                update_narrate_overrides(&rel, blocks);
            }
        }
        Err(e) => {
            log::error!("Failed to read {}: {}", path.display(), e);
        }
    }
}
