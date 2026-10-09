//! the refusals of the protocol an agent met: an act the port refused
//! before the dispatcher (`not_observed` with the reads it names,
//! `unknown_revision`, and the rest). only the agent's side sees what
//! they were: the port answers them on the connection, and the app, the
//! laboratory among it, never hears of them. the scripted agent keeps each
//! one it met; for an agent outside (the plev arm) the laboratory counts
//! the acts the port took and never dispatched, and what each one was is
//! in the agent's transcript (`plev act` exits 3 on `not_observed` and on
//! `unknown_revision`).

use serde_json::{Value as Json, json};
use socket::ProtoError;

use crate::report::row;

/// one act the port refused, as the agent met it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    pub target: String,
    pub action: String,
    /// the protocol's name for it, as it travels (`not_observed`,
    /// `unknown_revision`, `bad_message`, ...).
    pub error: String,
    /// for `not_observed`, the reads it names, in the action's order.
    pub reads: Vec<String>,
    pub text: String,
}

impl Refused {
    pub fn of(target: &str, action: &str, e: &ProtoError) -> Refused {
        let error = serde_json::to_value(e)
            .ok()
            .and_then(|v| v.get("error").and_then(Json::as_str).map(str::to_owned))
            .unwrap_or_else(|| "unknown".to_owned());
        let reads = match e {
            ProtoError::NotObserved { reads } => reads.iter().map(ToString::to_string).collect(),
            _ => Vec::new(),
        };
        Refused {
            target: target.to_owned(),
            action: action.to_owned(),
            error,
            reads,
            text: e.to_string(),
        }
    }
}

/// what the report says of the refusals of one run: `met` is what the
/// agent kept (`None` for an agent the laboratory does not hear), `taken`
/// the acts the port took and `dispatched` the ones it handed the app.
pub struct Account<'a> {
    pub met: Option<&'a [Refused]>,
    pub taken: u64,
    pub dispatched: usize,
}

