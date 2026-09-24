//! The Claude asked for a surface off the app's thread (task-69 phase
//! 3b-ii).
//!
//! `Claude::find` says whether the program is there without running it
//! (`ClaudeCli::exists`: `claude`, or what `PLEV_CLAUDE` names).
//! `Claude::start` clones the `ClaudeCli` and everything a `propose::Ask`
//! borrows (the intent, the todo's snapshot and registry, the surface
//! shown, the policy) into a thread that runs `propose_verified(Llm)` and
//! reports back through a channel: each attempt as it begins, then the
//! answer with what the calls took and cost. `Pending` is the page's
//! side of it: the request it will end (`Execution::Started` was
//! answered for it), the time gone by, the attempt in course, and the
//! `Cancel` that kills the child. Only `VerifiedIr` crosses the channel:
//! the worker verifies, as `propose_verified` always does.
//!
//! Off the desktop (wasm, mobile) there is no program and no thread:
//! `find` finds nothing, the page offers only the rules, and nothing here
//! but the types is used.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use propose::ProposeError;
use sem::{Actor, RequestId, Snapshot};
use uir::{Capabilities, GenerationPolicy, VerifiedIr};

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use claude_model::{Cancel, ClaudeCli};

/// How often the page looks at the channel while it waits.
pub(crate) const POLL: Duration = Duration::from_millis(100);

/// What the worker sends back.
// off the desktop there is no thread: nothing constructs a message.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]
enum Msg {
    /// Attempt `n` begins: a call of the program.
    Attempt(usize),
    /// The proposal ended (boxed: an answer holds an IR, an attempt a
    /// number).
    Done(Box<Answer>),
}

/// What asking the Claude came to.
// off the desktop there is no thread: nothing constructs an answer.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]
pub(crate) struct Answer {
    pub proposed: Result<VerifiedIr, ProposeError>,
    /// The calls made, one per attempt.
    pub attempts: usize,
    /// From the first call to the answer.
    pub took: Duration,
    /// What the calls cost, as the child counts it; none when no call
    /// said.
    pub cost_usd: Option<f64>,
}

/// The Claude the page may ask: the program, when it is there.
pub(crate) struct Claude {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    cli: Option<ClaudeCli>,
}

