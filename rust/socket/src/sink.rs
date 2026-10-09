//! the port's sink on the app's dispatcher (`PortSink`): it hears every
//! stage of every receipt, keeps the ones of an agent's request until the
//! server forwards those its agents issued (`Heard`), and follows the
//! dispatcher's order for the looks its acts rest on (`Looks`). a stage
//! of a request the server waits on wakes the app's loop (`Waits`), so it
//! goes out in the next drain even when it ends outside one.

use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use accord::{Actor, Mark, Millis, Receipt, ReceiptSink, RequestId, Stage};

use crate::shown::Looks;

/// a stage the sink heard.
pub(crate) struct Heard {
    pub(crate) id: RequestId,
    pub(crate) stage: Stage,
    pub(crate) at: Millis,
}

/// the requests the server issued and still forwards, and the port's
/// wake (task-92). the port forwards stages only when the shell drains
/// it, and the shell drains after the port's wake: a stage of one of
/// these requests, heard after the act that dispatched it, calls the wake,
/// so the next drain sends it without the agent sending anything. that
/// covers every stage that ends outside a drain: a person's answer to a
/// confirmation (confirmed and what runs after it, a decline, a second
/// check that refuses), and an execution that ends later on a worker. a
/// server with no wake (driven by hand, in a test) only queues.
#[derive(Default)]
pub(crate) struct Waits {
    pub(crate) ids: RefCell<HashSet<RequestId>>,
    pub(crate) wake: RefCell<Option<Box<dyn Fn()>>>,
}

impl Waits {
    /// a stage of request `id` was heard: wakes the loop when the server
    /// waits on it.
    fn heard(&self, id: RequestId) {
        if !self.ids.try_borrow().is_ok_and(|ids| ids.contains(&id)) {
            return;
        }
        if let Ok(wake) = self.wake.try_borrow()
            && let Some(wake) = wake.as_ref()
        {
            wake();
        }
    }
}

/// the port's ear on the dispatcher: every stage of an agent's request,
/// kept until the port forwards the ones it issued, and the dispatcher's
/// order with the look each of its acts rests on (`Looks`).
pub(crate) struct PortSink {
    pub(crate) heard: Rc<RefCell<VecDeque<Heard>>>,
    pub(crate) looks: Rc<RefCell<Looks>>,
    pub(crate) waits: Rc<Waits>,
}

impl ReceiptSink for PortSink {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        if let Ok(mut looks) = self.looks.try_borrow_mut() {
            looks.heard(receipt, stage);
        }
        if !matches!(receipt.request.actor, Actor::Agent { .. }) {
            return;
        }
        let at = receipt.stages.last().map_or(0, |(_, at)| *at);
        if let Ok(mut heard) = self.heard.try_borrow_mut() {
            heard.push_back(Heard {
                id: receipt.id,
                stage: stage.clone(),
                at,
            });
        }
        self.waits.heard(receipt.id);
    }

    fn attached(&mut self, mark: Mark) {
        if let Ok(mut looks) = self.looks.try_borrow_mut() {
            looks.attached(mark);
        }
    }

    fn looked(&self, receipt: &Receipt) -> Option<Mark> {
        self.looks.try_borrow().ok()?.of(receipt.id)
    }
}
