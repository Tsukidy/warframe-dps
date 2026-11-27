//! E-12's enemy side per layer (φ, Viral, Magnetic, V, armor and its floor of 1, non-armor DR R_ℓ, attenuation CD-09)
//! and ST-9 routing for the `layered` view (OG → SH → HP, Toxin routing and exceptions, overflow and the shield gate
//! CD-08; 05 E-5.14-E-5.25). Owner: T07.

use wfd_model::{DamageType, DamageVec, Layer, N_ST};

use crate::StageCtx;
use crate::enemy::scaling::ScaledEnemy;

/// The class of a damage instance for the enemy side (contract review ENG-9): the classes of 05 E-5.20, the
/// tick rules of E-5.24 and the multishot term of E-5.21. DoT ticks, Blast stack hits and extra hits take the weapon
/// class and radial flag of the instance that induced them (05 E-5.20 inheritance, OQ-535).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InstanceClass {
    /// Radial (classes `radial` and `radial_or_melee`).
    pub radial: bool,
    /// The weapon class of the inducing instance: `Some(true)` = an instance of a melee weapon (`radial_or_melee`),
    /// `Some(false)` = `non_melee` (a primary, secondary or arch-gun weapon, or a ranged instance of a glaive or
    /// gunblade), `None` = a special instance without an inducing weapon instance (a Magnetic break proc: no
    /// `dr_class`).
    pub melee: Option<bool>,
    /// A DoT tick (05 E-5.24: Cinematic SLA ticks use φ = 1 and arm = 1 on SH and HP; Toxin ticks are 0 while SH > 0
    /// on a `toxin_blocked = nullify` target).
    pub tick: bool,
    /// MS_att of 05 E-5.21: the modded multishot (E-10) for non-beam direct and radial instances; 1 for merged beam
    /// ticks, DoT ticks, extra hits and special instances.
    pub ms_att: f64,
}

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
    /// The instance's class (05 E-5.20, E-5.21, E-5.24). Addition, contract review ENG-9.
    pub class: InstanceClass,
    /// `adapt_res[t]` of 05 E-5.23 (layered passes only; 0 in the views, 05 §15.4), entering φ' on SH and HP.
    /// Addition, contract review ENG-9.
    pub adapt_res: DamageVec,
    /// The live shield is up (SH > 0; layered passes) for the Toxin-nullify rule of 05 E-5.24; false in the views.
    /// Addition, contract review ENG-9.
    pub shield_up: bool,
}

/// E-12's enemy side of one instance on one layer: Σ_t D_t for the weapon-side vector `w`, after the floor of 1 and
/// R_ℓ, and attenuation where the enemy has it (never on OG). The weapon-side overguard factor M_OG is a scalar the
/// caller applies on the OG layer. Exactly the sum of [`instance_on_layer_types`]. T07.
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

/// 05 E-5.24 per type: D^ℓ_t of one instance on one layer (R_ℓ with the class DR of E-5.20 and the Guardian aura of
/// E-5.27, attenuation E-5.21 attributed to the types in proportion, M_dps E-5.22, the tick rules, `adapt_res`), for
/// the per-type breakdown of 08 §8.2 and for `LayeredHit.hp_by_type`. Addition, contract review ENG-9. T07.
pub fn instance_on_layer_types(
    ctx: &StageCtx,
    w: &DamageVec,
    layer: Layer,
    st: &TargetStateView,
    en: &ScaledEnemy,
) -> DamageVec {
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
    /// g(instance) of 05 E-5.25: 1 for a weak-point hit, a DoT tick, an extra hit or a special instance (never
    /// gated), 0 for a slam or heavy-slam radial instance, 0.05 for every other instance. Replaces the flags
    /// `weak_point` and `tick`, which could not mark extra hits, special instances or slam radials (contract review
    /// ENG-9).
    pub gate_factor: f64,
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
    /// The instance broke the overguard (05 E-5.25 "emit OG broken": a Magnetic break proc, 03 §13; also when OG
    /// lands exactly on 0 with nothing discarded). Addition, contract review ENG-9.
    pub og_broken: bool,
    /// The instance broke the shield (05 E-5.25 "emit SH broken", Z ≥ SH: a Magnetic break proc, 03 §13). Addition,
    /// contract review ENG-9.
    pub sh_broken: bool,
}

/// ST-9 (05 E-5.25, CD-08): routes one instance at time `t` through OG → SH → HP on the enemy `en` (HP_max for the
/// thresholds and the adaptation gates, its adaptation kind and the Sentient per-instance cap of 05 E-5.23 under
/// T-28), updating the health-threshold flags of `pools` for the thresholds `thresholds` (the `StateVar::HealthBelow`
/// fractions of the scenario, in table order). `gate` = true applies the shield gate g' of E-5.25 (MC); false is the
/// EV TTK path of 05 §17 and 08 E-8.21 (g = g' = 1, fractional overflow). `t` is converted to the µs grid once
/// (08 E-8.0a) for the shield-gate test and `gate_until`. The parameters `en`, `thresholds` and `gate` are additions
/// (contract review ENG-9). T07.
pub fn route_layered(
    ctx: &StageCtx,
    dmg: &LayeredHit,
    pools: &mut Pools,
    t: f64,
    en: &ScaledEnemy,
    thresholds: &[f64],
    gate: bool,
) -> RouteOut {
    let _ = (ctx, dmg, pools, t, en, thresholds, gate);
    todo!("T07")
}
