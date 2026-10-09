//! The gallery's system notifications (task-76 item 5), desktop only: the
//! shell owns the platform's notifier (`engine::platform::notify`) and
//! decides when to tell the person something they were not looking at.
//! Two things are told:
//!
//! - what a page asked to tell (`ShowcaseView::take_notifications`): the
//!   Files page, when files land while no window of ours has the focus,
//!   with its open button as the notification's own;
//! - what ended in the background while no window had the focus: a
//!   detection of silences (its request completed or failed off the loop,
//!   heard from the gallery's dispatcher through a `ReceiptSink`), told
//!   with "show plev" (`shell.show`) as the button, and projected as
//!   `shell:notice` for an agent to read.
//!
//! The focus is the platform's word (`WindowEvent::Focused`, per window):
//! the shell keeps which window of ours has it and tells the view. A
//! click on a notification or its button wakes the loop
//! (`UserEvent::Clicked`), is polled there and dispatched by the view as
//! the person's request (`notice_clicks`), the path a bar item takes; a
//! capability that left the projection meanwhile asks nothing. Nothing
//! here opens a notification on this machine today: the desktop's
//! notifier says why it cannot yet (macOS wants an app bundle) and the
//! projected text stays the truth; the tests use the fake.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use accord::{Receipt, ReceiptSink, SemId, Stage, Value};
use engine::runtime::platform::menu::locked;
use engine::runtime::platform::notify::{Action, Notification, Notifier, default_notifier};
use winit::window::WindowId;

use super::super::{App, UserEvent};
use crate::views::ShowcaseView;

/// The shell's capability a notice's button asks for, as the projection
/// names it (`view::dispatch::shell`).
const SHOW: &str = "shell.show";

/// The action whose end off the loop is told: the Silences page's
/// detection (`silences.detect`, run by ffmpeg on a thread of its own).
const DETECT: &str = "detect_silence";

/// The notifier, and what the background jobs came to.
pub(crate) struct Notices {
    notifier: Box<dyn Notifier>,
    jobs: Arc<Mutex<VecDeque<Finished>>>,
}

/// A request that ran off the loop and ended.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Finished {
    /// A detection completed, with this many silences found.
    Silences { cuts: i64 },
    /// A detection failed, in the worker's words.
    Failed { why: String },
}

impl Finished {
    /// What the person is told.
    fn notification(&self) -> Notification {
        match self {
            Finished::Silences { cuts: 1 } => Notification::info("Silences", "1 silence found"),
            Finished::Silences { cuts } => {
                Notification::info("Silences", format!("{cuts} silences found"))
            }
            Finished::Failed { why } => Notification::error("Silences", why.clone()),
        }
    }
}

/// Hears every receipt of the gallery's dispatcher and keeps the ends of
/// the requests that run in the background (a detection): the other
/// stages, and every other request, pass by.
struct JobSink {
    jobs: Arc<Mutex<VecDeque<Finished>>>,
}

impl ReceiptSink for JobSink {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        if receipt.request.action.as_str() != DETECT {
            return;
        }
        let finished = match stage {
            Stage::Completed(outcome) => Finished::Silences {
                cuts: match outcome.result.get("cuts") {
                    Some(Value::Int(n)) => *n,
                    _ => 0,
                },
            },
            Stage::Failed { error } => Finished::Failed { why: error.clone() },
            _ => return,
        };
        locked(&self.jobs).push_back(finished);
    }
}

impl App {
    /// The platform's notifier, woken through the proxy on a click, and
    /// the ear on the gallery's dispatcher for the jobs that end off the
    /// loop. Once, when the main window is up.
    pub(crate) fn install_notices(&mut self) {
        let proxy = Mutex::new(self.access.proxy.clone());
        let notifier = default_notifier(move || {
            if let Ok(proxy) = proxy.lock()
                && let Some(proxy) = proxy.as_ref()
            {
                let _ = proxy.send_event(UserEvent::Clicked);
            }
        });
        self.set_notices(notifier);
    }

    /// Notifies through `notifier` (the platform's, or a fake) and hears
    /// the background jobs end from now on.
    pub(super) fn set_notices(&mut self, notifier: Box<dyn Notifier>) {
        let jobs = Arc::new(Mutex::new(VecDeque::new()));
        self.view.add_sink(Box::new(JobSink { jobs: jobs.clone() }));
        self.notices = Some(Notices { notifier, jobs });
    }

