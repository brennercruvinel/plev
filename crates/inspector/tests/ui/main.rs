//! the inspector's window without a window: the view is driven through
//! its own handlers and drawn into a `Compositor`, on the sessions the
//! repository keeps (`crates/ide/tests/fixtures/sessions`) and on sessions
//! recorded here the way an app records them.

mod accept;
mod model;
mod privacy;
mod readonly;
mod support;
mod viewport;
