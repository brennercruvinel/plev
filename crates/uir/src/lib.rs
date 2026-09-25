//! the plev IR: an interface surface described by meaning.
//!
//! a surface is a tree of `IrNode`s. each node has an identity (`SemId`),
//! a component of a closed vocabulary that mirrors `crates/comps`
//! (`Component`), props by token only (`Props`), a layout by primitives
//! (`Layout`, `Size`), and bindings: the state it shows comes from the
//! app's state (`StateRef`) and the actions it sends are capabilities the
//! app declares (`CapabilityRef`). the IR never holds a color, a pixel or
//! code, and it can only reference what the app already declares.
//!
//! a person, an LLM, an importer or a tool writes IR as JSON; the version
//! sits on top (`"plev_ir": 1`):
//!
//! ```
//! let ir: uir::Ir = serde_json::from_str(r#"{
//!     "plev_ir": 1,
//!     "surface": {
//!         "id": "commit",
//!         "title": "commit",
//!         "root": {
//!             "sem": "commit.form",
//!             "layout": { "stack": { "axis": "vertical", "gap": "md" } },
//!             "children": [
//!                 { "sem": "commit.message", "component": "text_field",
//!                   "label": "commit message",
//!                   "bind": { "state": { "text": "commit.message:text" },
//!                             "actions": { "set_value": { "target": "commit.message",
//!                                                         "action": "set_value" } } } },
//!                 { "sem": "commit.submit", "component": "button", "label": "commit",
//!                   "props": { "control_size": "md" },
//!                   "bind": { "actions": { "activate": { "target": "commit.submit",
//!                                                        "action": "submit" } } } }
//!             ]
//!         }
//!     }
//! }"#)?;
//! assert_eq!(ir.surface.root.children.len(), 2);
//! # Ok::<(), serde_json::Error>(())
//! ```
//!
//! the crate is pure data, like `sem`: no gpu, no window, no engine and
//! no comps. materializing a surface on the widgets is task-69; changing
//! a verified surface by small operations, each verified again and with
//! its inverse, is `transform` (task-70).

mod caps;
mod decode;
mod error;
mod ir;
mod item;
mod policy;
mod schema;
mod token;
mod transform;
mod verify;
mod vocab;

pub use caps::{ActionCap, Capabilities, DataCap};
pub use decode::{decode, parse};
pub use error::{IrError, Rule};
pub use ir::{Bind, CapabilityRef, ITEM, Ir, IrNode, Repeat, Surface, VERSION};
pub use policy::{GenerationPolicy, HARD_DEPTH};
pub use schema::schema;
pub use token::{
    Align, Axis, ColorRole, ControlSize, Elevation, Layout, MAX_FLEX, MAX_RATIO, MIN_RATIO, Props,
    ShapeRole, Size, SizeToken, SpacingToken, TypographyRole,
};
pub use transform::{Adapted, Aside, Context, Inverse, Look, Op, Step, TransformError};
pub use verify::{VerifiedIr, verify, verify_json};
pub use vocab::{
    CONTAINER, ChartKind, Component, Event, Named, Spec, StateProp, is_control, names, role_of,
    spec_of,
};
