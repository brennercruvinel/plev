//! Deep links in the gallery (task-76 item 6): the scheme
//! `plev-showcase` ([`SCHEME`]), `plev-showcase://<sem id>/<action>
//! [?value=<text>]`, read by `engine::platform::links`.
//!
//! A link comes from outside the window and from nobody the gallery
//! knows: a web page, a mail, another app, someone at a terminal. So it
//! is never the person's request: it asks as an agent named `link`
//! (`engine::platform::links::actor`), over the gallery as it projects
//! now, through the gallery's one dispatcher, and only for what [`OPEN`]
//! lists, going to a page and changing the theme. Anything else (the
//! camera, the print, the Files dialog, the page's own window, a todo
//! removed) is refused before any request is made; inside the list the
//! dispatcher still checks it as it checks an agent (a capability for a
//! person alone is `NotAuthorized`, a value out of the options is
//! refused, a confirmation waits for a person).
//!
//! Whatever is refused, nothing runs and the person is told in one line,
//! `link refused: <link>: <why>`, as a toast on screen and as
//! `shell:notice` for an agent to read; the log says it too.
//!
//! Where links come from (`engine::platform::links::default_links`): the
//! launch argument on every desktop, read once the main window is up (a
//! refusal has a window to show in), `plev-showcase://...` as an argument
//! being a link and never the page title (`App::launch_view`); on macOS
//! the system's own delivery waits for the bundle (the bring-up, in
//! docs/ADR/a-link-is-not-a-person.md). The shell polls on its loop: at
//! the launch, when the source wakes it (`UserEvent::Link`), after a
//! wake. Asleep, nothing is polled: a link waits in its source and is
//! applied on the first poll awake. A recorded session keeps the link's
//! look at the gallery and its receipt's stages, as an agent's; no input.

use accord::{Receipt, Rejection, Revision, Stage};
use engine::runtime::platform::links::{Delivered, LinkSource, default_links};
use engine::runtime::window::Background;
use engine::styling::theme::Intent;

use super::super::App;
use crate::views::ShowcaseView;

/// The gallery's URL scheme.
pub const SCHEME: &str = "plev-showcase";

/// What a link may ask of the gallery, `(id, action)`: the page and the
/// theme. Nothing here writes, deletes, opens a device, a dialog, a file,
/// a window or a printer, or runs a process.
pub const OPEN: &[(&str, &str)] = &[("shell.page", "select"), ("shell.theme", "select")];

/// What a link came to.
#[derive(Debug, Default)]
pub struct Followed {
    /// The receipt of its request, when one was dispatched.
    pub receipt: Option<Receipt>,
    /// The line the person was shown, when it was refused, failed or
    /// waits for a person.
    pub told: Option<String>,
}

/// Follows one delivered link on `view`: refused, or asked of the
/// gallery's dispatcher by the agent `link` when `open` lists it, the
/// refusal told. `looked` hears the revision the link observed, right
/// before its request is dispatched (the recording keeps it).
pub fn follow(
    view: &mut ShowcaseView,
    delivered: Delivered,
    open: &[(&str, &str)],
    looked: &mut dyn FnMut(&mut ShowcaseView, Revision),
) -> Followed {
    let link = match delivered {
        Ok(link) => link,
        Err(refused) => return tell(view, format!("link refused: {refused}"), None),
    };
    let url = link.url(SCHEME);
    if !link.is_in(open) {
        let why = format!("{} {} is not open to links", link.target, link.action);
        return tell(view, format!("link refused: {url}: {why}"), None);
    }
    let seen = match view.gallery_semantics() {
        Ok(seen) => seen,
        Err(e) => return tell(view, format!("link refused: {url}: {e}"), None),
    };
    let req = match link.request(&seen) {
        Ok(req) => req,
        Err(e) => return tell(view, format!("link refused: {url}: {e}"), None),
    };
    looked(view, seen.revision());
    let Some(receipt) = view.act(req) else {
        let line = format!("link refused: {url}: nothing in the gallery takes it");
        return tell(view, line, None);
    };
    let line = match receipt.stage() {
        Some(Stage::Rejected(why)) => format!("link refused: {url}: {}", said(why)),
        Some(Stage::Failed { error }) => format!("link failed: {url}: {error}"),
        Some(Stage::AwaitingConfirmation) => {
            format!("link waits for a person to confirm it: {url}")
        }
        stage => {
            log::info!("link: {url}: {}", stage.map_or("none", Stage::name));
            return Followed {
                receipt: Some(receipt),
                told: None,
            };
        }
    };
    tell(view, line, Some(receipt))
}

