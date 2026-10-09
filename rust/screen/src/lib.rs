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
//! - **pictures** asked of the app (`Frames`): an `image` binds its
//!   `source` to a frame ref the app projects (a media node and an
//!   instant), and the host asks the app for the decoded frame by that
//!   text when it draws (`Surface::render_with`); without one it draws
//!   the image's shell and label, and `Surface::sources` says what the
//!   app should decode;
//! - **events** as `sem::Request`s for the capability the IR bound, with
//!   the actor who acted and the state the surface showed as the
//!   observation, handed to the app's own dispatcher (`Dispatch`). the
//!   surface never runs anything;
//! - **transformation** by another `VerifiedIr` (`SurfaceHost::transform`),
//!   reconciled by `SemId`: a node that stays under the same widget keeps
//!   it (focus, caret, a drag in flight), a node born is a capability at
//!   once and fades in, a node gone is none at once and its picture fades
//!   out where it was, a node that moved slides, on the theme's motion and
//!   durations, instant with reduced motion; a dismissal (`close`) fades
//!   the surface out and then lets it go. render on demand: frames are
//!   asked for only while something moves (`wake`);
//! - **adaptation** by one small operation at a time (`SurfaceHost::apply`,
//!   a `uir::Op`: hide, show, group, reorder, emphasize, density, restyle,
//!   swap a view, promote an action), verified again and reconciled the
//!   same way, kept with its inverse and who asked; `undo` takes back the
//!   last one.
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

mod code;
mod draw;
mod expand;
mod host;
mod layout;
mod project;
mod surface;
mod token;
mod widget;
mod transition;

pub use host::{Change, Dispatch, Frames, NoFrames, SurfaceError, SurfaceHost};
pub use surface::Surface;
