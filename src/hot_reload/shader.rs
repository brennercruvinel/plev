//! Shader hot-reload watcher.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::EventKind;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;

/// Watches the `shaders/` directory for `.wgsl` file changes.
///
/// Uses `notify-debouncer-full` with 500ms debounce. The watcher runs on a
/// background thread; poll with [`ShaderWatcher::poll_changes`] from the
/// event loop.
pub struct ShaderWatcher {
    _debouncer: notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::RecommendedCache,
    >,
    rx: mpsc::Receiver<Vec<PathBuf>>,
}

impl ShaderWatcher {
    /// Start watching a shader directory.
    pub fn new(dir: &Path) -> Result<Self, notify::Error> {
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
                                .map_or(false, |e| e == "wgsl")
                        })
                        .collect();
                    if !paths.is_empty() {
                        if tx.send(paths).is_err() {
                            log::warn!(
                                "Shader hot-reload channel closed \
                                 — changes will not be applied"
                            );
                        }
                    }
                }
                Err(errors) => {
                    for e in errors {
                        log::error!("Shader watcher error: {:?}", e);
                    }
                }
            },
        )?;

        debouncer.watch(dir, RecursiveMode::NonRecursive)?;

        Ok(Self {
            _debouncer: debouncer,
            rx,
        })
    }

    /// Non-blocking poll for changed shader paths. Returns `None` if nothing changed.
    pub fn poll_changes(&self) -> Option<Vec<PathBuf>> {
        self.rx.try_recv().ok()
    }
}
