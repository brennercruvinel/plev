//! the semantic contract of plev.
//!
//! every control gets a persistent logical identity (`SemId`), chosen by the
//! app or the IR and never by a counter or a position. the rest of the
//! contract (meaning, state, actions, constraints) builds on it.
//!
//! the crate is pure data: no gpu, no window, no text stack.

mod error;
mod id;
mod name;
mod text;

pub use error::SemError;
pub use id::SemId;
