//! the command line of the laboratory.

use std::path::PathBuf;
use std::time::Duration;

pub const USAGE: &str = "\
usage:
  ide-lab --arm plev|scripted|git|screenshot [--human none|unstage-after-observe:<path>]
          [--out <dir>] [--until commit|timeout] [--timeout <s>] [--grace <s>] [--app <name>]
arms:
  plev        the workbench on the agent port, for an agent outside (plev observe / plev act)
  scripted    the same, driven by the laboratory's own agent over the wire (the control run)
  git         the same repository for an agent that uses git itself: hook, shim, no port
  screenshot  the frame of the workbench as PNG at scale 1 and 2, beside the bytes of an observation
the person of the script:
  none                              nobody intervenes (default)
  unstage-after-observe:<path>      once the agent has every file of the task in the index and a
                                    message written, right after it is served an observation, the
                                    person takes <path> out of the index; the observation is stale
defaults: --out ./ide-lab-out --until commit --timeout 600 --grace 1 --app ide
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
}

impl Arm {
    pub fn name(self) -> &'static str {
        match self {
            Arm::Plev => "plev",
            Arm::Scripted => "scripted",
            Arm::Git => "git",
            Arm::Screenshot => "screenshot",
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
}

impl Human {
    pub fn describe(&self) -> String {
        match self {
            Human::None => "none".to_owned(),
            Human::UnstageAfterObserve(path) => format!("unstage-after-observe:{path}"),
        }
    }

    pub fn path(&self) -> Option<&str> {
        match self {
            Human::None => None,
            Human::UnstageAfterObserve(path) => Some(path),
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
}

pub fn parse(args: &[String]) -> Result<Args, String> {
    let mut arm = None;
    let mut human = Human::None;
    let mut out = PathBuf::from("ide-lab-out");
    let mut until = Until::Commit;
    let mut timeout = Duration::from_secs(600);
    let mut grace = Duration::from_secs(1);
    let mut app = "ide".to_owned();
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
                    other => return Err(format!("unknown arm {other:?}")),
                });
            }
            "--human" => {
                let v = value()?;
                human = match v.as_str() {
                    "none" => Human::None,
                    _ => match v.strip_prefix("unstage-after-observe:") {
                        Some(path) if !path.is_empty() => Human::UnstageAfterObserve(path.into()),
                        _ => return Err(format!("unknown person {v:?}")),
                    },
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
            other => return Err(format!("unknown option {other}")),
        }
    }
    let arm = arm.ok_or("--arm is required")?;
    Ok(Args {
        arm,
        human,
        out,
        until,
        timeout,
        grace,
        app,
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
             --until timeout --timeout 30 --grace 2.5 --app lab",
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

        let d = parse(&args("--arm plev")).unwrap();
        assert_eq!(d.human, Human::None);
        assert_eq!(d.out, PathBuf::from("ide-lab-out"));
        assert_eq!(d.until, Until::Commit);
        assert_eq!(d.timeout, Duration::from_secs(600));
        assert_eq!(d.grace, Duration::from_secs(1));
        assert_eq!(d.app, "ide");
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
            ("--arm plev --until never", "--until"),
            ("--arm plev --timeout soon", "--timeout"),
            ("--arm plev --out", "needs a value"),
            ("--arm plev --fast", "unknown option"),
        ] {
            match parse(&args(line)) {
                Err(e) => assert!(e.contains(needle), "{line}: {e:?} lacks {needle:?}"),
                Ok(a) => panic!("{line}: {a:?}"),
            }
        }
        assert!(parse(&args("--help")).unwrap_err().starts_with("usage"));
    }
}
