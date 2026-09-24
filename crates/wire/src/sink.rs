//! the port's sink on the app's dispatcher (`PortSink`): it hears every
//! stage of every receipt, keeps the ones of an agent's request until the
//! server forwards those its agents issued (`Heard`), and follows the
//! dispatcher's order for the looks its acts rest on (`Looks`).

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use sem::{Actor, Mark, Millis, Receipt, ReceiptSink, RequestId, Stage};

use crate::shown::Looks;

/// a stage the sink heard.
pub(crate) struct Heard {
    pub(crate) id: RequestId,
    pub(crate) stage: Stage,
    pub(crate) at: Millis,
}

/// the port's ear on the dispatcher: every stage of an agent's request,
/// kept until the port forwards the ones it issued, and the dispatcher's
/// order with the look each of its acts rests on (`Looks`).
pub(crate) struct PortSink {
    pub(crate) heard: Rc<RefCell<VecDeque<Heard>>>,
    pub(crate) looks: Rc<RefCell<Looks>>,
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
