//! The first two questions of a check, answered without a pixel: did
//! the page's meaning change (its projection, by `Snapshot::diff`), and
//! did what it draws change (a layer's hash, a binding's rect).

use accord::{Change, Snapshot};
use serde::Serialize;

use crate::scene::{BindRecord, LayerRecord, Scene};

/// How the projection moved from the baseline to the page.
#[derive(Debug, Default)]
pub struct SemDelta {
    /// Every semantic change, in the order `Snapshot::diff` gives them.
    pub changes: Vec<Change>,
    /// Set when one side has no projection at all.
    pub note: Option<&'static str>,
}

/// How one layer, matched by name, moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Unchanged,
    Changed,
    Added,
    Removed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerDelta {
    pub name: String,
    pub status: Status,
    pub before: Option<LayerRecord>,
    pub after: Option<LayerRecord>,
}

/// A control bound in one frame and not the other, or bound elsewhere.
#[derive(Clone, Debug, PartialEq)]
pub struct BindDelta {
    pub id: String,
    pub before: Option<BindRecord>,
    pub after: Option<BindRecord>,
}

/// Everything a check found before any pixel.
#[derive(Debug, Default)]
pub struct Delta {
    pub semantics: SemDelta,
    /// Every layer of both frames: the page's in composite order, then
    /// the ones only the baseline has.
    pub layers: Vec<LayerDelta>,
    /// Only the bindings that differ.
    pub bindings: Vec<BindDelta>,
}

impl Delta {
    /// Nothing moved: the check ends here, without a frame drawn.
    pub fn unchanged(&self) -> bool {
        self.semantics.changes.is_empty()
            && self.semantics.note.is_none()
            && self.bindings.is_empty()
            && !self.drawn_changed()
    }

    /// Some layer's content moved: only this asks for pixels.
    pub fn drawn_changed(&self) -> bool {
        self.layers.iter().any(|l| l.status != Status::Unchanged)
    }
}

/// The projection's delta. Equal revisions are equal snapshots
/// (docs/ADR/semantic-contract-as-pure-projection.md), so `before` is
/// only read when the revisions differ.
pub fn semantics(
    revision: Option<&str>,
    before: impl FnOnce() -> Result<Option<Snapshot>, String>,
    after: Option<&Snapshot>,
) -> Result<SemDelta, String> {
    let now = after.map(|s| s.revision().to_string());
    if revision == now.as_deref() {
        return Ok(SemDelta::default());
    }
    let note = match (revision, after) {
        (None, Some(_)) => Some("the page projects now; the baseline has no projection"),
        (Some(_), None) => Some("the page no longer projects; the baseline has a projection"),
        _ => None,
    };
    let before = before()?.unwrap_or_default();
    let changes = match after {
        Some(after) => Snapshot::diff(&before, after),
        None => Snapshot::diff(&before, &Snapshot::default()),
    };
    Ok(SemDelta { changes, note })
}

/// The layers matched by name: same hash, another hash, only in the
/// page, only in the baseline.
pub fn layers(before: &[LayerRecord], after: &[LayerRecord]) -> Vec<LayerDelta> {
    let mut out: Vec<LayerDelta> = after
        .iter()
        .map(|a| {
            let b = before.iter().find(|b| b.name == a.name);
            let status = match b {
                None => Status::Added,
                Some(b) if b.hash == a.hash => Status::Unchanged,
                Some(_) => Status::Changed,
            };
            LayerDelta {
                name: a.name.clone(),
                status,
                before: b.cloned(),
                after: Some(a.clone()),
            }
        })
        .collect();
    for b in before {
        if !after.iter().any(|a| a.name == b.name) {
            out.push(LayerDelta {
                name: b.name.clone(),
                status: Status::Removed,
                before: Some(b.clone()),
                after: None,
            });
        }
    }
    out
}

/// The bindings that differ: another rect, clip or layer, or bound on
/// one side only. The page's order, then the ones only the baseline has.
pub fn bindings(before: &[BindRecord], after: &[BindRecord]) -> Vec<BindDelta> {
    let mut out = Vec::new();
    for a in after {
        let b = before.iter().find(|b| b.id == a.id);
        if b != Some(a) {
            out.push(BindDelta {
                id: a.id.clone(),
                before: b.cloned(),
                after: Some(a.clone()),
            });
        }
    }
    for b in before {
        if !after.iter().any(|a| a.id == b.id) {
            out.push(BindDelta {
                id: b.id.clone(),
                before: Some(b.clone()),
                after: None,
            });
        }
    }
    out
}

