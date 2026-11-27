//! Enemy side (engine.md §6; spec 05): scaling with the f32 blend of NP-2, Steel Path, Eximus, party and overguard
//! rules ([`scaling`]); A0 and A_eff (E-15, [`armor`]); E-12's enemy side per layer and ST-9 routing ([`layers`]);
//! attenuation, flat DR, adaptation and T-28 ([`dr`]); body parts and head-crit ([`parts`]). Owner: T07.

pub mod armor;
pub mod dr;
pub mod layers;
pub mod parts;
pub mod scaling;

pub use armor::{ArmorStrips, armor_dr, effective_armor};
pub use layers::{LayeredHit, Pools, RouteOut, TargetStateView, instance_on_layer, route_layered};
pub use scaling::{ScaledEnemy, ScaledPart, np2_blend, scale_enemy};
