//! The camera and the microphone in the shell (task-76 item 10): the
//! platform's devices given to the Capture page once the main window is
//! up (`install_capture`), and the windows' word passed on to it
//! (`capture_follows`): the page runs a device only while a window that
//! shows it is awake. Covered (minimized, hidden, fully behind others),
//! asleep, or with its own window closed while the main one is covered,
//! the device is stopped and released the moment the shell hears of it,
//! not at the next frame, which a covered window never draws; awake
//! again, what the person had turned on starts again. Nothing here
//! starts a device on its own: only a person's `toggle` does.

use super::super::App;

impl App {
    /// The platform's camera and microphone, once, when the main window
    /// is up. Building them runs nothing and opens nothing.
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub(crate) fn install_capture(&mut self) {
        self.view
            .set_capture(engine::runtime::platform::capture::default_capture());
        self.capture_follows();
    }

    /// Tells the Capture page whether a window that shows it is awake:
    /// the main window, or the page's own while it is open, neither
    /// covered, the app not asleep. A change it shows asks for a frame
    /// (which a window that is not awake withholds).
    pub(crate) fn capture_follows(&mut self) {
        let awake =
            |id: Option<winit::window::WindowId>| id.is_some_and(|id| self.windows.awake(id));
        let shown = !self.windows.asleep() && (awake(self.main) || awake(self.page_window_id()));
        if self.view.set_capture_shown(shown) {
            self.invalidate();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine::runtime::platform::capture::{DeviceKind, FakeCapture};
    use engine::runtime::platform::menu::locked;
    use engine::runtime::window::Background;
    use accord::{Actor, Request, Stage, StateRef, Value};
    use winit::window::WindowId;

    use super::super::super::App;
    use crate::views::ShowcaseView;

    /// The shell on the Capture page over a shared fake capture, with a
    /// bare main window.
    fn app() -> (App, Arc<Mutex<FakeCapture>>) {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page("Capture");
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_bare_window(view);
        let fake = Arc::new(Mutex::new(FakeCapture::new()));
        app.view.set_capture(Box::new(fake.clone()));
        app.capture_follows();
        app.advance(0.0);
        (app, fake)
    }

    fn line(app: &App, kind: &str) -> Value {
        app.view
            .semantics()
            .expect("the page projects")
            .value(&StateRef::parse(&format!("capture:{kind}")).unwrap())
            .cloned()
            .expect("projected")
    }

    fn turn(app: &mut App, actor: Actor, id: &str) -> Stage {
        let seen = app.view.semantics().expect("the page projects");
        let req = Request::observe(&seen, actor, id, "toggle", None).unwrap();
        let receipt = app.view.act(req).expect("the page takes it");
        receipt.stage().cloned().expect("a stage")
    }

    /// The person turns the camera on; the main window covered, the
    /// device is released at once, with no frame between; shown again, it
    /// starts again. Asleep the same, and the wake takes it back.
    #[test]
    fn covered_or_asleep_the_shell_releases_the_camera_and_takes_it_back() {
        let (mut app, fake) = app();
        let main = WindowId::dummy();
        let agent = Actor::Agent {
            name: "claude".to_owned(),
        };
        assert!(matches!(
            turn(&mut app, agent, "capture.camera"),
            Stage::Rejected(_)
        ));
        assert_eq!(locked(&fake).starts(DeviceKind::Camera), 0);
        assert!(matches!(
            turn(&mut app, Actor::Human, "capture.camera"),
            Stage::Completed(_)
        ));
        assert_eq!(locked(&fake).starts(DeviceKind::Camera), 1);
        app.advance(0.0);
        assert_eq!(line(&app, "camera"), Value::from("on 640x360 at 15 fps"));

        app.covered(main, true);
        assert_eq!(
            locked(&fake).stops(DeviceKind::Camera),
            1,
            "released at once"
        );
        assert_eq!(
            line(&app, "camera"),
            Value::from("paused while the page is not seen")
        );
        let taken = locked(&fake).frames();
        for _ in 0..5 {
            app.advance(0.1);
        }
        assert_eq!(locked(&fake).frames(), taken, "nothing taken covered");
        app.covered(main, false);
        assert_eq!(locked(&fake).starts(DeviceKind::Camera), 2);

        app.sleep();
        assert!(Background::asleep());
        assert_eq!(locked(&fake).stops(DeviceKind::Camera), 2);
        app.wake();
        assert_eq!(locked(&fake).starts(DeviceKind::Camera), 3);
        app.advance(0.0);
        assert_eq!(line(&app, "camera"), Value::from("on 640x360 at 15 fps"));

        // another page: released; back: taken again
        app.view.jump_to_page("Files");
        app.advance(0.0);
        assert_eq!(locked(&fake).stops(DeviceKind::Camera), 3);
        app.view.jump_to_page("Capture");
        app.advance(0.0);
        assert_eq!(locked(&fake).starts(DeviceKind::Camera), 4);
    }
}
