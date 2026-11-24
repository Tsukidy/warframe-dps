//! Enumerations (domain-model.md §3). Orders follow spec 00 §2.2 and are used for array indexing.

use crate::serde_util::{FlagCase, flags_serde};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Error of the `FromStr` impls of this module: the string names no member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownName {
    pub what: &'static str,
    pub name: String,
}

impl fmt::Display for UnknownName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown {} `{}`", self.what, self.name)
    }
}

impl std::error::Error for UnknownName {}

/// Declares an enum whose JSON form is its spec ID (`"IMP"`, `"OG"`), with `ALL`, `N`, `index`, `from_index`,
/// `name`, `Display` and `FromStr`.
macro_rules! id_enum {
    ($(#[$m:meta])* $name:ident, $what:literal { $($(#[$vm:meta])* $v:ident),+ $(,)? }) => {
        $(#[$m])*
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[repr(u8)]
        pub enum $name { $($(#[$vm])* $v),+ }

        impl $name {
            /// Every member in spec order.
            pub const ALL: &'static [$name] = &[$($name::$v),+];
            /// Number of members.
            pub const N: usize = $name::ALL.len();
            /// Position in spec order (array index).
            pub const fn index(self) -> usize {
                self as usize
            }
            /// Member at a position in spec order.
            pub fn from_index(i: usize) -> Option<Self> {
                Self::ALL.get(i).copied()
            }
            /// The spec ID.
            pub const fn name(self) -> &'static str {
                match self { $($name::$v => stringify!($v)),+ }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.name())
            }
        }

        impl FromStr for $name {
            type Err = UnknownName;
            fn from_str(s: &str) -> Result<Self, UnknownName> {
                match s {
                    $(stringify!($v) => Ok($name::$v),)+
                    _ => Err(UnknownName { what: $what, name: s.to_owned() }),
                }
            }
        }
    };
}

id_enum!(
    /// Damage types, spec 00 §2.2 order.
    DamageType, "damage type" {
        IMP, PUN, SLA, HEA, COL, ELE, TOX, BLA, RAD, GAS, MAG, VIR, COR, VOI, TAU, TRU, CIN, FIN
    }
);

id_enum!(
    /// Status types, spec 00 §2.2 order: the damage types IMP … TAU plus Microwave, Knockdown, Lifted.
    StatusType, "status type" {
        IMP, PUN, SLA, HEA, COL, ELE, TOX, BLA, RAD, GAS, MAG, VIR, COR, VOI, TAU, MIC, KND, LIF
    }
);

/// Number of damage types.
pub const N_DT: usize = 18;
/// Number of status types.
pub const N_ST: usize = 18;
/// Per-type damage vector indexed by [`DamageType::index`].
pub type DamageVec = [f64; N_DT];

const _: () = assert!(DamageType::N == N_DT && StatusType::N == N_ST);

impl DamageType {
    /// The physical types IPS (00 §2.2).
    pub const IPS: [DamageType; 3] = [DamageType::IMP, DamageType::PUN, DamageType::SLA];
    /// The primary elements PE in combination order HCET (00 §2.2, 01 §4.11).
    pub const PRIMARY_ELEMENTS: [DamageType; 4] =
        [DamageType::HEA, DamageType::COL, DamageType::ELE, DamageType::TOX];
    /// The secondary elements SE (00 §2.2).
    pub const SECONDARY_ELEMENTS: [DamageType; 6] = [
        DamageType::BLA,
        DamageType::RAD,
        DamageType::GAS,
        DamageType::MAG,
        DamageType::VIR,
        DamageType::COR,
    ];

    /// The status type of the same ID (IMP … TAU); `None` for TRU, CIN, FIN.
    pub fn status(self) -> Option<StatusType> {
        StatusType::from_str(self.name()).ok()
    }

    /// The combination of two distinct primary elements (00 §2.2: BLA = HEA+COL, RAD = HEA+ELE, GAS = HEA+TOX,
    /// MAG = COL+ELE, VIR = COL+TOX, COR = ELE+TOX); `None` otherwise.
    pub fn combine(a: DamageType, b: DamageType) -> Option<DamageType> {
        use DamageType::*;
        let (x, y) = if a.index() <= b.index() { (a, b) } else { (b, a) };
        match (x, y) {
            (HEA, COL) => Some(BLA),
            (HEA, ELE) => Some(RAD),
            (HEA, TOX) => Some(GAS),
            (COL, ELE) => Some(MAG),
            (COL, TOX) => Some(VIR),
            (ELE, TOX) => Some(COR),
            _ => None,
        }
    }
}

impl StatusType {
    /// The damage type of the same ID; `None` for MIC, KND, LIF.
    pub fn damage_type(self) -> Option<DamageType> {
        DamageType::from_str(self.name()).ok()
    }
}

bitflags::bitflags! {
    /// A set of status types, one bit per [`StatusType`] (readable form: list of IDs).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct StatusSet: u32 {
        const IMP = 1 << 0; const PUN = 1 << 1; const SLA = 1 << 2; const HEA = 1 << 3; const COL = 1 << 4;
        const ELE = 1 << 5; const TOX = 1 << 6; const BLA = 1 << 7; const RAD = 1 << 8; const GAS = 1 << 9;
        const MAG = 1 << 10; const VIR = 1 << 11; const COR = 1 << 12; const VOI = 1 << 13; const TAU = 1 << 14;
        const MIC = 1 << 15; const KND = 1 << 16; const LIF = 1 << 17;
    }
}
flags_serde!(StatusSet, u32, Upper);

impl StatusSet {
    pub fn of(s: StatusType) -> Self {
        StatusSet::from_bits_retain(1 << s.index())
    }
    pub fn has(self, s: StatusType) -> bool {
        self.contains(StatusSet::of(s))
    }
    pub fn iter_types(self) -> impl Iterator<Item = StatusType> {
        StatusType::ALL.iter().copied().filter(move |s| self.has(*s))
    }
}

impl FromIterator<StatusType> for StatusSet {
    fn from_iter<I: IntoIterator<Item = StatusType>>(iter: I) -> Self {
        iter.into_iter().fold(StatusSet::empty(), |acc, s| acc | StatusSet::of(s))
    }
}

bitflags::bitflags! {
    /// A set of damage types, one bit per [`DamageType`] (readable form: list of IDs).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct DamageTypeSet: u32 {
        const IMP = 1 << 0; const PUN = 1 << 1; const SLA = 1 << 2; const HEA = 1 << 3; const COL = 1 << 4;
        const ELE = 1 << 5; const TOX = 1 << 6; const BLA = 1 << 7; const RAD = 1 << 8; const GAS = 1 << 9;
        const MAG = 1 << 10; const VIR = 1 << 11; const COR = 1 << 12; const VOI = 1 << 13; const TAU = 1 << 14;
        const TRU = 1 << 15; const CIN = 1 << 16; const FIN = 1 << 17;
    }
}
flags_serde!(DamageTypeSet, u32, Upper);

impl DamageTypeSet {
    pub fn of(t: DamageType) -> Self {
        DamageTypeSet::from_bits_retain(1 << t.index())
    }
    pub fn has(self, t: DamageType) -> bool {
        self.contains(DamageTypeSet::of(t))
    }
    pub fn iter_types(self) -> impl Iterator<Item = DamageType> {
        DamageType::ALL.iter().copied().filter(move |t| self.has(*t))
    }
}

impl FromIterator<DamageType> for DamageTypeSet {
    fn from_iter<I: IntoIterator<Item = DamageType>>(iter: I) -> Self {
        iter.into_iter().fold(DamageTypeSet::empty(), |acc, t| acc | DamageTypeSet::of(t))
    }
}

id_enum!(
    /// Enemy layers (00 §2.2): overguard, shield, health. Armor is a reduction on HP, not a pool.
    Layer, "layer" { OG, SH, HP }
);

bitflags::bitflags! {
    /// A set of layers (readable form: `["OG", "SH", "HP"]`). Fields of this type default to all three.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct LayerSet: u8 {
        const OG = 1 << 0;
        const SH = 1 << 1;
        const HP = 1 << 2;
    }
}
flags_serde!(LayerSet, u8, Upper);

