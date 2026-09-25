//! The gallery's shell as a capability (task-76 item 4). What the shell
//! did on a key alone (T cycles the theme, the digits pick a page, D puts
//! the page in a window of its own, cmd-shift-s saves the recording) is
//! projected, `shell` (a region) with `shell.page` and `shell.theme`
//! (selects: `selected`, `select` with the name as text, any actor),
//! `shell.detach` (a toggle: `on`, `toggle`; a person only, as the Files
//! page's `open_file`), `shell.show` (a button: `activate`, brings the
//! main window forward; a person only; the tray icon's own click and a
//! notification's button ask it, task-76 item 5), `shell.print` (a
//! button: `activate`; a person only, cmd-p and the menu; the frame on
//! paper through the system's printer, task-76 item 9, `print.rs`) and,
//! while the session is recorded, `shell.save_session` (a button:
//! `activate`; a person only), and run through the gallery's one
//! dispatcher (the todo page's,
//! lent by `TodoPage::with_dispatcher` as the Surface, Silences and Files
//! pages do). A key, a menu item, a tray item, a notification, the theme
//! page's cards, the sidebar, a screen reader and an agent through the
//! port ask the same way and get a receipt; the policy holds for every
//! one of them. The region's `notice` is the last thing the shell told
//! the person by notification on its own (a job that ended in the
//! background), for an agent to read.
//!
//! The shell owns the windows, the recording and the platform's tray and
//! notifications; the view does not. So `execute` records what was asked
//! ([`Asks`]): a page or a theme the view applies right after the
//! dispatch; a detach, a save or a show the shell takes on its loop
//! ([`ShowcaseView::take_detach_request`],
//! [`ShowcaseView::take_save_request`],
//! [`ShowcaseView::take_show_request`]), as `D` always worked. The shell
//! also tells the view whether a window has the focus
//! ([`ShowcaseView::set_window_focused`]) and takes what a page asked to
//! tell the person ([`ShowcaseView::take_notifications`]).
//!
//! Where it shows: [`ShowcaseView::semantics`] stays the page's alone (the
//! goldens, the regress baselines, the accessibility tree, the recording
//! and cmd+click read it), and the shell enters
//! [`ShowcaseView::gallery_semantics`], the one snapshot the agent port
//! serves and the menu is built from.

use engine::platform::menu::Clicked;
use engine::platform::notify::Notification;
use sem::{
    Actor, Actors, Constraints, Execution, Operable, Outcome, ParamSpec, Policy, Receipt, Request,
    RequestId, Role, SemError, SemErrors, SemId, Semantics, Snapshot, SnapshotBuilder, Stage,
    Value, ValueKind,
};

use super::super::foundations::theme::{self, THEMES};
use super::super::{PAGES, ShowcaseView, pages};
use super::detached;

/// Who may ask for what opens a window or writes a file: a person, at the
/// screen or through assistive technology, and a test; never an agent
/// (the Files page's `open_file` says the same).
pub(crate) const PERSON_ONLY: Actors = Actors {
    human: true,
    agent: false,
    assistive: true,
    test: true,
};

/// The page's own window exists on the desktop alone.
const DESKTOP: bool = cfg!(not(any(
    target_arch = "wasm32",
    target_os = "android",
    target_os = "ios"
)));

/// A text the node's options bound.
const TEXT: ParamSpec = ParamSpec {
    kind: ValueKind::Text,
    constrained: true,
};

/// The shell's capabilities as the view keeps them: the ids it projects,
/// what was asked of it since the shell last looked, whether the session
/// is recorded (`shell.save_session` is projected while it is), and the
/// last notice the shell gave on its own (`shell:notice`).
pub struct Shell {
    ids: Option<Ids>,
    asks: Asks,
    recording: bool,
    notice: String,
    /// The printer, the destinations, the job in course and the last
    /// (`print.rs`).
    print: print::Printing,
}

impl Default for Shell {
    fn default() -> Self {
        Shell::new()
    }
}

/// What `execute` recorded and nobody took yet.
#[derive(Debug, Default)]
struct Asks {
    page: Option<usize>,
    theme: Option<String>,
    detach: bool,
    save: bool,
    show: bool,
}

struct Ids {
    shell: SemId,
    page: SemId,
    theme: SemId,
    detach: SemId,
    show: SemId,
    print: SemId,
    save: SemId,
}

impl Ids {
    fn new() -> Result<Ids, SemError> {
        let shell = SemId::name("shell")?;
        Ok(Ids {
            page: shell.child("page")?,
            theme: shell.child("theme")?,
            detach: shell.child("detach")?,
            show: shell.child("show")?,
            print: shell.child("print")?,
            save: shell.child("save_session")?,
            shell,
        })
    }

