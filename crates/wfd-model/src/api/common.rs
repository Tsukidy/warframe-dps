//! Small shapes shared by several routes.

use crate::enums::{DamageType, DamageVec, Layer, N_DT};
use serde::{Deserialize, Deserializer, Serialize};

macro_rules! per_damage_type {
    ($($t:ident),+ $(,)?) => {
        /// One number per damage type, every type present: server-api.md's `Record<DamageType, number>` (weapon base
        /// damage, faction modifiers). JSON `{ "IMP": 0, "PUN": 0, …, "FIN": 0 }`.
        #[allow(non_snake_case, missing_docs)]
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
        #[serde(deny_unknown_fields)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct PerDamageType {
            $(pub $t: f64),+
        }

        impl PerDamageType {
            /// From a vector in spec order.
            pub fn from_vec(v: &DamageVec) -> Self {
                let mut it = v.iter().copied();
                PerDamageType { $($t: it.next().unwrap_or(0.0)),+ }
            }

            /// The vector in spec order.
            pub fn to_vec(&self) -> DamageVec {
                [$(self.$t),+]
            }
        }
    };
}

per_damage_type!(
    IMP, PUN, SLA, HEA, COL, ELE, TOX, BLA, RAD, GAS, MAG, VIR, COR, VOI, TAU, TRU, CIN, FIN
);

// `to_vec` builds a `DamageVec` from the field list, so the list has exactly `N_DT` entries in spec order.
const _: [f64; N_DT] = [0.0; 18];

impl PerDamageType {
    /// The value of one type.
    pub fn get(&self, t: DamageType) -> f64 {
        self.to_vec()[t.index()]
    }
}

/// One number per enemy layer: `{ "OG": …, "SH": …, "HP": … }` (scaled pools, innate DR).
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LayerValues {
    /// Overguard.
    pub OG: f64,
    /// Shield.
    pub SH: f64,
    /// Health.
    pub HP: f64,
}

impl LayerValues {
    /// The value of one layer.
    pub fn get(&self, l: Layer) -> f64 {
        match l {
            Layer::OG => self.OG,
            Layer::SH => self.SH,
            Layer::HP => self.HP,
        }
    }
}

/// A base and a modded value (weapon panel rows, server-api.md §6.1).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BaseModded {
    /// Unmodded value.
    pub base: f64,
    /// Value with the build applied.
    pub modded: f64,
}

/// Damage falloff (06 §8): full damage up to `start` metres, `min_factor` from `end` on.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FalloffView {
    /// Start of falloff (m).
    pub start: f64,
    /// End of falloff (m).
    pub end: f64,
    /// Damage factor at and beyond `end`.
    pub min_factor: f64,
}

/// A value and its label: one part of a breakdown (`{ "source": "dot.SLA", "dps": 812.5 }`).
///
/// Source ids: `direct.<DamageType>` (direct damage per final type), `dot.<StatusType>` (DoT ticks), `heat` (Heat
/// pulses), `blast` (Blast hits), `xh.<EffectKey>` (extra hits per source), `duplicate` (Melee Duplicate),
/// `child.<attack slug>` (lingering child ticks), `magnetic_break` (TTK breakdowns only), 08 §8.2.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceDps {
    /// Source id (see the type documentation).
    pub source: Box<str>,
    /// Landed damage per second in the window (EV expectation, MC mean over runs).
    pub dps: f64,
}

/// Landed damage of one source up to the kill (TTK breakdown, 08 §8.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceDamage {
    /// Source id ([`SourceDps`] ids plus `magnetic_break`).
    pub source: Box<str>,
    /// Damage routed to the target up to the kill or `t_ttk_max` (HP).
    pub damage: f64,
}

/// Deserializer for `Option<Option<f64>>` fields written as `field?: number | null`: absent = `None`, `null` =
/// `Some(None)`, a number = `Some(Some(x))`. Use with `#[serde(default, deserialize_with = …,
/// skip_serializing_if = "Option::is_none")]`.
pub fn double_option<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Option<f64>>, D::Error> {
    Option::<f64>::deserialize(d).map(Some)
}
