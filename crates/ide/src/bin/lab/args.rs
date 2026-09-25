//! the command line of the laboratory.

use std::path::PathBuf;
use std::time::Duration;

pub const USAGE: &str = "\
usage:
  ide-lab --arm plev|scripted|git|screenshot|review
          [--human none|unstage-after-observe:<path>|unstage-before-submit:<path>]
          [--out <dir>] [--until commit|timeout] [--timeout <s>] [--grace <s>] [--app <name>]
          [--scripted observe|chain|ask] [--answer none|allow|decline] [--answer-after <ms>]
arms:
  plev        the workbench on the agent port, for an agent outside (plev observe / plev act)
  scripted    the same, driven by the laboratory's own agent over the wire (the control run)
  git         the same repository for an agent that uses git itself: hook, shim, no port
  screenshot  the frame of the workbench as PNG at scale 1 and 2, beside the bytes of an observation
  review      the demo 5 of task-80 (task-96): on a repository of three commits, a person reviews
              them with the fixed panel (stacks and diff), then with the surface her intent
              generates beside it (\"review these three commits\"), and closes it; no port, no
              agent; recorded to review.monster and measured against the fixed panel
the person of the script acts once, through the dispatcher, when the agent has every file of
the task in the index and a message written:
  none                              nobody intervenes (default)
  unstage-after-observe:<path>      takes <path> out of the index right after the agent is served
                                    an observation: the person right after the agent looked (the
                                    scenario of the rounds of task-62)
  unstage-before-submit:<path>      takes <path> out of the index right before the agent's submit
                                    reaches the dispatcher: the person between the agent's decision
                                    and its submit, as the hook of the git arm fires at the commit
the same person answers every request of an agent that waits for a person (an agent's question,
plev act --ask, or an action that asks a person, as discard), --answer-after milliseconds after
the dispatcher left it waiting, with a click on the modal's button, as a person at the screen:
through the dispatcher as Actor::Human, recorded as her input (plev and scripted arms):
  none                              she never answers, and the request waits (default)
  allow                             she allows it: Allow in the modal (Discard for a discard)
  decline                           she declines it: Cancel in the modal
the scripted agent (--arm scripted):
  observe   observes the workspace right before each submit and submits on it (default)
  chain     submits the first time on the revision the last stage carried, with no
            observation before it; after a refusal it observes and decides again. a terminal
            stage shows no state (task-89): that submit rests on the message the agent was last
            shown, empty, and is refused stale by the agent's own write, as an LLM agent that
            chains after writing will be
  ask       observes as `observe` does; when a submit comes back stale and the refusal names a
            person who acted, it asks the person before it puts a file of the task back in the
            index (`Act::ask`, as `plev act --ask`), and waits for the answer: allowed, it stages
            the file and commits; declined, it stops with no commit
defaults: --out ./ide-lab-out --until commit --timeout 600 --grace 1 --app ide --scripted observe
          --answer none --answer-after 0
--until commit ends --grace seconds after the first commit completes (or at --timeout);
--until timeout runs the whole --timeout.
exit: 0 the commit matches the task (and the session matches git); 1 a commit exists and
differs; 2 no commit; 3 error.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    Plev,
    Scripted,
    Git,
    Screenshot,
    Review,
}

impl Arm {
    pub fn name(self) -> &'static str {
        match self {
            Arm::Plev => "plev",
            Arm::Scripted => "scripted",
            Arm::Git => "git",
            Arm::Screenshot => "screenshot",
            Arm::Review => "review",
        }
    }
}

/// what the scripted person does during the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Human {
    None,
    /// takes this path out of the index once, right after the agent is
    /// served an observation in which the task is ready to commit.
    UnstageAfterObserve(String),
    /// takes this path out of the index once, right before an agent's
    /// submit reaches the dispatcher while the task is ready to commit.
    UnstageBeforeSubmit(String),
}

impl Human {
    pub fn describe(&self) -> String {
        match self {
            Human::None => "none".to_owned(),
            Human::UnstageAfterObserve(path) => format!("unstage-after-observe:{path}"),
            Human::UnstageBeforeSubmit(path) => format!("unstage-before-submit:{path}"),
        }
    }

    pub fn path(&self) -> Option<&str> {
        match self {
            Human::None => None,
            Human::UnstageAfterObserve(path) | Human::UnstageBeforeSubmit(path) => Some(path),
        }
    }
}

/// how the scripted agent comes to its first submit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Script {
    /// observes the workspace right before the submit and submits on it.
    Observe,
    /// submits on the revision the last stage carried, with no
    /// observation before it (task-87); after a refusal it observes and
    /// decides again, as `Observe` does.
    Chain,
    /// observes as `Observe` does; after a stale refusal that names a
    /// person, asks the person before it puts a file of the task back
    /// (`Act::ask`, task-92), and stops with no commit when declined.
    Ask,
}

