//! Weapons, attacks, stances, modular parts (domain-model.md §4).
//!
//! Kitguns and zaws are assembled at evaluation time from parts (`wfd-engine::assemble`, 06 §11.1, 07 E-7.42); the
//! pre-expanded wiki kitgun records are used only as a pipeline cross-check.

use crate::effect::{CompressionMode, EffectDef};
use crate::enums::{
    AttackRoles, CategorySet, ComboType, DamageType, DamageVec, HeadshotRule, MeleeClass,
    N_COMBO_TYPES, Polarity, RivenClass, ShotType, StatusType, StrikeType, Trigger, WeaponClass,
    WeaponSlot, WeaponTraits,
};
use crate::ids::{
    AttackIdx, AttackKey, ComboKey, ComboTableId, FamilyId, ItemId, NameKey, StableKey, StanceId,
    TagId, UniqueName, WeaponId,
};
use crate::item::IncarnonDef;
use crate::serde_util::{dual_serde, flags_serde};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;

/// A weapon record of the bundle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Weapon {
    /// Dense id (bundle order).
    pub id: WeaponId,
    /// Stable key used by documents and share codes.
    pub key: StableKey,
    /// PE uniqueName, the join key.
    pub unique_name: UniqueName,
    /// Name lookup key (`Bundle.names`).
    pub name: NameKey,
    /// Equipment slot.
    pub slot: WeaponSlot,
    /// Weapon class (09 §3.1).
    pub class: WeaponClass,
    /// Category bits of `Rules.categories`.
    pub categories: CategorySet,
    /// PROJECTILE, AOE, BEAM, BATTERY, augment and stance-class tags.
    #[cfg_attr(feature = "ts", ts(as = "Vec<TagId>"))]
    pub compat_tags: SmallVec<[TagId; 4]>,
    /// Trait flags (09 E-9.1).
    pub traits: WeaponTraits,
    /// Incarnon line / weapon family.
    pub family: Option<FamilyId>,
    /// Riven class (09 E-9.50), computed by the pipeline; `None` = cannot take rivens (addition, CONTRACTS.md T01a).
    pub riven_class: Option<RivenClass>,
    /// Normalised riven family (09 E-9.51: X-FAMILY, else wiki `Family`, else name; addition, CONTRACTS.md T01a).
    pub riven_family: Option<Box<str>>,
    /// Weapon-level trigger (attacks may override, 06 §4.2).
    pub trigger: Trigger,
    /// `None` = no magazine (melee).
    pub magazine: Option<MagazineSpec>,
    /// Reload model (06 §5.7).
    pub reload: ReloadSpec,
    /// Attacks in wiki order, then synthesized heavy and slide attacks.
    #[cfg_attr(feature = "ts", ts(as = "Vec<Attack>"))]
    pub attacks: SmallVec<[Attack; 3]>,
    /// "Combine attack effects" groups (T-22, CD-12), parent+child sums and volley groups (06 E-6.14b).
    #[cfg_attr(feature = "ts", ts(as = "Vec<AttackGroup>"))]
    pub groups: SmallVec<[AttackGroup; 1]>,
    /// Zoom levels; effects gated by `while_aiming`, level chosen by `toggles.zoom_level` (04 D-406).
    #[cfg_attr(feature = "ts", ts(as = "Vec<ZoomLevel>"))]
    pub zoom: SmallVec<[ZoomLevel; 3]>,
    /// Melee data (07); `None` for guns.
    pub melee: Option<MeleeData>,
    /// Incarnon line of the weapon (09 E-9.60).
    pub incarnon: Option<IncarnonDef>,
    /// Kitgun chamber / zaw strike: needs parts in the build.
    pub modular: Option<ModularRef>,
    /// Allowed elements and value range [0.25, 0.60] (T-24).
    pub progenitor: Option<ProgenitorSpec>,
    /// Exalted Strength model and per-rank values (01 E-1.1, Table 4.3-A; T-31); `None` = not an exalted weapon
    /// (m_str = 1).
    pub exalted: Option<ExaltedScaling>,
    /// Innate effects in the DSL (Cedo CO, Sagek, IPS amplifiers …).
    pub passives: Vec<EffectDef>,
    /// Innate polarities of the eight main slots.
    #[cfg_attr(feature = "ts", ts(as = "Vec<Polarity>"))]
    pub polarities: SmallVec<[Polarity; 8]>,
    /// Innate exilus polarity.
    pub exilus_polarity: Option<Polarity>,
    /// Innate stance polarity (melee).
    pub stance_polarity: Option<Polarity>,
    /// Maximum weapon rank (30 or 40).
    pub max_rank: u8,
    /// Mastery rank requirement.
    pub mastery: u8,
    /// Release update or date as given by the source.
    pub release: Option<Box<str>>,
    /// PE `textureLocation` (pipeline.md §3.2); served by key through `/api/img/{key}`.
    pub image: Option<Box<str>>,
    /// Accuracy, noise, ammo max … (weapon panel only).
    pub display: WeaponDisplay,
    /// Source and revision per field group; overridden fields listed.
    pub provenance: Provenance,
}

/// Magazine of a weapon.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MagazineSpec {
    /// Magazine size Mag0 (rounds).
    pub size: f64,
    /// Reserve ammo; `f64::INFINITY` on robotic weapons (06 §11.4, `AmmoMax = ∞`), written `"Infinity"` in the readable
    /// form ([`crate::serde_util::ext_f64`]).
    #[serde(default, with = "crate::serde_util::ext_f64::option")]
    #[cfg_attr(
        feature = "ts",
        ts(type = "number | \"Infinity\" | \"-Infinity\" | null")
    )]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::serde_util::ExtF64>")
    )]
    pub ammo_max: Option<f64>,
    /// No ammo consumption (06 §5.6).
    pub infinite: bool,
    /// Battery recharge instead of a reload (06 §5.7).
    pub battery: Option<Battery>,
}