impl Default for LayerSet {
    /// All three layers.
    fn default() -> Self {
        LayerSet::all()
    }
}

impl LayerSet {
    pub fn of(l: Layer) -> Self {
        LayerSet::from_bits_retain(1 << l.index())
    }
    pub fn has(self, l: Layer) -> bool {
        self.contains(LayerSet::of(l))
    }
    /// `true` when the set is all three layers (serde skip helper).
    pub fn is_all_layers(&self) -> bool {
        *self == LayerSet::all()
    }
}

/// Layer views of spec 00 §2.2: the inspiration's four result panels plus `layered` (OG → SH → HP, used for TTK).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum View {
    Health,
    Armor,
    Shield,
    Overguard,
    Layered,
}

impl View {
    pub const ALL: [View; 5] = [View::Health, View::Armor, View::Shield, View::Overguard, View::Layered];
}

/// Equipment slot of a weapon record.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum WeaponSlot {
    Primary,
    Secondary,
    Melee,
    ArchGun,
    ArchMelee,
    Exalted,
    Companion,
}

/// Weapon class for effect scopes and riven classes (melee subclass: `MeleeData.class`).
/// `Crossbow`, `DualShotguns` and `Tome` are wiki classes named by spec 09 §3.1 (CONTRACTS.md, T01a).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum WeaponClass {
    Rifle,
    Shotgun,
    Sniper,
    Bow,
    Launcher,
    Speargun,
    Pistol,
    DualPistols,
    Thrown,
    ShotgunSidearm,
    Melee,
    ArchGun,
    ArchMelee,
    ExaltedRanged,
    ExaltedMelee,
    CompanionWeapon,
    Amp,
    Crossbow,
    DualShotguns,
    Tome,
}

