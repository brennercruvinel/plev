//! `plev-inspect`: reads a recorded session and explains it.
//!
//! ```text
//! plev-inspect <session.monster> [--timeline] [--why <request id> [--origin <n>]]
//!              [--control <id> [--at <ms>]] [--json]
//! plev-inspect <session.monster> --export <t0> <t1> <excerpt.monster>
//! ```
//!
//! with the file alone it prints a summary; `--export` writes a stretch of
//! it as a session of its own, the bug report. the exit code says how it
//! ended. reading never runs anything: the session holds no app.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(cli::run(&args));
}

mod cli {
    use std::path::{Path, PathBuf};

    use replay::{Control, Inspector, Summary, Timeline, Why, prose};
    use record::{ReadError, Session};
    use accord::{Millis, RequestId, SemId};

    pub const USAGE: &str = "\
usage:
  plev-inspect <session.monster> [--timeline] [--why <request id> [--origin <n>]]
               [--control <id> [--at <ms>]] [--json]
  plev-inspect <session.monster> --export <t0> <t1> <excerpt.monster>
with the file alone: a summary: the build that recorded the session, its span,
the actors, every request with how it ended, and every refusal with a short why.
  --timeline      every event, one line each, after the tallies by kind and actor
  --why <id>      a request's causal chain: the control, its state, who initiated
                  it, the action, the observation it rested on, the precondition
                  (what the action reads, as the actor saw it), what moved in it
                  between the observation and the request and which receipt ran
                  then, the stages, the verdict with its delta, and the result
  --origin <n>    the dispatcher the request id belongs to, when the session has
                  several (one per sink the recorder gave out)
  --control <id>  a control at an instant: its node, state, actions and source,
                  and every request that touched it up to then, with its why
  --at <ms>       the instant for --control, in ms from the start of the session,
                  as every view prints times (default: the end)
  --json          the same answers as json, under summary, timeline, why, control
  --export <t0> <t1> <out>
                  writes the stretch from t0 to t1 (ms from the start of the
                  session, as --at) to out as a session of its own, the bug
                  report: from the full snapshot and the chunk of frames that
                  govern t0, up to t1, only the assets its frames use. it goes
                  alone, and never over the file it reads
who moved a value is a correlation in time, not a declared write: the receipts
that ran (executing, completed or failed) between the recorded state before the
change and the change; none is a change from outside the app, two or more are
ambiguous and all shown. reading a session never runs anything.
exit: 0 answered, 1 the request is not in the session, 2 bad invocation,
3 the file does not read as a session, 4 the export could not be written.";

    pub const DONE: i32 = 0;
    pub const NOT_FOUND: i32 = 1;
    pub const USAGE_ERROR: i32 = 2;
    pub const READ_ERROR: i32 = 3;
    pub const WRITE_ERROR: i32 = 4;

    /// a stretch of the session to write as its own: from `from` to `to`,
    /// in ms from the start of the session.
    #[derive(Debug, PartialEq)]
    pub struct Export {
        pub from: Millis,
        pub to: Millis,
        pub out: PathBuf,
    }

    #[derive(Debug, Default, PartialEq)]
    pub struct Ask {
        pub path: PathBuf,
        pub timeline: bool,
        pub why: Option<RequestId>,
        pub origin: Option<u16>,
        pub control: Option<SemId>,
        pub at: Option<Millis>,
        pub json: bool,
        pub export: Option<Export>,
    }

    #[derive(Debug, PartialEq)]
    pub enum Command {
        Help,
        Inspect(Ask),
    }

    pub fn run(args: &[String]) -> i32 {
        match parse(args) {
            Ok(Command::Help) => {
                println!("{USAGE}");
                DONE
            }
            Ok(Command::Inspect(ask)) => inspect(&ask),
            Err(e) => {
                eprintln!("plev-inspect: {e}\n\n{USAGE}");
                USAGE_ERROR
            }
        }
    }

