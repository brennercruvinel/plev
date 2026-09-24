//! Filesystem watcher: refreshes git data when the worktree or `.git`
//! change behind the app's back (editor saves, CLI commits, branch
//! switches…).
//!
//! Raw notify events arrive in bursts (one save can produce several), so a
//! debounce thread coalesces them: the first event arms a timer, further
//! events keep resetting it, and only ~300ms of silence fires `on_change`
//! once. The app maps that to a single `GitCommand::Refresh`.
//!
//! An event that only touches `.git/index.lock` is not a change
//! ([`is_change`]): a git process that refreshes the index's stat data
//! creates and removes that lock without changing anything the app
//! shows, and a refresh per lock kept the ide drawing forever.

use std::path::Path;
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

const DEBOUNCE: Duration = Duration::from_millis(300);

/// Keep this handle alive for as long as watching should continue —
/// dropping it stops the OS watcher and ends the debounce thread.
pub struct FsWatcher {
    _watcher: RecommendedWatcher,
}

/// Watches `root` recursively (the worktree root, which includes `.git`)
/// and calls `on_change` from a background thread after events settle.
pub fn spawn(root: &Path, on_change: impl Fn() + Send + 'static) -> notify::Result<FsWatcher> {
    let (tx, rx) = channel::<()>();
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if result.is_ok_and(|event| is_change(&event)) {
            let _ = tx.send(());
        }
    })?;
    watcher.watch(root, RecursiveMode::Recursive)?;

    std::thread::Builder::new()
        .name("fs-debounce".into())
        .spawn(move || debounce_loop(rx, DEBOUNCE, on_change))
        .expect("spawn fs debounce thread");

    Ok(FsWatcher { _watcher: watcher })
}

/// Whether a raw event is worth a refresh. Not when every path it names
/// is `.git/index.lock`: `git status` takes that lock to store the stat
/// data it refreshed, and the refresh the event asked for ran `git status`
/// again, a loop that never ended (task-82 phase b measured two frames
/// every ~0.34 s and a `git status` a turn, with the app idle). The
/// ide's own reads no longer take the lock (`GIT_OPTIONAL_LOCKS=0` in the
/// `git` crate); this keeps another git on the machine (a shell prompt,
/// an editor) from starting the loop through the lock. The index those
/// processes write lands on `.git/index`, which still refreshes, once.
/// An event with no path (a rescan) is a change.
fn is_change(event: &notify::Event) -> bool {
    event.paths.is_empty() || event.paths.iter().any(|path| !is_index_lock(path))
}

/// `<something>/.git/index.lock`.
fn is_index_lock(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == "index.lock")
        && path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name == ".git")
}

/// Coalesces bursts of ticks: fires `on_change` once per burst, after
/// `quiet` time with no new ticks. Returns when the sender side hangs up.
fn debounce_loop(rx: Receiver<()>, quiet: Duration, on_change: impl Fn()) {
    while rx.recv().is_ok() {
        loop {
            match rx.recv_timeout(quiet) {
                Ok(()) => continue, // burst still going — reset the timer
                Err(RecvTimeoutError::Timeout) => {
                    on_change();
                    break;
                }
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind};
    use std::path::PathBuf;
    use std::sync::mpsc::channel;

    fn event(kind: EventKind, paths: &[&str]) -> notify::Event {
        paths.iter().fold(notify::Event::new(kind), |e, p| {
            e.add_path(PathBuf::from(p))
        })
    }

    #[test]
    fn the_index_lock_alone_is_not_a_change() {
        let create = EventKind::Create(CreateKind::File);
        let remove = EventKind::Remove(RemoveKind::File);
        let lock = "/repo/.git/index.lock";
        assert!(!is_change(&event(create, &[lock])));
        assert!(!is_change(&event(remove, &[lock])));
        let modify = EventKind::Modify(ModifyKind::Any);
        // what the lock lands on, or anything else, still refreshes
        assert!(is_change(&event(modify, &["/repo/.git/index"])));
        assert!(is_change(&event(modify, &[lock, "/repo/.git/index"])));
        assert!(is_change(&event(modify, &["/repo/src/lib.rs"])));
        assert!(is_change(&event(
            create,
            &["/repo/.git/refs/heads/main.lock"]
        )));
        assert!(is_change(&event(create, &["/repo/src/index.lock"])));
        assert!(is_change(&event(modify, &[])), "a rescan names no path");
    }

    /// The real watcher on a real directory: churning the lock fires
    /// nothing, a file written in the worktree fires once.
    #[test]
    fn a_watched_repository_ignores_the_lock_and_sees_a_write() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let git = dir.path().join(".git");
        std::fs::create_dir(&git).expect(".git");
        let (fire_tx, fire_rx) = channel();
        let _watcher = spawn(dir.path(), move || {
            let _ = fire_tx.send(());
        })
        .expect("watch");
        // the OS stream may replay the directories created just before
        // it started: let those fire and fall quiet first
        while fire_rx.recv_timeout(Duration::from_millis(800)).is_ok() {}
        for _ in 0..3 {
            std::fs::write(git.join("index.lock"), b"x").expect("lock");
            std::fs::remove_file(git.join("index.lock")).expect("unlock");
        }
        assert!(
            fire_rx.recv_timeout(Duration::from_millis(1200)).is_err(),
            "the lock alone refreshed"
        );
        std::fs::write(dir.path().join("a.txt"), b"changed").expect("write");
        assert!(fire_rx.recv_timeout(Duration::from_secs(5)).is_ok());
    }

    #[test]
    fn debounce_coalesces_bursts_and_fires_per_burst() {
        let (tick_tx, tick_rx) = channel();
        let (fire_tx, fire_rx) = channel();
        std::thread::spawn(move || {
            debounce_loop(tick_rx, Duration::from_millis(50), move || {
                fire_tx.send(()).unwrap();
            });
        });

        // A burst of 5 ticks -> exactly one fire.
        for _ in 0..5 {
            tick_tx.send(()).unwrap();
        }
        assert!(fire_rx.recv_timeout(Duration::from_secs(2)).is_ok());
        assert!(
            fire_rx.recv_timeout(Duration::from_millis(200)).is_err(),
            "burst must coalesce into a single fire"
        );

        // A later burst fires again.
        tick_tx.send(()).unwrap();
        assert!(fire_rx.recv_timeout(Duration::from_secs(2)).is_ok());
    }
}