/// Battery recharge (06 §5.7): delay after the last shot, delay when empty, rounds per second.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Battery {
    /// Recharge delay after the last shot δ_rech (s, `ReloadDelay`).
    pub delay: f64,
    /// Recharge delay when empty (s, `ReloadDelayEmpty`).
    pub delay_empty: f64,
    /// Recharge rate ρ_rech (rounds/s, `ReloadRate`).
    pub rate: f64,
}

/// Reload model (06 §5.7).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ReloadSpec {
    /// Whole-magazine reload.
    Standard {
        /// Reload time R0 (s).
        time: f64,
    },
    /// Shell-by-shell reload (hand-authored split).
    PerShell {
        /// Start and end time t_se (s).
        start_end: f64,
        /// Time per shell t_sh (s).
        per_shell: f64,
    },
    /// The magazine regenerates (`ReloadStyle = regenerate`).
    Regenerate,
    /// Epitaph-style no-reload weapons.
    None,
}

/// One attack of a weapon (wiki `Attacks[]`, plus synthesized heavy and slide attacks).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Attack {
    /// Stable key of the attack within the weapon.
    pub key: AttackKey,
    /// Name lookup key.
    pub name: NameKey,
    /// Role flags (normal, charged, heavy, slam, radial …) used by scopes.
    pub roles: AttackRoles,
    /// Shot type (hitscan, projectile, AoE, thrown, DoT).
    pub shot: ShotType,
    /// Radial child of a parent hit (06 §9).
    pub parent: Option<AttackIdx>,
    /// B0_t per pellet.
    pub base: DamageVec,
    /// Base critical chance CC0 (fraction).
    pub cc: f64,
    /// Base critical multiplier CDM0 (factor).
    pub cd: f64,
    /// Base status chance SC0 (fraction).
    pub sc: f64,
    /// Base multishot MS0 (pellets per shot, default 1).
    pub multishot: f64,
    /// Firing model (06 §4-§7).
    pub fire: FireSpec,
    /// Ammo cost per shot cost0 (rounds, default 1; 06 §5.6).
    pub ammo_cost: f64,
    /// Incarnon form charges (06 §6.6).
    pub magazine_override: Option<f64>,
    /// Attack-level `Reload` (18 attacks; speargun throw retrieval time, 06 §6.4; addition, CONTRACTS.md T01a).
    pub reload_override: Option<f64>,
    /// 06 §8.
    pub falloff: Option<Falloff>,
    /// AoE radius (m).
    pub radius: Option<f64>,
    /// Innate punch-through (m; display and multi-target only).
    pub punch_through: f64,
    /// Projectile speed (m/s, `ShotSpeed`; flight time, 06 §8.5).
    pub projectile_speed: Option<f64>,
    /// Hitscan or beam range limit (06 §8.4).
    pub range: Option<f64>,
    /// Forced procs per pellet (02 §4.7.2).
    #[cfg_attr(feature = "ts", ts(as = "Vec<StatusType>"))]
    pub forced: SmallVec<[StatusType; 2]>,
    /// Conditional or corrected forced procs (02 §4.10 `[[attack.forced]]`, E-2.14); addition, CONTRACTS.md T01d.
    #[serde(default)]
    pub forced_cond: Vec<ForcedProcRule>,
    /// Headshot and weak-point capability of the attack.
    pub headshot: HeadshotRule,
    /// Unique headshot factor M_hsu (02 §4.10 `hs_unique`, E-2.10; Cernos Prime 1.5); `None` = 1 (a positive data
    /// `ExtraHeadshotDmg` is then a ΣHS entry, OQ-221). Addition, CONTRACTS.md T01d.
    #[serde(default)]
    pub hs_unique: Option<f64>,
    /// 04 §2.4-2.14: ingested catalogue row or the default rule.
    pub co: CoSpec,
    /// 06 §7.2.
    pub sniper_combo: Option<SniperComboSpec>,
    /// Explicit heavy flag (T-09 auto mode).
    pub is_heavy: Option<bool>,
    /// Child delays (06 §9.3).
    pub explosion_delay: Option<f64>,
    /// Embed delay before an embedded child detonates (s, 06 §9.3).
    pub embed_delay: Option<f64>,
    /// Lingering child (06 §9.4, E-6.25).
    pub lingering: Option<Lingering>,
    /// Synthesized heavy or slide attack (07 §9.1, §9.3).
    pub synthesized: Option<Synthesized>,
    /// Primary Compression effectiveness eff_a (04 §3.1, the imported `W:Primary_Compression` table); `None` = the
    /// attack gets no Primary Compression bonus (listed "Doesn't Work", or not in the table).
    pub compression_eff: Option<f64>,
    /// Where Primary Compression's value `0.8·dpm·R_mod·eff_a` goes on this attack (04 §3.1: "Multiplies" → a `G_F`
    /// factor, "Adds" → A); read only when `compression_eff` is set. Default `multiplies` ("1.00 for most"). The
    /// enum is the DSL's [`CompressionMode`] (`Per::CompressionEff`), so the effect and the attack name the same
    /// bucket. Addition, CONTRACTS.md "Contract review fixes (model-records)", DC-8.
    #[serde(default)]
    pub compression_mode: CompressionMode,
    /// Per-attack model-1 table s_0 … s_3 of Table 4.3-A where the attack's ratios s_r / s_3 differ from the weapon's
    /// ([`ExaltedScaling::BaseXStr`]; Noctua's active attack 2000 / 2250 / 2500 / 2750 against 175 / 200 / 225 / 250);
    /// `None` = the weapon's table (01 E-1.1). Addition, CONTRACTS.md "Contract review fixes (model-records)", M-1.
    #[serde(default)]
    pub exalted_s: Option<[f64; 4]>,
    /// Attack-level innate effects.
    pub passives: Vec<EffectDef>,
}

