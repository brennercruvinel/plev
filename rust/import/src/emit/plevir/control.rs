//! Semantics from the source: a tag or a handler that says "control"
//! (`<button>`, `onClick`, a text `<input>`) becomes a component with its
//! role, and its event is bound to a capability the importer names on the
//! app's node `<surface>.props`: a `binding required` in the report. The
//! importer never declares a capability; the IR only references one, so
//! with nothing declared the verifier refuses the IR naming what is
//! missing, and with the app's registry it passes.

use plevir::{CapabilityRef, Component, IrNode};

use super::props::{Decision, text};
use super::{Cx, parse_as, sanitize};
use crate::ParserError;
use crate::tree::{AttrValue, ParserNode, Tag, TextValue, snake_case};
use crate::report::{Binding, Need, Verdict};

/// A control the source names.
pub(crate) enum Control {
    Button,
    Field,
    /// An `<input>` of a type phase 1 does not infer.
    Other(String),
}

/// The text kinds of `<input>` a text field stands for.
const TEXT_TYPES: &[&str] = &["text", "email", "search", "url", "tel"];

fn attr<'a>(p: &'a ParserNode, name: &str) -> Option<&'a AttrValue> {
    p.origin
        .attrs
        .iter()
        .find(|a| a.name == name)
        .map(|a| &a.value)
}

/// The attribute `name` as the source writes it (`onClick={onOpen}`,
/// `onclick="open()"`).
fn written(p: &ParserNode, name: &str) -> String {
    p.origin
        .attrs
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.src.what.clone())
        .unwrap_or_else(|| name.to_string())
}

fn attr_text(p: &ParserNode, name: &str) -> Option<String> {
    match attr(p, name)? {
        AttrValue::Text(t) if !t.trim().is_empty() => Some(t.clone()),
        _ => None,
    }
}

/// What control `p` is, when its tag or its handler says so.
pub(crate) fn infer(p: &ParserNode) -> Option<Control> {
    match p.origin.tag.as_str() {
        "button" => Some(Control::Button),
        "textarea" => Some(Control::Field),
        "input" => match attr(p, "type") {
            None => Some(Control::Field),
            Some(AttrValue::Text(t)) if TEXT_TYPES.contains(&t.as_str()) => Some(Control::Field),
            Some(other) => Some(Control::Other(format!("{other:?}"))),
        },
        _ if attr(p, "onClick").is_some() => Some(Control::Button),
        _ => None,
    }
}

/// `onOpen` -> `open`, `handleSubmit` -> `submit`, `reply` -> `reply`.
pub(crate) fn action_name(handler: &str) -> String {
    let rest = ["handle", "on"]
        .iter()
        .find_map(|prefix| {
            handler
                .strip_prefix(prefix)
                .filter(|r| r.starts_with(|c: char| c.is_ascii_uppercase()))
        })
        .unwrap_or(handler);
    sanitize(&snake_case(rest), "activate")
}

