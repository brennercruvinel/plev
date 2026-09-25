//! the prompt a language model is given: what it is asked, how the
//! surface should read, the rules the verifier applies, the components,
//! the app's registry in a form to read, the surface shown now with the
//! ids to keep, and the IR's JSON schema; and, after a refusal, the reply
//! and every error the verifier found in it.

use std::fmt::Write as _;

use sem::{Actors, Confirm, Node, Value};
use uir::{Component, IrError, IrNode, Named, Rule, schema};

use crate::Ask;

/// the longest a value is shown in the registry, in characters.
const VALUE_CHARS: usize = 60;
/// the longest a refused reply is quoted back, in characters.
pub(crate) const REPLY_CHARS: usize = 16_000;

/// the rules of the verifier, in its order: an exhaustive match, so a
/// rule the verifier gains is a rule the prompt must describe.
pub(crate) fn rule(r: Rule) -> &'static str {
    match r {
        Rule::Schema => {
            "the JSON has the shape of the schema below, no field it lacks, and `plev_ir` is 1"
        }
        Rule::Identity => {
            "every `sem` id is valid (name segments [a-z0-9_]+ joined by '.', a segment may carry a \
             bracketed key) and unique on the surface, the surface's id included; `[{item}]` only \
             inside a repeat's template, once per id; a node that takes the id of an app node \
             sends only that node's actions"
        }
        Rule::Vocabulary => {
            "a known component; props by token only; children, a layout or a repeat only on a \
             container; a split holds exactly two children; a split's ratio 10 to 90, a flex 1 \
             to 12"
        }
        Rule::State => {
            "every bound state exists in the app's capabilities below, and its kind is one the \
             component shows under that name; a repeat runs over a list or a set"
        }
        Rule::Action => {
            "every bound action exists in the app's capabilities below, and the event sends the \
             kind the action takes (an activation sends nothing)"
        }
        Rule::Policy => {
            "bind only actions the author may ask for; a destructive action only to a dialog's \
             `confirm`, unless its policy asks everyone to confirm"
        }
        Rule::Access => {
            "every control, dialog, image, chart, stat, status and progress has a label (a text \
             or a badge may bind its `text` instead); a separator has none; a control binds at \
             least one action; in an overlay, controls sit in one layer"
        }
        Rule::Limits => "nodes, depth and text within the policy below",
    }
}

pub(crate) const RULES: [Rule; 8] = [
    Rule::Schema,
    Rule::Identity,
    Rule::Vocabulary,
    Rule::State,
    Rule::Action,
    Rule::Policy,
    Rule::Access,
    Rule::Limits,
];

