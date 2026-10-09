//! Saved `.rs` files. A literal-only change (see `literals`) is applied in
//! place and the window redraws. A code change is what the compiler owns:
//! `cargo build` the bin (or example) this process came from and `exec` the
//! fresh binary with the same args, the loop cargo-watch would run. The pid,
//! the terminal and `cargo run` all stay. A failed build logs and leaves the
//! old binary running.

use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use notify::EventKind;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;

use super::literals::Change;

pub struct SourceWatcher {
    _debouncer: notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::RecommendedCache,
    >,
}

impl SourceWatcher {
    /// Watch `dirs` (recursively) for saved `.rs` files; `wake` (the
    /// window's `request_redraw`) runs after a literal change so the frame
    /// that shows it happens now.
    pub fn new(dirs: &[PathBuf], wake: impl Fn() + Send + 'static) -> Result<Self, notify::Error> {
        let baseline_dirs = dirs.to_vec();
        std::thread::spawn(move || super::literals::take_baseline(&baseline_dirs));
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: notify_debouncer_full::DebounceEventResult| match result {
                Ok(events) => {
                    let mut paths: Vec<&PathBuf> = events
                        .iter()
                        .filter(|ev| matches!(ev.kind, EventKind::Create(_) | EventKind::Modify(_)))
                        .flat_map(|ev| ev.event.paths.iter())
                        .filter(|p| is_rust_source(p))
                        .collect();
                    paths.sort();
                    paths.dedup();
                    let mut redraw = false;
                    for path in paths {
                        let rel = path
                            .strip_prefix(super::project_root())
                            .unwrap_or(path)
                            .display();
                        match super::literals::apply_change(path) {
                            Change::None => {}
                            Change::Literals(n) => {
                                log::info!("{rel}: {n} literal(s) hot reloaded");
                                redraw = true;
                            }
                            Change::Code => {
                                rebuild_and_relaunch(path);
                                return;
                            }
                        }
                    }
                    if redraw {
                        wake();
                    }
                }
                Err(errors) => {
                    for e in errors {
                        log::error!("source watcher error: {e:?}");
                    }
                }
            },
        )?;
        for dir in dirs {
            debouncer.watch(dir, RecursiveMode::Recursive)?;
        }
        Ok(Self {
            _debouncer: debouncer,
        })
    }
}

/// The directories that hold the crates: the libraries, the apps and the
/// demos (the benches were examples before the move and were never watched).
const CRATE_ROOTS: [&str; 3] = ["rust", "apps", "demo"];

/// `src/` of every crate in the workspace (not the crate dirs themselves:
/// urnadbapp keeps its own target/ in there).
pub fn source_dirs() -> Vec<PathBuf> {
    let root = super::project_root();
    CRATE_ROOTS
        .iter()
        .flat_map(|r| {
            std::fs::read_dir(root.join(r))
                .into_iter()
                .flatten()
                .flatten()
        })
        .map(|e| e.path().join("src"))
        .filter(|d| d.is_dir())
        .collect()
}

fn is_rust_source(p: &Path) -> bool {
    p.extension().is_some_and(|e| e == "rs")
        && p.file_name()
            .and_then(|f| f.to_str())
            .is_some_and(|f| !f.starts_with('.'))
}

/// `cargo build` the target this process was built as, then replace the
/// process with the result. Runs on the watcher thread; `exec` never returns
/// on success.
fn rebuild_and_relaunch(changed: &Path) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(name) = exe.file_stem().and_then(|s| s.to_str()) else {
        return;
    };
    let is_example = exe
        .parent()
        .and_then(|d| d.file_name())
        .is_some_and(|d| d == "examples");
    let kind = if is_example { "--example" } else { "--bin" };
    // `cargo run` exports these; `-p` keeps the feature set of the original
    // build (a bare `--bin` at the workspace root unifies features across
    // every member and rebuilds the world).
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut args = vec!["build".to_string()];
    if let Ok(pkg) = std::env::var("CARGO_PKG_NAME") {
        args.push("-p".into());
        args.push(pkg);
    }
    args.push(kind.into());
    args.push(name.into());

    log::info!(
        "{}: code changed, cargo {}",
        changed
            .strip_prefix(super::project_root())
            .unwrap_or(changed)
            .display(),
        args.join(" ")
    );
    let status = Command::new(cargo)
        .current_dir(super::project_root())
        .args(&args)
        .status();
    match status {
        Ok(s) if s.success() => {
            let err = Command::new(&exe).args(std::env::args_os().skip(1)).exec();
            log::error!("relaunch of {} failed: {err}", exe.display());
        }
        Ok(s) => log::error!("build failed ({s}); the old binary keeps running"),
        Err(e) => log::error!("could not run cargo: {e}"),
    }
}
