//! What a check found, told by control first: for every `SemId` the page
//! projects or binds, whether its meaning changed, whether what it draws
//! changed, in which layer, and, when pixels were compared, how many
//! changed and where. Then the layers, and what fell outside every
//! control. As text for a person, as JSON for a tool.

use sem::{Change, Snapshot, Value};
use serde::Serialize;

use crate::compare::{LayerDelta, Status};
use crate::pixels::{Hits, Pixels};
use crate::run::Outcome;
use crate::scene::{self, BindRecord, Scene};

/// One baseline's report.
#[derive(Debug, Serialize)]
pub struct Report {
    pub baseline: String,
    pub changed: bool,
    /// Semantic changes about the page, not a node: the focus, the order
    /// of the roots, a projection on one side only.
    pub page: Vec<String>,
    pub pixels: PixelReport,
    pub controls: Vec<ControlReport>,
    pub layers: Vec<LayerReport>,
    pub timings_ms: Timings,
}

#[derive(Debug, Serialize)]
pub struct PixelReport {
    /// `not_needed` (no layer moved, no frame drawn), `no_baseline_image`,
    /// `no_adapter` or `diffed`.
    pub status: &'static str,
    pub changed: Option<u64>,
    pub compared: Option<u64>,
    pub region: Option<[f32; 4]>,
}

/// What became of a control's drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Visual {
    Unchanged,
    Changed,
    /// Its layer moved and no pixels could be compared (no baseline
    /// image, no adapter): changed as far as anyone can tell here.
    LayerChanged,
    /// Projected but bound in neither frame: nothing drawn to compare.
    Unbound,
}

#[derive(Debug, Serialize)]
pub struct ControlReport {
    pub id: String,
    /// Each semantic change about this node, in a few words.
    pub semantics: Vec<String>,
    /// The same changes as `sem` serializes them (a node added carries
    /// no `source`).
    pub delta: Vec<Change>,
    pub visual: Visual,
    pub layer: Option<String>,
    /// Changed pixels inside its rect (physical), when compared.
    pub pixels: Option<u64>,
    /// Where they are, logical px `[x, y, w, h]`.
    pub region: Option<[f32; 4]>,
    /// Its bound rect before and after, when that moved.
    pub moved: Option<[Option<[f32; 4]>; 2]>,
}

impl ControlReport {
    pub fn changed(&self) -> bool {
        !self.semantics.is_empty() || !matches!(self.visual, Visual::Unchanged | Visual::Unbound)
    }
}

#[derive(Debug, Serialize)]
pub struct LayerReport {
    pub name: String,
    pub status: Status,
    pub hash: Option<String>,
    /// The baseline's hash, when it differs.
    pub was: Option<String>,
    /// Changed pixels inside the layer's region, when compared.
    pub pixels: Option<u64>,
    pub region: Option<[f32; 4]>,
    /// The part of those no bound control covers (the chrome, text
    /// drawn outside any control).
    pub unbound_pixels: Option<u64>,
    pub unbound_region: Option<[f32; 4]>,
}

#[derive(Debug, Serialize)]
pub struct Timings {
    pub stage: f64,
    pub capture: f64,
    pub compare: f64,
    pub render: Option<f64>,
    pub decode: Option<f64>,
    pub diff: Option<f64>,
}

/// Milliseconds to the microsecond.
fn ms(d: std::time::Duration) -> f64 {
    (d.as_secs_f64() * 1_000_000.0).round() / 1000.0
}

