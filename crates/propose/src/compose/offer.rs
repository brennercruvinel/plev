//! `Composer`: the detector of one app, what it offers the person, and
//! what the person did with the offers, for one session.
//!
//! nothing installs by itself: an offer becomes a composite only when
//! the app runs the person's own request to accept it (`accept`); a
//! declined sequence is never offered again in the session. everything
//! lives in memory and ends with the session.

use std::sync::{Arc, Mutex, PoisonError};

use sem::{ActionName, Request, RequestId, SemId, Snapshot, SnapshotBuilder};

use super::chain::Chain;
use super::composite::Composite;
use super::{Detector, Found, Listener, Shape};

/// the composites of one app for one session.
#[derive(Debug)]
pub struct Composer {
    detector: Arc<Mutex<Detector>>,
    offered: Option<Found>,
    declined: Vec<Shape>,
    installed: Vec<Composite>,
}

impl Composer {
    /// a composer whose detector finds a sequence repeated `repeats`
    /// times in a row.
    pub fn new(repeats: usize) -> Composer {
        Composer {
            detector: Arc::new(Mutex::new(Detector::new(repeats))),
            offered: None,
            declined: Vec::new(),
            installed: Vec::new(),
        }
    }

    /// the sink that hears the app's dispatcher for this composer's
    /// detector (`Dispatcher::add_sink`).
    pub fn listener(&self) -> Listener {
        Listener::new(self.detector.clone())
    }

    fn detector(&self) -> std::sync::MutexGuard<'_, Detector> {
        self.detector.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// reads the detector again and says what stands offered: the
    /// sequence found last, when `admit` takes it (the app offers only
    /// what it can declare as its own) and it was neither declined nor
    /// installed in this session. an offer stays until the person
    /// answers it, or another sequence is found.
    pub fn offer(&mut self, admit: impl Fn(&Shape) -> bool) -> Option<&Found> {
        let found = self.detector().found();
        if let Some(found) = found {
            let answered = self.declined.contains(&found.shape)
                || self.installed.iter().any(|c| c.shape == found.shape);
            if !answered && admit(&found.shape) {
                self.offered = Some(found);
            }
        }
        self.offered.as_ref()
    }

    /// the offer standing, without reading the detector.
    pub fn offered(&self) -> Option<&Found> {
        self.offered.as_ref()
    }

    /// the person said not now: the offer goes, and its sequence is not
    /// offered again in this session. the sequence declined.
    pub fn decline(&mut self) -> Option<Shape> {
        let found = self.offered.take()?;
        self.declined.push(found.shape.clone());
        Some(found.shape)
    }

    /// the person accepted the offer: its sequence becomes a composite of
    /// the app, named from what `app` declares, and the detector knows
    /// its action from now on (a request for it runs steps, which are not
    /// something its actor did by hand).
    pub fn accept(&mut self, app: &Snapshot) -> Result<&Composite, String> {
        let found = self
            .offered
            .as_ref()
            .ok_or("nothing is offered: there is no sequence to install")?;
        let taken: Vec<ActionName> = self.installed.iter().map(|c| c.name.clone()).collect();
        let composite = Composite::new(found.shape.clone(), app, &taken)?;
        self.offered = None;
        self.detector().composite(composite.name.clone());
        self.installed.push(composite);
        self.installed
            .last()
            .ok_or_else(|| "the composite was not kept".to_owned())
    }

    /// the composites installed in this session, in the order accepted.
    pub fn installed(&self) -> &[Composite] {
        &self.installed
    }

    /// every composite installed, declared on the nodes it fits, after
    /// the app's own projection (`Composite::declare`).
    pub fn declare(&self, b: &mut SnapshotBuilder) {
        for composite in &self.installed {
            composite.declare(b);
        }
    }

    /// the composite installed that `action` on `home` asks for.
    pub fn composite(&self, home: &SemId, action: &ActionName) -> Option<&Composite> {
        self.installed.iter().find(|c| c.is(home, action))
    }

    /// the chain of the request `id` when it asks for a composite
    /// installed here, on a node it is declared on: what the app's
    /// `execute` keeps, answering `Execution::Started`, and advances right
    /// after the dispatch (`Chain::advance`).
    pub fn start(&self, id: RequestId, req: &Request) -> Option<Chain> {
        let composite = self.composite(&req.target, &req.action)?;
        let calls = composite.calls(&req.target)?;
        Some(Chain::new(id, req.actor.clone(), calls))
    }

    /// how many times in a row a sequence is repeated before it is
    /// offered.
    pub fn repeats(&self) -> usize {
        self.detector().repeats()
    }
}
