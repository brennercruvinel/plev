//! The view's own layer over the page, projected (task-75 phase 2a): the
//! toasts on screen, and the modal and the context menu the Dialog and
//! Context Menu pages open. The comps widgets declare themselves
//! (`ToastManager::project`, `Modal::project`, `ContextMenu::project`)
//! and bind their rects under the ids the view gives them (`toasts`,
//! `dialog`, `context_menu`); the view adds them to its projection
//! (`ShowcaseView::semantics`) while they are on screen, so an agent
//! through the port reads them as a screen reader hears them: a toast is
//! a status (an alert when destructive), a live region that never takes
//! the focus; the modal is a dialog, the menu a menu.
//!
//! While the modal or the menu holds the pointer the page beneath is
//! inert, and the focus is the overlay's: the dialog's cancel (the one
//! that changes nothing) or the menu's first item, or the one a reader
//! moved it to. The page keeps its own focus meanwhile, and it is the
//! projection's focus again when the overlay closes.
//!
//! What the overlays do runs through the gallery's one dispatcher (the
//! todo page's, lent as to the shell), `Overlays` being their `Operable`:
//! a person's click on a button of the modal or an item of the menu, a
//! screen reader's press and an agent's act are the same request, and a
//! toast's `dismiss` is how a reader or an agent takes one away.

use accord::{
    Actor, Execution, Operable, Outcome, Receipt, Request, RequestId, SemErrors, SemId, Semantics,
    Snapshot, SnapshotBuilder, Value,
};
use engine::styling::theme::{Intent, Theme};
use engine::widgets::overlay::{OverlayId, OverlayManager};
use engine::widgets::prelude::{ContextMenu, MenuEntry, ModalAction, ToastManager};

use engine::widgets::form::ClipKey;

use super::super::components::context_menu;
use super::super::text_input::field_menu;
use super::super::{ActiveOverlay, ShowcaseView};

/// The ids the view gives its overlays.
pub(in crate::views) const TOASTS: &str = "toasts";
pub(in crate::views) const DIALOG: &str = "dialog";
pub(in crate::views) const MENU: &str = "context_menu";

/// The id of one of the overlays; `None` (logged) if the name does not
/// parse, which a test proves it does.
pub(in crate::views) fn id(name: &str) -> Option<SemId> {
    SemId::name(name)
        .inspect_err(|e| log::warn!("overlay: {name}: {e}"))
        .ok()
}

/// Whether `active` holds the pointer: on the stack and not leaving.
pub(in crate::views) fn holds(mgr: &OverlayManager, active: &ActiveOverlay) -> bool {
    let id = active.id();
    mgr.stack.iter().any(|o| o.id == id && !o.is_closing())
}

/// The key of `target` under `parent` (`toasts[3]` is 3 under `toasts`).
fn key_under<'a>(target: &'a SemId, parent: &SemId) -> Option<&'a str> {
    target
        .as_str()
        .strip_prefix(parent.as_str())?
        .strip_prefix('[')?
        .strip_suffix(']')
}

/// The menu's items a person can pick, in order, as ids.
fn items(menu: &ContextMenu, id: &SemId) -> Vec<SemId> {
    menu.entries
        .iter()
        .filter_map(|e| match e {
            MenuEntry::Item {
                id: item,
                disabled: false,
                ..
            } => Some(id.key(item.to_string())),
            _ => None,
        })
        .collect()
}

/// The overlays as the dispatcher sees them: they project from the view's
/// parts and run what is asked of them.
struct Overlays<'a> {
    toasts: &'a mut ToastManager,
    mgr: &'a mut OverlayManager,
    active: &'a mut Option<ActiveOverlay>,
    focus: Option<&'a SemId>,
    theme: &'a Theme,
    /// A text field's menu picked: the clipboard key the view runs on
    /// the field once the dispatch is over (`field_menu.rs`).
    clip: &'a mut Option<ClipKey>,
}

/// The toasts on screen and the overlay that holds the pointer, into `b`,
/// with the overlay's focus.
fn project(
    toasts: &ToastManager,
    mgr: &OverlayManager,
    active: Option<&ActiveOverlay>,
    focus: Option<&SemId>,
    b: &mut SnapshotBuilder,
) {
    if let Some(id) = toasts.sem_id()
        && toasts.visible().any(|t| !t.is_closing())
    {
        toasts.project(b, id, "notifications", None);
    }
    let Some(active) = active.filter(|a| holds(mgr, a)) else {
        return;
    };
    match active {
        ActiveOverlay::Modal { widget, .. } => {
            let Some(id) = widget.sem_id() else {
                return;
            };
            widget.project(b, id, None);
            let buttons = [id.key("confirm"), id.key("cancel")];
            let at = focus.filter(|f| buttons.contains(f));
            b.focus(at.unwrap_or(&buttons[1]));
        }
        ActiveOverlay::Menu { widget, .. } => {
            let Some(id) = widget.sem_id() else {
                return;
            };
            widget.project(b, id, "context menu", None);
            let items = items(widget, id);
            if let Some(at) = focus.filter(|f| items.contains(f)).or(items.first()) {
                b.focus(at);
            }
        }
    }
}