/// The report of `o`, checked against a baseline whose scene is `base`.
pub fn build(o: &Outcome, base: &Scene) -> Report {
    let diffed = match &o.pixels {
        Pixels::Diffed(c) => Some(c),
        _ => None,
    };
    let mut page = Vec::new();
    if let Some(note) = o.delta.semantics.note {
        page.push(note.to_string());
    }
    let mut controls: Vec<ControlReport> = ids(
        o.semantics.as_ref(),
        &o.delta.semantics.changes,
        &o.now,
        base,
    )
    .into_iter()
    .map(|id| control(o, base, &id))
    .collect();
    for change in &o.delta.semantics.changes {
        let (node, what) = describe(change);
        match node.and_then(|n| controls.iter_mut().find(|c| c.id == n)) {
            Some(c) => {
                c.semantics.push(what);
                c.delta.push(lean(change));
            }
            None => page.push(what),
        }
    }
    let layers = o
        .delta
        .layers
        .iter()
        .map(|l| {
            let hits = diffed.zip(layer_region(l)).map(|(c, r)| c.within(r));
            let holes: Vec<[f32; 4]> = drawn(&o.now.bindings, &base.bindings)
                .into_iter()
                .filter(|(b, _)| b.layer == l.name)
                .map(|(_, r)| r)
                .collect();
            let rest = diffed
                .zip(layer_region(l))
                .map(|(c, r)| c.within_except(r, &holes));
            let hash =
                |r: &Option<scene::LayerRecord>| r.as_ref().map(|l| format!("{:016x}", l.hash));
            LayerReport {
                name: l.name.clone(),
                status: l.status,
                hash: hash(&l.after).or(hash(&l.before)),
                was: (l.status == Status::Changed)
                    .then(|| hash(&l.before))
                    .flatten(),
                pixels: hits.map(|h| h.pixels),
                region: hits.and_then(|h| h.region),
                unbound_pixels: rest.map(|h| h.pixels),
                unbound_region: rest.and_then(|h| h.region),
            }
        })
        .collect();
    let total = diffed.map(|c| c.total());
    Report {
        baseline: o.label.clone(),
        changed: !o.delta.unchanged(),
        page,
        pixels: PixelReport {
            status: match &o.pixels {
                Pixels::NotNeeded => "not_needed",
                Pixels::NoBaselineImage => "no_baseline_image",
                Pixels::NoAdapter => "no_adapter",
                Pixels::Diffed(_) => "diffed",
            },
            changed: total.map(|h| h.pixels),
            compared: diffed.map(|c| c.compared),
            region: total.and_then(|h| h.region),
        },
        controls,
        layers,
        timings_ms: Timings {
            stage: ms(o.staged),
            capture: ms(o.captured),
            compare: ms(o.compared),
            render: o.rendered.map(ms),
            decode: o.decoded.map(ms),
            diff: o.diffed.map(ms),
        },
    }
}

/// Every id the report speaks of: the page's projection in order, the
/// nodes only the baseline had, then ids bound but not projected.
fn ids(now: Option<&Snapshot>, changes: &[Change], scene: &Scene, base: &Scene) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut add = |id: &str| {
        if !out.iter().any(|o| o == id) {
            out.push(id.to_string());
        }
    };
    if let Some(s) = now {
        for id in s.nodes().keys() {
            add(id.as_str());
        }
    }
    for change in changes {
        if let Change::NodeRemoved(id) = change {
            add(id.as_str());
        }
    }
    for b in scene.bindings.iter().chain(&base.bindings) {
        add(&b.id);
    }
    out
}

fn control(o: &Outcome, base: &Scene, id: &str) -> ControlReport {
    let now = o.now.bindings.iter().find(|b| b.id == id);
    let was = base.bindings.iter().find(|b| b.id == id);
    let mut c = ControlReport {
        id: id.to_string(),
        semantics: Vec::new(),
        delta: Vec::new(),
        visual: Visual::Unbound,
        layer: now.or(was).map(|b| b.layer.clone()),
        pixels: None,
        region: None,
        moved: (now != was).then(|| [was.map(|b| b.rect), now.map(|b| b.rect)]),
    };
    let Some(bound) = now.or(was) else {
        return c;
    };
    let layer_moved = o
        .delta
        .layers
        .iter()
        .any(|l| l.name == bound.layer && l.status != Status::Unchanged);
    let hits: Option<Hits> = match &o.pixels {
        Pixels::Diffed(changed) => Some(
            drawn(std::slice::from_ref(bound), &[])
                .first()
                .map_or(Hits::default(), |(_, r)| changed.within(*r)),
        ),
        _ => None,
    };
    c.pixels = hits.map(|h| h.pixels);
    c.region = hits.and_then(|h| h.region);
    c.visual = match hits {
        Some(h) if h.pixels > 0 => Visual::Changed,
        _ if c.moved.is_some() => Visual::Changed,
        None if layer_moved => Visual::LayerChanged,
        _ => Visual::Unchanged,
    };
    c
}

