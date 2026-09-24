//! reading an IR from JSON with every shape error at once, each at its
//! JSON pointer.
//!
//! `serde_json::from_str::<Ir>` reads the same documents but stops at the
//! first error and names a line, not a place in the tree. this walks the
//! tree by hand and hands each leaf (an id, a component, a prop, a
//! binding) to the serde form of its type, so the two readers accept the
//! same documents and build the same `Ir`; the tests hold them to that.

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::error::{IrError, Ptr, Rule};
use crate::ir::{Bind, CapabilityRef, Ir, IrNode, Repeat, Surface, VERSION};
use crate::policy::HARD_DEPTH;
use crate::token::{Props, Size};

const TOP: &[&str] = &["plev_ir", "surface"];
const SURFACE: &[&str] = &["id", "title", "root"];
const NODE: &[&str] = &[
    "sem",
    "component",
    "label",
    "description",
    "props",
    "layout",
    "size",
    "bind",
    "children",
    "repeat",
];
const BIND: &[&str] = &["state", "actions"];
const REPEAT: &[&str] = &["each", "item"];

/// reads an IR from JSON text: a text that is not JSON is one error at the
/// document, anything else is every shape error at once.
pub fn parse(text: &str) -> Result<Ir, Vec<IrError>> {
    let value: Value = serde_json::from_str(text).map_err(|e| {
        vec![IrError::new(
            Rule::Schema,
            &Ptr::root(),
            format!("not JSON: {e}"),
        )]
    })?;
    decode(&value)
}

/// reads an IR from a JSON value, every shape error at once. a version
/// other than `VERSION` is the only error then: the rest of the document
/// is another language.
pub fn decode(value: &Value) -> Result<Ir, Vec<IrError>> {
    let mut d = Decoder { errors: Vec::new() };
    let ir = d.ir(value);
    match ir {
        Some(ir) if d.errors.is_empty() => Ok(ir),
        _ => Err(d.errors),
    }
}

struct Decoder {
    errors: Vec<IrError>,
}

impl Decoder {
    fn fail(&mut self, rule: Rule, at: &Ptr, reason: impl Into<String>) {
        self.errors.push(IrError::new(rule, at, reason));
    }

    fn ir(&mut self, v: &Value) -> Option<Ir> {
        let at = Ptr::root();
        let map = self.object(v, &at, "an IR", TOP)?;
        let version = self.required(map, &at, "plev_ir")?;
        if version.as_u64() != Some(u64::from(VERSION)) {
            self.fail(
                Rule::Schema,
                &at.key("plev_ir"),
                format!("version {version}; this runtime reads version {VERSION}"),
            );
            return None;
        }
        let surface = self
            .required(map, &at, "surface")
            .and_then(|s| self.surface(s, &at.key("surface")));
        Some(Ir::new(surface?))
    }

    fn surface(&mut self, v: &Value, at: &Ptr) -> Option<Surface> {
        let map = self.object(v, at, "a surface", SURFACE)?;
        let id = self.field(map, at, "id", Rule::Identity);
        let title = self.field(map, at, "title", Rule::Schema);
        let root = self
            .required(map, at, "root")
            .and_then(|r| self.node(r, &at.key("root"), 1));
        Some(Surface {
            id: id?,
            title: title?,
            root: root?,
        })
    }

    fn node(&mut self, v: &Value, at: &Ptr, depth: usize) -> Option<IrNode> {
        if depth > HARD_DEPTH {
            self.fail(
                Rule::Limits,
                at,
                format!("nested deeper than {HARD_DEPTH}; nothing below is read"),
            );
            return None;
        }
        let map = self.object(v, at, "a node", NODE)?;
        let sem = self.field(map, at, "sem", Rule::Identity);
        let component = self.optional(map, at, "component", Rule::Vocabulary);
        let label = self.optional(map, at, "label", Rule::Schema);
        let description = self.optional(map, at, "description", Rule::Schema);
        let props = self.fields::<Props>(map, at, "props");
        let layout = self.optional(map, at, "layout", Rule::Vocabulary);
        let size = self.fields::<Size>(map, at, "size");
        let bind = match map.get("bind") {
            Some(b) => self.bind(b, &at.key("bind")),
            None => Some(Bind::default()),
        };
        let children = self.children(map, at, depth);
        let repeat = match map.get("repeat") {
            None | Some(Value::Null) => Some(None),
            Some(r) => self
                .repeat(r, &at.key("repeat"), depth)
                .map(|r| Some(Box::new(r))),
        };
        Some(IrNode {
            sem: sem?,
            component: component?,
            label: label?,
            description: description?,
            props: props?,
            layout: layout?,
            size: size?,
            bind: bind?,
            children: children?,
            repeat: repeat?,
        })
    }

