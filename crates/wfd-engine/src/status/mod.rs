//! Status effects (ST-10; spec 03): the status rules from the bundle (03 §4.1), DoT ticks (E-13, 03 E-3.4, E-3.5),
//! Heat (03 §10), Blast (03 §11), the Magnetic break (03 §13), the distinct-status count (03 §14) and extra hits
//! (E-14). Pure functions over a [`crate::StageCtx`]; the evaluators decide when they run. Owner: T14.

pub mod blast;
pub mod distinct;
pub mod dot;
pub mod extra_hit;
pub mod heat;
pub mod magnetic_break;
pub mod rules;

pub use dot::{ProcSnapshot, dot_tick};
pub use extra_hit::{ExtraHitOut, HitWeaponSide, extra_hit};
