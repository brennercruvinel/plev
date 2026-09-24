//! The real projections the proposers read: the showcase todo's and the
//! ide workbench's, as the goldens of task-68 pin them (each kept equal to
//! its app by a test in that app's crate), and the surfaces the Surface
//! page shows over the todo.

use std::path::PathBuf;

use propose::Ask;
use sem::Snapshot;
use uir::{Capabilities, GenerationPolicy, IrNode, VerifiedIr, verify_json};

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