    fn holds(&self, id: &SemId) -> bool {
        [
            &self.shell,
            &self.page,
            &self.theme,
            &self.detach,
            &self.show,
            &self.print,
            &self.save,
        ]
        .contains(&id)
    }

    fn of(&self, target: ShellTarget) -> &SemId {
        match target {
            ShellTarget::Page => &self.page,
            ShellTarget::Theme => &self.theme,
            ShellTarget::Detach => &self.detach,
            ShellTarget::Print => &self.print,
            ShellTarget::Save => &self.save,
        }
    }
}

/// The shell's capabilities a key or a control of the view asks for,
/// named without the ids (the view's own name for them; outside the view
/// they are their ids: `shell.page`, `shell.theme`, `shell.detach`,
/// `shell.print`, `shell.save_session`). `shell.show` is asked from
/// outside the window alone (the tray, a notification), by `Clicked`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellTarget {
    Page,
    Theme,
    Detach,
    Print,
    Save,
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            ids: Ids::new().inspect_err(|e| log::warn!("shell: {e}")).ok(),
            asks: Asks::default(),
            recording: false,
            notice: String::new(),
            print: print::Printing::new(),
        }
    }

    /// Whether `id` names the shell or one of its capabilities.
    pub fn holds(&self, id: &SemId) -> bool {
        self.ids.as_ref().is_some_and(|ids| ids.holds(id))
    }

    /// The id of `target`.
    pub fn id(&self, target: ShellTarget) -> Option<SemId> {
        self.ids.as_ref().map(|ids| ids.of(target).clone())
    }

    /// The last thing the shell told the person on its own (`shell:notice`).
    pub(super) fn notice(&self) -> &str {
        &self.notice
    }
}

/// What of the view the shell projects from.
#[derive(Clone, Copy)]
struct View<'a> {
    theme_name: &'a str,
    page: usize,
    detached: bool,
}

/// The shell as the dispatcher sees it: it projects from the view and
/// writes the asks.
struct ShellOp<'a> {
    shell: &'a mut Shell,
    view: View<'a>,
}

fn options<'a>(names: impl Iterator<Item = &'a str>) -> Constraints {
    Constraints {
        editable: true,
        options: Some(names.map(str::to_owned).collect()),
        ..Constraints::default()
    }
}

fn project(shell: &Shell, view: View<'_>, b: &mut SnapshotBuilder) {
    let Some(ids) = &shell.ids else {
        return;
    };
    let page = &PAGES[view.page];
    b.node(&ids.shell, Role::Region, "plev")
        .state("notice", shell.notice.as_str())
        .state("print", shell.print.summary());
    let mut node = b.node(&ids.page, Role::Select, "page");
    node.parent(&ids.shell)
        .state("selected", page.title)
        .constraints(options(PAGES.iter().map(|p| p.title)))
        .focusable(false);
    node.action("select")
        .param(TEXT)
        .reads(format!("{}:selected", ids.page));
    let mut node = b.node(&ids.theme, Role::Select, "theme");
    node.parent(&ids.shell)
        .state("selected", view.theme_name)
        .constraints(options(THEMES.iter().copied()))
        .focusable(false);
    node.action("select")
        .param(TEXT)
        .reads(format!("{}:selected", ids.theme));
    let mut node = b.node(&ids.detach, Role::Toggle, "page in a window of its own");
    node.parent(&ids.shell)
        .state("on", view.detached)
        .enabled(DESKTOP && page.pane().is_some())
        .focusable(false);
    node.action("toggle")
        .reads(format!("{}:on", ids.detach))
        .policy(Policy {
            actors: PERSON_ONLY,
            ..Policy::default()
        });
    // the main window forward: what the tray icon's click and a
    // notification's button ask for; a window is a person's
    let mut node = b.node(&ids.show, Role::Button, "show plev");
    node.parent(&ids.shell).enabled(DESKTOP).focusable(false);
    node.action("activate").policy(Policy {
        actors: PERSON_ONLY,
        ..Policy::default()
    });
    // the frame on paper: a person's, with a destination to print to
    print::project(&shell.print, ids, b);
    if shell.recording {
        let mut node = b.node(&ids.save, Role::Button, "save the session recording");
        node.parent(&ids.shell).focusable(false);
        node.action("activate").policy(Policy {
            actors: PERSON_ONLY,
            ..Policy::default()
        });
    }
}

impl Semantics for ShellOp<'_> {
    fn project(&self, b: &mut SnapshotBuilder) {
        project(self.shell, self.view, b);
    }
}