    /// Window `id` of ours took or lost the focus (the platform's word):
    /// the view learns whether any window has it.
    pub(crate) fn focus_changed(&mut self, id: WindowId, on: bool) {
        if on {
            self.focus = Some(id);
        } else if self.focus == Some(id) {
            self.focus = None;
        }
        self.view.set_window_focused(self.focus.is_some());
    }

    /// What the pages asked to tell, and what ended in the background
    /// while no window had the focus (a job that ended while one had it
    /// was seen on screen and is not told): each shown through the
    /// platform, the text projected either way, a refusal logged.
    pub(crate) fn notify_pending(&mut self) {
        let Some(notices) = self.notices.as_mut() else {
            return;
        };
        let mut pending = self.view.take_notifications();
        let finished: Vec<Finished> = locked(&notices.jobs).drain(..).collect();
        if !finished.is_empty() && self.focus.is_none() {
            let show = show_action(&self.view);
            for job in finished {
                let mut notification = job.notification();
                if let Some(show) = &show {
                    notification = notification.action(show.clone());
                }
                self.view.set_notice(&notification.body);
                pending.push(notification);
            }
        }
        for notification in pending {
            match notices.notifier.notify(&notification) {
                Ok(id) => log::info!("notification {id}: {}", notification.body),
                Err(e) => log::info!("notification not shown ({e}): {}", notification.body),
            }
        }
    }

    /// The clicks on notifications since the last poll, each dispatched
    /// by the view as the person's request (`ShowcaseView::menu_click`).
    /// `true` when the gallery's projection moved, so a frame is needed.
    /// What the shell owns is left for the caller: `show_requested`.
    pub(crate) fn notice_clicks(&mut self) -> bool {
        let Some(notices) = self.notices.as_mut() else {
            return false;
        };
        let mut changed = false;
        for clicked in notices.notifier.poll() {
            changed |= self.view.menu_click(&clicked);
        }
        changed
    }

    /// The tray icon, its item or a notification asked for the main
    /// window to come forward: shown, unminimized, focused.
    pub(crate) fn show_requested(&mut self) {
        if !self.view.take_show_request() {
            return;
        }
        let window = self
            .main
            .and_then(|id| self.windows.get(id))
            .and_then(|host| host.window());
        match window {
            Some(window) => {
                window.set_visible(true);
                window.set_minimized(false);
                window.focus_window();
            }
            None => log::debug!("show: no main window to bring forward"),
        }
    }
}