bitflags::bitflags! {
    /// Wiki `Type` and `Mods/itemCompatibility` categories of a weapon. Bit meanings are data: `Rules.categories`
    /// maps bit index → name. Readable form: the list of set bit indices (`[0, 5, 17]`); compact form: the bits.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct CategorySet: u64 {
        const _ = !0;
    }
}

impl CategorySet {
    /// The set with exactly bit `i` (i < 64).
    pub fn bit(i: u8) -> Self {
        CategorySet::from_bits_retain(1u64 << (i & 63))
    }
    /// Indices of the set bits, ascending.
    pub fn indices(self) -> Vec<u8> {
        (0u8..64).filter(|i| self.bits() & (1u64 << i) != 0).collect()
    }
}

impl Serialize for CategorySet {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            self.indices().serialize(s)
        } else {
            self.bits().serialize(s)
        }
    }
}

impl<'de> Deserialize<'de> for CategorySet {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if d.is_human_readable() {
            let idx: Vec<u8> = Deserialize::deserialize(d)?;
            let mut out = CategorySet::empty();
            for i in idx {
                if i >= 64 {
                    return Err(serde::de::Error::custom(format!("category bit {i} out of range 0..64")));
                }
                out |= CategorySet::bit(i);
            }
            Ok(out)
        } else {
            Ok(CategorySet::from_bits_retain(u64::deserialize(d)?))
        }
    }
}

#[cfg(feature = "ts")]
impl ts_rs::TS for CategorySet {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;
    fn name(_: &ts_rs::Config) -> String {
        "Array<number>".to_owned()
    }
    fn inline(cfg: &ts_rs::Config) -> String {
        <Self as ts_rs::TS>::name(cfg)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for CategorySet {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "CategorySet".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({ "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 63 } })
    }
}

bitflags::bitflags! {
    /// Weapon traits. `PSEUDO_EXALTED` (`POWER_WEAPON_LITE`, 09 E-9.1) and `NECRAMECH` (Arquebex, Ironbride:
    /// 09 E-9.30) are additions for spec 09 (CONTRACTS.md, T01a).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct WeaponTraits: u32 {
        const PRIME = 1 << 0;
        const KUVA = 1 << 1;
        const TENET = 1 << 2;
        const CODA = 1 << 3;
        const INCARNON = 1 << 4;
        const KITGUN = 1 << 5;
        const ZAW = 1 << 6;
        const EXALTED = 1 << 7;
        const BATTERY = 1 << 8;
        const BEAM = 1 << 9;
        const SENTINEL = 1 << 10;
        const STRENGTH_SCALED = 1 << 11;
        const INFINITE_AMMO = 1 << 12;
        const PSEUDO_EXALTED = 1 << 13;
        const NECRAMECH = 1 << 14;
    }
}
flags_serde!(WeaponTraits, u32, Lower);

impl WeaponTraits {
    /// Adversary weapon (09 E-9.70: Kuva Lich, Tenet, Technocyte Coda).
    pub fn is_adversary(self) -> bool {
        self.intersects(WeaponTraits::KUVA | WeaponTraits::TENET | WeaponTraits::CODA)
    }
}

/// Trigger class after the 06 §4.2 normalisation of the 33 wiki strings. `Duplex` and `Active` schedule as SEMI,
/// `AutoBurst` as BURST (06 §4.2 table); they are kept distinct for compatibility rules (09 E-9.6: Duplex is not
/// Semi-Auto) and display.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Trigger {
    Auto,
    AutoSpool,
    Semi,
    Burst,
    MagBurst,
    Charge,
    Held,
    Duplex,
    Active,
    AutoBurst,
    Melee,
}

/// Shot type of an attack (wiki `ShotType`, plus beams, discharges and melee strikes).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ShotType {
    HitScan,
    Projectile,
    Aoe,
    Beam,
    Thrown,
    Discharge,
    Melee,
}

