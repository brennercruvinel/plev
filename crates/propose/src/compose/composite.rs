//! `Composite`: a sequence the person accepted, as one action of the
//! app: its name, its label, and the steps it runs.

use sem::{ActionName, Snapshot};

use super::{Shape, Target, item_of};

/// a sequence of steps the app runs as one action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composite {
    /// the action's name: the steps' actions joined by `_then_`
    /// (`preview_segment_then_apply_cut`).
    pub name: ActionName,
    /// what a person reads: the steps' labels joined by ", then "
    /// ("preview segment, then apply cut").
    pub label: String,
    pub shape: Shape,
}

impl Composite {
    /// the composite of `shape`, named and labelled from what `app`
    /// declares for each step; `taken` are the names already in use, and
    /// a name taken gets a number (`_2`, `_3`).
    pub fn new(shape: Shape, app: &Snapshot, taken: &[ActionName]) -> Result<Composite, String> {
        let joined: Vec<&str> = shape.steps.iter().map(|s| s.action.as_str()).collect();
        let base = joined.join("_then_");
        let mut name = ActionName::parse(&base).map_err(|e| e.to_string())?;
        let mut n = 2;
        while taken.contains(&name) {
            name = ActionName::parse(&format!("{base}_{n}")).map_err(|e| e.to_string())?;
            n += 1;
        }
        let labels: Vec<String> = shape.steps.iter().map(|s| label(s, app)).collect();
        Ok(Composite {
            name,
            label: labels.join(", then "),
            shape,
        })
    }
}

/// a step's label: the one the app gives its action, on the node or on
/// an item of the collection, or the action's name in words.
fn label(step: &super::Step, app: &Snapshot) -> String {
    let declared = app
        .nodes()
        .values()
        .filter(|n| match &step.target {
            Target::Node(id) => n.id == *id,
            Target::Item(base) => item_of(&n.id).is_some_and(|(b, _)| b == *base),
        })
        .flat_map(|n| n.actions.iter())
        .find(|a| a.name == step.action)
        .and_then(|a| a.label.clone());
    declared.unwrap_or_else(|| step.action.as_str().replace('_', " "))
}
