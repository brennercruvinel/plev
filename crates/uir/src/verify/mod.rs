//! the verifier: the only way to a `VerifiedIr`.
//!
//! `verify` walks the surface once, depth first, and checks every node
//! against the rules of `Rule` in their order, collecting every error with
//! its JSON pointer. only an IR with no error comes out verified: a
//! person, a model, an importer or a tool proposes; the verifier decides
//! what may exist; only a `VerifiedIr` reaches the runtime.

mod bind;

use std::collections::BTreeMap;

use sem::SemId;

use crate::caps::Capabilities;
use crate::decode::parse;
use crate::error::{IrError, Ptr, Rule};
use crate::ir::{Ir, IrNode, Repeat, VERSION};
use crate::item::{self, Pattern};
use crate::policy::{GenerationPolicy, HARD_DEPTH};
use crate::token::{Layout, MAX_FLEX, MAX_RATIO, MIN_RATIO};
use crate::vocab::{Component, spec_of};

/// an IR that passed every check against one capability registry under
/// one policy. there is no other way to build one: no public constructor,
/// no `Deserialize`, no `Default`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedIr {
    ir: Ir,
}

impl VerifiedIr {
    pub fn ir(&self) -> &Ir {
        &self.ir
    }

    pub fn into_ir(self) -> Ir {
        self.ir
    }
}

/// checks `ir` against what the app declares (`caps`) under `policy`:
/// `Ok` with the verified IR, or every error at once.
pub fn verify(
    ir: &Ir,
    caps: &Capabilities,
    policy: &GenerationPolicy,
) -> Result<VerifiedIr, Vec<IrError>> {
    let top = Ptr::root();
    if ir.plev_ir != VERSION {
        let reason = format!(
            "version {}; this runtime reads version {VERSION}",
            ir.plev_ir
        );
        return Err(vec![IrError::new(
            Rule::Schema,
            &top.key("plev_ir"),
            reason,
        )]);
    }
    let surface = top.key("surface");
    let nodes = count(&ir.surface.root);
    if nodes > policy.max_nodes {
        let reason = format!("{nodes} nodes; the policy allows {}", policy.max_nodes);
        return Err(vec![IrError::new(
            Rule::Limits,
            &surface.key("root"),
            reason,
        )]);
    }
    let mut walk = Walk {
        caps,
        policy,
        errors: Vec::new(),
        ids: BTreeMap::new(),
        statics: Vec::new(),
        templates: Vec::new(),
    };
    walk.surface_id(&ir.surface.id, &surface.key("id"));
    walk.text(Some(&ir.surface.title), &surface.key("title"));
    let root = Ctx {
        depth: 1,
        template: false,
    };
    walk.node(&ir.surface.root, &surface.key("root"), root);
    walk.collisions();
    if walk.errors.is_empty() {
        Ok(VerifiedIr { ir: ir.clone() })
    } else {
        Err(walk.errors)
    }
}

/// reads `text` (`parse`) and verifies it: the shape errors when it does
/// not read, the verifier's otherwise.
pub fn verify_json(
    text: &str,
    caps: &Capabilities,
    policy: &GenerationPolicy,
) -> Result<VerifiedIr, Vec<IrError>> {
    verify(&parse(text)?, caps, policy)
}

/// every node of the tree, a template once, without recursion.
fn count(root: &IrNode) -> usize {
    let mut stack = vec![root];
    let mut n = 0usize;
    while let Some(node) = stack.pop() {
        n += 1;
        stack.extend(node.children.iter());
        if let Some(repeat) = &node.repeat {
            stack.push(&repeat.item);
        }
    }
    n
}

/// where the walk is: how deep, and whether inside a repeat's template.
#[derive(Clone, Copy, Debug)]
struct Ctx {
    depth: usize,
    template: bool,
}

impl Ctx {
    fn child(self) -> Ctx {
        Ctx {
            depth: self.depth + 1,
            ..self
        }
    }
}

struct Walk<'a> {
    caps: &'a Capabilities,
    policy: &'a GenerationPolicy,
    errors: Vec<IrError>,
    /// where each id was first seen.
    ids: BTreeMap<SemId, Ptr>,
    /// the ids outside a template, and the patterns of the templates:
    /// an instance must never take a fixed id.
    statics: Vec<(SemId, Ptr)>,
    templates: Vec<(Pattern, Ptr)>,
}

