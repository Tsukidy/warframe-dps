Status: reviewed (architecture)

# Domain model and effect encoding

All types in this document live in the crate `wfd-model` unless a section says otherwise. `wfd-model` has no logic
beyond construction, validation helpers and (de)serialization. It is the contract every other task codes against;
changes follow the protocol of `work-breakdown.md` §2. Spec identifiers are used verbatim (`docs/mechanics/`).

Contents:

1. Conventions
2. Identifiers
3. Enumerations
4. Weapons, attacks, stances, modular parts
5. Enemies and factions
6. Items, sets, Incarnon lines, rivens
7. Rules tables
8. Bundle
9. Builds, scenarios, toggles and conditions
10. The stat registry
11. The effect DSL
12. Condition classes and the T-02 compile table
13. Element ordering
14. Multiplier groups
15. Curated TOML format and the effect-status lattice
16. Encoded examples
17. Coverage matrix: spec mechanics to encoding
18. Lowering: from effects to compiled contributions

---

## 1. Conventions

| Rule | Definition |
|---|---|
| Units | Inside the model every chance, bonus and percentage is a fraction (spec U-1): 0.30 means 30 %. Percent appears only in the UI and in upstream text. Time in seconds, damage in HP, rates per second. |
| Bonus vs factor | A bonus is an additive fraction summed inside a bucket; a factor is `1 + bonus` (spec U-2). Multiplier groups are bonuses inside the group and factors across groups. |
| Arrays | Enumerations index arrays in spec 00 §2.2 order. `DamageVec = [f64; 18]`. |
| Serde | Every public type derives `Serialize, Deserialize, Debug, Clone, PartialEq`; API and bundle types also derive `ts_rs::TS` (feature `ts`) and `schemars::JsonSchema` (feature `schema`). Enum JSON form is `snake_case` strings unless stated. Unknown fields are rejected (`deny_unknown_fields`) in curated TOML and in API requests. |
| Integers in TypeScript | ts-rs maps `u64`/`i64`/`u128` to `bigint`, but the JSON carries numbers. Every `u64` field of an API or document type has `#[ts(type = "number")]` and a validated range ≤ 2^53 − 1 (seeds, counts, node budgets, `timing_us`); `u128` (`CompatFlags`) never appears in an API type (flag names instead); a CI test parses the generated `.ts` files and fails on `bigint` |
| Hot-path types | Engine-internal compiled types (`CompiledScenario`, `ResolvedAttack`, `StatVec`) live in `wfd-engine`, not here. |
| Determinism | No `HashMap` in any type that is serialized or iterated for results; use `BTreeMap` or sorted `Vec`. |

---

## 2. Identifiers

```rust
// wfd-model/src/ids.rs
/// Permanent identity of anything in the bundle. DE items use their DE path ("/Lotus/Weapons/Tenno/Pistol/AutoPistol").
/// Things DE does not export use our namespace "wfd:<kind>/<slug>", e.g. "wfd:buff/roar",
/// "wfd:enemy/grineer/lancer", "wfd:perk/furis/haven_foray", "wfd:custom/base", "wfd:shard/violet/primary_electricity".
pub struct UniqueName(pub Box<str>);

/// Public key. Keys < LOCAL_KEY_BASE are global: assigned append-only by data/generated/stable_keys.json, never reused,
/// never renumbered. Keys ≥ LOCAL_KEY_BASE are session-local: assigned by an in-app refresh (pipeline.md §9.2) to
/// uniqueNames the embedded bundle does not know; they differ between machines, so every document that contains one
/// carries its uniqueName (`CalcInput.x`, AD-18). Used by the HTTP API, share codes and the browser's local storage.
pub struct StableKey(pub u32);
pub const LOCAL_KEY_BASE: u32 = 1 << 31;

/// Dense indices into the vectors of one loaded bundle. Never persisted, never sent over HTTP.
pub struct WeaponId(pub u32);
pub struct ItemId(pub u32);
pub struct EnemyId(pub u32);
pub struct AttackIdx(pub u8);
pub struct FactionId(pub u8);      // index into Rules.factions (data-driven: new factions need no code)
pub struct SetId(pub u16);
pub struct StanceId(pub u16);
pub struct ComboTableId(pub u16);  // index into Bundle.combo_tables (stance, stanceless class or unique-to-weapon table)
pub struct FrameId(pub u16);       // index into Bundle.frames
pub struct FamilyId(pub u16);      // Incarnon line / weapon family (Furis, MK1-Furis share one)
pub struct MultGroupId(pub u8);    // index into Rules.mult_groups
pub struct TagId(pub u16);         // compatibility tag (PROJECTILE, AOE, NIKANAS_STANCE …)

/// Identity of an attack that survives data updates: the slug of the wiki `AttackName` ("normal_attack",
/// "incarnon_form", "charged_shot", "heavy_slam_attack") plus its index. Decoders match the slug first and
/// fall back to the index, reporting a warning when only the index matched.
pub struct AttackKey { pub slug: Box<str>, pub idx: u8 }

pub struct ParamKey(pub Box<str>);   // item parameter ("element", "value")
/// "<UniqueName>#<effect id>", e.g. "/Lotus/Upgrades/Mods/Rifle/WeaponStatusChanceSPMod#co".
/// Used for per-effect condition overrides, traces and the MC mechanic id "eff/<effect key>" (FNV-1a 32-bit at L0,
/// spec 08 E-8.16; engine.md §8.2).
pub struct EffectKey(pub Box<str>);
pub struct NameKey(pub Box<str>);    // key into NameTable (the UniqueName of the thing named)
pub struct ComboKey(pub Box<str>);   // combo type slug of 07 E-7.0b: "neutral", "forward", "block", "forward_block" …
```

Enemies have no DE export (`DS:pe` TL;DR); their `UniqueName` is synthetic: `wfd:enemy/<wiki sub-module>/<slug of
the wiki Name>`, so a renamed wiki entry keeps its key only through `data/curated/aliases/enemies.toml`.

---

## 3. Enumerations

```rust
// wfd-model/src/enums.rs   — order is spec 00 §2.2 and is used for array indexing
#[repr(u8)] pub enum DamageType { IMP, PUN, SLA, HEA, COL, ELE, TOX, BLA, RAD, GAS, MAG, VIR, COR, VOI, TAU, TRU, CIN, FIN }
#[repr(u8)] pub enum StatusType { IMP, PUN, SLA, HEA, COL, ELE, TOX, BLA, RAD, GAS, MAG, VIR, COR, VOI, TAU, MIC, KND, LIF }
pub const N_DT: usize = 18; pub const N_ST: usize = 18;
pub type DamageVec = [f64; N_DT];
bitflags! { pub struct StatusSet: u32 { /* one bit per StatusType */ } }

pub enum Layer { OG, SH, HP }
bitflags! { pub struct LayerSet: u8 { OG SH HP } }                 // default: all three
bitflags! { pub struct DamageTypeSet: u32 { /* one bit per DamageType */ } }
pub enum View { Health, Armor, Shield, Overguard, Layered }        // spec 00 §2.2 layer views
pub enum WeaponSlot { Primary, Secondary, Melee, ArchGun, ArchMelee, Exalted, Companion }
pub enum WeaponClass {                                               // for scopes and riven classes
    Rifle, Shotgun, Sniper, Bow, Launcher, Speargun, Pistol, DualPistols, Thrown, ShotgunSidearm,
    Melee /* subclass in MeleeData.class */, ArchGun, ArchMelee, ExaltedRanged, ExaltedMelee, CompanionWeapon, Amp,
}
bitflags! { pub struct CategorySet: u64 { /* wiki Type + Mods/itemCompatibility categories; table in rules */ } }
bitflags! { pub struct WeaponTraits: u32 { PRIME KUVA TENET CODA INCARNON KITGUN ZAW EXALTED BATTERY BEAM SENTINEL
                                            STRENGTH_SCALED INFINITE_AMMO } }

/// Trigger class after 06 §4.2 normalisation of the 33 wiki strings. Duplex is its own class DUPLEX (two shots per
/// trigger sequence, 06 E-6.12; D-608); Active schedules as SEMI, AutoBurst as BURST.
pub enum Trigger { Auto, AutoSpool, Semi, Burst, MagBurst, Charge, Held, Duplex, Active, AutoBurst, Melee }
pub enum ShotType { HitScan, Projectile, Aoe, Beam, Thrown, Discharge, Melee }
bitflags! { pub struct AttackRoles: u32 {
    NORMAL ALT CHARGED UNCHARGED INCARNON_FORM RADIAL CHILD BEAM HEAVY SLAM HEAVY_SLAM SLIDE AERIAL THROWN
    FINISHER RELOAD_TRIGGERED WHOLE_MAGAZINE MELEE_PROJECTILE /* T01a: WALL EXPLOSION GAUGE_GATED RETURNING
    ZOOM_ONLY HEADSHOT_ONLY */
    SPEARGUN_THROW      /* 02 §4.10 speargun_throw: MS = 1 (E-2.1) */
    RADIAL_PARENT_PART  /* 02 §4.10: radial child evaluated on its parent instance's part record (E-2.8, E-2.10) */
    TORSO_ONLY          /* 02 §4.10: T-03 ignored, every direct instance hits the body part (E-2.10; Peacemaker) */
    DOT_DETONATE        /* 03 E-3.14 DoT detonation (Harmony heavy attacks) */ } }
pub enum HeadshotRule { Normal, OneX /* 1x-headshot weapons */, NoBodyPart /* radial */ }
pub enum Polarity { Madurai, Vazarin, Naramon, Zenurik, Unairu, Penjaga, Umbra, Aura, Any }
pub enum ShardColor { Crimson, Azure, Amber, Violet, Emerald, Topaz }
pub enum ArcaneClass { Primary, Secondary, Melee, Kitgun, Zaw, Bow, Shotgun, Sniper, Pistol, ArchGun, Frame }
pub enum Rarity { Common, Uncommon, Rare, Legendary, Riven, Peculiar }
pub enum CritTier { None, Yellow, Orange, Red /* 3+ */ }    // display colour class only
/// Crit-tier distributions and histograms (EV `crit_tiers`, MC `crit_hist`) are indexed by tier k = 0 ..= 7 with
/// index 7 meaning "7 or more" (8 entries); CritTier is only the UI's colour class of an index.
pub const N_TIER_BINS: usize = 8;
pub enum ComboType { Neutral, Forward, Block, ForwardBlock, Slide, Aerial, Wall, Finisher, Slam, Heavy, HeavySlam } // 07 E-7.0b
pub enum StrikeType { Plain, Spin360, Sweep, Thrust, Ranged, Slam }   // stance `Types[e]` (07 §4.1); Plain = ""
pub enum MeleeClass { /* wiki `Class` of melee weapons: AssaultSaw, BladeAndWhip, Claws, Dagger, … , Whip (07 table 9-A) */ }
```

---

## 4. Weapons, attacks, stances, modular parts

