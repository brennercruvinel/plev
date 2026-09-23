//! the semantic contract of plev.
//!
//! every control gets a persistent logical identity (`SemId`), chosen by the
//! app or the IR and never by a counter or a position, a `Role`, a state
//! made of `Value`s under `StateKey`s, the actions it accepts (`ActionDecl`,
//! with the state each one reads and its `Policy`) and the `Constraints`
//! that bound it.
//!
//! semantics is a pure projection of app state. the app implements
//! `Semantics::project`, which is called on demand and always from the live
//! state; `SnapshotBuilder::finish` validates the projection as a whole and
//! returns a `Snapshot` or every violation at once. what is not projected
//! does not exist for the contract: a hover, a cursor or an animation can
//! never invalidate an action, because none of them is in the snapshot.
//!
//! the git client of the thesis: the staged files and the message are
//! semantic state, and the commit action says it depends on both.
//!
//! ```
//! use std::collections::BTreeSet;
//!
//! use sem::{Change, Role, SemId, Semantics, Snapshot, SnapshotBuilder, StateRef, Value};
//!
//! struct GitState {
//!     staged: BTreeSet<String>,
//!     message: String,
//! }
//!
//! impl Semantics for GitState {
//!     fn project(&self, b: &mut SnapshotBuilder) {
//!         b.node("changes", Role::List, "changes")
//!             .state("staged", self.staged.clone());
//!         b.node("commit.message", Role::TextInput, "commit message")
//!             .state("text", self.message.as_str());
//!         b.node("commit.submit", Role::Button, "commit")
//!             .enabled(!self.staged.is_empty())
//!             .action("submit")
//!             .reads("changes:staged")
//!             .reads("commit.message:text");
//!     }
//! }
//!
//! let git = GitState {
//!     staged: BTreeSet::from(["lexer.rs".to_owned(), "parser.rs".to_owned()]),
//!     message: "fix parser".to_owned(),
//! };
//! let mut b = SnapshotBuilder::new();
//! git.project(&mut b);
//! let snap = b.finish()?;
//!
//! let staged = StateRef::parse("changes:staged")?;
//! let message = StateRef::parse("commit.message:text")?;
//! assert_eq!(snap.value(&staged), Some(&Value::Set(git.staged.clone())));
//! let submit = snap.get(&SemId::parse("commit.submit")?).ok_or("not projected")?;
//! assert_eq!(submit.actions[0].name.as_str(), "submit");
//! assert_eq!(submit.actions[0].reads, [staged.clone(), message]);
//!
//! let mut again = SnapshotBuilder::new();
//! git.project(&mut again);
//! assert_eq!(again.finish()?.revision(), snap.revision());
//!
//! let mut git = git;
//! git.staged.remove("lexer.rs");
//! let mut b = SnapshotBuilder::new();
//! git.project(&mut b);
//! let now = b.finish()?;
//! let delta = Snapshot::diff(&snap, &now);
//! assert!(matches!(
//!     &delta[..],
//!     [Change::StateChanged { at, removed, .. }] if *at == staged && removed == &["lexer.rs"]
//! ));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! a projection of the same state gives the same `Revision` on every run
//! and every target, so a revision can travel to an agent or a recording
//! and still mean the same thing. when the state moves under an agent that
//! observed it (a file unstaged while it was deciding to commit),
//! `Snapshot::diff` names what changed: `removed: ["lexer.rs"]`.
//!
//! the crate is pure data: no gpu, no window, no text stack.

mod builder;
mod decl;
mod diff;
mod error;
mod hash;
mod id;
mod name;
mod node;
mod request;
mod role;
mod snapshot;
mod text;
mod validate;
mod value;

pub use builder::{ActionBuilder, NodeBuilder, Semantics, SnapshotBuilder};
pub use decl::{ActionDecl, Actors, Confirm, Constraints, ParamSpec, Policy};
pub use diff::Change;
pub use error::{SemError, SemErrors};
pub use hash::Revision;
pub use id::SemId;
pub use name::{ActionName, StateKey};
pub use node::{Node, Source};
pub use request::{Actor, Observation, Request};
pub use role::Role;
pub use snapshot::Snapshot;
pub use value::{StateRef, Value, ValueKind};
