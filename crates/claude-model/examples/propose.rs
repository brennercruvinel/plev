//! an intent in text becomes a verified surface over a real projection,
//! proposed by the Claude:
//!
//! ```text
//! cargo run -p claude-model --example propose -- [--app todo|ide|silences] [--shown <ir.json>] [--model <name>] [--timeout <s>] "<intent>"
//! ```
//!
//! it reads the golden projection of the app (`crates/uir/fixtures`: the
//! showcase todo, with the compact surface of the Surface page shown, the
//! ide's workbench, with no surface shown, or the showcase silences, with
//! no surface shown unless `--shown` names an IR file to verify and show
//! as the current one), says what the rules make
//! of the intent (`propose::Rules`, for comparison), asks the Claude through
//! `propose::Llm` (the real `claude -p`, or what `PLEV_CLAUDE` names) and
//! `propose_verified`, and prints every attempt (its time, its cost, and
//! the verifier's errors when it was refused), the total, and the
//! verified IR, summed up and whole. exit 0 with a verified IR, 1 without
//! one, 2 on a usage error.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use claude_model::ClaudeCli;
use propose::{Ask, Llm, Model, ModelError, Rules, prompt, propose_verified};
use sem::Snapshot;
use uir::{
    Capabilities, Component, GenerationPolicy, IrNode, VerifiedIr, parse, verify, verify_json,
};

const USAGE: &str = "usage: propose [--app todo|ide|silences] [--shown <ir.json>] [--model <name>] \
                     [--timeout <seconds>] \"<intent>\"";

fn fixture(name: &str) -> Result<String, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uir/fixtures")
        .join(name);
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// what the command line asks.
struct Args {
    app: String,
    /// an IR file to show as the current surface, in place of the app's
    /// own.
    shown: Option<PathBuf>,
    model: Option<String>,
    timeout: Option<Duration>,
    intent: String,
}

fn args() -> Result<Args, String> {
    let mut out = Args {
        app: "todo".to_owned(),
        shown: None,
        model: None,
        timeout: None,
        intent: String::new(),
    };
    let mut all = std::env::args().skip(1);
    while let Some(arg) = all.next() {
        let mut value = || all.next().ok_or(format!("{arg} takes a value"));
        match arg.as_str() {
            "--app" => out.app = value()?,
            "--shown" => out.shown = Some(PathBuf::from(value()?)),
            "--model" => out.model = Some(value()?),
            "--timeout" => {
                let secs: u64 = value()?.parse().map_err(|e| format!("--timeout: {e}"))?;
                out.timeout = Some(Duration::from_secs(secs));
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ if out.intent.is_empty() => out.intent = arg,
            _ => return Err("one intent, in quotes".to_owned()),
        }
    }
    if out.intent.trim().is_empty() {
        return Err("no intent".to_owned());
    }
    Ok(out)
}

/// the app's golden projection, and the surface it shows (the todo's
/// compact form; the ide and the silences show none).
fn app(name: &str) -> Result<(Snapshot, Option<&'static str>), String> {
    let (snapshot, shown) = match name {
        "todo" => ("showcase-todo.snapshot.json", Some("showcase-todo.ir.json")),
        "ide" => ("ide-workbench.snapshot.json", None),
        "silences" => ("showcase-silences.snapshot.json", None),
        other => return Err(format!("no app {other}: todo, ide or silences")),
    };
    let app = serde_json::from_str(&fixture(snapshot)?).map_err(|e| format!("{snapshot}: {e}"))?;
    Ok((app, shown))
}

/// the model, and every answer it gave, in order.
struct Seen<'a> {
    cli: &'a mut ClaudeCli,
    replies: Vec<Result<String, ModelError>>,
}

impl Model for Seen<'_> {
    fn complete(&mut self, prompt: &str) -> Result<String, ModelError> {
        let reply = self.cli.complete(prompt);
        self.replies.push(reply.clone());
        reply
    }
}

