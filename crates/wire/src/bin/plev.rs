//! `plev`: an agent's command line over an app's agent port.
//!
//! ```text
//! plev observe [--scope <id>] [--keys <k,k>] [--since <rev>] [--bounds]
//! plev act <target> <action> [--from-revision <rev>] [--param <json>]
//! common: [--app <name or socket path>] [--name <agent>] [--timeout <ms>] [--pretty]
//! ```
//!
//! every message the app answers is printed as it arrives, one JSON line
//! each (`--pretty` indents them). the exit code says how it ended.

#[cfg(unix)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(cli::run(&args));
}

#[cfg(not(unix))]
fn main() {
    eprintln!("plev: the agent port is a unix socket, and this platform has none");
    std::process::exit(3);
}

#[cfg(unix)]
mod cli {
    use std::path::PathBuf;

    use sem::{ActionName, Revision, SemId, Stage, StateKey, Value};
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error as DeError, StrDeserializer};
    use wire::{
        Act, Client, Observe, Reply, ServerMsg, StageReply, WireError, encode_server, find,
        socket_dir,
    };

    pub const USAGE: &str = "\
usage:
  plev observe [--scope <id>] [--keys <k,k>] [--since <rev>] [--bounds]
  plev act <target> <action> [--from-revision <rev>] [--param <json>]
common options:
  --app <name|path>  the app's name (its socket in $TMPDIR/plev) or a socket
                     path; without it, the one app listening there
  --name <agent>     the actor's name (default $PLEV_AGENT_NAME, else plev)
  --timeout <ms>     how long to wait for the app (default 30000)
  --pretty           indented JSON
