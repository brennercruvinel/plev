//! rules 4 and 5: every binding names something the app declares, of the
//! kind the component shows or sends. inside a template a reference may
//! carry `[{item}]` and then names every node of the app that fits it,
//! which must all agree.

use std::collections::BTreeSet;

use sem::{StateRef, ValueKind};

use super::{Ctx, Walk};
use crate::caps::ActionCap;
use crate::error::{Ptr, Rule};
use crate::ir::{CapabilityRef, IrNode, Repeat};
use crate::item::{self, Pattern};
use crate::vocab::{Component, spec_of};

/// the names a list of state props or events offers, for a reason.
fn offered<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let names: Vec<String> = names.map(|n| format!("`{n}`")).collect();
    if names.is_empty() {
        "nothing".to_owned()
    } else {
        names.join(", ")
    }
}

fn kinds(list: &[ValueKind]) -> String {
    let names: Vec<&str> = list.iter().map(|k| k.as_str()).collect();
    names.join(" or ")
}

impl Walk<'_> {
    /// rules 4 and 5 for one node. returns the actions it binds, each
    /// with the capability it resolved to, for the policy checks.
    pub(super) fn bindings(&mut self, n: &IrNode, at: &Ptr, ctx: Ctx) -> Vec<(String, ActionCap)> {
        let spec = spec_of(n.component);
        let name = n.component.map_or("container", Component::name);
        let bind = at.key("bind");
        let state = bind.key("state");
        for (prop, r) in &n.bind.state {
            let here = state.key(prop);
            let Some(shown) = spec.state_prop(prop) else {
                let reason = format!(
                    "a {name} shows no `{prop}`; it shows {}",
                    offered(spec.state.iter().map(|p| p.name))
                );
                self.fail(Rule::State, &here, reason);
                continue;
            };
            let Some(kind) = self.state_kind(r, &here, ctx) else {
                continue;
            };
            if !shown.kinds.contains(&kind) {
                let reason = format!(
                    "`{r}` is {}; a {name}'s `{prop}` takes {}",
                    kind.as_str(),
                    kinds(shown.kinds)
                );
                self.fail(Rule::State, &here, reason);
            }
        }
        let actions = bind.key("actions");
        let chip = n.component == Some(Component::Chip);
        if chip && n.bind.actions.contains_key("toggle") && n.bind.actions.contains_key("activate")
        {
            self.fail(
                Rule::Action,
                &actions,
                "a chip is a toggle or a button: it sends `toggle` or `activate`, not both",
            );
        }
        let mut out = Vec::new();
        for (event, c) in &n.bind.actions {
            let here = actions.key(event);
            let Some(sent) = spec.event(event) else {
                let reason = format!(
                    "a {name} sends no `{event}`; it sends {}",
                    offered(spec.events.iter().map(|e| e.name))
                );
                self.fail(Rule::Action, &here, reason);
                continue;
            };
            let Some(cap) = self.action_cap(c, &here, ctx) else {
                continue;
            };
            let takes = cap.param.map(|p| p.kind);
            if sent.param != takes {
                let action = &c.action;
                let reason = match (sent.param, takes) {
                    (None, Some(k)) => format!(
                        "`{action}` takes a {}; a {name}'s `{event}` sends none",
                        k.as_str()
                    ),
                    (Some(k), None) => format!(
                        "`{action}` takes nothing; a {name}'s `{event}` sends a {}",
                        k.as_str()
                    ),
                    (Some(s), Some(t)) => format!(
                        "`{action}` takes a {}; a {name}'s `{event}` sends a {}",
                        t.as_str(),
                        s.as_str()
                    ),
                    (None, None) => String::new(),
                };
                self.fail(Rule::Action, &here, reason);
                continue;
            }
            out.push((event.clone(), cap));
        }
        out
    }

    /// rule 4 for a repeat: it runs over a list or a set the app declares.
    pub(super) fn each(&mut self, r: &Repeat, at: &Ptr, ctx: Ctx) {
        if ctx.template {
            return;
        }
        let Some(kind) = self.state_kind(&r.each, at, ctx) else {
            return;
        };
        if !matches!(kind, ValueKind::List | ValueKind::Set) {
            let reason = format!(
                "`{}` is {}; a repeat runs over a list or a set",
                r.each,
                kind.as_str()
            );
            self.fail(Rule::State, at, reason);
        }
    }

    /// the kind of the state `r` names, or the one kind of every state
    /// that fits it in a template.
    fn state_kind(&mut self, r: &StateRef, at: &Ptr, ctx: Ctx) -> Option<ValueKind> {
        let node = r.node.as_str();
        if item::slots(node) == 0 {
            if let Some(d) = self.caps.datum(r) {
                return Some(d.kind);
            }
            let reason = format!("the app declares no state `{r}`");
            self.fail(Rule::State, at, reason);
            return None;
        }
        let pattern = self.template_pattern(node, at, ctx, Rule::State)?;
        let found: BTreeSet<ValueKind> = self
            .caps
            .data()
            .iter()
            .filter(|d| d.at.key == r.key && pattern.matches(d.at.node.as_str()))
            .map(|d| d.kind)
            .collect();
        let mut kinds = found.iter();
        match (kinds.next(), kinds.next()) {
            (Some(&kind), None) => Some(kind),
            (None, _) => {
                let reason = format!("no state of the app fits `{r}`");
                self.fail(Rule::State, at, reason);
                None
            }
            (Some(_), Some(_)) => {
                let reason = format!("the states that fit `{r}` are not all of one kind");
                self.fail(Rule::State, at, reason);
                None
            }
        }
    }

    /// the capability `c` names, or the one every node that fits it
    /// declares alike in a template.
    fn action_cap(&mut self, c: &CapabilityRef, at: &Ptr, ctx: Ctx) -> Option<ActionCap> {
        let target = c.target.as_str();
        let action = &c.action;
        if item::slots(target) == 0 {
            if let Some(cap) = self.caps.action(&c.target, action) {
                return Some(cap.clone());
            }
            let reason = format!("the app declares no action `{action}` on `{target}`");
            self.fail(Rule::Action, at, reason);
            return None;
        }
        let pattern = self.template_pattern(target, at, ctx, Rule::Action)?;
        let members: Vec<_> = self
            .caps
            .nodes()
            .filter(|id| pattern.matches(id.as_str()))
            .collect();
        if members.is_empty() {
            let reason = format!("no node of the app fits `{target}`");
            self.fail(Rule::Action, at, reason);
            return None;
        }
        let mut first: Option<&ActionCap> = None;
        for id in &members {
            let Some(cap) = self.caps.action(id, action) else {
                let reason = format!(
                    "`{action}` is not on every node that fits `{target}`: `{id}` lacks it"
                );
                self.fail(Rule::Action, at, reason);
                return None;
            };
            match first {
                None => first = Some(cap),
                Some(f) if f.param != cap.param || f.policy != cap.policy => {
                    let reason =
                        format!("the nodes that fit `{target}` declare `{action}` differently");
                    self.fail(Rule::Action, at, reason);
                    return None;
                }
                Some(_) => {}
            }
        }
        first.map(|f| ActionCap {
            target: c.target.clone(),
            ..f.clone()
        })
    }

    /// the pattern of a reference that carries the item: only inside a
    /// template, and only once.
    fn template_pattern(&mut self, text: &str, at: &Ptr, ctx: Ctx, rule: Rule) -> Option<Pattern> {
        if !ctx.template {
            self.fail(rule, at, "`[{item}]` only means something inside a repeat");
            return None;
        }
        let pattern = Pattern::of(text);
        if pattern.is_none() {
            self.fail(
                rule,
                at,
                "one `[{item}]` per reference: the v1 repeat has one item",
            );
        }
        pattern
    }
}