/// `shell.show` as a notification's button, labelled "show", when the
/// shell projects it for a person now.
fn show_action(view: &ShowcaseView) -> Option<Action> {
    let gallery = view.gallery_semantics().ok()?;
    let show = SemId::parse(SHOW).ok()?;
    Action::of(&gallery, &show, "activate").map(|a| a.label("show"))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use accord::{Actor, Request, SemId, Value};
    use engine::control::input::FileDrop;
    use engine::runtime::platform::menu::locked;
    use engine::runtime::platform::notify::{FakeNotifier, Level};
    use winit::window::WindowId;

    use super::super::super::App;
    use super::Finished;
    use crate::views::ShowcaseView;

    fn sid(text: &str) -> SemId {
        SemId::parse(text).expect("a valid id")
    }

    /// The shell on `page` with a bare main window and a fake notifier,
    /// kept by the test through the shared handle.
    fn app(page: &str) -> (App, Arc<Mutex<FakeNotifier>>) {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page(page);
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_bare_window(view);
        let fake = Arc::new(Mutex::new(FakeNotifier::new()));
        app.set_notices(Box::new(fake.clone()));
        (app, fake)
    }

    fn projected(app: &App, at: &str) -> Value {
        app.view
            .gallery_semantics()
            .unwrap()
            .value(&at.parse().unwrap())
            .cloned()
            .expect("projected")
    }

    fn dropped(paths: &[&str]) -> FileDrop {
        FileDrop::Dropped {
            paths: paths.iter().map(std::path::PathBuf::from).collect(),
            position: None,
        }
    }

    #[test]
    fn files_that_land_while_no_window_has_the_focus_are_told_and_the_button_opens() {
        let (mut app, fake) = app("Files");
        let main = WindowId::dummy();

        // the window has the focus: a drop is seen, nothing is told
        app.focus_changed(main, true);
        assert!(app.view.handle_file_drop(&dropped(&["/a"])));
        app.notify_pending();
        assert!(locked(&fake).sent().is_empty());
        assert_eq!(projected(&app, "files:notice"), Value::from(""));

        // the focus goes elsewhere: the next drop is told, with the open
        // button as the notification's, and the text is state
        app.focus_changed(main, false);
        assert!(app.view.handle_file_drop(&dropped(&["/b", "/c"])));
        app.notify_pending();
        let sent = locked(&fake).sent().to_vec();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].notification.level, Level::Info);
        assert_eq!(sent[0].notification.title, "Files");
        assert_eq!(sent[0].notification.body, "2 files received");
        let button = sent[0].notification.action.as_ref().expect("a button");
        assert_eq!(button.sem, sid("files.open"));
        assert_eq!(button.action.as_str(), "open_file");
        assert_eq!(button.label, "open file");
        assert_eq!(
            projected(&app, "files:notice"),
            Value::from("2 files received")
        );
        assert_eq!(projected(&app, "files:count"), Value::Int(3));
        app.notify_pending();
        assert_eq!(locked(&fake).sent().len(), 1, "told once");

        // the person clicks the notification: open_file runs by the
        // person, through the dispatcher, over the fake dialog (which
        // cancels), and the status says so
        assert!(locked(&fake).click(sent[0].id));
        assert!(app.notice_clicks(), "the status moved");
        assert_eq!(
            projected(&app, "files.status:message"),
            Value::from("the dialog was cancelled")
        );

        // a second window of ours takes the focus: seen again
        locked(&fake).take_sent();
        app.focus_changed(WindowId::dummy(), true);
        assert!(app.view.handle_file_drop(&dropped(&["/d"])));
        app.notify_pending();
        assert!(locked(&fake).take_sent().is_empty());
        assert_eq!(
            projected(&app, "files:notice"),
            Value::from("2 files received"),
            "the last notice stays"
        );
    }

    #[test]
    fn a_job_that_ends_while_no_window_has_the_focus_is_told_with_show_as_the_button() {
        let (mut app, fake) = app("Silences");
        let main = WindowId::dummy();
        app.focus_changed(main, false);
        assert_eq!(projected(&app, "shell:notice"), Value::from(""));

        // a person asks for the detection; in the crate's tests ffmpeg is
        // never looked for, so the job ends off the loop saying so
        let gallery = app.view.gallery_semantics().unwrap();
        let req = Request::observe(
            &gallery,
            Actor::Human,
            "silences.detect",
            "detect_silence",
            None,
        )
        .unwrap();
        let receipt = app.view.act(req).expect("the page takes it");
        assert!(
            matches!(receipt.stage(), Some(accord::Stage::Executing)),
            "{:?}",
            receipt.stage()
        );
        app.advance(0.2);
        let sent = locked(&fake).sent().to_vec();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].notification.level, Level::Error);
        assert_eq!(sent[0].notification.title, "Silences");
        assert!(
            sent[0].notification.body.contains("ffmpeg"),
            "{}",
            sent[0].notification.body
        );
        let button = sent[0].notification.action.as_ref().expect("a button");
        assert_eq!(button.sem, sid("shell.show"));
        assert_eq!(button.label, "show");
        assert_eq!(
            projected(&app, "shell:notice"),
            Value::from(sent[0].notification.body.as_str()),
            "what the person was told, for an agent to read"
        );

        // the click brings the window forward: the shell takes the ask
        assert!(locked(&fake).click(sent[0].id));
        app.notice_clicks();
        assert!(app.view.take_show_request());
        app.show_requested();

        // with the focus back, a job's end is seen on screen, not told
        locked(&fake).take_sent();
        app.focus_changed(main, true);
        let gallery = app.view.gallery_semantics().unwrap();
        let req = Request::observe(
            &gallery,
            Actor::Human,
            "silences.detect",
            "detect_silence",
            None,
        )
        .unwrap();
        app.view.act(req);
        app.advance(0.2);
        assert!(locked(&fake).take_sent().is_empty());

        // what a completed detection would say
        assert_eq!(
            Finished::Silences { cuts: 2 }.notification().body,
            "2 silences found"
        );
        assert_eq!(
            Finished::Silences { cuts: 1 }.notification().body,
            "1 silence found"
        );
    }
}
