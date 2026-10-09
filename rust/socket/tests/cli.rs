//! the `plev` binary against the toy app's port: what it prints and how it
//! exits (0 observed or completed, 1 rejected, 2 failed, 3 error).

#![cfg(unix)]

mod toy;

use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use accord::Revision;
use toy::ToyHost;
use socket::{AgentPort, Config, Host, ProtoError, Reply, ServerMsg, decode_server};

struct Run {
    code: i32,
    out: Vec<ServerMsg>,
    /// stdout as it came.
    stdout: String,
    err: String,
}

/// runs `plev <args>` with `TMPDIR` at `tmp` while this thread serves the
/// port as the app's shell would.
fn plev(
    port: &mut AgentPort,
    woke: &Receiver<()>,
    host: &mut ToyHost,
    tmp: &Path,
    args: &[&str],
) -> Run {
    plev_with(port, woke, host, tmp, args, |_| {})
}

/// [`plev`], with `person` acting on the app between two drains, as a
/// person at the screen would.
fn plev_with(
    port: &mut AgentPort,
    woke: &Receiver<()>,
    host: &mut ToyHost,
    tmp: &Path,
    args: &[&str],
    mut person: impl FnMut(&mut ToyHost),
) -> Run {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_plev"));
    cmd.args(args)
        .env("TMPDIR", tmp)
        .env("PLEV_AGENT_NAME", "cli");
    let child = thread::spawn(move || cmd.output().unwrap());
    let deadline = Instant::now() + Duration::from_secs(30);
    while !child.is_finished() {
        assert!(Instant::now() < deadline, "plev did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(5));
        port.drain(host);
        // a sync ends a turn after it started, as a worker's result would,
        // and the shell drains again to send the late stage.
        if !host.app.syncing.is_empty() {
            host.finish_sync();
            port.drain(host);
        }
        person(host);
    }
    let output = child.join().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    Run {
        code: output.status.code().unwrap_or(-1),
        out: stdout
            .lines()
            .filter(|l| l.starts_with('{'))
            .map(|l| decode_server(l).unwrap_or_else(|e| panic!("{l}: {e:?}")))
            .collect(),
        stdout,
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn open(tmp: &Path) -> (AgentPort, Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    let config = Config {
        app: "toy".into(),
        dir: tmp.join("plev"),
        ring: 8,
    };
    let port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .unwrap();
    (port, rx)
}

fn revision(run: &Run) -> Revision {
    match run.out.first().map(|m| &m.reply) {
        Some(Reply::Snapshot(s)) => s.revision,
        other => panic!("{other:?}"),
    }
}

fn stages(run: &Run) -> Vec<&'static str> {
    run.out
        .iter()
        .map(|m| match &m.reply {
            Reply::Stage(s) => s.stage.name(),
            other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn observe_and_act_by_app_name_with_their_exit_codes() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let t = tmp.path();

    let seen = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["observe", "--scope", "todo", "--app", "toy"],
    );
    assert_eq!(seen.code, 0, "{}", seen.err);
    assert_eq!(seen.out.len(), 1);
    let r = revision(&seen);
    let r = r.to_string();
    let Reply::Snapshot(plain) = &seen.out[0].reply else {
        panic!("a snapshot");
    };
    assert!(
        plain.nodes.iter().all(|n| n.source.is_none()),
        "no source unless asked: {plain:?}"
    );

    // --source keeps where each node was projected.
    let sourced = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["observe", "--scope", "todo", "--source", "--app", "toy"],
    );
    assert_eq!(sourced.code, 0, "{}", sourced.err);
    let Some(Reply::Snapshot(sourced)) = sourced.out.first().map(|m| &m.reply) else {
        panic!("a snapshot: {:?}", sourced.out);
    };
    assert_eq!(sourced.nodes.len(), plain.nodes.len());
    assert!(sourced.nodes.iter().all(|n| n.source.is_some()));

    let done = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.item[2]", "toggle", "--from-revision", &r],
    );
    assert_eq!(done.code, 0, "{}", done.err);
    assert_eq!(
        stages(&done),
        ["requested", "accepted", "executing", "completed"]
    );
    assert_eq!(host.app.done(2), Some(true));
    assert!(
        done.out
            .iter()
            .all(|m| m.id == done.out[0].id && m.id.is_some()),
        "every stage under the act's id"
    );
    let Reply::Stage(first) = &done.out[0].reply else {
        panic!("a stage");
    };
    let asked = first.asked.as_ref().unwrap();
    assert_eq!(asked.actor, accord::Actor::Agent { name: "cli".into() });

    // the completed line carries the revision the app is at, and the next
    // command takes it: a delta since it, an act resting on it.
    let Some(Reply::Stage(end)) = done.out.last().map(|m| &m.reply) else {
        panic!("a stage: {:?}", done.out);
    };
    let landed = end.revision.expect("the completed carries a revision");
    assert_eq!(Some(landed), host.project().map(|s| s.revision()));
    let landed = landed.to_string();
    let since = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["observe", "--since", &landed],
    );
    assert_eq!(since.code, 0, "{}", since.err);
    assert!(
        matches!(&since.out[0].reply, Reply::Delta(d) if d.changes.is_empty()),
        "{:?}",
        since.out
    );
    let chained = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.item[3]", "toggle", "--from-revision", &landed],
    );
    assert_eq!(chained.code, 0, "{}", chained.err);
    assert_eq!(host.app.done(3), Some(false));

    // a person toggles item 1 after the observation: the act on the
    // revision the agent saw is stale.
    host.click("todo.item[1]", "toggle");
    let stale = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &[
            "act",
            "todo.item[1]",
            "toggle",
            "--from-revision",
            &r,
            "--app",
            "toy",
        ],
    );
    assert_eq!(stale.code, 1, "{}", stale.err);
    assert_eq!(stages(&stale), ["requested", "rejected"]);
    assert_eq!(host.app.done(1), Some(true), "only the person's toggle ran");

    // an execution that ends later: plev waits for its last stage.
    let later = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.sync", "sync"],
    );
    assert_eq!(later.code, 0, "{}", later.err);
    assert_eq!(
        stages(&later),
        ["requested", "accepted", "executing", "completed"]
    );

    // without a revision, plev acts on the state as it is now.
    let now = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.item[2]", "toggle"],
    );
    assert_eq!(now.code, 0, "{}", now.err);
    assert_eq!(host.app.done(2), Some(false));

    // the app ran it and it failed; the secret stays in.
    let failed = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.pin", "set_value", "--param", r#""123456""#],
    );
    assert_eq!(failed.code, 2, "{}", failed.err);
    let printed = format!("{:?}", failed.out);
    assert!(!printed.contains("123456"), "{printed}");

    // a delta since the first observation.
    let delta = plev(&mut port, &woke, &mut host, t, &["observe", "--since", &r]);
    assert_eq!(delta.code, 0, "{}", delta.err);
    assert!(matches!(delta.out[0].reply, Reply::Delta(_)));
}

