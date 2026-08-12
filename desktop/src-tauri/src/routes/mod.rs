//! HTTP routes, one module per domain.
//!
//! Each module owns **both** halves of its domain — the reads and the writes. The first cut of
//! this API was split the other way, `api.rs` holding every GET and `writes.rs` every POST, and
//! it made the seam run across features instead of between them: adding one endpoint meant
//! editing two files, and a read handler ended up reaching into the write module for a status
//! it needed. Grouping by domain keeps a feature's request shapes, SQL and rules in one place.
//!
//! To add an endpoint: find the module that owns the noun, put the handler beside its siblings,
//! and register it in that module's `router()`. Nothing here needs to change.

use axum::Router;

pub mod cards;
pub mod collection;
pub mod data;
pub mod decks;
pub mod games;
pub mod wishlist;

pub fn router() -> Router {
    Router::new()
        .merge(cards::router())
        .merge(decks::router())
        .merge(collection::router())
        .merge(games::router())
        .merge(data::router())
        .merge(wishlist::router())
}