/// Tells the person `line`: a toast on screen, `shell:notice` for an
/// agent, the log.
fn tell(view: &mut ShowcaseView, line: String, receipt: Option<Receipt>) -> Followed {
    log::warn!("{line}");
    view.set_notice(&line);
    let intent = match receipt.as_ref().and_then(Receipt::stage) {
        Some(Stage::AwaitingConfirmation) => Intent::Informational,
        _ => Intent::Destructive,
    };
    view.toasts.push(line.as_str(), intent, &view.theme);
    Followed {
        receipt,
        told: Some(line),
    }
}

/// Why the dispatcher refused, in the words of a refusal line.
fn said(why: &Rejection) -> String {
    match why {
        Rejection::NotFound { target } => format!("{target} is not there now"),
        Rejection::UnknownAction { target, action } => {
            format!("{target} does not offer {action} now")
        }
        Rejection::NotAuthorized { actor } => format!("{actor} may not ask this"),
        Rejection::Stale { .. } => "what it read changed before it ran".to_owned(),
        Rejection::Inert { target } => format!("{target} is behind an overlay now"),
        Rejection::Disabled { target, action } => format!("{target} {action} is disabled now"),
        Rejection::InvalidParam { reason } => reason.clone(),
        Rejection::InvalidProjection { .. } => {
            "the gallery's projection does not validate".to_owned()
        }
        Rejection::Declined { by } => format!("{by} declined it"),
    }
}

impl App {
    /// The platform's links for [`SCHEME`], woken through the loop's
    /// proxy; once, when the main window is up, so the launch argument is
    /// read then.
    pub(crate) fn install_links(&mut self) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        let wake = {
            let proxy = std::sync::Mutex::new(self.access.proxy.clone());
            move || {
                if let Ok(proxy) = proxy.lock()
                    && let Some(proxy) = proxy.as_ref()
                {
                    let _ = proxy.send_event(super::super::UserEvent::Link);
                }
            }
        };
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
        let wake = || {};
        self.set_links(default_links(SCHEME, wake));
    }

    /// Links come from `source` (the platform's, or a fake) from now on.
    pub(super) fn set_links(&mut self, source: Box<dyn LinkSource>) {
        self.links = Some(source);
    }

    /// The links delivered since the last poll, followed in order, and a
    /// frame for what they changed or told. Asleep, nothing: they wait in
    /// the source for the first poll awake.
    pub(crate) fn links_arrived(&mut self) {
        if Background::asleep() {
            return;
        }
        let Some(source) = self.links.as_mut() else {
            return;
        };
        let delivered = source.poll();
        if delivered.is_empty() {
            return;
        }
        for link in delivered {
            self.follow_link(link);
        }
        self.changed();
    }

    /// One link, its look at the gallery recorded when the session is.
    fn follow_link(&mut self, delivered: Delivered) -> Followed {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        let recorder = self
            .recording
            .as_ref()
            .map(super::super::window::record::Recording::recorder);
        let mut looked = |view: &mut ShowcaseView, revision: Revision| {
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            if let Some(recorder) = recorder {
                recorder.observed(
                    view.now(),
                    engine::runtime::platform::links::actor(),
                    None,
                    revision,
                );
            }
            #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
            let _ = (view, revision);
        };
        follow(&mut self.view, delivered, OPEN, &mut looked)
    }
}

#[cfg(test)]
mod tests;