```rust
// wfd-model/src/weapon.rs
pub struct Weapon {
    pub id: WeaponId, pub key: StableKey, pub unique_name: UniqueName, pub name: NameKey,
    pub slot: WeaponSlot, pub class: WeaponClass, pub categories: CategorySet,
    pub compat_tags: SmallVec<[TagId; 4]>,     // PROJECTILE, AOE, BEAM, BATTERY, augment and stance-class tags
    pub traits: WeaponTraits, pub family: Option<FamilyId>,
    pub trigger: Trigger,
    pub magazine: Option<MagazineSpec>,        // None = no magazine (melee)
    pub reload: ReloadSpec,
    pub attacks: SmallVec<[Attack; 3]>,
    pub groups: SmallVec<[AttackGroup; 1]>,    // "combine attack effects" groups (T-22, CD-12) and parent+child sums
    pub zoom: SmallVec<[ZoomLevel; 3]>,        // effects gated by `while_aiming`; level chosen by toggles.zoom_level (04 D-406)
    pub melee: Option<MeleeData>,
    pub incarnon: Option<IncarnonDef>,
    pub modular: Option<ModularRef>,           // kitgun chamber / zaw strike: needs parts in the build
    pub progenitor: Option<ProgenitorSpec>,    // allowed elements, value range [0.25, 0.60] (T-24)
    pub exalted: Option<ExaltedScaling>,       // Strength scaling of base damage (E-1 m_str, 01 §4.3)
    pub passives: Vec<EffectDef>,              // innate effects in the DSL (Cedo CO, Sagek, IPS amplifiers …)
    pub polarities: SmallVec<[Polarity; 8]>, pub exilus_polarity: Option<Polarity>, pub stance_polarity: Option<Polarity>,
    pub max_rank: u8, pub mastery: u8, pub release: Option<Box<str>>,
    pub image: Option<Box<str>>,               // PE `textureLocation` (pipeline.md §3.2); served by key through /api/img/{key}
    pub display: WeaponDisplay,                // accuracy, noise, ammo max, flight speed … (weapon panel only)
    pub provenance: Provenance,                // source + revision per field group; overridden fields listed
}
pub struct MagazineSpec { pub size: f64, pub ammo_max: Option<f64>, pub infinite: bool,
                          pub battery: Option<Battery> /* delay, delay_empty, rate */ }
pub enum ReloadSpec { Standard { time: f64 }, PerShell { start_end: f64, per_shell: f64 }, Regenerate,
                      None /* Epitaph-style no-reload */ }

pub struct Attack {
    pub key: AttackKey, pub name: NameKey, pub roles: AttackRoles, pub shot: ShotType,
    pub parent: Option<AttackIdx>,             // radial child of a parent hit (F:weapon §10.1; 06 §9)
    pub base: DamageVec,                       // B0_t per pellet
    pub cc: f64, pub cd: f64, pub sc: f64, pub multishot: f64,
    pub fire: FireSpec,
    pub ammo_cost: f64,
    pub magazine_override: Option<f64>,        // Incarnon form charges (06 §6.6)
    pub falloff: Option<Falloff>,              // { start, end, min_factor } (06 §8)
    pub radius: Option<f64>, pub punch_through: f64, pub projectile_speed: Option<f64>,
    pub forced: SmallVec<[StatusType; 2]>,     // forced procs per pellet (02 §4.7.2)
    pub forced_cond: Vec<ForcedProcRule>,      // conditional or corrected forced procs (02 §4.10 [[attack.forced]])
    pub headshot: HeadshotRule,
    pub hs_unique: Option<f64>,                // M_hsu unique headshot factor (02 §4.10; Cernos Prime 1.5)
    pub co: CoSpec,                            // 04 §2.4-2.14, ingested catalogue row or the default rule
    pub sniper_combo: Option<SniperComboSpec>, // 06 §7.2
    pub is_heavy: Option<bool>,                // explicit heavy flag (T-09 auto mode)
    pub explosion_delay: Option<f64>, pub embed_delay: Option<f64>,     // child delays (06 §9.3)
    pub lingering: Option<Lingering>,          // { effect_duration D_e, tick_rate FR_child } (06 §9.4, E-6.25)
    pub synthesized: Option<Synthesized>,      // HeavyFrom { normal: AttackIdx } | SlideFrom { normal } (07 §9.1, §9.3)
                                               // | LightWavesFrom { normal } | HeavyWavesFrom { heavy } (07 E-7.43:
                                               // attack ids LIGHT_WAVES = "Light Waves", HEAVY_WAVES = "Heavy Waves")
    pub compression_eff: Option<f64>,          // Primary Compression effectiveness (04 §3.1), imported table
    pub passives: Vec<EffectDef>,              // attack-level innate effects
}
pub struct FireSpec {
    pub trigger: Trigger, pub rate: f64,       // listed fire rate (attacks/s) or attack speed for melee
    pub charge_time: Option<f64>, pub partial_charge: Option<PartialCharge>,
    pub burst: Option<Burst>,                  // { count, delay }
    pub spool: Option<Spool>,                  // { start_frac, shots | cadence } (06 §7.3, hand table)
    pub held: Option<Held>,                    // { tick_rate, ramp_start (default 0.2), ramp_time } (06 §7.1)
    pub semi_cap: Option<f64>,                 // override of the semi-auto cap (06 §5.2)
    pub kohm: Option<KohmSpec>, pub peacemaker: Option<PeacemakerSpec>,
}
pub struct CoSpec {
    pub behaviour: CoBehaviour,                // Adding | Multiplying | None (E-11)
    pub ratio: f64,                            // ρ_co
    pub co_ref: CoRef,                         // SelfAttack | Parent (04 §2.6)
    pub fixed_abs: Option<f64>,                // B_co^abs
    pub flags: CoFlags,                        // DIRECT_TARGET_ONLY, NO_COMBO, NO_DOT_CO, DOT_RECALC_PER_TICK, …
    pub ms_proj_factor: Option<f64>,           // κ_ms
    pub off_with: SmallVec<[ItemId; 1]>,       // CO is 0 while one of these augments is equipped
    pub row: Option<u16>,                      // 04 §2.14 row id, None = default rule (04 §2.5)
}
/// 02 §4.10 `[[attack.forced]]`: procs delivered when `condition` holds; `replaces_data` drops the data's
/// unconditional forced proc of the same type; `rule` names a special rule of 02 §4.7.2.
pub struct ForcedProcRule { pub condition: ForcedCondition, pub procs: SmallVec<[ForcedCount; 1]>,
                            pub replaces_data: bool, pub rule: Option<ForcedRule /* Cedo | Evensong | Microwave */> }
pub enum ForcedCondition { Always, DistanceMax(f64) /* m, T-27 */, DirectHit, FullCharge, LastShot, Zoom /* T-38 */ }
pub struct ForcedCount { pub status: StatusType /* JSON "type" */, pub count: u8 }
pub struct AttackGroup { pub members: SmallVec<[AttackIdx; 3]>, pub combinable: bool /* T-22 */, pub summed: bool }
pub struct ZoomLevel { pub name: NameKey, pub effects: Vec<EffectDef> }

pub struct MeleeData {                         // spec 07 §4.3; everything per weapon, filled by the pipeline
    pub class: MeleeClass,                     // wiki `Class`: selects the stanceless table and table 9-A
    pub family: Option<Box<str>>,              // wiki `Family`: UniqueToWeapon matching (E-7.0b)
    pub fr_base: f64,                          // FR_w after the k/60 snap (E-7.1)
    pub wind_up: f64,                          // W0 (`WindUp`)
    pub combo_dur: Option<f64>,                // D0_cb (`ComboDur`); None = no expiry (Xoris, 07 §6.4)
    pub heavy_swing: Option<f64>,              // T_sw from data, else the E-7.5 fallback at evaluation
    pub follow_through: f64, pub range: f64,   // display and multi-target only
    pub stanceless: ComboTableId,              // the stanceless table of `class` (exalted classes: their stance table)
    pub unique: Option<ComboTableId>,          // UniqueToWeapon table matched by name or Family (E-7.0b)
    pub heavy: Option<HeavyDef>,               // synthesized heavy attack (07 §9.1); None for classes without heavies
    pub triggers: SmallVec<[MeleeTriggerRow; 2]>, // table 9-C rows of this weapon (inputs, strikes or heavies spawning attacks)
    pub specials: ComboSpecials,               // k_tier, M_cap, M_bcap, D0_cb override, innate IC (07 §6; D-063)
    pub ability_combo: bool,                   // one of the four ability-combo exalted weapons (E-7.44, D-701)
}
pub struct HeavyDef {                          // E-7.40, E-7.41: hit lists computed by the pipeline, damage by the engine
    pub attack: AttackIdx,                     // the synthesized Attack (roles HEAVY, synthesized HeavyFrom { normal })
    pub hits: [SmallVec<[HeavyStrike; 4]>; 2], // heavy h = 1, 2: H_j (κ_w applied), procs, spawned 9-C attacks
    pub kappa: f64,                            // κ_w (table 9-B, 1 unless listed)
    pub source: HeavySource,                   // UniqueOverride | ClassTable { sigma: [f64; 2] } | Breakdown9A
}
pub struct HeavyStrike { pub mult: f64, pub procs: SmallVec<[StatusType; 2]>, pub spawns: Option<AttackIdx> }

pub struct ComboTable {                        // one normalised Module:Stances/data table (07 §4.1)
    pub id: ComboTableId, pub source: ComboSource, // Stance(StanceId) | Stanceless(MeleeClass) | Unique { name: Box<str> }
    pub combos: SmallVec<[Combo; 8]>,          // at most one per ComboType
}
pub struct Combo {
    pub kind: ComboType, pub key: ComboKey, pub name: NameKey,
    pub duration: Option<f64>,                 // D_C at AS 1.0; None → E-7.3 fallback
    pub from_weapon: bool,                     // copied by addSharedCombos (`FromWeapon`); skipped by E-7.0b rules 1-2
    pub n_inputs: u8,
    pub strikes: Vec<Strike>,                  // E-7.0 step 6: entries expanded by Hits[e], inputs concatenated
}
pub struct Strike {
    pub input: u8, pub entry: u8,              // input index u and entry index e of the combo
    pub mult: f64,                             // S_j = Dmg[e]/100
    pub ty: StrikeType,                        // Types[e]: Slam strikes are radial parts, Ranged strikes spawn 9-C attacks
    pub phys: [f64; 3],                        // S^phys_t = factor − 1 for SLA, IMP, PUN (E-7.0 step 5)
    pub procs: SmallVec<[StatusType; 2]>,      // forced procs (E-7.0 steps 2-4, 03 table 4.2)
    pub status_mult: f64,                      // stored, not applied (OQ-714)
}
pub struct Stance { pub id: StanceId, pub item: ItemId, pub class: MeleeClass, pub table: ComboTableId }
/// Resolved per scenario by the engine (E-7.0b): for each ComboType the table that supplies it.
pub struct MeleeTables<'b> { pub combos: [Option<&'b Combo>; 11], pub heavy: Option<&'b HeavyDef>,
                              pub rules: &'b MeleeRules, pub specials: &'b ComboSpecials }
pub struct MeleeRules {                        // data/curated/rules/melee/*.toml (07 §4.4)
    pub heavy_class: Vec<HeavyClassRow>,       // table 9-A: class, T_{class,1}, T_{class,2}, breakdowns, typical wind-up
    pub heavy_kappa: Vec<(UniqueName, f64)>,   // table 9-B members (checked against the E-7.41 computation)
    pub attack_triggers: Vec<MeleeTriggerRow>, // table 9-C
    pub stance_co: Vec<StanceCoRow>,           // table 9-D: r_in per stance, combo and input (04 §2.12, 07 §7.6)
    pub finishers: Vec<FinisherRow>,           // table 9-E
    pub combo_specials: Vec<(UniqueName, ComboSpecials)>,
}
pub struct ModularTables { pub chambers: Vec<KitgunChamber>, pub grips: Vec<ModularPart>, pub loaders: Vec<ModularPart>,
                           pub strikes: Vec<ZawStrike>, pub links: Vec<ModularPart> }
pub struct ModularRef { pub kind: ModularKind /* Kitgun | Zaw */, pub base: u16 /* chamber or strike index */ }
pub struct ProgenitorSpec { pub elements: SmallVec<[DamageType; 7]>, pub min: f64, pub max: f64, pub default: f64 }
pub enum ExaltedScaling { BaseByStrength, Partial(f64), None }
```

Kitguns and zaws are assembled at evaluation time from parts (`wfd-engine::assemble`, 06 §11.1, 07); the
pre-expanded wiki kitgun records are used only as a pipeline cross-check (`DS:wiki` §3).

---

## 5. Enemies and factions

```rust
// wfd-model/src/enemy.rs  (record of 05 §4.1)
pub struct Enemy {
    pub id: EnemyId, pub key: StableKey, pub unique_name: UniqueName, pub name: NameKey,
    pub faction: FactionId, pub base_level: u16,
    pub health: f64, pub shield: f64, pub armor: f64, pub overguard: f64,
    pub eximus: EximusData,                    // { health, shield, armor, overguard, off, og,
                                               //   kinds: Vec<EximusKind> /* eximus_kinds of 05 §4.1, offered for T-23 */ }
    pub sp_health_bonus: f64,                  // D-055
    pub parts: SmallVec<[BodyPart; 4]>,        // 05 §4.3 part record, below
    pub immune: StatusSet, pub caps: [Option<u8>; N_ST],
    pub type_mod_override: Option<DamageVec>, pub resist: Option<DamageVec>,   // 05 §5.2-5.3
    pub innate_dr: [f64; 3],                   // per Layer
    pub status_dr: f64,
    pub special_dr: Option<u16>,               // index into Rules.special_dr (05 §4.5, §15)
    pub flags: EnemyFlags,                     // TOXIN_BLOCKED PARTY_SCALED BOSS STATUS_CAPPED STRIP_IMMUNE NO_HEADCRIT …
                                               // CC_IMMUNE (05 §4.1 cc_immune: bosses, Acolytes, Demolishers; 03 E-3.13,
                                               // 06 E-6.24; overguard adds it as state, og_present)
    pub scaling: u16,                          // scaling class (05 §6.2)
    pub image: Option<Box<str>>,                // None: enemies have no DE export; the UI shows a faction glyph
}
pub struct BodyPart {                          // 05 §4.3 (field names of the spec)
    pub name: NameKey, pub part_mult: f64, pub is_weak_point: bool, pub is_head: bool, pub headcrit: f64,
    pub tier_add: u8,                          // critical promotion (02 E-2.6 π_p): Rogue Culverin canisters, Scaldra backpacks
    pub status_ok: bool, pub no_armor: bool, pub no_faction_mod: bool,
    pub non_body: bool,                        // 05 §4.3 non-body hitbox: Part_B = 1 (02 E-2.10), implies !is_head,
                                               // !status_ok, no_armor, no_faction_mod
    /* T01a: column, crit_disabled, wp_bonus_ok, part_dr, is_default (CONTRACTS.md) */
}
pub struct Faction {                           // Rules.factions, ingested from 05 §5.1 + wiki DamageTypes/data
    pub id: FactionId, pub slug: Box<str>, pub name: NameKey,
    pub type_mods: DamageVec,                  // φ_t (1.5 / 1 / 0.5)
    pub bane_of: SmallVec<[FactionId; 2]>,     // which faction bonuses apply (Kuva Grineer takes Grineer banes, D-008)
    pub weakness: Option<DamageType>,          // weakness-element map (01 §4.10.4)
    pub scaling_family: u16,
}
```

```rust
// wfd-model/src/frame.rs  (AD-26; Module:Warframes/data joined to ExportWarframes on uniqueName)
pub struct Frame {
    pub id: FrameId, pub key: StableKey, pub unique_name: UniqueName, pub name: NameKey,
    pub variant_of: Option<FrameId>,           // Prime and Umbra variants are the same frame for Helminth (09 E-9.32 H6)
    pub polarities: SmallVec<[Polarity; 8]>, pub aura_polarities: SmallVec<[Polarity; 2]>, pub exilus_polarity: Option<Polarity>,
    pub aura_slots: u8,                        // 1; 2 for Jade (09 §2.2)
    pub abilities: SmallVec<[FrameAbility; 4]>,// { name, self_buff: Option<ItemId> } (H2: a self-cast buff cannot be replaced)
    pub helminth_host: Option<ItemId>,         // Host(frame) of 09 E-9.32 H3 (rules/helminth.toml)
    pub image: Option<Box<str>>,
}
```

Frames carry identity, slot layout and Helminth facts only: base health, shield and energy are not damage inputs
(`DS:wiki` §8) and are not stored.

---

## 6. Items, sets, Incarnon lines, rivens

Mods, arcanes, shards, Incarnon perks, buffs and custom stats share one shape. A buff or custom stat is an item that
lives in the scenario instead of a slot.

```rust
// wfd-model/src/item.rs
pub struct Item {
    pub id: ItemId, pub key: StableKey, pub unique_name: UniqueName, pub name: NameKey,
    pub kind: ItemKind,
    pub compat: CompatRule,                    // categories / tags accepted, forbidding tags, trigger requirement (Semi)
    pub incompat: SmallVec<[ItemId; 4]>,       // union of both directions of the wiki lists (D-093)
    pub polarity: Option<Polarity>, pub base_drain: i8, pub max_rank: u8, pub rarity: Rarity,
    pub set: Option<SetId>,
    pub params: Vec<ParamSpec>,                // user parameters (Chromatic Blade element, buff value)
    pub effects: Vec<EffectDef>,               // section 11
    pub status: EffectStatus,                  // section 15.3
    pub sources: Vec<Box<str>>,                // citation keys ("W:Serration", "F:core §2")
    pub upstream_text: Option<Box<str>>,       // max-rank description the effects were checked against
    pub inspiration_name: Option<Box<str>>,    // only for link import; never a data source
    pub image: Option<Box<str>>,
}
pub enum ItemKind {
    WeaponMod { exilus: bool, augment_of: Option<UniqueName> },
    Stance { stance: StanceId },
    WeaponArcane { class: ArcaneClass },
    FrameMod, Aura,
    FrameArcane,
    Shard { color: ShardColor, tauforged_factor: f64 /* 1.5 */ },
    IncarnonPerk { family: FamilyId, tier: u8 /* 1..=5, tier 1 = form */, index: u8 },
    Buff { group: BuffGroup /* ability, focus, companion, squad, debuff, environment */, toggle: Option<ToggleId>,
           strength_scaled: bool,              // value = listed(rank) · Str of the caster (04 E-4.10)
           duration_scaled: bool,              // durations × Dur_e of the caster (04 §4.2 dur_scaled; T-33)
           default_caster: CasterKind,         // self (default) | ally: caster of a pick without one (04 §4.2
                                               // caster; T-37: ally for the T-13 Roar buff)
           subsumable: bool },                 // offered in the Helminth slot (09 §10)
    CustomStat { inspiration_key: Box<str> /* "p.a" */ },
}
pub struct ParamSpec {
    pub key: ParamKey, pub kind: ParamKind,    // Percent { min, max } | Number { min, max, step } | Choice(Vec<Choice>) | Bool
    pub default: ParamValue, pub label: MessageKey,
    pub helper: Option<ParamHelper>,           // e.g. Roar: value = rank value × Strength (UI calculator)
}
pub enum ParamValue { Num(f64), Choice(Box<str>), Bool(bool) }
pub struct SetDef { pub id: SetId, pub name: NameKey, pub members: SmallVec<[ItemId; 6]>,
                    pub bonus: Vec<EffectDef> /* uses per = set_piece */, pub member_scale: Option<Vec<f64>> /* Sacrificial */ }
pub struct IncarnonDef { pub family: FamilyId, pub variant: Box<str> /* "furis", "mk1_furis" */,
                         pub tiers: [SmallVec<[ItemId; 3]>; 5] }
pub struct RivenTraitDef {
    pub id: Box<str>,                          // "critical_chance", "damage_to_grineer", "weak_point_critical_chance" …
    pub pe_tag: Option<Box<str>>,              // Public Export upgradeEntries tag (value formula base b)
    pub input: RivenUnit,                      // Percent | FactionMultiplier (typed 1.45 → bonus 0.45) | Absolute
    pub classes: SmallVec<[RivenClass; 4]>,
    pub template: Option<EffectDef>,           // None = no damage effect (zoom, recoil)
    pub label: MessageKey,
}
pub struct Riven { pub class: RivenClass, pub traits: ArrayVec<RivenTrait, 4> }
pub struct RivenTrait { pub trait_id: Box<str>, pub value: f64 /* model units: fraction or bonus */ }
```