/// The literal text a control shows: its own text run, or its text
/// children's.
fn literal_text(p: &ParserNode) -> Option<String> {
    if let Tag::Text(TextValue::Literal(s)) = &p.tag {
        return Some(s.clone());
    }
    let parts: Vec<&str> = p
        .children
        .iter()
        .filter_map(|c| match &c.tag {
            Tag::Text(TextValue::Literal(s)) => Some(s.as_str()),
            _ => None,
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join(" "))
}

/// A text parameter a control shows, when it shows one.
fn param_text(p: &ParserNode) -> Option<String> {
    std::iter::once(p)
        .chain(&p.children)
        .find_map(|c| match &c.tag {
            Tag::Text(TextValue::Param(name)) => Some(name.clone()),
            _ => None,
        })
}

impl Cx<'_> {
    /// Every prop of a control: a comps control draws its own box and its
    /// own text from the theme, so the source's are approximated by it.
    fn own_look(&mut self, p: &ParserNode, id: &str, what: &str) {
        for prop in &p.props {
            let verdict = Verdict::Approximated {
                ir: format!("component {what}"),
                difference: format!(
                    "a {what} draws its own box from the theme; the source's value is not carried"
                ),
            };
            self.decided(
                &Decision {
                    srcs: prop.src.clone(),
                    verdict,
                },
                id,
            );
        }
    }

    /// `p` as the control `kind`, or `None` when phase 1 does not infer it.
    pub(super) fn control(
        &mut self,
        p: &ParserNode,
        id: &str,
        kind: Control,
    ) -> Result<Option<IrNode>, ParserError> {
        match kind {
            Control::Button => self.button(p, id).map(Some),
            Control::Field => self.field(p, id).map(Some),
            Control::Other(ty) => {
                let reason = format!(
                    "an <input> of type {ty} is not inferred in phase 1: only text fields and buttons are"
                );
                self.lost_subtree(p, &reason);
                Ok(None)
            }
        }
    }

    fn button(&mut self, p: &ParserNode, id: &str) -> Result<IrNode, ParserError> {
        let mut node = self.new_node(id)?;
        node.component = Some(Component::Button);
        let named = literal_text(p)
            .or_else(|| attr_text(p, "aria-label"))
            .or_else(|| attr_text(p, "title"));
        let handler = attr(p, "onClick");
        let action = match handler {
            Some(AttrValue::Ident(h)) => action_name(h),
            _ => sanitize(&snake_case(named.as_deref().unwrap_or("")), "activate"),
        };
        let param = param_text(p);
        let label = named
            .clone()
            .or_else(|| param.clone())
            .unwrap_or_else(|| action.clone());
        node.label = Some(label.clone());
        let target = self.props_id();
        node.bind.actions.insert(
            "activate".into(),
            CapabilityRef::parse(&target, &action).map_err(|e| ParserError::Ir(e.to_string()))?,
        );
        let rule = if p.origin.tag == "button" {
            "a <button> is the button component, role button; what it does is the app's: binding required"
        } else {
            "an element with onClick is a control: the button component, role button; binding required"
        };
        self.origin_entries(
            p,
            id,
            Verdict::Transformed {
                ir: "component button".into(),
                rule: rule.into(),
            },
        );
        let handled = Verdict::Transformed {
            ir: format!("bind.actions.activate = `{action}` on `{target}`"),
            rule: "the handler names the action the app binds: binding required".into(),
        };
        let inline = Verdict::Approximated {
            ir: format!("bind.actions.activate = `{action}` on `{target}`"),
            difference: "an inline handler's code is the app's and is not carried; the action is named after the label".into(),
        };
        let by_label = Verdict::Transformed {
            ir: "label".into(),
            rule: "the attribute names the button".into(),
        };
        let on_click = match handler {
            Some(AttrValue::Ident(_)) => handled,
            _ => inline,
        };
        self.attrs(
            p,
            id,
            &[
                ("onClick", on_click),
                ("aria-label", by_label.clone()),
                ("title", by_label),
            ],
        );
        self.own_look(p, id, "button");
        if let Some(name) = &param
            && named.is_none()
        {
            let at = p.origin.src.clone().unwrap_or_default();
            self.ledger.add(&at, Some(id), Verdict::Approximated {
                ir: "label".into(),
                difference: format!("a button's label is fixed text in the IR v1: the parameter `{name}` is shown by its name"),
            });
        }
        self.label_children(p, id);
        let why = match handler {
            Some(AttrValue::Ident(_)) => {
                format!(
                    "`{}`: the app's action behind the handler",
                    written(p, "onClick")
                )
            }
            Some(_) => "an inline onClick: the app's action, named after the label".into(),
            None => "a <button> with no handler: the source does not say what it does".into(),
        };
        self.bindings.push(Binding {
            at: p
                .origin
                .src
                .as_ref()
                .map(|s| s.at.clone())
                .unwrap_or_default(),
            node: id.into(),
            need: Need::Action {
                event: "activate".into(),
                target,
                action,
                param: None,
            },
            why,
        });
        Ok(node)
    }

    /// A control's text children are its label; anything else under it is
    /// lost (a control holds no children in the IR).
    fn label_children(&mut self, p: &ParserNode, id: &str) {
        for child in &p.children {
            match &child.tag {
                Tag::Text(value) => {
                    let verdict = match value {
                        TextValue::Literal(_) => Verdict::Transformed {
                            ir: "label".into(),
                            rule: "the text inside a control is its label".into(),
                        },
                        TextValue::Param(name) => Verdict::Approximated {
                            ir: "label".into(),
                            difference: format!(
                                "a control's label is fixed text in the IR v1: the parameter `{name}` is not bound"
                            ),
                        },
                    };
                    self.origin_entries(child, id, verdict);
                    self.attrs(child, id, &[]);
                    let (_, decisions) = text(&child.props, self.tokens);
                    for d in decisions {
                        for s in &d.srcs {
                            let verdict = Verdict::Approximated {
                                ir: "label".into(),
                                difference: "a control draws its label in its own style from the theme; the source's value is not carried".into(),
                            };
                            self.ledger.add(s, Some(id), verdict);
                        }
                    }
                }
                _ => self.lost_subtree(child, "a control holds no children in the IR"),
            }
        }
    }

    /// An `<img>`: the `image` component named by its `alt` (or, with
    /// none, by its class, approximated), sized as a container is. The IR
    /// carries no picture: `src` is reported as impossible.
    pub(super) fn image(&mut self, p: &ParserNode, id: &str) -> Result<IrNode, ParserError> {
        let mut node = self.new_node(id)?;
        node.component = Some(Component::Image);
        let alt = attr_text(p, "alt");
        let name = sanitize(&snake_case(&p.origin.name), "image");
        node.label = Some(alt.clone().unwrap_or_else(|| name.clone()));
        self.origin_entries(p, id, Verdict::Transformed {
            ir: "component image".into(),
            rule: "an <img> is the image component, role image: its place, size and name, never its pixels".into(),
        });
        if alt.is_none()
            && let Some(at) = p.origin.src.clone()
        {
            self.ledger.add(
                &at,
                Some(id),
                Verdict::Approximated {
                    ir: "label".into(),
                    difference: format!(
                        "an image with no alt text is named `{name}`, after its class"
                    ),
                },
            );
        }
        self.attrs(
            p,
            id,
            &[
                (
                    "alt",
                    Verdict::Transformed {
                        ir: "label".into(),
                        rule: "the alt text names the image".into(),
                    },
                ),
                (
                    "src",
                    Verdict::Impossible {
                        reason: "the IR carries no image data: the picture is the app's".into(),
                    },
                ),
            ],
        );
        let c = super::props::container(&p.props, self.tokens);
        node.props = c.props;
        node.size = c.size;
        for d in &c.decisions {
            self.decided(d, id);
        }
        for child in &p.children {
            self.lost_subtree(child, "an image holds no children in the IR");
        }
        Ok(node)
    }

    fn field(&mut self, p: &ParserNode, id: &str) -> Result<IrNode, ParserError> {
        let mut node = self.new_node(id)?;
        node.component = Some(Component::TextField);
        let base = sanitize(&snake_case(&p.origin.name), "text");
        let label = attr_text(p, "placeholder")
            .or_else(|| attr_text(p, "aria-label"))
            .or_else(|| attr_text(p, "title"))
            .or_else(|| attr_text(p, "name"))
            .unwrap_or_else(|| base.clone());
        node.label = Some(label);
        let handler = attr(p, "onChange");
        let action = match handler {
            Some(AttrValue::Ident(h)) => action_name(h),
            _ => format!("set_{base}"),
        };
        let target = self.props_id();
        let cap =
            CapabilityRef::parse(&target, &action).map_err(|e| ParserError::Ir(e.to_string()))?;
        node.bind.actions.insert("set_value".into(), cap);
        let at = p
            .origin
            .src
            .as_ref()
            .map(|s| s.at.clone())
            .unwrap_or_default();
        let shown = match attr(p, "value") {
            Some(AttrValue::Ident(v)) => Some(sanitize(&snake_case(v), "value")),
            _ => None,
        };
        if let Some(key) = &shown {
            let reference = format!("{target}:{key}");
            node.bind.state.insert("text".into(), parse_as(&reference)?);
            self.bindings.push(Binding {
                at: at.clone(),
                node: id.into(),
                need: Need::State {
                    prop: "text".into(),
                    reference,
                    kind: "text".into(),
                },
                why: "`value` names the text the app holds".into(),
            });
        }
        self.origin_entries(p, id, Verdict::Transformed {
            ir: "component text_field".into(),
            rule: "an <input> is the text_field component, role text input; what it sets is the app's: binding required".into(),
        });
        let set = format!("bind.actions.set_value = `{action}` on `{target}`");
        let on_change = match handler {
            Some(AttrValue::Ident(_)) => Verdict::Transformed {
                ir: set.clone(),
                rule: "the handler names the action the app binds: binding required".into(),
            },
            _ => Verdict::Approximated {
                ir: set.clone(),
                difference: "an inline handler's code is the app's and is not carried; the action is named after the field".into(),
            },
        };
        let label_rule = |what: &str| Verdict::Transformed {
            ir: "label".into(),
            rule: format!("the {what} names the field"),
        };
        let value = Verdict::Transformed {
            ir: "bind.state.text".into(),
            rule: "the value is text the app's state holds: binding required".into(),
        };
        let kind = Verdict::Supported {
            ir: "component text_field".into(),
        };
        self.attrs(
            p,
            id,
            &[
                ("onChange", on_change),
                ("placeholder", label_rule("placeholder")),
                ("aria-label", label_rule("label")),
                ("title", label_rule("title")),
                ("name", label_rule("name")),
                ("value", value),
                ("type", kind),
            ],
        );
        self.own_look(p, id, "text field");
        for child in &p.children {
            self.lost_subtree(child, "a control holds no children in the IR");
        }
        let why = match handler {
            Some(AttrValue::Ident(_)) => {
                format!(
                    "`{}`: the app's action behind the handler",
                    written(p, "onChange")
                )
            }
            Some(_) => "an inline onChange: the app's action, named after the field".into(),
            None => {
                "an <input> with no handler: the source does not say where its text goes".into()
            }
        };
        self.bindings.push(Binding {
            at,
            node: id.into(),
            need: Need::Action {
                event: "set_value".into(),
                target,
                action,
                param: Some("text".into()),
            },
            why,
        });
        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Attr, Src};

    fn element(tag: &str, attrs: &[(&str, AttrValue)]) -> ParserNode {
        let mut p = ParserNode::new(Tag::Div, vec![], vec![]);
        p.origin.tag = tag.into();
        p.origin.attrs = attrs
            .iter()
            .map(|(name, value)| Attr {
                name: (*name).into(),
                value: value.clone(),
                src: Src::new("x.tsx:1", *name),
            })
            .collect();
        p
    }

    #[test]
    fn a_handler_names_the_action() {
        assert_eq!(action_name("onOpen"), "open");
        assert_eq!(action_name("handleSubmit"), "submit");
        assert_eq!(action_name("reply"), "reply");
        assert_eq!(
            action_name("online"),
            "online",
            "a word that starts with on"
        );
        assert_eq!(action_name("on"), "on");
        assert_eq!(action_name("--"), "activate");
    }

    #[test]
    fn a_tag_or_a_handler_says_control() {
        let is = |p: &ParserNode| match infer(p) {
            Some(Control::Button) => "button",
            Some(Control::Field) => "field",
            Some(Control::Other(_)) => "other",
            None => "none",
        };
        assert_eq!(is(&element("button", &[])), "button");
        assert_eq!(is(&element("input", &[])), "field");
        assert_eq!(is(&element("textarea", &[])), "field");
        let email = AttrValue::Text("email".into());
        assert_eq!(is(&element("input", &[("type", email)])), "field");
        let check = AttrValue::Text("checkbox".into());
        assert_eq!(is(&element("input", &[("type", check)])), "other");
        let click = AttrValue::Ident("onGo".into());
        assert_eq!(is(&element("div", &[("onClick", click)])), "button");
        assert_eq!(is(&element("div", &[])), "none");
        let text = AttrValue::Text("x".into());
        assert_eq!(is(&element("span", &[("title", text)])), "none");
    }
}
