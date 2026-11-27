//! L0: the compiled scenario (engine.md §3): enemy scaling, faction relations, caps, the T-02 mode and overrides, slot
//! allocation (`mech.<n>`, `mult.<n>`), buffs, custom stats, weapon and attack passives, the melee tables and plans,
//! the monotone slot classes and the mechanic ids of the scenario. Built by `Engine::compile_scenario`. Owner: T06.

use smallvec::SmallVec;
use wfd_model::{
    CompatFlags, CondOverride, ConditionMode, EffectKey, FactionId, MechanicKind, Monotone,
    MultGroupId, N_ST, N_STATS, Toggles,
};

use crate::assemble::AssembledWeapon;
use crate::compile::melee::CompiledMelee;
use crate::compile::monotone::Reader;
use crate::enemy::ScaledEnemy;
use crate::ev::ObjectiveSpec;
use crate::mc::keys::MechIdTable;
use crate::state::StateVarTable;
use crate::stats::SparseContrib;

/// A scenario compiled for one weapon (engine.md §3).
#[derive(Debug, Clone)]
pub struct CompiledScenario {
    /// Kitgun/zaw parts applied, progenitor V_t, exalted Strength factor (01 E-1, E-1.1).
    pub weapon: AssembledWeapon,
    /// Pools per layer, A0, φ_t, caps, immunities, parts, DR, attenuation (05).
    pub enemy: ScaledEnemy,
    /// Target faction with its bane relations resolved (01 §4.15).
    pub faction: FactionId,
    /// The scenario's toggles (00 §6.1).
    pub toggles: Toggles,
    /// T-02 mode.
    pub mode: ConditionMode,
    /// Per-effect condition overrides (domain-model.md §9.4).
    pub overrides: CompiledOverrides,
    /// E-7.0b combo tables for the equipped stance, the [`crate::MeleePlan`] per melee attack, specials (07).
    pub melee: Option<CompiledMelee>,
    /// Compatibility flags of the profile (empty in the game profile, engine.md §10).
    pub profile: CompatFlags,
    /// Buffs, custom stats, weapon and attack passives, zoom, toggles that are stats.
    pub fixed: SmallVec<[SparseContrib; 2]>,
    /// Which state variables exist in this scenario.
    pub state_vars: StateVarTable,
    /// Per-scenario monotonicity: declared class downgraded by readers (`compile::monotone`, domain-model.md §10).
    pub slot_class: [Monotone; N_STATS],
    /// Readers held by candidates: the optimizer's coupled candidates (optimizer.md §3).
    pub readers: SmallVec<[Reader; 4]>,
    /// Every MC mechanic id of the scenario, FNV-1a hashed, checked distinct (engine.md §8.2).
    pub mech_ids: MechIdTable,
    /// Allocation of the `mech.<n>` slots.
    pub mech_slots: MechSlotTable,
    /// Allocation of the `mult.<n>` slots from `Rules.mult_groups` (domain-model.md §14).
    pub mult_slots: MultSlotTable,
    /// Attacks and weights for the optimizer.
    pub objective: Option<ObjectiveSpec>,
    /// t_win (T-01, s).
    pub window: f64,
    /// Schedule horizon max(t_win, t_ttk_max) (s; 07 E-7.30, 08 E-8.21).
    pub horizon: f64,
    /// T-41 pellet hit fraction f_a per attack, indexed by `AttackIdx` (1 for an attack without an entry; 06 E-6.24,
    /// 08 E-8.2, E-8.17). Addition to engine.md §3 (CONTRACTS.md T02a).
    pub hit_fraction: SmallVec<[f64; 8]>,
    /// T-43 external stacks N_ext,s per status, T-06 folded in (`Toggles::external_stacks`), 0 for a type the
    /// target is immune to (03 §6.3, 08 `n_ext`). Addition to engine.md §3 (CONTRACTS.md T02a).
    pub external: [u8; N_ST],
}

/// Condition overrides of the scenario (domain-model.md §9.4), in canonical (key) order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompiledOverrides {
    /// (effect, override).
    pub entries: Vec<(EffectKey, CondOverride)>,
}

/// Allocation of the `mech.<n>` magnitude slots: one per distinct mechanic signature of the candidate pool
/// (domain-model.md §10: `(mechanic kind, event, status, exclusive group)`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MechSlotTable {
    /// Slot n's signature.
    pub signatures: SmallVec<[MechSignature; 16]>,
}

/// The signature that shares one `mech.<n>` slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MechSignature {
    /// The mechanic kind.
    pub kind: MechanicKind,
    /// Canonical text of (event, status, exclusive group), e.g. `"on_crit/SLA/hm"`.
    pub key: Box<str>,
}

/// Allocation of the `mult.<n>` slots: slot n holds the members of one multiplier group (domain-model.md §14).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MultSlotTable {
    /// Slot n's group.
    pub groups: Vec<MultGroupId>,
}