impl Operable for ShellOp<'_> {
    /// Everything here ends now but a print: a page or a theme is applied
    /// by the view right after the dispatch, a detach or a save is taken
    /// by the shell on its loop (a window that does not open is logged
    /// there); a print starts, and its request is ended by `settle_print`
    /// when the job answers.
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        let Some(ids) = &self.shell.ids else {
            return Execution::Failed("the shell has no ids".to_owned());
        };
        let asks = &mut self.shell.asks;
        let mut outcome = Outcome::default();
        match (req.action.as_str(), &req.target) {
            ("select", target) if *target == ids.page => {
                let Some(Value::Text(title)) = &req.param else {
                    return Execution::Failed("select takes a page title".to_owned());
                };
                let Some(index) = pages::find(title) else {
                    return Execution::Failed(format!("no page {title}"));
                };
                asks.page = Some(index);
                let title = Value::Text(PAGES[index].title.to_owned());
                outcome.result.insert("page".to_owned(), title);
            }
            ("select", target) if *target == ids.theme => {
                let Some(Value::Text(name)) = &req.param else {
                    return Execution::Failed("select takes a theme name".to_owned());
                };
                if theme::resolve(name).is_none() {
                    return Execution::Failed(format!("no theme {name}"));
                }
                asks.theme = Some(name.clone());
                outcome
                    .result
                    .insert("theme".to_owned(), Value::Text(name.clone()));
            }
            ("toggle", target) if *target == ids.detach => {
                asks.detach = true;
                // what was asked; the shell opens or closes the window
                outcome
                    .result
                    .insert("on".to_owned(), Value::Bool(!self.view.detached));
            }
            ("activate", target) if *target == ids.save => asks.save = true,
            ("activate", target) if *target == ids.show => asks.show = true,
            ("activate", target) if *target == ids.print => {
                self.shell.print.asked = Some(id);
                return Execution::Started;
            }
            _ => {
                return Execution::Failed(format!(
                    "the shell has no {} on {}",
                    req.action, req.target
                ));
            }
        }
        Execution::Done(outcome)
    }
}