/// Firing model of an attack (06 §4-§7).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FireSpec {
    /// Trigger class of the attack (06 §4.2).
    pub trigger: Trigger,
    /// Listed fire rate (attacks/s), or attack speed for melee; `f64::INFINITY` for `math.huge` (Lanka, 06 §4.1),
    /// written `"Infinity"` in the readable form ([`crate::serde_util::ext_f64`]).
    #[serde(with = "crate::serde_util::ext_f64")]
    #[cfg_attr(feature = "ts", ts(type = "number | \"Infinity\" | \"-Infinity\""))]
    #[cfg_attr(feature = "schema", schemars(with = "crate::serde_util::ExtF64"))]
    pub rate: f64,
    /// Charge time CT0 (s); `None` = 0.
    pub charge_time: Option<f64>,
    /// Charge time not affected by mods (Phantasma, 06 E-6.4; addition, CONTRACTS.md T01a).
    pub charge_fixed: bool,
    pub partial_charge: Option<PartialCharge>,
    /// `{ count, delay }`.
    pub burst: Option<Burst>,
    /// Reload lead after a magazine burst (`BurstReloadDelay`, 06 §5.8; addition).
    pub burst_reload_delay: Option<f64>,
    /// 06 §7.3, hand table.
    pub spool: Option<Spool>,
    /// 06 §7.1.
    pub held: Option<Held>,
    /// Override of the semi-auto cap (06 §5.2).
    pub semi_cap: Option<f64>,
    /// Kohm-family spooling pellets (06 E-6.18).
    pub kohm: Option<KohmSpec>,
    /// Peacemaker ramp (06 E-6.19).
    pub peacemaker: Option<PeacemakerSpec>,
    /// Kill-charged gauge instead of ammo (06 E-6.14a; addition).
    pub gauge: Option<GaugeSpec>,
}

/// Partial-charge release threshold of quick-shot weapons (Lanka 0.33, Opticor 0.40). Display only: the data models
/// quick shots as separate attacks with their own charge time (06 §6.4).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PartialCharge {
    /// Fraction of the charge time at which a quick shot releases.
    pub release_fraction: f64,
}

/// Burst: shots per burst (BC) and delay between them (BD0).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Burst {
    /// Shots per burst BC.
    pub count: u16,
    /// Delay between the shots of a burst BD0 (s).
    pub delay: f64,
}

/// Auto-spool cadence (06 E-6.17): `S_sp` (data `Spool`), start fraction `s_sp`, step `Δ_sp` (fallback
/// `(1 − s_sp)/(S_sp − 1)` when absent).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Spool {
    /// Spool-up shots S_sp (data `Spool`).
    pub shots: Option<u16>,
    /// Start fraction s_sp of the fire rate.
    pub start_frac: f64,
    /// Step Δ_sp per shot (fraction); `None` = `(1 − s_sp)/(S_sp − 1)`.
    pub step: Option<f64>,
}

/// Held (beam) attack: tick rate and damage ramp (06 E-6.15; ramp start s_rmp, default 0.2; ramp time 0.6 s).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Held {
    /// Damage ticks per second of the beam.
    pub tick_rate: f64,
    /// Ramp start level s_rmp (fraction, default 0.2).
    pub ramp_start: f64,
    /// Ramp time to full damage (s, 0.6).
    pub ramp_time: f64,
}

/// Kohm family spooling pellets (06 E-6.18): `P_max` (12 or 5) and the start fraction `FR0(single)/FR0(full)`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct KohmSpec {
    /// P_max, the pellet cap of the spool (12 or 5).
    pub max_pellets: u8,
    /// Start fraction FR0(single)/FR0(full).
    pub start_frac: f64,
}

/// Peacemaker (06 E-6.19): fire-rate ramp from `ramp_start` over `ramp_time` s, bursts of `burst` shots,
/// `step` damage per burst up to `max_bursts` bursts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PeacemakerSpec {
    /// Fire-rate ramp start (fraction of the full rate).
    pub ramp_start: f64,
    /// Fire-rate ramp duration (s).
    pub ramp_time: f64,
    /// Shots per burst (2; b_pm = floor(i / burst) + 1).
    pub burst: u8,
    /// Damage bonus added per burst (fraction).
    pub step: f64,
    /// Bursts after which the damage ramp stops.
    pub max_bursts: u8,
}

/// Gauge-gated attack (06 E-6.14a): charged by kills, one use from the fresh state (Battacor: 3 charges).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GaugeSpec {
    /// Charges of a full gauge (Battacor 3); the fresh state holds one full gauge.
    pub charges: u8,
}

/// Condition-Overload data of an attack (04 §2.4-2.6, catalogue §2.14).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CoSpec {
    /// E-11 / E-4.3.
    pub behaviour: CoBehaviour,
    /// ρ_co.
    pub ratio: f64,
    /// 04 §2.6.
    pub co_ref: CoRef,
    /// B_co^abs (`co_ref = fixed`).
    pub fixed_abs: Option<f64>,
    pub flags: CoFlags,
    /// κ_ms (Tenet Detron 0.65; proposed amendment OQ-428).
    pub ms_proj_factor: Option<f64>,
    /// CO is 0 while one of these augments is equipped.
    #[cfg_attr(feature = "ts", ts(as = "Vec<ItemId>"))]
    pub off_with: SmallVec<[ItemId; 1]>,
    /// 04 §2.14 row id; `None` = default rule (04 §2.5).
    pub row: Option<u16>,
    /// Weapon-damage bucket rule of a melee projectile (04 §3.4 `mproj_mode`; addition, CONTRACTS.md T01a).
    pub mproj: MprojMode,
}

impl Default for CoSpec {
    /// The D6 default of 04 §2.5: adding, ρ_co = 1, untested.
    fn default() -> Self {
        CoSpec {
            behaviour: CoBehaviour::Adding,
            ratio: 1.0,
            co_ref: CoRef::SelfAttack,
            fixed_abs: None,
            flags: CoFlags::UNTESTED,
            ms_proj_factor: None,
            off_with: SmallVec::new(),
            row: None,
            mproj: MprojMode::Standard,
        }
    }
}

