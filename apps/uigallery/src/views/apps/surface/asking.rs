//! The Claude asked for a surface off the app's thread (task-69 phase
//! 3b-ii).
//!
//! `Claude::find` says whether the program is there without running it
//! (`ClaudeCli::exists`: `claude`, or what `PLEV_CLAUDE` names).
//! `Claude::start` clones the `ClaudeCli` and everything a `offers::Ask`
//! borrows (the intent, the todo's snapshot and registry, the surface
//! shown, the policy) into a thread that runs `propose_verified(Llm)` and
//! reports back through a channel: each attempt as it begins, then the
//! answer with what the calls took and cost. `Pending` is the page's
//! side of it: the request it will end (`Execution::Started` was
//! answered for it), the time gone by, the attempt in course, and the
//! `Cancel` that kills the child. Only `VerifiedIr` crosses the channel:
//! the worker verifies, as `propose_verified` always does.
//!
//! An ask made while the app sleeps (`engine::runtime::window::Background`, task-76
//! item 7) does not start a thread: `Pending` holds everything the thread
//! would take and starts it on the first look after the app wakes, its
//! time counted from then. A cancel meanwhile ends it without a child.
//! An answer that arrives while the app sleeps stays on the channel: the
//! page's `settle` reads nothing until the wake.
//!
//! With a surface shown, the same worker asks for a change of it
//! (task-70 phase 2, `Claude::start_change`): it runs
//! `transform_verified(Llm)` over the surface as its transformations left
//! it, and only operations that applied, each verified, cross the
//! channel; the page applies them through its host with the actor who
//! asked (`Pending::by`).
//!
//! Off the desktop (wasm, mobile) there is no program and no thread:
//! `find` finds nothing, the page offers only the rules, and nothing here
//! but the types is used.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use accord::{Actor, RequestId, Snapshot};
use offers::ProposeError;
use plevir::{Adapted, Capabilities, GenerationPolicy, Op, VerifiedIr};

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use claude::{Cancel, ClaudeCli};
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use engine::runtime::window::Background;

/// Set once a page is staged without a window (`shell::stage_page`): a
/// staged frame is the same on every machine, so from then on `find`
/// does not look for the program on the host and the page offers the
/// rules alone, as on a machine without it (the regress baselines, which
/// CI checks on runners without `claude`).
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
static STAGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Called by `shell::stage_page` before it builds a page.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(crate) fn stage_without_host() {
    STAGED.store(true, std::sync::atomic::Ordering::Relaxed);
}

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
    pub proposed: Result<Proposal, ProposeError>,
    /// The calls made, one per attempt.
    pub attempts: usize,
    /// From the first call to the answer.
    pub took: Duration,
    /// What the calls cost, as the child counts it; none when no call
    /// said.
    pub cost_usd: Option<f64>,
}

/// What the Claude answered: a surface, or the operations of a change,
/// every one applied and verified over the surface it was asked about.
// off the desktop there is no thread: nothing constructs an answer.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]
pub(crate) enum Proposal {
    Surface(Box<VerifiedIr>),
    Change(Vec<Op>),
}

/// What the Claude is asked for: a surface over the one shown (or none),
/// or a change of the one shown, asked by someone.
// off the desktop there is no thread: nothing reads what is asked.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]
pub(crate) enum Job {
    Propose(Box<Option<VerifiedIr>>),
    Change(Box<Adapted>),
}

/// Everything the worker's thread takes, owned: what an ask holds while
/// the app sleeps, and what the thread is started with.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
struct Held {
    cli: ClaudeCli,
    intent: String,
    app: Snapshot,
    caps: Capabilities,
    job: Job,
    policy: GenerationPolicy,
    by: Actor,
}

/// Where a proposal is: held for the app to wake, or running on its
/// thread.
// off the desktop there is no thread: nothing constructs a pending.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]
enum Work {
    /// Asked while the app slept: the thread starts on the first look
    /// after the wake.
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    Held(Box<Held>),
    /// The thread runs; what it sends comes here.
    Running(Receiver<Msg>),
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
    /// Nor on a staged page (`stage_without_host`).
    pub fn find() -> Claude {
        #[cfg(all(
            not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")),
            not(test)
        ))]
        let cli = {
            let cli = ClaudeCli::new();
            (!STAGED.load(std::sync::atomic::Ordering::Relaxed) && cli.exists()).then_some(cli)
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
    /// what `job` asks for (a surface over the one shown, or a change of
    /// it) and the policy it is verified under, by `by`, for the request
    /// `id`: a thread runs it and `Pending` follows it. While the app
    /// sleeps the thread is not started: the ask is held and starts on
    /// the first look after the wake. The reason when it cannot start (no
    /// program, no thread).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        id: RequestId,
        intent: String,
        app: Snapshot,
        caps: Capabilities,
        job: Job,
        policy: GenerationPolicy,
        by: Actor,
    ) -> Result<Pending, String> {
        let mut cli = self.cli.clone().ok_or("there is no claude to ask")?;
        cli.cancel = Cancel::new();
        let cancel = cli.cancel.clone();
        let held = Held {
            cli,
            intent: intent.clone(),
            app,
            caps,
            job,
            policy: policy.clone(),
            by: by.clone(),
        };
        let work = if Background::asleep() {
            Work::Held(Box::new(held))
        } else {
            Work::Running(spawn(held)?)
        };
        Ok(Pending {
            id,
            intent,
            policy,
            by,
            started: web_time::Instant::now(),
            attempt: 0,
            shown_secs: 0,
            work,
            cancel,
            cancelled: None,
        })
    }

    /// No thread and no program here: the request cannot start.
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        _id: RequestId,
        _intent: String,
        _app: Snapshot,
        _caps: Capabilities,
        _job: Job,
        _policy: GenerationPolicy,
        _by: Actor,
    ) -> Result<Pending, String> {
        Err("there is no claude to ask on this platform".to_owned())
    }
}