bitflags::bitflags! {
    /// Roles of an attack (derived by the pipeline, pipeline.md §5.3). Readable form: lowercase names.
    /// `WALL`, `EXPLOSION` (07 §9.4), `GAUGE_GATED` (06 E-6.14a), `RETURNING` (06 §6.4), `ZOOM_ONLY` and
    /// `HEADSHOT_ONLY` (children gated by zoom or headshots, 06 §9.1-9.2) are additions for specs 06 and 07
    /// (CONTRACTS.md, T01a).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct AttackRoles: u32 {
        const NORMAL = 1 << 0;
        const ALT = 1 << 1;
        const CHARGED = 1 << 2;
        const UNCHARGED = 1 << 3;
        const INCARNON_FORM = 1 << 4;
        const RADIAL = 1 << 5;
        const CHILD = 1 << 6;
        const BEAM = 1 << 7;
        const HEAVY = 1 << 8;
        const SLAM = 1 << 9;
        const HEAVY_SLAM = 1 << 10;
        const SLIDE = 1 << 11;
        const AERIAL = 1 << 12;
        const THROWN = 1 << 13;
        const FINISHER = 1 << 14;
        const RELOAD_TRIGGERED = 1 << 15;
        const WHOLE_MAGAZINE = 1 << 16;
        const MELEE_PROJECTILE = 1 << 17;
        const WALL = 1 << 18;
        const EXPLOSION = 1 << 19;
        const GAUGE_GATED = 1 << 20;
        const RETURNING = 1 << 21;
        const ZOOM_ONLY = 1 << 22;
        const HEADSHOT_ONLY = 1 << 23;
    }
}
flags_serde!(AttackRoles, u32, Lower);

/// Headshot rule of an attack (02 §4.5): normal, 1x-headshot weapons, radial (no body part).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum HeadshotRule {
    #[default]
    Normal,
    OneX,
    NoBodyPart,
}

/// Mod and slot polarities (09 §6.1). The wiki slot string `Aura` is the "Any" symbol and deserializes to `Any`
/// (alias `"aura"`); "no polarity" is `Option::None` at the use site (CONTRACTS.md, T01a).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Polarity {
    Madurai,
    Vazarin,
    Naramon,
    Zenurik,
    Unairu,
    Penjaga,
    Umbra,
    #[serde(alias = "aura")]
    Any,
}

/// Archon shard colours (09 §9).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ShardColor {
    Crimson,
    Azure,
    Amber,
    Violet,
    Emerald,
    Topaz,
}

/// Arcane class from `M:Arcane.Type` (09 E-9.30). `Akimbo` is listed by 09 E-9.30 (secondary family; CONTRACTS.md,
/// T01a).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ArcaneClass {
    Primary,
    Secondary,
    Melee,
    Kitgun,
    Zaw,
    Bow,
    Shotgun,
    Sniper,
    Pistol,
    ArchGun,
    Frame,
    Akimbo,
}

/// Item rarity.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Legendary,
    Riven,
    Peculiar,
}

/// Display colour class of a crit tier (00 §2.2): 0 none, 1 yellow, 2 orange, 3+ red.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CritTier {
    None,
    Yellow,
    Orange,
    Red,
}