/// A change as the report keeps it: a node added without its `source`,
/// as the agent port sends it.
fn lean(change: &Change) -> Change {
    match change {
        Change::NodeAdded(node) => {
            let mut node = node.clone();
            node.source = None;
            Change::NodeAdded(node)
        }
        other => other.clone(),
    }
}

/// The report as text: the baseline and its timings, then [`body`].
pub fn text(r: &Report) -> String {
    let t = &r.timings_ms;
    let mut times = format!(
        "stage {:.1} ms, hash {:.1} ms, compare {:.1} ms",
        t.stage, t.capture, t.compare
    );
    for (what, v) in [("render", t.render), ("decode", t.decode), ("diff", t.diff)] {
        if let Some(v) = v {
            times.push_str(&format!(", {what} {v:.1} ms"));
        }
    }
    let head = if r.changed { "CHANGED " } else { "ok      " };
    format!("{head} {} ({times})\n{}", r.baseline, body(r))
}

/// Each control that changed, the layers that moved and what their
/// pixels say; empty when nothing moved.
pub fn body(r: &Report) -> String {
    let mut out = String::new();
    if !r.changed {
        return out;
    }
    for line in &r.page {
        out.push_str(&format!("  (page): {line}\n"));
    }
    let listed = r.controls.iter().filter(|c| c.changed());
    let width = listed.clone().map(|c| c.id.len()).max().unwrap_or(0);
    for c in listed {
        let meaning = if c.semantics.is_empty() {
            "semantics unchanged".to_string()
        } else {
            format!("semantics changed: {}", c.semantics.join("; "))
        };
        out.push_str(&format!("  {:width$}  {meaning}; {}\n", c.id, visual(c)));
    }
    let quiet = r.controls.iter().filter(|c| !c.changed()).count();
    if quiet > 0 {
        out.push_str(&format!("  {quiet} other controls unchanged\n"));
    }
    for l in r.layers.iter().filter(|l| l.status != Status::Unchanged) {
        let status = match l.status {
            Status::Changed => "changed",
            Status::Added => "added",
            Status::Removed => "removed",
            Status::Unchanged => "unchanged",
        };
        let hash = match (&l.was, &l.hash) {
            (Some(was), Some(now)) => format!(" ({was} -> {now})"),
            _ => String::new(),
        };
        let mut line = format!("  layer {}: {status}{hash}", l.name);
        if let Some(px) = l.pixels {
            line.push_str(&format!(", {px} px{}", at(l.region)));
        }
        if let Some(px) = l.unbound_pixels.filter(|px| *px > 0) {
            line.push_str(&format!(
                "; {px} px outside any bound control{}",
                at(l.unbound_region)
            ));
        }
        out.push_str(&line);
        out.push('\n');
    }
    match r.pixels.status {
        "diffed" => out.push_str(&format!(
            "  pixels: {} changed of {} compared{}\n",
            r.pixels.changed.unwrap_or(0),
            r.pixels.compared.unwrap_or(0),
            at(r.pixels.region)
        )),
        "no_baseline_image" => out.push_str(
            "  pixels: visual changed, no baseline image to diff (bless --png keeps one)\n",
        ),
        "no_adapter" => out.push_str(
            "  pixels: visual changed, no GPU adapter here to draw the frame; not compared\n",
        ),
        _ => {}
    }
    out
}

