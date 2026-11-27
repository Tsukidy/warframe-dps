//! `wfd-data`: the game data at run time (overview.md §3; pipeline.md §8).
//!
//! - [`encode_bundle`] and [`decode_bundle`]: the binary bundle of pipeline.md §8.2 (embedded at compile time by
//!   `build.rs`, written by the in-app refresh).
//! - [`GameData`]: one loaded bundle with its indexes by StableKey and uniqueName (pipeline.md §8.3), search, the
//!   catalogue, detail and schema assembly of server-api.md §5, candidates for `/api/candidates`, and document
//!   resolution with the `x` map of session-local keys (domain-model.md §9.2, AD-18).
//! - [`legality`]: spec 09, implemented once (AD-24); validation never blocks evaluation (09 §1.5).
//!
//! Depends on `wfd-model` only (overview.md §3.1). The public API was fixed by T02b (work-breakdown.md T02b) and is
//! implemented by T04; every body T04 owns is `todo!("T04 …")`, and the tests that need one are `#[ignore = "T04 …"]`.
//! A change to these signatures is a `CONTRACTS.md` entry (work-breakdown.md §2.1).

#![warn(missing_docs)]

mod codec;
mod error;
mod game_data;
pub mod legality;

pub use codec::{decode_bundle, encode_bundle};
pub use error::DataError;
pub use game_data::{Entity, EntityKind, GameData, SearchHit, SearchQuery};
