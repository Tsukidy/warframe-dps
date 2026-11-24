//! The stat registry (domain-model.md §10).
//!
//! `define_stats!` declares every additive bucket once. It generates [`StatKey`] (one variant per registry row; rows
//! with several slots carry a typed sub-index, so every `StatKey` value is exactly one slot), the TOML names
//! (`"cc_rel"`, `"forced.SLA"`, `"stack_cap.COR"`, `"vuln_add.HP.HEA"`, `"mult.7"`), the dense slot indices
//! ([`N_STATS`]), and the per-row metadata [`StatMeta`] (spec symbol, aggregation, declared monotonicity class, the
//! stage it enters, the owning spec document). Curators never pick an operation: the stat decides it.
//!
//! Rows added to the table of domain-model.md §10 because the spec defines the bucket (CONTRACTS.md, T01a):
//! `status_dur_type` (03 §4.1, §5), `projectile_speed` and `radius` (06 E-6.20, E-6.22; projectile speed moved out of
//! `display` because it changes falloff), `sniper_combo_dur` (06 E-6.16), `combo_dur_flat` (07 E-7.12), `combo_acc`,
//! `combo_rc`, `combo_curse`, `combo_gain_ext` (07 E-7.13, §2), `tennokai_chance` (07 E-7.17).

use crate::enums::{DamageType, Layer, StatusType, UnknownName};
use crate::serde_util::string_serde;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// How the contributions to a bucket combine.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Aggregate {
    /// Σ (a bonus).
    Sum,
    /// Σ, then clamped by the rule of the consuming stage (ammo efficiency ≤ 0.75, heavy efficiency ≤ 0.9).
    SumClamp,
    /// max over contributions.
    Max,
    /// true when any contribution is nonzero (locks).
    Any,
    /// Σ, used as the factor `1 + x`.
    SumFactor,
    /// Σ inside a multiplier group, `Π (1 + x)` across groups.
    SumProduct,
}

/// Declared monotonicity class of a slot for the optimizer (optimizer.md §4). The compiled scenario may downgrade
/// `Inc` slots to `Unknown` (domain-model.md §10, `compile::monotone`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Monotone {
    /// More of the stat never lowers any score.
    Inc,
    /// More of the stat never raises any score (locks).
    Dec,
    /// Changes the damage mix or proc weights: enumerated, never bounded.
    Unknown,
    /// No damage effect (weapon panel only).
    None,
}

/// Metadata of one registry row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatMeta {
    /// TOML name of the row (`"forced"`); a slot's full name adds the sub-index (`"forced.SLA"`).
    pub toml: &'static str,
    /// Spec symbol.
    pub symbol: &'static str,
    /// Number of slots of the row.
    pub slots: u16,
    /// Index of the row's first slot.
    pub offset: u16,
    pub aggregate: Aggregate,
    pub monotone: Monotone,
    /// The stage or equation the bucket enters (`"ST-4"`, `"E-5"`, `"06"`).
    pub enters: &'static str,
    /// Owning spec document or section (`"01"`, `"04 §3.4"`).
    pub owner: &'static str,
}

/// A sub-index type of a multi-slot row.
pub trait SubIndex: Copy + Sized {
    /// Number of sub-slots.
    const N: usize;
    /// Position of this sub-slot.
    fn sub(self) -> usize;
    /// Sub-slot at a position.
    fn from_sub(i: usize) -> Option<Self>;
    /// Name after the row name (`"SLA"`, `"HP.HEA"`, `"7"`).
    fn sub_name(self) -> String;
    /// Parses [`SubIndex::sub_name`].
    fn parse_sub(s: &str) -> Option<Self>;
}