---

## 7. Rules tables

Typed data the engine reads instead of literals. Sources: tables ingested from the spec (pipeline.md §5.2) and
curated `data/curated/rules/*.toml`.

```rust
// wfd-model/src/rules.rs
pub struct Rules {
    pub status: [StatusRule; N_ST],           // 03 §4.1 summary table (ingested): duration, cap, DoT coefficient K_s,
                                              // first-tick delay δ_s, timer kind, debuff function parameters
    pub factions: Vec<Faction>,               // 05 §5.1 (ingested) + wiki DamageTypes/data
    pub scaling: Vec<ScalingClass>,           // 05 §6.1 coefficients (ingested), smoothstep range, SP and Eximus rules
    pub mult_groups: Vec<MultGroupDef>,       // section 14 (curated)
    pub slot_layouts: Vec<SlotLayout>,        // per equipment class: slot ids, accepted kinds, element order (09; curated)
    pub co_default: CoDefaults,               // 04 §2.5 default rule
    pub body_parts: BodyPartRules,            // defaults and head-crit rules (02 §4.5, 05 §4.3)
    pub special_dr: Vec<SpecialDrDef>,        // attenuation, flat DR, adaptation (05 §4.5, §15)
    pub riven_traits: Vec<RivenTraitDef>,     // 04 §8 (curated) with PE base values
    pub riven_formula: RivenFormula,          // 04 §8.2 constants
    pub toggles: Vec<ToggleDescriptor>,       // UI descriptors of T-01..T-43 (group, label, range, default, visibility)
    pub melee: MeleeRules,                    // 07 tables 9-A..9-E and combo specials (rules/melee/*.toml, section 4)
    pub helminth: HelminthRules,              // 09 E-9.32: restricted buffs R and Host(frame) (rules/helminth.toml)
    pub categories: Vec<CategoryDef>,         // CategorySet bit → name
    pub tags: Vec<TagDef>,
}
pub struct ToggleDescriptor { pub id: ToggleId /* "T-13" */, pub group: ToggleGroup, pub label: MessageKey,
                              pub help: MessageKey, pub control: ControlKind, pub default: serde_json::Value,
                              pub visible_if: Option<VisibilityRule> /* melee only, sniper only, … */, pub advanced: bool }
pub struct ToggleId(pub Box<str>);
```

`spec_constants.rs` in `wfd-engine` holds only formula constants that the spec defines inside formulas (1/32 grid,
4095/32, 2700, 0.9, 300, smoothstep shape); everything with a game value is in `Rules` (I-1).

---

## 8. Bundle

```rust
// wfd-model/src/bundle.rs
pub struct Bundle {
    pub manifest: Manifest,
    pub weapons: Vec<Weapon>,                 // index = WeaponId
    pub items: Vec<Item>,                     // index = ItemId
    pub sets: Vec<SetDef>, pub stances: Vec<Stance>, pub modular: ModularTables,
    pub combo_tables: Vec<ComboTable>,        // index = ComboTableId: stance, stanceless class and unique-to-weapon tables (07 §4)
    pub frames: Vec<Frame>,                   // index = FrameId (AD-26)
    pub enemies: Vec<Enemy>,                  // index = EnemyId
    pub rules: Rules,
    pub names: NameTable,                     // locale → (NameKey → text); "en" only for now
    pub stable_keys: Vec<(StableKey, UniqueName)>,   // global keys; a refreshed user bundle may add local keys (§2)
}
pub struct Manifest {
    pub schema_version: u32,                  // bumped on any layout change (postcard is not self-describing)
    pub data_version: Box<str>,               // "2026.10.28.09.54+<blake3 of inputs, 8 hex>"
    pub game_baseline: Box<str>,              // "Hotfix 44.1.4" (spec 00 §1.1)
    pub created_at: Box<str>,
    pub sources: Vec<SourceStamp>,            // wiki module title + revid + timestamp; WFCD tag + sha; PE buildLabel + hashes
    pub curated_rev: Box<str>,                // git describe of data/curated at build time
    pub spec_rev: Box<str>,                   // hash of the ingested spec tables
    pub licences: Vec<LicenceNotice>,         // section pipeline.md §10
    pub coverage: Coverage,                   // counts per EffectStatus per item kind
}
```

---

## 9. Builds, scenarios, toggles and conditions

### 9.1 Document form (API, share codes, local storage)

Documents reference everything by `StableKey` and are versioned (`v`). The resolved form (9.2) uses dense ids.
JSON shapes are in `server-api.md` §4.

```rust
// wfd-model/src/build.rs        (slot ids, item fields and legality: spec 09 §2)
pub struct BuildDoc {
    pub weapon: StableKey,
    pub weapon_item: WeaponItemDoc,
    pub variant: VariantDoc,
    pub slots: Vec<SlotEntry>,                // only non-empty slots, any order; canonical order on encode
    pub stance_combo: Option<ComboKey>,       // selected combo type for light loops (07 §8.1); default "neutral"
    pub warframe: WarframeDoc,
    pub companion_set_mods: Vec<StableKey>,   // declared companion / robotic-weapon set pieces (09 E-9.40, E-9.41)
}
pub struct WeaponItemDoc {                    // 09 §2.1 item-level fields (all optional, defaults in brackets)
    pub rank: Option<u8>,                     // [MaxRank]
    pub catalyst: bool,                       // [true]
    pub capacity_override: Option<u16>,
    pub polarities: BTreeMap<SlotId, Polarity>, // pol_gen0..pol_gen7, pol_exilus, pol_stance [innate]
    pub genesis_installed: bool,              // Incarnon Genesis weapons (09 E-9.60) [false]
}
pub struct WarframeDoc {
    pub frame: Option<StableKey>,             // optional identity → Bundle.frames (Helminth host rules 09 §10, aura count, polarities)
    pub reactor: bool,                        // [true]
    pub polarities: BTreeMap<SlotId, Polarity>,
    pub frame_complete: bool,                 // [false] warframe capacity checked only when true (09 E-9.15)
}
pub struct VariantDoc {
    pub kitgun: Option<KitgunPicks>,          // { grip: StableKey, loader: StableKey }        (T-25)
    pub zaw: Option<ZawPicks>,                // { grip: StableKey, link: StableKey }           (T-25)
    pub progenitor: Option<ProgenitorPick>,   // { element: DamageType, v10: u16 }  progenitor_frac = v10/1000 (T-24, 09 E-9.70)
}
pub struct SlotEntry { pub slot: SlotId, pub content: SlotContent }
/// JSON ids exactly as spec 09 §2.1: "gen0".."gen7", "exilus", "stance", "arcane0", "arcane1", "inc2".."inc5",
/// "fgen0".."fgen7", "fexilus", "aura0", "aura1", "farcane0", "farcane1", "shard0".."shard4", "subsumed".
pub enum SlotId { Gen(u8), Exilus, Stance, Arcane(u8), Inc(u8 /* evolution 2..=5 */), FGen(u8), FExilus, Aura(u8),
                  FArcane(u8), Shard(u8), Subsumed }
pub enum SlotContent {
    Item { item: StableKey, rank: Option<u8> /* None = max */, params: BTreeMap<ParamKey, ParamValue> },
    Riven(Riven),                             // only in Gen(_) slots (a riven is not a slot, 09 §2.1)
    Perk { index: u8 },                       // Inc(t): index into IncarnonDef.tiers[t-1]
    Shard { item: StableKey /* colour + bonus */, tauforged: bool },
    Subsumed { ability: StableKey /* buff item */, replaced: Option<Box<str>> },   // 09 E-9.32
}
pub struct Violation { pub key: MessageKey /* the rule key of 09, e.g. "cap.exceeded" */, pub severity: RuleSeverity,
                       pub slots: SmallVec<[SlotId; 2]>, pub args: BTreeMap<Box<str>, MsgArg> }
pub enum RuleSeverity { Illegal, IllegalUnverified, Warning }

// wfd-model/src/scenario.rs
pub struct ScenarioDoc {
    pub enemy: EnemySetup,                    // T-23
    pub toggles: Toggles,
    pub buffs: Vec<BuffPick>,                 // T-13..T-19 and every other buff (BuffPick below)
    pub custom: BTreeMap<Box<str>, f64>,      // T-26: custom stat id ("base", "crit_chance" …) → model-unit value
    pub conditions: ConditionPolicy,          // T-02 + per-effect overrides
    pub eval: EvalSettings,                   // T-01, T-29, horizons
    pub profile: Profile,                     // Game (default) | Compat (oracle harness only; hidden in the UI)
}
pub struct EnemySetup {                       // T-23 (spec 00 §6.1, 05); every field after `party` optional in JSON
    pub enemy: StableKey, pub level: u16, pub steel_path: bool, pub eximus: bool,
    pub party: u8,                            // 1..=4, 1..=6 with mission Icebind (05 EC-05-91)
    pub mission: Option<MissionModifier>,     // 05 §7.5
    pub context_overguard: bool,              // 05 §4.1 (Dax Malleus)
    pub eximus_kind: EximusKind,              // Generic (default) | Entropic | Guardian (05 §4.1 "Eximus kind")
    pub armor_hydroid: bool,                  // 05 E-5.12 Hydroid Corrosive variant
    pub squad_cp: SquadAuras,                 // squadmates' Corrosive Projection: { count, coaction_drift } (05 E-5.10)
    pub squad_shield_disruption: SquadAuras,  // squadmates' Shield Disruption (05 E-5.16, capped 0.80)
    pub shield_strips: Vec<f64>,              // external ability shield strips s_j·Str_j, final fractions (05 E-5.16)
    pub flat_strip_rate: f64,                 // r_flat, base-armor points per second (05 E-5.11), default 0
    pub aura_guardian: bool,                  // inside a Guardian Eximus aura (05 E-5.27)
    pub aura_prosecutor: Vec<DamageType>,     // Prosecutor auras by element HEA/COL/ELE/TOX, repeats allowed (05 E-5.27)
    pub shield_removed_externally: bool,      // 05 E-5.26, default true
    pub pt_shield_type: Option<DamageType>,   // 05 E-5.26 Profit-Taker displayed type, default none
}
pub struct BuffPick {
    pub buff: StableKey, pub params: BTreeMap<ParamKey, ParamValue>,   // T-32: params spot_location, spot_nodes
    pub caster: Option<Caster>,               // None = the buff's ItemKind::Buff.default_caster (T-37: ally for Roar)
}                                             // T-13..T-17 values are final values: caster Strength is not applied
pub enum Caster { SelfCast,                   // Str = T-20 (T-33 manual) or derived (T-33 derive); Dur_e = T-33's
                                              // toggles.ability_duration
                  Ally { strength: f64 /* 1.0 */, duration: f64 /* 1.0 */ } }   // 04 E-4.10, T-33
pub enum CasterKind { SelfCast /* default */, Ally }   // the buff item's default caster (04 §4.2 caster)
pub struct EvalSettings { pub window_s: f64 /* T-01, 20 */, pub ttk_horizon_s: f64 /* 60 */,
                          pub mc_runs: u32 /* T-29, 1000 */, pub seed: SeedSel /* T-29, Fixed(0) */, pub series_bin_s: f64 /* 1 */ }
/// JSON: "auto" or an integer 0 ..= 2^53 − 1 (`#[ts(type = "\"auto\" | number")]`; larger seeds are rejected, so the
/// browser never loses precision). Auto: the frontend draws a fresh seed for every MC request (parity row 25) and
/// sends it in `want.mc.seed`; the document and the share code store only "auto".
pub enum SeedSel { Auto, Fixed(u64) }
/// JSON "game" or { "compat": ["D001_QUANTIZATION_GRID", …] }: flag names, never the u128 (CompatFlags is not exported to
/// TypeScript).
pub enum Profile { Game, Compat(CompatFlags) }
pub struct CalcInput { pub v: u16, pub build: BuildDoc, pub scenario: ScenarioDoc,
                       pub x: BTreeMap<StableKey, UniqueName> }   // uniqueName of every local key used (§2, AD-18); usually empty
```

### 9.2 Resolved form

```rust
pub struct Build {                            // dense ids, produced by wfd-data::GameData::resolve
    pub weapon: WeaponId, pub item: WeaponItem, pub variant: Variant,
    pub gen: [Slot; 8], pub exilus: Slot, pub stance: Option<StanceId>, pub combo: ComboType,
    pub arcanes: [Option<(ItemId, u8)>; 2],   // arcane0, arcane1 (families per 09 E-9.30)
    pub genesis_installed: bool, pub incarnon: [Option<u8>; 4],   // perk index for evolutions 2..=5
    pub frame: FrameLoadout,                  // identity Option<FrameId>, gen [Slot; 8], exilus Slot, auras [Slot; 2],
                                              // arcanes [Option<(ItemId,u8)>; 2], shards [Option<ShardPick>; 5],
                                              // subsumed Option<Subsumed>, polarities
    pub companion_set_mods: SmallVec<[ItemId; 4]>,
}
pub enum Slot { Empty, Mod { item: ItemId, rank: u8, params: SmallVec<[(ParamKey, ParamValue); 1]> }, Riven(Riven) }
pub struct Scenario { pub enemy: EnemyInstance /* EnemySetup with a dense EnemyId */, pub toggles: Toggles,
                      pub buffs: SmallVec<[BuffInstance; 4]> /* { item, params, caster: Caster } (caster resolved
                      against ItemKind::Buff.default_caster) */,
                      pub custom: SmallVec<[(ItemId, f64); 4]>, pub conditions: ConditionPolicy,
                      pub eval: EvalSettings, pub profile: Profile }
