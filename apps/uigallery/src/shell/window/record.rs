//! The gallery's session recording (task-69 phase 4c), in the form of the
//! ide's (`idestudio::interfaces::window::record`, task-63 phase d): with `PLEV_RECORD=1` in a debug
//! build, what happens in the window goes into a `record::Recorder` on the
//! gallery's clock (`ShowcaseView::now`, the one its receipts are stamped
//! with), so `plev-inspect` can answer "why" about a real session of a
//! person and an agent on a page, the generated screen of the Silences
//! page included: the recorded demo is one of these sessions
//! (`examples/demo.rs`, `fixtures/demo`).
//!
//! What the shell records, through this module:
//! - every stage of every receipt, from a sink on the gallery's one
//!   dispatcher (`Recording::attach`, `ShowcaseView::add_sink`);
//! - who observed what through the agent port, and the state an act
//!   left (`ShowcaseView::serve_agent_recorded`);
//! - the state as projected (`Recording::state`): after a person's click
//!   or key that changed something, after a screen reader's request,
//!   after a git result shown, and before every frame drawn; an unchanged
//!   state costs a diff and stores nothing;
//! - every frame drawn, with where each bound control shows in it
//!   (`Recording::frame`; render on demand, so no frame means nothing to
//!   record);
//! - a person's clicks and keys: the kind, where, the control under the
//!   pointer in the frame on screen, never a character
//!   (`Recording::input`).
//!
//! The recorder keeps a bounded window (the last 20 seconds by default)
//! and the shell writes it on the save keys (`SAVE_KEYS`: cmd-shift-s on
//! macOS, ctrl-shift-s elsewhere; the keys and the menu item are the
//! shell's `shell.save_session`, a person's request through the
//! gallery's dispatcher, task-76 item 4), when the window closes, and on
//! a panic, to `default_dir()`: `showcase-<pid>-<n>.monster` for a save,
//! `session-<unix ms>-<pid>-<n>.monster` from the panic hook. The
//! directory is made private to the user, and nothing leaves the
//! machine. Without the variable nothing is created: no recorder, no
//! directory, no key. Desktop only, like the port: on wasm and mobile the
//! shell has none of this.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use engine::drawing::compositor::Compositor;
use record::{FileError, InputKind, Limits, Persisted, Recorder};
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState};
use winit::window::WindowId;

use super::super::App;
use crate::views::ShowcaseView;

/// The variable that turns the recording on in a debug build.
pub const ENV: &str = "PLEV_RECORD";

/// The keys that save the retained window, as the person reads them: `s`
/// with shift and the primary modifier (cmd on macOS, ctrl elsewhere),
/// the ide's `session::Save`.
pub const SAVE_KEYS: &str = if cfg!(target_os = "macos") {
    "cmd-shift-s"
} else {
    "ctrl-shift-s"
};

/// True when `PLEV_RECORD` is `1` and this is a debug build.
pub fn enabled() -> bool {
    cfg!(debug_assertions) && says(std::env::var_os(ENV).as_deref())
}

fn says(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}

/// Where the gallery writes its sessions: `$TMPDIR/plev/sessions`, beside
/// the agent port's sockets, where the ide writes its own.
pub fn default_dir() -> PathBuf {
    std::env::temp_dir().join("plev").join("sessions")
}

/// True for `SAVE_KEYS`: the character `s`, shift held, and cmd on macOS
/// or ctrl elsewhere.
pub fn is_save_key(key: &Key, modifiers: ModifiersState) -> bool {
    let primary = if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    };
    primary
        && modifiers.shift_key()
        && matches!(key, Key::Character(c) if c.eq_ignore_ascii_case("s"))
}

