//! The ide's session recording (task-63 phase d): with `PLEV_RECORD=1` in
//! a debug build, what happens in the workbench goes into a
//! `rec::Recorder` on the workbench's clock (`Workbench::now`, the one its
//! receipts are stamped with), so `plev-inspect` can answer "why" about a
//! real session of a person and an agent in the ide.
//!
//! What the shell records, through this module:
//! - every stage of every receipt, from a sink on the workbench's one
//!   dispatcher (`Recording::attach`);
//! - who observed what through the agent port (`crate::agent`, the
//!   `Host::observed` of `Served`);
//! - the state as projected (`Recording::state`): after an agent's act,
//!   after a git result, after a person's click or key that changed
//!   something, and before every frame drawn; an unchanged state costs a
//!   diff and stores nothing;
//! - every frame drawn, with where each bound control shows in it
//!   (`Recording::frame`; render on demand, so no frame means nothing to
//!   record);
//! - a person's clicks and keys: the kind, where, the control under the
//!   pointer in the frame on screen, never a character
//!   (`Recording::input`).
//!
//! The recorder keeps a bounded window (the last 20 seconds by default)
//! and the shell writes it on the keymap's `session::Save` (`SAVE_KEYS`),
//! when the window closes, and on a panic, to `default_dir()`:
//! `ide-<pid>-<n>.monster` for a save, `session-<unix ms>-<pid>-<n>.monster`
//! from the panic hook. The directory is made private to the user, and
//! nothing leaves the machine. Without the variable nothing is created: no
//! recorder, no directory, no key.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use engine::control::keymap::{ActionRegistry, Keymap, KeymapError};
use engine::drawing::compositor::Compositor;
use record::{FileError, InputKind, Limits, Persisted, Recorder};

use crate::capability::Workbench;

/// The variable that turns the recording on in a debug build.
pub const ENV: &str = "PLEV_RECORD";

engine::actions!(session, [Save]);

/// The keys that save the retained window, in every context of the
/// workspace (the commit form's too): `cmd` is the Command key on macOS.
pub const SAVE_KEYS: [&str; 2] = ["cmd-shift-s", "ctrl-shift-s"];

/// True when `PLEV_RECORD` is `1` and this is a debug build.
pub fn enabled() -> bool {
    cfg!(debug_assertions) && std::env::var_os(ENV).is_some_and(|v| v == "1")
}

/// Where the ide writes its sessions: `$TMPDIR/plev/sessions`, beside the
/// agent port's sockets.
pub fn default_dir() -> PathBuf {
    std::env::temp_dir().join("plev").join("sessions")
}

/// Binds `session::Save` to `SAVE_KEYS` in `keymap`, in the workspace's
/// context, and registers it in `registry`. The shell calls it only when
/// it records.
pub fn bind_keys(keymap: &mut Keymap, registry: &mut ActionRegistry) -> Result<(), KeymapError> {
    for keys in SAVE_KEYS {
        keymap.bind(keys, Some("session::Save"), Some("Workspace"))?;
    }
    registry.register_default::<Save>();
    Ok(())
}

/// `dir`, made when missing, private to the user on unix (a directory
/// someone else owns fails here).
fn private_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
    }
    #[cfg(not(unix))]
    {
        fs::create_dir_all(dir)
    }
}

/// One recording of a workbench's session, and where it is saved.
pub struct Recording {
    rec: Recorder,
    dir: PathBuf,
    /// Saves written so far: the next file's number.
    saves: u32,
}

impl Recording {
    /// The ide's recording when `PLEV_RECORD=1` in a debug build, saving to
    /// `default_dir()`; `None` otherwise, and nothing is created.
    pub fn from_env() -> Option<Recording> {
        enabled().then(|| Recording::new(default_dir(), Limits::default()))
    }