```

`wfd-data::GameData::resolve(&CalcInput) -> Result<Resolved, Vec<Message>>` with
`Resolved { build, scenario, violations: Vec<Violation> }`. Following spec 09 §1.5, **validation never blocks
evaluation**: every rule of 09 (slot existence and kind E-9.2-9.3, compatibility §3, exclusion and duplicates §4,
ranks and drain §5, capacity and Forma §6 under the capacity mode, exilus §7, arcane families §8, shards §9,
Helminth §10, set counting §11, riven legality §12, Incarnon §13, valence §14) produces `Violation`s that the UI lists
and the optimizer treats as constraints; the engine evaluates the build regardless. `Err` is returned only for
malformed documents (unknown key, content kind impossible in that slot, such as a riven in `aura0`). Issues and
violations are `Message`s (the rule keys of 09 are the message keys), never English text.

### 9.3 Toggles

`Toggles` holds every toggle that is not already a field elsewhere. Where each T-nn lives:

| Toggle | Location | Type, default |
|---|---|---|
| T-01 window | `eval.window_s` | f64 1-60 (advanced 600), 20 |
| T-02 conditions | `conditions.mode` | `all_max` / `none` / `simulated`, `all_max` |
| T-03 hit location | `toggles.hit_location` | `body` / `head` / `weak_point` / `{ part: NameKey }` (any part of the enemy's 05 §4.3 list, `BodyPart.name`), `body` |
| T-04 CO multiplicative | `toggles.co_multiplicative` | bool, false |
| T-05 CO status count | `toggles.co_status_count` | `{ mode: expected | weapon_types | manual, n: 1..50 }`, expected |
| T-06 external Viral | `toggles.external_viral` | `Option<u8>` 1..10, None; T-43 restricted to Viral, kept for link import (a T-43 `VIR` entry wins) |
| T-07 external armor strip | `toggles.armor_strip` | f64 0-0.99, 0 |
| T-08 stealth | `toggles.stealth` | `Option<{ rank: Option<u8> }>`, None; rank 0..maximum rank (30 or 40), `None` = the weapon's maximum (C-15) |
| T-09 heavy attack | `toggles.heavy` | `Option<{ start_combo: 1..cap }>`, None; start 12 when switched on without a value |
| T-10 light-attack combo | `toggles.light_combo` | `Option<u8>`, None (simulated ramp) |
| T-11 ability combo | `toggles.ability_combo` | `Option<u8>` 1..12, None |
| T-12 sniper combo | `toggles.sniper_combo` | `Option<f64>` 1.5..100, None (simulated) |
| T-13 Roar, T-14 Eclipse, T-15 Xata's Whisper, T-16 Toxic Lash, T-17 Nourish, T-18 flat crit chance, T-19 Phoenix Talons | `buffs[]`: the buff items whose `ItemKind::Buff.toggle` is that id; param `value` (T-18 also `mode`: `flat` / `covenant`) | absent = off |
| T-20 Ability Strength | `toggles.strength` | f64 0-10, 1.0 (the total under T-33 `manual`) |
| T-21 weak-point CC multiplicative (hidden) | `toggles.wp_cc_multiplicative` | bool, false |
| T-22 combine attack effects | `toggles.combine_groups` | `Option<Vec<u8>>` indices of the families (`Weapon.groups`) evaluated as one unit; `None` = the profile default (every family in the game profile, none in the compat profile; CD-12) |
| T-23 enemy options | `enemy` | `EnemySetup` (§9.1) |
| T-24 progenitor | `BuildDoc.variant.progenitor` | `{ element, v10 }` |
| T-25 kitgun/zaw parts, Chromatic Blade element, Reactive Storm weakness | `BuildDoc.variant.kitgun/zaw`, item `params` | |
| T-26 custom stats | `custom` | map id → value |
| T-27 target distance | `toggles.distance_m` | f64 0-1000, 0 |
| T-28 unique damage reduction | `toggles.unique_dr` | bool, true |
| T-29 MC runs and seed | `eval.mc_runs`, `eval.seed` | 1-100,000 (1000), `"auto"` or 0 ..= 2^53 − 1 (0) |
| T-30 melee plan | `toggles.melee_plan`, `toggles.melee_plans` | `MeleePlanSel` below, `auto` (07 §8.1 default per attack kind); per attack `[{ attack: AttackKey, plan }]`, empty |
| T-31 exalted ability rank | `toggles.exalted_rank` | u8 0..3, 3 (01 E-1.1) |
| T-32 weak spots | `buffs[].params`: `spot_location` (`hit_part` / `torso`) and `spot_nodes` (ν_σ) of the Sonar and Detect Vulnerability buffs | `hit_part`; Sonar 1, Detect Vulnerability always 1 (02 E-2.10) |
| T-33 Strength sources | `toggles.strength_mode`, `toggles.ability_duration`, `buffs[].caster` (`Ally { strength, duration }`) | `manual` / `derive`, manual; self-cast Dur_e 1.0; ally Strength and Duration 1.0 (04 §4.4) |
| T-34 finisher mode | `toggles.finisher` | `Option<stealth | back | front | universal>`, None (04 §7.2) |
| T-35 capacity mode | `toggles.capacity`, `toggles.forma_types`; warframe part `BuildDoc.warframe.frame_complete` | `ignore` / `current` / `any_forma` / `{ budget: N }`, `any_forma`; Forma types (`forma`, `umbra`, `omni`, `stance`, `aura`), all (09 E-9.18, §6.1) |
| T-36 lower mod ranks in the optimizer | `OptRequest.constraints.capacity.ranks` | `max` / `allow_lower`, `max` |
| T-37 caster of T-13 Roar | `buffs[].caster` of the T-13 buff | `self` / `ally`; absent = Roar's `default_caster` `ally`; T-14 … T-16 always self |
| T-38 aiming and zoom | `toggles.aiming`, `toggles.zoom_level` | bool, true; `Option<u8>` index into `Weapon.zoom`, None = highest level under `all_max`, none under `none` (04 D-406) |
| T-39 Incarnon switch time | `toggles.incarnon_switch` | bool, false (06 §6.6) |
| T-40 timed reload | `toggles.timed_reload` | bool, false (06 §5.8) |
| T-41 pellet hit fraction | `toggles.pellet_hit` | `[{ attack: AttackKey, fraction: 0..1 }]`; an attack without an entry uses 1 |
| T-42 gauge uses per minute | `toggles.gauge_uses_per_min` | f64 ≥ 0, 0 (06 §6.7) |
| T-43 external statuses | `toggles.external_statuses` | map `StatusType` → N_s stacks 1..cap_s (COL 1..9), empty (03 §6.3, 08 `n_ext`) |

`wfd-model::scenario::TOGGLE_MAP: &[(&str /* "T-13" */, TogglePlace, Status /* Spec | Proposed */)]` lists every row
of this table with its document path (`|` separates several paths; a path through a list names the list, e.g.
`buffs.caster`); `xtask spec-sync` checks it against spec 00 §6.1 (every T-nn present exactly once). Every entry is
`Spec` since T01d renumbered the toggles to the final spec (CONTRACTS.md T01d); `Proposed` stays available for a
toggle a topic document proposes before 00 §6.1 lists it. A test (`toggle_map_paths_exist`) checks that every
scenario path exists in the JSON a scenario document writes.

```rust
pub struct Toggles {
    pub hit_location: HitLocation,             // Body | Head | WeakPoint | Part(NameKey)
    pub co_multiplicative: bool, pub co_status_count: CoStatusCount,
    pub external_viral: Option<u8>, pub armor_strip: f64, pub stealth: Option<Stealth>, pub heavy: Option<Heavy>,
    pub light_combo: Option<u8>, pub ability_combo: Option<u8>, pub sniper_combo: Option<f64>,
    pub strength: f64, pub wp_cc_multiplicative: bool,
    pub combine_groups: Option<SmallVec<[u8; 2]>>,   // T-22, None = profile default
    pub distance_m: f64, pub unique_dr: bool,
    pub capacity: CapacityMode,                // T-35: Ignore | Current | AnyForma (default) | Budget(u8)
    pub forma_types: FormaTypes,               // T-35 allowed Forma types: FORMA UMBRA OMNI STANCE AURA (default all)
    pub melee_plan: MeleePlanSel,              // T-30
    pub melee_plans: Vec<AttackPlan>,          // T-30 per attack: { attack: AttackKey, plan: MeleePlanSel }
    pub zoom_level: Option<u8>, pub aiming: bool,    // T-38
    pub exalted_rank: u8,                      // T-31, default 3
    pub strength_mode: StrengthMode,           // T-33: Manual (default) | Derive
    pub ability_duration: f64,                 // T-33 self-cast Dur_e, default 1.0
    pub finisher: Option<FinisherMode>,        // T-34: Stealth | Back | Front | Universal
    pub incarnon_switch: bool,                 // T-39
    pub timed_reload: bool,                    // T-40
    pub pellet_hit: Vec<AttackFraction>,       // T-41: { attack: AttackKey, fraction: f64 }
    pub gauge_uses_per_min: f64,               // T-42
    pub external_statuses: BTreeMap<StatusType, u8>,   // T-43; Toggles::external_stacks(s) folds T-06 in
}
/// 07 §8.1 plans with their parameters. `Auto`: Normal Attack → light_loop(selected combo), heavy (T-09 or selected)
/// → heavy_loop, slide → slide_loop, slam → slam_loop, heavy slam → heavy_slam_loop, throws → glaive_cycle, weapons
/// without light combos → cycle_loop, a directly selected data attack → data_attack_loop.
pub enum MeleePlanSel {
    Auto,
    LightLoop,                                 // combo K = BuildDoc.stance_combo (default neutral)
    SlideLoop,
    HeavyLoop { policy: HeavyPolicy /* FullRegen (default) | BackToBack */, heavy_cancel: Option<f64> /* t_dmg */ },
    BuildSpend { k: f64 /* M_trig, default M_cap */ },
    Tennokai,                                  // n_tk from E-7.17
    SlamLoop { t_slam: f64 /* 1.0 */ }, HeavySlamLoop { t_slam: f64 /* 1.0 */ },
    GlaiveCycle { throw: ThrowKind /* Charged (default) | Quick */ },
    CycleLoop,                                 // T_cyc = 1 / AS (07 §8.1)
    DataAttackLoop { t_cyc: Option<f64> /* None = 1 / FireRate of the attack */ },
}
```

The engine turns the selection into a `MeleePlan` per melee attack: the attack's `melee_plans` entry, else
`melee_plan` when the plan's kind names the attack (07 §8.1: `light_loop`, `build_spend` and `tennokai` the Normal
Attack, `heavy_loop` the heavy attack, `slide_loop` the slide attack, …), else the `Auto` mapping above; then the T-09
rule of 07 E-7.31. `melee_schedule` takes that `MeleePlan` (engine.md §5). An attack key that names no attack of the
weapon is dropped with `doc.attack_unknown`.

### 9.4 Conditions policy

```rust
pub struct ConditionPolicy {
    pub mode: ConditionMode,                          // AllMax (default) | None | Simulated   (T-02)
    pub overrides: BTreeMap<EffectKey, CondOverride>,
}
pub enum CondOverride {
    Force(bool),                                       // always on / always off, any mode
    Stacks(u16),                                       // fixed stack count
    Uptime { u: f64, stacks: Option<u16> },            // class (c) under `simulated`: u_e and m_e of 04 §4.3
}
```

Overrides improve on the inspiration's single global switch: the UI lists every conditional effect of the current
build with its derived class (section 12) and its current treatment, and lets the user pin it. The optimizer re-scores
finalists with `mode = None` to report condition sensitivity.

### 9.5 Compatibility flags

```rust
// wfd-model/src/compat_generated.rs — written by `cargo xtask spec-sync`, never by hand
bitflags! { pub struct CompatFlags: u128 {
    // exactly one flag per register row whose switch kind is `Compat` (spec 00 §5.1 rule 3: the first word of each
    // `+`-separated part of the Toggle cell before any parenthesis, `:`, `;` or `,`; a cell such as
    // "— (Compat path of D-065)" has none), over spec 00 §5.2 and the "Inspiration differences" register of every
    // topic document 01-09 (74 flags when T01d landed, bits 0-73; register order: 00 first, then 01 … 09, each row
    // once even when restated by several documents)
    const D001_QUANTIZATION_GRID = 1 << 0;  const D002_WHAT_IS_INSIDE_QUANTIZATION = 1 << 1;  /* … */
    const INSPIRATION = /* all of the above */;
} }
```

`xtask spec-sync` parses every `## … Inspiration differences` table of `docs/mechanics/0[1-9]-*.md` and the register
of 00 §5.2, generates the flag list (name = id without its hyphen + up to four words of the Topic column, upper case,
joined by `_`; doc comment = owner document and topic), and in `--check` mode fails when the file is stale, a `Compat`
row has no flag or a flag has no row; `cargo xtask spec-sync --list` prints the table. The count is not pinned in the
architecture: it follows the registers (the `xtask` tests pin it with the generated file). 128 bits leave room for
54 more rows; past that the type becomes a `[u64; N]` bitset (a contract change). Every branch on a flag is a
one-line `if ctx.compat.contains(CompatFlags::D0xx_…)` in the stage that owns the row, with a `// D-nnn` comment.
`CompatFlags` has no TypeScript form: documents carry flag names (`Profile`, §9.1).

---

## 10. The stat registry

`wfd-model/src/stats.rs` declares every additive bucket once with a macro. The macro generates the `StatKey` enum
(one variant per slot), the TOML names, the slot indices (`N_STATS`, about 220), unit conversions for validation,
display message keys, and the monotonicity class the optimizer uses (optimizer.md §4). Curators never pick an
operation: the stat decides it. A separate multiplier is never written as "multiply"; it is `mult = "<group>"`
(section 14).