fn visual(c: &ControlReport) -> String {
    let layer = c
        .layer
        .as_deref()
        .map_or(String::new(), |l| format!(", layer {l}"));
    let moved = match c.moved {
        Some([None, Some(now)]) => format!(", newly bound at {}", region(now)),
        Some([Some(was), None]) => format!(", no longer bound (was {})", region(was)),
        Some([Some(was), Some(now)]) => format!(", bound {} -> {}", region(was), region(now)),
        _ => String::new(),
    };
    match c.visual {
        Visual::Unchanged => format!("visual unchanged{layer}"),
        Visual::Unbound => "visual not bound".to_string(),
        Visual::LayerChanged => {
            format!("visual changed{layer} (the layer moved; its pixels were not compared){moved}")
        }
        Visual::Changed => match c.pixels {
            Some(px) if px > 0 => format!("visual changed{layer}, {px} px{}{moved}", at(c.region)),
            _ => format!("visual changed{layer}{moved}"),
        },
    }
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

/// Every bound control of `now`, then the ones only `base` bound, each
/// with the part of its rect its clip lets through.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare;
    use crate::pixels;
    use crate::scene::LayerRecord;
    use engine::headless::Frame;
    use sem::{Role, SemId, SnapshotBuilder, StateRef};
    use std::time::Duration;

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

    fn bind(id: &str, y: f32) -> BindRecord {
        BindRecord {
            id: id.to_string(),
            layer: "todo".to_string(),
            rect: [10.0, y, 30.0, 10.0],
            clip: None,
        }
    }

    fn layer(hash: u64) -> LayerRecord {
        LayerRecord {
            name: "todo".to_string(),
            z: 0,
            hash,
            nodes: 4,
            bounds: Some([0.0, 0.0, 100.0, 100.0]),
        }
    }

    fn frame() -> Frame {
        Frame {
            width: 100,
            height: 100,
            rgba: vec![40; 100 * 100 * 4],
        }
    }

    /// A small todo: `a` drew otherwise (pixels), `b` was relabelled
    /// (meaning), `c` is new (meaning and binding), `count` is projected
    /// and never bound; pixels also changed where no control is bound.
    fn outcome() -> (Outcome, Scene) {
        let base = Scene {
            layers: vec![layer(1)],
            bindings: vec![bind("todo.a", 10.0), bind("todo.b", 30.0)],
        };
        let now = Scene {
            layers: vec![layer(2)],
            bindings: vec![
                bind("todo.a", 10.0),
                bind("todo.b", 30.0),
                bind("todo.c", 50.0),
            ],
        };
        let project = |nodes: &[(&str, &str)]| {
            let mut b = SnapshotBuilder::new();
            for (n, label) in nodes {
                b.node(id(n), Role::Button, *label);
            }
            match b.finish() {
                Ok(s) => s,
                Err(e) => panic!("{e:?}"),
            }
        };
        let before = project(&[("todo.a", "A"), ("todo.b", "B"), ("todo.count", "2")]);
        let after = project(&[
            ("todo.a", "A"),
            ("todo.b", "B2"),
            ("todo.c", "C"),
            ("todo.count", "2"),
        ]);
        let revision = before.revision().to_string();
        let delta = match compare::delta(
            &base,
            Some(&revision),
            || Ok(Some(before)),
            &now,
            Some(&after),
        ) {
            Ok(d) => d,
            Err(e) => panic!("{e}"),
        };
        let mut drawn = frame();
        for (x0, y0, x1, y1) in [(15, 12, 20, 16), (60, 80, 70, 90)] {
            for y in y0..y1 {
                for x in x0..x1 {
                    drawn.rgba[(y * 100 + x) * 4] = 250;
                }
            }
        }
        let changed = match pixels::diff(&frame(), &drawn, &[[0.0, 0.0, 100.0, 100.0]], 1.0) {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };
        let o = Outcome {
            label: "todo@core@100x100".to_string(),
            delta,
            semantics: Some(after),
            now,
            pixels: Pixels::Diffed(changed),
            staged: Duration::from_millis(4),
            captured: Duration::from_micros(200),
            compared: Duration::ZERO,
            rendered: Some(Duration::from_millis(20)),
            decoded: Some(Duration::from_millis(3)),
            diffed: Some(Duration::from_millis(1)),
        };
        (o, base)
    }

    #[test]
    fn a_report_goes_by_control_meaning_first_then_drawing() {
        let (o, base) = outcome();
        let r = build(&o, &base);
        assert!(r.changed);
        let by = |id: &str| match r.controls.iter().find(|c| c.id == id) {
            Some(c) => c,
            None => panic!("no control {id}"),
        };
        let a = by("todo.a");
        assert!(a.semantics.is_empty());
        assert_eq!((a.visual, a.pixels), (Visual::Changed, Some(20)));
        assert_eq!(a.region, Some([15.0, 12.0, 5.0, 4.0]));
        let b = by("todo.b");
        assert_eq!(b.semantics, ["label \"B\" -> \"B2\""]);
        assert_eq!((b.visual, b.pixels), (Visual::Unchanged, Some(0)));
        let c = by("todo.c");
        assert_eq!(c.semantics, ["added (button \"C\")"]);
        assert_eq!(c.visual, Visual::Changed);
        assert_eq!(c.moved, Some([None, Some([10.0, 50.0, 30.0, 10.0])]));
        let count = by("todo.count");
        assert!(!count.changed() && count.visual == Visual::Unbound);
        // the layer: every changed pixel, and the block no control covers
        let todo = &r.layers[0];
        assert_eq!((todo.status, todo.pixels), (Status::Changed, Some(120)));
        assert_eq!(todo.unbound_pixels, Some(100));
        assert_eq!(todo.unbound_region, Some([60.0, 80.0, 10.0, 10.0]));
        assert_eq!(todo.was.as_deref(), Some("0000000000000001"));
    }

    #[test]
    fn the_text_says_meaning_and_drawing_per_control_and_the_json_the_same() {
        let (o, base) = outcome();
        let r = build(&o, &base);
        let text = text(&r);
        for needle in [
            "CHANGED  todo@core@100x100 (stage 4.0 ms, hash 0.2 ms, compare 0.0 ms, render 20.0 ms",
            "todo.a  semantics unchanged; visual changed, layer todo, 20 px at 15,12 5x4",
            "todo.b  semantics changed: label \"B\" -> \"B2\"; visual unchanged, layer todo",
            "todo.c  semantics changed: added (button \"C\"); visual changed, layer todo, \
             newly bound at 10,50 30x10",
            "1 other controls unchanged",
            "layer todo: changed (0000000000000001 -> 0000000000000002), 120 px at 15,12 55x78; \
             100 px outside any bound control at 60,80 10x10",
            "pixels: 120 changed of 10000 compared at 15,12 55x78",
        ] {
            assert!(text.contains(needle), "{needle}\n---\n{text}");
        }
        let json = match serde_json::to_value(&r) {
            Ok(v) => v,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(json["controls"][0]["id"], "todo.a");
        assert_eq!(json["controls"][0]["visual"], "changed");
        assert_eq!(
            json["controls"][1]["delta"][0]["label_changed"]["after"],
            "B2"
        );
        assert_eq!(json["pixels"]["status"], "diffed");
        assert_eq!(json["layers"][0]["status"], "changed");
    }

    #[test]
    fn a_baseline_that_held_is_one_line() {
        let (mut o, base) = outcome();
        o.delta = compare::Delta::default();
        o.now = base.clone();
        o.pixels = Pixels::NotNeeded;
        (o.rendered, o.decoded, o.diffed) = (None, None, None);
        let r = build(&o, &base);
        assert!(!r.changed && r.controls.iter().all(|c| !c.changed()));
        assert_eq!(
            text(&r),
            "ok       todo@core@100x100 (stage 4.0 ms, hash 0.2 ms, compare 0.0 ms)\n"
        );
    }
}
