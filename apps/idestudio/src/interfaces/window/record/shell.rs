//! The shell's side of the session recording (`ide::record`, on with
//! `PLEV_RECORD=1` in a debug build): the recording starts with the
//! workbench, its save keys join the keymap, and the shell's handlers
//! call these hooks where something happens (see the shell's module doc).
//! Without a recording every hook is an `Option` check.

use ide::record::{self, Recording};
use ide::workbench::Workbench;
use rec::InputKind;

use crate::App;

/// Starts `recording` on `bench`: its dispatcher's stages, the state now,
/// the panic hook, and where the files go, in the log.
pub(crate) fn start(recording: &Recording, bench: &mut Workbench) {
    recording.attach(bench);
    if let Err(e) = recording.catch_panics() {
        log::error!(
            "recording: no panic hook, {}: {e}",
            recording.dir().display()
        );
    }
    log::info!(
        "recording the session: saves go to {} ({} or {}, and on close); a panic writes session-<unix ms>-<pid>-<n>.monster there",
        recording.next_path().display(),
        record::SAVE_KEYS[0],
        record::SAVE_KEYS[1]
    );
}

impl App {
    /// Keeps `recording`, and binds its save keys when there is one.
    pub(crate) fn record(&mut self, recording: Option<Recording>) {
        if recording.is_some()
            && let Err(e) = record::bind_keys(self.matcher.keymap_mut(), &mut self.registry)
        {
            log::error!("recording: the save keys do not bind: {e}");
        }
        self.recording = recording;
    }

    /// The state now, when the session is recorded (stored only when it
    /// changed).
    pub(crate) fn record_state(&self) {
        if let Some(recording) = &self.recording {
            recording.state(&self.bench);
        }
    }

    /// A person's input, when the session is recorded: its kind, where it
    /// landed, and the control under it in the frame on screen.
    pub(crate) fn record_input(&self, kind: InputKind, at: Option<[f32; 2]>) {
        if let Some(recording) = &self.recording {
            recording.input(&self.bench, kind, at, &self.compositor);
        }
    }

    /// A person's input changed something: the state as it left it, and a
    /// frame.
    pub(crate) fn changed(&mut self) {
        self.record_state();
        self.invalidate();
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
