//! What the person at the screen sees of the requests and answers: a
//! toast when the dispatcher or the index refused a request or git failed
//! it, and the confirmation modal for a request another actor made that
//! waits for a person (`Confirm::ForAgents`, or an agent's question,
//! `Request::ask`). Confirming goes through `Dispatcher::confirm` as
//! `Actor::Human`, and closing the modal any other way declines through
//! `Dispatcher::decline`; nothing here runs an action.
//!
//! When the app chose the platform's dialog (`Workbench::confirm_natively`,
//! task-76 item 8) the same title, body and button go to
//! `engine::platform::alerts` instead of the modal, the window as the
//! dialog's parent, and its answer takes the same two paths. The dialog
//! blocks this thread until the person answers, as every native panel
//! does; the receipt, the recording and the inspector see the same
//! stages as with the modal (`confirmed { by: human }`, or `declined`),
//! and no input, since no click reached the window.

use accord::{Actor, Request, RequestId, Stage};
use engine::runtime::platform::alerts::{self, Answered};

use super::Workbench;

impl Workbench {
    /// Shows the person what request `id` came to: a toast for a refusal,
    /// a decline or a failure, the confirmation modal (now, or once the
    /// modal on screen closes) for a request that waits for a person.
    pub(crate) fn report(&mut self, id: RequestId) {
        let Some(receipt) = self.dispatcher.receipt(id) else {
            return;
        };
        if receipt.stage() == Some(&Stage::AwaitingConfirmation) {
            if !self.awaiting.contains(&id) {
                self.awaiting.push_back(id);
            }
            self.ask_next_confirmation();
            return;
        }
        if let Some((message, intent)) = self.notice(receipt) {
            self.workspace.notify(message, intent);
        }
    }

    /// Opens the confirmation modal for the oldest request that still
    /// waits for a person, unless a modal already holds the screen; or
    /// asks the platform's dialog, when the app chose it, and answers
    /// right away.
    pub(super) fn ask_next_confirmation(&mut self) {
        if self.workspace.modal_open() {
            return;
        }
        while let Some(&id) = self.awaiting.front() {
            let texts = self
                .dispatcher
                .receipt(id)
                .filter(|r| r.stage() == Some(&Stage::AwaitingConfirmation))
                .map(|r| (self.confirmation(&r.request), self.destructive(&r.request)));
            match texts {
                Some(((title, body, confirm), destructive)) => {
                    if !self.ask_natively(id, &title, &body, confirm, destructive) {
                        self.workspace.open_confirm_modal(id, title, body, confirm);
                    }
                    return;
                }
                // Answered some other way meanwhile: nothing to ask.
                None => {
                    self.awaiting.pop_front();
                }
            }
        }
    }

    /// Asks the platform's dialog in the modal's place, when the app
    /// chose it: the same words, the window as parent, and the answer
    /// through `confirmed_by_person` or `declined_by_person`, which end
    /// the request, close nothing (no modal opened) and ask the next one
    /// waiting. `false` when the app did not choose it, or when this
    /// target has no native dialog: the modal is drawn then.
    fn ask_natively(
        &mut self,
        id: RequestId,
        title: &str,
        body: &str,
        confirm: &str,
        destructive: bool,
    ) -> bool {
        let answer = {
            let Some(native) = self.native_confirm.as_mut() else {
                return false;
            };
            let ask = alerts::Confirm::new(title, body)
                .labels(confirm, "Cancel")
                .destructive(destructive);
            native.alerts.confirm(&ask, native.parent.as_deref())
        };
        match answer {
            Ok(Answered::Confirmed) => self.confirmed_by_person(id),
            Ok(Answered::Declined) => self.declined_by_person(id),
            Err(e) => {
                log::warn!("ide: {e}; asking in the modal");
                return false;
            }
        }
        true
    }

    /// Whether the action `req` asks for is declared destructive on the
    /// node it aims at, as the workspace projects it now (a discard is,
    /// a stage is not); `false` when the node or the action is gone.
    fn destructive(&self, req: &Request) -> bool {
        self.snapshot().ok().is_some_and(|snap| {
            snap.get(&req.target).is_some_and(|node| {
                node.actions
                    .iter()
                    .any(|a| a.name == req.action && a.policy.destructive)
            })
        })
    }

    /// The person confirmed request `id` in the modal.
    pub(super) fn confirmed_by_person(&mut self, id: RequestId) {
        if let Err(e) = self.confirm(id, Actor::Human) {
            log::warn!("ide: confirming request {id}: {e}");
        }
    }

    /// The person closed the modal of request `id` without confirming
    /// (Cancel, Escape, a click outside it): they declined it.
    pub(super) fn declined_by_person(&mut self, id: RequestId) {
        if let Err(e) = self.decline(id, Actor::Human) {
            log::warn!("ide: declining request {id}: {e}");
        }
    }
}
