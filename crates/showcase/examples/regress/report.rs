//! What a check found, told by control first.

use sem::{Change, Value};

use crate::compare::{Delta, LayerDelta, Status};
use crate::pixels::Pixels;
use crate::run::Outcome;
use crate::scene::{self, BindRecord, Scene};

/// The node a semantic change is about (`None`: the page, for the focus
/// and the order of the roots), and the change in a few words.
pub fn describe(change: &Change) -> (Option<String>, String) {
    let id = |id: &sem::SemId| Some(id.to_string());
    match change {
        Change::NodeAdded(node) => (
            id(&node.id),
            format!("added ({} \"{}\")", node.role.as_str(), node.label),
        ),
        Change::NodeRemoved(node) => (id(node), "removed".to_string()),
        Change::StateChanged {
            at, before, after, ..
        } => (
            id(&at.node),
            format!(
                "{} {} -> {}",
                at.key,
                value(before.as_ref()),
                value(after.as_ref())
            ),
        ),
        Change::LabelChanged {
            node,
            before,
            after,
        } => (id(node), format!("label \"{before}\" -> \"{after}\"")),
        Change::ActionsChanged {
            node,
            before,
            after,
        } => {
            let names = |a: &[sem::ActionDecl]| {
                a.iter()
                    .map(|d| d.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            (
                id(node),
                format!("actions [{}] -> [{}]", names(before), names(after)),
            )
        }
        Change::FlagsChanged {
            node,
            enabled,
            inert,
            focusable,
        } => {
            let flags: Vec<String> = [
                ("enabled", enabled),
                ("inert", inert),
                ("focusable", focusable),
            ]
            .iter()
            .filter_map(|(name, v)| v.map(|v| format!("{name} -> {v}")))
            .collect();
            (id(node), flags.join(", "))
        }
        Change::ParentChanged {
            node,
            before,
            after,
        } => (
            id(node),
            format!("parent {} -> {}", opt(before), opt(after)),
        ),
        Change::RoleChanged {
            node,
            before,
            after,
        } => (
            id(node),
            format!("role {} -> {}", before.as_str(), after.as_str()),
        ),
        Change::DescriptionChanged { node, .. } => (id(node), "description changed".to_string()),
        Change::ConstraintsChanged { node, .. } => (id(node), "constraints changed".to_string()),
        Change::Reordered { parent: Some(p) } => (id(p), "children reordered".to_string()),
        Change::Reordered { parent: None } => (None, "roots reordered".to_string()),
        Change::FocusChanged { before, after } => {
            (None, format!("focus {} -> {}", opt(before), opt(after)))
        }
    }
}

fn opt(id: &Option<sem::SemId>) -> String {
    id.as_ref().map_or("none".to_string(), ToString::to_string)
}

/// A state value as its JSON (`true`, `3`, `"text"`), `none` when absent.
fn value(v: Option<&Value>) -> String {
    let Some(v) = v else {
        return "none".to_string();
    };
    match v {
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Num(n) => n.to_string(),
        Value::Text(t) => format!("{t:?}"),
        Value::List(items) => format!("{items:?}"),
        Value::Set(items) => format!("{items:?}"),
    }
}

/// The lines under a baseline's name when something moved: the meaning
/// first, by control, then what is drawn.
pub fn lines(delta: &Delta) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(note) = delta.semantics.note {
        out.push(format!("semantics: {note}"));
    }
    for change in &delta.semantics.changes {
        let (node, what) = describe(change);
        out.push(format!(
            "{}: semantics changed: {what}",
            node.as_deref().unwrap_or("(page)")
        ));
    }
    for layer in delta.changed_layers() {
        let hash = |r: &Option<crate::scene::LayerRecord>| {
            r.as_ref()
                .map_or("-".to_string(), |l| format!("{:016x}", l.hash))
        };
        let what = match layer.status {
            Status::Changed => "changed",
            Status::Added => "added",
            Status::Removed => "removed",
            Status::Unchanged => "unchanged",
        };
        out.push(format!(
            "layer {}: {what} ({} -> {})",
            layer.name,
            hash(&layer.before),
            hash(&layer.after)
        ));
    }
    for b in &delta.bindings {
        let rect = |r: &Option<crate::scene::BindRecord>| {
            r.as_ref().map_or("unbound".to_string(), |b| {
                format!("{} {}", b.layer, region(b.rect))
            })
        };
        out.push(format!(
            "{}: bound at {} -> {}",
            b.id,
            rect(&b.before),
            rect(&b.after)
        ));
    }
    out
}

/// The pixel part of a check whose layers moved: how many changed, in
/// each layer's region, and in each bound control.
pub fn pixel_lines(o: &Outcome, base: &Scene) -> Vec<String> {
    let c = match &o.pixels {
        Pixels::NotNeeded => return Vec::new(),
        Pixels::NoBaselineImage => {
            return vec![
                "visual changed, no baseline image to diff (bless --png keeps one)".to_string(),
            ];
        }
        Pixels::NoAdapter => {
            return vec![
                "visual changed, no GPU adapter here to draw the frame; pixels not compared"
                    .to_string(),
            ];
        }
        Pixels::Diffed(c) => c,
    };
    let total = c.total();
    let mut out = vec![format!(
        "pixels: {} changed of {} compared{}",
        total.pixels,
        c.compared,
        at(total.region)
    )];
    for layer in o.delta.changed_layers() {
        if let Some(r) = layer_region(layer) {
            let h = c.within(r);
            out.push(format!(
                "layer {}: {} px{}",
                layer.name,
                h.pixels,
                at(h.region)
            ));
        }
    }
    for (b, drawn) in drawn(&o.now.bindings, &base.bindings) {
        let h = c.within(drawn);
        if h.pixels > 0 {
            out.push(format!(
                "{}: {} px{} (layer {})",
                b.id,
                h.pixels,
                at(h.region),
                b.layer
            ));
        }
    }
    out
}

/// Where a layer that moved drew, on either side.
pub fn layer_region(layer: &LayerDelta) -> Option<[f32; 4]> {
    let before = layer.before.as_ref().and_then(|l| l.bounds);
    let after = layer.after.as_ref().and_then(|l| l.bounds);
    match (before, after) {
        (Some(a), Some(b)) => Some(scene::union(a, b)),
        (a, b) => a.or(b),
    }
}

/// Every bound control of the page, then the ones only the baseline
/// bound, each with the part of its rect its clip lets through.
pub fn drawn<'a>(now: &'a [BindRecord], base: &'a [BindRecord]) -> Vec<(&'a BindRecord, [f32; 4])> {
    let gone = base.iter().filter(|b| !now.iter().any(|n| n.id == b.id));
    now.iter()
        .chain(gone)
        .filter_map(|b| {
            let r = match b.clip {
                Some(c) => scene::intersect(b.rect, c)?,
                None => b.rect,
            };
            Some((b, r))
        })
        .collect()
}