act without --from-revision observes the target first and acts on that
revision: the state as it is now. --param takes a sem value
({\"kind\":\"text\",\"value\":\"x\"}) or a plain JSON string, bool, number or
list of strings.
exit: 0 observed or completed, 1 rejected, 2 failed, 3 error.";

    /// how an invocation ended.
    pub const DONE: i32 = 0;
    pub const REJECTED: i32 = 1;
    pub const FAILED: i32 = 2;
    pub const ERROR: i32 = 3;

    #[derive(Debug, Default, PartialEq)]
    pub struct Common {
        pub app: Option<String>,
        pub name: Option<String>,
        pub timeout: u64,
        pub pretty: bool,
    }

    #[derive(Debug, PartialEq)]
    pub enum Command {
        Help,
        Observe(Common, Observe),
        Act(Common, Act, Option<Revision>),
    }

    pub fn run(args: &[String]) -> i32 {
        match parse(args) {
            Ok(Command::Help) => {
                println!("{USAGE}");
                DONE
            }
            Ok(Command::Observe(common, observe)) => report(&common, |c| observe_once(c, observe)),
            Ok(Command::Act(common, act, from)) => report(&common, |c| act_once(c, act, from)),
            Err(e) => {
                eprintln!("plev: {e}\n\n{USAGE}");
                ERROR
            }
        }
    }

    pub fn parse(args: &[String]) -> Result<Command, String> {
        let mut it = args.iter();
        let verb = it.next().map(String::as_str);
        let mut positional = Vec::new();
        let mut common = Common {
            timeout: 30_000,
            ..Common::default()
        };
        let mut observe = Observe::default();
        let mut from = None;
        let mut param = None;
        while let Some(arg) = it.next() {
            let mut value = || {
                it.next()
                    .cloned()
                    .ok_or_else(|| format!("{arg} needs a value"))
            };
            match arg.as_str() {
                "-h" | "--help" => return Ok(Command::Help),
                "--app" => common.app = Some(value()?),
                "--name" => common.name = Some(value()?),
                "--timeout" => {
                    let v = value()?;
                    common.timeout = v
                        .parse()
                        .map_err(|_| format!("--timeout takes ms, got {v:?}"))?;
                }
                "--pretty" => common.pretty = true,
                "--scope" => observe.scope = Some(text(&value()?)?),
                "--keys" => {
                    let keys: Result<Vec<StateKey>, String> =
                        value()?.split(',').map(|k| text(k.trim())).collect();
                    observe.keys = Some(keys?);
                }
                "--since" => observe.since = Some(text(&value()?)?),
                "--bounds" => observe.bounds = true,
                "--from-revision" => from = Some(text(&value()?)?),
                "--param" => param = Some(value_of(&value()?)?),
                flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
                other => positional.push(other.to_owned()),
            }
        }
        match (verb, positional.as_slice()) {
            (None | Some("help"), _) => Ok(Command::Help),
            (Some("-h" | "--help"), _) => Ok(Command::Help),
            (Some("observe"), []) => Ok(Command::Observe(common, observe)),
            (Some("act"), [target, action]) => {
                let target: SemId = text(target)?;
                let action: ActionName = text(action)?;
                let act = Act {
                    target,
                    action,
                    param,
                    observed: None,
                    from_revision: from,
                };
                Ok(Command::Act(common, act, from))
            }
            (Some("act"), _) => Err("act takes <target> <action>".to_owned()),
            (Some("observe"), extra) => Err(format!("observe takes no argument, got {extra:?}")),
            (Some(other), _) => Err(format!("unknown command {other:?}")),
        }
    }

    /// a canonical text into its type (an id, a key, a revision), through
    /// the type's own serde form.
    fn text<T: for<'de> Deserialize<'de>>(raw: &str) -> Result<T, String> {
        let de: StrDeserializer<'_, DeError> = raw.into_deserializer();
        T::deserialize(de).map_err(|e| e.to_string())
    }

    /// `--param`: a sem value, or plain JSON of an obvious kind.
    pub fn value_of(raw: &str) -> Result<Value, String> {
        use serde_json::Value as Json;
        let json: Json = serde_json::from_str(raw).map_err(|e| format!("--param: {e}"))?;
        match json {
            Json::Object(ref o) if o.contains_key("kind") => {
                serde_json::from_value(json).map_err(|e| format!("--param: {e}"))
            }
            Json::String(s) => Ok(Value::Text(s)),
            Json::Bool(b) => Ok(Value::Bool(b)),
            Json::Number(n) => match (n.as_i64(), n.as_f64()) {
                (Some(i), _) => Ok(Value::Int(i)),
                (None, Some(f)) => Ok(Value::Num(f)),
                _ => Err(format!("--param: {n} fits no value")),
            },
            Json::Array(items) => items
                .into_iter()
                .map(|item| match item {
                    Json::String(s) => Ok(s),
                    other => Err(format!("--param: a list holds strings, got {other}")),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Value::List),
            other => Err(format!("--param: {other} fits no value")),
        }
    }

    /// connects as `common` says, runs `job`, and turns what went wrong
    /// into the error exit, with the app's refusal printed as it came.
    fn report(common: &Common, job: impl FnOnce(&mut Session) -> Result<i32, WireError>) -> i32 {
        let result = connect(common).and_then(|mut client| {
            let mut session = Session {
                client: &mut client,
                pretty: common.pretty,
            };
            job(&mut session)
        });
        match result {
            Ok(code) => code,
            Err(WireError::Refused(e)) => {
                print(
                    &ServerMsg {
                        id: None,
                        reply: Reply::Error(e.clone()),
                    },
                    common.pretty,
                );
                eprintln!("plev: {e}");
                ERROR
            }
            Err(e) => {
                eprintln!("plev: {e}");
                ERROR
            }
        }
    }

    struct Session<'a> {
        client: &'a mut Client,
        pretty: bool,
    }

    fn connect(common: &Common) -> Result<Client, WireError> {
        let path = match &common.app {
            Some(app) if app.contains('/') || app.ends_with(".sock") => PathBuf::from(app),
            Some(app) => find(&socket_dir(), Some(app))?,
            None => find(&socket_dir(), None)?,
        };
        let mut client = Client::connect(&path)?;
        client.set_timeout(Some(common.timeout))?;
        let name = common
            .name
            .clone()
            .or_else(|| std::env::var("PLEV_AGENT_NAME").ok())
            .unwrap_or_else(|| "plev".to_owned());
        client.hello(&name)?;
        Ok(client)
    }

    fn observe_once(s: &mut Session, observe: Observe) -> Result<i32, WireError> {
        let reply = s.client.observe(observe)?;
        print(&ServerMsg { id: None, reply }, s.pretty);
        Ok(DONE)
    }

    fn act_once(s: &mut Session, mut act: Act, from: Option<Revision>) -> Result<i32, WireError> {
        if from.is_none() {
            let now = s.client.observe(Observe {
                scope: Some(act.target.clone()),
                ..Observe::default()
            })?;
            act.from_revision = Some(match now {
                Reply::Snapshot(snap) => snap.revision,
                Reply::Delta(delta) => delta.revision,
                _ => return Err(WireError::BadReply("no revision to act on".to_owned())),
            });
        }
        let pretty = s.pretty;
        let stages = s.client.act(act, |stage| {
            print(
                &ServerMsg {
                    id: None,
                    reply: Reply::Stage(stage.clone()),
                },
                pretty,
            );
        })?;
        Ok(stages.last().map_or(ERROR, exit_of))
    }

    pub fn exit_of(stage: &StageReply) -> i32 {
        match stage.stage {
            Stage::Completed(_) => DONE,
            Stage::Rejected(_) => REJECTED,
            Stage::Failed { .. } => FAILED,
            _ => ERROR,
        }
    }

    fn print(msg: &ServerMsg, pretty: bool) {
        let line = encode_server(msg);
        if pretty {
            match serde_json::from_str::<serde_json::Value>(&line)
                .and_then(|v| serde_json::to_string_pretty(&v))
            {
                Ok(text) => println!("{text}"),
                Err(_) => println!("{line}"),
            }
        } else {
            println!("{line}");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn args(line: &str) -> Vec<String> {
            line.split_whitespace().map(str::to_owned).collect()
        }

        #[test]
        fn observe_takes_its_options() {
            let Ok(Command::Observe(common, o)) = parse(&args(
                "observe --scope todo --keys text,completed --since 3ba694e7d3941ec0 --bounds \
                 --app showcase --name claude --timeout 500 --pretty",
            )) else {
                panic!("an observe");
            };
            assert_eq!(o.scope.map(|s| s.to_string()), Some("todo".into()));
            assert_eq!(o.keys.map(|k| k.len()), Some(2));
            assert_eq!(
                o.since.map(|r| r.to_string()),
                Some("3ba694e7d3941ec0".into())
            );
            assert!(o.bounds);
            assert_eq!(common.app.as_deref(), Some("showcase"));
            assert_eq!(common.name.as_deref(), Some("claude"));
            assert_eq!(common.timeout, 500);
            assert!(common.pretty);
        }

        #[test]
        fn act_takes_a_target_an_action_and_a_revision() {
            let Ok(Command::Act(_, act, from)) = parse(&args(
                "act todo.item[1] toggle --from-revision 3ba694e7d3941ec0",
            )) else {
                panic!("an act");
            };
            assert_eq!(act.target.as_str(), "todo.item[1]");
            assert_eq!(act.action.as_str(), "toggle");
            assert!(from.is_some() && act.from_revision == from);
            let Ok(Command::Act(_, act, None)) =
                parse(&args(r#"act todo.input set_value --param "milk""#))
            else {
                panic!("an act without a revision");
            };
            assert_eq!(act.param, Some(Value::Text("milk".into())));
        }

        #[test]
        fn a_param_is_a_sem_value_or_plain_json() {
            assert_eq!(value_of(r#""x""#), Ok(Value::Text("x".into())));
            assert_eq!(value_of("true"), Ok(Value::Bool(true)));
            assert_eq!(value_of("-3"), Ok(Value::Int(-3)));
            assert_eq!(value_of("0.5"), Ok(Value::Num(0.5)));
            assert_eq!(
                value_of(r#"["a","b"]"#),
                Ok(Value::List(vec!["a".into(), "b".into()]))
            );
            assert_eq!(
                value_of(r#"{"kind":"set","value":["b","a"]}"#),
                Ok(Value::Set(["a".to_owned(), "b".to_owned()].into()))
            );
            for bad in [
                "null",
                "[1]",
                r#"{"a":1}"#,
                "{",
                r#"{"kind":"bool","value":1}"#,
            ] {
                assert!(value_of(bad).is_err(), "{bad}");
            }
        }

        #[test]
        fn a_bad_invocation_says_why() {
            for (line, needle) in [
                ("teleport", "unknown command"),
                ("act todo.item[1]", "<target> <action>"),
                ("act Todo toggle", "Todo"),
                ("observe extra", "no argument"),
                ("observe --since nope", "revision"),
                ("observe --keys Bad", "Bad"),
                ("observe --scope", "needs a value"),
                ("observe --timeout soon", "--timeout"),
                ("observe --fast", "unknown option"),
            ] {
                match parse(&args(line)) {
                    Err(e) => assert!(e.contains(needle), "{line}: {e:?} lacks {needle:?}"),
                    Ok(c) => panic!("{line}: {c:?}"),
                }
            }
            assert_eq!(parse(&[]), Ok(Command::Help));
            assert_eq!(parse(&args("observe --help")), Ok(Command::Help));
        }
    }
}