#[test]
fn a_refusal_or_no_app_is_the_error_exit() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let (mut port, woke) = open(t);
    let mut host = ToyHost::new();

    let unknown = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &[
            "act",
            "todo.item[1]",
            "toggle",
            "--from-revision",
            "0123456789abcdef",
        ],
    );
    assert_eq!(unknown.code, 3);
    assert!(matches!(
        unknown.out[0].reply,
        Reply::Error(ProtoError::UnknownRevision { .. })
    ));
    assert!(unknown.err.contains("observe again"), "{}", unknown.err);

    let socket = port.path().display().to_string();
    let bad = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["observe", "--scope", "Not An Id", "--app", &socket],
    );
    assert_eq!(bad.code, 3);
    assert!(bad.out.is_empty(), "refused before connecting");

    let none = plev(&mut port, &woke, &mut host, t, &["observe", "--app", "ide"]);
    assert_eq!(none.code, 3);
    assert!(none.err.contains("no plev app"), "{}", none.err);

    let help = plev(&mut port, &woke, &mut host, t, &["--help"]);
    assert_eq!(help.code, 0);
    assert!(help.out.is_empty());
}

/// task-89: an act rests on what its name was shown. `plev act` without
/// a revision looks at the target's scope first, which counts as shown;
/// what the act reads outside the target was never shown to a fresh name,
/// so it exits 3 with `not_observed`. after `plev observe` of that node
/// under the same name, one command later, the same command is taken.
#[test]
fn an_act_that_reads_a_node_its_name_never_observed_is_the_error_exit() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let (mut port, woke) = open(t);
    let mut host = ToyHost::new();
    let clear = ["act", "todo.clear", "clear", "--name", "fresh"];
    let not_observed = |run: &Run| {
        assert_eq!(run.code, 3, "{}", run.err);
        assert!(
            matches!(
                &run.out[..],
                [ServerMsg { reply: Reply::Error(ProtoError::NotObserved { reads }), .. }]
                    if reads.iter().map(ToString::to_string).eq(["todo.counter:done"])
            ),
            "{:?}",
            run.out
        );
        assert!(run.err.contains("observe it"), "{}", run.err);
    };

    let refused = plev(&mut port, &woke, &mut host, t, &clear);
    not_observed(&refused);
    assert_eq!(host.app.items.len(), 3, "nothing cleared");

    // what another name was shown does not count for this one.
    let other = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["observe", "--scope", "todo.counter"],
    );
    assert_eq!(other.code, 0, "{}", other.err);
    not_observed(&plev(&mut port, &woke, &mut host, t, &clear));

    let seen = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["observe", "--scope", "todo.counter", "--name", "fresh"],
    );
    assert_eq!(seen.code, 0, "{}", seen.err);
    let done = plev(&mut port, &woke, &mut host, t, &clear);
    assert_eq!(done.code, 0, "{}", done.err);
    assert_eq!(
        stages(&done),
        ["requested", "accepted", "executing", "completed"]
    );
    let Reply::Stage(first) = &done.out[0].reply else {
        panic!("a stage");
    };
    let asked = first.asked.as_ref().unwrap();
    assert_eq!(
        asked.actor,
        accord::Actor::Agent {
            name: "fresh".into()
        }
    );
    assert_eq!(host.app.items.len(), 2, "item 3 cleared");
}

