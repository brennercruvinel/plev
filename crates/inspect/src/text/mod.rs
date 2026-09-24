//! every answer as plain english, the way `plev-inspect` prints it. times
//! are from the start of the session (`+111.744s`); revisions are their
//! 16 hex digits.
//!
//! the causal chain of a request answers the questions of the thesis, one
//! labelled line each: which control, its state, who initiated it, which
//! action, what the actor observed, the precondition, what came before,
//! the verdict, what changed, the result.

use std::fmt::Write as _;

use sem::{Millis, Node, Rejection, RequestId, SemId};

use crate::control::Control;
use crate::describe;
use crate::summary::Summary;
use crate::timeline::{Tally, Timeline, What};
use crate::why::{Ending, Why};

mod chain;

pub use chain::why;
use chain::{by, refusal};

/// a time from the start of the session: `+111.744s`.
pub fn stamp(at: Millis) -> String {
    format!("+{}.{:03}s", at / 1_000, at % 1_000)
}

/// how many changes of one event a line shows before it says how many
/// more there are.
const SHOWN: usize = 3;

/// the build, the span, the actors, every request with how it ended, and
/// every refusal with a short why.
pub fn summary(s: &Summary) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "session   rec {}, built from {}", s.version, s.git);
    let _ = writeln!(
        out,
        "span      {} to {}, {} events, {} frames",
        stamp(0),
        stamp(s.duration),
        s.events,
        s.frames
    );
    tallies(&mut out, "actors    ", &s.actors);
    let _ = writeln!(out, "requests  {}", s.requests.len());
    let rows: Vec<[String; 5]> = s
        .requests
        .iter()
        .map(|line| {
            let ending = match &line.detail {
                Some(detail) => format!("{}: {detail}", line.ending),
                None => line.ending.to_owned(),
            };
            [
                stamp(line.at),
                request(line.origin, line.id),
                line.actor.to_string(),
                format!("{} on {}", line.action, line.target),
                ending,
            ]
        })
        .collect();
    let width = |col: usize| rows.iter().map(|r| r[col].len()).max().unwrap_or(0);
    let (w0, w1, w2, w3) = (width(0), width(1), width(2), width(3));
    for [at, id, actor, what, ending] in &rows {
        let _ = writeln!(
            out,
            "  {at:<w0$}  {id:<w1$}  {actor:<w2$}  {what:<w3$}  {ending}"
        );
    }
    let _ = writeln!(out, "refusals  {}", s.refusals.len());
    for why in &s.refusals {
        short(&mut out, why);
    }
    out
}

/// a refusal on a few lines: what was refused, why, and who moved what it
/// read.
fn short(out: &mut String, why: &Why) {
    let Ending::Rejected { at, rejection, .. } = &why.ending else {
        return;
    };
    let _ = writeln!(
        out,
        "  {}, {} on {} by {}, rejected at {}: {}",
        request(why.origin, why.id),
        why.request.action,
        why.request.target,
        why.request.actor,
        stamp(*at),
        describe::rejection(rejection)
    );
    match rejection {
        Rejection::Stale { changes, .. } => {
            for change in changes {
                let _ = writeln!(out, "    {}", describe::change(change));
            }
        }
        other => {
            let _ = writeln!(out, "    {}", refusal(other));
        }
    }
    for read in &why.between.reads {
        for moved in &read.moves {
            let _ = writeln!(
                out,
                "    {} moved at {} {}",
                read.at,
                stamp(moved.at),
                by(&moved.by)
            );
        }
    }
}

/// every event, one line each, after the tallies.
pub fn timeline(t: &Timeline) -> String {
    let mut out = String::new();
    let span = t.end.saturating_sub(t.start);
    let _ = writeln!(
        out,
        "session   {} to {}, {} events",
        stamp(0),
        stamp(span),
        t.entries.len()
    );
    let kinds: Vec<String> = t
        .kinds
        .iter()
        .map(|c| format!("{} {}", c.kind, c.count))
        .collect();
    let _ = writeln!(out, "kinds     {}", kinds.join(", "));
    tallies(&mut out, "actors    ", &t.actors);
    let mut keyed = false;
    for entry in &t.entries {
        let line = match &entry.what {
            What::Observed {
                actor,
                scope,
                revision,
            } => format!(
                "{actor} observed {}, revision {revision}",
                scope_text(scope.as_ref())
            ),
            What::Snapshot { nodes, changes } => {
                let what = if changes.is_empty() {
                    if keyed { ", nothing changed" } else { "" }.to_owned()
                } else {
                    format!(": {}", few(changes))
                };
                keyed = true;
                format!("keyframe, {nodes} nodes{what}")
            }
            What::Changes { changes } => few(changes),
            What::Stage {
                origin,
                id,
                actor,
                target,
                action,
                stage,
                detail,
            } => {
                let detail = detail.as_ref().map_or(String::new(), |d| format!(": {d}"));
                format!(
                    "{} {stage}{detail} ({actor}, {action} on {target})",
                    request(*origin, *id)
                )
            }
            What::Input { input, target } => match target {
                Some(target) => format!("{} on {target}", input_name(*input)),
                None => input_name(*input),
            },
        };
        let _ = writeln!(
            out,
            "{:<11} {:<9} {line}",
            stamp(entry.at),
            entry.what.kind()
        );
    }
    out
}