impl Semantics for Overlays<'_> {
    fn project(&self, b: &mut SnapshotBuilder) {
        project(self.toasts, self.mgr, self.active.as_ref(), self.focus, b);
    }
}

impl Overlays<'_> {
    /// The modal answered: it leaves, and a confirm says so in a toast
    /// (what the Dialog page's demo did on a click).
    fn answer_modal(&mut self, overlay: OverlayId, confirm: bool, intent: Intent) {
        self.mgr.pop_id_animated(overlay);
        if !confirm {
            return;
        }
        if intent == Intent::Destructive {
            let text = "Repository deleted (not really).";
            self.toasts.push(text, Intent::Destructive, self.theme);
        } else {
            self.toasts
                .push("Settings applied.", Intent::Constructive, self.theme);
        }
    }
}

/// What a request asks of the overlay that holds the pointer.
enum Asked {
    /// The modal answered: confirmed or not.
    Answer(OverlayId, Intent, bool),
    /// The menu closes with nothing picked.
    Close(OverlayId),
    /// An item of the menu, picked.
    Pick(OverlayId, u64),
}

impl Overlays<'_> {
    /// What `req` asks of the active overlay, if anything.
    fn asked(&self, req: &Request) -> Option<Asked> {
        let action = req.action.as_str();
        match self.active.as_ref()? {
            ActiveOverlay::Modal { id, widget } => {
                let dialog = widget.sem_id()?;
                let button = key_under(&req.target, dialog);
                let confirm = match (req.target == *dialog, button, action) {
                    (true, _, "confirm") | (false, Some("confirm"), "activate") => true,
                    (true, _, "cancel") | (false, Some("cancel"), "activate") => false,
                    _ => return None,
                };
                Some(Asked::Answer(*id, widget.intent, confirm))
            }
            ActiveOverlay::Menu { id, widget, .. } => {
                let menu = widget.sem_id()?;
                if req.target == *menu && action == "close" {
                    return Some(Asked::Close(*id));
                }
                let item = key_under(&req.target, menu)?.parse::<u64>().ok()?;
                (action == "activate").then_some(Asked::Pick(*id, item))
            }
        }
    }
}

impl Operable for Overlays<'_> {
    /// Everything ends now: a toast leaves, the modal answers, an item of
    /// the menu is picked, the menu closes.
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let action = req.action.as_str();
        let mut outcome = Outcome::default();
        let toast = self
            .toasts
            .sem_id()
            .and_then(|t| key_under(&req.target, t))
            .and_then(|k| k.parse::<u64>().ok());
        if let (Some(toast), "dismiss") = (toast, action) {
            if !self.toasts.dismiss_id(toast) {
                return Execution::Failed(format!("{} is not on screen", req.target));
            }
            return Execution::Done(outcome);
        }
        match self.asked(req) {
            Some(Asked::Answer(overlay, intent, confirm)) => {
                self.answer_modal(overlay, confirm, intent);
                outcome
                    .result
                    .insert("confirmed".to_owned(), Value::Bool(confirm));
            }
            Some(Asked::Close(overlay)) => self.mgr.pop_id_animated(overlay),
            // A text field's menu: the clipboard is the person's, so an
            // agent's pick is refused and a person's (pointer, keys, a
            // screen reader) is the field's clipboard key.
            Some(Asked::Pick(overlay, item)) if field_menu::clip_of(item).is_some() => {
                if !matches!(req.actor, Actor::Human | Actor::Assistive) {
                    return Execution::Failed(format!(
                        "{}: the clipboard is the person's, never an agent's",
                        req.target
                    ));
                }
                let key = field_menu::clip_of(item);
                self.mgr.pop_id_animated(overlay);
                *self.clip = key;
                let label = key.map_or("", field_menu::label);
                outcome
                    .result
                    .insert("item".to_owned(), Value::Text(label.to_owned()));
            }
            Some(Asked::Pick(overlay, item)) => {
                let label = context_menu::menu_label(item);
                self.mgr.pop_id_animated(overlay);
                self.toasts.push(
                    format!("Context menu: {label}"),
                    Intent::Informational,
                    self.theme,
                );
                outcome
                    .result
                    .insert("item".to_owned(), Value::Text(label.to_owned()));
            }
            None => {
                return Execution::Failed(format!("no overlay has {action} on {}", req.target));
            }
        }
        Execution::Done(outcome)
    }
}