/// what the verifier said of `reply`, read as `Llm` reads it (the object
/// inside prose or a fence): nothing, or every error.
fn refusals(reply: &str, ask: &Ask<'_>) -> Vec<String> {
    let text = reply.trim();
    let inner = match (text.find('{'), text.rfind('}')) {
        (Some(open), Some(close)) if open < close => &text[open..=close],
        _ => text,
    };
    let checked = parse(inner).and_then(|ir| verify(&ir, ask.caps, ask.policy).map(|_| ()));
    checked
        .err()
        .unwrap_or_default()
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// the nodes of a tree, a repeat's template once.
fn size(n: &IrNode) -> usize {
    1 + n.children.iter().map(size).sum::<usize>() + n.repeat.as_ref().map_or(0, |r| size(&r.item))
}

fn tree(out: &mut String, n: &IrNode, depth: usize) {
    let pad = "  ".repeat(depth);
    let component = n.component.map_or("container", Component::name);
    let mut line = format!("{pad}{} ({component}", n.sem);
    if let Some(label) = &n.label {
        line.push_str(&format!(", {label:?}"));
    }
    if let Some(layout) = n
        .layout
        .as_ref()
        .and_then(|l| serde_json::to_string(l).ok())
    {
        line.push_str(&format!(", {layout}"));
    }
    line.push(')');
    let mut binds: Vec<String> = n
        .bind
        .state
        .iter()
        .map(|(k, r)| format!("{k} <- {r}"))
        .collect();
    binds.extend(
        n.bind
            .actions
            .iter()
            .map(|(e, c)| format!("{e} -> {} {}", c.target, c.action)),
    );
    if !binds.is_empty() {
        line.push_str(": ");
        line.push_str(&binds.join(", "));
    }
    out.push_str(&line);
    out.push('\n');
    if let Some(repeat) = &n.repeat {
        out.push_str(&format!("{pad}  repeat over {}:\n", repeat.each));
        tree(out, &repeat.item, depth + 2);
    }
    for c in &n.children {
        tree(out, c, depth + 1);
    }
}

fn usd(cost: Option<f64>) -> String {
    cost.map_or_else(|| "cost unknown".to_owned(), |c| format!("{c:.3} USD"))
}

fn report(ir: &VerifiedIr) -> String {
    let surface = &ir.ir().surface;
    let mut out = format!("surface {} {:?}\n", surface.id, surface.title);
    tree(&mut out, &surface.root, 0);
    out
}

fn run() -> Result<bool, String> {
    let args = args()?;
    let (app, shown) = app(&args.app)?;
    let caps = Capabilities::from_snapshot(&app);
    let policy = GenerationPolicy::default();
    let current = match (&args.shown, shown) {
        (Some(path), _) => {
            let text =
                std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            Some(
                verify_json(&text, &caps, &policy)
                    .map_err(|e| format!("{} does not verify: {e:?}", path.display()))?,
            )
        }
        (None, Some(name)) => Some(
            verify_json(&fixture(name)?, &caps, &policy)
                .map_err(|e| format!("{name} does not verify: {e:?}"))?,
        ),
        (None, None) => None,
    };
    let ask = Ask {
        intent: &args.intent,
        caps: &caps,
        app: &app,
        current: current.as_ref(),
        policy: &policy,
    };
    let mut cli = ClaudeCli::new();
    if let Some(model) = args.model {
        cli.model = model;
    }
    if let Some(timeout) = args.timeout {
        cli.timeout = timeout;
    }
    let shown_name = match &args.shown {
        Some(path) => path.display().to_string(),
        None => shown.unwrap_or("none").to_owned(),
    };
    println!("app {}, surface shown: {shown_name}", args.app);
    println!("intent: {:?}", args.intent);
    match propose_verified(&mut Rules, &ask) {
        Ok(ir) => println!(
            "the rules: a verified surface of {} nodes",
            size(&ir.ir().surface.root)
        ),
        Err(e) => println!("the rules: {e}"),
    }
    println!(
        "model {} through {} (timeout {} s, at most {} attempts), a prompt of {} bytes\n",
        cli.model,
        cli.program.display(),
        cli.timeout.as_secs(),
        Llm::<ClaudeCli>::ATTEMPTS,
        prompt(&ask).len()
    );

    let started = Instant::now();
    let mut seen = Seen {
        cli: &mut cli,
        replies: Vec::new(),
    };
    let proposed = propose_verified(&mut Llm::new(&mut seen), &ask);
    let took = started.elapsed();
    let replies = seen.replies;
    for (i, (call, reply)) in cli.calls().iter().zip(&replies).enumerate() {
        let head = format!(
            "attempt {}: {:.1} s, {}",
            i + 1,
            call.took.as_secs_f64(),
            usd(call.cost_usd)
        );
        match reply {
            Err(e) => println!("{head}: the model failed: {e}"),
            Ok(text) => {
                let errors = refusals(text, &ask);
                if errors.is_empty() {
                    println!("{head}, {} bytes: verified", text.len());
                } else {
                    println!(
                        "{head}, {} bytes: refused, {} errors",
                        text.len(),
                        errors.len()
                    );
                    for e in errors {
                        println!("  - {e}");
                    }
                }
            }
        }
    }
    println!();
    let total = format!(
        "{} attempts, {:.1} s, {}",
        cli.calls().len(),
        took.as_secs_f64(),
        usd(cli
            .calls()
            .iter()
            .any(|c| c.cost_usd.is_some())
            .then(|| cli.spent_usd()))
    );
    match proposed {
        Ok(ir) => {
            println!("verified: {total}\n");
            print!("{}", report(&ir));
            let json = serde_json::to_string_pretty(ir.ir()).map_err(|e| e.to_string())?;
            println!("\nthe IR:\n{json}");
            Ok(true)
        }
        Err(e) => {
            println!("no surface: {total}\n{e}");
            Ok(false)
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("{e}\n{USAGE}");
            ExitCode::from(2)
        }
    }
}
