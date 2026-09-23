//! the flight recorder's last act: on a panic, the retained window goes
//! to a file before the previous hook runs.
//!
//! a panic hook must not panic (a panic while panicking aborts the
//! process), so everything here is fallible and ignored on failure. the
//! panic may have come from inside the recorder, with its lock held by the
//! panicking thread: the hook only tries the lock, a few times, and gives
//! up rather than wait on itself.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::recorder::Recorder;

/// how long the hook waits for the recorder's lock: tries, and the pause
/// between two.
const TRIES: u32 = 20;
const PAUSE: Duration = Duration::from_millis(5);

/// persists `recorder`'s window to `dir` whenever a thread panics, then
/// runs the hook that was installed before (the default one prints the
/// message). the file is `session-<unix ms>-<pid>-<n>.monster`; the name
/// reads the wall clock, the recording never does. `dir` is created when
/// missing. nothing here ever panics: a window that cannot be written is
/// skipped.
pub fn install_panic_hook(recorder: Recorder, dir: impl Into<PathBuf>) {
    let dir = dir.into();
    let previous = std::panic::take_hook();
    let count = AtomicU32::new(0);
    std::panic::set_hook(Box::new(move |info| {
        let n = count.fetch_add(1, Ordering::Relaxed);
        let _ = persist_now(&recorder, &dir, n);
        previous(info);
    }));
}

/// the path of the n-th window the hook writes now.
fn name(dir: &Path, n: u32) -> PathBuf {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    dir.join(format!("session-{ms}-{}-{n}.monster", std::process::id()))
}

fn persist_now(recorder: &Recorder, dir: &Path, n: u32) -> Option<PathBuf> {
    let session = (0..TRIES).find_map(|at| {
        if at > 0 {
            std::thread::sleep(PAUSE);
        }
        recorder.try_window()
    })?;
    std::fs::create_dir_all(dir).ok()?;
    let path = name(dir, n);
    session.write(&path).ok()?;
    Some(path)
}
