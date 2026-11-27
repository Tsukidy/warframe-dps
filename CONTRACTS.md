# Contract changes

Change log of the frozen interfaces of work-breakdown.md §2.1: `wfd-model` types, crate public APIs, engine internal
types, stage and mechanic signatures, MC state and event types, API DTOs, message keys, the curated TOML schema, the
bundle JSON layout, the xtask command list, and the workspace dependencies.

Protocol (work-breakdown.md §2.1): a task that needs a change to a frozen interface adds an entry below (date, task,
change, reason, affected tasks), makes the smallest backward-compatible change (new optional field, new enum variant
with a `#[serde(other)]` fallback, new function) and regenerates the TypeScript types. A breaking change needs the
manager's approval before it is merged. A new workspace dependency is an entry here.

Entries are newest last. Each entry names the documents it deviates from, so a reader of `docs/architecture/` knows
where the code differs.

---

## 2026-11-24 · T01a · Workspace and model types (initial contract)

T01a creates the contract; the items below are where it differs from or adds to `docs/architecture/domain-model.md`
and `overview.md` §3.1. Unless marked otherwise, each addition is required by a spec section (docs/mechanics wins over
the architecture, work-breakdown.md §2.4) and is listed again under "Requests to the manager" at the end.

### Workspace and dependencies

- Members as overview.md §3; edition 2024, resolver 3, `rust-version = "1.97"`. Release profile `lto = "fat"`,
  `codegen-units = 1`, `panic = "unwind"`. `[workspace.lints]`: `unsafe_code = "forbid"`; `wfd-engine` and `wfd-opt`
  repeat the lints in their own `[lints]` table and add `clippy::float_cmp = "warn"` (Cargo cannot combine
  `workspace = true` with crate-level lints).
