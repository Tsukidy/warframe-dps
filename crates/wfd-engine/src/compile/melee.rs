//! Melee compile step (07 E-7.0b, §8.1, E-7.31; domain-model.md §9.3): the combo tables of the equipped stance
//! resolved once per scenario, and the [`MeleePlan`] of every melee attack from the T-30 toggles
//! (`Toggles.melee_plans`, `Toggles.melee_plan`) and the T-09 rule. `melee_schedule` (`schedule/melee.rs`) takes the
//! plan. Owner: T08b.

use smallvec::SmallVec;
use wfd_model::{
    Attack, AttackIdx, AttackKey, Bundle, ComboTableId, ComboType, HeavyPolicy, MeleeData,
    MeleePlanSel, MeleeTables, N_COMBO_TYPES, StanceId, ThrowKind, Toggles, Weapon,
};

/// The plan kinds of 07 §8.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanKind {
    /// `light_loop(K)`: combo K repeated.
    LightLoop,
    /// `slide_loop`: the shared Slide combo repeated.
    SlideLoop,
    /// `heavy_loop`: heavies back to back, alternating h = 1, 2.
    HeavyLoop,
    /// `build_spend(M_trig)`: light loop until M(c̄) ≥ M_trig, then one heavy.
    BuildSpend,
    /// `tennokai`: n_tk qualifying inputs, then one Tennokai heavy (E-7.17).
    Tennokai,
    /// `slam_loop`: one slam every T_slam.
    SlamLoop,
    /// `heavy_slam_loop`: one heavy slam every max(T_slam, regeneration wait).
    HeavySlamLoop,
    /// `glaive_cycle`: throw, hit, heavy detonation, catch (07 §9.4).
    GlaiveCycle,
    /// `cycle_loop`: one 100 % strike every T_cyc (weapons without a light combo).
    CycleLoop,
    /// `data_attack_loop`: one instance of a directly selected data attack every T_cyc.
    DataAttackLoop,
}

/// The plan of one melee attack with its parameters resolved (07 §8.1, E-7.31). Fields a kind does not use keep
/// their defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeleePlan {
    /// The plan kind.
    pub kind: PlanKind,
    /// The combo K of light phases (`BuildDoc.stance_combo`, default Neutral; Slide for `slide_loop`).
    pub combo: ComboType,
    /// `build_spend` M_trig (default M_cap).
    pub k: f64,
    /// `heavy_loop` policy (`full_regen` default).
    pub policy: HeavyPolicy,
    /// `heavy_loop` `heavy_cancel` t_dmg (s); `None` = off (07 E-7.30, OQ-701).
    pub heavy_cancel: Option<f64>,
    /// `slam_loop` / `heavy_slam_loop` T_slam (s, default 1).
    pub t_slam: f64,
    /// `glaive_cycle` throw kind (charged default).
    pub throw: ThrowKind,
    /// `cycle_loop` / `data_attack_loop` user cycle time T_cyc (s); `None` = the 07 §8.1 default.
    pub t_cyc: Option<f64>,
    /// Start count c(0) of E-7.31 (IC, or max(IC, k_tier·(M_start − 1)) under T-09).
    pub start_count: f64,
}

/// One combo of a table in the bundle: `Bundle.combo_tables[table].combos[index]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComboRef {
    /// The table.
    pub table: ComboTableId,
    /// Index of the combo within the table.
    pub index: u8,
}

/// The E-7.0b result held by a compiled scenario (owned indices instead of the borrowed `MeleeTables`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MeleeTablesOwned {
    /// The combo supplying each `ComboType`, indexed by `ComboType::index`.
    pub combos: [Option<ComboRef>; N_COMBO_TYPES],
}

impl MeleeTablesOwned {
    /// The borrowed form over the bundle; the heavy and the specials come from the (assembled) weapon's melee data.
    /// T08b.
    pub fn tables<'b>(&self, b: &'b Bundle, md: &'b MeleeData) -> MeleeTables<'b> {
        let _ = (b, md);
        todo!("T08b")
    }
}

/// Melee part of a compiled scenario (engine.md §5).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompiledMelee {
    /// E-7.0b result for the equipped stance.
    pub tables: MeleeTablesOwned,
    /// The plan of every melee attack the scenario evaluates.
    pub plans: SmallVec<[(AttackIdx, MeleePlan); 4]>,
}

/// 07 E-7.0b: the table that supplies each combo type for weapon `w` with stance `stance` (fixed stance of exalted
/// melee, UniqueToWeapon tables, the stanceless table of the class, `FromWeapon` combos). T08b.
pub fn resolve_combo_tables<'b>(
    b: &'b Bundle,
    w: &Weapon,
    stance: Option<StanceId>,
) -> MeleeTables<'b> {
    let _ = (b, w, stance);
    todo!("T08b")
}

/// T-30 selection for one melee attack (domain-model.md §9.3): the attack's `toggles.melee_plans` entry, else
/// `toggles.melee_plan` when that plan's kind names the attack (07 §8.1: `light_loop`, `build_spend`, `tennokai` the
/// Normal Attack, `heavy_loop` the heavy attack, `slide_loop` the slide attack, …), else `Auto`. Entries for unknown
/// attacks and duplicates were rejected when the document was resolved (`doc.attack_unknown`,
/// `doc.attack_duplicate`, wfd-data). T08b.
pub fn plan_selection(toggles: &Toggles, key: &AttackKey, atk: &Attack) -> MeleePlanSel {
    let _ = (toggles, key, atk);
    todo!("T08b")
}

/// 07 §8.1 and E-7.31: the [`MeleePlan`] of attack `atk` from its selection `sel` ([`plan_selection`]): `Auto` maps
/// by attack kind (Normal Attack → `light_loop(combo)`, heavy → `heavy_loop`, slide → `slide_loop`, slam →
/// `slam_loop`, heavy slam → `heavy_slam_loop`, throws → `glaive_cycle`, no light combo → `cycle_loop`, a directly
/// selected data attack → `data_attack_loop`); T-09 (`toggles.heavy`) moves attacks without a data role to
/// `heavy_loop` and sets the start count; T-10 is read by the schedule. T08b.
pub fn melee_plan(
    sel: &MeleePlanSel,
    atk: &Attack,
    toggles: &Toggles,
    combo: ComboType,
    md: &MeleeData,
) -> MeleePlan {
    let _ = (sel, atk, toggles, combo, md);
    todo!("T08b")
}