impl SubIndex for StatusType {
    const N: usize = StatusType::N;
    fn sub(self) -> usize {
        self.index()
    }
    fn from_sub(i: usize) -> Option<Self> {
        StatusType::from_index(i)
    }
    fn sub_name(self) -> String {
        self.name().to_owned()
    }
    fn parse_sub(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

macro_rules! sub_enum {
    ($(#[$m:meta])* $name:ident { $($v:ident => $s:literal),+ $(,)? }) => {
        $(#[$m])*
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub enum $name { $(#[serde(rename = $s)] $v),+ }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$v),+];
        }

        impl SubIndex for $name {
            const N: usize = $name::ALL.len();
            fn sub(self) -> usize {
                self as usize
            }
            fn from_sub(i: usize) -> Option<Self> {
                Self::ALL.get(i).copied()
            }
            fn sub_name(self) -> String {
                match self { $($name::$v => $s.to_owned()),+ }
            }
            fn parse_sub(s: &str) -> Option<Self> {
                match s { $($s => Some($name::$v),)+ _ => None }
            }
        }
    };
}

sub_enum!(
    /// The physical types IPS, sub-index of `phys`.
    Ips { IMP => "IMP", PUN => "PUN", SLA => "SLA" }
);

impl Ips {
    pub fn damage_type(self) -> DamageType {
        match self {
            Ips::IMP => DamageType::IMP,
            Ips::PUN => DamageType::PUN,
            Ips::SLA => DamageType::SLA,
        }
    }
}

sub_enum!(
    /// The ten element types that can be added without combination (01 §4.10.3), sub-index of `elem_nc`.
    NcElem {
        HEA => "HEA", COL => "COL", ELE => "ELE", TOX => "TOX", BLA => "BLA",
        RAD => "RAD", GAS => "GAS", MAG => "MAG", VIR => "VIR", COR => "COR",
    }
);

impl NcElem {
    pub fn damage_type(self) -> DamageType {
        DamageType::from_str(&self.sub_name()).unwrap_or(DamageType::HEA)
    }
    pub fn from_damage_type(t: DamageType) -> Option<NcElem> {
        NcElem::parse_sub(t.name())
    }
}

sub_enum!(
    /// Weapon-panel stats without damage effect (`display`; optimizer drops items that only touch these).
    DisplayStat {
        Range => "range", PunchThrough => "punch_through", Zoom => "zoom", Recoil => "recoil",
        Accuracy => "accuracy", Spread => "spread", AmmoMax => "ammo_max", Noise => "noise",
        HolsterSpeed => "holster_speed", FollowThrough => "follow_through", BlockAngle => "block_angle",
        AmmoPickup => "ammo_pickup",
    }
);

/// Sub-index of `vuln_add`: (layer ℓ, damage type t) of `vuln[ℓ][t]` (05 E-5.19); name `"HP.HEA"`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LayerType {
    pub layer: Layer,
    pub ty: DamageType,
}

impl SubIndex for LayerType {
    const N: usize = Layer::N * DamageType::N;
    fn sub(self) -> usize {
        self.layer.index() * DamageType::N + self.ty.index()
    }
    fn from_sub(i: usize) -> Option<Self> {
        Some(LayerType {
            layer: Layer::from_index(i / DamageType::N)?,
            ty: DamageType::from_index(i % DamageType::N)?,
        })
    }
    fn sub_name(self) -> String {
        format!("{}.{}", self.layer.name(), self.ty.name())
    }
    fn parse_sub(s: &str) -> Option<Self> {
        let (l, t) = s.split_once('.')?;
        Some(LayerType {
            layer: l.parse().ok()?,
            ty: t.parse().ok()?,
        })
    }
}

macro_rules! slot_no {
    ($(#[$m:meta])* $name:ident, $n:literal) => {
        $(#[$m])*
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(transparent)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name(pub u8);

        impl SubIndex for $name {
            const N: usize = $n;
            fn sub(self) -> usize {
                self.0 as usize
            }
            fn from_sub(i: usize) -> Option<Self> {
                (i < $n).then(|| $name(i as u8))
            }
            fn sub_name(self) -> String {
                self.0.to_string()
            }
            fn parse_sub(s: &str) -> Option<Self> {
                let i: usize = s.parse().ok()?;
                // Reject leading zeros and signs so that the name round-trips.
                (i.to_string() == s).then_some(())?;
                Self::from_sub(i)
            }
        }
    };
}

slot_no!(
    /// Extra-hit kind slot `xh.<n>` (p_xh by extra-hit kind, E-14), n < 8.
    XhSlot, 8
);
slot_no!(
    /// Mechanic magnitude slot `mech.<n>`, allocated per scenario (domain-model.md §10), n < 16.
    MechSlot, 16
);
slot_no!(
    /// Multiplier-group slot `mult.<n>`, allocated from `Rules.mult_groups` (domain-model.md §14), n < 48.
    MultSlot, 48
);

macro_rules! stat_row_len {
    () => {
        1usize
    };
    ($sub:ty) => {
        <$sub as SubIndex>::N
    };
}

macro_rules! stat_row_sub {
    ($key:ident, $v:ident) => {
        if let StatKey::$v = $key {
            Some(0usize)
        } else {
            None
        }
    };
    ($key:ident, $v:ident, $sub:ty) => {
        if let StatKey::$v(s) = $key {
            Some(<$sub as SubIndex>::sub(s))
        } else {
            None
        }
    };
}

macro_rules! stat_row_make {
    ($v:ident, $i:expr) => {
        Some(StatKey::$v)
    };
    ($v:ident, $i:expr, $sub:ty) => {
        <$sub as SubIndex>::from_sub($i).map(StatKey::$v)
    };
}

macro_rules! stat_row_parse {
    ($v:ident, $tail:ident) => {
        if $tail.is_none() {
            Some(StatKey::$v)
        } else {
            None
        }
    };
    ($v:ident, $tail:ident, $sub:ty) => {
        $tail
            .and_then(<$sub as SubIndex>::parse_sub)
            .map(StatKey::$v)
    };
}

macro_rules! stat_row_name {
    ($key:ident, $toml:literal, $v:ident) => {
        if let StatKey::$v = $key {
            Some($toml.to_owned())
        } else {
            None
        }
    };
    ($key:ident, $toml:literal, $v:ident, $sub:ty) => {
        if let StatKey::$v(s) = $key {
            Some(format!("{}.{}", $toml, <$sub as SubIndex>::sub_name(s)))
        } else {
            None
        }
    };
}

macro_rules! define_stats {
    ($(
        $(#[$m:meta])*
        $toml:literal $v:ident $(($sub:ty))? , $sym:literal, $agg:ident, $mono:ident, $enters:literal, $owner:literal;
    )+) => {
        /// A bucket slot of the stat registry. Readable and compact form: the full name (`"forced.SLA"`).
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum StatKey {
            $($(#[$m])* $v $(($sub))?),+
        }

        const ROW_SLOTS: &[usize] = &[$(stat_row_len!($($sub)?)),+];
        /// Number of registry rows.
        pub const N_STAT_ROWS: usize = ROW_SLOTS.len();
        /// Number of slots (length of a stat vector).
        pub const N_STATS: usize = 0 $(+ stat_row_len!($($sub)?))+;

        const ROW_OFFSETS: [usize; N_STAT_ROWS] = {
            let mut out = [0usize; N_STAT_ROWS];
            let mut i = 1;
            while i < N_STAT_ROWS {
                out[i] = out[i - 1] + ROW_SLOTS[i - 1];
                i += 1;
            }
            out
        };

        /// Metadata of every registry row, in registry order.
        pub static STAT_ROWS: [StatMeta; N_STAT_ROWS] = {
            let toml = [$($toml),+];
            let sym = [$($sym),+];
            let agg = [$(Aggregate::$agg),+];
            let mono = [$(Monotone::$mono),+];
            let enters = [$($enters),+];
            let owner = [$($owner),+];
            let mut out = [StatMeta {
                toml: "", symbol: "", slots: 0, offset: 0, aggregate: Aggregate::Sum, monotone: Monotone::None,
                enters: "", owner: "",
            }; N_STAT_ROWS];
            let mut i = 0;
            while i < N_STAT_ROWS {
                out[i] = StatMeta {
                    toml: toml[i], symbol: sym[i], slots: ROW_SLOTS[i] as u16, offset: ROW_OFFSETS[i] as u16,
                    aggregate: agg[i], monotone: mono[i], enters: enters[i], owner: owner[i],
                };
                i += 1;
            }
            out
        };

        impl StatKey {
            /// Registry row of this slot.
            pub fn row(self) -> usize {
                let key = self;
                let mut r = 0usize;
                $(
                    if stat_row_sub!(key, $v $(, $sub)?).is_some() {
                        return r;
                    }
                    r += 1;
                )+
                let _ = r;
                unreachable!("every StatKey variant is a registry row")
            }

            /// Dense slot index, `< N_STATS`.
            pub fn index(self) -> usize {
                let key = self;
                let mut r = 0usize;
                $(
                    if let Some(sub) = stat_row_sub!(key, $v $(, $sub)?) {
                        return ROW_OFFSETS[r] + sub;
                    }
                    r += 1;
                )+
                let _ = r;
                unreachable!("every StatKey variant is a registry row")
            }

            /// The slot at a dense index.
            pub fn from_index(i: usize) -> Option<StatKey> {
                let mut r = 0usize;
                $(
                    if i >= ROW_OFFSETS[r] && i < ROW_OFFSETS[r] + ROW_SLOTS[r] {
                        return stat_row_make!($v, i - ROW_OFFSETS[r] $(, $sub)?);
                    }
                    r += 1;
                )+
                let _ = r;
                None
            }

            /// Parses a full slot name (`"cc_rel"`, `"forced.SLA"`, `"stack_cap.COR"`, `"vuln_add.HP.HEA"`).
            pub fn parse(s: &str) -> Option<StatKey> {
                let (head, tail) = match s.split_once('.') {
                    Some((h, t)) => (h, Some(t)),
                    None => (s, None),
                };
                $(
                    if head == $toml {
                        return stat_row_parse!($v, tail $(, $sub)?);
                    }
                )+
                None
            }

            /// Full slot name, the inverse of [`StatKey::parse`].
            pub fn name(self) -> String {
                let key = self;
                $(
                    if let Some(n) = stat_row_name!(key, $toml, $v $(, $sub)?) {
                        return n;
                    }
                )+
                unreachable!("every StatKey variant is a registry row")
            }
        }
    };
}

define_stats! {
//  TOML key                 variant                   spec symbol                                 aggregate   monotone enters      owner
    /// Σ base damage bonus A.
    "base_dmg"               BaseDmg,                  "sum_base_dmg (A)",                         Sum,        Inc,     "ST-4",     "01";
    /// A_w, the melee-projectile "weapon damage" bucket (04 §3.4).
    "base_dmg_universal"     BaseDmgUniversal,         "A_w melee-projectile bucket",              Sum,        Inc,     "ST-4",     "04 §3.4";
    /// Incarnon flat X (HP), excluded from B_ref.
    "incarnon_flat"          IncarnonFlat,             "incarnon_flat (X, HP)",                    Sum,        Inc,     "ST-1",     "01";
    /// Σ physical bonus per IPS type.
    "phys"                   Phys(Ips),                "sum_phys[t] t ∈ IPS",                      Sum,        Unknown, "ST-2",     "01";
    /// Non-combining element additions.
    "elem_nc"                ElemNc(NcElem),           "sum_elem[t], non-combining",               Sum,        Unknown, "ST-2",     "01 §4.10.3";
    /// Phoenix Talons (D-007).
    "phoenix_talons"         PhoenixTalons,            "pt (01 §4.9)",                             Sum,        Unknown, "ST-2",     "01";
    /// Σ faction bonus F (resolved at L1 for the target).
    "faction"                Faction,                  "sum_faction (F)",                          Sum,        Inc,     "ST-5",     "01 §4.15";
    /// CO coefficient of the distinct status count n.
    "co_per_status"          CoPerStatus,              "C coefficient of n",                       Sum,        Inc,     "ST-4",     "04 §2";
    /// CO coefficient of the Cold stack count.
    "co_per_cold_stack"      CoPerColdStack,           "C coefficient of n_COL",                   Sum,        Inc,     "ST-4",     "04 §2";
    /// CO constant terms.
    "co_flat"                CoFlat,                   "C constant terms",                         Sum,        Inc,     "ST-4",     "04 §2";
    "cc_base_add"            CcBaseAdd,                "CC0+",                                     Sum,        Inc,     "E-5",      "02";
    "cc_rel"                 CcRel,                    "ΣCC_rel",                                  Sum,        Inc,     "E-5",      "02";
    "cc_abs"                 CcAbs,                    "ΣCC_abs",                                  Sum,        Inc,     "E-5",      "02";
    "cc_floor"               CcFloor,                  "CC_min",                                   Max,        Inc,     "E-5",      "02";
    "cc_final_mul"           CcFinalMul,               "custom p.d factor",                        SumFactor,  Inc,     "E-5",      "00 §6.3";
    "cd_base_add"            CdBaseAdd,                "CDM0+",                                    Sum,        Inc,     "E-4",      "02";
    "cd_rel"                 CdRel,                    "ΣCD_rel",                                  Sum,        Inc,     "E-4",      "02";
    "cd_abs"                 CdAbs,                    "ΣCD_abs",                                  Sum,        Inc,     "E-4",      "02";
    "cd_field_mul"           CdFieldMul,               "M_cdf",                                    SumFactor,  Inc,     "E-4",      "02";
    "tier_raise"             TierRaise,                "p_vig",                                    Sum,        Inc,     "E-6",      "02";
    "wp_dmg"                 WpDmg,                    "ΣWP",                                      Sum,        Inc,     "E-7",      "02";
    "hs_mult"                HsMult,                   "ΣHS",                                      Sum,        Inc,     "E-7",      "02";
    "sc_base_add"            ScBaseAdd,                "SC0+",                                     Sum,        Inc,     "E-8",      "02";
    "sc_rel"                 ScRel,                    "ΣSC_rel",                                  Sum,        Inc,     "E-8",      "02";
    "sc_abs"                 ScAbs,                    "ΣSC_abs",                                  Sum,        Inc,     "E-8",      "02";
    "sc_mul"                 ScMul,                    "M_sc (custom p.k)",                        SumFactor,  Inc,     "E-8",      "02";
    /// Forced procs per pellet.
    "forced"                 Forced(StatusType),       "forced_procs[s] per pellet",               Sum,        Unknown, "E-9",      "02 §4.7.2";
    "multishot"              Multishot,                "ΣMS",                                      Sum,        Inc,     "E-10",     "02";
    "multishot_base_add"     MultishotBaseAdd,         "MS0+",                                     Sum,        Inc,     "E-10",     "02";
    /// Acuity lock.
    "lock_multishot"         LockMultishot,            "Acuity lock",                              Any,        Dec,     "E-10",     "02";
    "status_dmg"             StatusDmg,                "A_sd",                                     Sum,        Inc,     "E-13",     "03";
    "status_dmg_type"        StatusDmgType(StatusType), "A_sd,s",                                  Sum,        Inc,     "E-13",     "03";
    "status_dur"             StatusDur,                "SD",                                       Sum,        Inc,     "CD-06",    "03";
    /// Per-type status duration (03 §4.1: `SD_p = sum_status_dur + status_dur_type[s]`).
    "status_dur_type"        StatusDurType(StatusType), "status_dur_type[s]",                      Sum,        Inc,     "CD-06",    "03 §4.1";
    "stack_cap"              StackCap(StatusType),     "stack_cap[s] increase",                    Sum,        Inc,     "CD-07",    "03";
    /// External stacks (T-06).
    "external"               External(StatusType),     "external stacks (T-06)",                   Max,        Inc,     "state",    "03";
    "dot_vuln"               DotVuln,                  "M_dot (custom p.o)",                       SumFactor,  Inc,     "E-13",     "03";
    "heat_inherit"           HeatInherit,              "Heat inherit (D-028, p.y)",                Sum,        Inc,     "E-13",     "03 §10.3";
    "fire_rate"              FireRate,                 "fire-rate bucket",                         Sum,        Inc,     "06",       "06 §5.1";
    /// Cannonade lock.
    "lock_fire_rate"         LockFireRate,             "Cannonade lock",                           Any,        Dec,     "06",       "06 §5.1";
    "charge_rate"            ChargeRate,               "charge speed",                             Sum,        Inc,     "06",       "06 §5.3";
    /// Magazine percent (downgraded by shot marks).
    "magazine"               Magazine,                 "magazine %",                               Sum,        Inc,     "06",       "06 §5.5";
    "magazine_flat"          MagazineFlat,             "magazine +N",                              Sum,        Inc,     "06",       "06 §5.5";
    "reload_speed"           ReloadSpeed,              "reload speed",                             Sum,        Inc,     "06",       "06 §5.7";
    "ammo_eff"               AmmoEff,                  "ammo efficiency ≤ 0.75",                   SumClamp,   Inc,     "06",       "06 §5.6";
    /// ΣPS: scales falloff distances and flight speed (06 E-6.20, E-6.22).
    "projectile_speed"       ProjectileSpeed,          "ΣPS",                                      Sum,        Inc,     "06",       "06 §8.1";
    /// ΣRadius: scales the AoE falloff end distance (06 E-6.20).
    "radius"                 Radius,                   "ΣRadius",                                  Sum,        Inc,     "06",       "06 §8.1";
    /// Sniper combo duration additions (Harkonar Scope; 06 E-6.16 D_snp).
    "sniper_combo_dur"       SniperComboDur,           "D_snp additions",                          Sum,        Inc,     "06",       "06 §7.2";
    "attack_speed"           AttackSpeed,              "AS",                                       Sum,        Inc,     "07",       "07";
    "initial_combo"          InitialCombo,             "initial combo",                            Sum,        Inc,     "07",       "07";
    /// Combo duration, percent part Σpct (07 E-7.12).
    "combo_dur"              ComboDur,                 "combo duration Σpct",                      Sum,        Inc,     "07",       "07";
    /// Combo duration, flat seconds Σflat (07 E-7.12).
    "combo_dur_flat"         ComboDurFlat,             "combo duration Σflat",                     Sum,        Inc,     "07",       "07 E-7.12";
    "heavy_eff"              HeavyEff,                 "heavy efficiency ≤ 0.9",                   SumClamp,   Inc,     "07",       "07";
    "wind_up"                WindUp,                   "wind-up speed",                            Sum,        Inc,     "07",       "07";
    /// Additional combo count chance p_acc (07 E-7.13).
    "combo_acc"              ComboAcc,                 "p_acc",                                    Sum,        Inc,     "07",       "07 E-7.13";
    /// Relentless Combination chance p_RC (07 E-7.13b).
    "combo_rc"               ComboRc,                  "p_RC",                                     Sum,        Inc,     "07",       "07 E-7.13b";
    /// Riven "Chance to Gain Combo Count" curse x_cur, as a positive fraction (07 E-7.13).
    "combo_curse"            ComboCurse,               "x_cur",                                    Sum,        Dec,     "07",       "07 E-7.13";
    /// External combo gain rate in points per second (07 §2, class (c), default 0).
    "combo_gain_ext"         ComboGainExt,             "external combo gain rate",                 Sum,        Inc,     "07",       "07 §6.9";
    /// Tennokai chance modifiers (07 E-7.17).
    "tennokai_chance"        TennokaiChance,           "p_tk modifiers",                           Sum,        Inc,     "07",       "07 E-7.17";
    /// Strength sources (04 §4.4).
    "strength"               Strength,                 "Str sources (04 §4.4)",                    Sum,        Inc,     "scenario", "04 §4.4";
    /// Additive vulnerability per (layer ℓ, type t).
    "vuln_add"               VulnAdd(LayerType),       "V additive group, per (layer ℓ, type t)",  Sum,        Inc,     "ST-8",     "05 E-5.19, vuln[ℓ][t]";
    "strip_ability"          StripAbility,             "E-15 ability strip",                       Sum,        Inc,     "E-15",     "05 §9.3";
    "strip_aura"             StripAura,                "E-15 Corrosive Projection",                Sum,        Inc,     "E-15",     "05 §9.3";
    /// p_xh by extra-hit kind.
    "xh"                     Xh(XhSlot),               "p_xh by extra-hit kind",                   Sum,        Inc,     "E-14",     "04 §6";
    /// Mechanic magnitudes (allocated per scenario).
    "mech"                   Mech(MechSlot),           "mechanic magnitudes",                      Sum,        Inc,     "E-9 …",    "02, 04";
    /// One slot per multiplier group (allocated from `Rules.mult_groups`).
    "mult"                   Mult(MultSlot),           "one slot per multiplier group",            SumProduct, Inc,     "group",    "04 §3";
    /// Weapon-panel stats.
    "display"                Display(DisplayStat),     "range, punch-through, zoom, recoil, …",    Sum,        None,    "panel",    "—";
}

impl StatKey {
    /// Metadata of the slot's registry row.
    pub fn meta(self) -> &'static StatMeta {
        &STAT_ROWS[self.row()]
    }

    /// Every slot in index order.
    pub fn all() -> impl Iterator<Item = StatKey> {
        (0..N_STATS).filter_map(StatKey::from_index)
    }

    /// Every slot name in index order.
    pub fn all_names() -> Vec<String> {
        StatKey::all().map(StatKey::name).collect()
    }
}

impl fmt::Display for StatKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

impl FromStr for StatKey {
    type Err = UnknownName;
    fn from_str(s: &str) -> Result<Self, UnknownName> {
        StatKey::parse(s).ok_or_else(|| UnknownName {
            what: "stat key",
            name: s.to_owned(),
        })
    }
}

string_serde!(StatKey, StatKey::all_names());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_shape() {
        assert_eq!(N_STAT_ROWS, STAT_ROWS.len());
        assert_eq!(
            STAT_ROWS.iter().map(|m| m.slots as usize).sum::<usize>(),
            N_STATS
        );
        assert_eq!(N_STATS, 296);
        assert_eq!(StatKey::BaseDmg.index(), 0);
        assert_eq!(
            StatKey::from_index(N_STATS - 1),
            Some(StatKey::Display(DisplayStat::AmmoPickup))
        );
        assert_eq!(StatKey::from_index(N_STATS), None);
        assert_eq!(
            StatKey::parse("forced.SLA"),
            Some(StatKey::Forced(StatusType::SLA))
        );
        assert_eq!(
            StatKey::parse("stack_cap.COR"),
            Some(StatKey::StackCap(StatusType::COR))
        );
        assert_eq!(
            StatKey::parse("vuln_add.HP.HEA"),
            Some(StatKey::VulnAdd(LayerType {
                layer: Layer::HP,
                ty: DamageType::HEA
            }))
        );
        assert_eq!(StatKey::parse("forced"), None);
        assert_eq!(StatKey::parse("cc_rel.SLA"), None);
        assert_eq!(StatKey::parse("mult.48"), None);
        assert_eq!(StatKey::parse("mult.07"), None);
        assert_eq!(StatKey::CcRel.meta().monotone, Monotone::Inc);
        assert_eq!(StatKey::Forced(StatusType::IMP).meta().toml, "forced");
        assert_eq!(StatKey::LockMultishot.meta().aggregate, Aggregate::Any);
    }

    #[test]
    fn every_key_round_trips() {
        let mut seen = std::collections::BTreeSet::new();
        for i in 0..N_STATS {
            let k = StatKey::from_index(i).expect("dense index");
            assert_eq!(k.index(), i);
            let name = k.name();
            assert!(seen.insert(name.clone()), "duplicate name {name}");
            assert_eq!(StatKey::parse(&name), Some(k), "{name}");
            let json = serde_json::to_string(&k).unwrap();
            assert_eq!(serde_json::from_str::<StatKey>(&json).unwrap(), k);
            let row = &STAT_ROWS[k.row()];
            assert!(i >= row.offset as usize && i < (row.offset + row.slots) as usize);
        }
    }
}
