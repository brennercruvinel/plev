//! Printing the page (task-76 item 9), the shell's side, desktop only:
//! the platform's printer given to the view once the main window is up
//! (`install_print`), and the print keys (cmd-p on macOS, ctrl-p
//! elsewhere) asking `shell.print` as the menu item does, the person's
//! request through the gallery's dispatcher. The job itself is the
//! view's (`view::dispatch::shell::print`): the scene built on the loop,
//! the render and the spool on a thread of its own, the answer read on
//! every catch-up (`pace.rs`), and the bar following the next frame. On
//! this machine nothing is printed: it has no destination, the button
//! stays disabled and `shell:print` says "no destination"; the tests use
//! the fake.

use engine::platform::print::default_printer;
use winit::keyboard::{Key, ModifiersState};

use super::App;

/// The keys that print the page, as the person reads them: `p` with the
/// primary modifier (cmd on macOS, ctrl elsewhere), the platform's own
/// print shortcut.
pub const PRINT_KEYS: &str = if cfg!(target_os = "macos") {
    "cmd-p"
} else {
    "ctrl-p"
};

/// True for `PRINT_KEYS`: the character `p` with cmd on macOS or ctrl
/// elsewhere, and no shift (cmd-shift-p is nobody's here).
pub fn is_print_key(key: &Key, modifiers: ModifiersState) -> bool {
    let primary = if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    };
    primary
        && !modifiers.shift_key()
        && matches!(key, Key::Character(c) if c.eq_ignore_ascii_case("p"))
}

impl App {
    /// The platform's printer, once, when the main window is up: its
    /// destinations are looked for off the loop and projected when they
    /// land.
    pub(super) fn install_print(&mut self) {
        self.view.set_printer(default_printer());
    }

    /// A key pressed: the print keys ask `shell.print` (the person's
    /// request, through the dispatcher; the job follows) and go to nobody
    /// else (`true`).
    pub(super) fn print_key(&mut self, key: &Key) -> bool {
        if !is_print_key(key, self.modifiers) {
            return false;
        }
        let taken = self.view.request_print();
        log::debug!("print keys: taken up {taken}");
        self.changed();
        true
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use engine::platform::menu::{FakeMenu, Shortcut, locked};
    use engine::platform::print::FakePrinter;
    use sem::{SemId, Value};
    use winit::keyboard::{Key, ModifiersState, NamedKey};

    use super::super::App;
    use super::{PRINT_KEYS, is_print_key};
    use crate::view::ShowcaseView;

    /// The primary modifier of the platform, as the print keys read it.
    const PRIMARY: ModifiersState = if cfg!(target_os = "macos") {
        ModifiersState::SUPER
    } else {
        ModifiersState::CONTROL
    };

    fn sid(text: &str) -> SemId {
        SemId::parse(text).expect("a valid id")
    }

    /// The shell on `page` with a bare main window and a fake menu.
    fn app(page: &str) -> (App, Arc<Mutex<FakeMenu>>) {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page(page);
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_bare_window(view);
        let fake = Arc::new(Mutex::new(FakeMenu::new()));
        app.set_menu(Box::new(fake.clone()));
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

    /// Catches up until the print's thread answered, or five seconds.
    fn settled(app: &mut App) -> bool {
        let until = Instant::now() + Duration::from_secs(5);
        while Instant::now() < until {
            if app.view.settle_print() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }

    #[test]
    fn the_print_keys_are_p_with_the_primary_modifier_alone() {
        let p = Key::Character("p".into());
        let upper = Key::Character("P".into());
        assert!(is_print_key(&p, PRIMARY));
        assert!(is_print_key(&upper, PRIMARY));
        assert!(!is_print_key(&p, PRIMARY | ModifiersState::SHIFT));
        assert!(!is_print_key(&p, ModifiersState::empty()));
        assert!(!is_print_key(&p, ModifiersState::ALT));
        assert!(!is_print_key(&Key::Character("s".into()), PRIMARY));
        assert!(!is_print_key(&Key::Named(NamedKey::Enter), PRIMARY));
        assert_eq!(Shortcut::parse(PRINT_KEYS).unwrap().to_string(), PRINT_KEYS);
    }

    /// While the look for destinations or a job is out, the frame's wake
    /// asks for a look at the channel, so the answer lands on the
    /// catch-up a deadline brings and not on whatever input comes next;
    /// once it answered the print asks nothing.
    #[test]
    fn a_print_thread_out_keeps_a_deadline_until_it_answers() {
        let (mut app, _menu) = app("Todo");
        assert_eq!(app.view.print_wake().next, None, "no printer, nothing out");
        app.view
            .set_printer(Box::new(FakePrinter::new().destination("Office", true)));
        assert!(app.view.print_wake().next.is_some(), "the look is out");
        assert!(app.frame_wake(0.0).next.is_some(), "the frame carries it");
        assert!(settled(&mut app), "the destinations land");
        assert_eq!(app.view.print_wake().next, None, "the look answered");
        assert!(app.view.request_print(), "a job starts");
        assert!(app.view.print_wake().next.is_some(), "the job is out");
        assert!(settled(&mut app), "the job answers");
        assert_eq!(app.view.print_wake().next, None, "the job answered");
    }

    /// Without a destination the item is in the bar, disabled, with its
    /// keys; the keys ask and are refused as the disabled button is, and
    /// the projection says why. A printer with a destination, looked for
    /// off the loop, enables the item on the next sync.
    #[test]
    fn the_bar_shows_the_print_with_its_keys_and_follows_the_destinations() {
        let (mut app, menu) = app("Todo");
        let bar = locked(&menu).installed().cloned().expect("a bar");
        let item = bar
            .find(&sid("shell.print"), "activate", None)
            .expect("the print is in the bar");
        assert_eq!(item.label, "print this page");
        assert!(!item.enabled, "no destination on a bare shell");
        assert_eq!(item.shortcut, Some(Shortcut::parse(PRINT_KEYS).unwrap()));
        assert_eq!(
            projected(&app, "shell:print"),
            Value::from("no destination")
        );

        app.modifiers = PRIMARY;
        assert!(
            app.print_key(&Key::Character("p".into())),
            "the keys are taken"
        );
        assert_eq!(
            projected(&app, "shell:print"),
            Value::from("no destination"),
            "refused: nothing runs"
        );
        assert!(!app.print_key(&Key::Character("t".into())));
        app.modifiers = ModifiersState::empty();
        assert!(
            !app.print_key(&Key::Character("p".into())),
            "a plain p is a key of the page"
        );

        let fake = FakePrinter::new().destination("Office", true);
        app.view.set_printer(Box::new(fake));
        assert!(settled(&mut app), "the destinations land");
        app.sync_menu();
        let bar = locked(&menu).installed().cloned().expect("a bar");
        assert!(
            bar.find(&sid("shell.print"), "activate", None)
                .unwrap()
                .enabled
        );
        assert_eq!(
            projected(&app, "shell:print"),
            Value::from("ready on Office")
        );
        assert_eq!(
            projected(&app, "shell.print:destinations"),
            Value::List(vec!["Office".to_owned()])
        );
    }
}
