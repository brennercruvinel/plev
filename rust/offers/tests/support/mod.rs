//! The real projections the proposers read: the showcase todo's and the
//! ide workbench's, as the goldens of task-68 pin them (each kept equal to
//! its app by a test in that app's crate), and the surfaces the Surface
//! page shows over the todo.

use std::path::PathBuf;

use offers::Ask;
use accord::Snapshot;
use plevir::{Capabilities, GenerationPolicy, IrNode, VerifiedIr, verify_json};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The showcase todo projected, as `crates/uir/fixtures` pins it.
pub fn todo() -> Snapshot {
    serde_json::from_str(&read("../uir/fixtures/showcase-todo.snapshot.json")).unwrap()
}

/// The ide workbench projected, as `crates/uir/fixtures` pins it.
pub fn ide() -> Snapshot {
    serde_json::from_str(&read("../uir/fixtures/ide-workbench.snapshot.json")).unwrap()
}

/// The compact form of the Surface page (the task-68 fixture).
pub fn compact() -> String {
    read("../uir/fixtures/showcase-todo.ir.json")
}

/// The two-column form of the Surface page (phase 2).
pub fn columns() -> String {
    read("../surface/tests/fixtures/showcase-todo-columns.ir.json")
}

/// `ir` verified against `app`.
pub fn verified(ir: &str, app: &Snapshot) -> VerifiedIr {
    let caps = Capabilities::from_snapshot(app);
    verify_json(ir, &caps, &GenerationPolicy::default()).unwrap()
}

/// What a proposer is asked about `app`.
pub struct Asking {
    pub app: Snapshot,
    pub caps: Capabilities,
    pub policy: GenerationPolicy,
    pub current: Option<VerifiedIr>,
}

impl Asking {
    pub fn new(app: Snapshot) -> Asking {
        Asking {
            caps: Capabilities::from_snapshot(&app),
            app,
            policy: GenerationPolicy::default(),
            current: None,
        }
    }

    pub fn over(mut self, current: VerifiedIr) -> Asking {
        self.current = Some(current);
        self
    }

    pub fn ask<'a>(&'a self, intent: &'a str) -> Ask<'a> {
        Ask {
            intent,
            caps: &self.caps,
            app: &self.app,
            current: self.current.as_ref(),
            policy: &self.policy,
        }
    }
}

/// Every node of a tree, depth first.
pub fn nodes(root: &IrNode) -> Vec<&IrNode> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        out.push(n);
        stack.extend(n.children.iter().rev());
    }
    out
}

/// The ids of a tree, depth first.
pub fn ids(root: &IrNode) -> Vec<String> {
    nodes(root).iter().map(|n| n.sem.to_string()).collect()
}

/// `(id, what it binds)` of every bound node: each state as
/// `<prop>=<ref>`, each action as `<event>=<target> <action>`.
pub fn bindings(root: &IrNode) -> Vec<(String, Vec<String>)> {
    nodes(root)
        .into_iter()
        .filter(|n| !n.bind.is_empty())
        .map(|n| {
            let mut b: Vec<String> = n
                .bind
                .state
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            b.extend(
                n.bind
                    .actions
                    .iter()
                    .map(|(k, c)| format!("{k}={} {}", c.target, c.action)),
            );
            (n.sem.to_string(), b)
        })
        .collect()
}

/// Who may ask for the vault's actions: a person, at the screen or
/// through assistive technology, and a test; never an agent (the
/// showcase's `PERSON_ONLY`).
pub const PERSON_ONLY: accord::Actors = accord::Actors {
    human: true,
    agent: false,
    assistive: true,
    test: true,
};

/// A vault whose lock and open are kept from agents, and a secret code.
pub fn vault() -> Snapshot {
    use accord::{Constraints, Policy, Role, SnapshotBuilder};
    let kept = Policy {
        actors: PERSON_ONLY,
        ..Policy::default()
    };
    let mut b = SnapshotBuilder::new();
    b.node("vault", Role::Region, "vault");
    let mut lock = b.node("vault.lock", Role::Toggle, "locked");
    lock.parent("vault").state("on", true);
    lock.action("toggle").reads("vault.lock:on").policy(kept);
    let mut open = b.node("vault.open", Role::Button, "open the vault");
    open.parent("vault");
    open.action("open").policy(kept);
    b.node("vault.note", Role::Label, "note")
        .parent("vault")
        .state("text", "the key is in the drawer");
    let mut code = b.node("vault.code", Role::TextInput, "code");
    code.parent("vault")
        .state("text", "hunter2-swordfish")
        .constraints(Constraints {
            editable: true,
            secret: true,
            ..Constraints::default()
        });
    b.finish().expect("the vault projects")
}

/// A surface a person wrote over the vault: its note and its lock.
pub const VAULT_PANEL: &str = r#"{"plev_ir": 1, "surface": {"id": "vault_panel", "title": "vault",
    "root": {"sem": "vault_panel.root", "layout": {"stack": {"axis": "vertical", "gap": "md"}},
      "children": [
        {"sem": "vault_panel.note", "component": "text", "bind": {"state": {"text": "vault.note:text"}}},
        {"sem": "vault_panel.lock", "component": "checkbox", "label": "locked",
         "bind": {"state": {"checked": "vault.lock:on"},
                  "actions": {"toggle": {"target": "vault.lock", "action": "toggle"}}}}
      ]}}}"#;

/// What a transformer is asked about a surface of `app`.
pub struct Changing {
    pub app: Snapshot,
    pub caps: Capabilities,
    pub policy: GenerationPolicy,
    pub current: plevir::Adapted,
    pub by: accord::Actor,
}

impl Changing {
    /// `ir` over `app`, verified under the default policy, asked by a
    /// person.
    pub fn over(app: Snapshot, ir: &str) -> Changing {
        Changing::under(app, ir, GenerationPolicy::default())
    }

    /// `ir` over `app`, verified under `policy`, asked by a person.
    pub fn under(app: Snapshot, ir: &str, policy: GenerationPolicy) -> Changing {
        let caps = Capabilities::from_snapshot(&app);
        let verified = verify_json(ir, &caps, &policy).unwrap_or_else(|e| panic!("{e:#?}"));
        Changing {
            app,
            caps,
            policy,
            current: plevir::Adapted::new(verified),
            by: accord::Actor::Human,
        }
    }

    pub fn by(mut self, by: accord::Actor) -> Changing {
        self.by = by;
        self
    }

    pub fn ask<'a>(&'a self, intent: &'a str) -> offers::Adapt<'a> {
        offers::Adapt {
            intent,
            caps: &self.caps,
            app: &self.app,
            current: &self.current,
            policy: &self.policy,
            by: &self.by,
        }
    }
}

/// An agent, as the port names one.
pub fn agent() -> accord::Actor {
    accord::Actor::Agent {
        name: "claude".to_owned(),
    }
}

/// The Silences projected, as `crates/uir/fixtures` pins it.
pub fn silences() -> Snapshot {
    serde_json::from_str(&read("../uir/fixtures/showcase-silences.snapshot.json")).unwrap()
}

/// The Claude's review of the checkpoint b's first round, as IR (the
/// cuts in a flat row: range, decision, duration, frame, three buttons).
pub fn claude_review() -> String {
    read("../showcase/fixtures/ir/silences-review-claude.ir.json")
}

/// The ids of the nodes set aside on `adapted`, in id order.
pub fn aside(adapted: &plevir::Adapted) -> Vec<String> {
    adapted.aside().map(|a| a.node.sem.to_string()).collect()
}
