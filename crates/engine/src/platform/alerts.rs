//! Native alerts and confirmations: a message with a level and one
//! button, or a question with two, behind a trait so an app shows the
//! platform's panel and a test never opens one. [`NativeAlerts`] wraps
//! `rfd::MessageDialog` on the desktop (macOS, windows, linux);
//! [`FakeAlerts`] answers what a test loaded, in order, and keeps what
//! was shown; [`NoAlerts`] is the wasm and mobile answer, `Unsupported`.
//! The same shape as `files.rs`: one trait, the real one under the
//! desktop cfg, the fake everywhere, [`default_alerts`] choosing.
//!
//! Every call is synchronous and modal, as `rfd::MessageDialog::show`
//! is: it blocks the thread that asked until the person answers. With a
//! [`Parent`] (the shell's `Arc<Window>`) the panel opens as that
//! window's sheet on macOS; without one it opens on its own.
//!
//! A native panel is a person's: an app that offers one as a capability
//! declares it for a person alone (`Actors { agent: false, .. }`, as the
//! Files page's `open_file`), and a confirmation the platform asks is
//! answered into the dispatcher by `confirm` or `decline` as
//! `Actor::Human`, the same contract the drawn modal answers.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// How grave the message is; the platform picks the icon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Level {
    #[default]
    Info,
    Warning,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Level::Info => "info",
            Level::Warning => "warning",
            Level::Error => "error",
        })
    }
}

/// A message with one button: the person reads it and closes it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Alert {
    pub level: Level,
    pub title: String,
    pub body: String,
}

impl Alert {
    pub fn new(level: Level, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            level,
            title: title.into(),
            body: body.into(),
        }
    }

    pub fn info(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(Level::Info, title, body)
    }

    pub fn warning(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(Level::Warning, title, body)
    }

    pub fn error(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(Level::Error, title, body)
    }
}

/// A question with two buttons: the one that confirms (its label says
/// what will happen: "Discard", "Allow") and the one that cancels.
/// `destructive` says the confirmation cannot be undone; the platform
/// shows it as a warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Confirm {
    pub title: String,
    pub body: String,
    pub confirm_label: String,
    pub cancel_label: String,
    pub destructive: bool,
}

impl Confirm {
    /// A question answered by "OK" or "Cancel", not destructive.
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            confirm_label: "OK".to_owned(),
            cancel_label: "Cancel".to_owned(),
            destructive: false,
        }
    }

    pub fn labels(mut self, confirm: impl Into<String>, cancel: impl Into<String>) -> Self {
        self.confirm_label = confirm.into();
        self.cancel_label = cancel.into();
        self
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }
}

/// What the person answered a confirmation with. Closing the panel any
/// other way (Escape) declines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answered {
    Confirmed,
    Declined,
}

/// Which panel was asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertKind {
    Alert,
    Confirm,
}

impl fmt::Display for AlertKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            AlertKind::Alert => "alert",
            AlertKind::Confirm => "confirm",
        })
    }
}

/// Why a panel gave no answer at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlertError {
    /// This target has no native panel (wasm, android, ios).
    Unsupported(AlertKind),
}

impl fmt::Display for AlertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(kind) => write!(f, "{kind}: no native dialog on this target"),
        }
    }
}

impl std::error::Error for AlertError {}

/// What a panel answers.
pub type Shown<T> = Result<T, AlertError>;

/// The window a panel opens over: anything with a window and a display
/// handle, the shell's `Arc<winit::window::Window>` first of all. The
/// trait exists so the parent travels as one `&dyn Parent`; nothing
/// implements it by hand.
pub trait Parent: HasWindowHandle + HasDisplayHandle {}

impl<W: HasWindowHandle + HasDisplayHandle> Parent for W {}