    pub fn parse(args: &[String]) -> Result<Command, String> {
        let mut it = args.iter();
        let mut path = None;
        let mut ask = Ask::default();
        while let Some(arg) = it.next() {
            let mut value = || {
                it.next()
                    .cloned()
                    .ok_or_else(|| format!("{arg} needs a value"))
            };
            match arg.as_str() {
                "-h" | "--help" => return Ok(Command::Help),
                "--timeline" => ask.timeline = true,
                "--json" => ask.json = true,
                "--why" => {
                    let v = value()?;
                    ask.why = Some(RequestId::parse(&v).map_err(|e| format!("--why: {e}"))?);
                }
                "--origin" => {
                    let v = value()?;
                    ask.origin = Some(
                        v.parse()
                            .map_err(|_| format!("--origin takes a number, got {v:?}"))?,
                    );
                }
                "--control" => {
                    let v = value()?;
                    ask.control = Some(SemId::parse(&v).map_err(|e| format!("--control: {e}"))?);
                }
                "--at" => {
                    let v = value()?;
                    ask.at = Some(v.parse().map_err(|_| format!("--at takes ms, got {v:?}"))?);
                }
                "--export" => {
                    let mut ms = || -> Result<Millis, String> {
                        let v = value()?;
                        v.parse()
                            .map_err(|_| format!("--export takes <t0> <t1> in ms, got {v:?}"))
                    };
                    let (from, to) = (ms()?, ms()?);
                    let out = PathBuf::from(value()?);
                    if to < from {
                        return Err(format!("--export: {to} comes before {from}"));
                    }
                    ask.export = Some(Export { from, to, out });
                }
                flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
                other if path.is_none() => path = Some(PathBuf::from(other)),
                other => return Err(format!("one session at a time, got {other:?} too")),
            }
        }
        if ask.origin.is_some() && ask.why.is_none() {
            return Err("--origin goes with --why".to_owned());
        }
        if ask.at.is_some() && ask.control.is_none() {
            return Err("--at goes with --control".to_owned());
        }
        let asked = ask.timeline || ask.why.is_some() || ask.control.is_some() || ask.json;
        if ask.export.is_some() && asked {
            return Err("--export goes alone".to_owned());
        }
        match path {
            Some(path) => Ok(Command::Inspect(Ask { path, ..ask })),
            None if args.is_empty() => Ok(Command::Help),
            None => Err("the session file is missing".to_owned()),
        }
    }

    /// what an invocation answers, in the order it prints it; the json
    /// keeps each answer's own field order.
    #[derive(Default, serde::Serialize)]
    struct Answer {
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<Summary>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeline: Option<Timeline>,
        #[serde(skip_serializing_if = "Option::is_none")]
        why: Option<Why>,
        #[serde(skip_serializing_if = "Option::is_none")]
        control: Option<Control>,
    }

    impl Answer {
        fn text(&self) -> String {
            let parts: Vec<String> = [
                self.summary.as_ref().map(text::summary),
                self.timeline.as_ref().map(text::timeline),
                self.why.as_ref().map(text::why),
                self.control.as_ref().map(text::control),
            ]
            .into_iter()
            .flatten()
            .collect();
            parts.join("\n")
        }
    }

    fn inspect(ask: &Ask) -> i32 {
        let shown = ask.path.display();
        let session = match Session::open(&ask.path) {
            Ok(session) => session,
            Err(e) => {
                eprintln!("plev-inspect: {shown}: {e}");
                return READ_ERROR;
            }
        };
        if let Some(stretch) = &ask.export {
            return export(&session, &ask.path, stretch);
        }
        let (answer, code) = match answer(&session, ask) {
            Ok(answered) => answered,
            Err(Failure::Read(e)) => {
                eprintln!("plev-inspect: {shown}: {e}");
                return READ_ERROR;
            }
            Err(Failure::Usage(e)) => {
                eprintln!("plev-inspect: {e}");
                return USAGE_ERROR;
            }
        };
        if ask.json {
            match serde_json::to_string_pretty(&answer) {
                Ok(json) => println!("{json}"),
                Err(e) => {
                    eprintln!("plev-inspect: {shown}: {e}");
                    return READ_ERROR;
                }
            }
        } else {
            print!("{}", answer.text());
        }
        code
    }