/// how a surface reads, for any app. the verifier checks none of it, and
/// the checkpoint b of task-69 found a model that passed every rule and
/// still put the task last, under the fold, and showed again the
/// controls the app's own screen has: guidance, not rules.
const READS: &str = "\
- first what the intent asks to do: the list to review, the action to run, the value to \
set. then what gives it context: a file, the counts, the settings. a person reads from \
the top, and the window may show only the top of the surface.
- every control has a label that says what it changes, with the unit when its value has \
one (the app's labels often carry it: keep them). a slider shows its label and its \
value; a number shows with at most two decimals.
- each control once, and only what the intent needs: the app's own screen already offers \
every capability below. a second control for an action already bound, or a state shown \
twice, needs a reason in the intent.
";

pub(crate) fn clip(text: &str, max: usize) -> String {
    let mut out: String = text.chars().take(max).collect();
    if text.chars().count() > max {
        out.push_str("...");
    }
    out
}

fn value(v: &Value) -> String {
    let text = match v {
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Num(n) => n.to_string(),
        Value::Text(t) => format!("{t:?}"),
        Value::List(items) => format!("{items:?}"),
        Value::Set(items) => format!("{items:?}"),
    };
    clip(&text, VALUE_CHARS)
}

fn who(actors: &Actors) -> Option<String> {
    let all = [
        (actors.human, "people"),
        (actors.agent, "agents"),
        (actors.assistive, "assistive technology"),
        (actors.test, "tests"),
    ];
    if all.iter().all(|(may, _)| *may) {
        return None;
    }
    let may: Vec<&str> = all.iter().filter(|(m, _)| *m).map(|(_, w)| *w).collect();
    Some(format!("only {} may ask", may.join(", ")))
}

pub(crate) fn components(out: &mut String) {
    out.push_str("- (no component): a layout container, role group; holds children.\n");
    let mut charts = false;
    for c in Component::ALL {
        if matches!(c, Component::Chart(_)) {
            if charts {
                continue;
            }
            charts = true;
        }
        let spec = c.spec();
        let name = match c {
            Component::Chart(_) => r#"{"chart": "line" | "bar" | "area" | "donut"}"#.to_owned(),
            c => c.name().to_owned(),
        };
        let role = spec.role.map_or("none", |r| r.as_str());
        let named = match spec.named {
            Named::Always => "needs a label".to_owned(),
            Named::OrBound(p) => format!("needs a label or its `{p}` bound"),
            Named::Optional => "a label is optional".to_owned(),
            Named::Never => "takes no label".to_owned(),
        };
        let shows: Vec<String> = spec
            .state
            .iter()
            .map(|p| {
                let kinds: Vec<&str> = p.kinds.iter().map(|k| k.as_str()).collect();
                format!("{} ({})", p.name, kinds.join(" or "))
            })
            .collect();
        let sends: Vec<String> = spec
            .events
            .iter()
            .map(|e| format!("{} ({})", e.name, e.param.map_or("nothing", |k| k.as_str())))
            .collect();
        let _ = write!(out, "- {name}: role {role}; {named}");
        if spec.container {
            out.push_str("; holds children");
        }
        if !shows.is_empty() {
            let _ = write!(out, "; shows {}", shows.join(", "));
        }
        if !sends.is_empty() {
            let _ = write!(out, "; sends {}", sends.join(", "));
        }
        out.push_str(".\n");
    }
    out.push_str(
        "a chip is a toggle when it shows `selected` or sends `toggle`, a button when it sends \
         `activate`.\n\
         an image shows a frame of the app's media when its `source` is bound to a text state \
         whose value names a media node and an instant, `<id>@<seconds>` (a frame ref): the \
         app decodes that frame. an image needs no other source.\n",
    );
}

fn node_line(out: &mut String, n: &Node) {
    let _ = write!(out, "{} ({}) {:?}", n.id, n.role.as_str(), n.label);
    if let Some(parent) = &n.parent {
        let _ = write!(out, ", in {parent}");
    }
    if n.constraints.secret {
        out.push_str(", secret: its value is only its length");
    }
    out.push('\n');
}

pub(crate) fn registry(out: &mut String, ask: &Ask<'_>) {
    for n in ask.app.nodes().values() {
        node_line(out, n);
        for (key, v) in &n.state {
            let at = sem::StateRef::new(n.id.clone(), key.clone());
            if let Some(d) = ask.caps.datum(&at) {
                let _ = writeln!(out, "  state {at}: {} = {}", d.kind.as_str(), value(v));
            }
        }
        for decl in &n.actions {
            let Some(cap) = ask.caps.action(&n.id, &decl.name) else {
                continue;
            };
            let takes = cap
                .param
                .map_or("nothing".to_owned(), |p| format!("a {}", p.kind.as_str()));
            let _ = write!(out, "  action {} {}: takes {takes}", n.id, decl.name);
            if cap.policy.destructive {
                out.push_str(", destructive");
            }
            match cap.policy.confirm {
                Confirm::Never => {}
                c => {
                    let _ = write!(out, ", confirm {}", c.as_str());
                }
            }
            if let Some(w) = who(&cap.policy.actors) {
                let _ = write!(out, ", {w}");
            }
            if !decl.enabled {
                out.push_str(", disabled now");
            }
            out.push('\n');
        }
    }
    let shown = |node: &sem::SemId| ask.app.get(node).is_some();
    for d in ask.caps.data().iter().filter(|d| !shown(&d.at.node)) {
        let _ = writeln!(out, "state {}: {}", d.at, d.kind.as_str());
    }
    for a in ask.caps.actions().iter().filter(|a| !shown(&a.target)) {
        let takes = a.param.map_or("nothing", |p| p.kind.as_str());
        let _ = writeln!(out, "action {} {}: takes {takes}", a.target, a.action);
    }
}

fn bound_nodes(root: &IrNode) -> Vec<&IrNode> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if !n.bind.is_empty() {
            out.push(n);
        }
        if let Some(repeat) = &n.repeat {
            stack.push(&repeat.item);
        }
        stack.extend(n.children.iter().rev());
    }
    out
}