/// CO behaviour (E-11).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CoBehaviour {
    Adding,
    Multiplying,
    None,
}

/// CO base reference (04 §2.6). `Uncharged` and `Fixed` are the catalogue's values (CONTRACTS.md, T01a).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CoRef {
    #[serde(rename = "self")]
    SelfAttack,
    Uncharged,
    Parent,
    Fixed,
}

bitflags::bitflags! {
    /// Catalogue flags of 04 §2.14. Readable form: lowercase names.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct CoFlags: u16 {
        /// Radial: CO only on the directly hit target.
        const DIRECT_TARGET_ONLY = 1 << 0;
        /// `co_omit = {combo}`.
        const NO_COMBO = 1 << 1;
        /// `co_dot_mode = exclude`.
        const NO_DOT_CO = 1 << 2;
        /// `co_dot_mode = per_tick`.
        const DOT_RECALC_PER_TICK = 1 << 3;
        /// Documentation of E-4.3.
        const FLAT_EXCLUDED = 1 << 4;
        /// Documentation of E-4.3.
        const FLAT_INCLUDED = 1 << 5;
        /// `co_tested = false` (D6, D5a): the UI flags the attack.
        const UNTESTED = 1 << 6;
        /// The catalogue row has no data attack.
        const NO_DATA_ATTACK = 1 << 7;
    }
}
flags_serde!(CoFlags, u16, Lower);

/// Weapon-damage bucket rule of a melee projectile attack (04 §3.4).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MprojMode {
    #[default]
    Standard,
    TwoBucket,
    SingleBucket,
    UniversalOnly,
}

/// Sniper combo data (06 E-6.16): `C_min` (`SniperComboMin`; `None` for reset-only weapons) and `D_snp`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SniperComboSpec {
    /// C_min (`SniperComboMin`, hits); `None` = no damage multiplier.
    pub min: Option<f64>,
    /// D_snp (`SniperComboReset`, s).
    pub reset: f64,
}

/// Falloff (06 E-6.20): `{ start d_s, end d_e, min_factor 1 − r_f }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Falloff {
    /// Falloff start distance d_s (m).
    pub start: f64,
    /// Falloff end distance d_e (m).
    pub end: f64,
    /// Damage factor at and beyond d_e, 1 − r_f.
    pub min_factor: f64,
}

/// Lingering child (06 E-6.25): effect duration D_e and tick rate FR_child.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Lingering {
    /// Effect duration D_e (s).
    pub effect_duration: f64,
    /// Ticks per second FR_child.
    pub tick_rate: f64,
}

/// Synthesized attack (07 §9.1, §9.3; the exalted waves of 07 E-7.43 are additions, CONTRACTS.md T01d).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Synthesized {
    HeavyFrom {
        normal: AttackIdx,
    },
    SlideFrom {
        normal: AttackIdx,
    },
    /// Exalted light and slide waves (attack id [`LIGHT_WAVES`], 07 E-7.43): the stance's Ranged entries, per-type
    /// vector of the Normal Attack.
    LightWavesFrom {
        normal: AttackIdx,
    },
    /// Exalted Blade heavy waves (attack id [`HEAVY_WAVES`], 07 E-7.43): two per heavy, base of the heavy.
    HeavyWavesFrom {
        heavy: AttackIdx,
    },
}

/// Attack id of synthesized light and slide waves (07 E-7.43; the 04 §2.14 join key of rows 188-189).
pub const LIGHT_WAVES: &str = "Light Waves";
/// Attack id of synthesized heavy waves (07 E-7.43).
pub const HEAVY_WAVES: &str = "Heavy Waves";

/// A conditional or corrected forced proc of an attack (02 §4.10 `[[attack.forced]]`, E-2.14). JSON
/// `{ "condition": "direct_hit", "procs": [{ "type": "SLA", "count": 1 }], "replaces_data": false }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ForcedProcRule {
    /// When the procs are delivered.
    pub condition: ForcedCondition,
    /// The procs per instance.
    #[cfg_attr(feature = "ts", ts(as = "Vec<ForcedCount>"))]
    pub procs: SmallVec<[ForcedCount; 1]>,
    /// The data's unconditional forced proc of the same type is removed (correction rows: Evensong).
    #[serde(default)]
    pub replaces_data: bool,
    /// Special rule of 02 §4.7.2 (`cedo`, `evensong`, `microwave`).
    #[serde(default)]
    pub rule: Option<ForcedRule>,
}

/// Condition of a [`ForcedProcRule`] (02 §4.10). JSON `"always"`, `{ "distance_max": 10.0 }`, `"direct_hit"` …
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ForcedCondition {
    /// Every instance of the attack (an unconditional hand-authored forced proc, 02 §4.10).
    Always,
    /// Target distance (T-27) at most this many metres.
    DistanceMax(f64),
    /// Non-radial instances only.
    DirectHit,
    /// Full charge (06 charge state).
    FullCharge,
    /// The last shot of the magazine ("upon depleting mag", 06 last-shot flag).
    LastShot,
    /// While aiming (T-38; T-02 class b).
    Zoom,
}

/// A forced proc count (02 §4.10 `{ type, count }`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ForcedCount {
    /// The status type.
    #[serde(rename = "type")]
    pub status: StatusType,
    /// Procs per instance.
    pub count: u8,
}

/// Special forced-proc rules of 02 §4.7.2.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ForcedRule {
    /// Cedo alt-fire explosion: two procs, the second uniform over HEA, COL, ELE, TOX (D-202).
    Cedo,
    /// Evensong: 7 Puncture instead of the data's 2.
    Evensong,
    /// Nukor and Kuva Nukor Microwave on every hit (OQ-215).
    Microwave,
}