```rust
define_stats! {
//  TOML key               slots  spec symbol                  aggregate      monotone  enters   owner
    base_dmg                 1    "sum_base_dmg (A)"           sum            Inc       ST-4     "01";
    base_dmg_universal       1    "A_w melee-projectile bucket" sum           Inc       ST-4     "04 §3.4";
    incarnon_flat            1    "incarnon_flat (X, HP)"      sum            Inc       ST-1     "01";
    phys                     3    "sum_phys[t] t ∈ IPS"        sum            Unknown   ST-2     "01";
    elem_nc                 10    "sum_elem[t], non-combining" sum            Unknown   ST-2     "01 §4.10.3";
    phoenix_talons           1    "pt (01 §4.9)"               sum            Unknown   ST-2     "01";
    faction                  1    "sum_faction (F)"            sum            Inc       ST-5     "01 §4.15";
    co_per_status            1    "C coefficient of n"         sum            Inc       ST-4     "04 §2";
    co_per_cold_stack        1    "C coefficient of n_COL"     sum            Inc       ST-4     "04 §2";
    co_flat                  1    "C constant terms"           sum            Inc       ST-4     "04 §2";
    cc_base_add              1    "CC0+"                       sum            Inc       E-5      "02";
    cc_rel                   1    "ΣCC_rel"                    sum            Inc       E-5      "02";
    cc_abs                   1    "ΣCC_abs"                    sum            Inc       E-5      "02";
    cc_floor                 1    "CC_min"                     max            Inc       E-5      "02";
    cc_final_mul             1    "custom p.d factor"          sum→(1+x)      Inc       E-5      "00 §6.3";
    cd_base_add              1    "CDM0+"                      sum            Inc       E-4      "02";
    cd_rel                   1    "ΣCD_rel"                    sum            Inc       E-4      "02";
    cd_abs                   1    "ΣCD_abs"                    sum            Inc       E-4      "02";
    cd_field_mul             1    "M_cdf"                      sum→(1+x)      Inc       E-4      "02";
    tier_raise               1    "p_vig"                      sum            Inc       E-6      "02";
    wp_dmg                   1    "ΣWP"                        sum            Inc       E-7      "02";
    hs_mult                  1    "ΣHS"                        sum            Inc       E-7      "02";
    sc_base_add              1    "SC0+"                       sum            Inc       E-8      "02";
    sc_rel                   1    "ΣSC_rel"                    sum            Inc       E-8      "02";
    sc_abs                   1    "ΣSC_abs"                    sum            Inc       E-8      "02";
    sc_mul                   1    "M_sc (custom p.k)"          sum→(1+x)      Inc       E-8      "02";
    forced                  18    "forced_procs[s] per pellet" sum            Unknown   E-9      "02 §4.7.2";
    multishot                1    "ΣMS"                        sum            Inc       E-10     "02";
    multishot_base_add       1    "MS0+"                       sum            Inc       E-10     "02";
    lock_multishot           1    "Acuity lock"                any            Dec       E-10     "02";
    status_dmg               1    "A_sd"                       sum            Inc       E-13     "03";
    status_dmg_type         18    "A_sd,s"                     sum            Inc       E-13     "03";
    status_dur               1    "SD"                         sum            Inc       CD-06    "03";
    stack_cap               18    "stack_cap[s] increase"      sum            Inc       CD-07    "03";
    external                18    "external stacks (T-06)"     max            Inc       state    "03";
    dot_vuln                 1    "M_dot (custom p.o)"         sum→(1+x)      Inc       E-13     "03";
    heat_inherit             1    "Heat inherit (D-028, p.y)"  sum            Inc       E-13     "03 §10.3";
    fire_rate                1    "fire-rate bucket"           sum            Inc       06       "06 §5.1";
    lock_fire_rate           1    "Cannonade lock"             any            Dec       06       "06 §5.1";
    charge_rate              1    "charge speed"               sum            Inc       06       "06 §5.3";
    magazine                 1    "magazine %"                 sum            Inc       06       "06 §5.5";   // downgraded by shot marks
    magazine_flat            1    "magazine +N"                sum            Inc       06       "06 §5.5";
    reload_speed             1    "reload speed"               sum            Inc       06       "06 §5.7";
    ammo_eff                 1    "ammo efficiency ≤ 0.75"     sum, clamp     Inc       06       "06 §5.6";
    attack_speed             1    "AS"                         sum            Inc       07       "07";
    initial_combo            1    "initial combo"              sum            Inc       07       "07";
    combo_dur                1    "combo duration"             sum            Inc       07       "07";
    heavy_eff                1    "heavy efficiency ≤ 0.9"     sum, clamp     Inc       07       "07";
    wind_up                  1    "wind-up speed"              sum            Inc       07       "07";
    strength                 1    "Str sources (04 §4.4)"      sum            Inc       scenario "04 §4.4";
    vuln_add                54    "V additive group, per (layer ℓ, type t)" sum   Inc       ST-8     "05 E-5.19, vuln[ℓ][t]";
    strip_ability            1    "E-15 ability strip"         sum            Inc       E-15     "05 §9.3";
    strip_aura               1    "E-15 Corrosive Projection"  sum            Inc       E-15     "05 §9.3";
    xh                       8    "p_xh by extra-hit kind"     sum            Inc       E-14     "04 §6";
    mech                    16    "mechanic magnitudes"        sum            Inc       E-9 …    "02, 04";
    mult                    48    "one slot per multiplier group" sum→Π(1+x)  Inc       group    "04 §3";
    display                 12    "range, punch-through, zoom, recoil, accuracy, flight speed, ammo max, noise, …" sum None panel "—";
}
```

- **Declared classes are scenario-independent; the compiled scenario downgrades them.** `Inc` holds for a slot only while
  no active effect or mechanic reads an outcome the slot feeds in a non-monotone way. After L0 and L1 compilation of
  the scenario and of the whole candidate pool, the scenario compiler (`compile::monotone`, engine.md §3) collects every
  *reader*: a trigger, gate, condition, mechanic parameter or `per` scalar that reads one of the quantities below, and
  downgrades the slots that feed that quantity to `Unknown` for this scenario (`CompiledScenario.slot_class`):

  | Quantity read | Readers (examples) | Slots downgraded |
  |---|---|---|
  | crit-tier outcome of an instance | `OnTier1Crit`, `repeat_hit` (tier = 1: Melee Duplicate), `random_multiplier` (tier = 0: Devouring Attrition), hit-outcome gates (Overwhelming Attrition), `counter_bonus.reset_on` crit tiers (Secondary Enervate), `Cond::Attack` on CC (Prelude of Might) | `cc_*`, `tier_raise` (and `cd_*` when the reader reads crit damage) |
  | proc outcome of an instance | hit-outcome gates (`status = none`), `OnStatus` with a non-monotone use, `TargetPred::DistinctBelow` | `sc_*`, `status_dur` (presence), `forced.*` (already `Unknown`) |
  | schedule marks (first or last shot of a magazine, reload, reloaded fraction) | `Cond::Shot`, `OnReload`, `OnReloadFromEmpty`, `Per::ReloadedFraction` (Primed Chamber, Charged Chamber, Synth Charge, Plated Round) | `magazine`, `magazine_flat`, `ammo_eff`, `fire_rate`, `reload_speed`, `charge_rate` |
  | a resolved stat through `Per::Resolved` or `Cond::Attack` | Melee Doughty, Prelude of Might, Plated Round (magazine) | the slots that feed the read stat, unless the reader is monotone (below) |

  A reader is *monotone* and causes no downgrade when its predicate is upward-closed in the fed quantity (crit tier
  `≥ m` with m ≥ 1, `OnCrit`, `OnStatus` presence, `Per::Resolved` with a non-decreasing curve) **and** every value it
  gates is non-negative: Hunter Munitions, Melee Doughty and Blood Rush keep their slots `Inc`; Devouring Attrition
  (tier = 0), Melee Duplicate (tier = 1), Overwhelming Attrition (no crit, no status) and Primed Chamber (fewer first
  shots with a larger magazine) do not. Readers held by candidates (not by fixed context items) are *coupled
  candidates* (optimizer.md §3): the search branches on including them first, and in the branch that includes none the
  slots keep `Inc`. The rule is data-independent code in one function with one property-test fixture per reader kind
  (testing.md §2.3).
- `forced`, `phys`, `elem_nc`, `phoenix_talons` change the damage mix and proc weights, so they are `Unknown`: the
  optimizer enumerates them instead of bounding them.
- `mech` slots are allocated at scenario compile time, one per distinct `(mechanic kind, event, status, exclusive
  group)` signature present in the candidate pool (Hunter Munitions and Internal Bleeding get two slots in one
  exclusive group).
- `mult` slots are allocated from `Rules.mult_groups` (section 14).
- `display` stats feed only the weapon panel; the optimizer drops items whose contributions are all `display`.
- `faction` is resolved at L1 for the scenario's target (`Faction.bane_of`), so it is one slot, not one per faction.

---

## 11. The effect DSL

### 11.1 Types

```rust
// wfd-model/src/effect.rs     (bundle form = authoring form after lowering of sugar and binding; same types)
pub struct EffectDef {
    pub id: Box<str>,                         // stable within the item ("e0", or curator-given "co"); EffectKey = item#id
    pub target: Target,
    pub value: ValueExpr,
    pub per: SmallVec<[Per; 2]>,              // value is multiplied by the product of these scalars
    pub when: Cond,                           // dynamic condition (default Always); T-02 class derived (section 12)
    pub stacks: Option<StackModel>,           // own stack counter (04 §4.2); `per = stack` reads it
    pub scope: Scope,                         // static applicability, never assumed (04 §4.3 "gates")
    pub scope_factor: SmallVec<[ScopeFactor; 1]>, // value ×k on a sub-scope ("x2 for Heavy Attacks", "x2 for Bows")
    pub order: Option<ElemOrder>,             // only for element targets (section 13)
    pub vuln: Option<VulnMask>,               // only for `vuln_add` and TargetVuln groups: { types, layers } (05 E-5.19)
    pub class: Option<ClassSet>,              // optional declared class; the validator checks it equals the derived one
    pub note: Option<Box<str>>,               // UNVERIFIED notes, carried to the UI and traces
}

pub enum Target {
    Stat(StatKey),                            // TOML `stat = "cc_rel"`, `stat = "forced.SLA"`, `stat = "stack_cap.COR"`
    Element(ElemSel),                         // TOML `element = "HEA"` | "target_weakness" | { param = "element" }
    Faction(FactionSel),                      // TOML `faction = "any"` | ["grineer", "kuva_grineer"]
    Mult(Box<str>),                           // TOML `mult = "eclipse"` (group id from Rules.mult_groups)
    Mechanic(Mechanic),                       // TOML `mechanic = "extra_hit"` + `[effect.params]`
}
pub enum ElemSel { Type(DamageType), TargetWeakness, Param(ParamKey) }
pub enum FactionSel { Any, Of(SmallVec<[Box<str>; 2]>) }

pub enum ValueExpr {
    Const(f64),                               // TOML `value = 1.65` (max rank)
    Ranked { max: f64, rank: RankCurve, bind: Option<Binding> },     // Linear (max·(r+1)/(R+1), default) | Table(Vec<f64>) | Fixed
    Variants { by: BTreeMap<Box<str>, f64> }, // Incarnon per-variant values (Furis 28, MK1-Furis 34)
    Param(ParamKey),                          // user parameter (buff value, custom stat value)
    Linear { base: Box<ValueExpr>, add: SmallVec<[Term; 1]> },       // base + Σ coeff·Π per (Violet shard)
    TableBy { scalar: Per, values: Vec<f64> },// value indexed by a static scalar (set pieces)
}
pub struct Term { pub coeff: f64, pub per: SmallVec<[Per; 1]> }
pub enum RankCurve { Linear, Table(Vec<f64>), Fixed }
pub struct Binding(pub Box<str>);           // "n1%", "n2", "n3x" (section 15.2)

/// Scalars an effect value can be multiplied by. Static scalars are resolved at L2 (build); dynamic ones become
/// state variables (section 12).
pub enum Per {
    Stack,                                    // own stack counter (dynamic; class of its trigger)
    DistinctStatus,                           // n, CO count (dynamic, class a; T-05)
    StatusStacks(StatusType), StatusPresent(StatusType),            // dynamic, class a
    ComboTier, ComboTierAboveOne,             // M(c), M(c) − 1 (dynamic, combo model; T-10 may fix it)
    SniperCombo,                              // dynamic (06 §7.2; T-12 may fix it)
    SetPiece,                                 // static: equipped pieces of the item's set (09 counting)
    ShardCount(SmallVec<[ShardColor; 3]>),    // static
    Strength,                                 // static: T-20 or the buff's caster Strength (04 E-4.10)
    Resolved { stat: ResolvedStat, curve: ResolvedCurve },          // static after stage ordering (Melee Doughty, Plated Round)
    ReloadedFraction,                         // dynamic, schedule-driven: (Mag − left)/Mag of the latest reload (06 ReloadRec)
}
pub enum ResolvedCurve {
    Step { step: f64 },                       // floor(x / step) (Melee Doughty: +CD per 10 % Puncture status chance)
    Pow { scale: f64, exp: f64 },             // (scale · x)^exp (Plated Round: 0.15·sqrt(5·Mag) = value 0.15, Pow { 5, 0.5 })
}
pub struct VulnMask { pub types: TypeSel /* Set(DamageTypeSet) | Param(ParamKey) (Theorem Contagion: orb type);
                                            default every type (04 §4.2 type_mask "all") */,
                      pub layers: LayerSet /* default {HP}: 05 §13 "an entry without an explicit mask applies to
                                              health only", 04 §4.2 layer_mask (ruling C-14) */ }

pub enum Cond {
    Always,
    Trigger { trigger: TriggerId, chance: f64,          // 04 §4.1 trigger ids; class (b) or (c) by id
              hit_unit: HitUnit /* Pellet (default) | Shot | Instance | Input: what one "hit" is, 04 §4.2 */ },
    Target(TargetPred),                                  // class (a)
    Shot(ShotPos),                                       // FirstOfMagazine | LastOfMagazine { min_base_magazine: u16 }
    Attack(AttackPred),                                  // StatBelow { stat, x } | StatAtLeast { stat, x } — formula, not T-02
    All(Vec<Cond>), Any(Vec<Cond>),
}
pub enum TriggerId {                                     // exactly the ids of 04 §4.1
    // (b) own events
    OnHit, OnCrit, OnTier1Crit, OnWeakPointHit, OnHeadshot, OnStatus(StatusType), OnReload, OnReloadFromEmpty,
    WhileAiming, FirstShot, LastShot, Combo(u8), OnMeleeHitCount(u16), NextShotAfter(Box<TriggerId>),
    CountWithin { of: Box<TriggerId>, n: u8, window: f64 },     // n events of `of` within `window` s (Phenmor Lingering
                                                                // Judgement: 2 headshots within 2 s); class (b)
    OnMeleeInput(InputKind),                                    // light / slide / aerial / wall input start (07 7.5 input_start)
    OnThrow,                                                    // glaive or thrown-weapon throw (07 §9.4)
    // (c) external
    OnKill, OnWeakPointKill, OnMeleeKill, OnPrimaryKill, OnAbilityCast, WhileInvisible, OnWallLatch, OnHolster,
    OnDamaged, OnRoll, WhileOvershields, OnShieldBreak, InZone, AllyBuffs, EnergyLevel, ShieldLevel, ArmorLevel,
}
pub enum TargetPred { HasStatus(StatusType), StatusCountAtLeast(u8), DistinctBelow(u8), Frozen, Marked,
                      Incapacitated, HealthBelow(f64) }
pub struct StackModel {                                  // field names and values of 04 §4.2
    pub max: u16, pub gain: u16 /* 1 */, pub duration: f64,
    pub unbounded: bool,                                 // max_stacks = none (04 §4.2): a class (c)/(u) user count
                                                         // with no limit (Magus Accelerant, 04 §5.5); `max` is then its
                                                         // default count, never a cap (ruling C-4)
    pub stack_timer: StackTimer,                         // SharedLoseOne | SharedLoseAll | Independent
    pub refresh: Refresh,                                // Refresh | NoRefreshWhileActive
    pub decay: ValueDecay,                               // None | Linear | Garuda
}
pub struct Scope {                                       // static; resolved per attack and hit kind at L1
    pub weapon: WeaponFilter,                            // slots, classes, categories, tags, exalted: Any | Never | Only
    pub attacks: RoleFilter,                             // any_of / none_of AttackRoles
    pub form: FormSel,                                   // Any | Normal | Incarnon
    pub hit: HitFilter,                                  // kinds: Direct | Radial | DotTick | ExtraHit;
                                                         // body_part: Body | Head | WeakPoint (T-03); pellet: Any | ExtraOnly;
                                                         // outcome: Option<OutcomeFilter> (below)
    pub layers: LayerSet,                                // default all; e.g. overguard only
}
/// Hit-outcome gate: the effect applies only to instances whose own rolls satisfy it (Laetum Overwhelming Attrition:
/// `{ crit = "none", status = "none" }`). Procs of extra hits do not count (`W:Extra_Hit` §Bugs, 04 §5.12). EV: the
/// kernel adds the effect's term only on the branch k = 0 with weight P(k = 0)·P(instance applies no proc) (02 E-2.6,
/// E-2.18; the crit factor is 1 on that branch); the split is exact because the gated value changes neither the
/// composition nor the proc chances of the instance, so gate splitting (engine.md §7.4) is not needed. MC: evaluated
/// live after the crit and proc rolls of the instance and before its damage is applied.
pub struct OutcomeFilter { pub crit: CritOutcome /* Any | None | Exactly(u8) | AtLeast(u8) */, pub status: ProcOutcome /* Any | None | Some */ }
pub struct ScopeFactor { pub when: Scope, pub factor: f64 }
pub enum ElemOrder { Slot, Riven, Innate, NonCombining }
```

