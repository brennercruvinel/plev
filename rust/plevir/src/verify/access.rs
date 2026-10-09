//! rules 6 and 7: the generation policy (who may bind what, and a
//! destructive action only behind a confirmation) and accessibility (every
//! control named, a focus order that follows what a person sees, no
//! control that is only visual).

use accord::{Actor, Actors, Confirm};

use super::{Ctx, Walk};
use crate::capabilities::ActionCap;
use crate::error::{Ptr, Rule};
use crate::ir::IrNode;
use crate::token::Layout;
use crate::vocabulary::{Component, Named, is_control, role_of, spec_of};

/// true when `actors` lets `actor` ask, the mapping `accord::decision` uses.
fn allows(actors: &Actors, actor: &Actor) -> bool {
    match actor {
        Actor::Human => actors.human,
        Actor::Agent { .. } => actors.agent,
        Actor::Assistive => actors.assistive,
        Actor::Test { .. } => actors.test,
    }
}

fn kind(actor: &Actor) -> &'static str {
    match actor {
        Actor::Human => "people",
        Actor::Agent { .. } => "agents",
        Actor::Assistive => "assistive technology",
        Actor::Test { .. } => "tests",
    }
}

fn blank(text: Option<&str>) -> bool {
    text.is_none_or(|t| t.trim().is_empty())
}

/// true when some node of the tree under `n`, `n` included, is a control.
fn holds_control(n: &IrNode) -> bool {
    let mut stack = vec![n];
    while let Some(node) = stack.pop() {
        if is_control(role_of(node.component, &node.bind)) {
            return true;
        }
        stack.extend(node.children.iter());
        if let Some(repeat) = &node.repeat {
            stack.push(&repeat.item);
        }
    }
    false
}

impl Walk<'_> {
    /// rule 6 for the actions one node binds, each with the capability it
    /// resolved to.
    pub(super) fn generation(&mut self, n: &IrNode, at: &Ptr, bound: &[(String, ActionCap)]) {
        let actions = at.key("bind").key("actions");
        let name = n.component.map_or("container", Component::name);
        let author = &self.policy.author;
        for (event, cap) in bound {
            let here = actions.key(event);
            let (target, action) = (&cap.target, &cap.action);
            if !allows(&cap.policy.actors, author) {
                let reason = format!(
                    "{author} may not bind `{action}` on `{target}`: its policy leaves {} out",
                    kind(author)
                );
                self.fail(Rule::Policy, &here, reason);
            }
            let confirmed = cap.policy.confirm == Confirm::Always
                || (n.component == Some(Component::Dialog) && event == "confirm");
            if cap.policy.destructive && !confirmed {
                let reason = format!(
                    "`{action}` on `{target}` is destructive and the app asks no one to confirm it: \
                     bind it to a dialog's `confirm`, not to a {name}'s `{event}`"
                );
                self.fail(Rule::Policy, &here, reason);
            }
        }
    }

    /// rule 7 for the surface's title, and rule 8 for its length.
    pub(super) fn title(&mut self, title: &str, at: &Ptr) {
        if blank(Some(title)) {
            self.fail(Rule::Access, at, "a surface needs a title that names it");
        }
        self.text(Some(title), at);
    }

    /// rule 7 for one node; a control joins the focus order.
    pub(super) fn access(&mut self, n: &IrNode, at: &Ptr, ctx: Ctx) {
        let spec = spec_of(n.component);
        let name = n.component.map_or("container", Component::name);
        let label = n.label.as_deref();
        let here = at.key("label");
        if ctx.item && label.is_some() {
            self.fail(
                Rule::Access,
                &here,
                "a repeated item is named by its item; a fixed label would name every item alike",
            );
        }
        match spec.named {
            Named::Always if blank(label) && !ctx.item => {
                let reason = format!("a {name} needs a label: nothing else names it");
                self.fail(Rule::Access, &here, reason);
            }
            Named::OrBound(prop)
                if blank(label) && !ctx.item && !n.bind.state.contains_key(prop) =>
            {
                let reason = format!("a {name} needs a label or its `{prop}` bound");
                self.fail(Rule::Access, &here, reason);
            }
            Named::Never if label.is_some() || n.description.is_some() => {
                let reason = format!("a {name} has no role, so nothing can name or describe it");
                self.fail(Rule::Access, &here, reason);
            }
            _ => {}
        }
        if is_control(role_of(n.component, &n.bind)) {
            if n.bind.actions.is_empty() {
                let reason = format!(
                    "a {name} with no action bound is only visual: bind the capability it operates"
                );
                self.fail(Rule::Access, &at.key("bind"), reason);
            }
            self.focus.push(n.sem.clone());
        }
        if n.layout == Some(Layout::Overlay) {
            let layers = n.children.iter().filter(|c| holds_control(c)).count();
            if layers > 1 {
                let reason = format!(
                    "controls in {layers} layers of an overlay share a place: keep them in one \
                     layer so the focus order follows what a person sees"
                );
                self.fail(Rule::Access, &at.key("layout"), reason);
            }
        }
    }
}