fn at(r: Option<[f32; 4]>) -> String {
    r.map_or(String::new(), |r| format!(" at {}", region(r)))
}

/// A logical rect as `x,y wxh`.
pub fn region(r: [f32; 4]) -> String {
    format!("{},{} {}x{}", r[0], r[1], r[2], r[3])
}

#[cfg(test)]
mod tests {
    use super::*;
    use sem::{SemId, StateRef};

    fn id(text: &str) -> SemId {
        match SemId::parse(text) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn a_change_is_told_by_the_node_it_is_about() {
        let label = Change::LabelChanged {
            node: id("todo.item[2]"),
            before: "Walk the dog".to_string(),
            after: "Walk the cat".to_string(),
        };
        assert_eq!(
            describe(&label),
            (
                Some("todo.item[2]".to_string()),
                "label \"Walk the dog\" -> \"Walk the cat\"".to_string()
            )
        );
        let at: StateRef = match "todo.item[2]:completed".parse() {
            Ok(at) => at,
            Err(e) => panic!("{e}"),
        };
        let state = Change::StateChanged {
            at,
            before: Some(Value::Bool(false)),
            after: Some(Value::Bool(true)),
            added: vec![],
            removed: vec![],
        };
        assert_eq!(describe(&state).1, "completed false -> true");
        let focus = Change::FocusChanged {
            before: None,
            after: Some(id("todo.input")),
        };
        assert_eq!(
            describe(&focus),
            (None, "focus none -> todo.input".to_string())
        );
    }
}