/// the request that waits for a person, if one does.
fn waiting(host: &ToyHost) -> Option<accord::RequestId> {
    (1..=64)
        .filter_map(|n| accord::RequestId::parse(&n.to_string()).ok())
        .find(|id| {
            host.dispatcher
                .receipt(*id)
                .is_some_and(|r| r.stage() == Some(&accord::Stage::AwaitingConfirmation))
        })
}

/// task-92: `plev act --ask` asks a person first. it prints
/// `awaiting_confirmation` and waits; the person confirms (exit 0, the
/// act ran) or declines (exit 1, `declined`, nothing ran). the question
/// goes out in `asked`, and one that does not fit never connects.
#[test]
fn act_ask_waits_for_the_person_and_exits_by_the_answer() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let (mut port, woke) = open(t);
    let mut host = ToyHost::new();
    let answer = |confirm: bool| {
        move |host: &mut ToyHost| {
            if let Some(id) = waiting(host) {
                host.clock += 1;
                if confirm {
                    host.dispatcher
                        .confirm(&mut host.app, id, accord::Actor::Human, host.clock)
                        .unwrap();
                } else {
                    host.dispatcher
                        .decline(id, accord::Actor::Human, host.clock)
                        .unwrap();
                }
            }
        }
    };

    let question = "item 2 is not done: mark it done?";
    let confirmed = plev_with(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.item[2]", "toggle", "--ask", question],
        answer(true),
    );
    assert_eq!(confirmed.code, 0, "{}", confirmed.err);
    assert_eq!(
        stages(&confirmed),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    let Reply::Stage(first) = &confirmed.out[0].reply else {
        panic!("a stage");
    };
    assert_eq!(
        first.asked.as_ref().and_then(|a| a.ask.as_deref()),
        Some(question)
    );
    assert_eq!(host.app.done(2), Some(true));

    let declined = plev_with(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.item[1]", "toggle", "--ask", "item 1 too?"],
        answer(false),
    );
    assert_eq!(declined.code, 1, "{}", declined.err);
    assert_eq!(
        stages(&declined),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    let Some(Reply::Stage(last)) = declined.out.last().map(|m| &m.reply) else {
        panic!("a stage");
    };
    assert_eq!(
        last.stage,
        accord::Stage::Rejected(accord::Rejection::Declined {
            by: accord::Actor::Human
        })
    );
    assert!(last.revision.is_some(), "a terminal stage");
    assert_eq!(host.app.done(1), Some(false), "nothing ran");
    assert_eq!(
        declined.err, "",
        "the person answered: nothing points at asking"
    );

    let long = "x".repeat(socket::MAX_ASK + 1);
    let refused = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.item[1]", "toggle", "--ask", &long],
    );
    assert_eq!(refused.code, 3);
    assert!(refused.out.is_empty(), "refused before connecting");
    assert!(refused.err.contains("--ask"), "{}", refused.err);
}