/// The two panels an app asks for. Each one blocks until the person
/// answers, on the thread that asked; `parent` makes it that window's.
pub trait Alerts {
    fn alert(&mut self, alert: &Alert, parent: Option<&dyn Parent>) -> Shown<()>;
    fn confirm(&mut self, ask: &Confirm, parent: Option<&dyn Parent>) -> Shown<Answered>;
}

// -- the fake -------------------------------------------------------------

/// What the fake answers a call with, in the order loaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Confirmed,
    Declined,
    Unsupported,
}

/// What one call asked the fake to show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dialog {
    Alert(Alert),
    Confirm(Confirm),
}

/// One call the fake took, and whether a parent window came with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asked {
    pub dialog: Dialog,
    pub with_parent: bool,
}

impl Asked {
    pub fn kind(&self) -> AlertKind {
        match self.dialog {
            Dialog::Alert(_) => AlertKind::Alert,
            Dialog::Confirm(_) => AlertKind::Confirm,
        }
    }
}

/// The panels of a test: answers what was loaded, in order, and keeps
/// everything shown. With nothing loaded an alert is shown and a
/// confirmation is declined, so a page under test that asks unexpectedly
/// runs nothing.
#[derive(Debug, Default)]
pub struct FakeAlerts {
    answers: VecDeque<Answer>,
    asked: Vec<Asked>,
}

impl FakeAlerts {
    pub fn new() -> Self {
        Self::default()
    }

    /// The next call is confirmed (an alert closed).
    pub fn will_confirm(mut self) -> Self {
        self.answers.push_back(Answer::Confirmed);
        self
    }

    /// The next call is declined (an alert closed).
    pub fn will_decline(mut self) -> Self {
        self.answers.push_back(Answer::Declined);
        self
    }

    /// The next call answers `Unsupported`, as the wasm and mobile
    /// panels do.
    pub fn will_refuse(mut self) -> Self {
        self.answers.push_back(Answer::Unsupported);
        self
    }

    /// Every call taken so far, in order.
    pub fn asked(&self) -> &[Asked] {
        &self.asked
    }

    /// Every call taken so far, leaving none.
    pub fn take_asked(&mut self) -> Vec<Asked> {
        std::mem::take(&mut self.asked)
    }

    /// Answers left to give.
    pub fn pending(&self) -> usize {
        self.answers.len()
    }

    fn answer(&mut self, dialog: Dialog, parent: Option<&dyn Parent>) -> Option<Answer> {
        self.asked.push(Asked {
            dialog,
            with_parent: parent.is_some(),
        });
        self.answers.pop_front()
    }
}

impl Alerts for FakeAlerts {
    fn alert(&mut self, alert: &Alert, parent: Option<&dyn Parent>) -> Shown<()> {
        match self.answer(Dialog::Alert(alert.clone()), parent) {
            Some(Answer::Unsupported) => Err(AlertError::Unsupported(AlertKind::Alert)),
            _ => Ok(()),
        }
    }

    fn confirm(&mut self, ask: &Confirm, parent: Option<&dyn Parent>) -> Shown<Answered> {
        match self.answer(Dialog::Confirm(ask.clone()), parent) {
            Some(Answer::Confirmed) => Ok(Answered::Confirmed),
            Some(Answer::Unsupported) => Err(AlertError::Unsupported(AlertKind::Confirm)),
            Some(Answer::Declined) | None => Ok(Answered::Declined),
        }
    }
}

// -- no panel at all -------------------------------------------------------

/// The panels of a target without any: wasm (the browser's `alert` and
/// `confirm` are another capability, and `rfd` is not built there) and
/// mobile (`rfd` has no backend). Every call is `Unsupported`, and the
/// app says so where it would have opened one, or asks in its own way.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoAlerts;

impl Alerts for NoAlerts {
    fn alert(&mut self, _alert: &Alert, _parent: Option<&dyn Parent>) -> Shown<()> {
        Err(AlertError::Unsupported(AlertKind::Alert))
    }