- Every version of overview.md §3.1 exists on crates.io and was used as the newest release of its line
  (`cargo search`, 2026-11-24), all checked to resolve and build together with Rust 1.97 (a probe crate depending on
  all of them passed `cargo check`, mlua's vendored Lua included): serde 1.0.229, serde_json 1.0.154, bitflags 2.13.2,
  smallvec 1.16.6 (1.x, not the 2.0 beta), arrayvec 0.7.8, ts-rs 12.0.1, schemars 1.2.3, toml 1.2.2, postcard 1.2.0,
  ruzstd 0.9.1, arc-swap 1.9.5, blake3 1.8.7, zstd 0.14.4, libm 0.2.16, rayon 1.12.0, rustc-hash 2.1.3,
  xxhash-rust 0.8.21, parking_lot 0.13.0, miniz_oxide 0.9.1, base64 0.23.1, lz-str 0.2.1, mlua 0.12.3, ureq 3.5.2,
  lzma-rs 0.3.0, regex 1.13.2, sha2 0.11.1, clap 4.6.9, include_dir 0.7.4, axum 0.8.9, tokio 1.54.1, tower-http 0.7.1,
  mime_guess 2.0.5, webbrowser 1.2.7, directories 6.0.0, tracing 0.1.44, tracing-subscriber 0.3.23,
  tracing-appender 0.2.5, proptest 1.12.0, insta 1.49.0, criterion 0.8.2. No substitutions were needed.
- Declared in addition to overview.md §3.1: `anyhow` 1.0.105 (named by T02d's `serve` signature), `thiserror` 2.0.21
  (error types), `tower` 0.5.3 with `util` and `http-body-util` 0.1.5 (testing.md §1: server tests use
  `tower::ServiceExt::oneshot`).
- Features chosen: smallvec `serde`, `union`, `const_generics`; arrayvec `serde`; ts-rs `arrayvec-impl`,
  `serde-json-impl`, `no-serde-warnings` (the dual-encoding types below use `#[serde(remote = "Self")]`, which ts-rs
  cannot parse and would print a note for on every build); schemars `smallvec1`, `arrayvec07`; postcard `use-std`;
  mlua `lua51`, `vendored`, `send`; clap `derive`; tokio `rt-multi-thread`, `macros`, `net`, `signal`, `sync`, `time`;
  tower-http `trace`, `limit`, `catch-panic`; tracing-subscriber `env-filter`; xxhash-rust `xxh3`; insta `json`.
- `wfd-engine` and `wfd-datagen` are declared with `default-features = false` in `[workspace.dependencies]`, so that a
  dependent crate opts in to `parallel` (rayon) and `refresh` explicitly (overview.md §3.1).
- `.cargo/config.toml` with `[alias] xtask = "run --package xtask --"` was first left out (no owner in
  work-breakdown.md §8); it is created by the review fixes below (C-16).

### Encodings (affects every task that adds a bundle type)

- Every model type has a readable form (JSON, TOML) and a compact form (postcard, the embedded bundle). postcard is
  positional and not self-describing, so **bundle types must not use untagged, internally tagged or flattened enums,
  `skip_serializing_if`, or `serde_json::Value` directly**. Types whose readable form needs such shapes derive the plain
  form with `#[serde(remote = "Self")]` and get their impls from `serde_util::dual_serde!` (readable form = a `…Repr`
  type; TypeScript `#[ts(as = "…Repr")]`). `serde_json::Value` fields use `rules::json_value_codec` (JSON text in the
  compact form). Flag sets (`bitflags`) serialize as name lists in the readable form and as bits in the compact form
  (`serde_util::flags_serde!`); `CategorySet` (data-defined bits) as a list of bit indices.
- Dual-encoded types: `EffectDef`, `ElemSel`, `FactionSel`, `ValueExpr`, `ResolvedCurve`, `Cond`, `TypeSel`,
  `DisplayValue`, `ParamSpec`, `ParamValue`, `Riven`. Their readable forms are the TOML forms of domain-model.md §16.
- Document types (`CalcInput` and everything below it) are JSON only and omit empty optional fields.
- The test `crates/wfd-model/tests/documents.rs` round-trips a fully populated `Bundle` through JSON and postcard; a
  task that adds a bundle type should add it there.

### Identifiers, enums (domain-model.md §2-§3)

- Rust field `gen` is the raw identifier `r#gen` (`Build.r#gen`, `FrameLoadout.r#gen`): edition 2024 reserves `gen`.
  The JSON name stays `gen`.
- `Polarity` has no `Aura` variant: the wiki slot string `Aura` is the "Any" symbol (09 §6.1), so `"aura"` deserializes
  to `Polarity::Any`. "No polarity" is `Option::None` at the use site.
- `WeaponClass` + `Crossbow`, `DualShotguns`, `Tome` (09 §3.1 classes). `WeaponTraits` + `PSEUDO_EXALTED`
  (`POWER_WEAPON_LITE`, 09 E-9.1), `NECRAMECH` (09 E-9.30). `ArcaneClass` + `Akimbo` (09 E-9.30). `AttackRoles` +
  `WALL`, `EXPLOSION` (07 §9.4), `GAUGE_GATED` (06 E-6.14a), `RETURNING` (06 §6.4), `ZOOM_ONLY`, `HEADSHOT_ONLY`
  (06 §9.1-9.2).
- `MeleeClass` is concrete: the 28 classes of 07 table 9-A (Bayonet included), `ExaltedWeapon` and `ArchMelee`, with
  `wiki_name`/`from_wiki`. New enums: `RivenColumn` (04 §8.3), `InputKind` (07 §7.5), `N_COMBO_TYPES`.
- `DamageType`/`StatusType`/`Layer` serialize as their spec IDs (`"HEA"`, `"OG"`), as the §16 examples do.

### Stat registry (domain-model.md §10)

- `StatKey` has one variant per registry row; multi-slot rows carry a typed sub-index (`StatKey::Forced(StatusType)`,
  `StatKey::VulnAdd(LayerType)`, `StatKey::Mult(MultSlot)`, …), so every value is exactly one slot. Names:
  `"forced.SLA"`, `"vuln_add.HP.HEA"`, `"xh.3"`, `"mech.7"`, `"mult.12"`, `"display.zoom"`.
- `StatMeta` has the five fields of work-breakdown.md T01a plus `slots`, `offset` and `enters`.
- Rows added because the spec defines the bucket: `status_dur_type` ×18 (03 §4.1), `projectile_speed` and `radius`
  (06 E-6.20, E-6.22: they change falloff, so projectile speed is not a `display` stat), `sniper_combo_dur`
  (06 E-6.16), `combo_dur_flat` (07 E-7.12), `combo_acc`, `combo_rc`, `combo_curse`, `combo_gain_ext` (07 E-7.13,
  §2), `tennokai_chance` (07 E-7.17). `N_STATS` = 296. The 12 `display` slots are `range`, `punch_through`, `zoom`,
  `recoil`, `accuracy`, `spread`, `ammo_max`, `noise`, `holster_speed`, `follow_through`, `block_angle`,
  `ammo_pickup`.

### Effect DSL (domain-model.md §11-§12)

- `EffectDef` + `tauforged_values` (named by §16.13). An empty `id` means "assigned at lowering" (`e<index>`); a
  missing readable `value` means `{ param = "value" }` (riven templates, buffs, custom stats). `TableBy` is written
  `{ scalar = …, values = […] }`; `ResolvedCurve::Step` is written `{ step = 0.10 }` as in §16.13.
- `Cond::Trigger` + `hit_unit` (04 §4.1-4.2) and `bind_chance` (authoring). `TriggerId` + `OnBigCrit`, `OnFire`,
  `OnMiss`, `OnDodge`, `HealthLevel` (all listed by 04 §4.1). `StackModel` + `consume_on`, `reset_on` and the decay
  `Idle { t_idle, rate }` (04 §4.2), `bind_max`, `bind_duration`; `duration` defaults to 0 (no timer, user-driven
  stacks). `WeaponFilter` + `melee_classes`, `traits` (04 §4.2 `weapon_class_scope`). `ResolvedStat` is a concrete
  list (`cc`, `cd`, `sc`, `sc_of.<status>`, `multishot`, `fire_rate`, `magazine`, `magazine_normal_form`, `reload_time`,
  `attack_speed`, `combo_dur`, `initial_combo`, `base_total`).
- `ClassSet` is a flag set `a`, `b`, `c`, `u` (04 §4.1 class (u)), `combo`, `schedule`, `formula`, `gate`, `static`;
  `EffectDef::derived_class()` implements the table of §12.1 once for T06 and T19.
- Mechanic kinds (§11.2): the magnitude is always the effect `value`, so the `chance` parameter of `repeat_hit` and
  the `amount` parameter of `flat_damage_on_proc` are the value, not separate params. `[effect.params]` is parsed into
  the union `MechanicParamsRepr` and checked per kind (fields of another kind are errors). Parameter types:
  `ExtraHitParams`, `ProcOnEventParams` (`proc` = status, `{ random = [...] }` or `"component_of_combined"`),
  `ConvertDamageParams` (`from` = `"all"` or types, `to` = `ElemSel`), `RepeatHitParams`, `FlatDamageOnProcParams`,
  `ProcAfterTicksParams`, `CounterBonusParams` (+ `hit_unit`), `RandomMultiplierParams`, `SpawnAttackParams`
  (`template: SpawnTemplate`, `on` = `hit` | `detonation` | `bounce`).

### Weapons, enemies, items, rules (domain-model.md §4-§8)

- `Weapon` + `riven_class`, `riven_family` (09 E-9.50, E-9.51). `Attack` + `reload_override` (06 §6.4 attack-level
  `Reload`), `range` (06 §8.4). `FireSpec` + `charge_fixed` (06 E-6.4), `burst_reload_delay` (06 §5.8), `gauge`
  (06 E-6.14a). `AttackGroup` + `volley` (06 E-6.14b). Concrete shapes: `Spool`, `Held`, `KohmSpec`, `PeacemakerSpec`,
  `PartialCharge` (display only, 06 §6.4), `GaugeSpec`, `SniperComboSpec { min: Option, reset }`, `WeaponDisplay`,
  `DisplayValue`, `Provenance`, `SourceRef`.
- `CoSpec` + `mproj: MprojMode` (04 §3.4); `CoRef` + `Uncharged`, `Fixed` (04 §2.6; serde `"self"` for `SelfAttack`);
  `CoFlags` = the catalogue flags of 04 §2.14 (`untested` = `co_tested = false`). `CoSpec::default()` is rule D6.
- `MeleeData` + `fixed_stance` (09 E-9.2, 07 E-7.0b). `Strike` and `HeavyStrike` + `cc: [CrowdControl]` (crowd-control
  names stay on the hit record, 07 §4.1). `ComboTable` + `weapon_type` (07 E-7.0b). `ComboSpecials` fields are
  `Option` overrides (`None` = the 07 E-7.10 default). Concrete rows: `HeavyClassRow`, `MeleeTriggerRow` (+
  `MeleeTriggerSource`, `TriggerCount`, `TriggerScaling`), `StanceCoRow` (cells per combo, one value or one per input),
  `FinisherRow`. Modular parts: `KitgunChamber` (+ `ChamberGrip`), `ModularPart` (+ `ModularPartKind`, `ZawHand`),
  `ZawStrike` (+ `ZawMulti`).
- `Enemy` + `damage_faction`, `spawn_level`, `sp_shield_bonus`, `archimedea_health_bonus`, `dr_class`, `dr_class_og`,
  `party_scaling`, `defaults`, `overguard_context` (05 §4.1, §7, §14); `EximusData` + `armor`, `overguard`, `og`;
  `BodyPart` + `column`, `crit_disabled`, `wp_bonus_ok`, `part_dr`, `is_default` (05 §4.3); `EnemyFlags` concrete
  (05 §4.1 enumerated fields as flags).
- `Item` + `effect_tier`, `auto_patched` (pipeline.md §6.2, §7). `ItemKind::FrameMod { exilus }` (09 E-9.3: warframe
  exilus mods); `ItemKind::Buff` + `inspiration_key` (§16.7 header field). `CompatRule` concrete (09 §3: slot type,
  categories, named weapons, tags, forbidden tags, X-EXALT lift and requirement tags, trigger, base-magazine
  requirement and exemptions). `ParamSpec.label` is `Option` (`None` = derived from the key). `SetDef` +
  `unique_name`, `noncounting` (09 E-9.40). `IncarnonDef` + `native` (09 E-9.60). `RivenTraitDef` + `coeff` per
  `RivenColumn`, `sign`, `spliced`, `recipes`, `inspiration_key` (04 §8.3, 09 §12); `label` is `Option`.
- `Riven` + optional `rank`, `polarity`, `drain`, `family` (09 §12.2, E-9.59); omitted from JSON when absent.
- `Rules` + `game_constants` (named game values, `"<equation id>.<name>"`). Concrete shapes: `StatusRule`
  (+ `StatusStacking`), `ScalingClass` (+ `ScalingCurve`), `SlotLayout` (+ `EquipmentClass`, `SlotDef`, `SlotKind`,
  `ArcaneFamily`), `CoDefaults` (D2-D6 of 04 §2.5), `BodyPartRules` (+ `FactionHeadRule`), `SpecialDrDef`
  (+ `Attenuation`, `Adaptation`), `RivenFormula` (+ `RivenLayoutRow`), `ToggleDescriptor` (+ `field`, the document
  path of server-api.md §5.6), `ToggleGroup` (string), `ControlKind` (struct: `kind`, range, options),
  `VisibilityRule`, `HelminthRules` (+ `HelminthHost`), `CategoryDef`, `TagDef`. `MultStage` names: `"G"`, `"GF"`,
  `"dot_only"`, `"overguard_only"`, `"spot"`, `"target_vuln"`.
- `Bundle` as §8; `BUNDLE_SCHEMA_VERSION = 1`; `LicenceNotice.modules` revision ids are `u32`; `Coverage` and
  `NameTable` are newtypes over their maps.

### Documents (domain-model.md §9)

- `BuildDoc` + `companion_kind` (09 E-9.41). `WeaponItemDoc.genesis_installed` is `Option<bool>`: `None` = the 09 §2.1
  default (true when the weapon has an Incarnon line); domain-model.md had `bool` with default false.
- `Build` + `stance_rank` (09 E-9.15 needs the stance mod's rank), `companion_kind`; `Variant` holds part indices
  `(grip, loader)` / `(grip, link)` and the `ProgenitorPick`. `Resolved { build, scenario, violations }` is a model type.
- `EnemySetup` and `EnemyInstance` + `mission: Option<MissionModifier>` and `context_overguard` (05 §7.5, §4.1).
- `MeleePlanSel::HeavyLoop` + `heavy_cancel: Option<f64>` (t_dmg), new `CycleLoop` and `DataAttackLoop { t_cyc }`
  (07 §8.1).
- `TOGGLE_MAP` lists T-01 … T-32 (T-30 … T-32 `Proposed`); the zoom level has no T-id and is not listed. Places are
  `TogglePlace::{Scenario, Build, Buffs, OptRequest}(path)`.
- Constants `DOC_VERSION = 1`, `MAX_SEED = 2^53 − 1`.

### Handover

- **T01b** owns `crates/wfd-model/src/{api,message}/**`. Both modules are empty. (Superseded by the review fixes
  below: `MessageKey` and `MsgArg` live in the T01a module `crate::i18n`, not in `message/mod.rs`.)
- **T01c** owns `crates/wfd-model/src/compat_generated.rs`: generate only the `bitflags!` invocation of
  `CompatFlags: u128` with the derives `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default`, one `const`
  per `Compat` row and `INSPIRATION`; serde (flag names), TypeScript (`Array<string>`) and schema impls live in
  `compat.rs` (T01a) and work for any flag list.

### Requests to the manager (spec conflicts, work-breakdown.md §2.4)

Provisional choice for each: the spec's, as implemented above. Affected: T01b (DTOs mirror these types), T02a-T02d,
T03, T03b, T03c, T04, T06, T07, T08a, T08b, T18, T19, T20-T29.

| # | Conflict | Spec | Provisional choice |
|---|---|---|---|
| C-1 | `genesis_installed` default | 09 §2.1: true for weapons with an Incarnon line; domain-model.md §9.1: false | `Option<bool>`, `None` = spec default |
| C-2 | `Polarity::Aura` | 09 §6.1: the `Aura` slot string means Any | no `Aura` variant; alias to `Any` |
| C-3 | Trigger ids | 04 §4.1 lists `on_big_crit`, `on_fire`, `on_miss`, `on_dodge`, `health_level`; domain-model.md §11.1 says "exactly the ids of 04 §4.1" but omits them | added |
| C-4 | Stack fields | 04 §4.2 `consume_on`, `reset_on`, `decay = idle(…)`, `hit_unit`; `max_stacks = none` (unbounded until reset) | added, except unbounded stacks: `StackModel.max` stays `u16`; Secondary Enervate is `counter_bonus` (domain-model.md §16.13). Request: confirm that every `max_stacks = none` effect of 04 is a `counter_bonus` |
| C-5 | Enemy record | 05 §4.1 has fields domain-model.md §5 cannot express (damage faction, spawn level, SP shield and Archimedea bonuses, class DR, party scaling, defaults, context overguard, Eximus armor/overguard/og, part fields) | added |
| C-6 | Melee plans | 07 §8.1 `cycle_loop`, `data_attack_loop`, `heavy_cancel` | added to `MeleePlanSel` |
| C-7 | Riven instance | 09 §12.2 `{class, family, rank, polarity, drain, traits}`; domain-model.md §6 `{class, traits}` | optional fields added |
| C-8 | Companion kind | 09 E-9.41 `companion_kind` scenario input | `BuildDoc.companion_kind` (next to `companion_set_mods`) |
| C-9 | Buckets | 03 §4.1 per-type status duration; 06 E-6.20 projectile speed and radius change damage; 07 combo-gain buckets | registry rows added; projectile speed moved out of `display` |
| C-10 | Mission modifiers | 05 §7.5 (proposed in OQ-523), 05 §4.1 `context_overguard` | optional `EnemySetup` fields |
| C-11 | Class (u) | 04 §4.1 class (u), proposed to overview 6.2 in OQ-431 | `ClassSet::U` exists; `derived_class` leaves it to the caller (item kind) |
| C-12 | CO per parent variant | 04 §2.14 rows 27-28 (Mutalist Cernos "Toxin Cloud": multiplying ×1 from the uncharged shot, adding ×41 from the charged shot) need two `CoSpec`s for one data attack | not representable on one `Attack`; provisional: T18 emits one child attack per parent variant |
| C-13 | Stance rank | 09 E-9.15 needs the stance mod's rank; domain-model.md §9.2 `Build.stance` has none | `Build.stance_rank` |

---

## 2026-11-24 · T01a · Review fixes (before acceptance)

Changes made after the independent review of T01a. Where an item changes a shape described above, this entry wins.

### Effect DSL (domain-model.md §11-§12, §16.13, §17)

- New `Target::VulnAdd` (appended variant): the additive vulnerability group on every slot `vuln_add.<ℓ>.<t>` with ℓ in
  `vuln.layers` and t in `vuln.types` (54 slots), as domain-model.md §16.13 and §17 write it for Magus Accelerant and
  Theorem Contagion (parameter type). Readable form `stat = "vuln_add"` with a required `vuln` mask; the readable
  `stat` key is the new `StatName` (slot name or the row name `vuln_add`). A sub-indexed `stat = "vuln_add.HP.HEA"` is
  one slot and is rejected together with `vuln` (it could contradict the mask); `vuln` is rejected on every target but
  `vuln_add` and `mult` (whether the `mult` group is TargetVuln is a validator check, T19). The §16.13 rows are now
  encoded in `tests/domain_examples.rs` exactly as written. Affects T06 (lowering spreads the value over the mask; a
  `{ param = … }` type is resolved at L2), T19, T20-T29.
- `EffectDef::derived_class()` adds class (a) for targets the kernel multiplies by a target-state count
  (`StatKey::is_target_state_count`: `co_per_status`, `co_per_cold_stack`, also as the `stat` of a `counter_bonus`):
  Condition Overload and Secondary Shiver are (a), Galvanized Aptitude's CO is (a) + (c) (spec 00 §6.2; domain-model.md
  §12.1, §16.2-16.3). Affects T06 (T-02 compile: `none` switches them off), T19 (declared-class check).
- `VulnMask` defaults follow the spec: `layers` defaults to `{HP}` (05 §13 "An entry without an explicit mask applies to
  health only (Decision)"; 04 §4.2 `layer_mask` default `{HP}`; 04 §5.7), `types` defaults to every type (04 §4.2
  `type_mask` default `all`). domain-model.md §11.1 says "layers default all" (request C-14). `Scope.layers` keeps the
  all-layer default. The same default applies to `MultGroupDef.vuln`: T03b writes `layers = ["OG", "SH", "HP"]` for
  every-layer groups (custom p.x `custom_vuln`, 05 §13; the U37 list of 04 §5.7). Affects T03b, T06, T07.
- Rank curves for numeric fields other than the value (09 E-9.13 covers durations, stack caps, thresholds; X-RANK lists
  Primary Plated Round's duration table, Secondary Enervate's `r + 1` big crits, Double Tap's cap table), all optional
  `RankCurve`s, `None` = the same at every rank: `StackModel.max_rank`, `StackModel.duration_rank`,
  `Cond::Trigger.chance_rank` (readable `when = { …, chance_rank = "linear" }`), `CounterBonusParams.max_rank` and
  `.reset_after_rank` (requires `reset_after`). The fields keep their maximum-rank values. Affects T06 (L2 rank
  resolution), T19 (per-rank checks against `levelStats`), T20-T29.
- `CounterBonusParams.applies_to` is `Option<Box<Scope>>` (was `Option<Scope>`): the larger parameter struct made
  `Target` trip `clippy::large_enum_variant`. Serde, TypeScript and schema forms are unchanged.

### Encodings

- New field codec `serde_util::ext_f64` (and `ext_f64::option`): the readable form writes ±∞ as the strings
  `"Infinity"` / `"-Infinity"` (the spelling of pipeline.md §4.1) and reads numbers, those strings and TOML `inf`; NaN is
  rejected; the compact form is the plain `f64`. Used by `FireSpec.rate` (Lanka `math.huge`, 06 §4.1) and
  `MagazineSpec.ammo_max` (robotic `AmmoMax = ∞`, 06 §11.4). TypeScript `number | "Infinity" | "-Infinity"`; JSON
  schema `ExtF64`. Without it `data/generated/*.json` could not carry Lanka (serde_json writes `null` and fails to read
  it back). Affects T04 (`wfd-data/build.rs` reads it), T18 (emit), T12 (frontend types).
- Slot-number sub-indices `XhSlot`, `MechSlot`, `MultSlot` have a private field: `new(i) -> Option<Self>`, `get()`,
  `N`; deserialization checks the range (`try_from = "u8"`). Every `StatKey` value is therefore a registry slot
  (`index() < N_STATS`). Breaking only for code that built `MultSlot(3)` directly (none exists). Affects T06.

### Documents and TypeScript (domain-model.md §9, server-api.md §2, §4)

- Every non-`Option` document field that serde omits is optional in TypeScript (`ts(optional = nullable)`, which ts-rs
  12 renders as `?: T` for a non-`Option` type): `BuildDoc.weapon_item`, `.warframe`, `.companion_set_mods`,
  `.companion_kind`; `WeaponItemDoc.catalyst`, `.polarities`; `WarframeDoc.reactor`, `.polarities`, `.frame_complete`;
  `CalcInput.x`; `ScenarioDoc.profile`; `EnemySetup.context_overguard`; `BuffPick.params`, `.caster`. Maps keyed by
  `SlotId` are partial (`{ [key in SlotId]?: V }`, server-api `Partial<Record<SlotId, Polarity>>`). `Toggles` fields
  are `T | null` and required (serde writes `null` for an off toggle), except the proposed `capacity?`, `melee_plan?`
  and `zoom_level?: number | null`, as server-api.md §4 has them. `tests/ts_schema.rs` compares the generated
  `CalcInput`, `BuildDoc`, `ScenarioDoc`, `Toggles` and their inline parts with server-api.md §4 (optionality and
  nullability per field) and checks that the generated types accept the JSON serde writes. Affects T01b (gen-ts), T12,
  T30, T31.
- T-08 `Stealth.rank` is `Option<u8>`: `None` = the weapon's maximum rank (spec 00 §6.1 "default the maximum", 30 or
  40); domain-model.md §9.3 has a required `0..30` and server-api.md §4 `{ rank: number }` (request C-15). T-09
  `Heavy.start_combo` defaults to 12 (spec 00 §6.1); `Heavy::DEFAULT_START_COMBO`. A toggle switched on without its
  value (`{ "stealth": {} }`, `{ "heavy": {} }`) is accepted. Affects T06/T08b (resolve `None` against the weapon),
  T31 (UI).

### Message keys (ownership, work-breakdown.md §8)

- `MessageKey` and `MsgArg` moved from `message/mod.rs` (owned by T01b) to the T01a module `crate::i18n`, with the same
  JSON forms; both are re-exported at the crate root. `message/mod.rs` is an empty, documented module. T01b builds
  `Message`, `Severity` and the per-area key enums in `message/` on top of them and re-exports them there
  (`pub use crate::i18n::{MessageKey, MsgArg};`); a change to either type is an entry here. `MessageKey::new` and
  `as_str` added.

### Curated loader (pipeline.md §2, §6)

- New `load_curated_files(impl IntoIterator<Item = (PathBuf, String)>)`: loads a tree given as relative paths and
  contents (the in-app refresh's `InputTree::Embedded`, AD-13); `load_curated_dir(&Path)` walks the directory and calls
  it, so the file-to-type dispatch exists once. New: `UNLOADED_TOP_FILES` (`priority.toml`, T19, pipeline.md §6.3;
  `schema.json`, T01b, pipeline.md §6.1) are skipped like `overrides/` and `aliases/`; hidden files and directories
  (`.gitkeep`) are skipped; any other non-`.toml` file in a data directory (`x.toml.bak`, `README.md`) is an error;
  every file must have `schema = 1` (`CuratedFile` trait, `parse_curated`); a rule table given twice is an error.
  Affects T18 (embedded refresh), T19, T03b, T03c.

### Workspace and tests

- `.cargo/config.toml` with `[alias] xtask = "run --package xtask --"` (T01a, as owner of the workspace
  configuration; request C-16). `cargo xtask spec-sync --check` now reaches the xtask binary; the command itself is
  T01c's.
- `wfd-model` has a dev-dependency on itself with the features `curated`, `ts` and `schema`, so the plain
  `cargo test --workspace` of the definition of done runs `tests/domain_examples.rs` and `tests/ts_schema.rs`.
  overview.md §3.1 also says `wfd-datagen` and `xtask` depend on `model (curated)`; their manifests are skeletons of
  T02c and T01c (work-breakdown.md §8) and do not enable it yet: T02c and T01c add `features = ["curated"]`.
- Documentation: every public method, function and constant of `wfd-model` is documented, and the fields whose units
  or meaning are not obvious from the name (weapon, enemy, scaling, effect, mechanic, item, document fields). Still
  undocumented: about 450 enum variants that are spec ids or self-describing names (`HEA`, `OnKill`, `Body`), the
  mirror fields of the readable `…Repr` forms (documented on their model type) and rows of curated rule tables. The
  crate does not enable `missing_docs`, which under `-D warnings` would force boilerplate on those.

### Requests to the manager (continued)

| # | Conflict | Spec | Provisional choice |
|---|---|---|---|
| C-14 | `VulnMask.layers` default | 05 §13, 04 §4.2, §5.7: health only; domain-model.md §11.1: "default all" | `{HP}`; `types` default every type (04 §4.2) |
| C-15 | T-08 stealth rank | spec 00 §6.1: 0..maximum rank (30 or 40), default the maximum; domain-model.md §9.3 `0..30` required, server-api.md §4 `{ rank: number }` | `Option<u8>`, `None` = weapon maximum |
| C-16 | Ownership of `.cargo/config.toml` | not in work-breakdown.md §8; the DoD runs `cargo xtask …` | created by T01a as workspace configuration; please add it to §8 |


## 2026-11-24 · Manager · Rulings on T01a requests C-1 to C-16

The spec wins over the architecture documents where they disagree (CLAUDE.md, work-breakdown.md §2.4).

| Id | Ruling |
|---|---|
| C-1, C-2, C-3, C-5 … C-11, C-13 | Accepted as implemented (each follows the spec). |
| C-4 | Open. Settled by T01d (below): either every `max_stacks = none` effect of 04 is a `counter_bonus`, or `StackModel.max` gains an unbounded form (04 §5.5 Magus Accelerant "no limit"). |
| C-12 | Accepted provisionally (T18 emits one child attack per parent variant); T01d confirms against the final 04 §2.14. |
| C-14 | Accepted: `VulnMask.layers` defaults to `{HP}`, `types` to every type (04 §4.2 table row `layer_mask, type_mask`: "default `{HP}` and `all`"). domain-model.md §11.1 is updated by T01d. |
| C-15 | Accepted: `Stealth.rank: Option<u8>`, `None` = the weapon's maximum rank (00 §6.1 T-08). domain-model.md §9.3 and server-api.md §4 are updated by T01d. |
| C-16 | `.cargo/config.toml` is owned by T01c (work-breakdown.md §8 updated). |

**T01d contract sync (manager-added task, after the spec follow-up round).** The spec consistency pass renumbered and
added toggles in 00 §6.1 (T-30 melee plan, T-31 exalted ability rank, T-32 weak spots, T-33 Strength sources,
T-34 finisher mode, T-35 capacity mode, T-36 lower mod ranks, T-37 caster of T-13 … T-16, T-38 aiming and zoom,
T-39 Incarnon switch time, T-40 timed reload, T-41 pellet hit fraction, T-42 gauge uses per minute, T-43 external
statuses) and added flags (e.g. 02 `radial_parent_part`, 05 part flags). `Toggles`, `TOGGLE_MAP` and the rule
descriptors in wfd-model still use the earlier proposal numbering (T-30 capacity, T-32 melee plan). T01d brings
wfd-model, the generated files and the architecture documents in line with the final spec in one change.

## 2026-11-24 · T01c · xtask foundation (initial contract)

### xtask command list and wiring (work-breakdown.md §2.1, §2.2)

- `xtask/src/main.rs` registers every subcommand of overview.md §3 in `COMMANDS` (name, owner, module file, one-line
  description, `run: Option<RunFn>`), in this order: `gen-ts` (T01b), `gen-schema` (T01b), `spec-sync` (T01c),
  `spec-vectors` (T09), `lint-engine` (T01c), `ci` (T01c), `effect-smoke` (T19), `oracle-record` (T36, module
  `oracle.rs`), `oracle-audit` (T40), `bench-check` (T37, module `bench.rs`), `dist` (T39), `dist-smoke` (T39),
  `doc-check` (T01c). A row with `run: None` is a stub: it prints its owner and module and exits with status 2, so a
  stub never passes a check; `cargo xtask ci` skips stubs and names the owner.
- **Wiring a command** (the only edit of `main.rs` other tasks make): add `mod <module>;` and set the row's `run` to
  `Some(<module>::run)` (work-breakdown.md §2.2 lets only a file's owner edit it: request C-21). Entry point:
  `pub fn run(args: &[String]) -> anyhow::Result<()>`; `args` are the arguments
  after the command name; `Err` = failure (exit 1, error chain on stderr). Parse arguments with
  `crate::parse_args::<YourClapArgs>("<name>", args)` (clap derive; `--help` exits 0, a usage error exits 2).
  `crate::workspace_root()` is the workspace root. `cargo xtask ci` calls `run` in-process with the arguments of
  testing.md §10.
- `xtask/Cargo.toml`: `wfd-model` with `features = ["curated"]` (T01a handover; overview.md §3.1), plus the workspace
  dependencies `anyhow`, `clap`, `regex`, `serde_json` (already declared in `[workspace.dependencies]`; no new
  workspace dependency). T01b adds the `ts` and `schema` features for `gen-ts`/`gen-schema`. `Cargo.lock` gains the
  packages this pulls in.
- `.cargo/config.toml` (owner T01c, ruling C-16): unchanged alias `xtask = "run --package xtask --"`; header comment
  names the owner.

### `CompatFlags` (crates/wfd-model/src/compat_generated.rs, domain-model.md §9.5)

- Generated by `cargo xtask spec-sync` from the committed registers: **73 flags today** (bits 0-72; work-breakdown.md
  T01c and domain-model.md §9.5 say 71, the count when the plan was written). Bit order = register order (00 §5.2
  first, then the rows only 01 … 09 list), each id once; `INSPIRATION = u128::MAX >> (128 − N)` (every flag),
  declared last so `CompatFlags::names()` of a full set lists the individual flags. `cargo xtask spec-sync --list`
  prints the table.
- **Flag names** (what `Profile::Compat` carries in documents and what engine code names): the id without its hyphen,
  then up to four words of the row's topic (from the first register listing it; parenthesised parts, apostrophes,
  quotes, code marks and the words a, an, and, at, by, for, from, in, of, on, or, the, to, vs, with, without
  dropped), upper case, joined by `_`: `D001_QUANTIZATION_GRID`, `D007_PHOENIX_TALONS`, `D706_DREAMERS_WRATH_GATING`.
  domain-model.md §9.5 and the doc comments of `compat.rs`/`scenario.rs` (T01a) show the illustrative
  `D001_QUANT_1_16`, `D002_QUANT_SCOPE`, which are not generated names (T01d may update the examples). A flag is
  renamed when its topic's first four words change in the spec; code that names it then fails to compile, which is
  intended.
- Toggle-column rule (spec 00 §5.1 rule 3): the switch kinds are the first words of the `+`-separated parts of the
  cell before any parenthesis, `:`, `;` or `,` (`User`, `Compat`, `—`; anything else is a parse error). So
  `User (T-19) + Compat`, `Compat: …`, `Compat (proposed, OQ-739)` are flags; `— (Compat path of D-065, OQ-739)`
  (07 D-708) and `User (T-26); a compat flag is proposed (OQ-335)` are not. This differs from the literal wording
  "Toggle column contains `Compat`" of domain-model.md §9.5, AD-20, testing.md §2.5 and work-breakdown.md T01c, which
  would also flag D-708 (request C-20). When registers disagree, any register marking the row `Compat` makes a flag
  and spec-sync prints a warning (today: D-028, below). `xtask` tests pin the rows and Compat rows per register
  (306 rows, 197 ids, 73 flags); a spec change that moves them updates the test with the regenerated file.

### Checks (testing.md §2.5, §10)

- `spec-sync [--check] [--list]`: `--check` fails on a stale or missing generated file, naming every Compat row
  without a flag and every flag without a row; both modes fail when `TOGGLE_MAP` disagrees with spec 00 §6.1 (every
  T-nn of the spec exactly once; `Spec` entries must be spec rows; a `Proposed` entry must not use an id the spec
  lists). **Fails today** on the toggle drift that T01d fixes (14 problems: T-30, T-31, T-32 are `Proposed` entries
  for capacity mode, lower ranks and melee plan while the spec now lists melee plan, exalted ability rank and weak
  spots under those ids; T-33 … T-43 have no entry).
- `lint-engine`: scans `crates/{wfd-engine,wfd-opt}/src/**/*.rs` minus test code: items under an outer attribute
  that holds only in test builds (`#[test]`, `#[tokio::test(…)]`, `cfg(p)` with `p` false whenever `test` is false:
  `cfg(test)`, `cfg(all(test, …))`, but not `cfg(any(test, feature = "…"))`, which ships), files starting with
  `#![cfg(test)]`, and the files of test-only `mod x;` declarations (honouring `#[path]`). I-1 exempts
  `crates/wfd-engine/src/spec_constants.rs` only (a `wfd-opt` constant file would need an overview.md I-1 change).
  I-2: `/Lotus/` anywhere (code, strings, comments); a fixture item name as a run of words, case and separators
  ignored, inside one string literal or across an identifier path (`braton_prime`, `Braton::Prime`). Not item names
  (request C-22): names in comments; a whole identifier that is model vocabulary (a generated `CompatFlags` name,
  a stat-registry row or variant, a resolved-stat name: `CompatFlags::D007_PHOENIX_TALONS`, `StatKey::PhoenixTalons`,
  `mod multishot`); custom-stat records of the fixture (their names are the labels of spec 00 §6.3). Item names come
  from `data/fixtures/mini/**/*.json`: every object with a string `unique_name` (or `uniqueName`) and `name`, the
  name resolved through the string maps of `names/*.json`, `kind` `custom_stat` (`{"custom_stat": …}`) marking
  custom stats. T03 should keep that shape (domain-model.md §8, pipeline.md §8.1).
- `ci [--dry-run] [--keep-going]`: the 18 gates of testing.md §10 in order (a unit test keeps the list equal to the
  §10 code block); a gate runs when it exists (xtask stub → skip; `wfd-datagen --help` must list the subcommand, and
  a failing `--help` fails the gate; `frontend/package.json` and its scripts; `frontend/playwright.config.*`). The
  `wfd-datagen check` gate first runs `wfd-datagen fetch --from-lock` (the §10 note; a unit test ties the two),
  once `--help` lists `fetch`; it needs the network hosts of pipeline.md §3.1.
- `doc-check [--strict]`: status lines of docs/{architecture,data,mechanics} and research/findings; task ids,
  commands, packages, paths and types named in docs/architecture and docs/data. A type exists when a workspace
  source defines it (Rust items, enum variants, macro-defined types such as `sub_enum!(Ips …)`; comments and string
  literals do not count; TypeScript declarations once the frontend exists). "Planned": a path work-breakdown.md
  names (spans, `--test`/`--bench` targets of its commands; a relative span resolves in its package only), or a path
  under a directory it assigns that a module layout tree of the documents lists (engine.md §2, frontend.md §2,
  pipeline.md §6.1, overview.md §3), a pattern, or a conventional Cargo entry; any other path under an assigned
  directory is a **warning** (printed, not a failure; 6 today, each a file no task names). Types: named by the plan,
  defined by a design code block or a prose `Name { … }`/`Name(…)` shape; subcommands: a CLI synopsis line
  (`wfd-datagen …`, `warframe-dps …`). `--strict` (T41) accepts only what exists. External names (std, crates, web
  platform, upstream data keys; `Origin`, `ExportModSet`, `Hitscan` added) are listed in `doc_check.rs::EXTERNAL`;
  a document that quotes a new external name needs a row there (a T01c-file change through this log).

### Requests to the manager (work-breakdown.md §2.4)

| # | Finding | Provisional choice |
|---|---|---|
| C-17 | Spec register disagreement D-028 (Heat inherit): 00 §5.2 Toggle `User (T-26) + Compat (…)`, 03 §17 `User (T-26); a compat flag is proposed (OQ-335)` | flag `D028_HEAT_INHERIT` generated (00 marks it); spec-sync warns until the registers agree |
| C-18 | `doc-check` finds four names that exist nowhere and no task plans: `crates/wfd-engine/benches/resolve.rs` (engine.md §13 l.636, testing.md §9; T15c/T16 own the other benches), `crates/wfd-engine/tests/measured.rs` and `research/test-cases/ingame/*.toml` (testing.md §1 l.30, §5, §11; no owner in work-breakdown.md §8, T40 "in-game cases" names no path), type `DocIssue` (pipeline.md §9 l.468, l.474: `DocIssue::ItemUnknown`, `ItemRetired`; not in domain-model.md or any task) | none: `cargo xtask doc-check` fails until work-breakdown.md assigns owners (or the documents drop the names) |
| C-19 | TOGGLE_MAP drift (T01d, known): `cargo xtask spec-sync --check` fails with the 14 toggle problems above, so T01c's acceptance line, the DoD line of work-breakdown.md §2.3 ("once T01c is accepted") and gate 3 of `cargo xtask ci` fail for every task until T01d changes `TOGGLE_MAP` (wfd-model, not a T01c file) | the check stays failing honestly; please either waive the spec-sync DoD line for waves 0-1 or sequence T01d before T01c's acceptance |
| C-20 | The flag rule: domain-model.md §9.5, AD-20, testing.md §2.5 and work-breakdown.md T01c say "one flag per row whose Toggle column contains `Compat`" and "71 rows today". 07 D-708's cell `— (Compat path of D-065, OQ-739)` contains the word but its switch kind is `—` (00 §5.1 rule 3; engine.md §10 treats `—` rows as no switch), and the registers now hold 73 Compat rows | flag rows whose switch kind is `Compat` (73 flags, D-708 none); please reword the rule as "switch kind `Compat` per spec 00 §5.1 rule 3" and update 71 → 73 in those four places |
| C-21 | Ownership of the xtask wiring: work-breakdown.md §2.2 says only a file's owner edits it, but each command owner (T01b, T09, T19, T36, T37, T39, T40) must add `mod x;` and `run: Some(x::run)` to `xtask/src/main.rs` (T01c). `xtask/Cargo.toml` has no owner in §8 (T01a created it, T01c added dependencies, T01b adds the `ts`/`schema` features) | please allow that two-line wiring edit in §2.2 and assign `xtask/Cargo.toml` to T01c in §8 (other tasks change it through a CONTRACTS.md entry). A build script that discovers the modules was rejected: `cargo fmt` does not follow modules declared through `include!`, so their formatting would go unchecked |
| C-22 | I-2 scope: overview.md §6.1 says lint-engine "greps for `/Lotus/` and for every item name in the mini fixture", testing.md §2.5 "no item name of the mini fixture in their sources". A literal grep fails code the architecture requires: domain-model.md §9.5 `if ctx.compat.contains(CompatFlags::D007_PHOENIX_TALONS)` (also D-016, D-060, D-401, D-404, D-706 name items), the registry stats `phoenix_talons`, `heat_inherit` (domain-model.md §10), engine.md §2 `resolve/multishot.rs` and the frozen `fn multishot`/`fn status_chance` of T02a once the §17 custom-stat items carry their 00 §6.3 labels ("Multishot", "Status Chance"), and every spec citation in a comment | `/Lotus/` is searched everywhere, comments included; item names are not searched in comments, a whole identifier that is model vocabulary does not count, and custom-stat records are not items; please confirm and amend I-2 and testing.md §2.5 |

---

## 2026-11-24 · T01b · API DTOs, message keys, TypeScript and JSON-schema generation (initial contract)

T01b creates `crates/wfd-model/src/{api,message}/**`, `xtask/src/{gen_ts,gen_schema}.rs`, the generated
`frontend/src/api/generated/**` and `data/curated/schema.json`. The items below are where the code differs from or
adds to `docs/architecture/server-api.md`, `engine.md` §7.7-§9 and `optimizer.md` §1, §9. A test
(`api::tests::shapes`) compares the generated TypeScript with every type of server-api.md §5-§9, `Message` and the
route-table shapes (optionality and nullability per field, inline objects recursively, union arms by tag); its
`KNOWN_DIFFERENCES` list holds exactly the spec-driven differences below (C-26, C-27; renumbered from T01b's C-17,
C-18 by the manager's rulings of 2026-11-27).

### Modules and conventions

- `wfd_model::api::{catalog, schema, calc, buildcode, opt, data, error}` as work-breakdown.md T01b, plus `common`
  (shared shapes), `int53` (u64 codec), `codegen` (generators, features `ts` / `schema`+`curated`); everything is
  re-exported at `wfd_model::api::*`. `API_VERSION = 1`.
- API types are JSON only: optional fields are omitted when absent (`field?: T`), `T | null` fields are always
  written; requests and responses reject unknown fields. Every `u64` uses `#[serde(with = "api::int53")]` (values
  above 2^53 − 1 are rejected when read and when written) and `#[ts(type = "number")]`.
- `PerDamageType` (one `f64` field per damage type, `IMP` … `FIN`) is server-api's total `Record<DamageType, number>`
  (`AttackDetail.base`, `EnemyDetail.faction_mods`); partial records are `BTreeMap<DamageType, f64>`. `LayerValues`
  is `{ OG, SH, HP }`.
- Values that can be infinite use `serde_util::ext_f64` (`"Infinity"`): `FireView.rate`, `StatBlock.fire_rate` (Lanka),
  `MagazineView.ammo_max` (robotic weapons), and all three float fields of `Score` (bounds and thresholds carry ±∞ in
  the tie fields, engine.md §7.9). (Extended by the review fixes below: every score-valued field and `BaseModded`.)

### Shapes that server-api.md leaves open or that differ from it

- `ErrorResponse { error: Message, issues?: Message[] }`: 422 `api.invalid` puts the document issues in `issues`;
  500 `api.internal` carries `error_id` and 409 conflicts carry `job` in `error.args`.
- Catalogue and item detail `kind` is `ItemKindName` (the kind slug of `ItemKind::slug`): the bundle's `ItemKind`
  carries dense ids (`StanceId`, `FamilyId`), which never leave the server (server-api.md §2).
- `Candidate` is `{ key, legal, reason? }` instead of server-api's union discriminated by a boolean (`legal: true |
  false`), which serde and ts-rs cannot express; the JSON is the same. (Superseded by the review fixes below: the
  TypeScript type is server-api's union.)
- `PartSummary` (named without fields in §5.3) is `{ key, name, image, display? }`.
- `CalcReport.capacity.mode` is `CapacityMode` (server-api: `string`); `ConditionView.override` is `CondOverride`
  (server-api: `unknown`); `Meta.app_id` is the one-value enum `AppId` (`"warframe-dps"`).
- `ScenarioSchema.slot_layouts` is a map (TypeScript `{ [key in WeaponSlot]?: … }`; the server fills every slot).
- `ParityOut.ttk` is `Option<Option<f64>>`: absent = not computed, `null` = not killed (server-api `ttk?: number |
  null`).
- `TimelineRequest.attack` is the untagged `TimelineTarget = AttackKey | GroupRef { group }`.
- `OptEvent` and `DataEvent` are internally tagged by `event` (`{ "event": "progress", … }`, the NDJSON form and
  `JobStatus.last_progress`); on SSE the `event:` field carries the tag and `data:` the other fields:
  `OptEvent::to_sse` / `from_sse`, `DataEvent::to_sse` / `from_sse`, `name()`, `is_terminal()`.
- optimizer.md names without fields, defined here: `Plan { positions: PlanPosition { group: OptGroup, pool }[],
  composition_sets, excluded }`, `ExcludedItem { key, reason: Message }`, `SearchStats { evaluated, bounded, pruned,
  leaves, phases: PhaseTime[], threads, reproducible, elapsed_ms }`, `BuffTable { rows: BuffScenarioRow { buffs,
  best, buff_value, rebuild_gain }[] }`, `ProofScope::MainGivenContext`. `leave_one_out` and `next_best` are both
  `SlotValue { slot, content: SlotContent, score }` (optimizer.md's `SlotAlternative` is the same shape).
  `RuleFlags` is a bitflags set (JSON: list of names). `BuffPolicy.allowed` is `BuffPick[]` (optimizer.md; adds the
  optional `caster` to server-api's shape). `OptResult.condition_sensitivity` and `buff_table` are `T | null` as in
  the §8.2 example. `SearchParams::default()` holds the defaults of optimizer.md §11. (The meaning of an absent and a
  `null` `time_limit_s` / `mc_confirm` is fixed by the review fixes below.)
- `JobStatus<R>.last_progress` is untyped JSON (`unknown`), as in server-api.md.
- `EvOut { views: (View, ViewEv)[] }` and `McOut { views: (View, ViewMc)[], runs, seed }` (engine.md §7.7, §8.4;
  re-exported by `wfd-engine`, T02a): `ViewEv` and `ViewMc` have the server-api shape, which has no `view` field, so
  the view travels in the pair.
- engine.md §7.7-§8.4 field types become the server-api JSON shapes in the DTOs: `per_type` is a map of the nonzero
  types, `per_source` / `breakdown` are `SourceDps { source, dps }` lists (source ids documented on `SourceDps`),
  `ttk.seconds` and TTK medians are `null` instead of +∞. `Explanation.formula` is a `Message` (server-api.md §6.3;
  engine.md §11.3 says `formula_key: MessageKey`).
- `ImportReport` is the shape of `DecodeResponse.report`; `DecodeReport` is an alias (T02b's `decode` signature).
- Toggle numbers in comments follow spec 00 §6.1 (capacity mode T-35, lower ranks T-36, melee plan T-30) and name
  server-api's proposed numbers; no field depends on the numbering (T01d renumbers `Toggles` and `TOGGLE_MAP`).

### Message keys (frontend.md §8, overview.md I-8)

- `Message { key: MessageKey, args, severity?: Severity }` in `message/mod.rs`; `MessageKey` and `MsgArg` are
  re-exported from `crate::i18n` (T01a) with `From` impls for numbers and strings. Constructors `Message::new(key)`,
  `.arg(name, value)`, `.with_severity(s)`.
- Per-area key enums `ApiKey`, `DocKey`, `EngineKey`, `OptKey`, `DataKey`, `ImportKey`, `LegalityKey`
  (`message/{api,doc,engine,opt,data,import,legality}.rs`), each with `ALL`, `AREA`, `as_str()`, `args()` (declared
  argument names), `parse()`, `message()`, `Into<MessageKey>`; `KnownKey` is their union and `message::all_keys()`
  the exported list. `LegalityKey` holds the 34 rule keys of spec 09 (§1.5 and the E-9 rules). Keys are unique across
  areas. Keys of curated data (toggle, parameter and riven-trait labels) are not listed; they come with the schema
  route. Later tasks add keys to their area file with an entry here.

### Generated files and xtask

- `cargo xtask gen-ts [--check]` writes one file per type reachable from the route types (ts-rs 12, default config,
  so an unannotated `u64` would appear as `bigint` and fail the tests), plus `messageKeys.ts` (`MESSAGE_KEYS`: key →
  area and argument names; `type KnownMessageKey`) and `index.ts` (re-exports every file). The directory holds
  generated output only: `gen-ts` removes files the model no longer produces and `--check` reports missing, stale and
  extra files. `codegen::typescript_files()` fails when two Rust types would export to one file.
- `cargo xtask gen-schema [--check]` writes `data/curated/schema.json`: root `CuratedItemFile` (domain-model.md §15.1
  header and the `EffectDef`s, pipeline.md §6.1), every other curated file type (`IncarnonFamilyFile`, the rule and
  melee tables) under `$defs` (`codegen::CURATED_FILE_SCHEMAS` maps file patterns to definitions; the root's
  `$comment` repeats it).
- Entry points `gen_ts::run` and `gen_schema::run` have T01c's signature `pub fn run(args: &[String]) ->
  anyhow::Result<()>`. T01b's branch carries an interim `xtask/src/main.rs` (T01c's file) that dispatches only these
  two commands; on merge T01c's `main.rs` wins and wiring is `mod gen_schema; mod gen_ts;` plus `run:
  Some(gen_ts::run)` and `run: Some(gen_schema::run)` in its command table. `xtask/Cargo.toml`: `wfd-model` with
  features `ts`, `schema`, `curated`, and `anyhow` (both already workspace dependencies; T01c's manifest adds
  `curated`, `anyhow`, `clap`, `regex`, `serde_json`, so the merge is the union).

### Requests to the manager (spec conflicts, work-breakdown.md §2.4)

Provisional choice for each: the spec's, as implemented. Affected: T02a, T12, T15a-T17, T32-T35.

| # | Conflict | Spec | Provisional choice |
|---|---|---|---|
| C-26 | `ViewEv.per_shot`, `.per_magazine` | 08 §8.5: none for N_mag = ∞, not available for gauge-gated attacks; server-api §6.1: required numbers; engine.md §7.7: `per_magazine: Option` | both optional |
| C-27 | `Summary.sd`, `.ci95` | 08 E-8.19: R = 1 gives no sd (and no SE, so no CI); server-api §6.1: required | `number \| null`; `ci99?` added (E-8.19, used by E-8.24; engine.md §8.4 lists it) |
| C-28 | Optimizer metric `ttk_noarmor` | 08 §10 defines it; server-api §8.1 and optimizer.md §1 omit it | `Metric::TtkNoarmor` |
| C-29 | MC crit histogram | 08 §8.4: "normalised" histogram; engine.md §8.4: `[u64; N_TIER_BINS]` counts | `[f64; 8]` fractions |
| C-30 | Outputs the spec lists and the DTOs lacked | 08 §8.2/§8.6 TTK breakdown, §8.10 EFR and caps in the panels, §8.9 Debilitate's added procs, §10 EV rank and gap classes of re-ranked finalists | optional additions: `TtkEv.breakdown`, `TtkDist.breakdown`, `StatBlock.efr`, `EnemyPanel.caps`, `TimelineHit.statuses_added`, `RankedBuild.gap_classes`, `.ev_rank`; `OptResult.capacity` (optimizer.md §7 step 4, 09 E-9.16) |

## 2026-11-24 · T01b · Review fixes (before acceptance)

Changes made after the independent review of T01b. Where an item changes a shape described in the T01b entry above,
this entry wins. TypeScript regenerated (`cargo xtask gen-ts`); `data/curated/schema.json` is unchanged.

### Encodings: values that can be infinite

Serde_json writes ±∞ and NaN as `null`, which then fails to read back, so every `f64` that can be non-finite under the
architecture's own definitions now has an explicit encoding (`serde_util::ext_f64`, `"Infinity"` / `"-Infinity"`,
TypeScript `number | "Infinity" | "-Infinity"`; NaN is a serialization error, never a silent `null`):

- Scores. The TTK metrics' unit is T_EV seconds, +∞ for a build that never kills (08 E-8.21, E-8.26 band 2;
  engine.md §7.7): `RankedBuild.score`, `SlotValue.score`, `OptResult.condition_sensitivity` (`T | null` as before,
  now `number | "Infinity" | "-Infinity" | null`), `Proof::Gap.relative` and `OptEvent::Progress.gap` (a bound in a
  better band than a never-killing incumbent).
- `BuffScenarioRow.buff_value` and `.rebuild_gain` are differences of scores: `Option<f64>` (`ext_f64::option`,
  always written), ±∞ when one side never kills, `null` when both do (∞ − ∞). `rebuild_gain` is defined as the
  scenario's best score minus the no-buff best build's score with the buffs on.
- `BaseModded.base` and `.modded`: the weapon panel's `extra["ammo_max"]` row is ∞ on robotic weapons (06 §11.4),
  as `MagazineView.ammo_max` already was. Affects `WeaponPanel.magazine`, `.reload` and `.extra` (TypeScript only
  widens).
- `TtkEv.kappa` is `Option<f64>`, present exactly in band `killed`: κ is defined at T_EV on the expected path within
  the horizon (E-8.21) and E-8.26 uses it only in band 0; the extrapolated and never bands have no κ (request C-24).

Affects T02a (re-exports), T15a/T15b (EV TTK, weapon panel), T16/T17 (report), T32-T35 (scores, buff table, gap),
T12, T31, T35 (frontend types).

### Report shapes (spec 08 §8, 09 E-9.19)

- `ViewEv` gains the EV window curves of 08 §8.3 (`uptime[s][k]`, `stacks_mean[s][k]`, listed as outputs in 08 §3):
  `uptime: StatusSeries[]` (optional, omitted when empty), bins as `series`, the counterpart of `ViewMc.uptime`.
  `StatusSeries` documents both evaluators. (C-30 addition.)
- 08 §8.4 defines two EV crit-tier distributions. `ViewEv.crit_tiers` is the EV **window** distribution (the
  counterpart of `ViewMc.crit_hist`, which already said "bins as `ViewEv.crit_tiers`"); the steady distribution is the
  new optional `crit_tiers_steady` (absent for gauge-gated attacks, which have no steady state). engine.md §7.7 does
  not say which one `crit_tiers` is. (C-30 addition.)
- `ViewEv.dps_burst`, `.dps_sustained` and `.hit_infight` are optional: 08 E-8.14 reports them, like `dmg_per_shot`
  and `dmg_per_mag` (C-26), as "not available" for gauge-gated attacks (request C-23; found while checking §8 for
  this fix, not raised by the review).
- `CapacityReport` (09 E-9.14-E-9.19): `forma` and `umbra_forma` are `number | null`, `null` when no layout reaches
  U ≤ C_cap (E-9.19 "No layout ⇒ cap.exceeded"); `used` is the U of the minimum-Forma layout (ties by the smaller U),
  or the smallest U any layout reaches when none fits, so `used − capacity` is the excess. New optional
  `forma_applied_min` (rank > 30: the lower bound ceil((rank − 30)/2) on Forma applied, E-9.16 and E-9.19 "Report")
  and `warframe: ItemCapacity { used, capacity, forma, umbra_forma }` (E-9.17 warframe form; present when the build has
  a warframe with `frame_complete`, the only case E-9.18 checks it). `CalcReport.capacity` is filled in every
  capacity mode, `ignore` included (E-9.19 is a report in every mode, a constraint only in `any_forma` and `budget`);
  absent only without a weapon. `OptResult.capacity` stays absent under `ignore` (optimizer.md §7 step 4 says so)
  (request C-25). Affects T09 (`min_forma` → this report), T15c/T16 (calc report), T32/T33, T30/T31 (capacity bar).

### `SearchParams`: absent versus `null`

`time_limit_s` and `mc_confirm` now separate "use the default" from "off": an absent field takes the optimizer.md §11
default (120 s; `McConfirm::DEFAULT` = 12 finalists, 1,000 runs, seed 0), `null` means no wall-clock limit / no MC
confirmation (`None`), and both are always written, so a round trip keeps the meaning. TypeScript `time_limit_s?:
number | null`, `mc_confirm?: McConfirm | null` (server-api: no `null`). `SearchParams::default()` equals the parse of
a request that omits both. New `SearchParams::DEFAULT_TIME_LIMIT_S`. Affects T33 (JobControl deadline, MC step), T35.

### `Candidate` (server-api.md §5.7)

The fields are private; `Candidate::legal(key)` and `Candidate::blocked(key, reason)` build one, `key()`,
`is_legal()`, `reason()` read it, and deserialization rejects `{ legal: false }` without a reason and `{ legal: true,
reason }` (`try_from` a raw struct). The TypeScript type is server-api's union `{ key, legal: true } | { key, legal:
false, reason }` (ts-rs `as` a TypeScript-only enum). The JSON is unchanged. Affects T07/T09 (candidates route), T30.

### Message keys

- `EngineKey::SteadyNoConvergence` = `ev.steady.no_convergence` `[attack, view]` (08 E-8.13: the steady fixed point
  stopped at n = 200).
- `DataKey::LockInputGone` = `data.lock_input_gone` `[source, file, locked]` (pipeline.md §3.3, work-breakdown.md
  T05: `fetch --from-lock` cannot get a locked input; `locked` is its revid, content hash or tag).

### Documentation and tests

- Every public item of `api` and `message` is documented, enforced by `#![warn(missing_docs)]` in both modules
  (an error under the definition of done's `clippy -D warnings`). Variants cite their source (00 §6.2 condition
  classes, 06 E-6.13 shot flags, 07 §8.1 plans, pipeline.md §2 steps, optimizer.md §9 phases); `SearchMode`'s
  `balanced` and `quick` are named but not defined by the architecture, so their docs only name them.
- `api::tests::shapes` compares types as well as optionality: scalar members after resolving names on both sides
  (base types, literal unions, template literals such as `SlotId`, arrays, tuples, maps, indexed access, total
  records against `PerDamageType`). The generated type may narrow a document `string` to the model's enum of slugs;
  every other difference is listed in `TYPE_DIFFERENCES` with its reason (`ItemKindName`, `api_version: number`,
  `ttk_noarmor`, riven `traits: number`, the ext_f64 fields, the 8-bin tuples, `CapacityMode`, `CondOverride`), and an
  entry that is no longer needed fails the test. `KNOWN_DIFFERENCES` gains the optionality changes above.
- `PerDamageType` maps fields to `DamageType` by name (`get` is an exhaustive match; `from_vec` / `to_vec` index by
  `DamageType::index`), so a reorder of either list cannot mis-map; the test checks every type's value and JSON key.

### xtask (merge with T01c)

`xtask/src/main.rs` and `xtask/Cargo.toml` are T01c's files (work-breakdown.md §8). T01b's branch still carries the
interim dispatcher, because `cargo xtask gen-ts --check` (this task's acceptance) cannot run on T01a's `fn main() {}`
and T01c is not merged into the base branch, so T01b cannot rebase onto it without carrying T01c's unaccepted commit.
Resolution when both are merged (whichever lands second): `xtask/src/main.rs` = T01c's file plus `mod gen_schema;
mod gen_ts;` and `run: Some(gen_ts::run)` / `run: Some(gen_schema::run)` in the `gen-ts` and `gen-schema` rows (the
edit T01c's header allows other tasks); `xtask/Cargo.toml` = T01c's manifest with `wfd-model` features `["curated",
"ts", "schema"]`; `Cargo.lock` regenerated by `cargo build`; `CONTRACTS.md` keeps both tasks' entries. Until then the
CI gate `cargo xtask gen-ts --check` exits 2 on a tree that has T01c's `main.rs` without the wiring.

### Requests to the manager (spec conflicts, work-breakdown.md §2.4)

Numbering: T01b's initial entry and T01c's entry (parallel branches) both used C-17 to C-21 for different requests,
and T01c also C-22; the manager renumbered T01b's initial requests to C-26 … C-30 (rulings of 2026-11-27, applied by
T01d). "C-30 (addition)" below is T01b's former C-21; the new requests start at C-23 to avoid a further collision.
Provisional choice for each: the spec's, as implemented. Affected: T02a, T09, T12, T15a-T17, T30-T35.

| # | Conflict | Spec | Provisional choice |
|---|---|---|---|
| C-30 (addition) | EV window curves and the two EV crit-tier distributions | 08 §3 output ids `uptime[s][k]`, `stacks_mean[s][k]`; 08 §8.3, §8.4 | `ViewEv.uptime?`; `crit_tiers` = window, `crit_tiers_steady?` |
| C-23 | `ViewEv.dps_burst`, `.dps_sustained`, `.hit_infight` | 08 E-8.14: "not available" for gauge-gated attacks; server-api §6.1 and engine.md §7.7: required numbers | optional |
| C-24 | `TtkEv.kappa` | 08 E-8.21: κ at T_EV; E-8.26 uses it in band 0 only; server-api §6.1 `kappa: number`, engine.md §7.7 `kappa: f64` | optional, present exactly in band `killed` |
| C-25 | Capacity report | 09 E-9.19: no layout possible, overlevel Forma bound, report in every mode; E-9.17/E-9.18 warframe capacity; server-api §6.1 `forma: number`, no warframe; optimizer.md §7 step 4 omits it under `ignore` | `forma: number \| null`, `forma_applied_min?`, `warframe?`; calc report in every mode, optimizer report as optimizer.md |

## 2026-11-24 · T03 · Mini fixture bundle (no interface change)

- No model type changed. T03 adds the test file `crates/wfd-model/tests/fixtures.rs`, which the T03 acceptance
  command (`cargo test -p wfd-model --test fixtures`) names; it lives in the T01a-owned crate directory but touches no
  existing file (auto-discovered test target, no manifest change). Affects nobody.
- The per-file shapes of the bundle JSON layout (pipeline.md §8.1 names the files, not their types) are written down
  in `data/fixtures/README.md` as the fixture uses them: each file is the readable serde form of its `wfd-model` type
  (`Vec<Weapon>`, `Vec<Item>`, `Rules`, …), `names/en.json` is the `"en"` map of `NameTable`, `stable_keys.json` is
  `[[key, uniqueName], …]`, enemies are split by wiki sub-module, dense ids follow the concatenation order of the
  files, and `extra/<task>.json` is an object of optional record lists appended after the base files. T04
  (`wfd-data/build.rs` reads `WFD_DATA_DIR`) and T18 (emit) should read and write this shape, or record a different
  one here and update the fixture loader in `tests/fixtures.rs` with it.
- Provisional data choices made in the fixture, for the owners of the real tables: `Secondary Fortifier` is
  `mult = "fortifier"` with value 7.0 (factor 1 + Σ = ×8; ranks between 0 and 5 interpolated, UNVERIFIED);
  custom stat `vuln_status_damage` targets `dot_vuln` (04 §10 puts it in the V_sd group of 03 §9.4, which has no
  registry slot; UNVERIFIED); custom stat `abilityCombo` has no effect (it sets T-11, status `unmodelled`); the
  inspiration keys of custom stats are the 04 §10 key names. Affects T19, T20-T29.

## 2026-11-27 · Manager · Rulings on the T01b and T01c requests

Renumbering: T01b's initial requests C-17 … C-21 become **C-26 … C-30** (C-17 → C-26, C-18 → C-27, C-19 → C-28,
C-20 → C-29, C-21 → C-30; "C-21 (T01b, addition)" becomes "C-30 (addition)"). T01c keeps C-17 … C-22; T01b's
review-fix requests keep C-23 … C-25. T01d applies the renumbering to the T01b sections above and to every reference
in code and documents. The next free number is C-31.

| Id | Ruling |
|---|---|
| C-17 (T01c, D-028) | Settled by the spec rounds: 00 §5.2 and 03 §17 both read "User (T-26) + Compat" now. The flag stays. |
| C-18 (T01c, doc-check names) | Owners assigned in work-breakdown.md §8: `crates/wfd-engine/benches/resolve.rs` → T37; `crates/wfd-engine/tests/measured.rs` and `research/test-cases/ingame/*.toml` → T40. `DocIssue` (pipeline.md §9) belongs to build-document decoding: T10 owns it in `wfd-buildcode`; if a DTO must carry it, T01d puts the type in `wfd-model` and documents it in domain-model.md. |
| C-19 (T01c, spec-sync DoD waiver) | Waived until T01d is merged; it lapses with T01d. From then on `cargo xtask spec-sync --check` is part of every task's Definition of done. |
| C-20 (T01c, flag rule) | Accepted: one flag per register row whose **switch kind** is `Compat` (00 §5.1 rule 3); D-708 has none. T01d rewrites the rule in overview.md AD-20, domain-model.md §9.5, testing.md §2.5 and work-breakdown.md T01c and replaces the pinned count ("71 rows today") with the count the registers hold when T01d lands. |
| C-21 (T01c, xtask wiring) | Accepted: each command's owning task may add its `mod x;` line and its `run: Some(x::run)` entry to `xtask/src/main.rs`; nothing else in that file. `xtask/Cargo.toml` is owned by T01c; other tasks change it through a CONTRACTS.md entry. work-breakdown.md §2.2 and §8 updated. |
| C-22 (T01c, I-2 scope) | Accepted as implemented: `/Lotus/` is searched everywhere including comments; item names are not searched in comments; a whole identifier that is model vocabulary does not count; custom-stat records are not items. T01d amends overview.md I-2 and testing.md §2.5 to say so. |
| C-26 … C-30, C-23 … C-25 (T01b) | Accepted as implemented: each follows spec 08 (§8.2-§8.10, E-8.14, E-8.19, E-8.21, §10) or 09 E-9.16-E-9.19. T01d updates server-api.md §6, §8.1, engine.md §7.7 / §8.4 references where they name the old required shapes only if T01d owns them (server-api.md); engine.md differences are recorded for T02a, which owns the engine contracts. |
| T03 fixture test file | `crates/wfd-model/tests/fixtures.rs` is owned by T03 (work-breakdown.md §8 updated; T03 review issue 15). |

## 2026-11-27 · T01d · Contract sync

T01d brings `wfd-model`, the generated files, `domain-model.md` and `server-api.md` in line with the converged spec
(00 §6.1 toggles T-01 … T-43, the T-23 options adopted from 05 OQ-523, the hand-authored attack data of 02 §4.10 and
03 §9.7, the 04 §4.2 effect fields, the 05 §4.1 / §4.3 record fields, 06 DUPLEX, the 07 wave attack ids), applies the
manager's rulings of 2026-11-27 and rules provisionally on C-4 and C-12 (below, for the manager to confirm). Items
marked **breaking** change a Rust type (no code outside `wfd-model` uses it yet; the JSON stays readable) and need the
manager's confirmation (work-breakdown.md §2.1). Every other item is a new optional field, a new enum variant or a
new type. Generated files regenerated with their commands: `compat_generated.rs` (`cargo xtask spec-sync`),
`frontend/src/api/generated/**` (`gen-ts`), `data/curated/schema.json` (`gen-schema`).

### Toggles (scenario.rs; domain-model.md §9.3; server-api.md §4)

- `TOGGLE_MAP` lists T-01 … T-43, every entry `Spec`: T-30 `toggles.melee_plan|toggles.melee_plans`, T-31
  `toggles.exalted_rank`, T-32 `buffs.params.spot_location|buffs.params.spot_nodes` (params of the Sonar and Detect
  Vulnerability buff items, 02 E-2.10; no type change), T-33 `toggles.strength_mode|toggles.ability_duration|
  buffs.caster`, T-34 `toggles.finisher`, T-35 `toggles.capacity|toggles.forma_types` (warframe part:
  `BuildDoc.warframe.frame_complete`), T-36 `OptRequest constraints.capacity.ranks` (unchanged field), T-37
  `buffs.caster` (of the T-13 buff), T-38 `toggles.aiming|toggles.zoom_level`, T-39 `toggles.incarnon_switch`, T-40
  `toggles.timed_reload`, T-41 `toggles.pellet_hit`, T-42 `toggles.gauge_uses_per_min`, T-43
  `toggles.external_statuses`. A path through a list names the list (`TogglePlace` doc). New test
  `documents.rs::toggle_map_paths_exist`: every scenario path exists in the JSON a scenario writes, and the ids are
  T-01 … T-43 in order.
- New `Toggles` fields (`#[serde(default)]`, always written, optional in TypeScript like the T01a proposed ones):
  `forma_types: FormaTypes` (flags `forma umbra omni stance aura`, default all; 09 §6.1), `melee_plans:
  Vec<AttackPlan { attack: AttackKey, plan: MeleePlanSel }>` (00 §6.1 "plan per selected melee attack"), `aiming: bool`
  (true), `exalted_rank: u8` (3, `Toggles::DEFAULT_EXALTED_RANK`), `strength_mode: StrengthMode` (`manual` | `derive`),
  `ability_duration: f64` (1.0, self-cast Dur_e, 04 E-4.10), `finisher: Option<FinisherMode>` (`stealth` | `back` |
  `front` | `universal`, None = off), `incarnon_switch: bool`, `timed_reload: bool`, `pellet_hit:
  Vec<AttackFraction { attack, fraction }>` (absent attack = 1), `gauge_uses_per_min: f64` (0), `external_statuses:
  BTreeMap<StatusType, u8>` (JSON `{ "VIR": 10 }`). `Toggles::external_stacks(s)` folds T-06 in.
- T-03 `part(<name>)`: `HitLocation::Part(NameKey)` (JSON `{ "part": "<BodyPart.name>" }`); `HitLocation` is no longer
  `Copy`. `BodyPartView.key` gives the UI the value.
- **breaking** T-22: `combine_groups: Option<SmallVec<[u8; 2]>>`, `None` (JSON `null`, the new default) = the profile
  default (every family one unit in the game profile, none in the compat profile); `Some(list)` = exactly those
  families. Reason: 00 §6.1 T-22 now reads "default on in the game profile, off in the compat profile" (CD-12, 08 §2),
  which an empty list could not express. An explicit `[]` keeps its old meaning (no family combined). Fixture edit
  forced by it: `scenarios/braton_prime_lancer_l100_sp.json` and its `real/` twin write `null` instead of `[]`
  (their test asserts `Toggles::default()`).
- **breaking** T-37: `BuffPick.caster: Option<Caster>`, `None` = the buff's new `ItemKind::Buff.default_caster:
  CasterKind` (`self` default, `ally` for the T-13 Roar buff, 00 §6.1 T-37, 04 §4.4). Reason: the T-37 default differs
  per buff, so a fixed serde default on `Caster` cannot express it. JSON shape and TypeScript unchanged (`caster?`),
  but the meaning of an omitted `caster` changed (review fixes below, item 9). `Caster::Ally` gains `duration: f64`
  (ally Ability Duration Dur_e, default 1, always written since the review fixes; T-33, 04 E-4.10);
  `Caster::ally()`, `CasterKind::caster()`. `ItemKind::Buff` also gains `duration_scaled: bool` (04 §4.2 `dur_scaled`).
  Fixture edit forced by T-37: the Roar item of `mini/items/buff.json` has `default_caster = "ally"`.
- Provisional architecture choices (recorded in domain-model.md §9.3; the manager may change them): a
  `melee_plans` entry wins for its attack, else `melee_plan` applies to the attacks its plan kind names (07 §8.1), else
  `Auto`; a T-43 `VIR` entry wins over T-06; `SquadAuras` count squadmates' auras only (the build's own aura slots
  count through the build).
- Comments and docs say T-35 / T-36 / T-30 where they said "proposed T-30 / T-31 / T-32" (`scenario.rs`,
  `api/opt.rs`, `api/schema.rs`, `rules.rs`, `curated.rs`).

### T-23 enemy options (`EnemySetup`, `EnemyInstance`; 05 OQ-523 as adopted by 00 §6.1)

New optional fields, omitted at their default: `eximus_kind: EximusKind` (`generic` | `entropic` | `guardian`),
`armor_hydroid` (E-5.12), `squad_cp: SquadAuras { count, coaction_drift }` (E-5.10), `squad_shield_disruption:
SquadAuras` (E-5.16), `shield_strips: Vec<f64>` (final s_j·Str_j, E-5.16), `flat_strip_rate: f64` (r_flat, E-5.11),
`aura_guardian` and `aura_prosecutor: Vec<DamageType>` (E-5.27), `shield_removed_externally` (default true) and
`pt_shield_type: Option<DamageType>` (E-5.26). `EnemyInstance` carries the same fields; `EnemySetup::new(enemy, level)`
builds the defaults. server-api.md §4 now names the type `EnemySetup` (party 1 … 6, mission, context_overguard).

### Records (05 §4.1, §4.3; 02 §4.10; 03 §9.7; 06 §4.2; 07 E-7.43)

- `BodyPart.non_body` (05 §4.3), `EnemyFlags::CC_IMMUNE` (05 §4.1 `cc_immune`), `EximusData.kinds: Vec<EximusKind>`
  (`eximus_kinds`; `EximusView.kinds` for the UI). New fields carry `#[serde(default)]`, so the fixture JSON is
  unchanged.
- `AttackRoles` + `SPEARGUN_THROW`, `RADIAL_PARENT_PART`, `TORSO_ONLY` (02 §4.10), `DOT_DETONATE` (03 E-3.14).
- `Attack.forced_cond: Vec<ForcedProcRule { condition: ForcedCondition, procs: [ForcedCount { type, count }],
  replaces_data, rule: Option<ForcedRule> }>` (02 §4.10 `[[attack.forced]]`: `always`, `distance_max(m)`,
  `direct_hit`, `full_charge`, `last_shot`, `zoom`; rules `cedo`, `evensong`, `microwave`) and `Attack.hs_unique:
  Option<f64>` (M_hsu, Cernos Prime 1.5).
- `Synthesized::LightWavesFrom { normal }`, `HeavyWavesFrom { heavy }` and the attack ids `LIGHT_WAVES = "Light
  Waves"`, `HEAVY_WAVES = "Heavy Waves"` (07 E-7.43; the 04 §2.14 join key of rows 188-189).
- `Trigger::Duplex` documented as its own class DUPLEX (06 §4.2, E-6.12, D-608), no longer "schedules as SEMI".

### Effect DSL (04 §4.2)

- `HitUnit::Input` (`hit_unit = input`: one per melee input, Melee Animosity).
- `StackModel.unbounded` and `CounterBonusParams.unbounded` (ruling C-4 below); `CounterReset::Reload` (Hata-Satya
  `reset_on = (on_reload, 1)`; holstering is not modelled). Readable counter form: `unbounded = true` instead of `max`
  (rejected together with `max` or `max_rank`, and without `reset_on`); the compact form stores `max = 0`.
- The 04 §5.3 exclusion rule for uncatalogued conditional lines: curated header `excluded_lines = [{ line, reason }]`
  (`ExcludedLine`) and the report key `data.check.conditional_line_excluded [item, line]`
  (`DataKey::CheckConditionalLineExcluded`). Curated headers also gain `duration_scaled` and `default_caster` (buffs).

### Messages and `DocIssue` (ruling C-18)

`DocKey::AttackUnknown` (`doc.attack_unknown [field, attack]`, an attack key of `melee_plans` / `pellet_hit` that names
no attack). `DocIssue` lives in `wfd-model`: `pub type DocIssue = DocKey` (`message::doc`, re-exported by `message`),
because the decode report, `CalcReport.warnings` and 422 `issues` carry document issues as `Message`s keyed by
`DocKey` (`DocIssue::ItemUnknown` = `DocKey::ItemUnknown`, `doc.item_unknown`). T10 may wrap it in richer decode types
in `wfd-buildcode`. `cargo xtask doc-check` passes. Documented in domain-model.md §9.2 by the review fixes (item 6).

### API DTOs and server-api.md

- `EximusView.kinds?`, `BodyPartView.key` (required), `BuffSchema.duration_scaled` and `.default_caster` (required);
  server-api.md §5.5 and §5.6 list them.
- server-api.md §6.1 and §8.1 now state the accepted shapes of C-23 … C-30 (`ViewEv` steady metrics, `per_shot`,
  `per_magazine` and `hit_infight` optional; `TtkEv.kappa?`; `Summary.sd` / `.ci95` nullable and `ci99?`;
  `CapacityReport` with nullable Forma counts, `forma_applied_min?`, `warframe?`; `crit_tiers_steady?`, `uptime?`,
  `StatBlock.efr?`, `EnemyPanel.caps?`, `TimelineHit.statuses_added?`, TTK `breakdown?`; metric `ttk_noarmor`), and §4
  the T-08 `rank?` of C-15. `api::tests::shapes` keeps only T01b's own `SearchParams` entries in
  `KNOWN_DIFFERENCES` and drops the `ttk_noarmor` type difference; `tests/ts_schema.rs` has no known difference left.
  engine.md §7.7 / §8.4 still name the old shapes (`kappa: f64`, required metrics): T02a, which owns engine.md, per the
  ruling.

### xtask (T01c files: test-only changes forced by the spec rounds and the §8 ownership row)

- `spec_sync.rs` tests re-pinned to the registers as T01d found them (checked row by row): 03 has 28 rows / 16 Compat
  (D-095 "Sonar on DoT" added with switch `—`; D-028's 03 row now `User (T-26) + Compat`, settling C-17), 06 has 8
  Compat (D-608 "Duplex cadence" became Compat: new flag `D608_DUPLEX_CADENCE`); 307 rows, 197 ids, **74 flags**;
  register warnings: none. Module comment cites ruling C-20.
- `main.rs::stub_modules_are_the_owned_files_of_section_8`: `xtask/Cargo.toml` (put in the T01c row of §8 by ruling
  C-21) is checked as T01c's and skipped as a command module (it failed on the branch head).

### Rulings requested of T01d (provisional; evidence for the manager)

| Id | Provisional ruling | Evidence |
|---|---|---|
| C-4 | Not every `max_stacks = none` effect is a `counter_bonus`. Event-driven counters without a cap until their reset are `counter_bonus` with `unbounded = true`: Secondary Enervate (`reset_on = { crit_tier_at_least = 2 }`, `reset_after = 6`) and Hata-Satya (`reset_on = "reload"`). User counts without a limit are `stacks = { max = <default count>, unbounded = true }`: Magus Accelerant / Destruct (class (c) count of applications; `CondOverride::Stacks` may exceed `max`). Smallest backward-compatible change: two `#[serde(default)] unbounded: bool` fields and one `CounterReset` variant. | 04 §4.2 row `gain g, max_stacks m` ("`none`: unbounded until reset"), E-4.8 "m = none: no cap", §4.3 "Effects without a maximum" (Secondary Enervate); the three rows found with `max_stacks = none` / "no limit" in 04: §5.3 Hata-Satya (`max_stacks = none`, `reset_on = (on_reload, 1)`), §5.4 Secondary Enervate (`max_stacks = none`, `reset_on = (on_big_crit, 6)`), §5.5 Magus Accelerant ("permanent, no limit", "(c) count of applications"). Tests: `domain_examples.rs::unbounded_counters_and_user_counts`; §16.13 Magus row now `{ max = 1, unbounded = true }` |
| C-12 | Confirmed, no model change: T18 emits one child attack per parent variant, each with its own `parent` and the `CoSpec` of the row whose `parent_variant` names that parent (Mutalist Cernos "Toxin Cloud": row 27 multiplying ×1 under the uncharged shot, row 28 adding ×41 under the charged shot). | 04 §2.14 (final text, l.523-526): "`parent_variant X` (rows 27, 28): the row applies to child instances spawned by the parent attack X only; the Mutalist Cernos data has one 'Toxin Cloud' attack, which is `multiplying` at ratio 1 when spawned by the uncharged shot and `adding` at ratio 41 when spawned by the charged shot"; 06 E-6.23 gives a child exactly one `parent(a)`, so a cloud spawned by two parents has to exist once per parent anyway (`Attack.parent: Option<AttackIdx>`) |

### Renumbering (ruling of 2026-11-27)

Applied: the T01b sections above use C-26 … C-30 (table rows, "C-30 addition", the `dmg_per_mag` reference, the
numbering note), `api/tests/shapes.rs` no longer cites "C-19" (the entry was removed), server-api.md cites C-23 …
C-30. T01c's C-17 … C-22 and their references in `xtask/` and work-breakdown.md §2.2 keep their numbers. No other
code or document under `crates/`, `xtask/`, `docs/architecture/`, `docs/data/` named T01b's old numbers
(`docs/project/` is the manager's).

### Documents changed (passages)

domain-model.md §3 (`Trigger`, `AttackRoles`), §4 (`Attack`, `ForcedProcRule`, `Synthesized`), §5 (`EximusData`,
`EnemyFlags`, `BodyPart`), §6 (`ItemKind::Buff`), §7 (toggle descriptor comment), §9.1 (`EnemySetup`, `BuffPick`,
`Caster`, `CasterKind`, the compat example), §9.2, §9.3 (table T-01 … T-43, `TOGGLE_MAP`, `Toggles`, `MeleePlanSel`,
plan mapping), §9.5 (switch-kind rule, real flag names, count 74 when T01d landed), §11.1 (`VulnMask` defaults of
C-14, `Cond::Trigger.hit_unit`, `StackModel.unbounded`), §11.2 (`counter_bonus`), §15.1 (`excluded_lines`, buff
fields), §16.13 (Spiteful Defilement `cd_abs` per D-067 as amended; Magus Accelerant unbounded), §17 (T-30, T-01 …
T-43). server-api.md §4, §5.5, §5.6, §6.1, §8.1. overview.md AD-20 and I-2 (C-20, C-22); testing.md §2.5 (C-20,
C-22); work-breakdown.md §2.4 (R-1 settled) and the T01c section (C-20 wording, count). Fixture edits forced by the
renamed/new toggles: the two Braton Prime scenarios (T-22), the Roar buff (T-37), the §17 row name in
`mini/COVERAGE.md`, the toggle note in `data/fixtures/README.md`.

### Left for other owners (stale toggle numbers or shapes in files T01d does not own)

- overview.md §1 l.40 ("T-01..T-29 (and the proposed T-30..T-32)"), AD-25 ("proposed T-32"), §6.2 R-1 (settled by
  00 §6.1): manager / T41.
- optimizer.md l.60, l.64, l.249, l.271, l.437 (proposed T-30 / T-31 / T-32 → T-35 / T-36 / T-30); frontend.md l.122
  ("proposed T-32" → T-30): T41.
- work-breakdown.md T03b section ("toggle descriptors for T-01..T-29 and proposed T-30..T-32"): manager.
- engine.md §7.7, §8.4 (C-23 … C-30 shapes) and §5 (`MeleePlan` from `melee_plans`, the T-41 / T-42 / T-43 inputs):
  T02a.
- domain-model.md §16.13 Melee Doughty still has the old `cd_rel` × floor(SC_PUN / 0.10) form; 02 E-2.5a defines a
  different bucket and formula (T03 review issue 5). Whether the DSL can express E-2.5a is a T19 / T20-T29 question;
  T01d did not change the row. The other T03 review issues are untouched (open-items.md §3).

### Review fixes (T01d, second pass)

An independent review of the T01d commit found eleven issues; each was checked against the code, this task's section
of work-breakdown.md and docs/mechanics. All eleven hold; fixes, in the reviewer's order:

1. **`on_plain_hit`** (04 §4.1, E-4.9; added in the spec's second tail round, after T01a): `TriggerId::OnPlainHit`,
   class (b), readable `"on_plain_hit"`. Laetum Overwhelming Attrition is now encoded as 04 §5.12 states it (an on-hit
   buff, +4.00 × 3 stacks, 10 s, `shared_lose_one`, used by every later instance whatever its rolls; D-410, TV-04-47):
   domain-model.md §16.13 row rewritten (it still had the pre-tail-round hit-outcome gate), §10 reader table names
   `OnPlainHit`, §11.1 and `effect.rs` no longer cite Laetum for `OutcomeFilter` (kept as an R-2 construct without a
   catalogue user), and the §17 "Hit-outcome gate" row says so (row names unchanged: T03's `COVERAGE.md` test pins
   them, and every §17 row needs a `COVERAGE.md` row). Tests: `effect::tests::trigger_ids_added_by_t01d`, the Laetum row of
   `tests/domain_examples.rs`.
2. **`ElemOrder::WarframeArcane`** (04 §4.2 `position = warframe_arcane`; 01 §4.10.1 position 10, E-1.8 sub-index =
   HCET index; TV-01-38). Position 9 (weapon arcane sub 0, Incarnon perk sub = tier) stays `ElemOrder::Slot` through the
   slot layout: `SlotDef.element_order = 9` for `arcane<n>` and `inc<t>`, sub-index from the slot kind (0) or the `inc<t>`
   evolution t; a slot without `element_order` cannot hold a combining `Slot` element (validator). domain-model.md §13
   table rewritten (with the 04 §4.2 value mapping), `SlotDef.element_order` doc in `rules.rs`. Appended as the last
   variant (compact indices unchanged). Test: `effect::tests::warframe_arcane_element_order`.
3. **Missing 04 §4.1 ids**: `WhileFiring` (b; Flux Overdrive), `WhileAirborne`, `WhileCrouching`, `WhileMoving`,
   `OnPull` (c; Soaring Strike, Lie In Wait, Exposing Harpoon), appended after `HealthLevel` so the compact variant
   indices of the T01a ids are unchanged. 04's `hits_within(k, Δt)` is `CountWithin { of: OnHit, n: k, window: Δt }`
   with the hits counted in the condition's `hit_unit` (`TriggerId` doc, domain-model.md §11.1, §16.13 Final Tap row).
   domain-model.md §11.1 now lists every id of the code (it also lacked T01a's `OnBigCrit`, `OnFire`, `OnMiss`,
   `OnDodge`, `HealthLevel`). Test: `effect::tests::trigger_ids_added_by_t01d` (TOML parse, class, compact round trip).
4. **T-32 binding** (02 E-2.10, 00 §6.1 T-32): new `EffectDef.spot: Option<SpotSource { location: ParamKey, nodes:
   Option<ParamKey> }>` (readable `spot = { location = "spot_location", nodes = "spot_nodes" }`, omitted when absent;
   rejected on non-`mult` targets at parse time; the validator, T19, checks that it is present exactly on members of
   `Spot`-stage groups). Semantics in domain-model.md §14: a `Spot` member is one source σ, its value is the factor s_σ
   of one node (not 1 + value; Sonar `value` 2 / 3 / 4 / 5 with `per = ["strength"]` gives 5 × Str as 02 §4.10 and 04
   E-4.10 require), `M_son = Π_σ s_σ^(ν_σ·spot_σ)` with L_σ and ν_σ read from the named params, and spot_σ = 1 on a
   direct hit makes the instance a weak-point hit. The same row now follows D-095 (`Part_dot = Part`, Sonar inherited
   by DoT; it said "not inherited by DoT"). **Bundle layout**: `EffectDef` gains a field (postcard is positional);
   `BUNDLE_SCHEMA_VERSION` stays 1 because no bundle has been built or published yet (no `data/generated/`), as for
   the other T01d bundle-type additions. Fixture edit forced by the binding: the Sonar item of
   `mini/items/buff.json` gains the `spot` field and the params `spot_location` (choice `hit_part` | `torso`, default
   `hit_part`) and `spot_nodes` (1 … 20, default 1; the upper bound is a UI bound, UNVERIFIED, `W:Sonar` L37), and its
   `value` param becomes the per-node factor (number 2 … 5, default 5.0; it was percent 4.0 read as 1 + value, which
   gives 1 + 4·Str instead of 5·Str); `mini/SOURCES.md` row 3612 updated. Tests: `effect::tests::spot_sources`, the
   Sonar row of `tests/domain_examples.rs`.
5. **`Caster::Ally.duration`**: no `skip_serializing_if` (it broke the postcard form of `Caster`, `BuffInstance`,
   `Scenario`, `Resolved` whenever the ally Duration was 1.0); `#[serde(default)]` keeps it optional on input, the
   readable form always writes it, TypeScript keeps `duration?`. Test: `scenario::tests::caster_compact_round_trip`.
   Note for the manager (not changed, T01a design): the resolved `Scenario` is still not postcard-safe in general,
   because `Toggles` holds `CoStatusCount.n` and `Stealth.rank` with `skip_serializing_if` (documents only); no code
   postcard-encodes a resolved scenario today.
6. **`DocIssue`**: kept in `wfd-model` and documented in domain-model.md §9.2 (the DTOs carry document issues as
   `Message`s keyed by `DocKey`, so the alias is the type they carry), as ruling C-18 requires for a type placed there.
7. **`CapacityRule` vs T-35 toggles** (non-breaking): precedence stated in server-api.md §8.1, domain-model.md §9.3 and
   the type's doc: the optimizer uses `constraints.capacity.mode` (not `input.scenario.toggles.capacity`) and
   `umbra_forma` replaces the `umbra` flag of `input.scenario.toggles.forma_types`; the other Forma types come from the
   scenario. New `CapacityRule::effective(&Toggles) -> (CapacityMode, FormaTypes)`; test
   `api::opt::capacity_tests`. Alternative for the manager (breaking, not done): drop `mode` and `umbra_forma` from
   `CapacityRule` and read T-35 from the scenario only.
8. **Files outside T01d's "Owns" line** — request for the manager's retroactive confirmation (no ruling grants them):
   `xtask/src/spec_sync.rs` (test pins re-counted to the registers, the D-028 / D-608 assertions, one module-comment
   line citing ruling C-20) and `xtask/src/main.rs` (`stub_modules_are_the_owned_files_of_section_8`: asserts
   `xtask/Cargo.toml` is T01c's per ruling C-21 and skips non-`.rs` paths when matching command modules; without it
   the test failed on the branch head after ruling C-21 put `xtask/Cargo.toml` in the §8 row), both T01c's files; and
   the T01a test files `crates/wfd-model/tests/{documents,domain_examples,ts_schema}.rs` (tests of the T01d changes;
   this pass changes `domain_examples.rs` again: the §16.13 Laetum row follows the document, Sonar and Final Tap rows
   added). If the manager prefers, T01c re-applies the xtask edits; the new tests of this pass live in `src/`.
9. **Semantic changes, stated** (both breaking items above stay pending the manager's approval): a document that omits
   `caster` on a T-13 Roar pick meant self-cast before T01d and means ally now (Roar's `default_caster`), which changes
   the 09 E-9.32 Helminth rule for it; a document that omits `combine_groups` meant "no family combined" and now means
   the profile default (every family in the game profile). Every document the code wrote before T01d wrote
   `combine_groups` (always written) and omitted `caster` only when self-cast, so the `caster` change is the one that
   reaches written documents. `DOC_VERSION` stays 1: acceptable while no share codes exist. The `Toggles` doc comment no
   longer claims that every older document keeps its meaning. Tests: `scenario::tests::combine_groups_forms` (`[]`
   stays `Some([])`, `null` and absent are `None`), `scenario::tests::buff_pick_caster_absent_is_default`.
10. **Duplicate attack entries** in `toggles.melee_plans` and `toggles.pellet_hit`: a second entry for one attack is
    the new document issue `DocKey::AttackDuplicate` (`doc.attack_duplicate [field, attack]`; the document is rejected,
    like `doc.slot_duplicate`), so no entry order decides a value (AD-19). domain-model.md §9.3, server-api.md §4,
    field docs. Enforcement belongs to resolve (T04).
11. Doc comments on the new variants `EximusKind::{Generic, Entropic, Guardian}` (05 §4.1, §4.5), `FinisherMode::*`
    (04 §7.2, 07 §9.11 Table 9-E) and `ForcedCondition::Always` (02 §4.10).

Regenerated: `frontend/src/api/generated/**` (`gen-ts`: `Caster`, `BuffPick`, `CapacityRule`, `FinisherMode`,
`Toggles`, `messageKeys`), `data/curated/schema.json` (`gen-schema`: `spot`, `SpotSource`, the new trigger ids and
`warframe_arcane`).

Left for other owners (added by this pass):

- T03 (fixture): the Laetum Overwhelming Attrition perk of `mini/items/incarnon.json` still has the gate form (+12.00
  with `scope.hit.outcome`); 04 §5.12 now gives `on_plain_hit` stacks (domain-model.md §16.13). The `sonar` group of
  `mini/rules.json` has `dot = "none"` citing the pre-rework DoT page; D-095 makes it `inherit`. The `arcane0` and
  `inc<t>` slots of the fixture layouts have no `element_order` (position 9 needs 9). No fixture item exercises
  `ElemOrder::WarframeArcane` (Theorem Demulcent, TV-01-38).
- T03b (rule tables): the same `element_order = 9` for the weapon arcane and Incarnon perk slots of the real layouts.
- T19 / T13: E-4.9 gives no rate for `while_firing` (Flux Overdrive's 04 row gives a per-mode value instead); the
  compile rule of domain-model.md §12.2 for it is open. The validator checks of items 2 and 4 (`Slot` elements only in
  slots with `element_order`; `spot` exactly on `Spot`-stage members, params present with the right kinds).
- T41: optimizer.md l.59 (`CapacityRule` comment: add the precedence of server-api.md §8.1); optimizer.md and
  testing.md describe Overwhelming Attrition as a reader of "instances with no crit and no status" (now the trigger
  rate of `OnPlainHit`; it stays a non-monotone reader).

## 2026-11-27 · Manager · Rulings on T01d

| Item | Ruling |
|---|---|
| C-4 (unbounded stacks) | Confirmed as T01d ruled: `counter_bonus` with `unbounded = true` (requires a reset event) for Secondary Enervate and Hata-Satya (`CounterReset::Reload`); user counts without a limit (Magus Accelerant, Destruct) are `stacks = { max = <default count>, unbounded = true }`. |
| C-12 (two CO specs for one data attack) | Confirmed: T18 emits one child attack per parent variant (04 §2.14 rows 27-28; 06 E-6.23 gives a child one parent). No model change. |
| Breaking T-22 `Toggles.combine_groups: Option<…>` | Approved. No crate outside `wfd-model` uses the type yet; the JSON form keeps `[]` meaning "no groups" and `null`/absent means the profile default. |
| Breaking T-37 `BuffPick.caster: Option<Caster>` | Approved, same reason; `None` = the buff's `default_caster` (Roar: ally). |
| Edits outside T01d's Owns line (`xtask/src/{main,spec_sync}.rs` pinned register tests and §8 ownership test; `crates/wfd-model/tests/*.rs`) | Confirmed retroactively: each re-pinned a test to the converged spec or followed a renamed type, which the contract sync requires. |
| `CapacityRule` vs T-35 | Keep the stated precedence (non-breaking, `CapacityRule::effective`); no clean-up of `mode` / `umbra_forma`. |
| Provisional choices (melee plan precedence: a `melee_plans` entry beats `melee_plan`; T-43 VIR overrides T-06; `SquadAuras` counts squadmates only) | Accepted provisionally; the whole-contract review before wave 1 rechecks them against 07 §8, 00 §6.1 and 05 OQ-523. |
| `BUNDLE_SCHEMA_VERSION` stays 1 | Accepted: no bundle has been built. The first bundle-emitting task (T18) owns the version from then on. |
| Resolved `Scenario` not postcard-safe (`skip_serializing_if` on `CoStatusCount.n`, `Stealth.rank`) | Noted for T02a (engine contracts) and T32/T33: whoever first postcard-encodes a resolved scenario (cache keys) makes it safe or uses another encoding; record it here. |
| Stale toggle numbers in overview.md, optimizer.md, frontend.md and the T03b section of work-breakdown.md | Fixed by the manager (T-30 melee plan, T-35 capacity mode, T-36 lower ranks). engine.md §5, §7.7, §8.4 shapes are T02a's (manager note in its run). |
| New Compat flag from the 2026-11-27 spec update | D-056 is now `User (T-05) + Compat` (the inspiration's DoT seeds count the status types of the hit's timeline entry, 04 §2.13). `compat_generated.rs` regenerated with `D056_CO_STATUS_COUNT` (75 flags; bits of D-058 and later shift by one: flags travel by name, so nothing persisted changes). |

## 2026-11-27 · T03c · Melee hand tables (one additive model field)

T03c adds `data/curated/rules/melee/{heavy_class,heavy_kappa,attack_triggers,stance_co,finishers,combo_specials}.toml`
(07 §4.4 tables 9-A to 9-E and the combo specials) and the acceptance test `crates/wfd-model/tests/melee_tables.rs`
(auto-discovered test target, no manifest change).

### Model change (additive)

- `FinisherRow.weapons: Vec<UniqueName>` (`#[serde(default)]`, empty = every weapon of `class`). Reason: table 9-E
  (`W:Finisher` §All Finishers Multiplier Comparison, 07 §9.11) lists two exalted weapons by name, Exalted Blade and
  Valkyr Talons ("Valkyr Talons ground 54× (10× × 5 + 4×)" is one of 07 §9.11's examples). Both have the class
  `ExaltedWeapon`, so `class` alone cannot tell their rows apart. Same shape as `MeleeTriggerRow.weapons`. TOML and
  JSON without the field read as before; the postcard layout of `MeleeRules` gains the field (no bundle has been
  built, `BUNDLE_SCHEMA_VERSION` stays 1, as ruled for T01d). `data/curated/schema.json` regenerated (`gen-schema`);
  `gen-ts` output unchanged (`FinisherRow` is not exported to TypeScript). The struct literal in the T01a test
  `crates/wfd-model/tests/documents.rs` gains `weapons: vec![]` (forced by the field). Affects T08b (finisher mode
  T-34 picks the weapon's row before the class row), T18 (copies the field into the bundle).

### Data choices the readers of these tables need (spec conflicts and gaps, work-breakdown.md §2.4)

- **9-A Bayonet has no row.** 07 table 9-A lists Bayonet with "–" for both heavies and the wind-up, and
  `HeavyClassRow.total: [Option<f64>; 2]` cannot hold `None` in TOML (no null; the field has no default). The row is
  omitted, which E-7.40 rule 2 treats identically (σ_h = 1 for a class without a row; TV-07-21 "no 9-A row"). If a
  later task needs the explicit row, `total` needs `#[serde(default)]` (request for the manager, not done).
- **9-A breakdowns** exist only where `W:Melee` gives a hit split in parentheses; "5× Bleed" without a split has no
  breakdown (its Bleed is in the row source). The Dagger heavy 1 Knockback is a `cc` name, not a proc. Note for T03:
  `data/fixtures/mini/rules.json` encodes that Knockback as an `IMP` proc; E-7.40 rule 2 keeps only Bleed procs.
- **9-D exalted rows.** The six exalted stances have class `exalted_weapon` and `first_of_class = false`; E-7.21 reads
  each one's shared columns (Slide, Aerial, …) from its own row ("exalted stances: their own row"), so T08b must not
  look up a class-first row for `ExaltedWeapon`. Blank cells have no entry (E-7.21: 1.0); the mini fixture writes 1.0.
- **9-C beyond the rows of the 07 §9.7 table**, both from 07 E-7.43: the Exalted Blade slide waves (`trigger =
  "slide"`, one `Light Waves` per slide strike, Decision of E-7.43) and the Exalted Blade heavy waves (`Heavy Waves`,
  `fixed = 2`, `heavy_combo = false` because the waves take no M_h, `heavy_bonus = true` for the `heavy` flag). Counts
  `hits` mean one attack per strike of the triggering entry (E-7.0 step 6, E-7.45); `combo_ranged` rows carry no
  `input` (every Ranged entry of the combo). Not mapped, as 07 §9.7 says: Incarnon/conditional attacks, the four
  unmapped Ranged entries (OQ-727) and the Ranged entries of the stanceless Gunblade Finisher (finishers use 9-E).
- **Combo specials: no D0_cb override.** Xoris has no `ComboDur`, which `MeleeData.combo_dur = None` already encodes as
  no expiry; `ComboSpecials.combo_dur` is a plain `f64`, so an infinite override would not survive the JSON form
  (serde_json writes ∞ as `null`, which reads back as "no override"). If an infinite override is ever needed it needs
  `serde_util::ext_f64::option` on that field.
- **9-E page inconsistencies kept as written** (flagged UNVERIFIED in the row sources): Sparring back finisher split
  1.6× + 5.4× under an 8× total; Assault Saw universal 1×. The page has no Dual Nikanas row (no row here). Exalted
  Blade's row conflicts with its ability page (ground 400 %, prompted 3200 %, 07 OQ-721).
- uniqueNames come from the Public Export; the wiki module's `InternalName` of Quassus Prime
  (`…/TnBrokenFramePrimeWarfanWeapon`) differs from PE and WFCD (`…/PrimeQuassus/PrimeQuassusWeapon`), which T05/T18
  should report as a data warning.

## 2026-11-27 · T03c · Review fixes

- **9-C Forward Block.** D:Stances has `Ranged` entries in the Forward Block combo of every gunblade table (Gunblade
  L5346, L5348; High Noon L5817, L5823; Bullet Dance L1310, L1319), of Exalted Blade (L3914-3932) and of Serene Storm
  (L8557-8569); 07 §9.7 ("Ranged entries"), E-7.43 and E-7.45 cover them. `attack_triggers.toml` now has
  `combo_ranged = { combo = "forward_block" }` rows for the five gunblades, Exalted Blade (and Prime, Umbra) and Desert
  Wind (and Prime); the Forward rows' source ranges no longer include the Forward Block lines. Affects T08b, T18: a
  forward-block loop (T-30) of these weapons now has shots or waves.
- **Completeness tests.** `melee_tables.rs` checks that 9-C is exactly the trigger map written out from 07 table 9-C,
  E-7.43, E-7.45 and the D:Stances `Ranged` listing (both directions). Where the gitignored cache exists
  (`research/external`, or the directory named by the new test-only variable `WFD_EXTERNAL_DIR`), it also derives the
  map from the cached `Stances_data.lua` (every `Ranged` entry is a 9-C row or a listed exclusion: the four OQ-727
  entries, the Gunblade Finisher, the Glaive Heavy) and compares 9-D and 9-E cell by cell with the cached
  `W:Condition_Overload_(Mechanic)` and `W:Finisher` pages. Without the cache those three tests print a note and pass.
- **`wave` scaling on a heavy trigger.** The `Heavy Waves` row and the file notes now state that `scaling = "wave"` on a
  `heavy` trigger means the E-7.20b base B'_t = H_j·(Bw_t + V_t) + X_t with H_j of the Sword heavy h (E-7.40 rule 2),
  and on a `combo_ranged` or `slide` trigger the E-7.20 base with S_j (E-7.43). Request to T01a (owner of
  `weapon.rs`): the doc comment of `TriggerScaling::Wave` ("Normal Attack × S_j") describes only the light waves; it
  could say "light waves: Normal Attack × S_j (E-7.20); heavy waves (heavy trigger): E-7.20b with H_j". No shape change
  is needed: T08b tells the two apart by the trigger.
- **Notes.** `attack_triggers.toml` names the Glaive Heavy "Falling Star" Ranged entries (Stances_data.lua L5000-5002)
  as covered by 07 §9.4, with no 9-C row. `stance_co.toml` no longer says the first row "alone" fills the shared
  columns: Four Riders (L588) and Blind Justice (L644) repeat Slide 100 %, the value of their class's first row, and
  E-7.21 ignores those cells; the test now allows exactly those two.

Requests to the manager (not done by T03c):

| # | Item | Request |
|---|---|---|
| T03c-1 | 07 §7.6 L873-874 says the shared columns "are filled only on the first row of each class" | Reword: the first row's shared cells are authoritative; Four Riders and Blind Justice repeat Slide 100 % (same value, no effect on E-7.21). Spec text only. |
| T03c-2 | 9-A Bayonet row (07 table 9-A: "–" for both heavies and the wind-up) | Either amend 07 table 9-A / §4.4 to say a class without a row has σ = 1 (current data, E-7.40 rule 2), or approve `#[serde(default)]` on `HeavyClassRow.total` (additive, T01a) so T03c can author a Bayonet row with no totals. |
| T03c-3 | `FinisherRow.weapons` changes the postcard layout of `MeleeRules` with `BUNDLE_SCHEMA_VERSION` still 1 | Confirm that the T01d ruling (no bundle built yet, version stays 1) covers this change, and record it in manager-log.md. |
