//! Filesystem watcher: refreshes git data when the worktree or `.git`
//! change behind the app's back (editor saves, CLI commits, branch
//! switches…).
//!
//! Raw notify events arrive in bursts (one save can produce several), so a
//! debounce thread coalesces them: the first event arms a timer, further
//! events keep resetting it, and only ~300ms of silence fires `on_change`
//! once. The app maps that to a single `GitCommand::Refresh`.

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
        if result.is_ok() {
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
    use std::sync::mpsc::channel;

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