impl ShowcaseView {
    fn shell_view(&self) -> View<'_> {
        View {
            theme_name: &self.theme_name,
            page: self.page,
            detached: self.panes.detached.is_some(),
        }
    }

    /// The shell projected now, alone: what a key of the shell observes.
    pub fn shell_semantics(&self) -> Result<Snapshot, SemErrors> {
        let mut b = SnapshotBuilder::new();
        project(&self.panes.shell, self.shell_view(), &mut b);
        b.finish()
    }

    /// The gallery projected now: the shell and the active page (with its
    /// own window while it has one), one snapshot. The agent port serves
    /// it and the menu is built from it. The page's nodes come in as
    /// [`ShowcaseView::semantics`] projects them, but for their `source`
    /// (see `detached::extend`); a page without a projection leaves the
    /// shell alone.
    pub fn gallery_semantics(&self) -> Result<Snapshot, SemErrors> {
        let mut b = SnapshotBuilder::new();
        project(&self.panes.shell, self.shell_view(), &mut b);
        if let Some(page) = self.semantics() {
            detached::extend(&mut b, &page);
        }
        b.finish()
    }

    /// Whether `id` is the shell's (`shell` or one of its capabilities).
    pub fn is_shell(&self, id: &SemId) -> bool {
        self.panes.shell.holds(id)
    }

    /// Runs `req` through the gallery's dispatcher, over the shell.
    fn shell_dispatch(&mut self, req: Request) -> RequestId {
        let view = View {
            theme_name: &self.theme_name,
            page: self.page,
            detached: self.panes.detached.is_some(),
        };
        let todo = &mut self.panes.todo;
        let mut op = ShellOp {
            shell: &mut self.panes.shell,
            view,
        };
        todo.with_dispatcher(|d, _, now| d.dispatch(&mut op, req, now))
    }

    /// A request aimed at the shell, from any actor (a key, a menu item,
    /// a screen reader, an agent), through the gallery's one dispatcher:
    /// the receipt as it stands, with a page or a theme it asked for
    /// applied.
    pub(super) fn act_shell(&mut self, req: Request) -> Option<Receipt> {
        let id = self.shell_dispatch(req);
        if let Some(index) = self.panes.shell.asks.page.take() {
            self.set_page(index);
        }
        if let Some(name) = self.panes.shell.asks.theme.take() {
            self.set_theme(&name);
        }
        // a print started: the scene is built now, the job runs beside
        // the loop, and the request waits for it (`print.rs`)
        if let Some(print) = self.panes.shell.print.asked.take() {
            self.start_print(print);
        }
        self.panes.todo.receipt(id).cloned()
    }

    /// The save keys (cmd-shift-s): the person asks `shell.save_session`,
    /// projected while the session is recorded. `true` when it ran; the
    /// shell then takes the ask (`take_save_request`).
    pub fn request_save(&mut self) -> bool {
        self.ask_shell(ShellTarget::Save, "activate", None)
    }

    /// A person's ask of a shell capability (a key, the theme page's
    /// cards, the sidebar): observed on the shell as it is now, by the
    /// person, through the dispatcher. `true` when it ran to the end.
    pub(crate) fn ask_shell(
        &mut self,
        target: ShellTarget,
        action: &str,
        param: Option<Value>,
    ) -> bool {
        self.ask_shell_receipt(target, action, param)
            .is_some_and(|r| matches!(r.stage(), Some(Stage::Completed(_))))
    }

    /// [`ask_shell`](Self::ask_shell) with the receipt as it stands after
    /// the dispatch (a print is `Executing` then); `None` when the ask
    /// never became a request.
    pub(crate) fn ask_shell_receipt(
        &mut self,
        target: ShellTarget,
        action: &str,
        param: Option<Value>,
    ) -> Option<Receipt> {
        let target = self.panes.shell.id(target)?;
        let seen = match self.shell_semantics() {
            Ok(seen) => seen,
            Err(e) => {
                log::warn!("shell: the projection does not validate: {e}");
                return None;
            }
        };
        let req = match Request::observe(&seen, Actor::Human, &target, action, param) {
            Ok(req) => req,
            Err(e) => {
                log::debug!("shell: {e}");
                return None;
            }
        };
        let receipt = self.act_shell(req);
        log::debug!(
            "shell: {target} {action}: {:?}",
            receipt.as_ref().and_then(Receipt::stage)
        );
        receipt
    }

    /// `D`, or the menu, asked for the page in a window of its own, or
    /// back: the shell, which owns the windows, takes the ask.
    pub fn take_detach_request(&mut self) -> bool {
        std::mem::take(&mut self.panes.shell.asks.detach)
    }

    /// cmd-shift-s, or the menu, asked for the recording's window to be
    /// saved: the shell, which owns the recording, takes the ask.
    pub fn take_save_request(&mut self) -> bool {
        std::mem::take(&mut self.panes.shell.asks.save)
    }

    /// The tray icon, its item or a notification asked for the main
    /// window to come forward: the shell, which owns the windows, takes
    /// the ask.
    pub fn take_show_request(&mut self) -> bool {
        std::mem::take(&mut self.panes.shell.asks.show)
    }

    /// What the shell told the person by notification on its own (a job
    /// that ended while no window had the focus), projected as
    /// `shell:notice` for an agent to read.
    pub fn set_notice(&mut self, text: &str) {
        self.panes.shell.notice = text.to_owned();
    }

    /// The shell says whether one of its windows has the focus; the pages
    /// that tell the person about what lands while nobody looks (the
    /// Files page) read it.
    pub fn set_window_focused(&mut self, on: bool) {
        self.panes.files.set_window_focused(on);
    }

    /// What the pages asked to tell the person since the shell last took
    /// it, in order; the shell shows it through the platform.
    pub fn take_notifications(&mut self) -> Vec<Notification> {
        self.panes.files.take_notifications()
    }

    /// The session is recorded (`PLEV_RECORD=1`): `shell.save_session`
    /// is projected while it is.
    pub fn set_recording(&mut self, on: bool) {
        self.panes.shell.recording = on;
    }

    /// A person clicked an item of the gallery's menu, of the tray, the
    /// tray icon itself or a notification: the request a click on the
    /// button makes, observed on the gallery as it is now, dispatched to
    /// the shell or the page it aims at (`act`). `true` when the gallery's
    /// projection moved (a frame is needed).
    pub fn menu_click(&mut self, clicked: &Clicked) -> bool {
        let seen = match self.gallery_semantics() {
            Ok(seen) => seen,
            Err(e) => {
                log::warn!("menu: the projection does not validate: {e}");
                return false;
            }
        };
        let req = match clicked.request(&seen) {
            Ok(req) => req,
            Err(e) => {
                log::debug!("menu: {e}");
                return false;
            }
        };
        let receipt = self.act(req);
        log::debug!(
            "menu: {} {}: {:?}",
            clicked.sem,
            clicked.action,
            receipt.as_ref().and_then(Receipt::stage)
        );
        self.gallery_semantics()
            .is_ok_and(|now| now.revision() != seen.revision())
    }
}

/// The frame on paper (task-76 item 9): `shell.print`, the job beside the
/// loop, and what is projected about it.
mod print;

#[cfg(test)]
mod tests;
