//! The Claude asked for a surface off the workbench's thread (task-96),
//! the way the showcase asks it
//! (`crates/showcase/src/view/apps/surface_page/asking.rs`): the program
//! looked for without running it (`ClaudeCli::exists`: `claude`, or what
//! `PLEV_CLAUDE` names); a thread that runs `propose_verified` with the
//! `propose::Llm` over a clone of the `ClaudeCli` and reports each attempt
//! as it begins, then the answer with what the calls took and cost; and
//! `Pending`, the line's side of it: the request it will end
//! (`Execution::Started` was answered for it), the time gone by, the
//! attempt in course, and the `Cancel` that kills the child. Only a
//! `VerifiedIr` crosses the channel: the worker verifies, as
//! `propose_verified` always does.
//!
//! What the thread is given is only what the workbench scoped for the
//! Claude (`scope.rs`): never the workspace whole, and never the surface
//! shown (a surface of another intent can name what the review does not).

use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use claude_model::{Cancel, ClaudeCli};
use propose::{Ask, Llm, Model, ModelError, ProposeError, propose_verified};
use sem::{Actor, RequestId, Snapshot};
use uir::{Capabilities, GenerationPolicy, VerifiedIr};

/// How often the workbench looks at the channel while it waits.
pub(crate) const POLL: Duration = Duration::from_millis(100);

/// What the worker sends back.
enum Msg {
    /// Attempt `n` begins: a call of the program.
    Attempt(usize),
    /// The proposal ended (boxed: an answer holds an IR, an attempt a
    /// number).
    Done(Box<Answer>),
}

/// What asking the Claude came to.
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

/// The Claude the line may ask: the program, when it is there.
#[derive(Default)]
pub(crate) struct Claude {
    cli: Option<ClaudeCli>,
}

impl Claude {
    /// Looks for the program without running it.
    pub fn find() -> Claude {
        let cli = ClaudeCli::new();
        Claude {
            cli: cli.exists().then_some(cli),
        }
    }

    /// This program, or none (a test's fake, or the rules alone).
    pub fn with(cli: Option<ClaudeCli>) -> Claude {
        Claude { cli }
    }

    pub fn is_some(&self) -> bool {
        self.cli.is_some()
    }

    /// Starts asking for `intent` over `app` (the scope the workbench
    /// gives the Claude) and its registry `caps`, verified under
    /// `policy`, for the request `id`: a thread runs it and `Pending`
    /// follows it. The reason when it cannot start.
    pub fn start(
        &self,
        id: RequestId,
        intent: String,
        app: Snapshot,
        caps: Capabilities,
        policy: GenerationPolicy,
    ) -> Result<Pending, String> {
        let mut cli = self.cli.clone().ok_or("there is no claude to ask")?;
        cli.cancel = Cancel::new();
        let cancel = cli.cancel.clone();
        let rx = spawn(cli, intent.clone(), app, caps, policy.clone())?;
        Ok(Pending {
            id,
            intent,
            policy,
            started: Instant::now(),
            attempt: 0,
            shown_secs: 0,
            rx,
            cancel,
            cancelled: None,
        })
    }
}

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

/// A thread that runs the proposal and reports each attempt and the
/// answer on the channel returned; the reason when none could start.
fn spawn(
    mut cli: ClaudeCli,
    intent: String,
    app: Snapshot,
    caps: Capabilities,
    policy: GenerationPolicy,
) -> Result<Receiver<Msg>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("ide-surface-claude".to_owned())
        .spawn(move || {
            let started = Instant::now();
            let mut counted = Counted {
                cli: &mut cli,
                tx: &tx,
                n: 0,
            };
            let mut llm = Llm::new(&mut counted);
            let ask = Ask {
                intent: &intent,
                caps: &caps,
                app: &app,
                current: None,
                policy: &policy,
            };
            let proposed = propose_verified(&mut llm, &ask);
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
    Ok(rx)
}

/// A proposal the Claude is answering: the request that started, what it
/// was asked, how long it has been, the attempt in course.
pub(crate) struct Pending {
    pub id: RequestId,
    pub intent: String,
    /// The policy what comes back is verified under.
    pub policy: GenerationPolicy,
    started: Instant,
    attempt: usize,
    /// The whole seconds the line last showed (the status moves once a
    /// second, not every frame).
    shown_secs: u64,
    rx: Receiver<Msg>,
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
    /// and the proposal ends as a model error nobody reads any more.
    pub fn cancel(&mut self, by: Actor) {
        self.cancel.cancel();
        self.cancelled = Some(by);
    }

    /// What the worker sent since the last look: true when what the
    /// status shows moved (the attempt, the whole seconds), and the
    /// answer when the proposal ended. A worker gone without an answer is
    /// an answer too: the model failed.
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
