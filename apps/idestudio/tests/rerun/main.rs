//! sessions become regression tests (task-64 phase c,
//! docs/adr/recording/0050-sessions-become-regression-tests.md).
//!
//! an excerpt of a recorded session (the `.monster` a bug report is) and a
//! small file of marks sit in `tests/fixtures/sessions/`; the harness
//! reruns the excerpt's acts, in the recorded order, against a fixture of
//! the app, and says where the verdicts or the marked values differ.
//!
//! this is re-execution under control, and only here: the fixture runs
//! the app in a temporary directory, on the clock the test gives where the
//! app takes one, with no network, and nothing touches real state outside
//! it. replay (the recorder, the inspector) still runs nothing.
//!
//! - `harness`: the `Fixture` an app provides and `rerun`, the executor.
//! - `marks`: the format of the marks.
//! - `report`: what the rerun says, request by request and mark by mark.
//! - `toy`: the harness against a toy fixture.
//! - `lab`: the ide's fixture, and the laboratory's sessions rerun on it.
//! - `inflight`: the same fixture settled only as far as its executions,
//!   and the task-84 race caught by a recorded session (task-80).

mod eval;
mod harness;
mod inflight;
mod marks;
mod report;
mod toy;
