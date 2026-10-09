//! System notifications as capabilities out of the window (task-76 item
//! 5): a [`Notification`] is a title, a body, a level (the `Level` of
//! `alerts`) and at most one [`Action`], which names a capability of the
//! app's projection (a `SemId` and an action, with a label). The platform
//! shows it without blocking and without an answer; a click on the
//! notification or on its button comes back later as a [`Clicked`] (the
//! menu's, item 4) and becomes the same `Request::observe` by
//! `Actor::Human` the button on screen makes, through the app's one
//! dispatcher. So the click is refused where the button would be: a
//! capability that left the projection meanwhile makes no request at all.
//!
//! Who notifies: the app, when something happened while nobody was
//! looking (files dropped on a window behind another, a job that ended),
//! and a person through a capability declared for a person alone. Never an
//! agent: it reads the text the app projected beside the notification.
//!
//! The same shape as `alerts.rs`: one trait ([`Notifier`]), the real one
//! under the desktop cfg ([`NativeNotifier`]), [`FakeNotifier`] everywhere
//! for the tests (it keeps what was sent and takes injected clicks),
//! [`NoNotifier`] on wasm and mobile, [`default_notifier`] choosing.
//!
//! What the real one does today: on macOS the UserNotifications framework
//! (`UNUserNotificationCenter`) refuses a process that runs outside an app
//! bundle (`currentNotificationCenter` wants the main bundle's proxy and
//! raises without one), and every `cargo run` of this repository is such a
//! process; the bundle is the platform bring-up's, and the framework path
//! (the crate `objc2-user-notifications`, a delegate for the response, a
//! category for the button) goes in when it exists. Until then
//! [`NativeNotifier`] answers [`NotifyError::NeedsBundle`], the app logs
//! it and the projected text stays the truth. Windows and linux have no
//! crate here: `Unsupported`.

use std::collections::VecDeque;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};

use accord::{ActionName, SemId, Snapshot, Value};

pub use super::alerts::Level;
use super::menu::{Clicked, MenuItem, locked};

/// The one button of a notification: a capability of the projection, as
/// the notification names it. A click on it, or on the notification
/// itself, asks the dispatcher for `action` of `sem` with `param`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub label: String,
    pub sem: SemId,
    pub action: ActionName,
    pub param: Option<Value>,
}

impl Action {
    /// `clicked`, labelled `label`.
    pub fn new(label: impl Into<String>, clicked: Clicked) -> Action {
        Action {
            label: label.into(),
            sem: clicked.sem,
            action: clicked.action,
            param: clicked.param,
        }
    }

    /// `action` of the node `sem` in `snapshot`, labelled by the action or
    /// the node, when a person may ask for it and it is enabled now;
    /// `None` otherwise: a button the person could not press is no button
    /// of a notification either.
    pub fn of(snapshot: &Snapshot, sem: &SemId, action: &str) -> Option<Action> {
        let item = MenuItem::of(snapshot, sem, action)?;
        let node = snapshot.get(sem)?;
        let decl = node.actions.iter().find(|a| a.name == item.action)?;
        if !decl.policy.actors.human || !item.enabled {
            return None;
        }
        Some(Action {
            label: item.label,
            sem: item.sem,
            action: item.action,
            param: item.param,
        })
    }

    pub fn label(mut self, label: impl Into<String>) -> Action {
        self.label = label.into();
        self
    }

    /// What a click asks the dispatcher for.
    pub fn clicked(&self) -> Clicked {
        Clicked {
            sem: self.sem.clone(),
            action: self.action.clone(),
            param: self.param.clone(),
        }
    }
}

/// What the platform shows: a title, a body, how grave it is, and the one
/// button, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub level: Level,
    pub title: String,
    pub body: String,
    pub action: Option<Action>,
}

impl Notification {
    pub fn new(level: Level, title: impl Into<String>, body: impl Into<String>) -> Notification {
        Notification {
            level,
            title: title.into(),
            body: body.into(),
            action: None,
        }
    }

    pub fn info(title: impl Into<String>, body: impl Into<String>) -> Notification {
        Notification::new(Level::Info, title, body)
    }

    pub fn warning(title: impl Into<String>, body: impl Into<String>) -> Notification {
        Notification::new(Level::Warning, title, body)
    }

    pub fn error(title: impl Into<String>, body: impl Into<String>) -> Notification {
        Notification::new(Level::Error, title, body)
    }

    pub fn action(mut self, action: Action) -> Notification {
        self.action = Some(action);
        self
    }
}

/// Names a notification shown, for the click that comes back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NotificationId(pub u64);

impl fmt::Display for NotificationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "notification-{}", self.0)
    }
}