/// An attack group: T-22 combinable group, parent + child sum, or volley group (06 E-6.14b; `volley` is an
/// addition, CONTRACTS.md T01a).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackGroup {
    #[cfg_attr(feature = "ts", ts(as = "Vec<AttackIdx>"))]
    pub members: SmallVec<[AttackIdx; 3]>,
    /// T-22.
    pub combinable: bool,
    pub summed: bool,
    #[serde(default)]
    pub volley: bool,
}

/// A zoom level and its effects.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ZoomLevel {
    pub name: NameKey,
    pub effects: Vec<EffectDef>,
    /// `ZOOM_ONLY` attacks that exist only while this level is the active one (T-38; 06 §9.1: Komorex "3.5x Zoom
    /// Radial Attack" exists only at the 3.5× level, EC-06-13, TV-06-32). A `ZOOM_ONLY` attack that no level lists
    /// exists at every zoom level (aiming on). Addition, CONTRACTS.md "Contract review fixes (model-records)", M-15.
    #[serde(default)]
    pub enables: Vec<AttackIdx>,
}

/// Weapon-panel values (accuracy, noise, flight speed …).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WeaponDisplay {
    pub values: BTreeMap<Box<str>, DisplayValue>,
}

/// A weapon-panel value: number or text (readable form untagged).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(as = "DisplayValueRepr"))]
pub enum DisplayValue {
    Num(f64),
    Text(Box<str>),
}

/// Readable form of [`DisplayValue`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DisplayValueRepr {
    Num(f64),
    Text(Box<str>),
}

impl From<&DisplayValue> for DisplayValueRepr {
    fn from(v: &DisplayValue) -> Self {
        match v {
            DisplayValue::Num(x) => DisplayValueRepr::Num(*x),
            DisplayValue::Text(t) => DisplayValueRepr::Text(t.clone()),
        }
    }
}

impl From<DisplayValueRepr> for DisplayValue {
    fn from(r: DisplayValueRepr) -> Self {
        match r {
            DisplayValueRepr::Num(x) => DisplayValue::Num(x),
            DisplayValueRepr::Text(t) => DisplayValue::Text(t),
        }
    }
}

dual_serde!(DisplayValue, DisplayValueRepr);

/// Where a record's fields come from: source and revision per field group, and the fields an override changed.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Provenance {
    pub sources: Vec<SourceRef>,
    pub overridden: Vec<Box<str>>,
}

/// Source of one field group (`"attacks"` from `"wiki:Module:Weapons/data/primary"` at revision `"123456"`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceRef {
    pub group: Box<str>,
    pub source: Box<str>,
    pub revision: Box<str>,
}

// ---------------------------------------------------------------------------------------------------------------
// Melee (spec 07)
// ---------------------------------------------------------------------------------------------------------------

/// Melee fields of a weapon (07 §4.3); everything per weapon, filled by the pipeline.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MeleeData {
    /// Wiki `Class`: selects the stanceless table and table 9-A.
    pub class: MeleeClass,
    /// Wiki `Family`: UniqueToWeapon matching (E-7.0b).
    pub family: Option<Box<str>>,
    /// FR_w after the k/60 snap (E-7.1).
    pub fr_base: f64,
    /// W0 (`WindUp`).
    pub wind_up: f64,
    /// D0_cb (`ComboDur`); `None` = no expiry (Xoris, 07 §6.4).
    pub combo_dur: Option<f64>,
    /// T_sw from data, else the E-7.5 fallback at evaluation.
    pub heavy_swing: Option<f64>,
    /// Display and multi-target only.
    pub follow_through: f64,
    /// Melee range (m; display and multi-target only).
    pub range: f64,
    /// The stanceless table of `class` (exalted classes: their stance table).
    pub stanceless: ComboTableId,
    /// UniqueToWeapon table matched by name or Family (E-7.0b).
    pub unique: Option<ComboTableId>,
    /// The fixed stance of exalted melee (09 E-9.2 `stance_fixed`; E-7.0b; addition, CONTRACTS.md T01a).
    pub fixed_stance: Option<StanceId>,
    /// Synthesized heavy attack (07 §9.1); `None` for classes without heavies.
    pub heavy: Option<HeavyDef>,
    /// Table 9-C rows of this weapon.
    #[cfg_attr(feature = "ts", ts(as = "Vec<MeleeTriggerRow>"))]
    pub triggers: SmallVec<[MeleeTriggerRow; 2]>,
    /// k_tier, M_cap, M_bcap, D0_cb override, innate IC (07 §6; D-063).
    pub specials: ComboSpecials,
    /// One of the four ability-combo exalted weapons (E-7.44, D-701).
    pub ability_combo: bool,
}

/// Synthesized heavy attack (E-7.40, E-7.41): hit lists computed by the pipeline, damage by the engine.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HeavyDef {
    /// The synthesized Attack (roles HEAVY, `synthesized = HeavyFrom { normal }`).
    pub attack: AttackIdx,
    /// Heavy h = 1, 2: H_j (κ_w applied), procs, spawned 9-C attacks.
    #[cfg_attr(feature = "ts", ts(as = "[Vec<HeavyStrike>; 2]"))]
    pub hits: [SmallVec<[HeavyStrike; 4]>; 2],
    /// κ_w (table 9-B, 1 unless listed).
    pub kappa: f64,
    /// Where the hit list comes from (07 E-7.40, E-7.41).
    pub source: HeavySource,
}

/// One strike of a heavy attack.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HeavyStrike {
    /// Damage multiplier H_j of the strike (κ_w applied).
    pub mult: f64,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<StatusType>"))]
    pub procs: SmallVec<[StatusType; 2]>,
    /// Crowd-control names kept on the hit record (07 §4.1; addition, CONTRACTS.md T01a).
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<CrowdControl>"))]
    pub cc: SmallVec<[CrowdControl; 1]>,
    #[serde(default)]
    pub spawns: Option<AttackIdx>,
}