impl Walk<'_> {
    fn fail(&mut self, rule: Rule, at: &Ptr, reason: impl Into<String>) {
        self.errors.push(IrError::new(rule, at, reason));
    }

    fn node(&mut self, n: &IrNode, at: &Ptr, ctx: Ctx) {
        let max = self.policy.max_depth.min(HARD_DEPTH);
        if ctx.depth > max {
            let reason = format!("depth {}; the policy allows {max}", ctx.depth);
            self.fail(Rule::Limits, at, reason);
            return;
        }
        self.identity(n, at, ctx);
        self.vocabulary(n, at);
        self.bindings(n, at, ctx);
        self.text(n.label.as_deref(), &at.key("label"));
        self.text(n.description.as_deref(), &at.key("description"));
        let children = at.key("children");
        for (i, child) in n.children.iter().enumerate() {
            self.node(child, &children.index(i), ctx.child());
        }
        if let Some(repeat) = &n.repeat {
            self.repeat(repeat, &at.key("repeat"), ctx);
        }
    }

    fn repeat(&mut self, r: &Repeat, at: &Ptr, ctx: Ctx) {
        if ctx.template {
            self.fail(
                Rule::Vocabulary,
                at,
                "a repeat inside a repeat needs two items; the v1 repeat has one",
            );
        }
        self.each(r, &at.key("each"), ctx);
        let item = Ctx {
            depth: ctx.depth + 1,
            template: true,
        };
        self.node(&r.item, &at.key("item"), item);
    }

    /// rule 2 for the surface's own id: it names the surface, so no node
    /// takes it.
    fn surface_id(&mut self, id: &SemId, at: &Ptr) {
        if item::slots(id.as_str()) > 0 {
            self.fail(
                Rule::Identity,
                at,
                "`[{item}]` only means something inside a repeat",
            );
        }
        self.statics.push((id.clone(), at.clone()));
        self.ids.insert(id.clone(), at.clone());
    }

    /// rule 2: the id is unique, carries the item exactly when inside a
    /// template, and a node that takes an app's id sends only that node's
    /// actions.
    fn identity(&mut self, n: &IrNode, at: &Ptr, ctx: Ctx) {
        let text = n.sem.as_str();
        let here = at.key("sem");
        let slots = item::slots(text);
        let pattern = Pattern::of(text);
        match (ctx.template, slots) {
            (true, 0) => self.fail(
                Rule::Identity,
                &here,
                format!("a node inside a repeat carries the item in its id, as `{text}[{{item}}]`"),
            ),
            (true, 1) => {
                if let Some(p) = &pattern {
                    self.templates.push((p.clone(), here.clone()));
                }
            }
            (true, _) => self.fail(
                Rule::Identity,
                &here,
                "one `[{item}]` per id: the v1 repeat has one item",
            ),
            (false, 0) => self.statics.push((n.sem.clone(), here.clone())),
            (false, _) => self.fail(
                Rule::Identity,
                &here,
                "`[{item}]` only means something inside a repeat",
            ),
        }
        match self.ids.get(&n.sem) {
            Some(first) => {
                let reason = format!("`{text}` is already the id at {}", first.as_str());
                self.fail(Rule::Identity, &here, reason);
            }
            None => {
                self.ids.insert(n.sem.clone(), here);
            }
        }
        let app = match &pattern {
            Some(p) if ctx.template => self.caps.nodes().any(|id| p.matches(id.as_str())),
            _ => self.caps.has_node(&n.sem),
        };
        if !app {
            return;
        }
        let actions = at.key("bind").key("actions");
        for (event, cap) in &n.bind.actions {
            if cap.target != n.sem {
                let reason = format!(
                    "`{text}` is the app's own node, so it sends its own actions only; `{}` is another node",
                    cap.target
                );
                self.fail(Rule::Identity, &actions.key(event), reason);
            }
        }
    }

    /// rule 2, once the walk is done: no fixed id is also what a template
    /// makes for some item.
    fn collisions(&mut self) {
        let mut found = Vec::new();
        for (id, at) in &self.statics {
            for (pattern, template) in &self.templates {
                if pattern.matches(id.as_str()) {
                    let reason = format!(
                        "`{id}` is also what the template at {} makes for one item",
                        template.as_str()
                    );
                    found.push(IrError::new(Rule::Identity, at, reason));
                }
            }
        }
        self.errors.extend(found);
    }

    /// rule 3: children, layout and repeat only where the component holds
    /// children, and a layout's numbers within their bounds.
    fn vocabulary(&mut self, n: &IrNode, at: &Ptr) {
        let spec = spec_of(n.component);
        let name = n.component.map_or("container", Component::name);
        if !spec.container {
            if !n.children.is_empty() {
                let reason = format!("a {name} holds no children");
                self.fail(Rule::Vocabulary, &at.key("children"), reason);
            }
            if n.layout.is_some() {
                let reason = format!("a {name} arranges no children");
                self.fail(Rule::Vocabulary, &at.key("layout"), reason);
            }
            if n.repeat.is_some() {
                let reason = format!("a {name} repeats nothing; a list or a container does");
                self.fail(Rule::Vocabulary, &at.key("repeat"), reason);
            }
        } else if n.repeat.is_some() {
            if !matches!(n.component, None | Some(Component::List)) {
                let reason = format!("a {name} repeats nothing; a list or a container does");
                self.fail(Rule::Vocabulary, &at.key("repeat"), reason);
            }
            if !n.children.is_empty() {
                self.fail(
                    Rule::Vocabulary,
                    &at.key("children"),
                    "a node that repeats has no children of its own: its instances are",
                );
            }
        }
        if let Some(Layout::Split { ratio, .. }) = n.layout {
            let kids = if n.repeat.is_some() {
                "a repeat".to_owned()
            } else {
                n.children.len().to_string()
            };
            if n.repeat.is_some() || n.children.len() != 2 {
                let reason = format!("a split holds exactly two children, not {kids}");
                self.fail(Rule::Vocabulary, &at.key("layout"), reason);
            }
            if !(MIN_RATIO..=MAX_RATIO).contains(&ratio) {
                let reason = format!(
                    "ratio {ratio}; a split gives its first child {MIN_RATIO} to {MAX_RATIO} percent"
                );
                let here = at.key("layout").key("split").key("ratio");
                self.fail(Rule::Vocabulary, &here, reason);
            }
        }
        if let Some(flex) = n.size.flex {
            if flex == 0 || flex > MAX_FLEX {
                let reason = format!("flex {flex}; a weight goes from 1 to {MAX_FLEX}");
                self.fail(Rule::Vocabulary, &at.key("size").key("flex"), reason);
            }
        }
    }

    /// rule 8 for one text.
    fn text(&mut self, text: Option<&str>, at: &Ptr) {
        let Some(text) = text else {
            return;
        };
        let len = text.chars().count();
        if len > self.policy.max_text {
            let reason = format!(
                "{len} characters; the policy allows {}",
                self.policy.max_text
            );
            self.fail(Rule::Limits, at, reason);
        }
    }
}