    fn confirm(&mut self, _ask: &Confirm, _parent: Option<&dyn Parent>) -> Shown<Answered> {
        Err(AlertError::Unsupported(AlertKind::Confirm))
    }
}

/// Panels shared by handle: a test keeps one `Arc` and gives the page the
/// other as its `Box<dyn Alerts>`, then reads what the page showed
/// ([`FakeAlerts::asked`]). A poisoned lock still answers: the panels
/// hold no invariant a panic elsewhere could break.
impl<A: Alerts> Alerts for Arc<Mutex<A>> {
    fn alert(&mut self, alert: &Alert, parent: Option<&dyn Parent>) -> Shown<()> {
        locked(self).alert(alert, parent)
    }

    fn confirm(&mut self, ask: &Confirm, parent: Option<&dyn Parent>) -> Shown<Answered> {
        locked(self).confirm(ask, parent)
    }
}

fn locked<A>(shared: &Mutex<A>) -> MutexGuard<'_, A> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

// -- the native one ----------------------------------------------------------

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use winit::raw_window_handle::{DisplayHandle, HandleError, WindowHandle};

/// A parent as `rfd` takes it: `set_parent` wants a sized type with both
/// handles, and a `&dyn Parent` has them, one call away.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
struct ParentRef<'a>(&'a dyn Parent);

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl HasWindowHandle for ParentRef<'_> {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.0.window_handle()
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl HasDisplayHandle for ParentRef<'_> {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.0.display_handle()
    }
}