/// a control at an instant, then the chain of every request that touched
/// it up to then.
pub fn control(c: &Control) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "control {} at {}", c.id, stamp(c.at));
    match &c.node {
        Some(node) => {
            let focus = if c.focused { ", focused" } else { "" };
            field(
                &mut out,
                "node",
                &format!("{}{focus}", control_line(&c.id, Some(node))),
            );
            field(&mut out, "state", &state_line(Some(node)));
            let actions: Vec<String> = node
                .actions
                .iter()
                .map(|a| {
                    let reads: Vec<String> = a.reads.iter().map(ToString::to_string).collect();
                    let off = if a.enabled { "" } else { ", disabled" };
                    if reads.is_empty() {
                        format!("{}{off}", a.name)
                    } else {
                        format!("{} (reads {}){off}", a.name, reads.join(", "))
                    }
                })
                .collect();
            let actions = if actions.is_empty() {
                "none".to_owned()
            } else {
                actions.join("; ")
            };
            field(&mut out, "actions", &actions);
            let source = match &node.source {
                Some(s) => format!("{}:{}:{}, in build {}", s.file, s.line, s.column, c.build),
                None => format!("not recorded (build {})", c.build),
            };
            field(&mut out, "source", &source);
        }
        None => field(
            &mut out,
            "node",
            "not in the recorded state at that instant",
        ),
    }
    field(
        &mut out,
        "requests",
        &format!(
            "{} touched it up to {} (their target, or a state they read)",
            c.receipts.len(),
            stamp(c.at)
        ),
    );
    for chain in &c.receipts {
        out.push('\n');
        for line in why(chain).lines() {
            let _ = writeln!(out, "  {line}");
        }
    }
    out
}

fn tallies(out: &mut String, label: &str, tallies: &[Tally]) {
    if tallies.is_empty() {
        let _ = writeln!(out, "{label}none");
        return;
    }
    for (i, t) in tallies.iter().enumerate() {
        let mut parts = Vec::new();
        for (n, one, many) in [
            (t.observations, "observation", "observations"),
            (t.requests, "request", "requests"),
            (t.stages, "stage", "stages"),
            (t.inputs, "input", "inputs"),
        ] {
            if n > 0 {
                parts.push(format!("{n} {}", if n == 1 { one } else { many }));
            }
        }
        let pad = if i == 0 {
            label.to_owned()
        } else {
            " ".repeat(label.len())
        };
        let _ = writeln!(out, "{pad}{}: {}", t.actor, parts.join(", "));
    }
}

const LABEL: usize = 14;

fn field(out: &mut String, label: &str, text: &str) {
    let _ = writeln!(out, "  {label:<LABEL$}{text}");
}

/// a line that goes on with the field above it.
fn cont(out: &mut String, text: &str) {
    let _ = writeln!(out, "  {:<LABEL$}{text}", "");
}

/// an entry of the list the field above it opens.
fn item(out: &mut String, text: &str) {
    let _ = writeln!(out, "  {:<LABEL$}  {text}", "");
}

/// a line under an entry that starts with a time.
fn deeper(out: &mut String, text: &str) {
    let _ = writeln!(out, "  {:<LABEL$}            {text}", "");
}

fn request(origin: u16, id: RequestId) -> String {
    if origin == 0 {
        format!("request {id}")
    } else {
        format!("request {id} (origin {origin})")
    }
}

fn scope_text(scope: Option<&SemId>) -> String {
    match scope {
        Some(scope) => format!("the subtree {scope}"),
        None => "the whole app".to_owned(),
    }
}

fn input_name(input: rec::InputKind) -> String {
    match serde_json::to_value(input) {
        Ok(serde_json::Value::String(name)) => name,
        _ => format!("{input:?}"),
    }
}

fn few(changes: &[String]) -> String {
    let mut shown: Vec<&str> = changes.iter().take(SHOWN).map(String::as_str).collect();
    let rest = changes.len().saturating_sub(SHOWN);
    let tail = format!("and {rest} more");
    if rest > 0 {
        shown.push(&tail);
    }
    shown.join("; ")
}

fn control_line(id: &SemId, node: Option<&Node>) -> String {
    match node {
        Some(node) => {
            let on = if node.enabled { "enabled" } else { "disabled" };
            let inert = if node.inert { ", inert" } else { "" };
            format!(
                "{id}: {} {:?}, {on}{inert}",
                describe::role(node.role),
                node.label
            )
        }
        None => format!("{id}, not in the recorded state right before the request"),
    }
}

fn state_line(node: Option<&Node>) -> String {
    match node {
        Some(node) if node.state.is_empty() => "none of its own".to_owned(),
        Some(node) => node
            .state
            .iter()
            .map(|(key, v)| format!("{key} = {}", describe::value(v)))
            .collect::<Vec<_>>()
            .join(", "),
        None => "unknown".to_owned(),
    }
}
