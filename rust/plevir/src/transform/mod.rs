//! transformations of a verified surface (task-70,
//! docs/adr/generated/0067-adaptive-transforms-preserve-identity.md): the interface
//! changes without being built again.
//!
//! an `Op` is one small change, as data (a person, the rules or an agent
//! asks for it; the recorder keeps it): hide or show a node, group
//! siblings, reorder children, emphasize, change the density, restyle,
//! swap how a datum is shown, promote an action the app declares. each one
//! applies to an `Adapted` surface, the verified IR and the nodes set
//! aside, and gives the surface it makes and the `Inverse` that makes the
//! one before again, byte for byte. both go through the verifier again:
//! nothing comes out that is not a `VerifiedIr`.
//!
//! identity is kept: every id of the surface is in its tree or set aside,
//! and only `Hide` sets one aside. a node set aside is out of the tree the
//! host draws and projects, and kept whole with its place, for `Show` and
//! for the inverse; the vocabulary has no hidden node, and needs none.
//!
//! the IR holds a template once, so an operation on a template acts on
//! every item; an instance's id is refused with the template that makes
//! it.
//!
//! the person's preferences of accessibility (larger text, more contrast,
//! a density) are lists of these operations (`Preferences::over`, phase
//! 4), made on the person's behalf and verified the same way.

mod look;
mod prefer;
mod shape;
mod tree;
mod view;

pub use prefer::{Applied, Preference, Preferences, Preferred};

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use accord::{ActionName, Actor, SemId};
use serde::{Deserialize, Serialize};

use crate::capabilities::Capabilities;
use crate::error::IrError;
use crate::ir::{Ir, IrNode};
use crate::policy::GenerationPolicy;
use crate::token::{ColorRole, Layout, Props};
use crate::verify::{VerifiedIr, verify};
use crate::vocabulary::Component;

/// which way `Density` moves every gap and padding: one step of the
/// spacing scale, down or up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Denser,
    Sparser,
}

/// one transformation of a surface, as data. its JSON form carries the
/// operation's name in `op`: `{"op": "hide", "id": "review.controls"}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Op {
    /// the node leaves what is drawn and projected, kept aside.
    Hide { id: SemId },
    /// a node set aside goes back where it was.
    Show { id: SemId },
    /// siblings become the children of a new container `into`, in the
    /// place of the first of them.
    Group { ids: Vec<SemId>, into: SemId },
    /// `ids` first, in this order, then the other children as they were.
    Reorder { parent: SemId, ids: Vec<SemId> },
    /// one step up the token the node is drawn by.
    Emphasize { id: SemId },
    /// every gap and padding one step of the spacing scale.
    Density { step: Step },
    /// a text's color or a container's fill, by role.
    Restyle { id: SemId, role: ColorRole },
    /// the same datum shown by another component.
    SwapView { id: SemId, component: Component },
    /// a button for an action the app declares and the surface does not
    /// show, at the end of the container `into`.
    Promote {
        target: SemId,
        action: ActionName,
        into: SemId,
    },
}

/// the op in a line, for a history or a log.
impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list = |ids: &[SemId]| {
            let names: Vec<&str> = ids.iter().map(SemId::as_str).collect();
            names.join(", ")
        };
        match self {
            Op::Hide { id } => write!(f, "hide {id}"),
            Op::Show { id } => write!(f, "show {id}"),
            Op::Group { ids, into } => write!(f, "group {} into {into}", list(ids)),
            Op::Reorder { parent, ids } => write!(f, "reorder {parent}: {}", list(ids)),
            Op::Emphasize { id } => write!(f, "emphasize {id}"),
            Op::Density { step: Step::Denser } => f.write_str("density denser"),
            Op::Density {
                step: Step::Sparser,
            } => f.write_str("density sparser"),
            Op::Restyle { id, role } => write!(f, "restyle {id} {}", role.as_str()),
            Op::SwapView { id, component } => write!(f, "swap {id} to {}", view::name(*component)),
            Op::Promote {
                target,
                action,
                into,
            } => write!(f, "promote {action} on {target} into {into}"),
        }
    }
}

