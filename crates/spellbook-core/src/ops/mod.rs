//! Operations, one module per domain - the reads and the writes together.
//!
//! These were axum handlers. The extractors (`Query`, `Path`, `Json`) are now
//! plain arguments and the responses are `Result<T, Error>`; the SQL and the
//! rules inside each one are unchanged. Grouping by domain rather than by
//! read/write is deliberate and predates this port: it keeps a feature's
//! query shapes, SQL and rules in one place.
//!
//! Nothing here spawns a thread or blocks a UI. `client::SpellbookClient`
//! decides what runs where.

pub mod cards;
