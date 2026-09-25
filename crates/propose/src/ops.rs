//! the prompt of a change (task-70 phase 2): a language model is asked
//! to change the surface shown by the operations of `uir::transform`, and
//! answers `{"ops": [...]}`. the prompt carries the intent, how a change
//! reads, the nine operations as JSON with what each refuses, the rules
//! of identity, the verifier's rules and the policy (as the proposal's
//! prompt does), the components, the app's registry (the same text: a
//! secret as its length, a picture as the text that names it), and the
//! surface shown, its IR and what is hidden. after a refusal, the reply
//! and the operation refused, with why.

use std::fmt::Write as _;

use serde_json::Value;
use uir::Op;

use crate::adapt::Adapt;
use crate::prompt::{REPLY_CHARS, RULES, clip, components, registry, rule};
use crate::{Ask, ProposeError};

/// how a change reads, for any app: guidance, not rules.
const READS: &str = "\
- change what the intent asks about, and nothing else: the rest stays where the person left \
it.
- what a person holds in a node that stays (focus, the caret, a drag) goes on: an operation \
that keeps a node (group, reorder, emphasize, density) keeps that, one that hides it does not.
- hide only what the intent does not need: the app's own screen still offers it. to review \
items is to see what each one is: an item's picture, its name and its state stay.
- `{\"ops\": []}` is an answer: the intent asks for nothing these operations do.
";

/// the nine operations, their JSON and what each refuses.
const OPERATIONS: &str = "\
each is a JSON object with its name in `op`, applied in the order given, each over what the \
ones before made:

- `{\"op\": \"hide\", \"id\": \"<id>\"}`: the node leaves the surface, kept aside whole to show \
again. not the root, not a repeat's item, not a node hidden already.
- `{\"op\": \"show\", \"id\": \"<id>\"}`: a hidden node goes back where it was.
- `{\"op\": \"group\", \"ids\": [\"<id>\", ...], \"into\": \"<new id>\"}`: siblings become the \
children of a new container `into`, in their order, in the place of the first, laid out as \
their parent's stack.
- `{\"op\": \"reorder\", \"parent\": \"<id>\", \"ids\": [\"<id>\", ...]}`: `ids` first, in \
this order, then the other children as they were. not a repeat: its order is the app's.
- `{\"op\": \"emphasize\", \"id\": \"<id>\"}`: one step up the token the node is drawn by: a \
text a size up the type ramp, a button or an image a control size up, a container, a card or \
a list an elevation up. refused at the top, and for a component no token of emphasis draws.
- `{\"op\": \"density\", \"step\": \"denser\"}` or `\"sparser\"`: every gap and padding set, \
one step of the spacing scale.
- `{\"op\": \"restyle\", \"id\": \"<id>\", \"role\": \"<role>\"}`: a text's color (text, \
text_dim, text_mid, accent, success, danger, warning, info) or a container's or a list's fill \
(bg, surface, bg_panel, bg_hover, accent_dim).
- `{\"op\": \"swap_view\", \"id\": \"<id>\", \"component\": <component>}`: the same datum shown \
by another component of its family, every other binding kept: a text, a stat and a badge (one \
value); a checkbox, a switch and a chip (one bool); a select and tabs (a choice); a table and \
the charts (a list; a chart draws the numbers its items read as). never a list that repeats a \
template.
- `{\"op\": \"promote\", \"target\": \"<app node>\", \"action\": \"<action>\", \"into\": \
\"<container id>\"}`: a button for an action the app declares and the surface does not show, \
at the end of `into`, its id `<into>.<action>`. only an action whoever asks may ask for, and \
one that takes no parameter.
";