/// a node set aside by `Hide`: the node whole, the parent it left and
/// where it was among the parent's children.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Aside {
    pub node: IrNode,
    pub parent: SemId,
    pub at: usize,
}

/// the props and the layout a node had.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Look {
    pub id: SemId,
    #[serde(default, skip_serializing_if = "Props::is_empty")]
    pub props: Props,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
}

/// the way back from one operation, made with it: applied to the surface
/// the operation made, it gives the one before, byte for byte.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Inverse {
    /// the contrary operation, where it is exact: `Show` for `Hide`, the
    /// whole order before for `Reorder`, the component before for
    /// `SwapView`.
    Op(Op),
    /// `Show`'s: the node set aside again, at the place it was kept with.
    Aside { id: SemId, at: usize },
    /// the props and layouts before (`Emphasize`, `Restyle`, `Density`).
    Looks(Vec<Look>),
    /// the node before, whole, in place of the node with its id (`Group`
    /// and `Promote`, the parent).
    Restore(IrNode),
}

/// what an operation is checked against: the registry the surface is made
/// in and the policy it is verified under.
#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    pub caps: &'a Capabilities,
    pub policy: &'a GenerationPolicy,
}

/// why an operation made no surface.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformError {
    /// no node of the surface has the id, in its tree or set aside.
    NotFound(SemId),
    /// the id is an instance of a template: the IR holds the template,
    /// and an operation on it acts on every item.
    Instance { id: SemId, template: SemId },
    /// the operation means nothing for that node, and why.
    Refused(String),
    /// the surface it makes does not pass the verifier: every error.
    Unverified(Vec<IrError>),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::NotFound(id) => write!(f, "no node of the surface is `{id}`"),
            TransformError::Instance { id, template } => write!(
                f,
                "`{id}` is one item of the template `{template}`: transform the template, for every item"
            ),
            TransformError::Refused(why) => f.write_str(why),
            TransformError::Unverified(errors) => {
                let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
                write!(f, "the result does not verify: {}", lines.join("; "))
            }
        }
    }
}

impl std::error::Error for TransformError {}

/// refused, with why.
pub(super) fn refused(why: impl Into<String>) -> TransformError {
    TransformError::Refused(why.into())
}

/// a surface under transformation: the verified IR the host draws and
/// projects, and the nodes set aside, by id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adapted {
    ir: VerifiedIr,
    aside: BTreeMap<SemId, Aside>,
}

/// a change in the making: the IR being changed and the nodes aside.
pub(super) struct Draft {
    pub ir: Ir,
    pub aside: BTreeMap<SemId, Aside>,
}

impl Adapted {
    /// a verified surface, nothing aside.
    pub fn new(ir: VerifiedIr) -> Adapted {
        Adapted {
            ir,
            aside: BTreeMap::new(),
        }
    }

    pub fn verified(&self) -> &VerifiedIr {
        &self.ir
    }

    pub fn ir(&self) -> &Ir {
        self.ir.ir()
    }

    /// the nodes set aside, in id order.
    pub fn aside(&self) -> impl Iterator<Item = &Aside> {
        self.aside.values()
    }

    /// every id of the surface: in its tree, then set aside.
    pub fn ids(&self) -> BTreeSet<SemId> {
        let mut out: BTreeSet<SemId> = tree::ids(&self.ir().surface.root).into_iter().collect();
        for kept in self.aside.values() {
            out.extend(tree::ids(&kept.node));
        }
        out
    }

    /// `op`, asked by `by`: the surface it makes and the way back, or why
    /// there is none.
    pub fn apply(
        &self,
        op: &Op,
        by: &Actor,
        cx: Context<'_>,
    ) -> Result<(Adapted, Inverse), TransformError> {
        let mut d = self.draft();
        let inverse = match op {
            Op::Hide { id } => shape::hide(&mut d, id, None)?,
            Op::Show { id } => shape::show(&mut d, id)?,
            Op::Group { ids, into } => shape::group(&mut d, ids, into)?,
            Op::Reorder { parent, ids } => shape::reorder(&mut d, parent, ids)?,
            Op::Emphasize { id } => look::emphasize(&mut d, id)?,
            Op::Density { step } => look::density(&mut d, *step)?,
            Op::Restyle { id, role } => look::restyle(&mut d, id, *role)?,
            Op::SwapView { id, component } => view::swap(&mut d, id, *component)?,
            Op::Promote {
                target,
                action,
                into,
            } => view::promote(&mut d, (target, action), into, by, cx.caps)?,
        };
        Ok((d.verify(cx)?, inverse))
    }