impl Script {
    pub fn name(self) -> &'static str {
        match self {
            Script::Observe => "observe",
            Script::Chain => "chain",
            Script::Ask => "ask",
        }
    }
}

/// how the scripted person answers a request of an agent that waits for
/// a person (`awaiting_confirmation`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// she never answers: the request waits, as before this scenario.
    None,
    /// she allows it in the modal.
    Allow,
    /// she declines it in the modal.
    Decline,
}

impl Answer {
    pub fn name(self) -> &'static str {
        match self {
            Answer::None => "none",
            Answer::Allow => "allow",
            Answer::Decline => "decline",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Until {
    Commit,
    Timeout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Args {
    pub arm: Arm,
    pub human: Human,
    pub out: PathBuf,
    pub until: Until,
    pub timeout: Duration,
    /// after the first completed commit, how long the port stays up for
    /// the late stages and a last observation.
    pub grace: Duration,
    pub app: String,
    pub script: Script,
    /// how the person answers what an agent asks her.
    pub answer: Answer,
    /// how long after the dispatcher left a request waiting she answers.
    pub answer_after: Duration,
}

pub fn parse(args: &[String]) -> Result<Args, String> {
    let mut arm = None;
    let mut human = Human::None;
    let mut out = PathBuf::from("ide-lab-out");
    let mut until = Until::Commit;
    let mut timeout = Duration::from_secs(600);
    let mut grace = Duration::from_secs(1);
    let mut app = "ide".to_owned();
    let mut script = Script::Observe;
    let mut answer = Answer::None;
    let mut answer_after = Duration::ZERO;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let mut value = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag.as_str() {
            "-h" | "--help" => return Err(USAGE.to_owned()),
            "--arm" => {
                arm = Some(match value()?.as_str() {
                    "plev" => Arm::Plev,
                    "scripted" => Arm::Scripted,
                    "git" => Arm::Git,
                    "screenshot" => Arm::Screenshot,
                    "review" => Arm::Review,
                    other => return Err(format!("unknown arm {other:?}")),
                });
            }
            "--human" => {
                let v = value()?;
                let after = v.strip_prefix("unstage-after-observe:");
                let before = v.strip_prefix("unstage-before-submit:");
                human = match (v.as_str(), after, before) {
                    ("none", _, _) => Human::None,
                    (_, Some(path), _) if !path.is_empty() => {
                        Human::UnstageAfterObserve(path.into())
                    }
                    (_, _, Some(path)) if !path.is_empty() => {
                        Human::UnstageBeforeSubmit(path.into())
                    }
                    _ => return Err(format!("unknown person {v:?}")),
                };
            }
            "--out" => out = PathBuf::from(value()?),
            "--until" => {
                until = match value()?.as_str() {
                    "commit" => Until::Commit,
                    "timeout" => Until::Timeout,
                    other => return Err(format!("--until takes commit or timeout, got {other:?}")),
                };
            }
            "--timeout" => timeout = seconds(flag, &value()?)?,
            "--grace" => grace = seconds(flag, &value()?)?,
            "--app" => app = value()?,
            "--scripted" => {
                script = match value()?.as_str() {
                    "observe" => Script::Observe,
                    "chain" => Script::Chain,
                    "ask" => Script::Ask,
                    other => {
                        return Err(format!(
                            "--scripted takes observe, chain or ask, got {other:?}"
                        ));
                    }
                };
            }
            "--answer" => {
                answer = match value()?.as_str() {
                    "none" => Answer::None,
                    "allow" => Answer::Allow,
                    "decline" => Answer::Decline,
                    other => {
                        return Err(format!(
                            "--answer takes none, allow or decline, got {other:?}"
                        ));
                    }
                };
            }
            "--answer-after" => {
                let raw = value()?;
                let ms = raw
                    .parse::<u64>()
                    .map_err(|_| format!("--answer-after takes milliseconds, got {raw:?}"))?;
                answer_after = Duration::from_millis(ms);
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    let arm = arm.ok_or("--arm is required")?;
    if answer != Answer::None && matches!(arm, Arm::Git | Arm::Screenshot | Arm::Review) {
        return Err(format!(
            "--answer {}: the {} arm has no modal to answer; it takes the plev or scripted arm",
            answer.name(),
            arm.name()
        ));
    }
    Ok(Args {
        arm,
        human,
        out,
        until,
        timeout,
        grace,
        app,
        script,
        answer,
        answer_after,
    })
}

fn seconds(flag: &str, raw: &str) -> Result<Duration, String> {
    match raw.parse::<f64>() {
        Ok(s) if s.is_finite() && s >= 0.0 => Ok(Duration::from_secs_f64(s)),
        _ => Err(format!("{flag} takes seconds, got {raw:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn every_option_parses_and_the_defaults_hold() {
        let a = parse(&args(
            "--arm scripted --human unstage-after-observe:src/session.rs --out /tmp/x \
             --until timeout --timeout 30 --grace 2.5 --app lab --scripted chain",
        ))
        .unwrap();
        assert_eq!(a.arm, Arm::Scripted);
        assert_eq!(a.human, Human::UnstageAfterObserve("src/session.rs".into()));
        assert_eq!(a.human.path(), Some("src/session.rs"));
        assert_eq!(a.out, PathBuf::from("/tmp/x"));
        assert_eq!(a.until, Until::Timeout);
        assert_eq!(a.timeout, Duration::from_secs(30));
        assert_eq!(a.grace, Duration::from_secs_f64(2.5));
        assert_eq!(a.app, "lab");
        assert_eq!(a.script, Script::Chain);

        let d = parse(&args("--arm plev")).unwrap();
        assert_eq!(d.human, Human::None);
        assert_eq!(d.out, PathBuf::from("ide-lab-out"));
        assert_eq!(d.until, Until::Commit);
        assert_eq!(d.timeout, Duration::from_secs(600));
        assert_eq!(d.grace, Duration::from_secs(1));
        assert_eq!(d.app, "ide");
        assert_eq!(d.script, Script::Observe);

        let b = parse(&args(
            "--arm scripted --human unstage-before-submit:src/session.rs",
        ))
        .unwrap();
        assert_eq!(b.human, Human::UnstageBeforeSubmit("src/session.rs".into()));
        assert_eq!(b.human.describe(), "unstage-before-submit:src/session.rs");
        assert_eq!(b.human.path(), Some("src/session.rs"));
    }

    #[test]
    fn a_bad_invocation_says_why() {
        for (line, needle) in [
            ("", "--arm is required"),
            ("--arm teleport", "unknown arm"),
            ("--arm plev --human sleep", "unknown person"),
            (
                "--arm plev --human unstage-after-observe:",
                "unknown person",
            ),
            (
                "--arm plev --human unstage-before-submit:",
                "unknown person",
            ),
            ("--arm plev --until never", "--until"),
            ("--arm plev --timeout soon", "--timeout"),
            ("--arm plev --out", "needs a value"),
            ("--arm plev --fast", "unknown option"),
            ("--arm scripted --scripted blind", "--scripted"),
        ] {
            match parse(&args(line)) {
                Err(e) => assert!(e.contains(needle), "{line}: {e:?} lacks {needle:?}"),
                Ok(a) => panic!("{line}: {a:?}"),
            }
        }
        assert!(parse(&args("--help")).unwrap_err().starts_with("usage"));
        let review = parse(&args("--arm review --out /tmp/r")).unwrap();
        assert_eq!((review.arm, review.arm.name()), (Arm::Review, "review"));
        assert!(parse(&args("--arm review --answer allow")).is_err());
    }

    #[test]
    fn the_person_who_answers_and_the_script_that_asks_parse() {
        let a = parse(&args(
            "--arm scripted --scripted ask --human unstage-before-submit:src/session.rs \
             --answer allow --answer-after 250",
        ))
        .unwrap();
        assert_eq!(a.script, Script::Ask);
        assert_eq!(a.script.name(), "ask");
        assert_eq!(a.answer, Answer::Allow);
        assert_eq!(a.answer_after, Duration::from_millis(250));
        assert_eq!(a.human, Human::UnstageBeforeSubmit("src/session.rs".into()));

        let d = parse(&args("--arm plev --answer decline")).unwrap();
        assert_eq!((d.answer, d.answer.name()), (Answer::Decline, "decline"));
        assert_eq!(d.answer_after, Duration::ZERO);
        let quiet = parse(&args("--arm scripted")).unwrap();
        assert_eq!(
            (quiet.answer, quiet.answer_after),
            (Answer::None, Duration::ZERO)
        );
        assert_eq!(
            parse(&args("--arm git --answer none")).unwrap().answer,
            Answer::None,
            "nobody answers on any arm"
        );

        for (line, needle) in [
            (
                "--arm plev --answer maybe",
                "--answer takes none, allow or decline",
            ),
            (
                "--arm plev --answer-after soon",
                "--answer-after takes milliseconds",
            ),
            (
                "--arm plev --answer-after -5",
                "--answer-after takes milliseconds",
            ),
            (
                "--arm plev --answer-after 1.5",
                "--answer-after takes milliseconds",
            ),
            ("--arm git --answer allow", "the git arm has no modal"),
            (
                "--arm screenshot --answer decline",
                "the screenshot arm has no modal",
            ),
        ] {
            match parse(&args(line)) {
                Err(e) => assert!(e.contains(needle), "{line}: {e:?} lacks {needle:?}"),
                Ok(a) => panic!("{line}: {a:?}"),
            }
        }
    }
}