/// Where a heavy hit list comes from (E-7.40).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HeavySource {
    UniqueOverride,
    ClassTable { sigma: [f64; 2] },
    Breakdown9A,
}

/// Crowd-control proc names of stance data that are not status types (07 §4.1, 03 table 4.2; `Finisher` and
/// `Disarmed` per OQ-715).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CrowdControl {
    Ragdoll,
    Knockback,
    Stagger,
    BigStagger,
    Stun,
    Finisher,
    Disarmed,
}

/// One normalised `Module:Stances/data` table (07 §4.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComboTable {
    pub id: ComboTableId,
    pub source: ComboSource,
    /// The table's `WeaponType` (class of FromWeapon combos taken from an exalted stance, E-7.0b; addition,
    /// CONTRACTS.md T01a).
    pub weapon_type: Option<MeleeClass>,
    /// At most one per ComboType.
    #[cfg_attr(feature = "ts", ts(as = "Vec<Combo>"))]
    pub combos: SmallVec<[Combo; 8]>,
}

/// Origin of a combo table.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ComboSource {
    Stance(StanceId),
    Stanceless(MeleeClass),
    Unique { name: Box<str> },
}

/// One combo of a table.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Combo {
    /// Combo type (07 E-7.0b).
    pub kind: ComboType,
    /// Combo key (`"neutral"`, a stance combo slug).
    pub key: ComboKey,
    /// Name lookup key.
    pub name: NameKey,
    /// D_C at AS 1.0; `None` → E-7.3 fallback.
    pub duration: Option<f64>,
    /// Copied by `addSharedCombos` (`FromWeapon`); skipped by E-7.0b rules 1-2.
    pub from_weapon: bool,
    /// Number of inputs of the combo.
    pub n_inputs: u8,
    /// E-7.0 step 6: entries expanded by `Hits[e]`, inputs concatenated.
    pub strikes: Vec<Strike>,
}

/// One strike of a combo.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Strike {
    /// Input index u and entry index e of the combo.
    pub input: u8,
    /// Entry index e within the input.
    pub entry: u8,
    /// `S_j = Dmg[e]/100`.
    pub mult: f64,
    /// `Types[e]`: Slam strikes are radial parts, Ranged strikes spawn 9-C attacks.
    pub ty: StrikeType,
    /// S^phys_t = factor − 1 for SLA, IMP, PUN (E-7.0 step 5), in that order.
    pub phys: [f64; 3],
    /// Forced procs (E-7.0 steps 2-4, 03 table 4.2).
    #[cfg_attr(feature = "ts", ts(as = "Vec<StatusType>"))]
    pub procs: SmallVec<[StatusType; 2]>,
    /// Crowd-control names kept on the hit record (07 §4.1; addition, CONTRACTS.md T01a).
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Vec<CrowdControl>"))]
    pub cc: SmallVec<[CrowdControl; 1]>,
    /// Stored, not applied (OQ-714).
    pub status_mult: f64,
}

/// A stance mod and its table.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Stance {
    pub id: StanceId,
    pub item: ItemId,
    pub class: MeleeClass,
    pub table: ComboTableId,
}

/// Resolved per scenario by the engine (E-7.0b): for each ComboType the table that supplies it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeleeTables<'b> {
    /// Indexed by [`ComboType::index`].
    pub combos: [Option<&'b Combo>; N_COMBO_TYPES],
    pub heavy: Option<&'b HeavyDef>,
    pub rules: &'b MeleeRules,
    pub specials: &'b ComboSpecials,
}

impl<'b> MeleeTables<'b> {
    /// The resolved combo of a combo type, `None` when the weapon has none of that type.
    pub fn combo(&self, kind: ComboType) -> Option<&'b Combo> {
        self.combos[kind.index()]
    }
}

/// The hand tables of 07 §4.4 (`data/curated/rules/melee/*.toml`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MeleeRules {
    /// Table 9-A.
    pub heavy_class: Vec<HeavyClassRow>,
    /// Table 9-B members (checked against the E-7.41 computation).
    pub heavy_kappa: Vec<(UniqueName, f64)>,
    /// Table 9-C.
    pub attack_triggers: Vec<MeleeTriggerRow>,
    /// Table 9-D: r_in per stance, combo and input (04 §2.12, 07 §7.6).
    pub stance_co: Vec<StanceCoRow>,
    /// Table 9-E.
    pub finishers: Vec<FinisherRow>,
    pub combo_specials: Vec<(UniqueName, ComboSpecials)>,
}

/// A row of table 9-A (07 §9.1): heavy totals T_{class,h}, explicit breakdowns, typical wind-up.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HeavyClassRow {
    pub class: MeleeClass,
    /// T_{class,1}, T_{class,2}. A class whose 9-A row is "–" (Bayonet) has no row: σ_h = 1 by E-7.40 rule 2, the
    /// same as for a "–" entry (the curated TOML cannot write `None`; CONTRACTS.md "Contract review fixes
    /// (model-records)", T03c-2).
    pub total: [Option<f64>; 2],
    /// Explicit per-hit breakdown of heavy 1 and 2 (empty = none given).
    #[serde(default)]
    pub breakdown: [Vec<HeavyStrike>; 2],
    /// Typical wind-up (s); the weapon's `WindUp` field wins.
    pub wind_up: Option<f64>,
    pub source: Box<str>,
}

/// A row of table 9-C (07 §9.7): which trigger spawns which data attack. In `MeleeRules` the row names its weapons;
/// in `MeleeData.triggers` it is the weapon's own row with `attack_idx` resolved.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MeleeTriggerRow {
    #[serde(default)]
    pub weapons: Vec<UniqueName>,
    pub trigger: MeleeTriggerSource,
    /// The data attack name (`AttackName` of the weapon record).
    pub attack: Box<str>,
    #[serde(default)]
    pub attack_idx: Option<AttackIdx>,
    pub count: TriggerCount,
    pub scaling: TriggerScaling,
    /// × M_h and spends with its heavy.
    pub heavy_combo: bool,
    /// Heavy-only A members and crit doubling.
    pub heavy_bonus: bool,
    pub source: Box<str>,
}