    /// the surface before the operation `inverse` came with, verified
    /// again.
    pub fn undo(&self, inverse: &Inverse, cx: Context<'_>) -> Result<Adapted, TransformError> {
        let mut d = self.draft();
        match inverse {
            Inverse::Op(op) => {
                // an inverse is never a promotion: no one's permission is asked
                let by = &cx.policy.author;
                return self.apply(op, by, cx).map(|(adapted, _)| adapted);
            }
            Inverse::Aside { id, at } => {
                shape::hide(&mut d, id, Some(*at))?;
            }
            Inverse::Looks(looks) => {
                for look in looks {
                    let node = d.node_mut(&look.id)?;
                    node.props = look.props;
                    node.layout = look.layout;
                }
            }
            Inverse::Restore(before) => {
                let node = d.node_mut(&before.sem)?;
                before.clone_into(node);
            }
        }
        d.verify(cx)
    }

    fn draft(&self) -> Draft {
        Draft {
            ir: self.ir().clone(),
            aside: self.aside.clone(),
        }
    }
}

impl Draft {
    pub fn root(&self) -> &IrNode {
        &self.ir.surface.root
    }

    /// why `id` is not a node of the tree: set aside, an instance, or
    /// nothing.
    pub fn missing(&self, id: &SemId) -> TransformError {
        if let Some(root) = self.aside_root(id) {
            return refused(format!("`{id}` is hidden: show `{root}` first"));
        }
        match tree::template_of(self.root(), id) {
            Some(template) => TransformError::Instance {
                id: id.clone(),
                template,
            },
            None => TransformError::NotFound(id.clone()),
        }
    }

    /// the node `id` of the tree, to change it.
    pub fn node_mut(&mut self, id: &SemId) -> Result<&mut IrNode, TransformError> {
        let Some(path) = tree::path(self.root(), id) else {
            return Err(self.missing(id));
        };
        tree::at_mut(&mut self.ir.surface.root, &path)
            .ok_or_else(|| TransformError::NotFound(id.clone()))
    }

    /// the node set aside whose subtree holds `id`.
    pub fn aside_root(&self, id: &SemId) -> Option<&SemId> {
        self.aside
            .iter()
            .find(|(_, kept)| tree::find(&kept.node, id).is_some())
            .map(|(root, _)| root)
    }

    /// refused when `id` is taken, in the tree or set aside: a new node
    /// takes a new id.
    pub fn free(&self, id: &SemId) -> Result<(), TransformError> {
        if tree::find(self.root(), id).is_some() {
            return Err(refused(format!("`{id}` is already a node of the surface")));
        }
        if let Some(root) = self.aside_root(id) {
            return Err(refused(format!(
                "`{id}` is a node of the surface, hidden in `{root}`"
            )));
        }
        Ok(())
    }

    /// the draft verified, with what is aside: no id both in the tree and
    /// aside.
    fn verify(self, cx: Context<'_>) -> Result<Adapted, TransformError> {
        let ir = verify(&self.ir, cx.caps, cx.policy).map_err(TransformError::Unverified)?;
        let drawn: BTreeSet<SemId> = tree::ids(&self.ir.surface.root).into_iter().collect();
        for kept in self.aside.values() {
            if let Some(twice) = tree::ids(&kept.node)
                .into_iter()
                .find(|id| drawn.contains(id))
            {
                return Err(refused(format!("`{twice}` would be both shown and hidden")));
            }
        }
        Ok(Adapted {
            ir,
            aside: self.aside,
        })
    }
}
