//! Identifiers (domain-model.md §2).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Permanent identity of anything in the bundle. DE items use their DE path
/// (`"/Lotus/Weapons/Tenno/Pistol/AutoPistol"`). Things DE does not export use our namespace `"wfd:<kind>/<slug>"`,
/// e.g. `"wfd:buff/roar"`, `"wfd:enemy/grineer/lancer"`, `"wfd:perk/furis/haven_foray"`, `"wfd:custom/base"`,
/// `"wfd:shard/violet/primary_electricity"`. Enemies have synthetic names `wfd:enemy/<wiki sub-module>/<slug>`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "string"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UniqueName(pub Box<str>);

impl UniqueName {
    pub fn new(s: impl Into<Box<str>>) -> Self {
        UniqueName(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// `true` for our own namespace (`wfd:…`), i.e. things DE does not export.
    pub fn is_wfd(&self) -> bool {
        self.0.starts_with("wfd:")
    }
}

impl fmt::Display for UniqueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for UniqueName {
    fn from(s: &str) -> Self {
        UniqueName(s.into())
    }
}

/// Keys at or above this value are session-local (assigned by an in-app refresh, pipeline.md §9.2); documents that
/// contain one carry its uniqueName in `CalcInput.x` (AD-18).
pub const LOCAL_KEY_BASE: u32 = 1 << 31;

/// Public key. Keys < [`LOCAL_KEY_BASE`] are global: assigned append-only by `data/generated/stable_keys.json`, never
/// reused, never renumbered. Keys ≥ [`LOCAL_KEY_BASE`] are session-local and differ between machines. Used by the HTTP
/// API, share codes and the browser's local storage. JSON: an integer. As a map key (`CalcInput.x`) it is the decimal
/// string, as JSON requires.
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[serde(transparent)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "number"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StableKey(pub u32);

impl StableKey {
    /// `true` for a session-local key (≥ 2^31).
    pub fn is_local(self) -> bool {
        self.0 >= LOCAL_KEY_BASE
    }
}

impl fmt::Display for StableKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Declares a dense index newtype: never persisted in documents, never sent over HTTP (it is serialized only inside
/// the bundle, where it is the position in a vector).
macro_rules! dense_id {
    ($(#[$m:meta])* $name:ident($inner:ty)) => {
        $(#[$m])*
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[serde(transparent)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "number"))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name(pub $inner);

        impl $name {
            /// The index as `usize`.
            pub fn idx(self) -> usize {
                self.0 as usize
            }
        }
    };
}

dense_id!(
    /// Index into `Bundle.weapons`.
    WeaponId(u32)
);
dense_id!(
    /// Index into `Bundle.items`.
    ItemId(u32)
);
dense_id!(
    /// Index into `Bundle.enemies`.
    EnemyId(u32)
);
dense_id!(
    /// Index into `Weapon.attacks`.
    AttackIdx(u8)
);
dense_id!(
    /// Index into `Rules.factions` (data-driven: new factions need no code).
    FactionId(u8)
);
dense_id!(
    /// Index into `Bundle.sets`.
    SetId(u16)
);
dense_id!(
    /// Index into `Bundle.stances`.
    StanceId(u16)
);
dense_id!(
    /// Index into `Bundle.combo_tables` (stance, stanceless class or unique-to-weapon table).
    ComboTableId(u16)
);
dense_id!(
    /// Index into `Bundle.frames`.
    FrameId(u16)
);
dense_id!(
    /// Incarnon line / weapon family (Furis and MK1-Furis share one).
    FamilyId(u16)
);
dense_id!(
    /// Index into `Rules.mult_groups`.
    MultGroupId(u8)
);
dense_id!(
    /// Compatibility tag (PROJECTILE, AOE, NIKANAS_STANCE …), index into `Rules.tags`.
    TagId(u16)
);

/// Identity of an attack that survives data updates: the slug of the wiki `AttackName` (`"normal_attack"`,
/// `"incarnon_form"`, `"charged_shot"`, `"heavy_slam_attack"`) plus its index. Decoders match the slug first and fall
/// back to the index, reporting a warning when only the index matched.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AttackKey {
    pub slug: Box<str>,
    pub idx: u8,
}

/// Declares a string newtype.
macro_rules! string_id {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[serde(transparent)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(type = "string"))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name(pub Box<str>);

        impl $name {
            pub fn new(s: impl Into<Box<str>>) -> Self {
                $name(s.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                $name(s.into())
            }
        }
    };
}

string_id!(
    /// Item parameter key (`"element"`, `"value"`).
    ParamKey
);
string_id!(
    /// `"<UniqueName>#<effect id>"`, e.g. `"/Lotus/Upgrades/Mods/Rifle/WeaponStatusChanceSPMod#co"`. Used for
    /// per-effect condition overrides, traces and the MC mechanic id `"eff/<effect key>"` (FNV-1a 32-bit at L0, spec 08
    /// E-8.16; engine.md §8.2).
    EffectKey
);
string_id!(
    /// Key into `NameTable` (the UniqueName of the thing named).
    NameKey
);
string_id!(
    /// Combo type slug of 07 E-7.0b: `"neutral"`, `"forward"`, `"block"`, `"forward_block"` …
    ComboKey
);

impl EffectKey {
    /// `"<unique name>#<effect id>"`.
    pub fn of(item: &UniqueName, effect_id: &str) -> Self {
        EffectKey(format!("{}#{}", item.0, effect_id).into())
    }
    /// Splits into (unique name, effect id) at the last `#`.
    pub fn split(&self) -> Option<(&str, &str)> {
        self.0.rsplit_once('#')
    }
}