/// Trigger of a 9-C row.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum MeleeTriggerSource {
    /// A `Ranged` entry of a combo (input index when the combo has several Ranged entries).
    ComboRanged {
        combo: ComboType,
        input: Option<u8>,
    },
    /// Heavy h (None = both heavies).
    Heavy {
        h: Option<u8>,
    },
    Slide,
}

/// Count per trigger.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TriggerCount {
    /// The entry's `Hits`.
    Hits,
    /// The attack's `Multishot`.
    Multishot,
    Fixed(u8),
}

/// Scaling of a 9-C attack.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TriggerScaling {
    /// s_j = Dmg/100.
    Scaled,
    /// s_j = 1 (the stance Dmg% is a consistency check).
    AsData,
    /// Exalted waves (07 §9.8, E-7.43): light and slide waves (`combo_ranged` or `slide` trigger) are E-7.20 with
    /// the strike's S_j on the Normal Attack; heavy waves (`heavy` trigger) are E-7.20b with H_j
    /// (`B'_t = H_j·(Bw_t + V_t) + X_t`, `B_ref = H_j·B_ref`).
    Wave,
}

/// A row of table 9-D (07 §7.6): r_in per combo column; a cell holds one value for every input or one per input.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StanceCoRow {
    /// Stance name as on the wiki page (the pipeline maps it to the stance item).
    pub stance: Box<str>,
    pub class: MeleeClass,
    /// The class's first row, which carries the shared columns (E-7.21).
    #[serde(default)]
    pub first_of_class: bool,
    pub cells: BTreeMap<ComboType, Vec<f64>>,
    pub source: Box<str>,
}

/// A row of table 9-E (07 §9.11): finisher multipliers per kind as hit splits.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FinisherRow {
    pub class: MeleeClass,
    /// Weapons the row is for (empty = every weapon of `class`). Set on the rows of the exalted weapons that
    /// `W:Finisher` lists by name (Exalted Blade, Valkyr Talons; 07 §9.11), which share the class `ExaltedWeapon`
    /// (addition, CONTRACTS.md T03c).
    #[serde(default)]
    pub weapons: Vec<UniqueName>,
    pub kinds: BTreeMap<FinisherKind, Vec<f64>>,
    pub source: Box<str>,
}

/// Finisher kinds of table 9-E.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FinisherKind {
    Ground,
    Stealth,
    Back,
    Front,
    Universal,
}

/// Combo specials of a weapon (07 §6; D-063). `None` = the default of 07 E-7.10 (k_tier 20, M_cap 12, M_bcap 12),
/// the weapon's `ComboDur`, no innate Initial Combo.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComboSpecials {
    /// k_tier override: combo count per multiplier tier (07 E-7.10).
    #[serde(default)]
    pub k_tier: Option<u16>,
    /// M_cap override: cap of the combo multiplier (07 E-7.10).
    #[serde(default)]
    pub m_cap: Option<u8>,
    /// M_bcap override: cap of the multiplier read by combo-scaled bonuses (07 E-7.11).
    #[serde(default)]
    pub m_bcap: Option<u8>,
    /// D0_cb override (s).
    #[serde(default)]
    pub combo_dur: Option<f64>,
    /// Innate Initial Combo.
    #[serde(default)]
    pub initial_combo: Option<f64>,
}

// ---------------------------------------------------------------------------------------------------------------
// Modular weapons
// ---------------------------------------------------------------------------------------------------------------

/// Kitgun and zaw parts (`Module:Modular/data`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ModularTables {
    pub chambers: Vec<KitgunChamber>,
    pub grips: Vec<ModularPart>,
    pub loaders: Vec<ModularPart>,
    pub strikes: Vec<ZawStrike>,
    pub links: Vec<ModularPart>,
}

/// A kitgun chamber for one slot (06 E-6.30).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct KitgunChamber {
    pub key: StableKey,
    pub unique_name: UniqueName,
    pub name: NameKey,
    pub slot: WeaponSlot,
    /// The chamber's weapon record (trigger, ammo cost, multishot, shot type, falloff, forced procs).
    pub weapon: WeaponId,
    /// Per grip (index into `ModularTables.grips`): damage map, fire rate, charge time, range.
    pub by_grip: Vec<ChamberGrip>,
    /// Loader magazine tier name → Mag0.
    pub magazine_by_tier: BTreeMap<Box<str>, f64>,
    pub cc: f64,
    pub cd: f64,
    pub sc: f64,
}

/// Chamber stats for one grip (06 E-6.30).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChamberGrip {
    pub grip: u16,
    pub damage: DamageVec,
    pub fire_rate: f64,
    pub charge_time: Option<f64>,
    pub range: Option<f64>,
}

/// A kitgun grip or loader, or a zaw grip or link.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ModularPart {
    pub key: StableKey,
    pub unique_name: UniqueName,
    pub name: NameKey,
    pub kind: ModularPartKind,
    /// Kitgun grips: the slot they make the kitgun.
    pub slot: Option<WeaponSlot>,
    /// Zaw grips: one- or two-handed.
    pub hand: Option<ZawHand>,
    /// Zaw grip and link damage (E-7.42).
    pub damage: f64,
    /// Zaw speed offsets (E-7.42 FR_src).
    pub speed: f64,
    /// Additive crit chance, crit multiplier and status offsets (loaders, links).
    pub cc: f64,
    pub cd: f64,
    pub sc: f64,
    /// Loader reload time R0.
    pub reload: Option<f64>,
    /// Loader magazine tier name.
    pub magazine_tier: Option<Box<str>>,
    pub image: Option<Box<str>>,
}

