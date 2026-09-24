//! surfaces materialized from verified plev IR.
//!
//! a `uir::VerifiedIr` becomes a live surface over an app's state
//! (`Surface::materialize`), kept with the others the app has open in a
//! `SurfaceHost`:
//!
//! - **layout** by the engine's `LayoutEngine` (a stack is a flex row or
//!   column, a grid a wrapping row, a split two weighted panes, an overlay
//!   one pass per layer), every token read from the current `Theme`;
//! - **drawing** by the comps widgets, one retained widget per node,
//!   indexed by its `SemId` (a repeat unrolled over the live value of its
//!   `List` or `Set`, each instance's id keyed by its item); what a widget
//!   shows is read from the app's `Snapshot`, again whenever its revision
//!   moves (`Surface::sync`);
//! - **semantics** from the IR itself, not from the widgets: role by
//!   `uir::role_of`, label, the state bound, the actions bound
//!   (`Surface::project`), so an agent or a screen reader observes the
//!   surface and acts on it like on any page;
//! - **events** as `sem::Request`s for the capability the IR bound, with
//!   the actor who acted and the state the surface showed as the
//!   observation, handed to the app's own dispatcher (`Dispatch`). the
//!   surface never runs anything.
//!
//! only a `VerifiedIr` materializes; no function here takes an `Ir`:
//!
//! ```
//! fn open(ir: uir::VerifiedIr, app: &sem::Snapshot) -> surface::Surface {
//!     surface::Surface::materialize(ir, app)
//! }
//! ```
//!
//! ```compile_fail
//! // an IR the verifier did not accept does not type-check as one.
//! fn open(ir: uir::Ir, app: &sem::Snapshot) -> surface::Surface {
//!     surface::Surface::materialize(ir, app)
//! }
//! ```

mod draw;
mod expand;
mod host;
mod layout;
mod project;
mod surface;
mod token;
mod widget;

pub use host::{Dispatch, SurfaceError, SurfaceHost};
pub use surface::Surface;