/// a person's act through the toy's dispatcher by assistive technology.
fn assistive(host: &mut ToyHost, target: &str, action: &str) {
    let seen = host.snapshot();
    let req = accord::Request::observe(&seen, accord::Actor::Assistive, target, action, None).unwrap();
    host.clock += 1;
    host.dispatcher.dispatch(&mut host.app, req, host.clock);
}

/// task-94: an act refused as stale whose `acted` names a person, when it
/// did not ask, gets one line on stderr after its stages: who acted, and
/// `--ask`. stdout is the stages, as the app sent them, and nothing else.
/// no line when only another agent acted, or when the act asked.
#[test]
fn a_stale_refusal_naming_a_person_points_at_the_question_on_stderr() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let (mut port, woke) = open(t);
    let mut host = ToyHost::new();
    let seen = plev(&mut port, &woke, &mut host, t, &["observe"]);
    let r = revision(&seen).to_string();
    let on_r = |item| ["act", item, "toggle", "--from-revision", r.as_str()];

    // only another agent acted since the look: refused, no line.
    let other = ["act", "todo.item[1]", "toggle", "--name", "other"];
    assert_eq!(plev(&mut port, &woke, &mut host, t, &other).code, 0);
    let by_agent = plev(&mut port, &woke, &mut host, t, &on_r("todo.item[1]"));
    assert_eq!(by_agent.code, 1, "{}", by_agent.err);
    assert_eq!(stages(&by_agent), ["requested", "rejected"]);
    assert_eq!(by_agent.err, "", "only an agent acted");

    // a person at the screen, then one through assistive technology.
    host.click("todo.item[2]", "toggle");
    assistive(&mut host, "todo.item[3]", "toggle");
    let by_people = plev(&mut port, &woke, &mut host, t, &on_r("todo.item[2]"));
    assert_eq!(by_people.code, 1, "{}", by_people.err);
    assert_eq!(stages(&by_people), ["requested", "rejected"]);
    let canonical: String = by_people
        .out
        .iter()
        .map(|m| format!("{}\n", socket::encode_server(m)))
        .collect();
    assert_eq!(by_people.stdout, canonical, "the stages and nothing else");
    assert_eq!(
        by_people.err,
        "plev: a person acted since this agent looked (request 3, human: toggle todo.item[2]; \
         request 4, assistive: toggle todo.item[3]); to act over that, ask the person first: \
         add --ask \"<why>\" to the act (one line, at most 280 characters)\n"
    );

    // the same act asking the person: refused as stale before any
    // question, and no line, since it asked.
    let ask = ["--ask", "toggle item 2 back?"];
    let asking = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &[&on_r("todo.item[2]")[..], &ask].concat(),
    );
    assert_eq!(asking.code, 1, "{}", asking.err);
    assert_eq!(stages(&asking), ["requested", "rejected"]);
    assert_eq!(asking.err, "", "it asked");
    assert_eq!(host.app.done(2), Some(true), "only the person's toggle ran");
}