/// Why a notification was not shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotifyError {
    /// This target has no system notification (wasm, mobile, and the
    /// desktops without a crate here).
    Unsupported,
    /// macOS: the notification center wants an app bundle, and this
    /// process runs outside one.
    NeedsBundle,
    /// The platform refused, in its words.
    Platform(String),
}

impl fmt::Display for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotifyError::Unsupported => {
                f.write_str("notification: no system notification on this target")
            }
            NotifyError::NeedsBundle => f.write_str(
                "notification: the system's notification center wants an app bundle, and this \
                 binary runs outside one",
            ),
            NotifyError::Platform(why) => {
                write!(f, "the platform refused the notification: {why}")
            }
        }
    }
}

impl std::error::Error for NotifyError {}

/// What a notification comes to at once: shown under an id, or not.
pub type Shown<T> = Result<T, NotifyError>;

/// The platform's notifications. `notify` shows one and returns at once
/// with its id; `poll` hands back the clicks since the last poll, in
/// order (a click on a notification or on its button is the `Clicked` of
/// its action; one without an action gives none), for the app to dispatch
/// on its loop, before the frame.
pub trait Notifier {
    fn notify(&mut self, notification: &Notification) -> Shown<NotificationId>;
    fn poll(&mut self) -> Vec<Clicked>;
}

// -- the fake ---------------------------------------------------------------

/// One notification the fake showed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sent {
    pub id: NotificationId,
    pub notification: Notification,
}

/// The notifications of a test: keeps everything sent, in order, refuses
/// what a test asked it to refuse, and takes injected clicks
/// ([`FakeNotifier::click`]), which `poll` hands back as the platform
/// would.
#[derive(Debug, Default)]
pub struct FakeNotifier {
    next: u64,
    sent: Vec<Sent>,
    refusals: VecDeque<NotifyError>,
    clicks: VecDeque<Clicked>,
}

impl FakeNotifier {
    pub fn new() -> FakeNotifier {
        FakeNotifier::default()
    }

    /// The next `notify` answers `Unsupported`, as wasm and mobile do,
    /// and shows nothing.
    pub fn will_refuse(mut self) -> FakeNotifier {
        self.refusals.push_back(NotifyError::Unsupported);
        self
    }

    /// The next `notify` answers `NeedsBundle`, as macOS does today.
    pub fn will_want_a_bundle(mut self) -> FakeNotifier {
        self.refusals.push_back(NotifyError::NeedsBundle);
        self
    }

    /// Everything shown so far, in order.
    pub fn sent(&self) -> &[Sent] {
        &self.sent
    }

    /// Everything shown so far, leaving none.
    pub fn take_sent(&mut self) -> Vec<Sent> {
        std::mem::take(&mut self.sent)
    }

    /// A person clicks notification `id`, or its button: its action is
    /// queued for `poll`. `false`, and nothing queued, when no such
    /// notification was shown or it carries no action.
    pub fn click(&mut self, id: NotificationId) -> bool {
        let Some(action) = self
            .sent
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| s.notification.action.as_ref())
        else {
            return false;
        };
        self.clicks.push_back(action.clicked());
        true
    }

    /// Clicks taken and not polled yet.
    pub fn pending(&self) -> usize {
        self.clicks.len()
    }
}

impl Notifier for FakeNotifier {
    fn notify(&mut self, notification: &Notification) -> Shown<NotificationId> {
        if let Some(refusal) = self.refusals.pop_front() {
            return Err(refusal);
        }
        self.next += 1;
        let id = NotificationId(self.next);
        self.sent.push(Sent {
            id,
            notification: notification.clone(),
        });
        Ok(id)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        self.clicks.drain(..).collect()
    }
}

// -- no notification at all -------------------------------------------------

/// The notifications of a target without any: wasm (the browser's
/// Notification API is another capability, with its own permission) and
/// mobile. Every `notify` is `Unsupported`, nothing is ever clicked; the
/// app's projected text is what an agent and a person read.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoNotifier;

impl Notifier for NoNotifier {
    fn notify(&mut self, _notification: &Notification) -> Shown<NotificationId> {
        Err(NotifyError::Unsupported)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        Vec::new()
    }
}

/// Notifications shared by handle: the shell keeps one `Arc` (it polls the
/// clicks on its loop) and gives a page the other as its
/// `Box<dyn Notifier>` (it notifies from `execute`); a test keeps one and
/// reads what was sent. A poisoned lock still answers: the notifier holds
/// no invariant a panic elsewhere could break.
impl<N: Notifier> Notifier for Arc<Mutex<N>> {
    fn notify(&mut self, notification: &Notification) -> Shown<NotificationId> {
        locked(self).notify(notification)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        locked(self).poll()
    }
}