    /// A recording kept within `limits`, saving to `dir` (made on the first
    /// save, or when the panic hook is installed).
    pub fn new(dir: impl Into<PathBuf>, limits: Limits) -> Recording {
        Recording {
            rec: Recorder::new(limits),
            dir: dir.into(),
            saves: 0,
        }
    }

    pub fn recorder(&self) -> &Recorder {
        &self.rec
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Puts a sink on `bench`'s dispatcher, so every stage of every receipt
    /// is recorded as it happens, and records the state now, the first
    /// keyframe.
    pub fn attach(&self, bench: &mut Workbench) {
        bench.add_sink(Box::new(self.rec.sink()));
        self.state(bench);
    }

    /// The state `bench` projects now; stored only when it changed.
    pub fn state(&self, bench: &Workbench) {
        match bench.snapshot() {
            Ok(snap) => self.rec.snapshot(bench.now(), &snap),
            Err(e) => log::warn!("recording: the projection does not validate: {e}"),
        }
    }

    /// The frame `compositor` holds, just drawn from `bench`, with where
    /// each bound control shows in it, after the state it was drawn from.
    pub fn frame(&self, bench: &Workbench, compositor: &Compositor) {
        self.state(bench);
        if let Err(e) = self.rec.frame(bench.now(), compositor) {
            log::debug!("recording: a frame is not recorded: {e}");
        }
    }

    /// A person's input now: its kind, where it landed in logical window
    /// pixels, and the control bound under it in the frame on screen
    /// (`compositor`). Never its content.
    pub fn input(
        &self,
        bench: &Workbench,
        kind: InputKind,
        at: Option<[f32; 2]>,
        compositor: &Compositor,
    ) {
        let target = at.and_then(|[x, y]| compositor.hit(x, y).cloned());
        self.rec.input(bench.now(), kind, at, target);
    }

    /// The path the next save writes.
    pub fn next_path(&self) -> PathBuf {
        let pid = std::process::id();
        self.dir.join(format!("ide-{pid}-{}.monster", self.saves))
    }

    /// Writes the retained window to the next `ide-<pid>-<n>.monster` in
    /// the directory; the path and what went in it.
    pub fn save(&mut self) -> Result<(PathBuf, Persisted), FileError> {
        private_dir(&self.dir)?;
        let path = self.next_path();
        let written = self.rec.persist(&path)?;
        self.saves += 1;
        Ok((path, written))
    }

    /// On a panic, the retained window goes to the directory
    /// (`rec::install_panic_hook`), before the hook installed before runs.
    pub fn catch_panics(&self) -> io::Result<()> {
        private_dir(&self.dir)?;
        record::install_panic_hook(self.rec.clone(), self.dir.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use engine::control::keymap::{
        ActionRegistry, ContextStack, KeyContext, Keymap, KeymapMatcher,
    };
    use engine::control::keymap::{Keystroke, MatchResult};

    use super::{SAVE_KEYS, bind_keys};

    fn stack(names: &[&str]) -> ContextStack {
        names
            .iter()
            .map(|n| KeyContext::new(*n))
            .collect::<Vec<_>>()
            .into()
    }

    #[test]
    fn the_save_keys_resolve_in_every_context_and_the_action_builds() {
        let mut keymap = Keymap::new();
        let mut registry = ActionRegistry::new();
        bind_keys(&mut keymap, &mut registry).expect("the keys parse");
        let mut matcher = KeymapMatcher::new(keymap);
        for keys in SAVE_KEYS {
            let keystroke: Keystroke = keys.parse().expect("a keystroke");
            for contexts in [
                &["Workspace"][..],
                &["Workspace", "CommitForm"],
                &["Workspace", "CommitForm", "Overlay"],
            ] {
                assert_eq!(
                    matcher.match_keystroke(&keystroke, &stack(contexts)),
                    MatchResult::Complete("session::Save".into()),
                    "{keys} in {contexts:?}"
                );
            }
        }
        assert!(registry.build("session::Save").is_some());
    }
}