impl CritTier {
    pub fn of(k: u32) -> CritTier {
        match k {
            0 => CritTier::None,
            1 => CritTier::Yellow,
            2 => CritTier::Orange,
            _ => CritTier::Red,
        }
    }
}

/// Crit-tier distributions and histograms (EV `crit_tiers`, MC `crit_hist`) are indexed by tier k = 0 ..= 7 with
/// index 7 meaning "7 or more" (8 entries); [`CritTier`] is only the UI's colour class of an index.
pub const N_TIER_BINS: usize = 8;

/// Combo types of 07 E-7.0b, in the order of that equation (array index of `MeleeTables.combos`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ComboType {
    Neutral,
    Forward,
    Block,
    ForwardBlock,
    Slide,
    Aerial,
    Wall,
    Finisher,
    Slam,
    Heavy,
    HeavySlam,
}

/// Number of combo types (07 E-7.0b).
pub const N_COMBO_TYPES: usize = 11;

impl ComboType {
    pub const ALL: [ComboType; N_COMBO_TYPES] = [
        ComboType::Neutral,
        ComboType::Forward,
        ComboType::Block,
        ComboType::ForwardBlock,
        ComboType::Slide,
        ComboType::Aerial,
        ComboType::Wall,
        ComboType::Finisher,
        ComboType::Slam,
        ComboType::Heavy,
        ComboType::HeavySlam,
    ];
    pub const fn index(self) -> usize {
        self as usize
    }
    /// The combo key slug (`"forward_block"`), the serde name.
    pub const fn slug(self) -> &'static str {
        match self {
            ComboType::Neutral => "neutral",
            ComboType::Forward => "forward",
            ComboType::Block => "block",
            ComboType::ForwardBlock => "forward_block",
            ComboType::Slide => "slide",
            ComboType::Aerial => "aerial",
            ComboType::Wall => "wall",
            ComboType::Finisher => "finisher",
            ComboType::Slam => "slam",
            ComboType::Heavy => "heavy",
            ComboType::HeavySlam => "heavy_slam",
        }
    }
    /// Parses a combo key slug.
    pub fn from_slug(s: &str) -> Option<ComboType> {
        ComboType::ALL.iter().copied().find(|c| c.slug() == s)
    }
    /// The four stance-specific combos (07 §4.2); the others are shared by the weapon class.
    pub fn is_stance_specific(self) -> bool {
        matches!(self, ComboType::Neutral | ComboType::Forward | ComboType::Block | ComboType::ForwardBlock)
    }
}

/// Strike type of a stance entry, `Types[e]` (07 §4.1): `""` = Plain, `"360"` = Spin360.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum StrikeType {
    #[default]
    Plain,
    Spin360,
    Sweep,
    Thrust,
    Ranged,
    Slam,
}

impl StrikeType {
    /// Parses the wiki `Types[e]` string (`""`, `"360"`, `"Sweep"`, `"Thrust"`, `"Ranged"`, `"Slam"`).
    pub fn from_wiki(s: &str) -> Option<StrikeType> {
        Some(match s {
            "" => StrikeType::Plain,
            "360" => StrikeType::Spin360,
            "Sweep" => StrikeType::Sweep,
            "Thrust" => StrikeType::Thrust,
            "Ranged" => StrikeType::Ranged,
            "Slam" => StrikeType::Slam,
            _ => return None,
        })
    }
}

macro_rules! melee_classes {
    ($($v:ident => $wiki:literal),+ $(,)?) => {
        /// Wiki `Class` of melee weapons: the 28 stanceless classes of 07 table 9-A (Bayonet included), the data
        /// class `"Exalted Weapon"` (07 E-7.0b) and archmelee weapons (09 E-9.1; 07 OQ-737).
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(rename_all = "snake_case")]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub enum MeleeClass { $($v),+ }

        impl MeleeClass {
            pub const ALL: &'static [MeleeClass] = &[$(MeleeClass::$v),+];
            /// The wiki `Class` string (`"Two-Handed Nikana"`).
            pub const fn wiki_name(self) -> &'static str {
                match self { $(MeleeClass::$v => $wiki),+ }
            }
            /// Parses a wiki `Class` string (exact match; `"Archmelee"` is accepted for `ArchMelee`).
            pub fn from_wiki(s: &str) -> Option<MeleeClass> {
                match s {
                    $($wiki => Some(MeleeClass::$v),)+
                    "Archmelee" => Some(MeleeClass::ArchMelee),
                    _ => None,
                }
            }
        }
    };
}