/// the rules of identity every change keeps.
const IDENTITY: &str = "\
- every id of the surface stays, in its tree or hidden: only `hide` takes one out of the tree.
- a new id (a group's `into`) is one no node has, shown or hidden.
- the IR holds a repeat's template once, its ids with `[{item}]`: an operation on the \
template acts on every item, and a new id inside it carries `[{item}]`. an item's own id \
(`list.row[3]`) is not a node of the IR, and is refused.
";

/// the prompt for `ask`: everything a model needs to answer operations
/// the verifier accepts, and nothing it could grant itself.
pub(crate) fn prompt(ask: &Adapt<'_>) -> String {
    let mut out = String::new();
    out.push_str(
        "you change one surface of a plev app, shown now, by small operations on its plev IR: \
         each is applied in order and verified again before anything is shown, and can be \
         undone. an operation the runtime refuses comes back to you with why. answer with the \
         operations only: one JSON object, `{\"ops\": [...]}`, no prose.\n\n## the intent\n\n",
    );
    let _ = writeln!(out, "{}\n", ask.intent.trim());
    out.push_str("## how a change reads\n\n");
    out.push_str(READS);
    out.push_str("\n## the operations\n\n");
    out.push_str(OPERATIONS);
    out.push_str("\n## identity\n\n");
    out.push_str(IDENTITY);
    out.push_str("\n## the verifier's rules\n\n");
    for (i, r) in RULES.iter().enumerate() {
        let _ = writeln!(out, "{}. {}: {}.", i + 1, r.as_str(), rule(*r));
    }
    let p = ask.policy;
    let _ = writeln!(
        out,
        "\nthe surface is verified under: author {}, at most {} nodes, depth {}, {} characters \
         per title, label or description. the change is asked by {}.\n",
        p.author, p.max_nodes, p.max_depth, p.max_text, ask.by
    );
    out.push_str("## the components\n\n");
    components(&mut out);
    out.push_str(
        "\n## the app's capabilities\n\nthe only state and actions a surface may bind: each node \
         of the app with its role, its label and its parent, then its states (`state <ref>: \
         <kind> = <value now>`) and its actions (`action <target> <name>: takes <kind>`).\n\n",
    );
    let surface = Ask {
        intent: ask.intent,
        caps: ask.caps,
        app: ask.app,
        current: Some(ask.current.verified()),
        policy: ask.policy,
    };
    registry(&mut out, &surface);
    out.push_str("\n## the surface shown now\n\n");
    let ir = ask.current.ir();
    let _ = writeln!(out, "its id is `{}`. its IR:", ir.surface.id);
    let _ = writeln!(out, "{}", serde_json::to_string(ir).unwrap_or_default());
    let hidden: Vec<String> = ask
        .current
        .aside()
        .map(|a| format!("- `{}`, from `{}`", a.node.sem, a.parent))
        .collect();
    if hidden.is_empty() {
        out.push_str("\nnothing is hidden.\n");
    } else {
        let _ = writeln!(
            out,
            "\nhidden now (`show` brings one back):\n{}",
            hidden.join("\n")
        );
    }
    out
}

/// the operations in a reply: the object `{"ops": [...]}`, alone or
/// inside a fence or prose (from the first `{` to the last `}`), each one
/// read as `uir::Op` reads it (an unknown field is an error).
pub(crate) fn read(reply: &str) -> Result<Vec<Op>, String> {
    let text = reply.trim();
    let inner = match (text.find('{'), text.rfind('}')) {
        (Some(open), Some(close)) if open < close => &text[open..=close],
        _ => text,
    };
    let shape = "the answer is not one object `{\"ops\": [...]}`";
    let value: Value = serde_json::from_str(inner).map_err(|e| format!("{shape}: {e}"))?;
    let Some(object) = value.as_object() else {
        return Err(shape.to_owned());
    };
    if let Some(other) = object.keys().find(|k| *k != "ops") {
        return Err(format!("{shape}: it has `{other}`"));
    }
    let Some(ops) = object.get("ops").and_then(Value::as_array) else {
        return Err(format!("{shape}: `ops` is not a list"));
    };
    ops.iter()
        .enumerate()
        .map(|(at, op)| {
            serde_json::from_value::<Op>(op.clone())
                .map_err(|e| format!("operation {} is no operation: {e}", at + 1))
        })
        .collect()
}

/// the prompt after attempt `attempt` was refused: the first prompt, the
/// reply, and why.
pub(crate) fn repair(base: &str, attempt: usize, reply: &str, why: &[String]) -> String {
    let mut out = String::from(base);
    let _ = writeln!(
        out,
        "\n## attempt {attempt} was refused\n\nyour answer:\n\n{}\n\nwhy, every reason:\n",
        clip(reply.trim(), REPLY_CHARS)
    );
    for w in why {
        let _ = writeln!(out, "- {w}");
    }
    out.push_str(
        "\nthe operations before the one refused applied; none after it was tried. answer again \
         with the whole list, corrected, one JSON object `{\"ops\": [...]}`.\n",
    );
    out
}

/// the reasons an operation `op`, the `at`th, was refused: one per error
/// of the verifier, or the refusal.
pub(crate) fn reasons(at: usize, op: &Op, error: &uir::TransformError) -> Vec<String> {
    let head = format!("operation {at}, `{op}`, was refused");
    match error {
        uir::TransformError::Unverified(errors) => errors
            .iter()
            .map(|e| format!("{head}: the surface it makes does not verify: {e}"))
            .collect(),
        other => vec![format!("{head}: {other}")],
    }
}

/// the error of a change no answer made: the reasons of the last one.
pub(crate) fn unchanged(attempts: usize, errors: Vec<String>) -> ProposeError {
    ProposeError::Unchanged { attempts, errors }
}
