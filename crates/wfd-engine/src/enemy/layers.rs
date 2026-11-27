//! E-12's enemy side per layer (φ, Viral, Magnetic, V, armor and its floor of 1, non-armor DR R_ℓ, attenuation CD-09)
//! and ST-9 routing for the `layered` view (OG → SH → HP, Toxin routing and exceptions, overflow and the shield gate
//! CD-08; 05 E-5.14-E-5.25). Owner: T07.

use wfd_model::{DamageType, DamageVec, Layer, N_ST};

use crate::StageCtx;
use crate::enemy::scaling::ScaledEnemy;

/// The target state one instance sees (CD-04: just before the instance), as E-12 reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetStateView {
    /// Live stacks n_s (external stacks included, 03 §6.3).
    pub stacks: [u8; N_ST],
    /// Effective armor A_eff at the instance (E-15).
    pub armor_eff: f64,
    /// Vulnerability V_ℓ,t (05 E-5.19, `vuln[ℓ][t]`), indexed by `Layer::index`.
    pub vuln: [DamageVec; Layer::N],
    /// The instance is a status instance (status DR applies, 05 E-5.20).
    pub status_instance: bool,
    /// Part DR of the part hit (05 E-5.20).
    pub part_dr: f64,
}

/// E-12's enemy side of one instance on one layer: Σ_t D_t for the weapon-side vector `w`, after the floor of 1 and
/// R_ℓ, and attenuation where the enemy has it (never on OG). The weapon-side overguard factor M_OG is a scalar the
/// caller applies on the OG layer. T07.
pub fn instance_on_layer(
    ctx: &StageCtx,
    w: &DamageVec,
    layer: Layer,
    st: &TargetStateView,
    en: &ScaledEnemy,
) -> f64 {
    let _ = (ctx, w, layer, st, en);
    todo!("T07")
}

/// One instance's damage per layer for routing (ST-9).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayeredHit {
    /// Every type against OG (M_OG applied).
    pub og: f64,
    /// The types not in B_SH against SH.
    pub sh: f64,
    /// The types in B_SH against HP while SH > 0.
    pub hp_bypass: f64,
    /// Every type against HP (after SH is gone).
    pub hp: f64,
    /// D^HP_t per type (E-5.24 values; `hp` is their sum): routing forms h_t from them for `dmg_by_type` of the
    /// adaptation state (05 E-5.23, E-5.25). Addition to engine.md §6 (CONTRACTS.md T02a).
    pub hp_by_type: DamageVec,
    /// Weak-point hit (overflow at 100 % through the shield gate, CD-08).
    pub weak_point: bool,
    /// A status tick (not gated, CD-08).
    pub tick: bool,
}

/// Remaining pools of the layered target with the state routing keeps (05 §10, E-5.23, E-5.25; 08 §6.2 "pools
/// (layered passes): OG, SH, HP, gate_until, adaptation state, health-threshold flags").
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pools {
    /// Overguard left.
    pub og: f64,
    /// Shields left.
    pub sh: f64,
    /// Health left.
    pub hp: f64,
    /// End of the active shield gate τ̂ + 0.1 s on the µs grid (08 E-8.0a; the gate test `τ < gate_until` of E-5.25
    /// is an integer comparison); `None` when no gate runs (−∞ at the start of the pass).
    pub gate_until: Option<i64>,
    /// Adaptation state of Sentients and the Shadow Stalker (05 E-5.23; layered passes only).
    pub adaptation: AdaptationState,
    /// Health-threshold flags: bit k is set while HP/HP_max is below the k-th `StateVar::HealthBelow` threshold of
    /// the scenario's state-variable table, in table order (04 §4.3 `target_health_below(f)`: live in the layered
    /// pass only; 08 §6.2). Updated after each routed instance.
    pub health_below: u32,
}

/// Adaptation state (05 E-5.23): adapted types, health damage by type, gates crossed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AdaptationState {
    /// Adapted types `a[1..4]`, in adaptation order (`adapted[..n]`).
    pub adapted: [Option<DamageType>; 4],
    /// Number of adapted types.
    pub n: u8,
    /// Gates g = 0.25, 0.45, 0.65, 0.80 of HP_max lost, crossed once each (bit j = gate j).
    pub gates_crossed: u8,
    /// Health damage taken by type after all modifiers (`dmg_by_type[t]`).
    pub dmg_by_type: DamageVec,
}

/// What one routed instance did (ST-9).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RouteOut {
    /// Damage dealt per layer, indexed by `Layer::index`.
    pub dealt: [f64; Layer::N],
    /// Overflow discarded (the instance that breaks OG, CD-08).
    pub discarded: f64,
    /// The instance started the shield gate.
    pub gate_started: bool,
    /// The instance killed the target (HP ≤ 0).
    pub killed: bool,
}

/// ST-9 (05 E-5.25, CD-08): routes one instance at time `t` through OG → SH → HP, with the Sentient per-instance cap
/// and the adaptation gates (05 E-5.23) and the health-threshold flags. `t` is converted to the µs grid once
/// (08 E-8.0a) for the shield-gate test and `gate_until`. T07.
pub fn route_layered(ctx: &StageCtx, dmg: &LayeredHit, pools: &mut Pools, t: f64) -> RouteOut {
    let _ = (ctx, dmg, pools, t);
    todo!("T07")
}