/// The platform's panels through `rfd::MessageDialog`: an `NSAlert` on
/// macOS (a sheet of the parent window when one is given), the task
/// dialog on windows, gtk or zenity on linux (the backend there is the
/// linux bring-up's choice; the engine turns `rfd`'s default features
/// off). Modal on the calling thread, which must be the main one.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeAlerts;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl NativeAlerts {
    pub fn new() -> Self {
        Self
    }

    fn level(level: Level) -> rfd::MessageLevel {
        match level {
            Level::Info => rfd::MessageLevel::Info,
            Level::Warning => rfd::MessageLevel::Warning,
            Level::Error => rfd::MessageLevel::Error,
        }
    }

    /// The `rfd` builder for an alert: one button. Building opens
    /// nothing; only `show` does.
    fn alert_dialog(alert: &Alert, parent: Option<&dyn Parent>) -> rfd::MessageDialog {
        Self::dialog(
            Self::level(alert.level),
            &alert.title,
            &alert.body,
            rfd::MessageButtons::Ok,
            parent,
        )
    }

    /// The `rfd` builder for a confirmation: the two labels as the
    /// buttons, a warning icon when it cannot be undone.
    fn confirm_dialog(ask: &Confirm, parent: Option<&dyn Parent>) -> rfd::MessageDialog {
        let level = if ask.destructive {
            rfd::MessageLevel::Warning
        } else {
            rfd::MessageLevel::Info
        };
        let buttons = rfd::MessageButtons::OkCancelCustom(
            ask.confirm_label.clone(),
            ask.cancel_label.clone(),
        );
        Self::dialog(level, &ask.title, &ask.body, buttons, parent)
    }

    fn dialog(
        level: rfd::MessageLevel,
        title: &str,
        body: &str,
        buttons: rfd::MessageButtons,
        parent: Option<&dyn Parent>,
    ) -> rfd::MessageDialog {
        let dialog = rfd::MessageDialog::new()
            .set_level(level)
            .set_title(title)
            .set_description(body)
            .set_buttons(buttons);
        match parent {
            Some(parent) => dialog.set_parent(&ParentRef(parent)),
            None => dialog,
        }
    }

    /// What the platform's result means for `ask`: the confirm button
    /// confirms, anything else (the cancel button, Escape) declines.
    fn answered(ask: &Confirm, result: rfd::MessageDialogResult) -> Answered {
        match result {
            rfd::MessageDialogResult::Custom(label) if label == ask.confirm_label => {
                Answered::Confirmed
            }
            rfd::MessageDialogResult::Ok | rfd::MessageDialogResult::Yes => Answered::Confirmed,
            _ => Answered::Declined,
        }
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl Alerts for NativeAlerts {
    fn alert(&mut self, alert: &Alert, parent: Option<&dyn Parent>) -> Shown<()> {
        let _closed = Self::alert_dialog(alert, parent).show();
        Ok(())
    }

    fn confirm(&mut self, ask: &Confirm, parent: Option<&dyn Parent>) -> Shown<Answered> {
        Ok(Self::answered(
            ask,
            Self::confirm_dialog(ask, parent).show(),
        ))
    }
}

/// The panels of the current target: native on the desktop, none on
/// wasm and mobile.
pub fn default_alerts() -> Box<dyn Alerts> {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        Box::new(NativeAlerts::new())
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    {
        Box::new(NoAlerts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk_full() -> Alert {
        Alert::error(
            "Disk full",
            "The export stopped: no space left on /Volumes/work.",
        )
    }

    fn discard() -> Confirm {
        Confirm::new("Discard changes?", "discard all changes to b.txt.")
            .labels("Discard", "Cancel")
            .destructive(true)
    }

    #[test]
    fn the_fake_answers_in_order_and_keeps_what_was_shown() {
        let mut alerts = FakeAlerts::new()
            .will_confirm()
            .will_decline()
            .will_confirm();
        assert_eq!(alerts.pending(), 3);

        alerts.alert(&disk_full(), None).unwrap();
        assert_eq!(
            alerts.confirm(&discard(), None).unwrap(),
            Answered::Declined
        );
        assert_eq!(
            alerts
                .confirm(&Confirm::new("Allow stage?", ""), None)
                .unwrap(),
            Answered::Confirmed
        );
        assert_eq!(alerts.pending(), 0);

        let asked = alerts.take_asked();
        assert_eq!(asked.len(), 3);
        assert_eq!(asked[0].kind(), AlertKind::Alert);
        assert_eq!(asked[0].dialog, Dialog::Alert(disk_full()));
        assert!(!asked[0].with_parent);
        assert_eq!(asked[1].kind(), AlertKind::Confirm);
        let Dialog::Confirm(ask) = &asked[1].dialog else {
            panic!("a confirmation");
        };
        assert_eq!(ask.title, "Discard changes?");
        assert_eq!(ask.confirm_label, "Discard");
        assert_eq!(ask.cancel_label, "Cancel");
        assert!(ask.destructive);
        assert_eq!(asked[2].kind(), AlertKind::Confirm);
        assert!(alerts.asked().is_empty(), "taken");
    }

    #[test]
    fn the_fake_with_nothing_loaded_shows_an_alert_and_declines_a_confirmation() {
        let mut alerts = FakeAlerts::new();
        assert_eq!(alerts.alert(&disk_full(), None), Ok(()));
        assert_eq!(alerts.confirm(&discard(), None), Ok(Answered::Declined));
        assert_eq!(alerts.asked().len(), 2);
    }

    #[test]
    fn the_fake_refuses_as_a_target_without_panels_does() {
        let mut alerts = FakeAlerts::new().will_refuse().will_refuse();
        let err = alerts.alert(&disk_full(), None).unwrap_err();
        assert_eq!(err, AlertError::Unsupported(AlertKind::Alert));
        assert_eq!(err.to_string(), "alert: no native dialog on this target");
        let err = alerts.confirm(&discard(), None).unwrap_err();
        assert_eq!(err, AlertError::Unsupported(AlertKind::Confirm));
        assert_eq!(err.to_string(), "confirm: no native dialog on this target");
    }

    #[test]
    fn a_shared_fake_answers_through_one_handle_and_reports_through_the_other() {
        let fake = Arc::new(Mutex::new(FakeAlerts::new().will_confirm()));
        let mut page: Box<dyn Alerts> = Box::new(fake.clone());
        assert_eq!(page.confirm(&discard(), None).unwrap(), Answered::Confirmed);
        assert_eq!(page.alert(&disk_full(), None), Ok(()));
        let asked = locked(&fake).take_asked();
        assert_eq!(asked.len(), 2);
        assert_eq!(asked[0].kind(), AlertKind::Confirm);
        assert_eq!(asked[1].kind(), AlertKind::Alert);
    }

    #[test]
    fn no_alerts_is_unsupported_for_both_panels() {
        let mut none = NoAlerts;
        assert_eq!(
            none.alert(&disk_full(), None),
            Err(AlertError::Unsupported(AlertKind::Alert))
        );
        assert_eq!(
            none.confirm(&discard(), None),
            Err(AlertError::Unsupported(AlertKind::Confirm))
        );
    }

    #[test]
    fn the_builders_and_the_defaults() {
        let plain = Confirm::new("Allow stage?", "test-agent asks to stage b.txt.");
        assert_eq!(plain.confirm_label, "OK");
        assert_eq!(plain.cancel_label, "Cancel");
        assert!(!plain.destructive);
        assert_eq!(Alert::default().level, Level::Info);
        assert_eq!(Alert::warning("t", "b").level, Level::Warning);
        assert_eq!(Level::Error.to_string(), "error");
    }

    /// The shell's window is a parent as it stands: `Arc<Window>` has
    /// both handles (`raw-window-handle` 0.6 implements them for `Arc`),
    /// so it goes in as `&dyn Parent` with no wrapper. Checked at compile
    /// time; no window is made.
    #[test]
    fn the_hosts_window_is_a_parent() {
        fn accepts(_: &dyn Parent) {}
        fn as_parent(window: &Arc<winit::window::Window>) {
            accepts(window);
        }
        let _ = as_parent;
    }

    /// Building the `rfd` dialog opens nothing; its debug form shows the
    /// request went through whole. (The panel itself is opened only by a
    /// person, never by a test.)
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    #[test]
    fn the_native_builders_carry_the_request() {
        let built = format!("{:?}", NativeAlerts::alert_dialog(&disk_full(), None));
        assert!(built.contains("Disk full"), "{built}");
        assert!(built.contains("no space left"), "{built}");
        assert!(built.contains("Error"), "{built}");
        assert!(built.contains("buttons: Ok"), "{built}");
        assert!(built.contains("parent: None"), "{built}");

        let built = format!("{:?}", NativeAlerts::confirm_dialog(&discard(), None));
        assert!(built.contains("Discard changes?"), "{built}");
        assert!(
            built.contains("Warning"),
            "destructive is a warning: {built}"
        );
        assert!(
            built.contains("OkCancelCustom(\"Discard\", \"Cancel\")"),
            "{built}"
        );
        let built = format!(
            "{:?}",
            NativeAlerts::confirm_dialog(&Confirm::new("Allow?", ""), None)
        );
        assert!(built.contains("Info"), "{built}");
    }

    /// The platform answers by the button's label: the confirm label
    /// confirms, the cancel label and an Escape (`Cancel`) decline.
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    #[test]
    fn the_native_answer_is_read_by_the_button_pressed() {
        let ask = discard();
        let custom = |s: &str| rfd::MessageDialogResult::Custom(s.to_owned());
        assert_eq!(
            NativeAlerts::answered(&ask, custom("Discard")),
            Answered::Confirmed
        );
        assert_eq!(
            NativeAlerts::answered(&ask, custom("Cancel")),
            Answered::Declined
        );
        assert_eq!(
            NativeAlerts::answered(&ask, rfd::MessageDialogResult::Cancel),
            Answered::Declined
        );
        assert_eq!(
            NativeAlerts::answered(&ask, rfd::MessageDialogResult::Ok),
            Answered::Confirmed
        );
    }
}