/// The worker: a thread that runs the proposal over `held` and reports
/// each attempt and the answer on the channel returned. The reason when
/// no thread could start.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn spawn(held: Held) -> Result<Receiver<Msg>, String> {
    use std::sync::mpsc::Sender;

    use offers::{
        Adapt, Ask, Llm, Model, ModelError, Transformed, propose_verified, transform_verified,
    };

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

    let Held {
        mut cli,
        intent,
        app,
        caps,
        job,
        policy,
        by,
    } = held;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("surface-page-claude".to_owned())
        .spawn(move || {
            let started = web_time::Instant::now();
            let mut counted = Counted {
                cli: &mut cli,
                tx: &tx,
                n: 0,
            };
            let mut llm = Llm::new(&mut counted);
            let proposed = match &job {
                Job::Propose(current) => {
                    let ask = Ask {
                        intent: &intent,
                        caps: &caps,
                        app: &app,
                        current: current.as_ref().as_ref(),
                        policy: &policy,
                    };
                    propose_verified(&mut llm, &ask).map(|ir| Proposal::Surface(Box::new(ir)))
                }
                Job::Change(current) => {
                    let ask = Adapt {
                        intent: &intent,
                        caps: &caps,
                        app: &app,
                        current,
                        policy: &policy,
                        by: &by,
                    };
                    transform_verified(&mut llm, &ask).and_then(|done| match done {
                        Transformed::Changed { ops, .. } => Ok(Proposal::Change(ops)),
                        Transformed::Undo => Err(ProposeError::NoChange(
                            "the Claude answered no operation".to_owned(),
                        )),
                    })
                }
            };
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
// off the desktop there is no thread: nothing constructs a pending.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]
pub(crate) struct Pending {
    pub id: RequestId,
    pub intent: String,
    /// The policy what comes back is verified under.
    pub policy: GenerationPolicy,
    /// Who asked: a change is applied as theirs.
    pub by: Actor,
    started: web_time::Instant,
    attempt: usize,
    /// The whole seconds the page last showed (the status moves once a
    /// second, not every frame).
    shown_secs: u64,
    work: Work,
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

    /// Asked while the app slept and not started yet.
    #[cfg(all(
        test,
        not(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))
    ))]
    pub fn held(&self) -> bool {
        matches!(self.work, Work::Held(_))
    }

    /// Raises the signal: the child is killed at the worker's next poll
    /// (within 10 ms) and the proposal ends as a model error nobody
    /// reads any more; an ask still held never starts. `by` is who asked.
    pub fn cancel(&mut self, by: Actor) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        self.cancel.cancel();
        self.cancelled = Some(by);
    }

    /// A held ask starts once the app is awake, its time counted from
    /// now; while the app sleeps nothing moves. The answer when no thread
    /// could start.
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    fn start_held(&mut self) -> Option<Answer> {
        if !matches!(self.work, Work::Held(_)) || Background::asleep() {
            return None;
        }
        let (_, idle) = std::sync::mpsc::channel();
        let Work::Held(held) = std::mem::replace(&mut self.work, Work::Running(idle)) else {
            return None;
        };
        self.started = web_time::Instant::now();
        match spawn(*held) {
            Ok(rx) => {
                self.work = Work::Running(rx);
                None
            }
            Err(why) => Some(Answer {
                proposed: Err(ProposeError::Model(why)),
                attempts: 0,
                took: Duration::ZERO,
                cost_usd: None,
            }),
        }
    }

    /// What the worker sent since the last look: true when what the
    /// status shows moved (the attempt, the whole seconds), and the
    /// answer when the proposal ended. A worker gone without an answer
    /// is an answer too: the model failed. An ask held for the wake
    /// starts here once the app is awake, and moves nothing before.
    pub fn poll(&mut self) -> (bool, Option<Answer>) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(answer) = self.start_held() {
            return (true, Some(answer));
        }
        // Where nothing is held (web, mobile) the match has one arm.
        #[cfg_attr(
            any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
            allow(clippy::infallible_destructuring_match)
        )]
        let rx = match &self.work {
            Work::Running(rx) => rx,
            // asleep still: the ask waits, nothing moves
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            Work::Held(_) => return (false, None),
        };
        let mut moved = false;
        loop {
            match rx.try_recv() {
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