/// The kind a key is recorded as: a printable is `text`, anything else
/// (an editing key, Escape, Enter) a `key`; never which one.
fn kind_of(key: &Key) -> InputKind {
    match key {
        Key::Character(_) => InputKind::Text,
        _ => InputKind::Key,
    }
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

/// One recording of the gallery's session, and where it is saved.
pub struct Recording {
    rec: Recorder,
    dir: PathBuf,
    /// Saves written so far: the next file's number.
    saves: u32,
}

impl Recording {
    /// The gallery's recording when `PLEV_RECORD=1` in a debug build,
    /// saving to `default_dir()`; `None` otherwise, and nothing is
    /// created.
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

    /// Puts a sink on the gallery's dispatcher, so every stage of every
    /// receipt is recorded as it happens, and records the state now, the
    /// first keyframe.
    pub fn attach(&self, view: &mut ShowcaseView) {
        view.add_sink(Box::new(self.rec.sink()));
        self.state(view);
    }

    /// The state the active page projects now; stored only when it
    /// changed. A page without a projection records nothing.
    pub fn state(&self, view: &mut ShowcaseView) {
        if let Some(snapshot) = view.semantics() {
            self.rec.snapshot(view.now(), &snapshot);
        }
    }

    /// The frame `compositor` holds, just drawn from `view`, with where
    /// each bound control shows in it, after the state it was drawn from.
    pub fn frame(&self, view: &mut ShowcaseView, compositor: &Compositor) {
        self.state(view);
        if let Err(e) = self.rec.frame(view.now(), compositor) {
            log::debug!("recording: a frame is not recorded: {e}");
        }
    }

    /// A person's input now: its kind, where it landed in logical window
    /// pixels, and the control bound under it in the frame on screen
    /// (`compositor`). Never its content.
    pub fn input(
        &self,
        view: &mut ShowcaseView,
        kind: InputKind,
        at: Option<[f32; 2]>,
        compositor: &Compositor,
    ) {
        let target = at.and_then(|[x, y]| compositor.hit(x, y).cloned());
        self.rec.input(view.now(), kind, at, target);
    }

    /// The path the next save writes.
    pub fn next_path(&self) -> PathBuf {
        let pid = std::process::id();
        self.dir
            .join(format!("showcase-{pid}-{}.monster", self.saves))
    }

    /// Writes the retained window to the next `showcase-<pid>-<n>.monster`
    /// in the directory; the path and what went in it.
    pub fn save(&mut self) -> Result<(PathBuf, Persisted), FileError> {
        private_dir(&self.dir)?;
        let path = self.next_path();
        let written = self.rec.persist(&path)?;
        self.saves += 1;
        Ok((path, written))
    }

    /// On a panic, the retained window goes to the directory
    /// (`record::install_panic_hook`), before the hook installed before runs.
    pub fn catch_panics(&self) -> io::Result<()> {
        private_dir(&self.dir)?;
        record::install_panic_hook(self.rec.clone(), self.dir.clone());
        Ok(())
    }
}

/// The shell's side: the recording starts with the view, and the shell's
/// handlers call these where something happens (see the module doc).
/// Without a recording every hook is an `Option` check.
impl App {
    /// Starts `recording` on the view, when there is one: its
    /// dispatcher's stages, the state now, the panic hook, and where the
    /// files go, in the log.
    pub(crate) fn record(&mut self, recording: Option<Recording>) {
        self.view.set_recording(recording.is_some());
        if let Some(recording) = &recording {
            recording.attach(&mut self.view);
            if let Err(e) = recording.catch_panics() {
                log::error!(
                    "recording: no panic hook, {}: {e}",
                    recording.dir().display()
                );
            }
            log::info!(
                "recording the session: saves go to {} ({SAVE_KEYS}, and on close); a panic \
                 writes session-<unix ms>-<pid>-<n>.monster there",
                recording.next_path().display()
            );
        }
        self.recording = recording;
    }

    /// The state now, when the session is recorded (stored only when it
    /// changed).
    pub(crate) fn record_state(&mut self) {
        if let Some(recording) = &self.recording {
            recording.state(&mut self.view);
        }
    }

    /// A key pressed, when the session is recorded: the save keys ask the
    /// shell's `shell.save_session` (the person's request, through the
    /// dispatcher; the save follows) and go to nobody else (`true`); any
    /// other key is recorded by its kind, never its character (`false`).
    pub(super) fn record_key(&mut self, key: &Key) -> bool {
        if self.recording.is_none() {
            return false;
        }
        if is_save_key(key, self.modifiers) {
            self.view.request_save();
            self.save_requested();
            return true;
        }
        self.record_input(kind_of(key), None, None);
        false
    }

    /// `shell.save_session` ran (the keys, the menu, a screen reader):
    /// the shell, which owns the recording, writes the window.
    pub(crate) fn save_requested(&mut self) {
        if self.view.take_save_request() {
            self.save_session("asked");
        }
    }

    /// A button of the pointer pressed or released at `(x, y)` in window
    /// `id`, when the session is recorded: with the control under it in
    /// that window's frame on screen. A press in the page's own window
    /// is recorded in that window's coordinates (the frames recorded are
    /// the main window's).
    pub(super) fn record_pointer(&mut self, id: WindowId, state: ElementState, x: f32, y: f32) {
        let kind = match state {
            ElementState::Pressed => InputKind::PointerDown,
            ElementState::Released => InputKind::PointerUp,
        };
        self.record_input(kind, Some([x, y]), Some(id));
    }

    fn record_input(&mut self, kind: InputKind, at: Option<[f32; 2]>, in_window: Option<WindowId>) {
        let Some(recording) = &self.recording else {
            return;
        };
        let window = in_window.or(self.main);
        if let Some(host) = window.and_then(|id| self.windows.get(id)) {
            recording.input(&mut self.view, kind, at, host.compositor());
        }
    }

    /// The frame just drawn in the main window, where each bound control
    /// shows in it, and the state it was drawn from, when the session is
    /// recorded.
    pub(super) fn record_frame(&mut self) {
        let Some(recording) = &self.recording else {
            return;
        };
        if let Some(host) = self.main.and_then(|id| self.windows.get(id)) {
            recording.frame(&mut self.view, host.compositor());
        }
    }

    /// Writes the retained window of the recording, and says where.
    pub(crate) fn save_session(&mut self, why: &str) {
        let Some(recording) = self.recording.as_mut() else {
            return;
        };
        match recording.save() {
            Ok((path, written)) => log::info!(
                "session saved ({why}): {} ({} B, {} events, {} chunks)",
                path.display(),
                written.bytes,
                written.events,
                written.chunks
            ),
            Err(e) => log::error!("session not saved ({why}): {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use record::{InputKind, Kind, Limits};
    use winit::keyboard::{Key, ModifiersState, NamedKey};

    use super::{ENV, Recording, is_save_key, kind_of, says};
    use crate::views::ShowcaseView;

    /// The primary modifier of the platform, as the save keys read it.
    const PRIMARY: ModifiersState = if cfg!(target_os = "macos") {
        ModifiersState::SUPER
    } else {
        ModifiersState::CONTROL
    };

    #[test]
    fn only_the_variable_set_to_one_records_and_nothing_is_made_without_it() {
        assert!(!says(None));
        assert!(!says(Some(OsStr::new(""))));
        assert!(!says(Some(OsStr::new("0"))));
        assert!(!says(Some(OsStr::new("true"))));
        assert!(says(Some(OsStr::new("1"))));
        if std::env::var_os(ENV).is_none() {
            assert!(Recording::from_env().is_none());
        }
    }

    #[test]
    fn the_save_keys_are_s_with_shift_and_the_primary_modifier() {
        let s = Key::Character("s".into());
        let upper = Key::Character("S".into());
        assert!(is_save_key(&s, PRIMARY | ModifiersState::SHIFT));
        assert!(is_save_key(&upper, PRIMARY | ModifiersState::SHIFT));
        assert!(!is_save_key(&s, PRIMARY));
        assert!(!is_save_key(&s, ModifiersState::SHIFT));
        assert!(!is_save_key(
            &s,
            ModifiersState::ALT | ModifiersState::SHIFT
        ));
        let t = Key::Character("t".into());
        assert!(!is_save_key(&t, PRIMARY | ModifiersState::SHIFT));
        let enter = Key::Named(NamedKey::Enter);
        assert!(!is_save_key(&enter, PRIMARY | ModifiersState::SHIFT));
    }

    #[test]
    fn a_key_is_recorded_by_its_kind_and_never_its_character() {
        assert_eq!(kind_of(&Key::Character("a".into())), InputKind::Text);
        assert_eq!(kind_of(&Key::Character(" ".into())), InputKind::Text);
        assert_eq!(kind_of(&Key::Named(NamedKey::Enter)), InputKind::Key);
        assert_eq!(kind_of(&Key::Named(NamedKey::Escape)), InputKind::Key);
        assert_eq!(kind_of(&Key::Named(NamedKey::Tab)), InputKind::Key);
    }

    /// The recording keeps the state, the frames and the inputs of the
    /// Silences page on the gallery's clock, and names its saves after
    /// the process; a key's character is not in it.
    #[test]
    fn the_recording_keeps_the_state_a_frame_and_an_input_and_names_its_saves() {
        let dir = tempfile::TempDir::new().expect("a temp dir");
        let mut recording = Recording::new(dir.path().join("sessions"), Limits::default());
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.pin_sidebar(true);
        view.jump_to_page("silences");
        view.resize(1200.0, 800.0, 1.0);
        recording.attach(&mut view);
        let mut c = engine::drawing::compositor::Compositor::new();
        view.render(&mut c);
        recording.frame(&mut view, &c);
        let [x, y, w, h] = c
            .binding(&"silences.apply_all".parse().unwrap())
            .and_then(|b| b.visible_rect())
            .expect("the button shows");
        let at = [x + w / 2.0, y + h / 2.0];
        recording.input(&mut view, InputKind::PointerDown, Some(at), &c);
        recording.input(&mut view, InputKind::Text, None, &c);

        let pid = std::process::id();
        let expected = dir
            .path()
            .join("sessions")
            .join(format!("showcase-{pid}-0.monster"));
        assert_eq!(recording.next_path(), expected);
        let (path, written) = recording.save().expect("the window is saved");
        assert_eq!(path, expected);
        assert!(written.bindings_bytes > 0, "{written:?}");
        assert_eq!(
            recording.next_path(),
            dir.path()
                .join("sessions")
                .join(format!("showcase-{pid}-1.monster"))
        );

        let session = record::Session::open(&path).expect("the session opens");
        let kinds: Vec<&str> = session.events().iter().map(|e| e.kind.name()).collect();
        assert_eq!(kinds, ["snapshot", "input", "input"], "{kinds:?}");
        let inputs: Vec<(InputKind, Option<String>)> = session
            .events()
            .iter()
            .filter_map(|e| match &e.kind {
                Kind::Input { kind, target, .. } => {
                    Some((*kind, target.as_ref().map(ToString::to_string)))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            inputs,
            [
                (
                    InputKind::PointerDown,
                    Some("silences.apply_all".to_owned())
                ),
                (InputKind::Text, None)
            ]
        );
        assert_eq!(session.frame_times().len(), 1);
        let t = session.frame_times()[0];
        assert_eq!(
            session.hit_at(t, at[0], at[1]),
            Some("silences.apply_all".parse().unwrap())
        );
    }
}