melee_classes! {
    AssaultSaw => "Assault Saw",
    BladeAndWhip => "Blade and Whip",
    Claws => "Claws",
    Dagger => "Dagger",
    DualDaggers => "Dual Daggers",
    DualNikanas => "Dual Nikanas",
    DualSwords => "Dual Swords",
    Fist => "Fist",
    Glaive => "Glaive",
    Gunblade => "Gunblade",
    Hammer => "Hammer",
    HeavyBlade => "Heavy Blade",
    HeavyScythe => "Heavy Scythe",
    Machete => "Machete",
    Nikana => "Nikana",
    Nunchaku => "Nunchaku",
    Polearm => "Polearm",
    Rapier => "Rapier",
    Scythe => "Scythe",
    Sparring => "Sparring",
    Staff => "Staff",
    Sword => "Sword",
    SwordAndShield => "Sword and Shield",
    Tonfa => "Tonfa",
    TwoHandedNikana => "Two-Handed Nikana",
    Warfan => "Warfan",
    Whip => "Whip",
    Bayonet => "Bayonet",
    ExaltedWeapon => "Exalted Weapon",
    ArchMelee => "Arch-Melee",
}

/// Riven class (09 E-9.50; `riv_class`). "none" is `Option::None` at the use site.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RivenClass {
    Rifle,
    Shotgun,
    Pistol,
    Kitgun,
    Melee,
    Zaw,
    Archgun,
    Companion,
}

/// Coefficient column of the riven value table (04 §8.3; 09 E-9.50: Kitgun → Pistol, Zaw → Melee, Companion → the
/// column of the weapon class).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RivenColumn {
    Rifle,
    Shotgun,
    Pistol,
    Archgun,
    Melee,
}

/// Kind of a melee input (07 §7.5 `input_start`): the inputs that count for Melee Animosity and Tennokai.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum InputKind {
    Light,
    Slide,
    Aerial,
    Wall,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_order() {
        assert_eq!(DamageType::COR.index(), 12);
        assert_eq!(DamageType::FIN.index(), 17);
        assert_eq!(StatusType::LIF.index(), 17);
        assert_eq!(DamageType::HEA.status(), Some(StatusType::HEA));
        assert_eq!(DamageType::TRU.status(), None);
        assert_eq!(DamageType::combine(DamageType::TOX, DamageType::ELE), Some(DamageType::COR));
        assert_eq!(ComboType::HeavySlam.index(), N_COMBO_TYPES - 1);
        for c in ComboType::ALL {
            assert_eq!(ComboType::from_slug(c.slug()), Some(c));
            let json = serde_json::to_string(&c).unwrap();
            assert_eq!(json, format!("\"{}\"", c.slug()));
        }
        for m in MeleeClass::ALL {
            assert_eq!(MeleeClass::from_wiki(m.wiki_name()), Some(*m));
        }
    }

    #[test]
    fn flag_forms() {
        let roles = AttackRoles::HEAVY | AttackRoles::HEAVY_SLAM;
        let json = serde_json::to_string(&roles).unwrap();
        assert_eq!(json, r#"["heavy","heavy_slam"]"#);
        assert_eq!(serde_json::from_str::<AttackRoles>(&json).unwrap(), roles);
        assert!(serde_json::from_str::<AttackRoles>(r#"["heavvy"]"#).is_err());
        let bytes = postcard::to_stdvec(&roles).unwrap();
        assert_eq!(postcard::from_bytes::<AttackRoles>(&bytes).unwrap(), roles);
        let st: StatusSet = [StatusType::SLA, StatusType::VIR].into_iter().collect();
        assert_eq!(serde_json::to_string(&st).unwrap(), r#"["SLA","VIR"]"#);
        let cats = CategorySet::bit(3) | CategorySet::bit(40);
        assert_eq!(serde_json::to_string(&cats).unwrap(), "[3,40]");
        assert_eq!(serde_json::from_str::<CategorySet>("[3,40]").unwrap(), cats);
        assert_eq!(LayerSet::default(), LayerSet::all());
        assert_eq!(serde_json::from_str::<Polarity>("\"aura\"").unwrap(), Polarity::Any);
    }
}
