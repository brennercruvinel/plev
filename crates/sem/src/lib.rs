//! the semantic contract of plev.
//!
//! every control gets a persistent logical identity (`SemId`), chosen by the
//! app or the IR and never by a counter or a position, a state made of
//! `Value`s under `StateKey`s, the actions it accepts (`ActionDecl`, with the
//! state each one reads and its `Policy`) and the `Constraints` that bound
//! it.
//!
//! the crate is pure data: no gpu, no window, no text stack.

mod decl;
mod error;
mod id;
mod name;
mod text;
mod value;

pub use decl::{ActionDecl, Actors, Confirm, Constraints, ParamSpec, Policy};
pub use error::SemError;
pub use id::SemId;
pub use name::{ActionName, StateKey};
pub use value::{StateRef, Value, ValueKind};
