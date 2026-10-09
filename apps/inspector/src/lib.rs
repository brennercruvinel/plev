//! the session inspector's window (task-64 phase b,
//! docs/adr/recording/0046-the-inspector-reads-a-session-never-runs-it.md).
//!
//! it opens a recorded session (`record::Session`, from the path on the
//! command line) and shows what the `inspect` model answers about it:
//!
//! - the strip (`Strip`): every event on a lane per actor and one for the
//!   app's state, in the tone of its kind, the counts by kind, and the
//!   scrub over the session's span;
//! - the frame on screen at the instant (`Session::scene_at`), fit into
//!   its rect (`Fit`), with the outline of every control where it showed
//!   (`Session::bindings_at`) and the id of the one under the pointer
//!   (`Session::hit_at`);
//! - the panel (`Panel`): a click on a control shows the model's answers
//!   about it at the instant (`Inspector::control`, written by
//!   `replay::prose`): which control, its state, who observed what, the
//!   request, the verdict with its delta, what happened in between, the
//!   result; then what came after the instant, as the whole session tells
//!   it.
//!
//! it only reads. `InspectorView` owns the session and nothing else: no
//! app, nothing that dispatches, no recorder, no socket, no process. its
//! handlers move the instant, the pointer and the pick; none has anything
//! to send. the recorded frame is a picture: it binds nothing in the
//! view's compositor. the view draws into any `Compositor`, so its tests
//! run without a window; `window` (desktop) is the only part that opens
//! one.

mod fit;
mod input;
mod panel;
mod render;
mod strip;
mod view;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub mod window;

pub use fit::{Fit, extent};
pub use panel::{Panel, wrap};
pub use strip::{Geometry, Mark, Strip, Tone};
pub use view::{InspectorView, Key, Layout, OpenError};
