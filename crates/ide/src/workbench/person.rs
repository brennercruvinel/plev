//! What the person at the screen sees of the requests and answers: a
//! toast when the dispatcher or the index refused a request or git failed
//! it, and the confirmation modal for a request another actor made that
//! waits for a person (`Confirm::ForAgents`). Confirming goes through
//! `Dispatcher::confirm` as `Actor::Human`; nothing here runs an action.

use sem::{Actor, RequestId, Stage};

use super::Workbench;

impl Workbench {
    /// Shows the person what request `id` came to: a toast for a refusal
    /// or a failure, the confirmation modal (now, or once the modal on
    /// screen closes) for a request that waits for a person.
    pub(super) fn report(&mut self, id: RequestId) {
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
    /// waits for a person, unless a modal already holds the screen.
    pub(super) fn ask_next_confirmation(&mut self) {
        if self.workspace.modal_open() {
            return;
        }
        while let Some(&id) = self.awaiting.front() {
            let texts = self
                .dispatcher
                .receipt(id)
                .filter(|r| r.stage() == Some(&Stage::AwaitingConfirmation))
                .map(|r| self.confirmation(&r.request));
            match texts {
                Some((title, body, confirm)) => {
                    self.workspace.open_confirm_modal(id, title, body, confirm);
                    return;
                }
                // Confirmed some other way meanwhile: nothing to ask.
                None => {
                    self.awaiting.pop_front();
                }
            }
        }
    }

    /// The person confirmed request `id` in the modal.
    pub(super) fn confirmed_by_person(&mut self, id: RequestId) {
        if let Err(e) = self.confirm(id, Actor::Human) {
            log::warn!("ide: confirming request {id}: {e}");
        }
    }

    /// The person closed the modal of request `id` without confirming.
    /// `sem` has no way to decline yet, so the receipt stays awaiting
    /// confirmation; the workbench stops asking and says so.
    pub(super) fn declined_by_person(&mut self, id: RequestId) {
        self.awaiting.retain(|r| *r != id);
        if let Some(receipt) = self.dispatcher.receipt(id) {
            let message = self.unconfirmed(&receipt.request);
            self.workspace
                .notify(message, engine::theme::Intent::Informational);
        }
        self.ask_next_confirmation();
    }
}
