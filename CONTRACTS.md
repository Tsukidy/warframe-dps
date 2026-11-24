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
