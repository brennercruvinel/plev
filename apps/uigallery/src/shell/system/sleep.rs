//! The shell asleep, covered, and awake again (task-76 item 7): what the
//! platform says (`WindowEvent::Occluded` per window; `suspended` and
//! `resumed`, which the desktop never sends and the mobile bring-up
//! will) and what the shell does with it.
//!
//! Covered, a window's host asks for no frame and has no deadline
//! (`engine::window::WindowHost::set_occluded`): the view goes on, its
//! catch-up ticks with the real time, and as the window shows again the
//! host asks for one frame, built from the state as it is then. Asleep,
//! the shell does two things less: it neither settles the view (no
//! catch-up, so the pages read no worker's channel) nor draws (no host is
//! awake); the clock does not tick either, so the time asleep lands
//! whole on the first tick after the wake, and a spring that would have
//! ended has ended. On the wake the shell wakes the windows (one new frame
//! each), catches up (the answers that arrived asleep are applied, an ask
//! held is started, what ended is told), and the frame that follows shows
//! it all. The work beside the frames hears the sleep through
//! `engine::window::Background`, set by `Windows::sleep` and `wake`. The
//! camera and the microphone hear it at once, asleep or covered
//! (`capture.rs`): a device is released, not paused.

use winit::window::WindowId;

use super::super::App;

impl App {
    /// The app falls asleep: no window draws and nothing settles until
    /// `wake`.
    pub(super) fn sleep(&mut self) {
        // android takes the window away while asleep: each host drops its
        // surface and waits for `resume`. the old surface, configured
        // again on the way back from the home screen, failed wgpu's
        // validation and killed the render thread (task-18)
        #[cfg(target_os = "android")]
        self.windows.suspend();
        #[cfg(not(target_os = "android"))]
        self.windows.sleep();
        self.capture_follows();
    }

    /// The app wakes: every window is invalidated and asks for one new
    /// frame; the view catches up with the whole time asleep, applying
    /// what the workers answered meanwhile and starting what was held.
    pub(super) fn wake(&mut self) {
        if !self.windows.asleep() {
            return;
        }
        self.windows.wake();
        self.capture_follows();
        self.catch_up();
    }

    /// Back on android: each suspended host gets the new window and a
    /// surface on the kept device, the main window's id follows, and the
    /// shell catches up (`Windows::resume` wakes the windows).
    #[cfg(target_os = "android")]
    pub(super) fn resume(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attrs = || winit::window::WindowAttributes::default().with_title(super::super::TITLE);
        match self.windows.resume(event_loop, attrs) {
            Ok(ids) => {
                if let Some(&id) = ids.first() {
                    self.main = Some(id);
                    self.viewport_changed(id);
                }
            }
            Err(e) => log::error!("resume: {e}"),
        }
        self.capture_follows();
        self.catch_up();
    }