    /// true when `out` names the file `input` does.
    fn same_file(input: &Path, out: &Path) -> bool {
        match (input.canonicalize(), out.canonicalize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    /// `+s.mmms`, the way every view prints a time from the start.
    fn stamp(ms: Millis) -> String {
        format!("+{}.{:03}s", ms / 1_000, ms % 1_000)
    }

    /// writes the stretch of `session` that `stretch` names to its own
    /// file, and says what went in it.
    fn export(session: &Session, input: &Path, stretch: &Export) -> i32 {
        let out = &stretch.out;
        if same_file(input, out) {
            eprintln!(
                "plev-inspect: --export would write over {}, the session it reads",
                input.display()
            );
            return USAGE_ERROR;
        }
        let start = session.start();
        let excerpt = session.excerpt(
            start.saturating_add(stretch.from),
            start.saturating_add(stretch.to),
        );
        let written = match excerpt.write(out) {
            Ok(written) => written,
            Err(e) => {
                eprintln!("plev-inspect: {}: {e}", out.display());
                return WRITE_ERROR;
            }
        };
        let opens = excerpt.start().saturating_sub(start);
        println!(
            "exported {} to {} of {} to {}",
            stamp(stretch.from),
            stamp(stretch.to),
            input.display(),
            out.display()
        );
        println!(
            "  opens at  {}, where the state and the frames that govern {} start",
            stamp(opens),
            stamp(stretch.from)
        );
        println!(
            "  events    {} ({} left out: no json form)",
            written.events, written.dropped
        );
        println!(
            "  frames    {} in {} chunks, {} assets",
            excerpt.frame_times().len(),
            written.chunks,
            written.assets
        );
        println!("  size      {} bytes", written.bytes);
        DONE
    }

    enum Failure {
        Read(ReadError),
        Usage(String),
    }

    impl From<ReadError> for Failure {
        fn from(e: ReadError) -> Self {
            Failure::Read(e)
        }
    }

    /// the answers `ask` asks for, and the exit code.
    fn answer(session: &Session, ask: &Ask) -> Result<(Answer, i32), Failure> {
        let inspector = Inspector::new(session);
        let mut out = Answer::default();
        let mut code = DONE;
        if !ask.timeline && ask.why.is_none() && ask.control.is_none() {
            out.summary = Some(inspector.summary()?);
        }
        if ask.timeline {
            out.timeline = Some(inspector.timeline());
        }
        if let Some(id) = ask.why {
            let origins = inspector.origins_of(id);
            let origin = match (ask.origin, origins.as_slice()) {
                (Some(origin), _) => Some(origin),
                (None, [one]) => Some(*one),
                (None, []) => None,
                (None, many) => {
                    return Err(Failure::Usage(format!(
                        "request {id} is in origins {many:?}; pass --origin"
                    )));
                }
            };
            out.why = match origin {
                Some(origin) => inspector.why(origin, id)?,
                None => None,
            };
            if out.why.is_none() {
                let origin = ask
                    .origin
                    .map_or(String::new(), |o| format!(" of origin {o}"));
                eprintln!("plev-inspect: the session holds no request {id}{origin}");
                code = NOT_FOUND;
            }
        }
        if let Some(id) = &ask.control {
            let t = ask
                .at
                .map_or(inspector.end(), |at| inspector.start().saturating_add(at));
            out.control = Some(inspector.control(id, t)?);
        }
        Ok((out, code))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn args(line: &str) -> Vec<String> {
            line.split_whitespace().map(str::to_owned).collect()
        }

        #[test]
        fn the_file_alone_is_a_summary_and_every_option_reads() {
            let Ok(Command::Inspect(plain)) = parse(&args("s.monster")) else {
                panic!("an inspection");
            };
            assert_eq!(plain.path, PathBuf::from("s.monster"));
            assert!(!plain.timeline && plain.why.is_none() && plain.control.is_none());
            let Ok(Command::Inspect(all)) = parse(&args(
                "--timeline s.monster --why 7 --origin 1 --control commit.submit --at 1500 --json",
            )) else {
                panic!("an inspection");
            };
            assert!(all.timeline && all.json);
            assert_eq!(all.why.map(|id| id.to_string()), Some("7".into()));
            assert_eq!(all.origin, Some(1));
            assert_eq!(
                all.control.map(|id| id.to_string()),
                Some("commit.submit".into())
            );
            assert_eq!(all.at, Some(1_500));
            let Ok(Command::Inspect(cut)) = parse(&args("s.monster --export 100 2500 out.monster"))
            else {
                panic!("an export");
            };
            assert_eq!(
                cut.export,
                Some(Export {
                    from: 100,
                    to: 2_500,
                    out: PathBuf::from("out.monster")
                })
            );
        }

        #[test]
        fn a_bad_invocation_says_why() {
            for (line, needle) in [
                ("s.monster --why", "needs a value"),
                ("s.monster --why seven", "--why"),
                ("s.monster --why 07", "--why"),
                ("s.monster --control Bad", "--control"),
                ("s.monster --at soon --control x", "--at"),
                ("s.monster --origin 1", "--origin goes with --why"),
                ("s.monster --at 5", "--at goes with --control"),
                ("s.monster --fast", "unknown option"),
                ("a.monster b.monster", "one session"),
                ("--timeline", "missing"),
                ("s.monster --export 1 2", "needs a value"),
                ("s.monster --export 1 soon x.monster", "--export takes"),
                ("s.monster --export 9 2 x.monster", "comes before"),
                ("s.monster --export 1 2 x.monster --timeline", "goes alone"),
                ("s.monster --json --export 1 2 x.monster", "goes alone"),
            ] {
                match parse(&args(line)) {
                    Err(e) => assert!(e.contains(needle), "{line}: {e:?} lacks {needle:?}"),
                    Ok(c) => panic!("{line}: {c:?}"),
                }
            }
            assert_eq!(parse(&[]), Ok(Command::Help));
            assert_eq!(parse(&args("s.monster --help")), Ok(Command::Help));
        }

        #[test]
        fn the_help_says_what_by_means_and_the_exit_codes() {
            let one_line = USAGE.split_whitespace().collect::<Vec<_>>().join(" ");
            for words in [
                "who moved a value is a correlation in time, not a declared write",
                "reading a session never runs anything",
                "exit: 0 answered, 1 the request is not in the session, 2 bad invocation, 3 \
                 the file does not read as a session, 4 the export could not be written",
            ] {
                assert!(one_line.contains(words), "{words:?} in {USAGE}");
            }
        }
    }
}