impl Account<'_> {
    /// the acts the port took and did not hand the app: refused before
    /// the dispatcher.
    pub fn before_dispatch(&self) -> u64 {
        self.taken
            .saturating_sub(u64::try_from(self.dispatched).unwrap_or(u64::MAX))
    }

    fn count(&self, error: &str) -> Option<usize> {
        self.met
            .map(|met| met.iter().filter(|r| r.error == error).count())
    }

    fn other(&self) -> Option<usize> {
        self.met.map(|met| {
            met.iter()
                .filter(|r| r.error != "not_observed" && r.error != "unknown_revision")
                .count()
        })
    }

    pub fn json(&self) -> Json {
        json!({
            "seen_by": if self.met.is_some() { "the scripted agent" } else { "the port's counts" },
            "before_dispatch": self.before_dispatch(),
            "not_observed": self.count("not_observed"),
            "unknown_revision": self.count("unknown_revision"),
            "other": self.other(),
            "each": self.met.unwrap_or_default().iter().map(|r| json!({
                "target": r.target,
                "action": r.action,
                "error": r.error,
                "reads": r.reads,
                "text": r.text,
            })).collect::<Vec<_>>(),
        })
    }

    pub fn rows(&self) -> Vec<(String, String)> {
        let n = |c: Option<usize>| c.unwrap_or_default().to_string();
        let mut rows = vec![row(
            "acts refused before the dispatcher",
            &self.before_dispatch().to_string(),
        )];
        match self.met {
            Some(met) => {
                rows.push(row(
                    "not_observed / unknown_revision / other",
                    &format!(
                        "{} / {} / {} (as the scripted agent met them)",
                        n(self.count("not_observed")),
                        n(self.count("unknown_revision")),
                        n(self.other())
                    ),
                ));
                for r in met {
                    rows.push(row(&format!("{} {}", r.target, r.action), &r.text));
                }
            }
            None => rows.push(row(
                "what each was",
                "not heard here: the port answers it on the connection; see the agent's transcript",
            )),
        }
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accord::{
        ActionName, ReceiptSink, Request, RequestId, Role, SemId, Snapshot, SnapshotBuilder,
    };
    use socket::{Act, Ask, ClientMsg, Host, Observe, Reply, Server};

    /// an app of two nodes: `form` holds `text`, and `send` reads it.
    struct App(Snapshot);

    impl Host for App {
        fn project(&self) -> Option<Snapshot> {
            Some(self.0.clone())
        }

        fn dispatch(&mut self, _req: Request) -> Option<RequestId> {
            None
        }

        fn add_sink(&mut self, _sink: Box<dyn ReceiptSink>) {}
    }

    fn app() -> App {
        let mut b = SnapshotBuilder::new();
        b.node("form", Role::Group, "form")
            .state("text", "fix login");
        b.node("send", Role::Button, "send")
            .action("send")
            .reads("form:text");
        App(b.finish().expect("the app projects"))
    }

    fn ask(server: &mut Server, host: &mut App, id: &str, ask: Ask) -> Reply {
        let out = server.handle(
            1,
            ClientMsg {
                id: id.to_owned(),
                ask,
            },
            host,
        );
        out.into_iter()
            .find(|o| o.msg.id.as_deref() == Some(id))
            .map(|o| o.msg.reply)
            .expect("a reply")
    }

    /// what the scripted agent keeps when the real port refuses it, and
    /// how the report counts it: an act on a revision whose observation
    /// showed only `send` reads `form:text`, never shown (`not_observed`,
    /// naming it); an act on a revision never given is `unknown_revision`.
    #[test]
    fn the_refusals_the_port_answers_are_counted_with_their_reads() {
        let mut host = app();
        let mut server = Server::new("lab", 16);
        let hello = Ask::Hello {
            name: "scripted".to_owned(),
        };
        assert!(matches!(
            ask(&mut server, &mut host, "1", hello),
            Reply::Welcome { .. }
        ));
        let scoped = Ask::Observe(Observe {
            scope: Some(SemId::parse("send").expect("an id")),
            ..Observe::default()
        });
        let Reply::Snapshot(seen) = ask(&mut server, &mut host, "2", scoped) else {
            panic!("a snapshot");
        };
        let send = |revision| {
            Ask::Act(Act::from_revision(
                SemId::parse("send").expect("an id"),
                ActionName::parse("send").expect("a name"),
                revision,
            ))
        };
        let mut met = Vec::new();
        for (id, act) in [("3", send(seen.revision)), ("4", send(unknown()))] {
            match ask(&mut server, &mut host, id, act) {
                Reply::Error(e) => met.push(Refused::of("send", "send", &e)),
                other => panic!("refused before the dispatcher: {other:?}"),
            }
        }

        assert_eq!(met[0].error, "not_observed");
        assert_eq!(met[0].reads, ["form:text"]);
        assert_eq!(met[1].error, "unknown_revision");
        assert!(met[1].reads.is_empty());

        let account = Account {
            met: Some(&met),
            taken: server.traffic().acts,
            dispatched: 0,
        };
        let report = account.json();
        assert_eq!(report["seen_by"], "the scripted agent");
        assert_eq!(report["before_dispatch"], 2);
        assert_eq!(report["not_observed"], 1);
        assert_eq!(report["unknown_revision"], 1);
        assert_eq!(report["other"], 0);
        assert_eq!(report["each"][0]["reads"], json!(["form:text"]));

        // an agent outside: only the port's count, and no kinds.
        let outside = Account {
            met: None,
            taken: 5,
            dispatched: 3,
        };
        let report = outside.json();
        assert_eq!(report["before_dispatch"], 2);
        assert!(report["not_observed"].is_null());
        assert_eq!(report["each"], json!([]));
    }

    fn unknown() -> accord::Revision {
        serde_json::from_str("\"00000000000000aa\"").expect("a revision")
    }
}