impl ShowcaseView {
    /// Whether a toast is on screen, or an overlay holds the pointer: the
    /// view's projection carries them then.
    pub(in crate::views) fn overlays_shown(&self) -> bool {
        self.toasts.visible().any(|t| !t.is_closing()) || self.overlay_holds_pointer()
    }

    /// The toasts on screen and the overlay that holds the pointer, into
    /// `b`, after the page.
    pub(in crate::views) fn project_overlays(&self, b: &mut SnapshotBuilder) {
        project(
            &self.toasts,
            &self.overlay_mgr,
            self.active_overlay.as_ref(),
            self.overlay_focus.as_ref(),
            b,
        );
    }

    /// The overlays projected now, alone: what a person's click on one
    /// observes.
    fn overlay_semantics(&self) -> Result<Snapshot, SemErrors> {
        let mut b = SnapshotBuilder::new();
        self.project_overlays(&mut b);
        b.finish()
    }

    /// Whether `id` is one of the overlays' (a toast, the dialog or one of
    /// its buttons, the menu or one of its items), on screen or not.
    pub fn is_overlay(&self, id: &SemId) -> bool {
        let modal = match &self.active_overlay {
            Some(ActiveOverlay::Modal { widget, .. }) => widget.sem_id(),
            Some(ActiveOverlay::Menu { widget, .. }) => widget.sem_id(),
            None => None,
        };
        [self.toasts.sem_id(), modal]
            .into_iter()
            .flatten()
            .any(|at| at == id || key_under(id, at).is_some())
    }

    /// A request aimed at an overlay, from any actor, through the
    /// gallery's one dispatcher: the receipt as it stands.
    pub(in crate::views) fn act_overlay(&mut self, req: Request) -> Option<Receipt> {
        let mut clip = None;
        let mut op = Overlays {
            toasts: &mut self.toasts,
            mgr: &mut self.overlay_mgr,
            active: &mut self.active_overlay,
            focus: self.overlay_focus.as_ref(),
            theme: &self.theme,
            clip: &mut clip,
        };
        let id = self
            .panes
            .todo
            .with_dispatcher(|d, _, now| d.dispatch(&mut op, req, now));
        if let Some(key) = clip {
            self.clip_to(Some(key));
        }
        self.panes.todo.receipt(id).cloned()
    }

    /// A person's click on a button of the modal or an item of the menu
    /// (`target`, `action`): observed on the overlays as they are now, by
    /// the person, through the dispatcher. `true` when it ran.
    pub(in crate::views) fn ask_overlay(&mut self, target: &SemId, action: &str) -> bool {
        let seen = match self.overlay_semantics() {
            Ok(seen) => seen,
            Err(e) => {
                log::warn!("overlay: the projection does not validate: {e}");
                return false;
            }
        };
        let req = match Request::observe(&seen, Actor::Human, target, action, None) {
            Ok(req) => req,
            Err(e) => {
                log::debug!("overlay: {e}");
                return false;
            }
        };
        let receipt = self.act_overlay(req);
        log::debug!(
            "overlay: {target} {action}: {:?}",
            receipt.as_ref().and_then(Receipt::stage)
        );
        receipt.is_some()
    }

    /// Keyboard focus from a screen reader to a button of the modal or an
    /// item of the menu that holds the pointer: the overlay keeps it and
    /// projects it. `false` for anything else of the overlays.
    pub(in crate::views) fn focus_overlay(&mut self, id: &SemId) -> bool {
        let Ok(seen) = self.overlay_semantics() else {
            return false;
        };
        let takes = seen.get(id).is_some_and(|n| n.focusable && n.enabled);
        if takes {
            self.overlay_focus = Some(id.clone());
            self.show_overlay_focus_on(id);
        }
        takes
    }

    /// The focus a reader or the keyboard moved into the overlay, drawn:
    /// the ring on the modal's button (task-75 phase 2c), the highlight
    /// on the menu's item, which the pointer's hover shares.
    fn show_overlay_focus_on(&mut self, id: &SemId) {
        match &mut self.active_overlay {
            Some(ActiveOverlay::Modal { widget, .. }) => {
                let button = widget.sem_id().and_then(|dialog| key_under(id, dialog));
                widget.set_focused(match button {
                    Some("confirm") => ModalAction::Confirm,
                    Some("cancel") => ModalAction::Cancel,
                    _ => ModalAction::None,
                });
            }
            Some(ActiveOverlay::Menu { widget, .. }) => {
                let item = widget
                    .sem_id()
                    .and_then(|menu| key_under(id, menu))
                    .and_then(|key| key.parse::<u64>().ok());
                widget.set_highlighted(item);
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests;