/// Kind of a modular part.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ModularPartKind {
    KitgunGrip,
    KitgunLoader,
    ZawGrip,
    ZawLink,
}

/// Zaw grip hand type (E-7.42 τ).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ZawHand {
    OneHanded,
    TwoHanded,
}

/// A zaw strike (E-7.42).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ZawStrike {
    pub key: StableKey,
    pub unique_name: UniqueName,
    pub name: NameKey,
    /// Z_s.Damage_t.
    pub damage: DamageVec,
    /// Class by hand type: [one-handed, two-handed].
    pub class: [MeleeClass; 2],
    /// Multipliers by hand type: [one-handed, two-handed].
    pub multi: [ZawMulti; 2],
    pub cc: f64,
    pub cd: f64,
    pub sc: f64,
    pub speed: f64,
    /// Plague Keewar / Kripath split `strike_t + (total − base)/4` (E-7.42).
    pub plague_split: bool,
    pub image: Option<Box<str>>,
}

/// Zaw strike multipliers for one hand type (`Z_s.Multi[τ]`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ZawMulti {
    pub base: f64,
    pub leap: f64,
    pub slide: f64,
    pub wall: f64,
}

/// A modular weapon's base part: kitgun chamber or zaw strike index.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ModularRef {
    pub kind: ModularKind,
    /// Index into `ModularTables.chambers` or `.strikes`.
    pub base: u16,
}

/// Kitgun or zaw.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ModularKind {
    Kitgun,
    Zaw,
}

/// Progenitor (valence) data of an adversary weapon (T-24, 09 §14).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProgenitorSpec {
    #[cfg_attr(feature = "ts", ts(as = "Vec<DamageType>"))]
    pub elements: SmallVec<[DamageType; 7]>,
    pub min: f64,
    pub max: f64,
    pub default: f64,
}

/// Exalted Strength model of a weapon (01 §2 input `exalted_model`, E-1.1, Table 4.3-A) with its per-rank values.
/// Index r of every array is the exalted ability rank (T-31, 0 ..= 3, default 3); the data base B0 is the rank-3
/// value. The variants and their readable names are the five values of `exalted_model`. JSON
/// `{ "base_x_str": { "s": [100, 125, 200, 250] } }`, `{ "mult_in_A": { "m": [1, 1.08, 1.16, 1.25] } }`, `"none"` …
/// Replaces the T01a variants `BaseByStrength` (no per-rank values) and `Partial(f64)` (no spec model;
/// CONTRACTS.md "Contract review fixes (model-records)", M-1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ExaltedScaling {
    /// Model 1 (`base_x_str`): `m_str = Str·R_r`, `R_r = s_r / s_3`, on every attack of the weapon unless the attack
    /// carries its own table ([`Attack::exalted_s`], Noctua's active attack).
    BaseXStr {
        /// s_0 … s_3 of Table 4.3-A (only the ratios s_r / s_3 enter E-1.1).
        s: [f64; 4],
    },
    /// Model 2 (`mult_in_A`): `m_str = 1`, `A += m_r·Str − 1` (Neutralizer, Lizzie; the form is UNVERIFIED, OQ-109).
    #[serde(rename = "mult_in_A")]
    MultInA {
        /// m_0 … m_3 of Table 4.3-A.
        m: [f64; 4],
    },
    /// Model 3 (`peacemaker`): `m_str = 1`, `A += p_r·Str` (Regulators).
    Peacemaker {
        /// p_0 … p_3 of Table 4.3-A.
        p: [f64; 4],
    },
    /// `none`: `m_str = 1` (Garuda Talons, Table 4.3-A).
    None,
    /// `out_of_scope`: `m_str = 1`; the weapon is outside the model (Necramech exalted weapons, overview §1.3).
    OutOfScope,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The readable names are the five values of 01 §2 `exalted_model` (M-1 of the contract review), and every
    /// variant round-trips in both encodings.
    #[test]
    fn exalted_models_use_the_spec_names() {
        let cases = [
            (
                ExaltedScaling::BaseXStr {
                    s: [100.0, 125.0, 200.0, 250.0],
                },
                r#"{"base_x_str":{"s":[100.0,125.0,200.0,250.0]}}"#,
            ),
            (
                ExaltedScaling::MultInA {
                    m: [1.0, 1.08, 1.16, 1.25],
                },
                r#"{"mult_in_A":{"m":[1.0,1.08,1.16,1.25]}}"#,
            ),
            (
                ExaltedScaling::Peacemaker {
                    p: [1.0, 1.15, 1.33, 1.5],
                },
                r#"{"peacemaker":{"p":[1.0,1.15,1.33,1.5]}}"#,
            ),
            (ExaltedScaling::None, r#""none""#),
            (ExaltedScaling::OutOfScope, r#""out_of_scope""#),
        ];
        for (v, json) in cases {
            assert_eq!(serde_json::to_string(&v).unwrap(), json);
            assert_eq!(serde_json::from_str::<ExaltedScaling>(json).unwrap(), v);
            let bytes = postcard::to_stdvec(&v).unwrap();
            assert_eq!(postcard::from_bytes::<ExaltedScaling>(&bytes).unwrap(), v);
        }
        // The T01a forms are gone.
        assert!(serde_json::from_str::<ExaltedScaling>(r#""base_by_strength""#).is_err());
        assert!(serde_json::from_str::<ExaltedScaling>(r#"{"partial":0.5}"#).is_err());
    }

    /// `compression_mode` defaults to `multiplies` when absent (DC-8 of the contract review).
    #[test]
    fn compression_mode_defaults_to_multiplies() {
        assert_eq!(CompressionMode::default(), CompressionMode::Multiplies);
        assert_eq!(
            serde_json::to_string(&CompressionMode::Adds).unwrap(),
            r#""adds""#
        );
    }
}