    /// The platform's word on window `id`'s visibility: covered, it asks
    /// for nothing; shown again, its host asks for one new frame.
    pub(super) fn covered(&mut self, id: WindowId, covered: bool) {
        if let Some(host) = self.windows.get_mut(id) {
            host.set_occluded(covered);
        }
        self.capture_follows();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine::runtime::platform::menu::locked;
    use engine::runtime::platform::notify::FakeNotifier;
    use engine::styling::theme::Intent;
    use engine::runtime::window::Background;
    use accord::{Actor, Request, SemId, Stage, Value};
    use winit::window::WindowId;

    use super::super::super::App;
    use crate::views::ShowcaseView;

    /// The shell on `page` with a bare main window and a fake notifier.
    fn app(page: &str) -> (App, Arc<Mutex<FakeNotifier>>) {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page(page);
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_bare_window(view);
        let fake = Arc::new(Mutex::new(FakeNotifier::new()));
        app.set_notices(Box::new(fake.clone()));
        (app, fake)
    }

    /// A frame drawn for the main window: the pending ask taken, the
    /// scene built.
    fn frame(app: &mut App) {
        let host = app.windows.get_mut(WindowId::dummy()).unwrap();
        host.begin_frame();
        app.view.render(host.compositor_mut());
    }

    /// A frame was asked of the main window (a bare host keeps the ask
    /// for its driver; its compositor is never resolved, so the ask is
    /// the word).
    fn asked(app: &App) -> bool {
        app.windows.get(WindowId::dummy()).unwrap().redraw_asked()
    }

    /// A state of the page as it projects it now.
    fn projected(app: &App, at: &str) -> Value {
        app.view
            .semantics()
            .expect("the page projects")
            .value(&at.parse().expect("a state ref"))
            .cloned()
            .expect("projected")
    }

    /// Whether the page offers a detection now: not while one runs.
    fn detect_enabled(app: &App) -> bool {
        app.view
            .semantics()
            .expect("the page projects")
            .get(&SemId::parse("silences.detect").unwrap())
            .expect("the detect button")
            .enabled
    }

    /// A detection asked of the Silences page ends off the loop at once
    /// in the crate's tests (ffmpeg is not looked for). Asleep, the shell
    /// neither settles it nor asks for a frame nor tells anyone, however
    /// many times the loop wakes; the wake applies it, asks for the
    /// frame, and tells the person, who was not looking.
    #[test]
    fn a_job_that_ends_while_the_app_sleeps_is_applied_on_the_wake_with_no_frame_between() {
        let (mut app, fake) = app("Silences");
        app.focus_changed(WindowId::dummy(), false);
        frame(&mut app);
        assert!(!asked(&app), "at rest");
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
        assert!(matches!(receipt.stage(), Some(Stage::Executing)));
        assert!(!detect_enabled(&app), "a detection runs");
        // the act's own frame
        frame(&mut app);
        assert!(!asked(&app));

        app.sleep();
        assert!(app.windows.asleep());
        assert!(Background::asleep());
        for _ in 0..5 {
            app.advance(0.2);
            app.catch_up();
        }
        assert!(!detect_enabled(&app), "not settled asleep: still running");
        assert!(!asked(&app), "no frame asleep");
        assert!(locked(&fake).sent().is_empty(), "nobody told asleep");
        // a change from outside asks for nothing either
        app.changed();
        assert!(!asked(&app));

        app.wake();
        assert!(!app.windows.asleep());
        assert!(!Background::asleep());
        assert!(detect_enabled(&app), "settled on the wake: the job ended");
        let Value::Text(status) = projected(&app, "silences.status:message") else {
            panic!("a status");
        };
        assert!(status.contains("ffmpeg"), "{status}");
        assert!(asked(&app), "the wake asks for the frame that shows it");
        let sent = locked(&fake).sent().to_vec();
        assert_eq!(sent.len(), 1, "told on the wake");
        assert!(sent[0].notification.body.contains("ffmpeg"));
        app.wake();
        assert!(!app.windows.asleep(), "waking awake changes nothing");
    }

    /// The main window covered: a change asks for no frame and a toast's
    /// motion has no deadline; shown again, one frame is asked and the
    /// toast is where the real time put it.
    #[test]
    fn a_covered_main_window_asks_for_no_frame_and_asks_for_one_as_it_shows() {
        let (mut app, _) = app("Button");
        let main = WindowId::dummy();
        frame(&mut app);
        app.advance(1.0);
        assert!(!asked(&app), "at rest");

        app.covered(main, true);
        assert!(!app.windows.awake(main));
        let theme = app.view.theme.clone();
        app.view.toasts.push("saved", Intent::Constructive, &theme);
        app.changed();
        assert!(!asked(&app), "covered: the change waits");
        // the loop ticks on (a worker woke it): the toast springs in and
        // out by the real time, and the window asks for nothing
        for _ in 0..70 {
            app.advance(0.1);
            assert!(!asked(&app), "covered: motion waits");
            assert_eq!(app.windows.earliest_deadline(), None);
        }
        assert!(
            app.view.toasts.visible().next().is_none(),
            "the toast lived its time while covered"
        );

        app.covered(main, false);
        assert!(app.windows.awake(main));
        assert!(asked(&app), "shown: one frame asked");
        frame(&mut app);
        assert!(!asked(&app), "served");
        app.covered(main, false);
        assert!(!asked(&app), "the same word twice asks nothing");
    }
}