impl Claude {
    /// Looks for the program without running it. In the crate's own
    /// tests it is never looked for: no test of the gate reaches the
    /// real program, and a test that wants the fake sets it (`with`).
    pub fn find() -> Claude {
        #[cfg(all(
            not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")),
            not(test)
        ))]
        let cli = {
            let cli = ClaudeCli::new();
            cli.exists().then_some(cli)
        };
        #[cfg(all(
            not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")),
            test
        ))]
        let cli = None;
        Claude {
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            cli,
        }
    }

    /// This program (the fake in the tests, the real one in the proof).
    #[cfg(all(
        test,
        not(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))
    ))]
    pub fn with(cli: ClaudeCli) -> Claude {
        Claude { cli: Some(cli) }
    }

    /// True when there is a program to ask.
    pub fn is_some(&self) -> bool {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            self.cli.is_some()
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
        {
            false
        }
    }

    /// Starts asking for `intent` over the app as it is (`app`, `caps`),
    /// the surface shown (`current`) and the policy of whoever asked, for
    /// the request `id`: a thread runs the proposal and `Pending` follows
    /// it. The reason when it cannot start (no program, no thread).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub fn start(
        &self,
        id: RequestId,
        intent: String,
        app: Snapshot,
        caps: Capabilities,
        current: Option<VerifiedIr>,
        policy: GenerationPolicy,
    ) -> Result<Pending, String> {
        use std::sync::mpsc::{self, Sender};

        use propose::{Ask, Llm, Model, ModelError, propose_verified};

        /// The model as the worker runs it: says which attempt begins.
        struct Counted<'a> {
            cli: &'a mut ClaudeCli,
            tx: &'a Sender<Msg>,
            n: usize,
        }

        impl Model for Counted<'_> {
            fn complete(&mut self, prompt: &str) -> Result<String, ModelError> {
                self.n += 1;
                let _ = self.tx.send(Msg::Attempt(self.n));
                self.cli.complete(prompt)
            }
        }

        let mut cli = self.cli.clone().ok_or("there is no claude to ask")?;
        cli.cancel = Cancel::new();
        let cancel = cli.cancel.clone();
        let (tx, rx) = mpsc::channel();
        let asked = intent.clone();
        std::thread::Builder::new()
            .name("surface-page-claude".to_owned())
            .spawn(move || {
                let ask = Ask {
                    intent: &asked,
                    caps: &caps,
                    app: &app,
                    current: current.as_ref(),
                    policy: &policy,
                };
                let started = web_time::Instant::now();
                let mut counted = Counted {
                    cli: &mut cli,
                    tx: &tx,
                    n: 0,
                };
                let proposed = propose_verified(&mut Llm::new(&mut counted), &ask);
                let took = started.elapsed();
                let calls = cli.calls();
                let cost_usd = calls
                    .iter()
                    .any(|c| c.cost_usd.is_some())
                    .then(|| cli.spent_usd());
                let _ = tx.send(Msg::Done(Box::new(Answer {
                    proposed,
                    attempts: calls.len(),
                    took,
                    cost_usd,
                })));
            })
            .map_err(|e| format!("could not start a thread for the claude: {e}"))?;
        Ok(Pending {
            id,
            intent,
            started: web_time::Instant::now(),
            attempt: 0,
            shown_secs: 0,
            rx,
            cancel,
            cancelled: None,
        })
    }

    /// No thread and no program here: the request cannot start.
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    pub fn start(
        &self,
        _id: RequestId,
        _intent: String,
        _app: Snapshot,
        _caps: Capabilities,
        _current: Option<VerifiedIr>,
        _policy: GenerationPolicy,
    ) -> Result<Pending, String> {
        Err("there is no claude to ask on this platform".to_owned())
    }
}

/// A proposal the Claude is answering: the request that started, what it
/// was asked, how long it has been, the attempt in course.
// off the desktop there is no thread: nothing constructs a pending.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]
pub(crate) struct Pending {
    pub id: RequestId,
    pub intent: String,
    started: web_time::Instant,
    attempt: usize,
    /// The whole seconds the page last showed (the status moves once a
    /// second, not every frame).
    shown_secs: u64,
    rx: Receiver<Msg>,
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    cancel: Cancel,
    /// Who cancelled it, when someone did: the request ends failed, as
    /// cancelled by them.
    pub cancelled: Option<Actor>,
}

impl Pending {
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// The attempt in course, 1 to `Llm::ATTEMPTS`; 0 before the first
    /// call began.
    pub fn attempt(&self) -> usize {
        self.attempt
    }

    /// Raises the signal: the child is killed at the worker's next poll
    /// (within 10 ms) and the proposal ends as a model error nobody
    /// reads any more. `by` is who asked.
    pub fn cancel(&mut self, by: Actor) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        self.cancel.cancel();
        self.cancelled = Some(by);
    }

    /// What the worker sent since the last look: true when what the
    /// status shows moved (the attempt, the whole seconds), and the
    /// answer when the proposal ended. A worker gone without an answer
    /// is an answer too: the model failed.
    pub fn poll(&mut self) -> (bool, Option<Answer>) {
        let mut moved = false;
        loop {
            match self.rx.try_recv() {
                Ok(Msg::Attempt(n)) => {
                    self.attempt = n;
                    moved = true;
                }
                Ok(Msg::Done(answer)) => return (true, Some(*answer)),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let answer = Answer {
                        proposed: Err(ProposeError::Model(
                            "the worker ended without an answer".to_owned(),
                        )),
                        attempts: self.attempt,
                        took: self.elapsed(),
                        cost_usd: None,
                    };
                    return (true, Some(answer));
                }
            }
        }
        let secs = self.elapsed().as_secs();
        if secs != self.shown_secs {
            self.shown_secs = secs;
            moved = true;
        }
        (moved, None)
    }
}
