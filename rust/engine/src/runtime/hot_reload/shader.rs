//! Shader hot-reload watcher.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::EventKind;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;

/// Watches the `src/gpu/shaders/` directory for `.wgsl` file changes.
///
/// Uses `notify-debouncer-full` with 500ms debounce. The watcher runs on a
/// background thread; after queueing a change it calls `wake` (the window's
/// `request_redraw`) so an idle event loop picks it up without waiting for
/// input. Poll with [`ShaderWatcher::poll_changes`] from the render path.
pub struct ShaderWatcher {
    _debouncer: notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::RecommendedCache,
    >,
    rx: mpsc::Receiver<Vec<PathBuf>>,
}

impl ShaderWatcher {
    /// Start watching a shader directory.
    pub fn new(dir: &Path, wake: impl Fn() + Send + 'static) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel::<Vec<PathBuf>>();

        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: notify_debouncer_full::DebounceEventResult| match result {
                Ok(events) => {
                    // editors and sed save through hidden temp files
                    // (`.!123!quad.wgsl`, `.quad.wgsl.swp`): skip those, and
                    // the same path twice in one batch (notify sends dupes).
                    let mut paths: Vec<PathBuf> = events
                        .into_iter()
                        .filter(|ev| matches!(ev.kind, EventKind::Create(_) | EventKind::Modify(_)))
                        .flat_map(|ev| ev.event.paths)
                        .filter(|p| {
                            p.extension().is_some_and(|e| e == "wgsl")
                                && p.file_name()
                                    .and_then(|f| f.to_str())
                                    .is_some_and(|f| !f.starts_with('.'))
                        })
                        .collect();
                    paths.sort();
                    paths.dedup();
                    if !paths.is_empty() {
                        if tx.send(paths).is_err() {
                            log::warn!("shader hot-reload channel closed, change dropped");
                        }
                        wake();
                    }
                }
                Err(errors) => {
                    for e in errors {
                        log::error!("Shader watcher error: {e:?}");
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

impl crate::gpu::GpuContext {
    /// Apply every shader saved since the last frame to its pipeline(s).
    /// Returns true when something reloaded: the caller redraws every layer,
    /// since clean layers keep textures drawn with the old shader.
    pub fn poll_shader_reload(&mut self, effects: &mut crate::effects::EffectProcessor) -> bool {
        let Some(paths) = self.shader_watcher.as_ref().and_then(|w| w.poll_changes()) else {
            return false;
        };
        let mut reloaded = false;
        for name in paths.iter().filter_map(|p| p.file_name()?.to_str()) {
            let source = super::shader_source(name);
            let gpu_ok = self.reload_shader(name, &source);
            let fx_ok = effects.reload_shader(&self.device, name, &source);
            log::info!("shader hot reload: {name} (gpu: {gpu_ok}, effects: {fx_ok})");
            reloaded |= gpu_ok || fx_ok;
        }
        reloaded
    }
}
