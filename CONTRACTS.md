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
`KNOWN_DIFFERENCES` list holds exactly the spec-driven differences below (C-17, C-18).

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
| C-17 | `ViewEv.per_shot`, `.per_magazine` | 08 §8.5: none for N_mag = ∞, not available for gauge-gated attacks; server-api §6.1: required numbers; engine.md §7.7: `per_magazine: Option` | both optional |
| C-18 | `Summary.sd`, `.ci95` | 08 E-8.19: R = 1 gives no sd (and no SE, so no CI); server-api §6.1: required | `number \| null`; `ci99?` added (E-8.19, used by E-8.24; engine.md §8.4 lists it) |
| C-19 | Optimizer metric `ttk_noarmor` | 08 §10 defines it; server-api §8.1 and optimizer.md §1 omit it | `Metric::TtkNoarmor` |
| C-20 | MC crit histogram | 08 §8.4: "normalised" histogram; engine.md §8.4: `[u64; N_TIER_BINS]` counts | `[f64; 8]` fractions |
| C-21 | Outputs the spec lists and the DTOs lacked | 08 §8.2/§8.6 TTK breakdown, §8.10 EFR and caps in the panels, §8.9 Debilitate's added procs, §10 EV rank and gap classes of re-ranked finalists | optional additions: `TtkEv.breakdown`, `TtkDist.breakdown`, `StatBlock.efr`, `EnemyPanel.caps`, `TimelineHit.statuses_added`, `RankedBuild.gap_classes`, `.ev_rank`; `OptResult.capacity` (optimizer.md §7 step 4, 09 E-9.16) |

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
  `StatusSeries` documents both evaluators. (C-21 addition.)
- 08 §8.4 defines two EV crit-tier distributions. `ViewEv.crit_tiers` is the EV **window** distribution (the
  counterpart of `ViewMc.crit_hist`, which already said "bins as `ViewEv.crit_tiers`"); the steady distribution is the
  new optional `crit_tiers_steady` (absent for gauge-gated attacks, which have no steady state). engine.md §7.7 does
  not say which one `crit_tiers` is. (C-21 addition.)
- `ViewEv.dps_burst`, `.dps_sustained` and `.hit_infight` are optional: 08 E-8.14 reports them, like `dmg_per_shot`
  and `dmg_per_mag` (C-17), as "not available" for gauge-gated attacks (request C-23; found while checking §8 for
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
and T01c also C-22; the manager renumbers one set on merge. "C-21" below is T01b's C-21; the new requests start at
C-23 to avoid a further collision. Provisional choice for each: the spec's, as implemented. Affected: T02a, T09, T12,
T15a-T17, T30-T35.

| # | Conflict | Spec | Provisional choice |
|---|---|---|---|
| C-21 (T01b, addition) | EV window curves and the two EV crit-tier distributions | 08 §3 output ids `uptime[s][k]`, `stacks_mean[s][k]`; 08 §8.3, §8.4 | `ViewEv.uptime?`; `crit_tiers` = window, `crit_tiers_steady?` |
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