/// A boxed notifier is one too, so `Arc<Mutex<Box<dyn Notifier>>>` shares
/// whatever `default_notifier` chose.
impl<N: Notifier + ?Sized> Notifier for Box<N> {
    fn notify(&mut self, notification: &Notification) -> Shown<NotificationId> {
        (**self).notify(notification)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        (**self).poll()
    }
}

// -- the native one ----------------------------------------------------------

/// Whether `exe` runs inside an app bundle: `<Name>.app/Contents/MacOS/exe`.
/// The notification center finds the bundle from the executable's place
/// the same way; a `cargo run` binary under `target/` is outside one.
pub fn bundled(exe: &Path) -> bool {
    let Some(macos) = exe.parent() else {
        return false;
    };
    let Some(contents) = macos.parent() else {
        return false;
    };
    macos.file_name().is_some_and(|n| n == "MacOS")
        && contents.file_name().is_some_and(|n| n == "Contents")
        && contents
            .parent()
            .and_then(Path::extension)
            .is_some_and(|ext| ext == "app")
}

/// The platform's notifications on the desktop. macOS: the
/// UserNotifications framework wants an app bundle, so a process outside
/// one is told [`NotifyError::NeedsBundle`] and a bundled one
/// [`NotifyError::Unsupported`] until the framework path is written
/// against a bundle that exists (the platform bring-up's). Windows and
/// linux: no crate here, `Unsupported`.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeNotifier;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl NativeNotifier {
    pub fn new() -> NativeNotifier {
        NativeNotifier
    }

    /// Why this process cannot notify yet.
    fn refusal() -> NotifyError {
        if cfg!(target_os = "macos") {
            let in_bundle = std::env::current_exe().is_ok_and(|exe| bundled(&exe));
            if !in_bundle {
                return NotifyError::NeedsBundle;
            }
        }
        NotifyError::Unsupported
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl Notifier for NativeNotifier {
    fn notify(&mut self, notification: &Notification) -> Shown<NotificationId> {
        let why = Self::refusal();
        log::info!("notification not shown ({why}): {}", notification.title);
        Err(why)
    }

    fn poll(&mut self) -> Vec<Clicked> {
        Vec::new()
    }
}

/// The notifications of the current target, woken by `wake` on every
/// click (from the platform's thread; a shell sends itself an event
/// through its loop proxy and polls on the loop): native on the desktop,
/// none on wasm and mobile.
pub fn default_notifier(wake: impl Fn() + Send + Sync + 'static) -> Box<dyn Notifier> {
    let _ = wake;
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        Box::new(NativeNotifier::new())
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    {
        Box::new(NoNotifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accord::{Policy, Role, SnapshotBuilder};

    fn sid(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    /// An inbox: a button a person may press, one for agents alone, and
    /// one disabled.
    fn snapshot() -> Snapshot {
        let mut b = SnapshotBuilder::new();
        b.node("inbox", Role::Region, "inbox").state("unread", 2i64);
        b.node("inbox.open", Role::Button, "open the inbox")
            .parent("inbox")
            .action("open")
            .policy(Policy {
                actors: accord::Actors {
                    agent: false,
                    ..accord::Actors::default()
                },
                ..Policy::default()
            });
        b.node("inbox.archive", Role::Button, "archive")
            .parent("inbox")
            .action("archive")
            .policy(Policy {
                actors: accord::Actors {
                    human: false,
                    ..accord::Actors::default()
                },
                ..Policy::default()
            });
        b.node("inbox.reply", Role::Button, "reply")
            .parent("inbox")
            .enabled(false)
            .action("reply");
        b.finish().unwrap()
    }

    fn mail() -> Notification {
        let snap = snapshot();
        Notification::info("Inbox", "2 new messages")
            .action(Action::of(&snap, &sid("inbox.open"), "open").unwrap())
    }

    #[test]
    fn an_action_is_a_capability_a_person_may_ask_for_now() {
        let snap = snapshot();
        let open = Action::of(&snap, &sid("inbox.open"), "open").unwrap();
        assert_eq!(open.label, "open the inbox");
        assert_eq!(open.sem, sid("inbox.open"));
        assert_eq!(open.action.as_str(), "open");
        assert_eq!(open.param, None);
        assert_eq!(
            open.clicked(),
            Clicked {
                sem: sid("inbox.open"),
                action: ActionName::parse("open").unwrap(),
                param: None,
            }
        );
        assert_eq!(open.clone().label("open").label, "open");
        assert!(
            Action::of(&snap, &sid("inbox.archive"), "archive").is_none(),
            "an agent's capability is no button of a person's notification"
        );
        assert!(
            Action::of(&snap, &sid("inbox.reply"), "reply").is_none(),
            "disabled"
        );
        assert!(Action::of(&snap, &sid("inbox.open"), "close").is_none());
        assert!(Action::of(&snap, &sid("nobody"), "open").is_none());
        let by_hand = Action::new("open", open.clicked());
        assert_eq!(by_hand.sem, open.sem);
        assert_eq!(by_hand.label, "open");
    }

    #[test]
    fn the_fake_keeps_what_was_sent_and_a_click_is_the_actions() {
        let mut fake = FakeNotifier::new();
        let first = fake.notify(&mail()).unwrap();
        let second = fake
            .notify(&Notification::warning("Disk", "nearly full"))
            .unwrap();
        assert_ne!(first, second);
        assert_eq!(first.to_string(), "notification-1");
        assert_eq!(fake.sent().len(), 2);
        assert_eq!(fake.sent()[0].notification, mail());
        assert_eq!(fake.sent()[1].notification.level, Level::Warning);
        assert!(fake.sent()[1].notification.action.is_none());

        assert!(fake.click(first));
        assert!(!fake.click(second), "no action, no click to hand back");
        assert!(!fake.click(NotificationId(99)), "never shown");
        assert_eq!(fake.pending(), 1);
        let clicks = fake.poll();
        assert_eq!(clicks, [mail().action.unwrap().clicked()]);
        assert!(fake.poll().is_empty());

        let taken = fake.take_sent();
        assert_eq!(taken.len(), 2);
        assert!(fake.sent().is_empty());
        assert!(!fake.click(first), "taken");
    }

    #[test]
    fn the_fake_refuses_as_a_target_without_notifications_does() {
        let mut fake = FakeNotifier::new().will_refuse().will_want_a_bundle();
        assert_eq!(fake.notify(&mail()), Err(NotifyError::Unsupported));
        assert_eq!(fake.notify(&mail()), Err(NotifyError::NeedsBundle));
        assert!(fake.sent().is_empty(), "nothing shown");
        assert!(fake.notify(&mail()).is_ok(), "the refusals are spent");
        assert_eq!(
            NotifyError::Unsupported.to_string(),
            "notification: no system notification on this target"
        );
        assert!(NotifyError::NeedsBundle.to_string().contains("app bundle"));
        assert_eq!(
            NotifyError::Platform("x".into()).to_string(),
            "the platform refused the notification: x"
        );
    }

    #[test]
    fn a_shared_fake_notifies_through_one_handle_and_is_clicked_through_the_other() {
        let fake = Arc::new(Mutex::new(FakeNotifier::new()));
        let mut page: Box<dyn Notifier> = Box::new(fake.clone());
        let id = page.notify(&mail()).unwrap();
        assert!(locked(&fake).click(id));
        let mut shell: Box<dyn Notifier> = Box::new(fake.clone());
        assert_eq!(shell.poll().len(), 1);
        assert_eq!(locked(&fake).sent().len(), 1);
        // a boxed one shares too
        let boxed = Arc::new(Mutex::new(Box::new(NoNotifier)));
        let mut through: Box<dyn Notifier> = Box::new(boxed.clone());
        assert_eq!(through.notify(&mail()), Err(NotifyError::Unsupported));
    }

    #[test]
    fn no_notifier_is_unsupported_and_never_clicked() {
        let mut none = NoNotifier;
        assert_eq!(none.notify(&mail()), Err(NotifyError::Unsupported));
        assert!(none.poll().is_empty());
        assert_eq!(Notification::error("t", "b").level, Level::Error);
        assert_eq!(Notification::new(Level::Info, "t", "b").action, None);
    }

    #[test]
    fn a_bundle_is_told_from_the_executables_place() {
        assert!(bundled(Path::new(
            "/Applications/plev.app/Contents/MacOS/showcase"
        )));
        assert!(!bundled(Path::new("/Users/me/plev/target/debug/showcase")));
        assert!(!bundled(Path::new(
            "/Applications/plev.app/Contents/Resources/showcase"
        )));
        assert!(!bundled(Path::new(
            "/Applications/plev/Contents/MacOS/showcase"
        )));
        assert!(!bundled(Path::new("showcase")));
    }

    /// The desktop's notifier today: nothing is shown and the reason is
    /// the one the platform would give. No framework is touched (a test
    /// binary runs outside a bundle, as `cargo run` does).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    #[test]
    fn the_native_notifier_says_why_it_cannot_yet() {
        let mut native = NativeNotifier::new();
        let why = native.notify(&mail()).unwrap_err();
        if cfg!(target_os = "macos") {
            assert_eq!(why, NotifyError::NeedsBundle);
        } else {
            assert_eq!(why, NotifyError::Unsupported);
        }
        assert!(native.poll().is_empty());
    }
}