### 11.2 Mechanic kinds (closed set)

Each kind is implemented once in `wfd-engine::mechanics` for both evaluators, against the spec section named here. A
mechanic that fits no kind needs a new kind (code change reviewed against the spec plus a spec vector); there are no
per-item hooks (I-2).

| Kind | Parameters | Magnitude slot | Used by | Spec |
|---|---|---|---|---|
| `extra_hit` | `element` (type), `part` (`once` \| `twice`), `status` (`roll_weapon_sc` \| `forced` \| `none`), `melee_factor`, `status_duration_add`, `triggers` (`pellet_hit`, `blast_detonation`) | `xh.<kind index>` (value = p_xh) | Xata's Whisper, Toxic Lash, Resupply, Silken Stride, Demonium, Sangodae, Gastroparesis | E-14, 04 §6 |
| `proc_on_event` | `on` (`crit_instance` \| `tier1_crit` \| `proc` + `status` \| `hit`), `proc` (status \| `random` + set \| `component_of_combined`), `count`, `chance_if` (list of `{ fire_rate_below, factor }`), `exclusive_group` | `mech.<n>` (value = chance) | Hunter Munitions, Internal Bleeding, Hemorrhage, Magnetic Welt, Secondary Encumber, Secondary Cryogenic, Archon Continuity | 02 §4.7.3-4.7.5 |
| `convert_damage` | `from` (types \| `all`), `to` (type \| `param` \| `target_weakness`), `fraction` | — (composition) | Impact-to-Puncture, Chromatic Blade, Reactive Storm | 01 §4.4, D-009 |
| `repeat_hit` | `on` (`crit_tier_eq` k \| `crit_tier_at_least` k), `chance` | `mech.<n>` | Melee Duplicate (`crit_tier_eq = 1`) | 04 E-4.15, 08 E-8.12 |
| `flat_damage_on_proc` | `statuses`, `amount`, `scaling` (`none` \| `strength`) | `mech.<n>` | Cascadia Empowered | 04 §5.4 |
| `proc_after_ticks` | `combined` (status), `ticks`, `adds` (`component`) | — | Primary Debilitate | 04 §6.5 |
| `counter_bonus` | `stat`, `per_event` (`hit` \| `input` + `InputKind` \| `throw`), `max` or `unbounded = true` (no cap until the reset, 04 §4.2 `max_stacks = none`; ruling C-4), `reset_on` (`crit_tier_at_least` k \| `heavy` \| `other_input` \| `reload`), `reset_after` (count), `applies_to` (Scope: e.g. heavies only), `hit_unit` | `mech.<n>` | Secondary Enervate (per hit, unbounded, reset after 6 hits of tier ≥ 2), Hata-Satya (per pellet, unbounded, reset by a reload), Melee Animosity (per light input, max 10, consumed by the next heavy), Power Throw (per consecutive throw, 3 stacks, reset by another input) | 04 §5.3, §5.4, §5.12, 07 §7.5 |
| `random_multiplier` | `factor`, `on` (`crit_tier_eq` k) | `mech.<n>` (value = chance) | Devouring Attrition | 04 §3.2 |
| `spawn_attack` | `template` (attack data), `fraction`, `on` (event) | `mech.<n>` | Nightwatch Napalm, Volatile Quick Return | 06 §9, D-074 |

Exclusive groups implement "at most one forced Slash per instance, `P = 1 − (1 − a)(1 − b)`" (`F:addmul` §8) for any
members, without naming Hunter Munitions in code.

---

## 12. Condition classes and the T-02 compile table

### 12.1 Derived class

The T-02 class (spec 00 §6.2) of an effect is computed, never trusted from the author:

| Construct | Class |
|---|---|
| `Cond::Trigger` with a (b) id of 04 §4.1 | (b) own events |
| `Cond::Trigger` with a (c) id | (c) external |
| `Cond::Target(_)` | (a) target state |
| `per` contains `DistinctStatus`, `StatusStacks`, `StatusPresent` | (a) |
| `per` contains `Stack` | the class of the `StackModel`'s gain trigger (its `when`) |
| `per` contains `ComboTier*`, `SniperCombo` | combo model (driven by 07/06 or fixed by T-10/T-12; not switched off by T-02) |
| `Cond::Shot`, `Per::ReloadedFraction` | schedule-driven in every mode (spec 00 §6.2 last paragraph); a `ReloadedFraction` effect also has the class of its `on_reload` trigger |
| `Scope.hit.outcome` | gate bound to the instance's own crit and proc rolls, never assumed |
| `mechanic = "counter_bonus"` | the class of its `per_event` (own hits, inputs, throws: (b)) |
| `Cond::Attack` | formula: evaluated on the resolved attack in every mode (Prelude of Might) |
| `Scope.hit.body_part` | gate bound to T-03, never assumed |
| `Per::SetPiece`, `ShardCount`, `Strength`, `Resolved` | static |

An effect with several classes (Galvanized Aptitude's CO: stacks gained on kill (c), multiplied by distinct statuses
(a)) is active only when every involved class is active in the current mode.

### 12.2 Compile rules per mode

The scenario compiler (engine L1) turns each effect into a constant or a coefficient on a state variable:

| Construct | `all_max` | `none` | `simulated` |
|---|---|---|---|
| `Trigger` class (b) with stacks | constant `value × max` | 0 | coefficient on `CondStacks(id)`; E-4.8 (MC) / E-4.9 (EV) from the attack's own events, from 0 stacks |
| `Trigger` class (b), single stack | constant `value` | 0 | coefficient on `CondStacks(id)` |
| `Trigger` class (c) | constant `value × max` | 0 | constant `u_e · value · m_e` (04 §4.3: external events are not simulated; default u = 1, m = max; overrides 9.4) |
| `Target` (a) count-based (`per` DistinctStatus, StatusStacks) | coefficient on the state variable; n per T-05 | 0 | coefficient on the state variable |
| `Target` (a) threshold (`HasStatus`, `Frozen`, `HealthBelow`, `DistinctBelow`) | constant (true) | 0 | probability of the predicate (EV) / live (MC); `HealthBelow` false in non-depleting views, live in the `layered` pass |
| `WhileAiming`, `WhileInvisible` (conditions, not gates) | true | 0 | user uptime (default 1) |
| `Shot` | schedule-driven | schedule-driven | schedule-driven |
| `Per::ReloadedFraction` (with its `on_reload` trigger) | 1 (full reloads under continuous fire, 06 EC-06-50) | 0 | from the latest `ReloadRec` while its duration lasts; 0 before the first reload (fresh magazine) |
| `Scope.hit.outcome` gate | gated per instance (EV branch weight, MC live) | same | same |
| `counter_bonus` | counter at `max` | 0 | coefficient on `Counter(mech)`: expected path (EV) / live (MC) |
| `Attack` | evaluated on the resolved attack | same | same |
| `Per::ComboTier*` | coefficient on `ComboTier` (T-10 fixes it) | same | same |
| Unconditional effect of a conditional item (Galvanized Aptitude's +80 % SC) | applies | applies | applies |
| `CondOverride` present | override wins: Force → constant on/off; Stacks → constant `value × k`; Uptime → constant `u·value·m` |

State variables (`wfd-engine::state::StateVar`): `Distinct`, `Stacks(s)`, `Present(s)`, `Frozen`, `HealthBelow(x)`,
`ComboTier`, `SniperCombo`, `CondStacks(id)`, `Counter(mech)`, `ReloadedFraction`, `FirstShot`, `LastShot`. EV reads
expectations over their distributions
(engine.md §6), MC reads live values.

---

## 13. Element ordering

Element contributions carry an `ElemOrder`; at L2 the build turns it into
`OrderKey { class_rank, slot_index, sub_index }` using `Rules.slot_layouts` and the rules of 01 §4.10-4.11:

| `ElemOrder` | Position | Default for |
|---|---|---|
| `Slot` | the slot the item sits in: main slots 0-7 by index, exilus after slot 7 (OQ-024), riven handled below | primary elements (HEA, COL, ELE, TOX) on weapon mods |
| `Riven` | the riven's slot, lines in listed order with the bottom-up rule of `F:mods` §8.5 (sub-index) | primary elements on riven traits |
| `Innate` | after modded elements, together with innate and progenitor elements, in Heat > Cold > Electricity > Toxin order (D-004, D-006) | Violet shard elements, valence element, weapon innate elements |
| `NonCombining` | added to its type after combination (01 §4.10.3) | secondary elements on mods and rivens, Nourish, field elements, Valence Formation, Melee Exposure, heatAdd (p.aa) |

When `order` is omitted the validator derives it from the item kind and the element (table above) and rejects
combinations that make no sense (a combining order on a secondary element). The element combination algorithm of
01 §4.11 consumes the sorted `Vec<(OrderKey, DamageType, f64)>`.

---

## 14. Multiplier groups

`data/curated/rules/mult_groups.toml` declares every separate multiplier as a group. Members of one group add; groups
multiply. An unlisted weapon-side multiplier defaults to a new `g` group (04 §3.2 default rule).

```rust
pub struct MultGroupDef {
    pub id: Box<str>,                          // "eclipse", "chamber", "synth_charge", "mecha", "careen", "slam", "finisher" …
    pub stage: MultStage,                      // G | GF | DotOnly | OverguardOnly | Spot | TargetVuln
    pub dot: DotInheritance,                   // Inherit (part of the DoT seed) | Twice (Mecha, Vauban: seed and M_dot) | None
    pub twice_on_two_bucket: bool,             // Eclipse, Garuda on two-bucket melee projectiles (04 §3.4)
    pub vuln: Option<VulnMask>,                // TargetVuln groups only: types and layers the group multiplies (05 E-5.19);
                                               // a member with a different mask belongs to a different group
    pub sources: Vec<Box<str>>,                // citations
}
```

| `stage` | Spec placement |
|---|---|
| `G` | `mult_other` G (ST-5), multiplies the adding CO term |
| `GF` | `mult_final_omitted` G_F (ST-4), omitted by the adding CO term (E-11, D-058) |
| `DotOnly` | `M_dot,s` (E-13) |
| `OverguardOnly` | `M_OG` (E-12) |
| `Spot` | `M_son` (E-7), not inherited by DoT |
| `TargetVuln` | multiplicative members of `vuln[ℓ][t]` V (ST-8, 05 E-5.19), masked by the group's `vuln` |

```toml
[[group]]
id = "eclipse"
stage = "GF"
dot = "inherit"
twice_on_two_bucket = true
sources = ["04 §3.1", "W:Eclipse", "D-058"]

[[group]]
id = "chamber"          # Primed Chamber + Charged Chamber add inside one factor
stage = "G"
dot = "inherit"
sources = ["04 §3.2", "W:Primed_Chamber", "W:Charged_Chamber"]

[[group]]
id = "mecha"
stage = "G"
dot = "twice"           # second dip on DoT ticks (D-041)
sources = ["04 §3.2", "W:Mecha_Empowered"]
```

---

## 15. Curated TOML format and the effect-status lattice

### 15.1 File header

One file per item: `data/curated/<kind>/<family>/<slug>.toml` (pipeline.md §6.1). Common header:

```toml
schema = 1
id = "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod"   # UniqueName: the join key ("wfd:…" for our items)
name = "Serration"                                        # must equal the catalogue English name (checked)
kind = "weapon_mod"                                       # checked against the catalogue kind
status = "verified"                                       # verified | draft | unmodelled | no_damage_effect
sources = ["W:Serration", "F:core §2"]
upstream = { field = "wiki.Description", text = "+165% Damage", sha256 = "9b1c…" }   # written by `curate rebind`
ignore_numbers = []                                       # [{ token = "n5%", reason = "recoil" }]
excluded_lines = []                                       # [{ line = "<upstream line>", reason = "…" }] (04 §5.3)
notes = ""
```

`excluded_lines` lists the conditional stat lines of the upstream text ("On …", "While …", "for N s", …) that no
effect of the file implements: 04 §5.3 ("Uncatalogued conditional lines") excludes them from every bucket, never
applies them as unconditional, and the data build reports each one (`data.check.conditional_line_excluded`).

Non-catalogue items add kind fields: buffs `group`, `toggle`, `strength_scaled`, `duration_scaled` (04 §4.2
`dur_scaled`), `default_caster` (`self` default, `ally` for the T-13 Roar buff, T-37), `[[param]]`; shards `color`,
`tauforged_factor`;
Incarnon lines are one file per family (section 16.10); custom stats `inspiration_key`. An item-level `scope` is the
default scope of all its effects; an effect-level `scope` replaces it. Binding fields (`bind`, `bind_max`,
`bind_duration`, `bind_chance`) exist only in the authoring form; lowering removes them after the check.

### 15.2 Number binding

Every number of the upstream max-rank text is bound by an effect field or listed in `ignore_numbers`:

| Token | Meaning |
|---|---|
| `nK` | K-th number token of the upstream text, as written |
| `nK%` | K-th token divided by 100 |
| `nKx` | K-th token written as a multiplier ("x1.55"), stored as bonus `1.55 − 1` |
| `bind_max = "nK"`, `bind_duration = "nK"`, chance `bind = "nK%"` | stack count, duration and chance fields |

The validator checks every binding (value equals token after conversion), reports unbound numbers, checks every rank
against WFCD `levelStats` text, and uses the bindings to auto-patch number-only drift after a game update
(pipeline.md §7).

### 15.3 Effect-status lattice

| Status | Meaning | UI | Optimizer pool |
|---|---|---|---|
| `verified` | reviewed curated entry with sources | normal | yes |
| `auto` | tier-1 stat-line import or tier-2 spec-table ingestion, all lines mapped, validators clean | small "auto" badge | yes |
| `draft` | scaffolded or partially reviewed | "draft" badge | yes, marked in results |
| `stale` | upstream text changed since review; numbers auto-patched when only numbers changed | "auto-updated, not reviewed" badge | yes, marked |
| `unmodelled` | known to affect damage, not encoded yet (also every catalogue item with no effects) | "not modelled" badge; listed in `CalcReport.warnings` | no (I-6) |
| `no_damage_effect` | recoil, zoom without stat change, utility | dimmed | no |

---

## 16. Encoded examples

All values at max rank unless a curve is given. Values and citations come from the spec and the findings cited per
entry; the curator re-checks them through the binding validator. UniqueNames are from `Module:Mods/data` and
`Module:Arcane/data` as quoted by the proposals; abbreviated ones (`…`) are filled in by `curate scaffold`. Every TOML
block below parses with a TOML 1.0 parser.

**16.1 Serration** (base-damage bucket A; `F:core` §2):

```toml
schema = 1
id = "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod"
name = "Serration"
kind = "weapon_mod"
status = "verified"
sources = ["W:Serration", "F:core §2"]
upstream = { field = "wiki.Description", text = "+165% Damage" }

[[effect]]
stat = "base_dmg"
value = { max = 1.65, rank = "linear", bind = "n1%" }
```

Compiles to `StatVec[base_dmg] += 1_650_000_000` (1e-9 units) at max rank; rank r gives `1.65·(r+1)/11`.

**16.2 Galvanized Aptitude** (relative status chance plus a CO-family bonus with stacks; whether C is added or
multiplied is a property of the attack's `CoSpec`, not of the mod; `F:mods` §6.1, `F:addmul` §1.1-1.4, 04 §4.2):

```toml
schema = 1
id = "/Lotus/Upgrades/Mods/Rifle/WeaponStatusChanceSPMod"
name = "Galvanized Aptitude"
kind = "weapon_mod"
status = "verified"
sources = ["W:Galvanized_Aptitude", "F:mods §6.1", "F:addmul §1.1-1.4", "04 §4.2"]
upstream = { field = "wiki.Description", text = "+80% Status Chance\nOn Kill:\n+40% Direct Damage per Status Type affecting the target for 20s. Stacks up to 2x." }

[[effect]]
stat = "sc_rel"
value = { max = 0.80, bind = "n1%" }

[[effect]]
id = "co"
stat = "co_per_status"
value = { max = 0.40, bind = "n2%" }
per = ["stack"]                                   # C += 0.40 · stacks · n   (n enters through co_per_status)
when = { trigger = "on_kill" }
stacks = { max = 2, bind_max = "n4", duration = 20.0, bind_duration = "n3", stack_timer = "shared_lose_one" }
```

Derived class of `co`: (c) via the trigger and (a) via `co_per_status`. `all_max`: `co_per_status += 0.80`
(constant), times n per T-05 in the kernel. `none`: 0. `simulated`: `0.40 × u·m` (class (c) is an uptime input) times
`Distinct`. Rank 4 gives `0.80·5/11` and `0.40·5/11`.

**16.3 Condition Overload** (CO family, class (a), no stacks; slams get nothing because their attack's
`co.behaviour = none`; `F:addmul` §1):

```toml
schema = 1
id = "/Lotus/Upgrades/Mods/Melee/WeaponDamageIfVictimProcActive"
name = "Condition Overload"
kind = "weapon_mod"
status = "verified"
sources = ["W:Condition_Overload", "W:Condition_Overload_(Mechanic)", "F:addmul §1"]
upstream = { field = "wiki.Description", text = "+80% Melee Damage per Status Type affecting the target" }

[[effect]]
stat = "co_per_status"
value = { max = 0.80, bind = "n1%" }
scope = { weapon = { slots = ["melee"] } }
```

The stance-input CO ratio r_in (`F:addmul` §1.10) is table 9-D (`data/curated/rules/melee/stance_co.toml`, 07 §7.6),
applied by the engine per strike.

**16.4 Primary Deadhead** (weapon arcane: stacking base damage on weak-point kills, passive headshot multiplier;
`F:addmul` §5, 04 §4.2 where the timer kind is UNVERIFIED):

```toml
schema = 1
id = "/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnNoMelee"
name = "Primary Deadhead"
kind = "weapon_arcane"
status = "verified"
sources = ["W:Primary_Deadhead", "F:addmul §5", "F:core §2", "04 §4.2"]
upstream = { field = "wiki.Description", text = "On Weak Point Kill:\n+120% Damage for 24s. Stacks up to 3x.\n+30% Headshot Multiplier" }
scope = { weapon = { slots = ["primary"] } }

[[effect]]
id = "dmg"
stat = "base_dmg"
value = { max = 1.20, rank = "linear", bind = "n1%" }
per = ["stack"]
when = { trigger = "on_weak_point_kill" }
stacks = { max = 3, bind_max = "n3", duration = 24.0, bind_duration = "n2", stack_timer = "shared_lose_all" }
note = "UNVERIFIED: stack timer kind (04 §4.2)"

[[effect]]
stat = "hs_mult"
value = { max = 0.30, rank = "linear", bind = "n4%" }
```

`all_max` adds 3.60 to A. The optimizer's "no on-kill bonuses" rule zeroes the first effect only, not the arcane.

**16.5 Hunter Munitions and Internal Bleeding** (forced Slash on critical instances and on Impact procs, one
exclusive group; 02 §4.7.3-4.7.4, D-016, D-070):

```toml
schema = 1
id = "/Lotus/Upgrades/Mods/Sets/Hunter/PrimaryHunterMunitionsMod"
name = "Hunter Munitions"
kind = "weapon_mod"
status = "verified"
sources = ["W:Hunter_Munitions", "F:addmul §8", "02 §4.7.4", "D-016"]
upstream = { field = "wiki.Description", text = "+30% chance to apply a Slash Status Effect on Critical Hits" }

[[effect]]
mechanic = "proc_on_event"
value = { max = 0.30, rank = "linear", bind = "n1%" }

[effect.params]
on = "crit_instance"
proc = "SLA"
exclusive_group = "forced_slash"
```

Internal Bleeding: `on = "proc"`, `on_status = "IMP"`, `value = 0.35`, `chance_if = [{ fire_rate_below = 2.5,
factor = 2.0 }]`, same `exclusive_group`. The engine combines the group per instance as `1 − Π(1 − p_i)` and the
Slash proc counts as a stack (D-070).

**16.6 A riven** (user input, not a curated file). The Furis riven of `research/test-cases/user-example-furis.json`
as a `SlotContent::Riven` in JSON (model units: fractions):

```json
{ "slot": "gen7",
  "content": { "riven": { "class": "pistol", "traits": [
      { "trait": "critical_damage", "value": 1.114 },
      { "trait": "multishot",       "value": 1.481 },
      { "trait": "status_damage",   "value": 1.114 },
      { "trait": "zoom",            "value": -0.793 } ] } } }
```

Each trait maps once in `data/curated/rules/riven_traits.toml` to an effect template (04 §8.1), and its value is
validated against the riven value formula (04 §8.2, PE `upgradeEntries` bases):

```toml
[[trait]]
id = "critical_damage"
pe_tag = "WeaponCritDamageMod"
input = "percent"
template = { stat = "cd_rel" }

[[trait]]
id = "damage_to_grineer"
pe_tag = "WeaponFactionDamageGrineer"
input = "faction_multiplier"            # typed x1.45 → bonus 0.45; typed x0.55 → bonus −0.45
template = { faction = ["grineer", "kuva_grineer"] }

[[trait]]
id = "weak_point_critical_chance"       # D-010: a real stat, not ignored
pe_tag = "WeaponWeakPointCritChanceMod"
input = "percent"
template = { stat = "cc_rel", scope = { hit = { body_part = ["weak_point"] } } }

[[trait]]
id = "weak_point_damage"                # D-011
input = "percent"
template = { stat = "wp_dmg" }

[[trait]]
id = "heat"
pe_tag = "WeaponFireDamageMod"
input = "percent"
template = { element = "HEA", order = "riven" }

[[trait]]
id = "fire_rate"
input = "percent"
template = { stat = "fire_rate", scope_factor = [{ when = { weapon = { classes = ["bow"] } }, factor = 2.0 }] }

[[trait]]
id = "zoom"
input = "percent"                       # no template: no damage effect
```

**16.7 Roar** (buff with one parameter, faction bucket for any faction; T-13 enters the final value, 04 §4.4):

```toml
schema = 1
id = "wfd:buff/roar"
name = "Roar"
kind = "buff"
group = "ability"
toggle = "T-13"
status = "verified"
sources = ["W:Roar", "F:buffs §1.1", "T-13"]
inspiration_key = "o.r"

[[param]]
key = "value"
kind = "percent"
min = 0.0
max = 99.99
default = 0.30
helper = { formula = "rank_value * strength", rank_values = [0.10, 0.15, 0.25, 0.50] }

[[effect]]
faction = "any"
value = { param = "value" }
```

Eclipse (T-14) is the same shape with `mult = "eclipse"` (group stage `GF`, so adding-CO attacks omit it, D-058).

**16.8 Xata's Whisper** (extra hit, E-14, D-071):

```toml
schema = 1
id = "wfd:buff/xatas_whisper"
name = "Xata's Whisper"
kind = "buff"
group = "ability"
toggle = "T-15"
status = "verified"
sources = ["W:Xata's_Whisper", "W:Extra_Hit", "F:buffs §1.3", "E-14", "D-071"]
inspiration_key = "o.x"

[[param]]
key = "value"
kind = "percent"
min = 0.0
max = 99.99
default = 0.26

[[effect]]
mechanic = "extra_hit"
value = { param = "value" }
note = "UNVERIFIED: blast-detonation trigger magnitude (D-071); modelled in MC only"

[effect.params]
element = "VOI"
part = "twice"                          # E-14: (Xata ? Part : 1)
status = "roll_weapon_sc"
triggers = ["pellet_hit", "blast_detonation"]
```

Toxic Lash (T-16): `element = "TOX"`, `part = "once"`, `status = "forced"`, `melee_factor = 2.0`,
`status_duration_add = 0.25` (Saryn passive, D-037).

**16.9 Violet archon shard, Primary Electricity** (non-mod element ordered with innate elements; value depends on
the number of Crimson, Azure and Violet shards; Tauforged ×1.5; `F:buffs` §5, D-006, OQ-024):

```toml
schema = 1
id = "wfd:shard/violet/primary_electricity"
name = "+30% Primary Electricity Damage"
kind = "shard"
color = "violet"
tauforged_factor = 1.5
status = "verified"
sources = ["W:Violet_Archon_Shard", "F:buffs §5", "D-006", "OQ-024"]
inspiration_name = "+30% Primary Electricity Damage"

[[effect]]
element = "ELE"
order = "innate"
value = { base = 0.30, add = [{ coeff = 0.10, per = [{ shard_count = ["crimson", "azure", "violet"] }] }] }
scope = { weapon = { slots = ["primary", "arch_gun"] } }
```

Check: five normal Violet shards give `5 × (0.30 + 0.10 × 5) = 4.00`, the 400 % of `F:buffs` §5. The shard-count
term is static, so the optimizer enumerates shard multisets as one compiled vector each.

**16.10 Incarnon evolution perks, Furis line** (one file per family; per-variant values, per-form scope, a resolved-
stat condition; tier numbers and values per `W:Furis_Incarnon_Genesis`, re-checked by the curator):

```toml
schema = 1
family = "furis"
variants = { furis = "/Lotus/Weapons/Tenno/Pistol/AutoPistol", mk1_furis = "…" }
sources = ["W:Furis_Incarnon_Genesis", "F:mods §9"]

[[perk]]
id = "wfd:perk/furis/haven_foray"
name = "Haven Foray"
tier = 2
index = 0
status = "verified"

  [[perk.effect]]
  stat = "incarnon_flat"
  value = { variants = { furis = 28.0, mk1_furis = 34.0 } }
  scope = { form = "normal" }

[[perk]]
id = "wfd:perk/furis/extended_volley"
name = "Extended Volley"
tier = 3
index = 1
status = "verified"

  [[perk.effect]]
  stat = "magazine_flat"
  value = 25.0
  scope = { form = "normal" }            # "does not apply to the Incarnon form"

[[perk]]
id = "wfd:perk/furis/prelude_of_might"
name = "Prelude of Might"
tier = 4
index = 1
status = "verified"
upstream = { field = "wiki.Genesis", text = "Increase Base Critical Damage Multiplier by +3x when Critical Chance is below 40%" }

  [[perk.effect]]
  stat = "cd_base_add"
  value = 3.0
  when = { attack = { stat_below = { stat = "cc", x = 0.40 } } }   # final CC incl. Puncture bonus
```

`incarnon_flat` enters `B'` but not `B_ref` (E-1, E-11). The `attack` condition marks the CC slots `Unknown` for this
scenario, so the optimizer branches on CC items instead of bounding them.

**16.11 Blood Rush** (relative crit chance per combo tier above 1; combo model; `F:addmul` §2):

```toml
schema = 1
id = "/Lotus/Upgrades/Mods/Melee/Event/ComboCritChanceMod"
name = "Blood Rush"
kind = "weapon_mod"
status = "verified"
sources = ["W:Blood_Rush", "F:addmul §2"]
upstream = { field = "wiki.Description", text = "+40% Critical Chance per Combo Multiplier" }

[[effect]]
stat = "cc_rel"
value = { max = 0.40, rank = "linear", bind = "n1%" }
per = ["combo_tier_above_one"]          # CC = CC0·(1 + ΣCC_rel + 0.4·(M − 1)) + ΣCC_abs
scope = { weapon = { slots = ["melee"] } }
```

The heavy-attack doubling of other melee crit mods is `scope_factor = [{ when = { attacks = { any_of = ["heavy"] }
}, factor = 2.0 }]`.

**16.12 Arcane Avenger** (warframe arcane; absolute crit chance; external trigger with chance and duration; class
(c); E-5):

```toml
schema = 1
id = "/Lotus/Upgrades/CosmeticEnhancers/Defensive/CritChanceOnDamage"
name = "Arcane Avenger"
kind = "frame_arcane"
status = "verified"
sources = ["W:Arcane_Avenger", "F:core §8", "F:addmul §5", "F:buffs §7"]
upstream = { field = "wiki.Description", text = "On Damaged:\n21% chance for +45% Critical Chance for 12s" }

[[effect]]
stat = "cc_abs"
value = { max = 0.45, rank = "linear", bind = "n2%" }
when = { trigger = "on_damaged", chance = 0.21, bind_chance = "n1%" }
stacks = { max = 1, duration = 12.0, bind_duration = "n3", refresh = "refresh" }
```

**16.13 More items, one effect line each** (all from the spec's bucket lists; each becomes a fixture row of the
coverage test):

| Item | Encoding |
|---|---|
| Primed Chamber, Charged Chamber | `mult = "chamber"`, value 1.00 / 0.40, `when = { shot = "first_of_magazine" }` (04 §3.2) |
| Synth Charge | `mult = "synth_charge"`, value 2.00, `when = { shot = { last_of_magazine = { min_base_magazine = 6 } } }`, `scope = { attacks = { none_of = ["beam", "incarnon_form", "whole_magazine"] } }` |
| Eclipse (T-14) | `mult = "eclipse"`, `value = { param = "value" }` |
| Nourish (T-17) | `element = "VIR"`, `order = "non_combining"`, `value = { param = "value" }` |
| Flat crit chance (T-18) | `stat = "cc_abs"`, `value = { param = "value" }` |
| Phoenix Talons (T-19) | `stat = "phoenix_talons"`, value 0.30 (01 §4.9, D-007) |
| Secondary Shiver | `stat = "co_per_cold_stack"`, value 0.45, `scope = { weapon = { slots = ["secondary"] } }` (D-060) |
| Kunai Incarnon low-health perk | `stat = "co_flat"`, `when = { target = { health_below = 0.5 } }` (layered pass only) |
| Spiteful Defilement | `stat = "cd_abs"`, value 1.0, `when = { target = { distinct_below = 3 } }` (ΣCD_abs per D-067 as amended and 02 §4.3, OQ-406 closed) |
| Vigilante set | set bonus `stat = "tier_raise"`, value 0.05, `per = ["set_piece"]` (04 §9) |
| Gladiator set | set bonus `stat = "cc_rel"`, value 0.10, `per = ["set_piece", "combo_tier_above_one"]` (04 §9) |
| Sacrificial set | `member_scale = [1.0, 1.25]` on the set (index = pieces − 1) |
| Primary Acuity | `stat = "wp_dmg"`; `stat = "cc_rel"` with `scope = { hit = { body_part = ["weak_point"] } }`; `stat = "lock_multishot"` |
| Semi-Rifle Cannonade | `stat = "base_dmg"` + `stat = "lock_fire_rate"`; item `compat.trigger = "semi"` |
| Killing Blow | `stat = "base_dmg"`, `scope = { attacks = { any_of = ["heavy"] } }` |
| Melee Doughty | `stat = "cd_rel"`, `per = [{ resolved = { stat = "sc_of.PUN", curve = { step = 0.10 } } }]` |
| Primary Plated Round | `stat = "base_dmg"`, value 0.15, `per = [{ resolved = { stat = "magazine_normal_form", curve = { pow = { scale = 5.0, exp = 0.5 } } } }, "reloaded_fraction"]`, `when = { trigger = "on_reload" }`, `stacks = { max = 1, duration = 10.0 }` (04 §5.4, TV-04-16, EC-04-87) |
| Laetum Overwhelming Attrition | `stat = "base_dmg"`, value 12.0, `scope = { hit = { outcome = { crit = "none", status = "none" } } }` (04 §5.12, D-410) |
| Phenmor Lingering Judgement | `stat = "hs_mult"`, value 0.50, `when = { trigger = { count_within = { of = "on_headshot", n = 2, window = 2.0 } } }`, `stacks = { max = 1, duration = 8.0 }` (04 §5.12) |
| Melee Animosity | `mechanic = "counter_bonus"`, params `stat = "cc_rel"`, `per_event = { input = "light" }`, value 0.42, `max = 10`, `reset_on = "heavy"`, `applies_to = { attacks = { any_of = ["heavy"] } }` (04 §5.12, EC-07-28) |
| Power Throw | `mechanic = "counter_bonus"`, params `stat = "base_dmg_universal"`, `per_event = "throw"`, value 1.00, `max = 3`, `reset_on = "other_input"` (04 §3.4, EC-07-42) |
| Magus Accelerant | `stat = "vuln_add"`, value 0.65 per application (`per = ["stack"]`, `stacks = { max = 1, unbounded = true }`: a user count with no limit, default 1, class (c); ruling C-4), `vuln = { types = ["HEA"] }`; Magus Destruct the same with `["PUN"]` (04 §5.5) |
| Theorem Contagion | `stat = "vuln_add"`, value 2.00, `vuln = { types = { param = "element" } }`, class (c), 6 s (04 §5.5) |
| Chromatic Blade | `[[param]] key = "element"` (choices HEA, TOX, COL, ELE); `mechanic = "convert_damage"` with `from = "all"`, `to = { param = "element" }`; status chance `per = ["strength"]` |
| Reactive Storm, Sentient Incision | `mechanic = "convert_damage"` / `element = "target_weakness"` (01 §4.10.4 map) |
| Corrosive Projection | `stat = "strip_aura"`, value 0.18 per aura (E-15) |
| Emerald shard Corrosion cap | `stat = "stack_cap.COR"`, value 2 (Tauforged 3: `tauforged_values = [3]` overrides the ×1.5 rule for integers) |
| Secondary Fortifier | `mult = "fortifier"` (stage `OverguardOnly`) |
| Mecha Empowered | `mult = "mecha"` (stage `G`, `dot = "twice"`), `when = { target = "marked" }` (D-041) |
| Melee Careen | `mult = "careen"`, value 1.5, `when = { target = "frozen" }` (D-068) |
| Devouring Attrition | `mechanic = "random_multiplier"`, value 0.5, params `factor = 21.0`, `on = { crit_tier_eq = 0 }` (G, 04 §5.12) |
| Galvanized Scope | `stat = "cc_rel"` with `when = { trigger = "while_aiming" }` and `stacks = { stack_timer = "independent", … }` gained `on_weak_point_hit` |
| Melee Duplicate | `mechanic = "repeat_hit"`, params `on = { crit_tier_eq = 1 }` (tier-1 crits only, 04 E-4.15; weight `1 + p_dup·P(k = 1)`, 08 E-8.12) |
| Custom stat `p.b` (Base Damage per Status) | `wfd:custom/base_per_status`: `stat = "co_per_status"`, `value = { param = "value" }` |
| Custom stat `p.x` (Damage Vulnerability) | `wfd:custom/dmg_vulnerability`: `mult = "custom_vuln"` (stage `TargetVuln`) |

---

## 17. Coverage matrix: spec mechanics to encoding

Every row has a fixture item in `data/fixtures/mini/` and a hand-computed expected value in
`wfd-engine/tests/effects_coverage.rs` (testing.md §3).

| Spec mechanic | Encoding | Fixture |
|---|---|---|
| Base bucket A incl. negatives (00 §3.1a) | `stat = "base_dmg"` | Serration, negative riven damage |
| Melee-projectile second bucket (04 §3.4) | `base_dmg_universal` + role scope | Arcane Arachne on a glaive throw |
| Physical P_t, stance physical | `phys.*`; stance data | Rupture; stance hit with Slash bonus |
| Elements with slot order, riven bottom-up, exilus, innate HCET | `element` + `ElemOrder` | Infected Clip + Stormbringer; riven Heat/Cold; Kuva innate + progenitor |
| Non-combining and already-combined elements | `order = "non_combining"` | Nourish, Biotic Rounds |
| Progenitor V_t (T-24) | `VariantDoc.progenitor` + `Weapon.progenitor` | Kuva weapon at 0.60 Toxin |
| Incarnon flat X, excluded from B_ref | `incarnon_flat` | Haven Foray |
| Exalted Strength scaling | `Weapon.exalted` + `Per::Strength` | exalted fixture weapon |
| Faction F, sub-factions, Roar, riven faction | `faction` + `Faction.bane_of` | Primed Bane vs Kuva Grineer; Roar |
| CO adding / multiplying / none, ρ_co, stance ratio | `co_*` + `Attack.co` + table 9-D r_in | Galvanized Aptitude on hitscan, on Arca Plasmor, on a slam, Paris charged |
| CO per Cold stack; CO under a health threshold | `co_per_cold_stack`; `co_flat` + `health_below` | Secondary Shiver; Kunai perk |
| G_F and G multipliers, additive pairs | `mult` groups | Eclipse; Primed + Charged Chamber |
| First and last shot | `Cond::Shot` | Primed Chamber, Synth Charge |
| Heavy-only, slam, finisher | role scopes; `mult = "slam"`, `"finisher"` | Killing Blow, Nira's set, Covert Lethality |
| Body part ΣWP (1.5× on head), ΣHS, Sonar | `wp_dmg`, `hs_mult`, `mult` stage `Spot` | Primary Acuity, Deadhead, Sonar |
| CC relative, absolute, floor, base add, custom final factor | `cc_*` | Point Strike, Arcane Avenger, Smeeta Charm, Piercing Navigator, custom p.d |
| Combo-scaled CC/SC, Gladiator | `per = combo_tier_above_one`, set bonus | Blood Rush, Weeping Wounds, Gladiator ×3 |
| CD relative, absolute before tiers (D-014), base add, field factor | `cd_*` | Vital Sense, Arcane Crepuscular, Prelude of Might, Electric Shield |
| Tier raise (Vigilante) | `tier_raise` | Vigilante ×6 |
| SC relative, absolute, base add, factor | `sc_*` | Rifle Aptitude, custom p.l, custom p.k |
| Forced procs, Hunter Munitions, Internal Bleeding | `forced.*`, `proc_on_event` + exclusive group | trait weapon; both mods together |
| Multishot and Acuity lock | `multishot`, `lock_multishot` | Split Chamber, Primary Acuity |
| Status damage global and per type | `status_dmg`, `status_dmg_type.*` | Elementalist, Emerald shard |
| Status duration and caps | `status_dur`, `stack_cap.*` | Lasting Sting, Emerald Corrosion cap |
| DoT-only multipliers M_dot | `mult` stage `DotOnly`, `dot_vuln` | custom p.o, Mecha second dip |
| Extra hits | `extra_hit` | Xata's Whisper, Toxic Lash |
| Melee Duplicate, Devouring Attrition | `repeat_hit` (tier = 1), `random_multiplier` (tier = 0) | both |
| Hit-outcome gate | `scope.hit.outcome` | Overwhelming Attrition |
| Stat from a resolved stat with a power curve, reloaded fraction | `Per::Resolved { Pow }`, `Per::ReloadedFraction` | Primary Plated Round (TV-04-16) |
| Counters per input or throw, consumed by a heavy or another input; n events within a window | `counter_bonus`, `TriggerId::CountWithin` | Melee Animosity, Power Throw, Phenmor Lingering Judgement |
| Vulnerability per layer and type (multiplicative and additive group) | `mult` stage `TargetVuln` with `vuln` mask, `vuln_add` with `vuln` mask (54 slots) | Molecular Prime, custom p.x, Magus Accelerant (Heat only), Theorem Contagion (param type) |
| Critical promotion on body parts | `BodyPart.tier_add` | Rogue Culverin canister (TV-02-04) |
| Armor strip (ability, aura), external Viral | `strip_ability`, `strip_aura`, T-06 → `external.VIR` | Corrosive Projection, T-07, T-06 |
| Overguard-only multiplier | `mult` stage `OverguardOnly` | Secondary Fortifier |
| Fire rate, lock, charge, magazine, reload, ammo efficiency | `fire_rate`, `lock_fire_rate`, `charge_rate`, `magazine(_flat)`, `reload_speed`, `ammo_eff` | Speed Trigger, Cannonade, Extended Volley |
| Melee timing stats | `attack_speed`, `initial_combo`, `combo_dur`, `heavy_eff`, `wind_up` | Fury, Ready Steel, Focus Energy |
| Combo tables, unique overrides, strikes, synthesized heavy, plans | `ComboTable`, `MeleeData.unique`, `Strike`, `HeavyDef`, `MeleePlanSel` (T-30) | Nikana Prime with Guiding Light (TV-07-01/02), Machete heavy (TV-07-10), Blind Justice normalisation (TV-07-12), build_spend and tennokai (TV-07-13/14) |
| Lingering children, delayed explosions | `Attack.lingering`, `explosion_delay`, `embed_delay` | Torid cloud, Lenz bubble (06 §9.3-9.4) |
| Zoom levels | `Weapon.zoom` + `toggles.zoom_level` gated by `while_aiming` (D-406) | Rubico |
| Sniper combo, beam ramp, spool, falloff, semi cap | attack data + rules + T-12, T-27 | sniper, beam and spool fixture weapons |
| Set bonuses (count-based, member scale) | `SetDef` | Vigilante, Gladiator, Sacrificial |
| Resolved-stat conditions and stat-from-stat | `Cond::Attack`, `Per::Resolved` | Prelude of Might, Melee Doughty |
| Target-dependent element | `element = "target_weakness"` | Sentient Incision |
| Item parameters | `ParamSpec`, `convert_damage { to = param }` | Chromatic Blade |
| Custom stats (26) and toggles (T-01 … T-43) | custom-stat items, `Toggles` + `TOGGLE_MAP` | one per custom stat |
| Buff caster Strength | `BuffPick.caster`, `Per::Strength` (04 E-4.10) | an ally-cast Strength-scaled buff |
| Kitgun and zaw assembly | `ModularTables` + `assemble` | Catchmoon parts; a zaw |
| Incarnon form attack and perks per form | role `INCARNON_FORM`, `IncarnonDef`, `scope.form` | Furis line |
| Heat inherit (D-028) | `heat_inherit` | custom p.y |
| Phoenix Talons (D-007) | `phoenix_talons` | T-19 buff |

---

## 18. Lowering: from effects to compiled contributions

`wfd-engine::compile::item` (engine L1) lowers one item in a scenario context to:

```rust
// wfd-engine/src/compile/item.rs
pub struct CompiledItem {
    pub per_attack: SmallVec<[SparseContrib; 2]>,   // one per objective attack (scope resolved per attack)
    pub drain: i8, pub incompat_mask: u64, pub set: Option<SetId>,
    pub touched: StatMask,                          // slots this item can change (dominance, relevance)
    pub has_unknown: bool,                          // touches an Unknown slot: composition candidate
}
pub struct SparseContrib {
    pub entries: SmallVec<[(u16 /* StatKey index */, i64 /* 1e-9 units */); 6]>,   // constants
    pub coeffs: SmallVec<[Coeff; 2]>,               // value · Π state variables (dynamic terms)
    pub elements: SmallVec<[ElemEntry; 2]>,         // (ElemOrder, DamageType or selector, value) — positions at L2
    pub mechanics: SmallVec<[MechanicInst; 1]>,     // parameters of mechanic kinds (magnitudes are in `entries`)
    pub statics: SmallVec<[StaticTerm; 1]>,         // set-piece, shard-count, resolved-stat terms (finalizers, L2)
    pub sources: SmallVec<[EffectRef; 4]>,          // for traces (item, effect index)
}
pub struct Coeff { pub slot: u16, pub coef: f64, pub vars: SmallVec<[StateVar; 2]> }
```

Galvanized Aptitude at rank 10 on a rifle under `simulated`, with the default uptime input (u = 1, m = 2): the `co`
effect is class (c) (stacks gained on kill), so it compiles to the constant `0.40 · u · m = 0.80` (04 §4.3; §12.2),
not to a `CondStacks` variable; kills are not simulated in either evaluator:

```text
entries:  [(sc_rel, 800_000_000), (co_per_status, 800_000_000)]     // 0.40·u·m; × Distinct in the kernel (class a)
coeffs:   []
```

With an override `Uptime { u: 0.5, stacks: Some(1) }` the constant is `0.40 · 0.5 · 1 = 0.20` (200_000_000). Under
`all_max` the entries are the same as with the default uptime (`0.40 × max = 0.80`); under `none` only `sc_rel`
remains. A class (b) stacking effect (Galvanized Scope's crit chance gained on weak-point hits) is the case that
compiles under `simulated` to `Coeff { slot: cc_rel, coef: v, vars: [CondStacks(#id)] }`. Bucket
totals are sums of contributions; fixed-point addition is associative, so the order of items never changes a result
(I-4), and adding or removing one item is an exact delta (optimizer.md §5).
