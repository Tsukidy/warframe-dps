//! Derive (pipeline.md §5.3): attack keys and roles, parent links and groups, triggers, the hand tables of 06 §4.3,
//! enemy body parts and special DR, compatibility, modular parts, stance normalisation and combo-table links, heavy
//! attacks, melee triggers and specials, frames, stable keys. Owner: T18.

mod heavy;
mod stances;

pub use heavy::heavy_def;
pub use stances::normalise_stances;