/// The whole delta of a staged page against its baseline.
pub fn delta(
    base: &Scene,
    revision: Option<&str>,
    base_semantics: impl FnOnce() -> Result<Option<Snapshot>, String>,
    now: &Scene,
    now_semantics: Option<&Snapshot>,
) -> Result<Delta, String> {
    Ok(Delta {
        semantics: semantics(revision, base_semantics, now_semantics)?,
        layers: layers(&base.layers, &now.layers),
        bindings: bindings(&base.bindings, &now.bindings),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use accord::{Role, SemId, SnapshotBuilder};

    fn layer(name: &str, hash: u64) -> LayerRecord {
        LayerRecord {
            name: name.to_string(),
            z: 0,
            hash,
            nodes: 1,
            bounds: Some([0.0, 0.0, 10.0, 10.0]),
        }
    }

    fn bind(id: &str, x: f32) -> BindRecord {
        BindRecord {
            id: id.to_string(),
            layer: "todo".to_string(),
            rect: [x, 0.0, 10.0, 10.0],
            clip: None,
        }
    }

    fn snapshot(labels: &[(&str, &str)]) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        for (id, label) in labels {
            let id = match SemId::parse(id) {
                Ok(id) => id,
                Err(e) => panic!("{e}"),
            };
            b.node(&id, Role::Button, *label);
        }
        match b.finish() {
            Ok(s) => s,
            Err(e) => panic!("{e:?}"),
        }
    }

    fn scene(layers: Vec<LayerRecord>, bindings: Vec<BindRecord>) -> Scene {
        Scene { layers, bindings }
    }

    fn check(base: (&Scene, Option<&Snapshot>), now: (&Scene, Option<&Snapshot>)) -> (Delta, bool) {
        let revision = base.1.map(|s| s.revision().to_string());
        let mut read = false;
        let delta = delta(
            base.0,
            revision.as_deref(),
            || {
                read = true;
                Ok(base.1.cloned())
            },
            now.0,
            now.1,
        );
        match delta {
            Ok(d) => (d, read),
            Err(e) => panic!("{e}"),
        }
    }

    /// The four cases of a small page: nothing, only the drawing, only
    /// the meaning, both.
    #[test]
    fn equal_scenes_and_projections_are_unchanged_without_reading_the_baseline() {
        let s = scene(
            vec![layer("todo", 1), layer("z1000", 2)],
            vec![bind("todo.a", 0.0)],
        );
        let p = snapshot(&[("todo.a", "Buy milk")]);
        let (d, read) = check((&s, Some(&p)), (&s.clone(), Some(&p.clone())));
        assert!(d.unchanged() && !d.drawn_changed());
        assert!(
            !read,
            "the baseline projection was read with equal revisions"
        );
    }

    #[test]
    fn only_a_hash_moved_is_a_drawing_change_with_the_meaning_intact() {
        let base = scene(vec![layer("todo", 1), layer("z1000", 2)], vec![]);
        let now = scene(vec![layer("todo", 9), layer("z1000", 2)], vec![]);
        let p = snapshot(&[("todo.a", "Buy milk")]);
        let (d, read) = check((&base, Some(&p)), (&now, Some(&p)));
        assert!(!d.unchanged() && d.drawn_changed());
        assert!(d.semantics.changes.is_empty() && !read);
        let changed: Vec<&str> = d
            .layers
            .iter()
            .filter(|l| l.status != Status::Unchanged)
            .map(|l| l.name.as_str())
            .collect();
        assert_eq!(changed, ["todo"]);
    }

    #[test]
    fn only_a_label_moved_is_a_meaning_change_with_the_drawing_intact() {
        let s = scene(vec![layer("todo", 1)], vec![bind("todo.a", 0.0)]);
        let before = snapshot(&[("todo.a", "Buy milk")]);
        let after = snapshot(&[("todo.a", "Buy oat milk")]);
        let (d, read) = check((&s, Some(&before)), (&s, Some(&after)));
        assert!(!d.unchanged() && !d.drawn_changed() && read);
        assert!(matches!(
            d.semantics.changes.as_slice(),
            [Change::LabelChanged { node, before, after }]
                if node.as_str() == "todo.a" && before == "Buy milk" && after == "Buy oat milk"
        ));
    }

    #[test]
    fn both_moved_and_layers_and_bindings_match_by_name() {
        let base = scene(
            vec![layer("todo", 1), layer("z10", 3)],
            vec![bind("todo.a", 0.0), bind("todo.b", 0.0)],
        );
        let now = scene(
            vec![layer("todo", 2), layer("z20", 4)],
            vec![
                bind("todo.a", 0.0),
                bind("todo.b", 5.0),
                bind("todo.c", 0.0),
            ],
        );
        let before = snapshot(&[("todo.a", "a")]);
        let after = snapshot(&[("todo.a", "a"), ("todo.c", "c")]);
        let (d, _) = check((&base, Some(&before)), (&now, Some(&after)));
        let layers: Vec<(&str, Status)> = d
            .layers
            .iter()
            .map(|l| (l.name.as_str(), l.status))
            .collect();
        assert_eq!(
            layers,
            [
                ("todo", Status::Changed),
                ("z20", Status::Added),
                ("z10", Status::Removed)
            ]
        );
        let moved: Vec<(&str, bool, bool)> = d
            .bindings
            .iter()
            .map(|b| (b.id.as_str(), b.before.is_some(), b.after.is_some()))
            .collect();
        assert_eq!(moved, [("todo.b", true, true), ("todo.c", false, true)]);
        assert!(
            matches!(d.semantics.changes.as_slice(), [Change::NodeAdded(n)] if n.id.as_str() == "todo.c")
        );
    }

    #[test]
    fn a_projection_on_one_side_only_is_noted_and_diffed_against_nothing() {
        let s = scene(vec![layer("z0", 1)], vec![]);
        let p = snapshot(&[("todo.a", "a"), ("todo.b", "b")]);
        let (d, _) = check((&s, None), (&s, Some(&p)));
        assert!(d.semantics.note.is_some_and(|n| n.contains("projects now")));
        assert_eq!(d.semantics.changes.len(), 2);
        let (d, _) = check((&s, Some(&p)), (&s, None));
        assert!(d.semantics.note.is_some_and(|n| n.contains("no longer")));
        assert!(
            d.semantics
                .changes
                .iter()
                .all(|c| matches!(c, Change::NodeRemoved(_)))
        );
    }
}