fn current(out: &mut String, ask: &Ask<'_>) {
    let Some(verified) = ask.current else {
        out.push_str(
            "no surface is shown now. choose a surface id that no node of the app has and no \
             node of the app starts with.\n",
        );
        return;
    };
    let ir = verified.ir();
    let _ = writeln!(
        out,
        "its id is `{}`: keep it, and the surface changes form in place.",
        ir.surface.id
    );
    out.push_str(
        "keep the id of every node whose capability stays on the surface, so what a person \
         holds in it (focus, the caret, a drag) goes on; a new id is a node that leaves and \
         another that arrives:\n",
    );
    for n in bound_nodes(&ir.surface.root) {
        let component = n.component.map_or("container", Component::name);
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
        let _ = writeln!(out, "- `{}`: {component}, {}", n.sem, binds.join(", "));
    }
    let json = serde_json::to_string(ir).unwrap_or_default();
    let _ = writeln!(out, "its IR:\n{json}");
}

/// the prompt for `ask`: everything a model needs to propose an IR the
/// verifier accepts, and nothing it could grant itself.
pub fn prompt(ask: &Ask<'_>) -> String {
    let mut out = String::new();
    out.push_str(
        "you propose one surface of a plev app as a plev IR document: JSON that describes the \
         surface by meaning, by binding the app's own state and actions to components of a \
         closed vocabulary. the runtime verifies it before anything is shown; a document the \
         verifier refuses comes back to you with every error. answer with the IR only: one \
         JSON object, no prose.\n\n## the intent\n\n",
    );
    let _ = writeln!(out, "{}\n", ask.intent.trim());
    out.push_str("## how the surface reads\n\n");
    out.push_str(READS);
    out.push_str("\n## the verifier's rules\n\n");
    for (i, r) in RULES.iter().enumerate() {
        let _ = writeln!(out, "{}. {}: {}.", i + 1, r.as_str(), rule(*r));
    }
    let p = ask.policy;
    let _ = writeln!(
        out,
        "\nthis proposal is verified under: author {}, at most {} nodes, depth {}, {} characters \
         per title, label or description.\n",
        p.author, p.max_nodes, p.max_depth, p.max_text
    );
    out.push_str("## the components\n\n");
    components(&mut out);
    out.push_str(
        "\n## the app's capabilities\n\nthe only state and actions a surface may bind: each node \
         of the app with its role, its label and its parent, then its states (`state <ref>: \
         <kind> = <value now>`) and its actions (`action <target> <name>: takes <kind>`).\n\n",
    );
    registry(&mut out, ask);
    out.push_str("\n## the surface shown now\n\n");
    current(&mut out, ask);
    out.push_str("\n## the JSON schema of the IR\n\n");
    out.push_str(&serde_json::to_string_pretty(&schema()).unwrap_or_default());
    out.push('\n');
    out
}

/// the prompt after attempt `attempt` was refused: the first prompt, the
/// reply, and every error of the verifier at its JSON pointer.
pub(crate) fn repair(base: &str, attempt: usize, reply: &str, errors: &[IrError]) -> String {
    let mut out = String::from(base);
    let _ = writeln!(
        out,
        "\n## attempt {attempt} was refused\n\nyour answer:\n\n{}\n\nthe verifier's errors, \
         every one, each at its JSON pointer into your IR:\n",
        clip(reply.trim(), REPLY_CHARS)
    );
    for e in errors {
        let _ = writeln!(out, "- {e}");
    }
    out.push_str("\nanswer again with the whole corrected IR, one JSON object.\n");
    out
}