    fn children(
        &mut self,
        map: &Map<String, Value>,
        at: &Ptr,
        depth: usize,
    ) -> Option<Vec<IrNode>> {
        let Some(v) = map.get("children") else {
            return Some(Vec::new());
        };
        let at = at.key("children");
        let Some(list) = v.as_array() else {
            self.fail(Rule::Schema, &at, "children are a list of nodes");
            return None;
        };
        let mut out = Some(Vec::with_capacity(list.len()));
        for (i, child) in list.iter().enumerate() {
            let node = self.node(child, &at.index(i), depth + 1);
            out = out.zip(node).map(|(mut all, node)| {
                all.push(node);
                all
            });
        }
        out
    }

    fn repeat(&mut self, v: &Value, at: &Ptr, depth: usize) -> Option<Repeat> {
        let map = self.object(v, at, "a repeat", REPEAT)?;
        let each = self.field(map, at, "each", Rule::State);
        let item = self
            .required(map, at, "item")
            .and_then(|i| self.node(i, &at.key("item"), depth + 1));
        Some(Repeat {
            each: each?,
            item: item?,
        })
    }

    fn bind(&mut self, v: &Value, at: &Ptr) -> Option<Bind> {
        let map = self.object(v, at, "a binding", BIND)?;
        let state = self.entries::<sem::StateRef>(map, at, "state", Rule::State);
        let actions = self.entries::<CapabilityRef>(map, at, "actions", Rule::Action);
        Some(Bind {
            state: state?,
            actions: actions?,
        })
    }

    /// a map of names to values of `T`, each entry read on its own.
    fn entries<T: DeserializeOwned>(
        &mut self,
        map: &Map<String, Value>,
        at: &Ptr,
        key: &str,
        rule: Rule,
    ) -> Option<std::collections::BTreeMap<String, T>> {
        let Some(v) = map.get(key) else {
            return Some(Default::default());
        };
        let at = at.key(key);
        let Some(entries) = v.as_object() else {
            self.fail(Rule::Schema, &at, format!("{key} is an object of names"));
            return None;
        };
        let mut out = Some(std::collections::BTreeMap::new());
        for (name, value) in sorted(entries) {
            let one = self.typed::<T>(value, &at.key(name), rule);
            out = out.zip(one).map(|(mut all, one)| {
                all.insert(name.clone(), one);
                all
            });
        }
        out
    }

    /// a struct whose fields are all optional, each field read on its own
    /// so an error points at the field.
    fn fields<T: DeserializeOwned + Default>(
        &mut self,
        map: &Map<String, Value>,
        at: &Ptr,
        key: &str,
    ) -> Option<T> {
        let Some(v) = map.get(key) else {
            return Some(T::default());
        };
        let at = at.key(key);
        let Some(fields) = v.as_object() else {
            self.fail(
                Rule::Vocabulary,
                &at,
                format!("{key} is an object of tokens"),
            );
            return None;
        };
        let mut ok = true;
        for (name, value) in sorted(fields) {
            let one = Value::Object(Map::from_iter([(name.clone(), value.clone())]));
            ok &= self
                .typed::<T>(&one, &at.key(name), Rule::Vocabulary)
                .is_some();
        }
        if ok {
            self.typed::<T>(v, &at, Rule::Vocabulary)
        } else {
            None
        }
    }

