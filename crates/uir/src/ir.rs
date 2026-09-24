//! the IR itself: a `Surface` of `IrNode`s, each one an identity, a
//! component of the vocabulary, props by token, a layout, and bindings to
//! the state and the actions the app declares.
//!
//! the JSON form is the one an LLM writes: every field but `sem` optional,
//! nothing written at its default, and no field the types do not know, so
//! a literal color or a pixel has nowhere to go. there is one reader, the
//! pointer decoder (`decode`), and `Ir`'s `Deserialize` goes through it:
//! the node types derive only `Serialize`, because serde's derived reader
//! would also take a struct written as a JSON array, a second form the
//! schema does not have. reading checks the shape only; `verify` decides
//! whether an IR may exist.

use std::collections::BTreeMap;

use sem::{ActionName, SemError, SemId, StateRef};
use serde::Serialize;

use crate::token::{Layout, Props, Size};
use crate::vocab::Component;

/// the version this crate reads and writes, the value of `plev_ir`.
pub const VERSION: u32 = 1;

/// the key a template id carries where the item goes:
/// `changes.file[{item}]`.
pub const ITEM: &str = "{item}";

/// a surface described by meaning. its `Deserialize` is `decode`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Ir {
    /// always `VERSION`; another value is refused before anything else is
    /// read.
    pub plev_ir: u32,
    pub surface: Surface,
}

impl Ir {
    /// an IR of this version.
    pub fn new(surface: Surface) -> Ir {
        Ir {
            plev_ir: VERSION,
            surface,
        }
    }
}

/// one surface: its identity, the title that names it, and its tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Surface {
    pub id: SemId,
    pub title: String,
    pub root: IrNode,
}

/// one node of a surface.
///
/// `sem` is its identity, chosen by whoever wrote the IR and never by a
/// position. without a `component` the node is a layout container. a
/// node takes children or a `repeat`, never both, and only when its
/// component is a container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IrNode {
    pub sem: SemId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Props::is_empty")]
    pub props: Props,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    #[serde(skip_serializing_if = "Size::is_empty")]
    pub size: Size,
    #[serde(skip_serializing_if = "Bind::is_empty")]
    pub bind: Bind,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<IrNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<Box<Repeat>>,
}

impl IrNode {
    /// a node with only its identity: a container with no layout, no
    /// label and nothing bound.
    pub fn new(sem: SemId) -> IrNode {
        IrNode {
            sem,
            component: None,
            label: None,
            description: None,
            props: Props::default(),
            layout: None,
            size: Size::default(),
            bind: Bind::default(),
            children: Vec::new(),
            repeat: None,
        }
    }
}

/// what a node shows from the app and what it asks the app to do.
///
/// `state` maps a state the component shows (`text`, `checked`) to the
/// app state it comes from; `actions` maps an event the component sends
/// (`activate`, `set_value`) to the capability that runs. the names come
/// from the component's `Spec`. the IR only references: nothing here can
/// create a capability.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Bind {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub state: BTreeMap<String, StateRef>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub actions: BTreeMap<String, CapabilityRef>,
}

impl Bind {
    pub fn is_empty(&self) -> bool {
        self.state.is_empty() && self.actions.is_empty()
    }
}

/// an action of the app: the node that declares it and its name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct CapabilityRef {
    pub target: SemId,
    pub action: ActionName,
}

impl CapabilityRef {
    pub fn parse(target: &str, action: &str) -> Result<CapabilityRef, SemError> {
        Ok(CapabilityRef {
            target: SemId::parse(target)?,
            action: ActionName::parse(action)?,
        })
    }
}

/// a node repeated once per item of a `List` or `Set` of the app state.
///
/// every id inside `item` carries the key `[{item}]`, and each instance
/// replaces it with its item: over `changes:files`, the template
/// `changes.file[{item}]` becomes `changes.file[src/lib.rs]`. bindings
/// inside the template may carry it too, and then name one capability per
/// item. an instance is named by its item, so the template root has no
/// label.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Repeat {
    pub each: StateRef,
    pub item: IrNode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Axis, SpacingToken};

    fn id(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    #[test]
    fn a_node_writes_only_what_it_says() {
        let mut submit = IrNode::new(id("commit.submit"));
        submit.component = Some(Component::Button);
        submit.label = Some("commit".to_owned());
        submit.bind.actions.insert(
            "activate".to_owned(),
            CapabilityRef::parse("commit.submit", "submit").unwrap(),
        );
        let mut form = IrNode::new(id("commit.form"));
        form.layout = Some(Layout::Stack {
            axis: Axis::Vertical,
            gap: Some(SpacingToken::Md),
            align: None,
        });
        form.children.push(submit);
        let ir = Ir::new(Surface {
            id: id("commit"),
            title: "commit".to_owned(),
            root: form,
        });
        let json = serde_json::to_value(&ir).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "plev_ir": 1,
                "surface": {
                    "id": "commit",
                    "title": "commit",
                    "root": {
                        "sem": "commit.form",
                        "layout": { "stack": { "axis": "vertical", "gap": "md" } },
                        "children": [{
                            "sem": "commit.submit",
                            "component": "button",
                            "label": "commit",
                            "bind": { "actions": {
                                "activate": { "target": "commit.submit", "action": "submit" }
                            } }
                        }]
                    }
                }
            })
        );
        assert_eq!(serde_json::from_value::<Ir>(json).unwrap(), ir);
    }

    /// a document whose root is `node`.
    fn doc(node: &str) -> String {
        format!(r#"{{"plev_ir": 1, "surface": {{"id": "s", "title": "t", "root": {node}}}}}"#)
    }

    #[test]
    fn a_repeat_round_trips_with_its_template() {
        let text = doc(r#"{
            "sem": "review.files",
            "component": "list",
            "repeat": {
                "each": "changes:files",
                "item": {
                    "sem": "review.file[{item}]",
                    "bind": { "state": { "text": "changes.file[{item}]:status" } }
                }
            }
        }"#);
        let ir: Ir = serde_json::from_str(&text).unwrap();
        let repeat = ir.surface.root.repeat.as_deref().unwrap();
        assert_eq!(repeat.each.to_string(), "changes:files");
        assert_eq!(repeat.item.sem.as_str(), "review.file[{item}]");
        let back: Ir = serde_json::from_value(serde_json::to_value(&ir).unwrap()).unwrap();
        assert_eq!(back, ir);
    }

    #[test]
    fn an_unknown_field_a_bad_id_or_a_second_form_is_refused() {
        for bad in [
            r#"{"sem": "a", "colour": "red"}"#,
            r#"{"sem": "A"}"#,
            r#"{"sem": "a", "bind": {"state": {"text": "no_colon"}}}"#,
            r#"{"sem": "a", "bind": {"actions": {"activate": {"target": "a"}}}}"#,
            r#"{"sem": "a", "bind": {"actions": {"activate": ["a", "go"]}}}"#,
            r#"{"sem": "a", "bind": []}"#,
            r#"{"sem": "a", "props": []}"#,
            r#"{"sem": "a", "layout": {"split": ["horizontal", 50]}}"#,
            r#"["a"]"#,
            r#"{"component": "button"}"#,
        ] {
            assert!(serde_json::from_str::<Ir>(&doc(bad)).is_err(), "{bad}");
        }
        let text = r#"{"plev_ir": 1, "surface": ["s", "t", {"sem": "a"}]}"#;
        assert!(serde_json::from_str::<Ir>(text).is_err());
    }
}