    fn object<'v>(
        &mut self,
        v: &'v Value,
        at: &Ptr,
        what: &str,
        known: &[&str],
    ) -> Option<&'v Map<String, Value>> {
        let Some(map) = v.as_object() else {
            self.fail(Rule::Schema, at, format!("{what} is an object"));
            return None;
        };
        for (key, _) in sorted(map) {
            if !known.contains(&key.as_str()) {
                let fields = known.join(", ");
                self.fail(
                    Rule::Schema,
                    &at.key(key),
                    format!("unknown field `{key}`; {what} has {fields}"),
                );
            }
        }
        Some(map)
    }

    fn required<'v>(
        &mut self,
        map: &'v Map<String, Value>,
        at: &Ptr,
        key: &str,
    ) -> Option<&'v Value> {
        let v = map.get(key);
        if v.is_none() {
            self.fail(Rule::Schema, at, format!("missing field `{key}`"));
        }
        v
    }

    fn field<T: DeserializeOwned>(
        &mut self,
        map: &Map<String, Value>,
        at: &Ptr,
        key: &str,
        rule: Rule,
    ) -> Option<T> {
        let v = self.required(map, at, key)?;
        self.typed(v, &at.key(key), rule)
    }

    /// an optional field: absent or `null` is `None`, as serde reads it.
    fn optional<T: DeserializeOwned>(
        &mut self,
        map: &Map<String, Value>,
        at: &Ptr,
        key: &str,
        rule: Rule,
    ) -> Option<Option<T>> {
        match map.get(key) {
            None | Some(Value::Null) => Some(None),
            Some(v) => self.typed(v, &at.key(key), rule).map(Some),
        }
    }

    fn typed<T: DeserializeOwned>(&mut self, v: &Value, at: &Ptr, rule: Rule) -> Option<T> {
        match serde_json::from_value(v.clone()) {
            Ok(t) => Some(t),
            Err(e) => {
                self.fail(rule, at, e.to_string());
                None
            }
        }
    }
}

/// the entries of `map` in key order, whatever order the map keeps, so
/// the errors come in the same order on every build.
fn sorted(map: &Map<String, Value>) -> Vec<(&String, &Value)> {
    let mut entries: Vec<(&String, &Value)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(text: &str) -> Vec<(Rule, String)> {
        parse(text)
            .unwrap_err()
            .into_iter()
            .map(|e| (e.rule, e.pointer))
            .collect()
    }

    #[test]
    fn every_shape_error_comes_at_once_with_its_place() {
        let text = r##"{
            "plev_ir": 1,
            "surface": {
                "id": "s",
                "title": "t",
                "extra": true,
                "root": {
                    "sem": "Root",
                    "props": { "padding": 12, "color": "#fff", "shape": "card" },
                    "children": [
                        { "sem": "a", "component": "video" },
                        { "sem": "b", "bind": { "state": { "text": "nope" },
                                                "actions": { "activate": { "target": "b" } } } },
                        { "component": "text" }
                    ]
                }
            }
        }"##;
        assert_eq!(
            errors(text),
            [
                (Rule::Schema, "/surface/extra".to_owned()),
                (Rule::Identity, "/surface/root/sem".to_owned()),
                (Rule::Vocabulary, "/surface/root/props/color".to_owned()),
                (Rule::Vocabulary, "/surface/root/props/padding".to_owned()),
                (
                    Rule::Vocabulary,
                    "/surface/root/children/0/component".to_owned()
                ),
                (
                    Rule::State,
                    "/surface/root/children/1/bind/state/text".to_owned()
                ),
                (
                    Rule::Action,
                    "/surface/root/children/1/bind/actions/activate".to_owned()
                ),
                (Rule::Schema, "/surface/root/children/2".to_owned()),
            ]
        );
    }

    #[test]
    fn another_version_is_the_only_error() {
        let text = r#"{ "plev_ir": 2, "surface": { "whatever": [] } }"#;
        assert_eq!(errors(text), [(Rule::Schema, "/plev_ir".to_owned())]);
        assert_eq!(
            errors(r#"{ "surface": {} }"#),
            [(Rule::Schema, String::new())]
        );
        assert_eq!(errors("{ nope"), [(Rule::Schema, String::new())]);
    }

    #[test]
    fn a_null_optional_field_reads_as_absent_like_serde() {
        let text = r#"{ "plev_ir": 1, "surface": { "id": "s", "title": "t",
            "root": { "sem": "a", "label": null, "component": null } } }"#;
        let ours = parse(text).unwrap();
        let theirs: Ir = serde_json::from_str(text).unwrap();
        assert_eq!(ours, theirs);
        assert_eq!(ours.surface.root.label, None);
    }

    #[test]
    fn a_tree_deeper_than_the_hard_limit_is_cut_not_walked() {
        let mut v = serde_json::json!({ "sem": "leaf" });
        for _ in 0..HARD_DEPTH + 5 {
            v = serde_json::json!({ "sem": "n", "children": [v] });
        }
        let doc =
            serde_json::json!({ "plev_ir": 1, "surface": { "id": "s", "title": "t", "root": v } });
        let errs = decode(&doc).unwrap_err();
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].rule, Rule::Limits);
    }
}
