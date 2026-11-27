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

## 2026-11-27 · T02a · Engine skeleton and internal contracts

T02a creates every file of `crates/wfd-engine/src/**` with the public signatures of work-breakdown.md T02a and the
internal types of engine.md §2.1-§8. Bodies that implement a spec rule are `todo!("Txx")` naming the owner of
work-breakdown.md §8; their tests are `#[ignore = "Txx"]` (known answers already written: TV-02-01 crit quantization,
the E-2.5 f32 summation order, 02 E-2.7 tier distributions, E-3, E-5, E-7, E-8, E-10, 01 §4.11 HEA + COL, TV-08-01
Philox, TV-08-02 keyed draws and FNV-1a ids, an FNV-1a collision pair, the E-8.16 block-index overflow, TV-08-04
capped Poisson, TV-08-16 and the TTK quantum of E-8.25, the 03 E-3.5 "+1" of extra-hit procs, the `max` aggregate of
`cc_floor`, the engine.md §11.2 probe examples). Implemented because they carry no spec rule: `Engine::new`/`bundle`,
`StageCtx`, `CancelToken`, `EngineError`/`Cancelled` messages, `NoTrace`, the `Score` order and its DTO conversions,
`StatMask`, `StatVec::ZERO`, `mechanic_impl` (exhaustive dispatch), `Kernel` accessors, `Workspace`, `MechIdTable::get`,
`OrdinalTable`, `McRng::check`, `SubKey`, the (empty) stage and probe registries.

engine.md: T02a edited only the sections the manager's note gave it (§5, §7.7, §8.4, which now describe the code).
Every other place where the code differs from engine.md is listed below; the text T02a proposes for those sections is
request C-34 and is not applied. Nothing outside `wfd-engine` uses these types yet, so every item is an addition, not
a breaking change.

### Dependencies

`wfd-engine` declares `libm`, `smallvec`, `arrayvec`, `rayon` (optional, feature `parallel`, default on; overview.md
§3.1) and `serde_json` (needed by the frozen `stage_registry` signature, `fn(&serde_json::Value) -> …`; not listed
for the engine in overview.md §3.1; declared once, in `[dependencies]`). All are existing `[workspace.dependencies]`;
no new workspace dependency. `Cargo.lock` gains `rayon`, `rayon-core`, `either`, `crossbeam-*` and `libm` (no package
used them before).

### Engine API (engine.md §1)

- **L4 calls take the compiled build** (C-32). engine.md §1 gave `ev`, `mc` and `timeline` an `EvalUnit` (attack
  indices) but no build. `EvRequest<'r> { build: &'r CompiledBuild, fidelity, views: &'r [View], series_bin_s,
  breakdown }`, `McRequest<'r> { build, runs, seed, views, series_bin_s, parity, breakdown }` and
  `timeline(&self, cs, cb: &CompiledBuild, unit, view, run, limit)`; the evaluators resolve (L3) and schedule the
  unit's attacks themselves. Affects T15a-T15c, T16, T17, T34.
- **`mc` and `timeline` return `EngineError`** (C-35): `Engine::mc(..) -> Result<McOut, EngineError>` and
  `Engine::timeline(..) -> Result<Timeline, EngineError>` (engine.md §1: `Result<McOut, Cancelled>` and `Timeline`).
  A run can fail with `EngineError::RngIndexRange` (08 E-8.16, engine.md §8.2: "never a panic"), which neither
  former type can carry; a cancelled run set is `EngineError::Cancelled` (`From<Cancelled>` exists). Affects T16, T17,
  T33, T34.
- `Want.mc: Option<McWant { runs, seed, series_bin_s }>` (the resolved T-29 values; the DTO `api::McWant` has optional
  fields). `StageCtx::new(rules, compat)` (no-op trace) and `StageCtx::traced`.
- `EngineError::Cancelled` (engine.md §15 had no variant for a cancelled `calculate`; key `engine.cancelled`),
  `EngineError::message()` / `messages()` (the `{ key, args }` form of §15), `From<Cancelled> for EngineError`.
- The report DTOs are re-exported: `CalcReport`, `EvOut`, `Explanation`, `McOut`, `ParityOut`, `ProbeValue`,
  `Summary`, `Timeline`, `TtkDist`, `TtkEv`, `ViewEv`, `ViewMc`.
- `Metric` is `wfd_model::api::Metric` re-exported (it has `TtkNoarmor`, ruling C-28, and spells `HitInfight`;
  engine.md §7.9 has its own enum without `TtkNoarmor`).
- `Score` is the engine's own type (the DTO `api::Score` cannot get `Ord` outside `wfd-model`): `a > b` means a is
  better (band ascending, then primary, tie, tie2 descending); `PartialEq`/`Eq` agree with that order, so `Score` is
  not `#[derive(PartialEq)]` as engine.md §7.9 wrote. Floats compare by `total_cmp` after `Score::key`: −0.0 reads as
  +0.0 (two one-shot kills with T_EV = 0, EC-08-33, are ordered by κ, not by the sign of zero) and NaN as −∞ (a NaN
  never becomes the incumbent); `Score::new` stores −0.0 as +0.0 and debug-asserts that no field is NaN (a NaN bound
  would prune). **Band 2 follows 08 E-8.26** (C-33): primary = fraction of OG + SH + HP removed by t_ttk_max on the
  expected path of E-8.21, tie = `dps_sustained[auto]`; band 0 tie2 = −ammo_to_kill (08 §10). `From` conversions both
  ways; `Score::WORST`. `Kernel::new(cs, objective, statics)` and accessors (the fields stay private).

### Compile levels (engine.md §3, §5; domain-model.md §10, §18)

- `CompiledScenario.hit_fraction: SmallVec<[f64; 8]>` (T-41 f_a per attack, 1 without an entry) and
  `.external: [u8; N_ST]` (T-43 N_ext,s with T-06 folded in by `Toggles::external_stacks`, 0 on immune types).
- `CompiledItem` lives in `compile/pick.rs` (engine.md §2 layout; domain-model.md §18 names `compile/item.rs`) and
  adds `readers: SmallVec<[Reader; 1]>`: `monotone_classes(cs, pool)` must see each candidate's readers, which §18's
  fields do not carry. `Reader { reads: ReadQuantity, monotone, source: EffectRef }`, `ReadQuantity::{CritTier {
  crit_damage }, ProcOutcome, ScheduleMark, Resolved(ResolvedStat)}` (domain-model.md §10 reader table).
- `SparseContrib` parts: `ElemEntry { order: ElemOrder, line, ty: DamageType, value: i64 }`, `MechanicInst {
  mechanic, slot, source }`, `StaticTerm { slot, per: Per, value, source }`, `EffectRef { item, effect }`,
  `SourceIndex`; `StatVec::add/sub/to_f64`, `StatVecF::get`, `StatMask` bit set; `FIXED_SCALE = 10^9`.
- **Rows with the `max` aggregate** (`cc_floor`, `external.*`, domain-model.md §10) cannot be a fixed-point sum (two
  sources 2.0 and 1.5 must give 2.0, not 3.5). They travel in the new `SparseContrib.max_entries: SmallVec<[(u16,
  i64); 1]>` (never in `entries`); `StatVec::add`/`sub` never touch them (so `sub` stays the exact inverse of `add`);
  the new `StatVec::raise_max(&mut self, &SparseContrib)` keeps the maximum in the slot (L2 applies every
  contribution, scenario-fixed sources included); `to_f64` reads them as stored. A maximum has no inverse: the
  optimizer's incremental search (T32) recomputes a `max` slot from the remaining contributions when it removes one
  with `max_entries`. Affects T06, T32, T33.
- `OrderedElem` is the alias `(OrderKey, DamageType, f64)`, so engine.md §3's `elements::combine(&[(OrderKey,
  DamageType, f64)])` and work-breakdown.md's `combine_elements(&[OrderedElem])` are one function:
  `resolve::combine_elements` is `pub use crate::elements::combine as combine_elements` (T06 implements it once).
  `ElementSolver::outcomes(fixed: &[OrderedElem], set: &[ElemCandidate], slots: &[OrderKey])`.
- **T-30 melee plans** (manager note; domain-model.md §9.3): new `plan_selection(toggles, key: &AttackKey, atk) ->
  MeleePlanSel` applies the precedence (`melee_plans` entry, else `melee_plan` when its kind names the attack, else
  `Auto`; an unknown attack key was already dropped with `doc.attack_unknown` and a duplicate rejected the document);
  the frozen `melee_plan(sel, …)` maps the selection and applies T-09 (E-7.31). `MeleePlan` gains
  `heavy_cancel: Option<f64>` and `t_cyc: Option<f64>` (the parameters of `MeleePlanSel::HeavyLoop` and
  `DataAttackLoop` that engine.md §5 left out). `MeleeTablesOwned { combos: [Option<ComboRef { table, index }>;
  N_COMBO_TYPES] }` with `tables(&Bundle, &MeleeData) -> MeleeTables`.
- `AssembledWeapon { id, weapon: Weapon, progenitor: Option<ProgenitorBonus>, m_str }`, `assemble(b, build, toggles)`.

### Resolve, schedule, enemy, status (engine.md §4-§6)

- `ResolvedAttack.hit_fraction` (T-41); `FiringResolved.gauge_gated` and `.gauge_uses_per_min` (T-42, set at L3);
  `StatusResolved.n_ext` (T-43); the parts `CoResolved`, `HitPart`, `PartResolved`, `CritResolved` (with the Prelude
  pair `q`, `q_cond`, `theta_p`), `StatusResolved`, `ExtraHitResolved`, `MechResolved` (16 slots), `GateSet` (4 gates)
  are concrete; `N_XH = 8`, `N_MECH = 16`, `MAX_SPLIT_GATES = 4`. A unit test keeps `ResolvedAttack`, `PointState`,
  `ShotRec`, `ReloadRec`, `MeleePlan` and `ScheduleInputs` `Copy`.
- **Schedule inputs.** `ranged_schedule(ctx, ra, horizon)` reads no bundle, so `FiringResolved` gains
  `fire_rate_trig` (FR_trig, 06 E-6.3) and `sched: ScheduleInputs` (fixed size, `Copy`) with every per-attack input of
  06 E-6.11 and of the variants it names: `TimingBase` (FR0, ΠM_fr, FR_cap, lock, CT0, `charge_fixed`, BD0, Mag0,
  `mag_fixed`, cost0) with the static `TimingSums` (ΣFR, charge, ΣMag, Mag+, RS, ΣAE) for the re-evaluation at the
  points of 06 §5.9; `timing_buffs: [TimingBuff { trigger: TimingTrigger::{Reload, ReloadFromEmpty, FirstShot,
  LastShot}, duration, add: TimingSums }; MAX_TIMING_BUFFS = 8]` with `n_timing_buffs`; `cost_factor` (infinite-ammo
  state, 06 §5.6, §5.9); `ReloadInputs { style: ReloadStyleInputs::{Magazine { r0 }, ByRound { t_se, t_sh },
  Regenerate { delay, delay_empty, rate, rs_scales_delay }, None}, no_lead, burst_reload_delay, timed_reload }` (06
  E-6.9, E-6.10, T-40); `form: Option<FormInputs { mag, t_sw, switch_counted }>` (E-6.14, T-39); `t_ret` (06 §6.4);
  `volley: Option<VolleyInputs { lead, members, cost_per_pull }>` (E-6.14b); `on_reload: ReloadTriggered {
  from_empty, partial }` (06 §9.5); `ramp_start`; `spool: Option<SpoolInputs { start, step }>`; `kohm_max_pellets`;
  `peacemaker: Option<PeacemakerInputs>`; `sniper: Option<SniperInputs { min, reset, fixed, aiming }>` (E-6.16, T-12,
  T-38); `flight_time`, `out_of_range` (E-6.22, §8.4, T-27); `children: [ChildLink { attack, parent, per_projectile,
  split, needs_head, needs_stun, delay, lingering: Option<LingerInputs { duration, rate }> }; MAX_CHILD_LINKS = 8]`
  with `n_children` (E-6.23-E-6.25). `firing_stats` (T08a) fills the data-derived fields; L3 (`Engine::resolve`,
  T13) sets the toggle-derived ones (as for T-42) and `timing_buffs` from the compiled build (a build with more than
  8 timing buffs on one attack is rejected at L2). The roles mark the variants (`WHOLE_MAGAZINE`, `RETURNING`,
  `RELOAD_TRIGGERED`, `GAUGE_GATED`, `INCARNON_FORM`). Affects T06, T08a, T13, T15a, T16. engine.md §5 shows it.
- `CritMult { x: f32, code, q, cdm }`, `TierDist { base, p: [f64; 3], mean }` (02 E-2.7: at most three adjacent
  tiers), `ScalarY` affine in C (`at(c)`), `WeaponMult { faction, g, g_f, m_og, dot_only }` and `weapon_mult(ctx, sv,
  slots)` (ST-5 had no frozen function).
- **Crit multiplier summation order** (02 E-2.5): new `crit_multiplier_terms(cdm0, cdm0_plus: &[f64], cdm0c, cd_rel,
  cd_abs, m_cdf) -> CritMult` sums x32 in f32 left to right, f32(CDM0), each CDM0+ entry in catalogue order, then
  f32(cond·CDM0c); the frozen `crit_multiplier(cdm0, cd_base_add, ..)` is `crit_multiplier_terms(cdm0,
  &[cd_base_add], 0.0, ..)`. The Prelude term is never folded into `cd_base_add` (a pre-sum in f64 can change x32 by
  an ulp and so the code). Several CDM0+ entries: request C-36.
- `CAP_UNBOUNDED = u8::MAX` (`status::rules`) is the stored cap of a status without a cap (03 §4.1 base cap ∞ of SLA,
  HEA, ELE, TOX) in `StatusResolved.caps`, `ScaledEnemy.caps` and `capped_poisson_mean` (result μ).
- `part_factor(p_part, k_wp, wp, hs, m_son)` computes `(P_p + k_wp·ΣWP)·(1 + ΣHS)·M_son`; the caller passes 0 for the
  terms that do not apply and applies M_hsu, `non_body` and ΣBP (E-7).
- The schedule functions are `ranged_schedule` / `melee_schedule` (engine.md §5 said `schedule::ranged::generate`,
  `schedule::melee::generate`). `Schedule` adds `children: Vec<ChildHits>` and `sniper_decay: Vec<f64>` (06 §6.1
  records); `ShotRec` carries every 06 §6.1 field plus the melee `strike_mult`, `combo`, `weight`; `ShotFlags` is a
  `u16` newtype (no `bitflags` dependency in the engine). `MeleeTrack { heavies: Vec<HeavyRec>, combo: Vec<ComboRec> }`.
- `instance_on_layer` leaves the weapon-side M_OG to the caller (a scalar on the OG layer); `TargetStateView { stacks,
  armor_eff, vuln: [DamageVec; 3], status_instance, part_dr }`; `LayeredHit { og, sh, hp_bypass, hp, hp_by_type,
  weak_point, tick }` (`hp_by_type` = D^HP_t per type, from which routing forms h_t for the adaptation state, 05
  E-5.23, E-5.25); `Pools { og, sh, hp, gate_until: Option<i64> (µs grid, 08 E-8.0a), adaptation: AdaptationState {
  adapted, n, gates_crossed, dmg_by_type }, health_below: u32 }` (08 §6.2 "pools: OG, SH, HP, gate_until, adaptation
  state, health-threshold flags"; route_layered converts its `t` to the grid once); `RouteOut`;
  `ScaledEnemy`/`ScaledPart` field sets of 05.
- `dot_tick(ctx, snap, s)` is the single-proc weapon-side tick `(seed_p + a)·X_p` before M_tick and the enemy side,
  with a = `dot_acc` when the new `ProcSnapshot.plus_one` is set and 0 for the forced proc of an extra hit (00 E-14:
  S_p = XH, no "+1") and Primary Debilitate's proc (03 §9.8, E-3.5); `extra_hit(ctx, parent: &HitWeaponSide, xh) ->
  ExtraHitOut` is the new file `status/extra_hit.rs`.

### Evaluators (engine.md §2.1, §7, §8)

- The state types of the mechanic traits live with their evaluators and are re-exported by `mechanics`: `PointState`,
  `EvAcc`, `ProcMass` in `ev/kernel.rs` (T15a), `McState`, `McEvent`, `McEventKind`, `Phase`, `SubKey`, `Anchor`,
  `McSink` in `mc/events.rs` / `mc/state.rs`, `McRng` in `mc/rng.rs`, `MechanicId` in `mc/keys.rs` (T16).
  `MechInst { mechanic, magnitude, source, key: EffectKey, ids: SmallVec<[MechanicId; 2]> }` (`ids`: the hashes of
  `mechanic_ids`, so `mc_instance` draws without hashing). Each mechanic kind is a unit type in `mechanics/<kind>.rs`.
- **`MechanicMc::mechanic_ids` returns the id strings** (`SmallVec<[Box<str>; 2]>`; engine.md §2.1:
  `SmallVec<[MechanicId; 2]>`): L0 registers each with `MechIdTable::insert(&str)`, which needs the strings to detect
  and report a collision (`MechanicIdCollision { a, b }`, engine.md §8.2, 08 E-8.17); two colliding strings would give
  equal hashes and could not be told apart.
- **MC event order (08 §4.3, §6.2).** `McEvent { t, t_us, phase, sub_key: SubKey, seq, kind, anchor }` with the heap
  key `(t_us, phase, sub_key, seq)`. `t_us` is the grid time τ̂ of 08 E-8.0a. `Phase` has the four heap phases of
  08 §4.3 (`Transitions` = 1, `Damage` = 2, `HeatEnd` = 3, `Shot` = 4): the steps a-d of a shot (instances, procs,
  zero-delay ticks, DoT detonation) run inside that shot's P4 step, in the shot order of 08 §4.2, so a stack applied
  in P4b of S affects every later shot at τ (03 §7.1, §7.2); they are not heap phases. `SubKey::shot(firing, shot,
  child)` packs the 08 §4.2 order key (firing attack, shot index, child attack; a parent shot before its delayed
  children), `SubKey::tick_group(oldest_proc_seq)` the P2 key, `SubKey::NONE` P1 and P3. `McEventKind::Child {
  attack, entry, tick }` is a delayed child or lingering tick of `Schedule.children` (08 §4.2: a child at a later time
  forms its own event). `Anchor { a: u16, i: u32, j: u16 }` with the sentinels of E-8.16.
- **MC run state (08 §6.2).** `McState { t_us, og_view, stacks: [Vec<StackEntry>; N_ST], external, dots:
  Vec<Option<DotInst>>, heat: HeatState, blast_fuses: Vec<BlastFuse>, frozen_until, ragdoll_until, single:
  [Option<SingleState>; 4] (VOI, KND, LIF, MIC), armor_strip_flat, pools, cursor, combo: ComboState, sniper_combo,
  conditions: Vec<CondState>, switches: Vec<SwitchState>, counters }`. `StackEntry { t_apply_us, t_expire_us, source,
  seq, dot }` keeps both times and the source attack, ordered by application, so 03 §6.2 replaces the stack with the
  smallest application time (not the first to expire). The per-status stores are `Vec`s: SLA, ELE and TOX have cap
  ∞ and COR 10 + β_COR (03 §4.1), so no fixed size and no panic. `DotInst { status, seq, snap, next_tick, cancelled,
  magnetic_break }`, `HeatState { live, t_ign, t_exp, next_pulse, pool: Vec<HeatProc>, strip, strip_steps }`. The
  accumulators and the event heap of 08 §6.2 are the simulator's own (`mc/sim.rs`). engine.md §8.1's
  `ArrayVec<f64, 16>` stack lists are replaced (C-34).
- **RNG range errors.** `McRng::new(seed, run, &mut OrdinalTable)`, `set_anchor` (returns the range error), and
  `next_u(mechanic) -> f64`, which records the first range error (block b ≥ 65536 after 131,072 draws of one mechanic
  at one anchor) instead of wrapping or panicking, returns 0.0 and draws nothing; `McRng::error()` and
  `McRng::check() -> Result<(), EngineError>`, which the simulator calls after each event (08 E-8.16, engine.md
  §8.2). `FIXED_MECHANIC_IDS` = the 22 ids of TV-08-02 in E-8.17 draw order.
- **OC2 quantum.** New `oracle_compare_q(x, y, q)` (08 E-8.25: q = 1 for the DPS and per-bullet values, 0.001 s for
  the TTK median, with the +∞ rules); the frozen `oracle_compare(x, y)` is the q = 1 case. Constants
  `OC2_QUANTUM_DPS`, `OC2_QUANTUM_TTK`.
- `ev::kernel::point(ra, st, view) -> PointOut` (engine.md §7.1 had a `ws` argument the point does not need).
- engine.md §7.7 and §8.4 now state the accepted DTO shapes of C-23 … C-30 (optional steady and per-shot metrics,
  κ only in band killed, `Summary.sd` / `ci95` nullable and `ci99`, MC crit histogram as fractions, `null` = not
  killed), as the manager's ruling on T01d asked of T02a.
- `TraceEntry` (engine side) adds `value: f64` (the DTO has it); `trait TraceSink: Sync { enabled, record,
  touch_row }`. `ProbePath { scope, name, state }` is parsed (`ProbePath::parse`, T17); `probe::registry()` and
  `stage_registry()` are empty until T17 and T09 fill them. `stage_registry` returns `&'static [(&'static str,
  StageFn)]` where `StageFn` is exactly `fn(&serde_json::Value) -> Result<serde_json::Value, StageError>`.
- `spec_constants.rs` starts with the constants named by the equations the skeleton cites (E-3 grid 32, E-4 clamp and
  4095 codes, the 10^6 time grid, `ARITH_EPS` = 1e-9, the E-8.24 test constants, the E-8.25 quanta), each with its
  equation. `ARITH_EPS` guards the CD-06 tick count and the ammo and magazine arithmetic of 06 only; it is never a time
  tolerance: every time test compares grid values (CD-14, 08 E-8.0a, 06 E-6.11).

### Manager ruling on postcard and resolved scenarios

No T02a contract postcard-encodes a resolved `Scenario` (the engine computes no cache keys); the ruling of
2026-11-27 passes unchanged to T32 / T33 (`wfd-opt` `keys.rs`).

### Left for other owners

- domain-model.md §18 (T01a's document, T41 after wave 3): `CompiledItem` path `compile/pick.rs` and its `readers`
  field; `SparseContrib.max_entries` and `StatVec::raise_max` (the `max` rows).
- optimizer.md l.45 (`Metric` list: `HitInFight` → `HitInfight`, add `TtkNoarmor`) (T41).

### Requests to the manager (work-breakdown.md §2.4)

Numbered from C-31, the next free number at the manager's 2026-11-27 rulings; parallel tasks (T03, T12) numbered
their own requests from the same point, so renumber at merge.

| # | Finding | Provisional choice |
|---|---|---|
| C-31 | `crates/wfd-engine/src/{error,cancel}.rs` have no owner in work-breakdown.md §8 after T02a (the T17 row lists `lib, outputs, compat, trace, probe`) | T02a wrote them completely; please assign them (T17 suggested) for later changes |
| C-32 | engine.md §1 gave the L4 calls no compiled build | `EvRequest.build`, `McRequest.build`, `timeline(.., cb, ..)` in the code (additions); engine.md §1 not edited (C-34) |
| C-33 | engine.md §7.9 ranks TTK band 2 by "primary = dps_sustained on the layered view's last layer"; 08 E-8.26 ranks band 2 by the larger fraction of the total pool (OG + SH + HP) removed by t_ttk_max on the expected path of E-8.21, then by the higher `dps_sustained[auto]`. The spec wins (work-breakdown.md §2.4). | `Score` band 2: primary = removed fraction, tie = `dps_sustained[auto]` (doc of `ev::score::Score`). Proposed §7.9 text: "band 2 never: primary = fraction of OG + SH + HP removed by t_ttk_max on the expected path of E-8.21, tie = dps_sustained[auto]"; and in "Why the TTK bound is valid": "band 2 only when every completion is band 2, and then the envelope path removes at least the fraction of every completion (pools deplete no later on it)". Affects T15a, T15c, T32. |
| C-34 | engine.md sections outside §5, §7.7, §8.4 differ from the code; T02a may not edit them (manager's note) | Not applied; please apply or rule. Proposed edits: §1 the L4 signatures of C-32 and C-35, `McWant`, `EvRequest`, `McRequest`, `StageCtx`, the re-exported report types; §2 the module layout (`stage_registry.rs`, `status/extra_hit.rs`, `mechanics/mod.rs` with `MechInst` and `mechanic_impl`, the state types per evaluator file); §2.1 `MechanicImpl`, `mechanic_impl`, `MechInst` with `ids`, `mechanic_ids -> SmallVec<[Box<str>; 2]>`, `McEvent { t, t_us, phase, sub_key, seq, kind, anchor }` with the heap key `(t_us, phase, sub_key, seq)` and `Phase` = P1 … P4 (steps a-d inside the shot's P4 step, 08 §4.3); §3 `CompiledScenario.hit_fraction` / `.external`, `CompiledItem.readers`, `OrderedElem`, `ElementSolver::outcomes(fixed, set, slots)`, `SparseContrib.max_entries`; §4 `ResolvedAttack.hit_fraction`, `FiringResolved.fire_rate_trig` / `.sched`; §6 `LayeredHit.hp_by_type`, `Pools` with the grid `gate_until`, `adaptation`, `health_below`; §7.1 `point(ra, st, view)`; §7.9 `Metric` re-exported, `Score` order (C-33, `Score::key`, `Score::new`), `Kernel::new`; §8.1 the `McState` of 08 §6.2 above (no `ArrayVec<f64, 16>`), and "Order: a binary heap keyed `(time, phase, seq)`" becomes `(τ̂, phase, sub_key, seq)` with the four phases of 08 §4.3 (the reviewed text contradicts 08 §4.3; the code follows the spec); §8.2 `OrdinalTable`, `set_anchor`, `next_u` with `McRng::check`, the anchor sentinels; §11.1 `TraceEntry.value`, `TraceSink`; §15 `EngineError::Cancelled`, `message()` / `messages()` |
| C-35 | engine.md §1 `Engine::mc -> Result<McOut, Cancelled>` and `timeline -> Timeline` cannot report `EngineError::RngIndexRange`, which 08 E-8.16 and engine.md §8.2 require instead of a wrap or a panic | `Result<McOut, EngineError>` and `Result<Timeline, EngineError>` in the code (cancellation is `EngineError::Cancelled`) |
| C-36 | 02 E-2.5 sums several CDM0+ entries in f32 in catalogue order, but a `StatVec` keeps only their fixed-point sum in `cd_base_add`; with one entry (every weapon of the research, 02 E-2.5 note) the sum is the entry | `crit_multiplier_terms` takes the ordered entries. The calculator path passes them in catalogue order from `CompiledBuild.sources`; a path with only the `StatVec` (the optimizer's `BuildPoint`) passes the sum as one entry and may differ from E-2.5 by one f32 ulp in x32 when several CDM0+ entries exist (custom stats only). Please confirm, or ask for a per-entry store. |

## 2026-11-27 · T02b · `wfd-data` and `wfd-buildcode` skeletons (initial contract)

T02b fixes the public API of both crates as work-breakdown.md T02b writes it. Bodies are `todo!("T04 …")` (wfd-data)
and `todo!("T10 …")` (wfd-buildcode); tests that need them are `#[ignore = "T04 …"]` / `#[ignore = "T10 …"]`
(`crates/wfd-data/tests/load.rs`, `crates/wfd-buildcode/tests/codes.rs`). `tests/api.rs` of each crate pins every
signature of the "Provides" block as a function pointer, so a change fails to compile there. Implemented and tested
now: the `Entity` accessors (`entity_accessors_read_the_record`) and `CodeError::messages`
(`errors_become_import_messages`). Written but untested until T04, because a `GameData` can only be built by T04's
constructors: `GameData::bundle`, `GameData::image_path` (= `by_key(k)?.image()`; covered by the ignored
`every_stable_key_is_indexed`) and `min_forma` (delegates to `min_forma_with`, below). Items marked **request** are
listed at the end; every other item fills a shape the plan leaves open.

### `wfd-model` (T01a's crate; one new type)

- New `wfd_model::Pick` (`src/pick.rs`, re-exported at the root), shape exactly engine.md §1 (`Mod`, `Riven`,
  `Arcane`, `Perk`, `Shard`, `Stance`, `FrameMod`, `Aura`, `KitgunParts`, `ZawParts`, `Progenitor`), serde
  `snake_case`, no TS/schema derives (never sent over HTTP). A new type is the backward-compatible kind of change of
  work-breakdown.md §2.1; no existing model type changes. Reason: `legality::check_slot(…, pick: &Pick)`
  (work-breakdown.md T02b) names `Pick`, which engine.md §1 and work-breakdown.md T02a place in `wfd-engine`; but
  `wfd-data` and `wfd-engine` both depend on `wfd-model` and not on each other (overview.md §3.1), and the
  optimizer's pools hand one candidate to both `Engine::compile_pick` and `check_slot` (optimizer.md §2 steps 2 and
  5), so the model is the only crate both can name it from. **Request** T02b-2, **merge-blocking**: T02a, running in
  parallel, defines its own `pub enum Pick` with the same variants in `crates/wfd-engine/src/lib.rs` (its worktree,
  uncommitted at the time of this entry; no serde derives). Merged as they are, the two branches give two types and
  the optimizer's path does not type-check. Before either branch merges the manager rules on T02b-2; if accepted, the
  merge replaces T02a's enum with `pub use wfd_model::Pick;` and engine.md §1 says so (its owner's edit). Affects
  T02a, T32, T33. Test `pick::tests::pick_round_trips` (JSON and postcard, every variant).
- `Pick` gaps inherited from engine.md §1 (documented on `Pick` and `check_slot`, no variant added while T02b-2 is
  open): `Arcane` serves both the weapon slots `arcane0/1` and the warframe slots `farcane0/1` (the slot argument
  tells which, the family rule E-9.30 checks the fit), which covers the optimizer's "frame arcanes" position
  (optimizer.md §2 step 2). `SlotId::Subsumed` has no `Pick`, so the Helminth rules (09 §10, E-9.32) run in
  `legality::check` only; `KitgunParts`, `ZawParts` and `Progenitor` fill `Build.variant`, which no `SlotId` names,
  so their rules (T-25 parts, valence E-9.70) also run in `check`. `check_slot` reports a pairing outside its table
  as `slot.kind` (E-9.3). If `/api/candidates` for the `subsumed` slot (T04) or an optimizer position needs
  `check_slot` there, the fix is a new variant `Pick::Subsumed { ability: ItemId }` (a new enum variant of §2.1;
  `Engine::compile_pick` gains an arm) with the matching engine.md §1 edit. Affects T04, T32, T33.

`Resolved` is unchanged (domain-model.md §9.2: `{ build, scenario, violations }`); the resolve warnings travel in
`wfd-data`'s return type instead (request T02b-3).

### `wfd-data` (work-breakdown.md T02b; pipeline.md §8.3)

- `GameData` keeps its fields private (the bundle and T04's indexes, which must stay consistent with it) and exposes
  `bundle()`; pipeline.md §8.3 writes `pub bundle: Bundle`. Manual `Debug` (data version and record counts).
- `GameData::resolve(&CalcInput) -> Result<(Resolved, Vec<Message>), Vec<Message>>`: `Ok` carries the document issues
  resolution dropped with a warning instead of rejecting the document, in document order. work-breakdown.md T02b and
  domain-model.md §9.2 write `Result<Resolved, Vec<Message>>`, which has no channel for them, while
  domain-model.md §9.2 says document issues "travel as warnings (decode reports, `CalcReport.warnings`)",
  server-api.md §6.1 answers 422 only for a retired item that *cannot* be dropped, and `message/doc.rs` marks
  `doc.item_retired`, `doc.effect_unknown`, `doc.attack_unknown` as "dropped with this warning". **Request** T02b-3.
  Affects T04 (fills it), T11/T34 (append to `CalcReport.warnings`), T32/T33 (resolve the optimizer's input).
- `Entity<'a>` = `Weapon(&Weapon) | Item(&Item) | Enemy(&Enemy) | Frame(&Frame) | Part(u16, &ModularPart)` with
  `kind`, `key`, `unique_name`, `name`, `image`; `EntityKind` (five kinds). `Part` is added to the "Weapon | Item |
  Enemy | Frame" of work-breakdown.md (pipeline.md §8.3: "Weapon | Item | Enemy"): kitgun grips and loaders and zaw
  grips and links have StableKeys of their own (`VariantDoc.kitgun`, `.zaw`; `PartSummary.image` of server-api.md
  §5.3 is served by `/api/img/{key}`); the `u16` is the index in the `ModularTables` list of the part's kind, as
  `Variant.kitgun` / `.zaw` hold it. Kitgun chambers and zaw strikes share key and uniqueName with their weapon
  record (`data/fixtures/README.md`) and are found as `Entity::Weapon`. `Entity::image` is `None` for enemies
  (enemy.rs: no DE export; server-api.md §10 serves weapons, items and frames), although the T03 fixture enemies
  carry wiki image names.
- `SearchQuery { text, locale, kinds: Vec<EntityKind>, item_kinds: Vec<ItemKindName>, faction, stat, limit }`
  (`Default` = no filter) and `SearchHit { key, kind, score: u32 }`: an integer score, ties ordered by key (AD-19).
- `DataError` (`thiserror`, `#[non_exhaustive]`): `Io { path, source }`, `Json { path, message }`, `BadMagic`,
  `SchemaVersion { found, expected }`, `HashMismatch`, `Corrupt(Box<str>)`, `Invalid(Vec<Box<str>>)` (the reference
  validator of pipeline.md §8.2).
- "`weapon_detail` … and item_detail, enemy_detail" spelled out: `item_detail(k) -> Option<ItemDetail>`,
  `enemy_detail(k) -> Option<EnemyDetail>`.
- `legality::check_slot` documents which `SlotId`s each `Pick` goes into (09 §2.1) and the gaps of the `Pick` item
  above.
- `legality::FormaReport` = `wfd_model::api::CapacityReport` (the E-9.19 report the T01b review fixes tie to
  `min_forma`, request C-25), so the server copies it into `CalcReport.capacity` unchanged.
- `legality::TraitRange { trait_id, role: Option<TraitRole>, range: Option<ValueRange> }` with `TraitRole { Positive,
  Negative }` (09 E-9.54 sgn_i, E-9.56 [v_lo, v_hi]), one entry per trait in the riven's order. The fields follow the
  steps of E-9.53a, each of which uses only the traits that passed the earlier ones, so `range` present implies
  `role` present and not the reverse: both `None` for a trait that failed T1 or T2 (step 1) or has v_i = 0
  (sgn = 0, in neither n_pos nor n_neg); `role` only for an illegal layout (E-9.54; 04 E-4.18 has no layout factor L
  for it) or a trait that failed T3-T6 (step 3); both for the rest (step 4). Spec gap for T04: 09 does not say which
  rule a zero-valued trait breaks (E-9.56 cannot place it without L); T04 asks the manager.
- New `legality::min_forma_with(gd, b, mode, forma: FormaTypes) -> FormaReport`; `min_forma(gd, b, mode)` is
  `min_forma_with(…, FormaTypes::all())` (the T-35 default). Reason: T-35 includes the allowed Forma types (00 §6.1
  T-35; 09 §6.1 table; `Toggles.forma_types`, `CapacityRule::effective`), which change N_forma* of E-9.19, and the
  T02b signature has no place for them. **Request** T02b-1. `check` reads mode and Forma types from the scenario.
- Dependencies: `smallvec`, `thiserror`; dev `serde`, `serde_json` (all in `[workspace.dependencies]`, no new
  workspace dependency). T04 adds the codec dependencies of overview.md §3.1. Note for T04: `encode_bundle` also runs
  outside `build.rs` (`wfd-datagen`, the in-app refresh), where overview.md §3.1 lists `ruzstd` but not `zstd`
  (a build-dependency only), while pipeline.md §8.2 asks for zstd level 19.

### `wfd-buildcode` (work-breakdown.md T02b, T10; server-api.md §7)

- `CodeError` (`thiserror`, `Clone`, `PartialEq`, `#[non_exhaustive]` like `DataError`, so T10 can add a kind
  without breaking matchers in `wfd-server`): `Unrecognized`, `Corrupt { stage: DecodeStage, detail }`,
  `VersionUnsupported { v, max }`, `Invalid(Vec<Message>)` (document issues, e.g. an export of a document that does
  not resolve), `ExportUnsupported { feature }`. `CodeError::messages()` gives `import.unrecognized`,
  `import.corrupt [stage]`, `import.version_unsupported [v, max]`, the `doc.*` issues unchanged,
  `import.export_unsupported [feature]`. `DecodeStage { Base64, Deflate, Lz, Json }` (`#[non_exhaustive]`) displays
  as the `stage` values of `import.corrupt`; a new stage also adds its value to that key's documentation. Problems
  that do not stop a decode go into the report, never into `CodeError` (09 §2.4).
- `CODE_PREFIX = "wfd1-"` (AD-18); `CodeSource`, `DecodeReport` and `ImportReport` re-exported from `wfd_model::api`.
- `import_inspiration(url_or_param, data) -> Result<(CalcInput, CodeSource, ImportReport), CodeError>`: the
  `CodeSource` (`InspirationLz` or `InspirationBase64`) is `DecodeResponse.source` (server-api.md §7;
  `api/buildcode.rs`), which work-breakdown.md T02b's `(CalcInput, ImportReport)` leaves out. Only the importer knows
  it for a bare `p` parameter, which carries no `n=1`; the tuple order is `DecodeResponse`'s. **Request** T02b-4.
  Affects T10, T34.
- `decode` reads our code and our URL forms (bare code, `/?b=…`, a full localhost URL); an inspiration link is
  `Unrecognized` there (it has no game data to map names; the server tries `import_inspiration` next).
- Dependencies: `serde_json` (the `migrate` signature), `thiserror`. T10 adds miniz_oxide, base64, lz-str.

### Requests to the manager (work-breakdown.md §2.4)

Unnumbered, because T02a-T02d run in parallel and would each take C-31; the manager assigns the numbers.

| # | Conflict | Sources | Provisional choice |
|---|---|---|---|
| T02b-1 | `min_forma` cannot honour the T-35 allowed Forma types | 00 §6.1 T-35; 09 §6.1, E-9.19; work-breakdown.md T02b `min_forma(gd, b, mode)` | additional `min_forma_with(…, forma: FormaTypes)`; `min_forma` = every type. Alternative (breaking): the parameter on `min_forma` itself |
| T02b-2 (merge-blocking) | Where `Pick` lives | engine.md §1 and work-breakdown.md T02a (engine type); work-breakdown.md T02b `check_slot(…, &Pick)`; overview.md §3.1 (`wfd-data` and `wfd-engine` are siblings over `wfd-model`); optimizer.md §2 steps 2, 5 | `wfd_model::Pick`; T02a's `wfd_engine::Pick` is replaced at merge by `pub use wfd_model::Pick;` and engine.md §1 updated. Rule before either branch merges. Alternative: `check_slot` takes a `wfd-data` type and the optimizer converts each candidate |
| T02b-3 | No channel for resolve warnings | domain-model.md §9.2; server-api.md §6.1; `message/doc.rs` (keys "dropped with this warning") vs `resolve -> Result<Resolved, Vec<Message>>` | `resolve -> Result<(Resolved, Vec<Message>), Vec<Message>>`, model untouched. Alternative (breaks struct literals of a frozen T01a type): a field `Resolved.warnings` |
| T02b-4 | Import does not say which link encoding it read | server-api.md §7 `DecodeResponse.source`; `api/buildcode.rs` `CodeSource`; work-breakdown.md T02b `import_inspiration -> (CalcInput, ImportReport)` | `import_inspiration -> (CalcInput, CodeSource, ImportReport)`. Alternative: the plan's signature, and the server reads `n=1` itself (cannot classify a bare `p` parameter) |

## 2026-11-27 · T02c · `wfd-datagen` skeleton (initial contract)

T02c creates `crates/wfd-datagen/**`: the library API of pipeline.md §2, the CLI of pipeline.md §2 with every
subcommand, and the module signatures of work-breakdown.md T02c. Bodies are `todo!("Txx: …")` naming the implementing
task; tests that need them are `#[ignore = "Txx"]` (four Lua sandbox tests in `lua/mod.rs`, an executable spec for
T05). The items below are where the code differs from or adds to `docs/data/pipeline.md` and work-breakdown.md.
Requests to the manager carry task-scoped labels (`T02c-R1` …) until the manager numbers them: parallel wave-0 tasks
use the next free `C-` numbers too.

### Library API (pipeline.md §2)

- **Frozen files** (no later owner in work-breakdown.md §8, request T02c-R2): `src/lib.rs` (module list, re-exports,
  the entry points), `src/config.rs` (`PipelineConfig`, `InputTree`, `InputFile`, `SpecSource`, `StableKeySource`,
  `LockMode`, `SourceKind`, path constants), `src/error.rs` (`DatagenError`), `src/progress.rs` (`Progress`,
  `PipelineStep`), `src/cli.rs` (`CommonArgs`). The entry points check cancellation and the configuration, then
  delegate, so implementing tasks never edit them: `fetch()` → `fetch::fetch_from_lock` for `LockMode::FromLock`, else
  `fetch::fetch_sources` (T05); `build()` → `emit::run_build` (T18); `diff()` → `report::diff_bundles` (T18).
- **Network**: only `fetch()` touches the network (pipeline.md §2 "the only stage that touches the network").
  `build()` reads the source cache only (`LockMode::Pinned`/`FromLock`: the lock file's inputs; `Latest`: the newest the
  cache holds) and fails with `DatagenError::Offline` when an input is missing. A caller that wants fresh inputs calls
  `fetch()` first with the same configuration: the CLI's `build` without `--offline` (with `LockMode::Pinned`), the
  in-app refresh ("runs `fetch` + `build`", §9.2). So nothing is fetched twice.
- `PipelineConfig` has the five fields of pipeline.md §2 plus six additions. Three are fetch-only and ignored by
  `build()`: `lock_file: Option<PathBuf>` (`data/sources.lock.toml`; `None` in the in-app refresh, which has no lock),
  `update_lock: bool` (`fetch --update-lock`), `only: Option<SourceKind>` (`fetch --only`); `offline` is fetch-only too.
  Three are inputs `build()` needs that §2 does not list (pipeline.md §5.2, §5.3, §7, §9.2):
  - `spec_tables: InputFile`: `data/spec_tables.toml`, the table list `ingest_spec_tables(spec, cfg)` needs in both
    `SpecSource` modes (`SpecTablesConfig::read(&InputFile)`). `InputFile` is `Path(PathBuf)` or
    `Embedded { name: &'static str, text: &'static str }` (`include_str!`); methods `name()`, `read()`.
  - `validation: Option<InputTree>`: `data/validation/` (`base.toml`, `coverage/<task>.toml`). `None` = the checks of
    §7 run without allow-lists and coverage minimums (the in-app refresh unless the server embeds the tree,
    T02c-R6).
  - `stable_keys: StableKeySource`: `File(PathBuf)` (developer: `data/generated/stable_keys.json`, append-only global
    keys, the extended table emitted as `stable_keys.json`; T18 defines the format and whether a missing file is an
    error) or `Embedded(Vec<(StableKey, UniqueName)>)` (in-app: the embedded bundle's `Bundle.stable_keys`, never
    extended; new uniqueNames get session-local keys ≥ `LOCAL_KEY_BASE` in uniqueName order).

  No repository root is stored: every input is its own field, so `--curated DIR` and the in-app refresh need none.
  The struct is `#[non_exhaustive]` with public fields, built by `PipelineConfig::developer(root)` (CLI: every path
  under `root` from the constants `CACHE_DIR`, `CURATED_DIR`, `LOCK_FILE`, `SPEC_TABLES_FILE`, `VALIDATION_DIR`,
  `STABLE_KEYS_FILE`; `GENERATED_DIR` for `build --out`) or `PipelineConfig::in_app(cache_dir, curated, ingested_spec,
  spec_tables, stable_keys)` (server; `validation` `None`), then adjusted, so a field added later is not a breaking
  change for the server.
- **Contradictory settings** are `DatagenError::Config { detail }` before anything runs: `check_fetch()` (called by
  `fetch()`) rejects `update_lock` without `LockMode::Latest`, with `offline` or without a lock file, and
  `FromLock` with `offline`; `check_build()` (called by `build()`, and by `check_fetch()`) rejects `Pinned`/`FromLock`
  without a lock file. The CLI rejects `fetch --offline --update-lock` (and `--from-lock` with either) as a usage
  error.
- `LockMode`: `Pinned` = the lock's inputs (`build`); `Latest` = the newest (`fetch`, `fetch --update-lock`, in-app
  refresh); `FromLock` = `fetch --from-lock` (a build in this mode reads the same inputs as `Pinned`). Plain `fetch`
  (pipeline.md §2 "refresh the cache") is provisionally `Latest` without writing the lock (`fetch::cli::FetchArgs::
  config`, T05's file).
- `SourceKind { Wiki, Pe, Wfcd }` (readable `wiki`, `pe`, `wfcd`; also the clap value of `--only`).
- `SpecSource::Docs(PathBuf)` holds the repository root: the `file` entries of `data/spec_tables.toml` are relative to
  it.
- `InputTree` methods, with the same results for both variants (tests on `tests/fixtures/{input_tree,curated_binary,
  curated_bad_toml}/`): `files()` (raw listing: every file as relative path and UTF-8 text, sorted, hidden files
  included; a non-UTF-8 file is `Parse`, the first in path order); `read(rel)` (`None` for a missing file or a
  directory; a path that leaves the tree, absolute or with `..`, is `Io` with kind `InvalidInput`; on disk a lookup
  error other than not-found is `Io`); `load_curated()` (`load_curated_dir` for a directory; for an embedded tree the
  same walk: hidden directories and the top-level `UNLOADED_DIRS` not entered, non-UTF-8 files passed by name only, so
  binary files where the loader never reads load in both variants; a non-UTF-8 file the loader parses fails both, with
  "cannot read" on disk and a schema error embedded). An `Embedded` tree may be an `include_dir!` root or a
  sub-directory of one (`DATA.get_dir("curated")`): paths are relative to the given directory.
- `PipelineStep` is a type alias of `wfd_model::api::DataStep` (the steps of pipeline.md §2), so progress maps to
  `DataEvent::Step` unchanged. `Progress` exactly as pipeline.md; added `NoProgress` and `check_cancel(&AtomicBool)`.
- `DatagenError` (thiserror, `#[non_exhaustive]`): `Cancelled`, `Config { detail }`, `Offline { what }`,
  `HostNotAllowed { host }`, `Fetch { host, url, status }`, `LockInputGone { kind, file, locked }`, `Lock { path, detail
  }`, `Lua { title, detail }`, `Parse { file, detail }`, `Curated(CuratedErrors)`, `SpecTable { id, detail }`,
  `Stale { what, detail }` (`--check` modes), `Validation { report: Box<Report> }` (validation errors fail a build; the
  report travels with them), `Bundle { detail }`, `Io { path, source }`. `step()` names its pipeline step;
  `to_message()` gives the UI message (`data.host_not_allowed`, `data.fetch_failed`, `data.lock_input_gone` with their
  declared arguments, otherwise `data.refresh_failed [step, detail]`), severity error. No new message key.
- `BuildOutput { bundle, json, report }` as pipeline.md. `GeneratedJson { files: BTreeMap<PathBuf, String> }` (path
  under the output directory → canonical text).
- `DataDiff { added: Vec<DiffRecord>, removed: Vec<DiffRecord>, changed: Vec<FieldChange> }` (`#[non_exhaustive]`),
  `DiffSection` (the bundle vectors; `#[non_exhaustive]`), `FieldChange { section, id, name, path, old, new }` whose
  `Display` is the line of pipeline.md §2 (`Furis/normal_attack cc 0.05 → 0.07`), `DataDiff::summary(max_samples) ->
  DataDiffSummary` (the server-api.md §9 DTO of `DataReport.diff`).
- `Report` (`#[non_exhaustive]`; shared by T05, T18, T19; serializable as `report.json`): `errors`, `warnings`
  (validation, pipeline.md §7), `conflicts` (§5.1), `corrections` (stance normalisation lines, §5.3), `unmatched:
  Vec<Unmatched { source, id, reason: UnmatchedReason }>` (reason classes of §5.1 plus `unexplained`;
  `#[non_exhaustive]`), `unknown_fields` (wire type → field → count, §4.1), `coverage`, `tiers` (kind → `TierCounts {
  stat_line, spec_table, curated, none }`, §6.2 "the coverage report counts each tier"), `targets: Vec<TargetProgress {
  target, modelled, total, missing }>` (§7 coverage targets, reported as progress), `new_items`, `stale`,
  `auto_patched` (uniqueNames; the server maps them to `StableKey`s for `DataReport`). Helpers `error`, `warning` (set
  the severity), `has_errors`, `count_tier`, `count_unknown_field`. `tiers` and `targets` are filled by T19 (fields in
  T18's file, added now so T19 does not edit it); they default when absent from an older `report.json`.
- `FetchReport { inputs: Vec<SourceStamp>, downloaded, unchanged, bytes, build_label, wfcd_tag, lock_written }`
  (`#[non_exhaustive]`; `inputs` become `Manifest.sources`).
- Addition for the server: `fetch::check_updates(cfg, active: &Manifest, cancel) -> Result<DataCheck, DatagenError>`
  ("Check for updates", `POST /api/data/check`, pipeline.md §9.2). No task's Provides names it (request T02c-R3).

### Module signatures (work-breakdown.md T02c)

- Exactly the eight signatures of the task, re-exported at the crate root: `fetch_from_lock` (`fetch/mod.rs`),
  `eval_wiki_modules` (`lua/mod.rs`), `join` (`join/mod.rs`), `normalise_stances` (`derive/stances.rs`), `heavy_def`
  (`derive/heavy.rs`), `ingest_spec_tables` (`spec_tables/mod.rs`), `attach_effects` (`effects/mod.rs`), `validate`
  (`validate/mod.rs`). `ComboTable`, `HeavyDef` and `MeleeRules` are the `wfd-model` types.
- Types with fields from the documents: `WikiSource { title, revid: u32, text }` (`revid` as in
  `LicenceNotice.modules`), `LuaLimits { hook_every, max_instructions, max_memory }` (default the §4.1 limits: 10^6,
  10^9, 256 MiB), `SpecTablesConfig { tables: Vec<SpecTableDef { id, file, heading, columns: Option<…> }> }` (the §5.2
  TOML; `parse`, `read(&InputFile)`, `load`; unique ids).
- **Created empty** (`pub struct X {}` with `Debug, Clone, Default, PartialEq`): `WikiRecords`, `PeRecords`,
  `WfcdRecords` (T05), `Joined`, `JoinedWeapon`, `SpecTables` (T18), `EffectInputs` (T19), `ValidationConfig` (T18 and
  T19). Their names and module paths are the contract; their fields are the owning task's to define (its own files)
  and need no entry here. Renaming or moving them does.
- Constants from the documents, each tied to them by a test that reads `docs/data/pipeline.md`: `fetch::USER_AGENT`
  (the §3.1 "User-Agent" row, `<version>` = crate version), `fetch::ALLOWED_HOSTS` with `is_allowed_host` (the §3.1
  "Hosts allowed" row before "nothing else"; the hosts named after it are refused), `LuaLimits::default()` (the §4.1
  limits bullet), `wiki::WIKI_MODULES` (the 29 modules of §11), `pe::{PE_BASE_URL, PE_INDEX, PE_MANIFESTS}`,
  `wfcd::{WFCD_RAW_BASE, WFCD_REGISTRY_URL, WFCD_FILES}` (§3.2, §11). They live in T05's files; T05 may change them
  with the documents.
- The ignored Lua tests (T05's executable spec): `math.huge`/`-math.huge` become the JSON strings `"Infinity"`/
  `"-Infinity"` in the `serde_json::Value` (it cannot hold ±∞; the wire structs read them through
  `serde_util::ext_f64`); the sandbox has no `io`, `os`, `package`, `debug`, `load`, `loadstring`, `loadfile`, `dofile`
  (checked with `type()`; `loadstring` and `loadfile` come with mlua's Lua 5.1 base library, request T02c-R7);
  `require`/`mw.loadData` of an unknown title fail with the module's title; the instruction limit and the memory
  limit each trip on their own (`while true do end`; `('x'):rep(2^22)` under 1 MiB) with the module's title and the
  limit named in the detail.

### CLI (pipeline.md §2)

- Every subcommand: `fetch`, `build`, `check`, `diff`, `report`, `spec-tables`, `curate {todo, scaffold, rebind,
  render}`; a unit test parses every synopsis line of pipeline.md §2 (all options given; `--offline` and
  `--update-lock` of the `fetch` line one at a time) and checks that the CLI has no other subcommand; `tests/cli.rs`
  runs the binary (`--help` lists them the way `cargo xtask ci` detects them; usage errors exit 2).
- **Where the handlers live** (work-breakdown.md gives `main.rs` to T18 but the `fetch` CLI to T05 and the curator
  tools to T19): `fetch::cli::{FetchArgs, run}` (T05), `curated::cli::{CurateCommand, TodoKind, run}` (T19); `main.rs`
  (T18) defines `build`, `check`, `diff`, `report`, `spec-tables` and dispatches. Handler form: `fn run(common:
  &CommonArgs, args: &X) -> Result<(), DatagenError>`; `main` prints the error chain and exits 1; clap usage errors
  exit 2.
- Global options (accepted before or after the subcommand; not in pipeline.md §2): `--root ROOT` (default `.`; every
  default path is relative to it), `--cache-dir DIR` (named in pipeline.md §3.3), `--curated DIR` (named in §9.2).
  `build --out` defaults to `ROOT/data/generated`; `build` without `--offline` runs `fetch()` of the locked inputs
  first. `fetch --from-lock` conflicts with `--update-lock` and `--offline`; `fetch --update-lock` conflicts with
  `--offline`.
- `curate todo --kind` takes a `TodoKind`: the directory names of pipeline.md §6.1 (`weapon_mod`, `weapon_arcane`,
  `frame_mod`, `aura`, `frame_arcane`, `shard`, `stance`, `set`, `incarnon`, `buff`, `custom`), with the item-kind
  names of `ItemKindName` accepted as aliases (`incarnon_perk` → `incarnon`, `custom_stat` → `custom`; the others are
  equal). So set bonuses and Incarnon files can be listed. `TodoKind::{curated_kind, dir, of_item_kind}` map between
  the names. `--limit` defaults to 50.

### Dependencies

`crates/wfd-datagen/Cargo.toml` declares now every crate overview.md §3.1 lists for `wfd-datagen` (mlua, ureq,
lzma-rs, toml, serde_json, regex, sha2, clap, include_dir), plus `serde`, `thiserror` (the error type), `wfd-model`
with feature `curated` (T01a handover above), `wfd-data` (overview.md §3.1 "writes bundles via wfd-data"; see
T02c-R5); dev-dependencies `insta` and `proptest` (testing.md §4 snapshots and the stable-key property test). All are
existing workspace dependencies: no new workspace dependency. `wfd-datagen` has no features (the server's feature
`refresh` enables the whole crate). `Cargo.lock` gains the packages these pull in (mlua's vendored Lua, ureq with
rustls and ring, insta, proptest, …).

### Requests to the manager (work-breakdown.md §2.4; task-scoped labels until the manager numbers them)

| # | Finding | Provisional choice |
|---|---|---|
| T02c-R1 | pipeline.md §5.2's `data/spec_tables.toml` example pins 11 columns for `co_catalogue`; the 04 §2.14 table has 12 (`Data attack` after `Attack`). The spec wins (§2.4) | no code depends on it (the T02c test checks only that the example pins a header and that every heading exists); T18 pins the 12 columns of the spec; pipeline.md §5.2 needs the column |
| T02c-R2 | Ownership after the skeleton: `crates/wfd-datagen/{Cargo.toml, src/lib.rs, src/config.rs, src/error.rs, src/progress.rs, src/cli.rs, tests/cli.rs, tests/fixtures/{input_tree,curated_binary,curated_bad_toml}/**}` have no owner in §8 | the five `src/` files and the three fixture trees are T02c's frozen API and its tests, changed through this log; `Cargo.toml` and `tests/cli.rs` to T18 |
| T02c-R3 | `fetch::check_updates` (`POST /api/data/check`) is in no task's Provides | T05 implements it (it is a fetch over `PoliteClient`); T34 calls it |
| T02c-R4 | `cargo xtask ci` gates 7 (`cargo run -p wfd-datagen -- spec-tables --check`) and 8 (`… -- check`, pre-step `… -- fetch --from-lock`): `wfd-datagen --help` now lists `spec-tables`, `fetch` and `check`, so these gates run and fail (a `todo!` panic, exit 101) until T05 (fetch), T18 (spec-tables, check) and T19 (validators) implement them; before T02c they were skipped. `cargo xtask ci` therefore fails for every task from now until then. Not part of the definition of done (§2.3) | needs a ruling. (a) T01c's `ci` skips a datagen gate whose owner task is not yet accepted (a table gate → owner in `ci.rs`); or (b) the unimplemented handlers return a distinguishable "not implemented (Txx)" error with its own exit status, which `ci` reports as a skip (changes T05's, T18's and T19's stub bodies and `ci.rs`). T02c leaves the `todo!` bodies the task asks for and changes nothing in `xtask/` (T01c's) |
| T02c-R5 | AD-11 fallback bootstrap: if the user declines Q-U1, `data/generated/` is gitignored and `wfd-data/build.rs` fails when it is missing (overview.md AD-11, pipeline.md §8.2). `wfd-datagen` depends on `wfd-data` (overview.md §3.1, for the bundle codec), so `cargo run -p wfd-datagen -- fetch --from-lock && … build --offline`, the command that creates the directory, cannot compile | for T04 and T18, only under the fallback: provisionally the bootstrap runs with `WFD_DATA_DIR=data/fixtures/mini` (pipeline.md §8.2: `build.rs` reads `WFD_DATA_DIR`), documented by T04 next to the AD-11 hint; cleaner alternatives: a `wfd-data` feature `embedded` (default on, off in `wfd-datagen`) that `build.rs` checks through `CARGO_FEATURE_EMBEDDED`, or the codec in a module without the embedding `build.rs` |
| T02c-R6 | The in-app refresh needs `data/spec_tables.toml` (`ingest_spec_tables(spec, cfg)` takes the table list in `SpecSource::Ingested` mode too) and ideally `data/validation/` (§7: every build runs the checks with its allow-lists and minimums); AD-13 and pipeline.md §9.2 list only `data/curated/` and `data/generated/spec/` as embedded inputs | `PipelineConfig::in_app` takes `spec_tables: InputFile` (the server embeds the file with `include_str!`) and leaves `validation` `None` (checks without allow-lists and coverage minimums), which the server (T34) sets to the embedded tree if the ruling adds `data/validation/` to AD-13; AD-13 and §9.2 then name both |
| T02c-R7 | pipeline.md §4.1 lists `io`, `os`, `package`, `debug`, `load`, `dofile`, `require` as absent; the Lua 5.1 base library that mlua always opens also has `loadstring` and `loadfile` (`loadfile` reads a file from disk, and its parse error returns part of the file's text). Checked with mlua 0.12.3: `type()` of `load`, `loadstring`, `loadfile`, `dofile` is `function` in `Lua::new_with(MATH \| STRING \| TABLE)` | the ignored T05 test requires all eight absent; §4.1's list should add `loadstring` and `loadfile` |

## 2026-11-27 · T02d · `wfd-opt` and `wfd-server` skeletons (initial contract)

T02d creates the initial `crates/wfd-opt/**` and `crates/wfd-server/**` (work-breakdown.md T02d). The items below are
where the code adds to or differs from `docs/architecture/optimizer.md` §1, §5, §8, §9 and `server-api.md` §1, §3,
§11, §12.

### `wfd-opt` (optimizer.md §1, §5.2, §8, §9)

- Frozen signatures exactly as work-breakdown.md T02d: `optimize(engine: &Engine, data: &GameData, req: &OptRequest,
  ctl: &JobControl, sink: &dyn EventSink) -> Result<OptResult, OptError>` (lib.rs, T33), `solve_main(kernel: &Kernel,
  problem: &MainProblem, batch: &BatchCtx, ctl: &JobControl) -> MainSolution` (bnb.rs, T32, re-exported at the root),
  `BatchCtx { pub theta: Score, pub budget: Option<u64> }` (batch.rs, T32). `JobControl { cancel: AtomicBool, deadline:
  Option<Instant>, node_budget: Option<u64> }` (lib.rs) and `trait EventSink: Send + Sync { fn emit(&self, e: OptEvent) }`
  (events.rs, re-exported) as optimizer.md §1. `Score` is `wfd_engine::Score`, re-exported as `wfd_opt::Score`.
  `tests/api.rs::frozen_signatures` coerces both functions to function pointers of the documented types.
- Defined here because the documents name them without fields: `OptError` (`#[non_exhaustive]`; `Invalid(Vec<Message>)`
  = document issues, 422 `api.invalid`; `Objective(Message)` = `opt.failed.objective` / `mixed_metric` /
  `ttk_group_sum`; `Infeasible(Message)` = `opt.failed.infeasible`; `Engine(Message)` = engine.md §15 as `{ key, args }`;
  `message()` gives the `OptEvent::Failed` message, `issues()` the 422 issues). Cancellation is not an error: `optimize`
  returns `Ok` with the anytime result and the caller emits `cancelled` when `ctl.is_cancelled()` (§9).
  `MainProblem` (problem.rs) and `MainSolution { keep: Vec<KeptLeaf { score, picks }>, best: Option<Score>, gap:
  Option<Score>, stats: NodeStats }` (bnb.rs): their names are frozen by `solve_main`, their fields are T32's (the
  skeleton's fields follow §5.2: `free_slots`, fixed part `ActiveVec = [i64; 32]`, `GroupMask = u128`
  incompatibility masks, monotone pool, `DrainLimit`; T32 adds the engine-side base point and active-slot map).
- Helpers with no design content beyond the documents: `JobControl::{new, request_cancel, is_cancelled,
  past_deadline, should_stop}`; `CANCEL_CHECK_NODES = 1024`, `CANCEL_CHECK_EVALS = 64` (§9);
  `events::{PROGRESS_MIN_INTERVAL = 250 ms, NullSink, CollectSink}` and `EventSink` for every `Fn(OptEvent) + Send +
  Sync`; `batch::{BATCH_SIZE = 64, LEAST_SCORE, BatchCtx::without_threshold}` (§8); `keys::CanonKey = u128` (§8);
  `bnb::NodeStats::add`. Every other body is `todo!("T32")` / `todo!("T33")`; ignored placeholder tests name the
  acceptance items of T32 (`bnb`, `composition`, `dominance`, `capacity`, `batch` unit tests) and T33 (`tests/api.rs`).
- Cache keys and the postcard ruling (manager, "Rulings on T01d"): the skeleton defines no encoding, only
  `CanonKey`; optimizer.md §8 scopes the memo cache to one `CompiledScenario`, so no key contains the scenario.
  `keys.rs` repeats the ruling for T32 in case a key ever does.
- Manifest: feature `parallel` (default) = optional `rayon` (overview.md §3.1: wfd-opt builds without default
  features); dependencies `rustc-hash`, `xxhash-rust`, `parking_lot` declared for T32/T33 (all existing workspace
  dependencies; no new workspace dependency). After T02a is merged, `parallel` should also enable
  `wfd-engine/parallel` (the feature does not exist on T02d's base).
- `#![warn(missing_docs)]` on both crates.

### `wfd-server` (server-api.md)

- `router(state: Arc<AppState>) -> axum::Router` and `serve(cfg: Config) -> anyhow::Result<()>` as work-breakdown.md
  T02d. `routes::ROUTES: &[RouteSpec { method, path, module, owner, wired_by }]` lists every row of §3 in order;
  `tests/routes.rs` parses the §3 table of server-api.md and fails when they differ. One module per route group
  (work-breakdown.md §2.2): `routes/{meta,catalog}.rs` (T11), `routes/{calc,timeline,explain,build,optimize,data,img}.rs`
  (T34), `web.rs` (the `GET /*` fallback, T11). Every route answers 501 `api.not_implemented` with `args.route =
  "METHOD path-template"`; unknown `/api` paths and non-GET non-`/api` requests answer 404 `api.not_found` with
  `what = "route"`; other GET paths answer 501 for `GET /*` until T11 embeds the frontend. JSON bodies carry
  `application/json; charset=utf-8` (§2).
- `serve` (skeleton of §1.2): binds `cfg.listen` or `127.0.0.1` on `cfg.port` (0 = OS-assigned), prints the §1.2 step 5
  line (`banner()`: `(no data loaded)` until T11 loads data) and serves until Ctrl-C. Port fallback, IPv6 loopback,
  single-instance detection, data loading, browser opening, logging, middleware are T11's.
- `AppState { config }` only: `GameData` cannot be constructed before T04, and the route tests need a state. T11 adds
  the fields of §11 (`data: ArcSwap<GameData>`, pools, `jobs`, `http`).
- `Config` holds the keys of §12.1 with their defaults plus the CLI-only `listen` and `bundle`; `cli` parses the whole
  §1.1 synopsis (clap derive; serve options with or without the word `serve`); `calc`, `optimize`, `import` and `data`
  exit 1 naming T34. `--threads N` sets `optimizer_threads` (§1.1 does not say which pool; T11 may change it).
- `tests/smoke.rs` (the file of T11's acceptance command) spawns `warframe-dps --no-browser --port 0`, reads the
  printed URL and expects 501 from `/api/meta`; T11 changes it to 200 with `app_id`.
- Dependencies: axum, tokio, serde, serde_json, clap, anyhow; dev tower, http-body-util (existing workspace
  dependencies).

### Message key (T01b's file, backward-compatible)

- `ApiKey::NotImplemented` = `api.not_implemented [route]`, appended to `message/api.rs`: work-breakdown.md T11 and the
  skeleton's routes answer 501 with it, and T01b's keys did not have it. `frontend/src/api/generated/messageKeys.ts`
  regenerated (`cargo xtask gen-ts`).

### Interim files in T02a's and T02b's crates (merge instruction)

T02a and T02b write the `wfd-engine` and `wfd-data` skeletons in parallel with T02d, and `wfd-opt`'s frozen signatures
name their types, so T02d's branch carries interim versions of `crates/wfd-engine/src/lib.rs` (declares only `Engine<'b>`,
`Kernel<'a>` and `Score` with its four public fields and `Ord`) and `crates/wfd-data/src/lib.rs` (declares only
`GameData`), each marked INTERIM in its first line (the precedent is T01b's interim `xtask/src/main.rs`). **On merge the
owner's file wins entirely**; nothing else of T02d's touches those crates. `wfd-opt` and `wfd-server` use only
`wfd_engine::{Engine, Kernel, Score}` and `wfd_data::GameData` at the crate roots, `Score`'s public fields and its
`Clone`, `Copy`, `PartialEq`, `Ord` (engine.md §7.9); T02a's in-progress `lib.rs` re-exports all three at the root
(checked in its worktree before this commit: same names, lifetimes, fields and order) and T02b's re-exports `GameData`.
Request: the manager's confirmation of these two edits outside T02d's Owns line.

### Requests to the manager (work-breakdown.md §2.4)

Numbered T02d-n because T02a-T02c run in parallel and the next free C-number (C-31) may be taken twice; the manager
assigns the numbers. Provisional choice for each: as implemented.

| # | Conflict | Provisional choice |
|---|---|---|
| T02d-1 | `Score` order direction: engine.md §7.9 comments `Ord` as "band asc, primary desc, tie desc, tie2 desc", which reads like a best-first sort order; optimizer.md §5.2 prunes `bound < θ`, keeps leaves with `s ≥ θ`, takes `max(s_best, s)`, and §7 sorts finalists by `(Score desc, …)`, and engine.md §7.9 states `bound(upper) ≥ score(y)` | the greater key is the better build (lower band, then higher primary, tie, tie2), which is also how T02a's in-progress `ev/score.rs` implements it; `tests/api.rs::score_order_used_by_the_search` and `batch_without_threshold_prunes_nothing` pin it against T02a's `Score`. Request: reword the engine.md §7.9 comment to "greater is better". Affects T02a, T15c, T32, T33 |
| T02d-2 | Ownership overlap: work-breakdown.md T11 wires `GET /api/data/status` and `GET /api/img/{key}`, but §8 gives `routes/data.rs` and `routes/img.rs` to T34 | T11 writes those two handlers in T34's files before T34 starts (T34 depends on T11, so the files are never edited concurrently); T34 owns the files afterwards and adds the image disk cache. `RouteSpec.wired_by` records it |
| T02d-3 | Status 501 `api.not_implemented` is used by work-breakdown.md T11 but missing from server-api.md §2 "Status codes" | add "501 route not wired yet (`api.not_implemented`, skeleton phase only)" to §2 (T41 or the manager) |

## 2026-11-27 · T02d · Review fixes (before acceptance)

Changes made after the independent review of T02d. Where an item changes a shape or a merge step described in the
T02d entry above, this entry wins.

### `wfd-opt`

- `JobControl.cancel` is the engine's `CancelToken` (`wfd_engine::CancelToken`, re-exported as `wfd_opt::CancelToken`),
  not an `AtomicBool`. engine.md §1 `Engine::mc(.., cancel: &CancelToken)` takes `CancelToken(Arc<AtomicBool>)`, which
  cannot be built around an existing flag, so with a bare `AtomicBool` the finalists' Monte Carlo (optimizer.md §7
  step 3) could not see the job's cancel, against §9 (≤ 100 ms, "cancel never loses the best build") and T33's
  acceptance test, unless T33 added a watcher thread or cut MC into chunks. `request_cancel` calls `cancel.cancel()`;
  `is_cancelled` and `should_stop` are unchanged; the ordering is the token's `Relaxed` (a stop flag publishes no data).
  The server may keep a clone of `ctl.cancel` instead of the `Arc<JobControl>`. Request T02d-4. Test
  `job_cancel_is_the_engine_cancel_token`.
- Job id of `started`: `optimize` has no job id (its frozen signature, `JobControl` and `OptRequest` carry none), so it
  emits `OptEvent::Started { job: JOB_UNSET, plan }` with `events::JOB_UNSET = ""`, and the job's sink fills the id in
  with the new `events::stamp_job(event, job) -> OptEvent` (only `Started` changes) before storing or sending the event
  (server: its job id, server-api.md §8.2). The `optimize` doc comment says so. Request T02d-5. Affects T33 (emits
  `Started` with `JOB_UNSET`), T34 (its sink stamps). Test `sinks_stamp_the_job_id_on_started`.
- `batch::LEAST_SCORE` (band 255) is removed: `BatchCtx::without_threshold` uses `Score::WORST` (T02a: band 2, every
  float −∞, the initial incumbent), so the merged code has one "worst" sentinel. It is at most every key the kernel
  returns (bands 0-2, no NaN), so as θ it prunes nothing (§5.2 prunes `bound < θ`, keeps `s ≥ θ`).
- `Debug` is derived on `BatchCtx`, `MainSolution` and `KeptLeaf` (test `search_types_are_debug_and_comparable`).
- `crates/wfd-opt/Cargo.toml`: the `parallel` forwarding is a merge step (below); its owner is request T02d-6.

### `wfd-server`

- Method mismatches: `router` adds `method_not_allowed_fallback(web::method_not_allowed)`, so a path of §3 requested
  with a method its row does not list (`GET /api/calc`, `PUT /api/meta`) answers 404 `api.not_found` with
  `what = "route"` as JSON (server-api.md §2: every non-2xx response has `{ "error": Message }`) instead of axum's
  empty 405; axum still sets the `Allow` header. 404 and not 405 because §2's status list has no 405 and
  `api.not_found` with `what = "route"` already means "no such route". The `id` of every route 404 is now
  `"METHOD path"` (was the bare path), the form of `api.not_implemented`'s `route`. `method_not_allowed_fallback` covers
  only the routes registered before it, so T11 and T34 add routes inside `routes::api()`. Tests
  `method_mismatches_are_not_found_as_json` (5 rows), `unknown_api_paths_are_not_found`.
- `serve` prints the warning of §1.2 step 4 to stderr when `--listen` names a non-loopback address (`listen_warning`;
  `127.0.0.1` and `::1` print none).
- `--threads` accepts 1 and up (clap range parser); `--threads 0` is a usage error instead of being raised to 1.

### Message key `api.not_implemented`: English catalogue entry (T12's file)

overview.md I-8 and T12's `frontend/src/i18n/messages.test.ts` ("has an English message for every server key, with
the declared arguments") need an English message for every key of `frontend/src/api/generated/messageKeys.ts`. T12
owns `frontend/src/i18n/en/server.json` (`en.json` and `messages.ts` are generated from it by `npm run gen`), and T12's
branch has no entry for the key T02d adds. Request T02d-7: T12, or the manager in the merge commit, adds after
`"api.bad_origin"`:

```json
  "api.not_implemented": "Not available yet: {route}.",
```

with exactly the argument `route`. The key (T02d's `message/api.rs` and `messageKeys.ts`) and this line must land in the
same merge, or the frontend tests fail for whichever of T02d and T12 is merged second.

### Interim files and merge steps (replace the list of the T02d entry)

The interim `crates/wfd-engine/src/lib.rs` now also declares `CancelToken` (`new`, `cancel`, `is_cancelled`; `Debug`,
`Clone`, `Default`), `Score::WORST` and `Debug` on `Score`, with the shapes of T02a's commit e9344cf (`cancel.rs`,
`ev/score.rs`, both re-exported at its root). Merge steps for the manager:

1. `crates/wfd-engine/src/lib.rs` and `crates/wfd-data/src/lib.rs`: take T02a's and T02b's files entirely.
2. `crates/wfd-opt/Cargo.toml`, in the merge that brings T02a and T02d together: `parallel = ["dep:rayon"]` becomes
   `parallel = ["dep:rayon", "wfd-engine/parallel"]`. The workspace declares `wfd-engine` with
   `default-features = false`, so without the line `wfd-opt` with default features builds the engine without rayon;
   `wfd-server` gets the engine's `parallel` through `wfd-opt`'s default features (T11 may declare it directly).
3. `frontend/src/i18n/en/server.json`: the line of T02d-7, unless T12 already has it.

Checked on a scratch tree (this commit's `wfd-opt` and `wfd-server` + T02a e9344cf + T02b 21775d2, steps 1-2):
`cargo test -p wfd-opt -p wfd-server` and `cargo clippy -p wfd-opt -p wfd-server --all-targets -- -D warnings` pass,
and `cargo tree -e features -i rayon` shows `wfd-engine` pulled in by `wfd-opt`'s `parallel`. The manager's
confirmation of the edits outside T02d's Owns line (the two interim files, `message/api.rs`, `messageKeys.ts`) is still
requested.

### Requests to the manager (work-breakdown.md §2.4), continued

Provisional choice for each: as implemented.

| # | Conflict | Provisional choice |
|---|---|---|
| T02d-4 | optimizer.md §1 `JobControl { pub cancel: AtomicBool, … }` vs engine.md §1 `Engine::mc(.., cancel: &CancelToken)` with `CancelToken(Arc<AtomicBool>)`, optimizer.md §7 step 3 (finalists' MC) and §9 (≤ 100 ms cancel, cancel never loses the best build) | `pub cancel: CancelToken`; reword optimizer.md §1 to `pub cancel: CancelToken /* engine.md §1, shared with Engine::mc */`. Affects T33, T34 |
| T02d-5 | optimizer.md §9 `Started { job: JobId, plan }`, but no input of `optimize` (§1) carries the job id, and no document says who fills it | `optimize` emits `job = ""` (`events::JOB_UNSET`) and the job's sink stamps the id (`events::stamp_job`); add that sentence to optimizer.md §9. Affects T33, T34 |
| T02d-6 | work-breakdown.md §8 gives `crates/wfd-opt/Cargo.toml` to no task after T02d, but T32's `benches/` need `[[bench]]` entries and T33's proptests a dev-dependency | T32 owns `crates/wfd-opt/Cargo.toml` (add it to its §8 row); T33 changes it through a CONTRACTS.md entry (T33 depends on T32). The `wfd-engine/parallel` line is merge step 2 above, done by the manager |
| T02d-7 | `api.not_implemented` (T02d) has no English entry in T12's `frontend/src/i18n/en/server.json` (I-8) | T12 or the manager adds `"api.not_implemented": "Not available yet: {route}."` in the same merge as the key (above). Affects T12 |

## 2026-11-27 · T03b · Rule tables (real data)

T03b adds the eleven rule tables of pipeline.md §6.1 it owns (`data/curated/rules/{mult_groups,slot_layout,
riven_traits,riven_formula,body_parts,special_dr,toggles,factions,categories,helminth,frames}.toml`) and their
acceptance test `crates/wfd-model/tests/rules_tables.rs` (auto-discovered; the crate's dev-dependency enables
`curated`, so no manifest change). Every file has file-level `sources`; every row cites its source (row `sources`,
`source`, or a comment where the type has no field).

### Interface change (backward compatible)

- `rules::ToggleDescriptor.default` gains `#[serde(default)]`: absent in a readable form = JSON `null`. Reason: TOML
  has no null, and 17 of the 43 toggles are off (null) by default. The compact (postcard) form is unchanged; TypeScript
  is unchanged (`gen-ts --check` passes); `data/curated/schema.json` regenerated with `cargo xtask gen-schema`
  (`default` no longer required). Doc comments of `default` and `field` state the conventions below. Affects T12, T30,
  T31 (readers of `ScenarioSchema.toggles`), T18 (emits `Rules.toggles`).

### Data conventions (for the consumers; no type change)

- Toggles (`toggles.toml`): `field` is the toggle's `TOGGLE_MAP` place: the ScenarioDoc path(s) as written there,
  `"buffs"` for T-13 … T-19, BuildDoc paths prefixed `build.`, optimizer-request paths prefixed `opt.`. `default` is the
  JSON the default document holds at that path (`ScenarioDoc::against(..)`, `Toggles::default()`), an object keyed by
  path for multi-path toggles (a missing key = null); exceptions T-23 (null: the enemy is the user's pick) and T-37
  (`"ally"`, the meaning of an absent caster on Roar). The test checks every default against the model. i18n keys
  follow frontend.md §8: `toggle.<id>.label`, `toggle.<id>.help`, `toggle.<id>.opt.<value>` (the mini fixture used
  `toggle.t01.label` and `opt.<value>`; T03 may align it). The `en.json` entries are T12's / T30's / T31's.
- Special DR (`special_dr.toml`): an enemy listed in several `[[enemy]]` rows gets the union of their `flags` and
  `immune`; any other field is set by at most one row (tested), so row order never matters. Rows of enemies without a
  module record (Kuva Liches, Sisters of Parvos, Sister Hounds, Technocyte Coda, Kuva Lich Thralls, Skittergirl) have
  `enemies = []` (data gap, 05 §4.1, OQ-526): T18 attaches them once base records exist. 05 §4.4 "ALL" is written as
  the full `StatusSet`. `[[dr]] orphix`: `dps_dr` points (X_dps, DR); between consecutive points X·(1 − DR) is linear
  in X and the last segment continues beyond the last point, which is exactly E-5.22 (tested at every band). T07
  interprets `dps_dr` this way. `[[dr]] attenuation_unknown` has `p_att` absent (flag only, 05 §15.1). Adaptation
  `params` keys: `g1` … `g4`, `extra_res`, `hit_cap_band{1,2,3}`, `band{1,2,3}_floor` (E-5.23).
- Not rows (rules or other owners): the `boss` caps COL 4 / BLA 4 and the `og_present` caps (state rules; T07/T18),
  Entropic and Guardian Eximus DR (Eximus kinds of T-23, not records; T07), part records of the 05 §4.3 hand rows,
  the Juggernaut per-part status immunity and the Scaldra `part_dr` (`overrides/enemies/`, T18), `eximus_capable` and
  `eximus_kinds` from `W:Eximus/Compatibilities` (T18).
- Multiplier groups (`mult_groups.toml`): `target_vuln` groups have `dot = "none"` (V is applied to every tick at tick
  time and "never enters ... the DoT seed", 05 §13); the overguard-only `fortifier` is `none` (M_OG,p travels with the
  proc, 03 §9.4); Sonar and Detect Vulnerability are `spot` with `dot = "inherit"` (D-095). The mini fixture's
  `molecular_prime` and `custom_vuln` groups say `inherit` (T03). Engine-computed factors (combo, sniper combo, beam
  ramp, Peacemaker ramp, falloff) are not groups.
- Slot layouts (`slot_layout.toml`): one layout per `EquipmentClass`; `element_order` 0-7 / 8 / 9 per CONTRACTS.md
  T01d review fix 2 (every `arcane<n>` and `inc<t>` slot is 9); `exalted_ranged.arcane0` has no `arcane_family` (the
  family is the weapon's slot class, 09 E-9.30); the warframe layout has `aura1`, usable only with `aura_slots = 2`
  (`frames.toml`: Jade).
- Rivens (`riven_traits.toml`): "Damage / Melee Damage" and "Fire Rate / Attack Speed" are two traits each (sign rule
  and bucket differ; same inspiration key `base` / `speed`). Ammo Efficiency and Chance to Gain Combo Count have b < 0;
  the typed value's conversion to the bucket value (04 §8.2) belongs to the riven input converter (T04/T13), noted on
  their templates. `pe_tag` values are the Public Export tags (`WeaponWeakpointCriticalChance`; domain-model.md §16.6
  shows `WeaponWeakPointCritChanceMod`, a stale example for T41).
- Helminth (`helminth.toml`): host buffs are `wfd:buff/<slug>` (`vex_armor`, `resupply`, `eclipse`, `amp`,
  `silken_stride`, `roar`, `rippers_wail`, `demonium`, `xatas_whisper`); the buff curation task uses these slugs or
  updates the table. Frames (`frames.toml`): Jade `aura_slots = 2`; 53 Prime/Umbra `variant_of` rows from WFCD.

### Requests to the manager (spec conflicts, work-breakdown.md §2.4; provisional choice applied)

| Id | Conflict | Provisional choice |
|---|---|---|
| T03b-R1 | 04 §5.7 row "Custom `dmgVulnerability` (T-26)" gives layer mask `HP`; 05 §13 (stage owner) says the custom stat is "multiplicative on every layer and type". | 05 §13 (and CONTRACTS.md C-14): `custom_vuln` on OG, SH, HP. 04 §5.7 should be aligned. |
| T03b-R2 | Kitguns, zaws and archguns have two weapon arcane slots; 01 E-1.8 gives every arcane slot position 9, sub-index 0, so two arcanes with combining elements would tie. | Both slots `element_order = 9`; no known second-slot arcane carries a combining element (UNVERIFIED). |
| T03b-R3 | 05 §4.5 last row lists "Protea" and "Coda Duets" among the U40 attenuation-unknown enemies; no module record resolves "Protea" unambiguously (Commandeered Protea Prime, Protea Specter) and "Coda Duets" has none. | Left out of the `attenuation_unknown` row (named in its comment). |
| T03b-R4 | 05 §4.4 Viral list names "Demolisher Leaping Thrasher"; the module record is `infestation:Demolisher Thrasher`. | Mapped to `infestation:Demolisher Thrasher` (same unit; W:Damage/Viral_Damage names "Demolisher Boiler/Thrasher"). |

### T03b review fixes (2026-11-27)

Interface changes, all backward compatible (new optional field, new enum variant, new flag bit, new function, doc
comments); `gen-ts` and `gen-schema` regenerated (`ControlType.ts`, `EnemyDetail.ts`, `ToggleSchema.ts`,
`data/curated/schema.json`). No existing readable or compact value changes meaning.

| Change | Reason | Affects |
|---|---|---|
| `rules::SpecialDrDef::dps_multiplier(x) -> Option<f64>` (new function) and the `dps_dr` doc: the points are read **only** through it; the effective DPS X·(1 − DR) is linear between points, the last segment continues, M = g(X)/X. | A reader that interpolates DR linearly gets M wrong inside every E-5.22 band and negative at high X (review). One reader in the model removes the ambiguity; the test checks E-5.22 through it. | T07 (calls it), T18 (emits `dps_dr`) |
| `rules::DotInheritance` / `MultGroupDef.dot` docs: `dot` is DoT-**seed** inheritance only; `TargetVuln` groups are `none` and still multiply every tick at tick time (05 §13, CD-05). | `None` had no doc and could be read as "V does not reach ticks". | T07, T13, T14 |
| `rules::ControlType::Custom` (new variant) + docs of every kind and of `ToggleDescriptor.field` / `api::schema::ToggleSchema.field`: generic kinds write a value of their shape at one path; `custom` toggles have a structured value (object, list, several paths, buff caster) and a component of their own (frontend.md §4), `min`/`max`/`step`/`options` describing the main scalar input. A `select` with a `null` default is optional (`null` = off, T-34). | Several toggles' controls did not match the value at `field` (T-05 `{mode, n?}`, T-08 `{rank?}`, T-09 `{start_combo}`, T-24 `{element, v10}`, T-26 `{}`, T-41 `[]`, T-43 `{}`, multi-path T-29/T-30/T-32/T-33/T-35/T-38, option-less selects T-22/T-23/T-25, T-37 caster), so a schema-driven panel could not write them generically. `custom` = T-05, T-08, T-09, T-22 … T-26, T-29, T-30, T-32, T-33, T-35, T-37, T-38, T-41, T-43 (tested). | T12, T30, T31 (OptionsPanel switch on `kind`), T34 (schema route) |
| `rules::Attenuation.mdps_frac: Option<f64>` (new optional field, `#[serde(default)]`). | 05 §15.2: "Documented fractions (Hemocyte 0.20, Eidolons 1.00) are stored as data for a later model"; no stage reads it. | T07 (ignores it), T18 |
| `EnemyFlags::ABILITY_DURATION_QUARTER` (bit 22, readable `ability_duration_quarter`). | 05 §13.1: the Hounds' 1/4 ability duration "is a flag only"; set on the Sister Hounds data-gap row. No stage reads it. | T07, T18, frontend flag labels |
| `curated::CategoriesFile.stance_tag_rename` (new optional map). | `[stance_tag]` mixed two relations (class → tag and the EC-09-21 tag renames), told apart only by key case. The renames moved to `[stance_tag_rename]`. | T04 (legality), T18 |

Data changes (T03b-owned files): T-20 control `percent` (frontend.md §6 row 20 detail); the toggles header states the
T-19 exception (a bool, default `false`) next to the null-default buff values, and the test checks buff defaults;
Technocyte Coda row `special_dr = "attenuation_unknown"` (05 §4.1 adversary, §4.5 last row "Adversaries"); Hemocyte
and Eidolon `mdps_frac`; Hounds flag. Tests added: per-class slot sets and arcane families (E-9.2 table, E-9.30),
weakness and scaling per faction (01 Table 4.10-A, 05 §6.1), the full 04 §8.3 coefficient table (spliced traits
included), generic-control shapes, stance-tag tables, adversary attenuation.

Requests:

| Id | Request | Provisional state |
|---|---|---|
| T03b-R5 | server-api.md §5.6 (`ScenarioSchema.toggles`) still says `field: string /* path in ScenarioDoc */` and lists five control kinds. Update it to the `ToggleDescriptor.field` conventions (CONTRACTS.md T03b) and the `custom` kind (manager; the docs are not T03b's). The widened `field` meaning was recorded by T03b as a data convention without manager approval; the reviewer asks for that approval. | Code, data and generated TS follow this entry; `api/tests/shapes.rs` lists `ControlSchema.kind` in `TYPE_DIFFERENCES` until server-api.md has `custom` (the entry then fails as no longer needed and is removed). |
| T03b-R6 (to T03) | The mini fixture gives `molecular_prime` and `custom_vuln` `dot = "inherit"`; the real table and 05 §13 give `none` for every `target_vuln` group (seed inheritance only; ticks get V at tick time either way). Align the fixture so T07/T13 tests see the real value; also its toggles: T-20 `percent` (fixture: `number`), T-09 and T-24 `custom` (fixture: `optional_number`, `number`), T-25 `custom` with the multi-path field (fixture: `select`, `build.variant.kitgun`). | Fixture unchanged (T03 owns it). |

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

## 2026-11-27 · T03 · Review fixes (fix run; no model type changed)

The fix run of T03 for the 15 review issues of `docs/project/open-items.md` §3 and the T03 items of "T01d → Review
fixes (second pass) → Left for other owners". Files: `data/fixtures/**` and `crates/wfd-model/tests/fixtures.rs` (T03's
per the 2026-11-27 ruling) and this entry. No `wfd-model` type changed.

### Format contract: extension files `data/fixtures/mini/extra/<task>.json` (issue 9; affects T04, T18 and every task that adds fixture data)

- Top-level keys: `weapons`, `items`, `enemies`, `sets`, `frames`, `combo_tables`, `stances` (record lists, appended
  after the base files and after earlier extension files, in file-name order; dense ids continue the sequence),
  `names` (added; an existing key is an error), `stable_keys` (appended; duplicates are errors), and the new
  **`rules`** and **`modular`**: partial readable JSON of `Rules` and `ModularTables`, deep-merged into the base.
- Deep merge: objects merge key by key, arrays are concatenated (base first), a key absent or `null` in the base takes
  the extension's value, every other collision (a scalar, an existing map key such as a `game_constants` entry) is an
  error. An extension adds rows and keys and never overrides one; list tables and the list fields of `melee`,
  `body_parts`, `helminth` and `modular` append, `game_constants` gains keys, scalars and the fixed-size `status`
  table cannot change. Appended rows that carry dense ids (`Faction.id`, `TagDef.id`, `CategoryDef.bit`) continue the
  base sequence. The merged result must deserialize into the model types.
- Reference implementation: `apply_extra` / `deep_merge` in `crates/wfd-model/tests/fixtures.rs` (test
  `extension_files_merge_by_the_documented_rule`); prose in `data/fixtures/README.md` "Extension files". T04's
  `wfd-data` loader (`WFD_DATA_DIR`) and T18's emitter implement the same rule, or record a different one here and
  update the test loader with it.

### Other layout rules the fixture now states (affect T04, T18)

- Every record list is sorted by uniqueName, including the five lists of `modular.json` (`ChamberGrip.grip` indexes
  the sorted `grips`); `stable_keys.json` is sorted by key (pipeline.md §8.1; test `record_files_are_sorted`).
- `IncarnonDef.tiers[t − 1]` lists every perk of evolution t in the page's order, so `tiers[t − 1][i]` is the item
  with `ItemKind::IncarnonPerk { tier: t, index: i }` and `SlotContent::Perk.index` indexes that list (the Furis
  fixture listed only its own perks, which made `inc3`/`inc4` index 1 point past the end; issue 2). The id test checks
  the invariant and the scenario test checks every slot against its layout and every perk index against its tier.
- A toggle descriptor's `field` is the `TOGGLE_MAP` path list, `build.`-prefixed for build fields (T-25:
  `build.variant.kitgun|build.variant.zaw|build.slots.content.item.params`). For T03b.

### Requests to the manager (work-breakdown.md §2.4: the spec wins; provisional choice in the fixture)

| # | Conflict | Spec | Provisional choice |
|---|---|---|---|
| C-37 | DSL gap: Melee Doughty cannot be written as 02 E-2.5a. The DSL has no resolved stat for the unquantized modded Puncture share times the weapon-side SC (`ResolvedStat::ScOf(PUN)` is SC·w_PUN of E-9, final SC incl. target-side terms), no `round1` and no cap 50 in `ResolvedCurve`, and no scope "the Puncture-bearing part of an attack" (U38.6). domain-model.md §16.13 still shows the old `cd_rel` × floor(SC_PUN / 0.10) form (stale, for T41). | 02 E-2.5a, 02 §4.3 ΣCD_abs (OQ-209); 04 §5.4 row (b_r = 0.5 + 0.1·rank); TV-02-14, TV-04-22 | fixture item `draft`: `stat = "cd_abs"`, value `{ max = 10, rank = { table = [5, 6, 7, 8, 9, 10] } }` (10·b_r), `per = [{ resolved = { stat = "sc_of.PUN", curve = { pow = { scale = 1, exp = 1 } } } }]`, note naming the gaps. A fix needs a model addition (a resolved stat or a dedicated mechanic, e.g. `MechanicKind` "doughty"); owner T01a / T19, affects T06, T13, T20-T29. |
| C-38 | Theorem Demulcent has "no default type, so an entry without one adds nothing" (04 §5.5), but a choice `ParamSpec` needs a default value; and one entry holds one Residual element while the arcane keeps separate stacks per element. | 04 §5.5 row; 01 §4.10.1 position 10; TV-01-38 | fixture item `draft`, param `element` (HEA/COL/ELE/TOX) default `TOX` (TV-01-38's case), `order = "warframe_arcane"`; several zones need several entries. A "no default" choice (or a per-element stack set) is a model question for T01a / T19. |
| C-39 | domain-model.md §17 lists two encodings with no sourced catalogue member: `DotOnly` (V_sd is the `dot_vuln` slot, Mecha/Vauban second dips are `dot = "twice"` G groups) and `scope.hit.outcome` (no user since D-410); its "Hit-outcome gate" fixture text ("the fixture item still has the gate form") is stale now that Laetum has the `on_plain_hit` form. | 03 §9.4; 04 §5.12, D-410 | the coverage test (`every_coverage_encoding_is_exercised`) names these two (and `assemble`, an engine function) as the only non-data tokens; `mini/COVERAGE.md` explains them. §17 wording: T41. |

### Data choices for the owners of the real tables

- T03b: `sonar` group `dot = "inherit"` (D-095); new group `slam` (stage G, `dot = "inherit"`; 04 §3.2 Slam Damage,
  07 §7.4 M_slam) for Nira's set (+1.50 per piece, 04 §9); `element_order = 9` on `arcane0` and `inc<t>` (done in
  the fixture layouts); fixture descriptors for T-06, T-07, T-30, T-38 (one control each, simplified).
- T03c: table 9-A Machete row (6× / 5×, breakdown 150 % Bleed + 3 × 150 %), used by the fixture's Machete heavy
  (σ_1 = 1.2, TV-07-10).
- T20-T29: Sacrificial Steel and Pressure carry only their melee part (the Sentient part is UNVERIFIED, 04 §5.13,
  OQ-415) and are `draft`; Stormburst and Headcracker are `draft` (stack timer UNVERIFIED); Executioner's Fortune is
  `unmodelled` (instant reload). Magus Accelerant is `stacks = { max = 1, unbounded = true }` (ruling C-4).

## 2026-11-27 · T03 · Second review fixes (fix run 2; no model type changed)

Files: `data/fixtures/**`, `crates/wfd-model/tests/fixtures.rs` and this entry. Where an item changes a rule of the
entry above, this entry wins.

### Format contract: extension files (amends "Deep merge" above; affects T04, T18 and every task that adds fixture data)

- Only keyed list tables and citation lists (`…sources`) append. Every other list (the fixed-size `status` table, the
  `co_default` `flags` / `off_with` lists) is an error. After the merge no two rows of a keyed list share a key, so a
  second row for an existing key is an error, not an override. Keys: `factions` `id` and `slug`; `scaling` `name`;
  `mult_groups`, `special_dr`, `riven_traits`, `toggles` `id`; `slot_layouts` `class`; `body_parts.faction_heads`
  each faction; `riven_formula.layouts` (`positives`, `negatives`); `melee.heavy_class`, `melee.finishers` `class`;
  `melee.stance_co` (`stance`, `class`); `melee.attack_triggers` (`weapons`, `trigger`, `attack`, `attack_idx`);
  `melee.heavy_kappa`, `melee.combo_specials`, `helminth.restricted` the uniqueName; `helminth.hosts` `frame`;
  `categories` `bit` and `name`; `tags` `id` and `name`; the five `modular` lists `unique_name` and `key`.
  Reference: `LIST_KEYS`, `check_list_keys`, `deep_merge` in `tests/fixtures.rs`; table in `data/fixtures/README.md`
  "Extension files" rule 3. The test now appends new keys and asserts that a duplicate of each keyed list is
  rejected. For T03b/T03c: the real tables should satisfy the same keys (the `attack_triggers` key is the fixture's
  choice; T03c records a different one here if 07 §9.7 needs it).
- New rule 5: `manifest.coverage` is derived (count per `ItemKind::slug()` per `EffectStatus`); a loader adds an
  extension's items to it. Test `manifest_coverage_counts_the_items`.

### Data

- `manifest.json`: `coverage` recomputed from the item files (126 items; it still held the counts of e3a0049),
  `spec_rev` and the spec source stamp `49c7dd3` (the 2026-11-27 spec update the records follow: D-067 amended,
  ruling C-4), `data_version` `fixture-mini.2`, `created_at` 2026-11-27. The new test checks coverage against the
  items and the spec stamp against `spec_rev`.
- Haven Foray (`wfd:perk/furis/haven_foray`) gains its second line, `incarnon_flat` +30 under
  `when = { trigger = "while_overshields" }` (class (c), 04 §5.13 "Conditional Incarnon flat perks";
  W:Furis_Incarnon_Genesis L50-55, L129), scope `form = normal`, adding to X. The Furis scenario (`all_max`) now
  includes it. For T41: the domain-model.md §16.10 example shows only the unconditional +X line.
- `rules.json`: descriptors for T-33 (`toggles.strength_mode|toggles.ability_duration|buffs.caster`, select
  `manual`/`derive`) and T-37 (`buffs.caster`, select `self`/`ally`, default `ally`), 00 §6.1. The Gorgon scenario's
  ally Roar (`wfd:buff/ally_roar`, not the T-13 buff) at Strength 2.0 is T-33; the toggle test now checks buff
  casters too. For T03b: fixture simplifications as for T-30 / T-38.

### Tools (`data/fixtures/tools/`)

The scripts that write and audit the fixture are committed: `fixture_io.py` (load/save, recomputes coverage),
`coverage.py`, `lua_table.py` + `stance_e70.py` (07 E-7.0 with `addSharedCombos`; an unquoted Lua name is nil, as
07 §4.1 requires for Blind Justice Forward Block input 3), `check_combo_tables.py` (recomputes all seven combo
tables and their names from the cached Module:Stances/data and reports differences; all seven match), and
`history/` (the data edits of both fix runs; the first reproduces d0b7dd0's JSON from e3a0049 apart from the stale
coverage). T18's emitter may reuse the E-7.0 converter as a cross-check.

## 2026-11-27 · T12 · Frontend foundation (initial contract)

T12 creates `frontend/` (everything except the feature folders of T30, T31, T35) and the root file
`playwright.config.mjs`. No Rust type, generated file or document changed. Items below are the interfaces later
frontend tasks (T30, T31, T35, T38) and the server (T11, T34) build on, and where the code differs from frontend.md.

### Dependencies (work-breakdown.md §2.1)

- `frontend/package.json` pins the frontend.md §1 versions exactly (TypeScript 6.0.3, typescript-eslint 8.74.0,
  ESLint 10.15.0, eslint-plugin-formatjs 8.1.4, vite 8.3.8, react-intl 12.2.1, @formatjs/cli 6.23.0); for the rows
  that name a line (`react 19.3`, `zustand 5.1` …) the pin is the newest patch of that line published by 2026-11-23
  (react/react-dom 19.3.1, react-aria-components 1.22.1, tailwindcss/@tailwindcss/vite 4.3.5, zustand 5.1.0, immer
  11.2.10, @tanstack/react-query 5.108.4, @tanstack/react-router 1.170.46, recharts 3.11.1, @tanstack/react-virtual
  3.14.28, fuse.js 7.5.0, @vitejs/plugin-react 6.3.0, vitest 5.1.1, @testing-library/react 16.3.3, msw 3.0.7,
  @playwright/test 1.65.1, @axe-core/playwright 4.14.0, @fontsource-variable/{inter,jetbrains-mono} 5.3.0).
- Additions the table does not list, needed by the listed tools: `@eslint/js` 10.0.1 and `globals` 17.14.0 (ESLint flat
  config), `eslint-plugin-react-hooks` 7.1.1 (the Lint row names it without a version), `@testing-library/dom` 10.4.2
  (peer of Testing Library), `@testing-library/user-event` 14.6.13, `jsdom` 31.0.1 (Vitest environment),
  `@types/react` / `@types/react-dom` 19.3.0, `@types/node` 22.20.5.
- `package-lock.json` was written by npm 11.6.2 (`npx npm@11.6.2 install`): the container's npm 10.9.9 crashes in
  arborist (`Cannot read properties of null (reading 'edgesOut')`) while resolving vitest's optional peers. `npm ci`
  with npm 10.9.9 installs from the lockfile without error. Regenerate the lockfile with npm ≥ 11.

### Root file `playwright.config.mjs` (created by T12; owner pending request C-40)

`npx --prefix frontend playwright test …` (T12 acceptance, `cargo xtask ci`) runs in the repository root, where
Playwright looks for its configuration; the root file re-exports `frontend/playwright.config.ts` (an ES module, so the
TypeScript config loads as ESM). Every path in the real config is resolved from `frontend/`. The file lies outside
T12's `frontend/` ownership: request C-40 (below) asks the manager to assign it.

### Playwright targets (for T11, T34, T38)

- `WFD_E2E_TARGET=server`: the webServer runs `${WFD_BIN:-target/release/warframe-dps} --no-browser --port 0
  --data-dir frontend/test-results/server-data [--bundle $WFD_E2E_BUNDLE]` with environment `WFD_IMAGES_FETCH=false`
  (`images.fetch = false`, through the `WFD_*` layer of server-api.md §12.1) and `WFD_OPEN_BROWSER=false`, and waits
  for the startup line of server-api.md §1.2 step 5, capturing the port from `http://127.0.0.1:<port>/` into
  `WFD_PORT` (tests read it through `frontend/e2e/fixtures.ts`). **T11**: make these two variable names and the
  `--data-dir` flag work, or record the names you chose here (T12 or T38 then updates `frontend/playwright.config.ts`).
- Default (`mock`), used until the server exists: `vite --mode mock` with MSW in the browser, on 127.0.0.1 at
  `WFD_MOCK_PORT` or else a port in 5200-5999 derived from the checkout's path (parallel worktrees do not collide).
  An already running server is reused only with `WFD_E2E_REUSE=1`; otherwise a taken port fails the run
  (`--strictPort`) instead of testing another checkout's code.
- Browser: @playwright/test 1.65.1 expects Chromium revision 1255; when it is not installed the config launches
  `WFD_CHROMIUM` or `/opt/pw-browsers/chromium` (revision 1243 in this container). Runs pass `--trace off`.
- `frontend/e2e/fixtures.ts` exports `test`, `expect` and `expectAxeClean(page, context)` (WCAG 2.2 A/AA tags, fails on
  serious or critical violations); later specs import from it.

### Feature registry (`frontend/src/features/registry.ts`)

The shell finds feature modules by file name, so feature tasks never edit shell files. Each module exports a
component with the file's name: `loadout/LoadoutPane.tsx` (`LoadoutPane`), `results/ResultsPane.tsx`
(`ResultsPane`), `target/TargetPane.tsx` (`TargetPane`) (eager); `optimizer/OptimizerPage.tsx`,
`data/DataPage.tsx`, `about/AboutPage.tsx` (lazy routes, no props); `share/ShareDialog.tsx`,
`share/ImportDialog.tsx` (lazy, props `DialogSlotProps { isOpen, onOpenChange }`, opened by the top bar). A missing
module shows a placeholder or a disabled button.

### State, API and i18n interfaces

- Store (`state/store.ts`, `useAppStore`, `createAppStore` for tests): `doc: CalcInput`, `ready`, `ui: UiState`,
  `baseline`, `canUndo`/`canRedo`, `bootDecode`, and the actions `replaceDoc`, `edit(label, recipe, coalesceKey?)`,
  `setWeapon(key, keep?) → { dropped }` (always clears `inc2`-`inc5` and `arcane1`, variant picks, weapon-item
  settings, `melee_plans`, `pellet_hit`; every other slot, the stance included, stays when `keep(entry)` is true
  (default: keep all); the stance combo stays only with a kept stance; the caller passes the legality predicate from
  `/api/candidates` and shows the undoable toast, T30; frontend.md §5), `setBootDecode`, `placeItem`, `moveItem` (swaps), `removeItem`, `setItemRank`, `setItemParam`, `setRiven`,
  `setVariant`, `setWeaponItem`, `setStanceCombo`, `setWarframe`, `setToggle(path, value)`, `setEnemy`, `setBuff`,
  `setBuffCaster`, `setCustom`, `resetCustom`, `setConditionsMode`, `setOverride`, `setEval`, `undo`, `redo`,
  `clearHistory`, `pinBaseline(report)`, `setBaselineReport`, `clearBaseline`, `setUi`, `setChartMode`,
  `togglePanel`, `reroll`. History: 200 steps, same-key edits within 800 ms coalesce.
- Boot decode (frontend.md §9): `useBoot` (shell) decodes the `?b=` code or the stored last build and records the
  result in the store as `bootDecode: BootDecode | null` (`{ code, report?, error?, issues? }`; `error` set when the
  decode failed and a new build was started). The shell's `BootNotice` shows a failure (role `alert`) or a report
  with entries (`reportHasEntries`; role `status`) above the panes with a Dismiss button; dismissing calls
  `setBootDecode(null)`. **T30**: the share/import feature may read `bootDecode` to show the full report in its own
  UI and dismiss it the same way.
- Prefs (`state/prefs.ts`): `loadPrefs` type-checks every field (theme, density, locale, `lastBuild` must be a
  string starting with `wfd1-`, prohibited entries, collapsed) and falls back to the default per field.
- Schema paths (`state/paths.ts`): a `ToggleSchema.field` starting with `build.` is relative to the `CalcInput`, any
  other path to its `scenario` (the mini fixture uses `eval.window_s`, `conditions.mode`, `toggles.*`,
  `build.variant.progenitor`). The generic control (`schema/SchemaControl.tsx`) binds bool, number, percent, select
  and optional-number toggles; `toggles.heavy` and `toggles.stealth` hold `{ start_combo }` / `{ rank }` objects
  (`NUMBER_MEMBER`). Schema-driven object toggles use the server default: switching one on writes `{}` (T-09 start
  combo 12, T-08 the weapon's maximum rank; spec 00 §6.1, C-15), an object without its member reads as on with an
  empty number field showing "Default", and clearing the number writes `{}` again; `buffs` and `build.variant.*` fields are left to the target and loadout features.
- API (`api/client.ts` `api.*`, one function per route of server-api.md §3; errors are `ApiError { status, msg:
  Message, issues }`); hooks in `api/queries.ts` (`useMeta`, `useCatalog`, `useSchema`, `useWeaponDetail`,
  `useItemDetail`, `useEnemyDetail`, `useCandidates`, `useCalcEv`, `useCalcMc`, `useTimeline`, `useExplain`, the
  build-code, optimizer and data mutations). `useCandidates(doc, slot, query?, { stat?, debounceMs? })` has the
  frontend.md §5 signature and builds the `CandidatesRequest`; `useCandidatesRequest(req)` takes a prebuilt one; `api/seed.ts` (`drawSeed`, `buildMcRequest`: a fresh 53-bit seed in
  `want.mc.seed` per MC request when `eval.seed = "auto"`, nothing for a fixed seed); `api/sse.ts`
  (`openOptimizerEvents`, `openDataEvents`: fetch-based, `Last-Event-ID` sent on every reconnect and on a resumed
  open; a connection counts as successful only once it delivers an event; it gives up after `maxAttempts` (20)
  consecutive failures or `maxEmptyEnds` (3) consecutive clean ends without an event: the server closes a stream
  only after a terminal event, so an empty clean end means the job finished at or before the resume id). `features/shell/useCalcSession.ts` runs the EV and MC queries of the store document; the results feature
  calls the same hook (same query keys).
- i18n: feature messages live in `src/i18n/en/<feature>.json` (flat `id → ICU message`, ids prefixed by the feature
  name, `loadout.modGrid.empty`); `scripts/gen-i18n.mjs` merges them into `src/i18n/en.json` and the `MessageId`
  union `src/i18n/messages.ts`, both **generated and gitignored** (the `pre*` npm scripts run it), so parallel feature
  tasks never conflict on them; a duplicate id fails the merge. `en/server.json` (T12) holds an English message for
  every `MESSAGE_KEYS` key with exactly its declared arguments (Vitest `i18n/messages.test.ts`); the Rust I-8 test
  should read `src/i18n/en/*.json`. `en/schema.json` holds the curated label keys the mini fixture uses
  (`toggle.tNN.label|help` for T-01 … T-43, `opt.*`, `view.*`, `condclass.*`); data-driven ids render through
  `formatDynamic`, which falls back to the id. **T41**: `doc-check --strict` should treat `frontend/src/i18n/en.json`
  and `messages.ts` (named in frontend.md §2) as generated paths.
- Mocks: `src/mocks/handlers.ts` answers every route from a digest of `data/fixtures/mini` generated by
  `scripts/gen-mocks.mjs` (gitignored `src/mocks/fixture.generated.json`); calculation numbers are synthetic
  placeholders, not engine values. Feature tasks add handlers in `src/mocks/extra/<task>.ts` (export `handlers`),
  which take precedence. Mock share codes are `wfd1-mock-<base64url JSON>`.

### Differences from frontend.md (spec alignment, work-breakdown.md §2.4; for the manager / T41)

| # | Item | frontend.md | Implemented |
|---|---|---|---|
| F-1 | `--text-3` | §10 table: #7a8699 dark, #5f6b7d light, "all ≥ 4.5:1 on their surface" | the table's values give 4.23 and 3.71 (dark, on `--surface-2` / `-3`) and 4.39 (light, on `--surface-3`); #8a95a8 dark and #5a6678 light pass on all four surfaces (`design/tokens.test.ts`). Additions: `--accent-fill` / `--on-accent` (light #8f5a00 / white: the table's light accent with any text colour stays below 4.5:1), `--border`, `--border-strong`, the 21 `--dt-*` colours. Proposed: update the §10 table. |
| F-2 | Where UI text lives | §8: "All UI text lives in `src/i18n/en.json`" | per-feature source files `src/i18n/en/*.json` (work-breakdown.md §2.2) merged into the generated `en.json` |
| F-3 | Default seed of a new document | domain-model.md §9.3 `eval.seed` default `Fixed(0)` (serde default) | new documents of the UI use `"auto"` (frontend.md §5, parity row 25); the serde default for a document without `seed` is unchanged |
| F-4 | `useCandidates` | §5 `useCandidates(doc, slot, query)` | that signature plus an options object (`stat`, `debounceMs`); the prebuilt-request form is `useCandidatesRequest` |

### Requests to the manager (work-breakdown.md §2.4)

Numbered C-31 on the branch; renumbered C-40 at the merge (T02a took C-31 … C-36, T03 C-37 … C-39).

| # | Conflict | Source | Provisional choice |
|---|---|---|---|
| C-40 | Owner of the root file `playwright.config.mjs` | work-breakdown.md §8 assigns T12 only `frontend/`; the T12 acceptance and `cargo xtask ci` (T01c, `xtask/src/ci.rs`) run `npx --prefix frontend playwright test …` from the repository root, where Playwright looks for its configuration | the one-line re-export, created by T12; either add it to §8 under T12, or change the acceptance command and `ci.rs` to pass `--config frontend/playwright.config.ts` (T01c) and delete the root file |

## 2026-11-27 · Manager · Rulings on the whole-contract review (pass 1)

Input: `docs/project/inputs/contract-review-pass1.json`, five independent xhigh reviews of every frozen interface after
wave 0: model `M-1` … `M-19`, engine contracts `ENG-1` … `ENG-24`, crate APIs `API-1` … `API-23`, data contracts
`DC-1` … `DC-29`, and a verified recommendation for every open request (C-31 … C-40, T02b-n, T02c-Rn, T02d-n,
T03b-Rn, T03c-n, LINT-GLAIVE). 133 items: 5 critical, 68 major, 60 minor.

1. Every item is accepted with the reviewer's proposed fix. The fixer of each area verifies the item against the code
   and the spec first; an item that does not hold, or a better fix of the same scope, is recorded with the reason in
   the area's CONTRACTS.md section.
2. Breaking changes approved now, because nothing implements these interfaces yet and wave 1 has not started:
   ENG-1, ENG-4, ENG-6, ENG-7, ENG-14, API-1, C-32, M-8, M-14, DC-3, API-22.
3. Specific choices where the reviewer offered options:
   - T03b-R6: target-vulnerability groups get `dot = "none"` in the fixture; T-20 is `percent`; **T-08 and T-09 become
     `optional_number`** with the one-member-object convention documented on `ControlType` (T12's tested binding
     stays); T-24 and T-25 are `custom` in the fixture.
   - T02c-R4: both code options rejected. `cargo xtask ci` is not a merge gate before T38; merges run the §2.3
     Definition of done plus each task's acceptance; `ci --keep-going` is informational.
   - T02c-R5: adopt T04's Provides: without `WFD_DATA_DIR` and without `data/generated/`, `wfd-data/build.rs` embeds
     `data/fixtures/mini/` and emits a cargo warning; AD-11 is reworded; `xtask dist` (T39) refuses a fixture bundle.
   - T02c-R6: accepted as implemented; `data/spec_tables.toml` added to AD-13, pipeline.md §9.2 and T11; T11 embeds
     the refresh inputs through its build.rs with an empty placeholder while `data/generated/spec/` is missing.
   - C-31: `crates/wfd-engine/{Cargo.toml, src/error.rs, src/cancel.rs}` are owned by T17; any engine task adds its
     dev-dependencies, `[[bench]]` entries and `EngineError` variants through a CONTRACTS.md entry (the C-21 rule).
     `EngineError` becomes `#[non_exhaustive]`.
   - C-32 and API-1: `Engine::timeline` takes the seed (or a small request struct); `Engine::units` is added.
   - C-34: engine.md is brought in line with the frozen code in this pass (§1-§6 and §14 at least) by the engine fixer.
   - C-37: one stat-registry row `doughty_rank` (no new mechanic kind); T13 computes 02 E-2.5a per instance.
   - C-40: the root `playwright.config.mjs` shim joins T12's §8 row.
   - T02c-R2, T02d-6: unowned crate files get owners (T18 for the datagen shared files, T32 for
     `crates/wfd-opt/Cargo.toml`), changed by others through CONTRACTS.md entries.
   - T02c-R3, T02d-2, API-13, API-18: task sections of work-breakdown.md amended as the reviewer proposes.
   - DC-18: the override-file authoring types are defined now in `curated.rs` (T01a role, this pass) and added to
     gen-schema.
   - LINT-GLAIVE: the wider fix (MeleeClass names, a `MeleePlanKind::ALL`, spec symbol and mechanic ids read from the
     spec at lint time), with tests; overview.md I-2 and testing.md §2.5 amended.
4. Spec-text items (T03b-R1 … R4, T03c-1, C-36, the 04 §5.4 wording of C-37, DC-12, DC-13, DC-16, M-17 and every other
   item whose fix names a spec document) go to a spec round; code changes do not wait for it and stay additive.
5. For this pass only, each area fixer (model-core, model-records, engine, crates, data) may edit outside its usual Owns
   line where an accepted fix requires it, and records every such edit in its section "Contract review fixes (<area>)".

---

## 2026-11-27 · Contract review fixes (model-core)

Area: `crates/wfd-model/src/` effect DSL, mechanic kinds, stat registry, curated authoring types, rule-table and item
types (`effect.rs`, `mechanic.rs`, `stats.rs`, `curated.rs`, `rules.rs`, `item.rs`, plus `frame.rs` and the new
`sets.rs`), `data/curated/schema.json` (regenerated), domain-model.md §10-§17. Input: the items of
`docs/project/inputs/contract-review-pass1.json` assigned to model-core, applied under the manager's rulings above.
Every change is additive for documents and curated files except the approved DC-3. The postcard layout of the bundle
types changes (new fields and variants); no bundle has been built, so `BUNDLE_SCHEMA_VERSION` stays 1 (T01d ruling).
`N_STATS` is 308 (was 296).

### Applied

| Item | Change |
|---|---|
| M-2, DC-13 (b) | Registry row `heat_inherit_faction` (`HeatInheritFaction`; F_inh, 03 §3 `heat_inh_faction`, §10.3; Sum, Inc, E-13). The custom-stat fixture item is T03's (below). |
| M-3, DC-16 | **Better fix of the same scope, reviewer's (a)-(c) not applied as written.** One source-kind flag on the catalogue entry: `ItemKind::Buff.ability_source: bool` (`#[serde(default)]`) and the curated header field `ability_source` (`CuratedItemFile.ability_source: Option<bool>`). The gate of 05 §13.1 depends on the enemy (`EnemyFlags::ABILITY_DEBUFF_IMMUNE`), which the scenario compiler knows when it lowers an item (L1, domain-model.md §18 "lowers one item in a scenario context"), so L1 drops the item's target-side debuffs (`vuln_add`, `target_vuln` groups, `strip_ability`, `strip_points`) there. This needs neither `EffectDef.ability_source`, nor `MultGroupDef.ability`, nor a second 54-slot `vuln_add_ability` row (which would have grown every `StatVec` by 54 slots for a per-scenario constant); it also gates Polarize's point strips and any future strip slot without a new row. Rule documented on the field, in domain-model.md §12.2 and §15.1. |
| M-4, DC-13 (d) | Rows `strip_flat_per_hit` (per-hit base-armor points; the 05 §9.4 qualifying-hit rule stays in the stage, E-5.11), `strip_points` (P_pts, E-5.10) and `strip_companion` (companion % strips in Σ_add, Sharpened Claws; not gated by 05 §13.1). The reviewer's alternative of reusing `strip_aura` was not taken: Corrosive Projection carries `aura_str` (Coaction Drift), a companion strip does not. Names follow the existing `strip_*` rows; the symbols name 05's `armor_flat_strip`, `armor_pts_strip`. |
| M-5, C-37, ENG-19 (model part) | Ruling C-37: one row `doughty_rank` (b_r = 0.5 + 0.1·rank; Max, Inc, E-4, "02 E-2.5a"), never summed into `cd_abs`; the crit stage computes E-2.5a per instance and the compat path places it per D-014. `CritResolved.doughty` (ENG-19) is the engine fixer's. |
| M-6, DC-18 | Authoring types in `curated.rs`, each with the common header (`schema`, `sources`, `notes`) and `deny_unknown_fields`: `WeaponOverrideFile` (`overrides/weapons/<slot>/<slug>.toml`: `id`, `trigger`, `magazine: MagazineOverride { size, infinite, battery }`, `reload: ReloadSpec`, `bow`, `volley`, `[[passive]]`, `[[attack]]` = `AttackFireOverride { name, trigger, spool, kohm, ramp_start, semi_cap, charge_time, charge_fixed, magazine_override, gauge, sniper_combo, roles, [[attack.passive]] }`), `AttackOverrideFile` (`overrides/attacks/<weapon-slug>.toml`: `weapon`, `[[attack]]` = `AttackRecord { name, headshot_1x, hs_unique, speargun_throw, radial_parent_part, torso_only, [[attack.forced]] (ForcedProcRule), parent, no_parent, roles }` per 02 §4.10, `[[group]]` = `AttackGroupOverride { members, combinable, summed }`), `EnemyOverrideFile` (`overrides/enemies/<slug>.toml`: `enemies`, `replace_parts`, `default_part`, `[[part]]` = `PartOverride` with every `BodyPart` field optional), `ProgenitorFile` (`overrides/progenitor.toml`: `default: ProgenitorSpec`, `[[weapon]]` = `ProgenitorRow { id, spec, source }`) and `EnemyAliasFile` (`aliases/enemies.toml`: `[[alias]]` = `EnemyAlias { unique_name, wiki, source }`). Split: firing tables and passives in `weapons/` (T20-T22, T29), attack structure and 02 §4.10 records in `attacks/` (T18). `UNLOADED_DIRS` doc names the §8 owners; the loader still skips `overrides/` and `aliases/` (T18 reads them with these types). Test `curated::tests::override_and_alias_files_parse`. Adding them to `data/curated/schema.json` needs `api/codegen.rs` (model-records, below). |
| DC-1 | `CounterEvent::AnyInput` (`per_event = "any_input"`: every light, slide, aerial or wall input; heavies and slams add none). Chosen over `Inputs(Vec<InputKind>)` because `InputKind` holds exactly those four kinds and `CounterEvent` stays `Copy`. §11.2, §16.13 Melee Animosity updated. |
| DC-2 | `proc_on_event` accepts `on = "proc"` without `on_status` (= any delivered status, π_any of 02 E-2.17); `on_status` with another `on` is still rejected. New `ProcOnEventParams.limit: ProcLimit { PerInstance (default), PerEvent }` (readable `limit = "per_event"`). `TriggerId::OnAnyStatus` (class (b)), appended after `OnPull`. The unit test that pinned "on = proc needs on_status" now pins the relaxed rule. |
| DC-3 (breaking, approved) | `ProcAfterTicksParams { combined: StatusSet, stacks_at_least: u8, adds }` (was `combined: StatusType, ticks: u8`); readable `combined` takes one name or a list (`StatusesRepr`), `ticks` stays an alias of `stacks_at_least`; magnitude slot `mech.<n>` = p_deb (docs on `Mechanic::ProcAfterTicks`, §11.2). |
| DC-4 | `MechanicKind::StackPresentStatuses` / `Mechanic::StackPresentStatuses(StackPresentStatusesParams { statuses: StatusSet, ragdoll: bool })`, magnitude N = rank + 1 (04 E-4.20). Engine skeleton added (below). |
| DC-5 | Row `glaive_volatile_chance` (p_vol, 07 E-7.36; Sum, Inc). Volatile removed from the `spawn_attack` docs (`Mechanic::SpawnAttack`, `SpawnAttackParams.fraction`, `SpawnEvent::Bounce` kept with "no catalogue user") and from §11.2. |
| DC-6 | `HitFilter.flags: InstanceFlags` (new bitflags, `TENNOKAI`; readable `flags = ["tennokai"]`); rows `tennokai_window` (T_tk, Max, Inc; 0 = the 2 s default) and `tennokai_every` (n_tk, Max, **Unknown**: it replaces a chance by a spacing). |
| DC-7 | `HitFilter.min_distance: Option<f64>`: T-27 d strictly greater (04 §3.2 "d > 15 m"); chosen over `ResolvedStat::Distance` because `stat_at_least` is ≥ and the spec gate is strict. A gate (derived class GATE). |
| DC-8 | `ResolvedStat::Radius` (`"radius"`), `Per::CompressionEff(CompressionMode)` with `CompressionMode { Adds, Multiplies }` (`effect.rs`): eff_a when the attack's mode matches, else 0, so one Multiplies effect on the G_F group and one Adds effect on A cover the page's three modes without a scope construct. The attack field (`Attack.compression`) is model-records' (below). |
| DC-9 | `StackModel.read: ReadTiming { BeforeInstance (default), AfterShot }`. §16.13 Double Tap encodes TV-04-25 exactly (`value = { base = -0.20, add = [{ coeff = 0.20, per = ["stack"] }] }`, `max = 21`, `read = "after_shot"`). |
| DC-10 | `StackModel.gain_on: Option<Cond>` (the gain event with filters; `when` is then the activity condition; its class joins the derived class). The reading of `Cond::All`/`Any` as a gain event (one trigger conjunct = the event, the others filters at the event) is documented on `Cond` and in domain-model.md §12.1. |
| DC-11 | `EffectDef.dot_scaling: Option<bool>`, only on element targets (`element`, `elem_nc.<t>`; rejected elsewhere); `None` = the 03 §9.3 default (true, false for `target_weakness`). |
| DC-12 | `sc_mul` is M_sc,w (`sc_mult_w`); new `MultStage::ScVuln` (`"sc_vuln"`): one target-side M_sc,t factor per group (Pageflight, Dante ×1.5). Whether custom p.k is weapon side (`sc_mul`, 00 §6.3) or target side (a `sc_vuln` group, 02 E-2.11) is the spec round's; both are expressible. |
| DC-13 (a), (c), (e) | Rows `part_body_pen` (ΣBP; Sum, Dec, E-7), `cc_set` and `sc_set` ("set to"; Max, Unknown; placement in E-5/E-8 is a question to 02), `infinite_ammo` (Any, Inc, 06). |
| DC-14 | `Frame.passives: Vec<EffectDef>` (`#[serde(default)]`; applied when the frame is the build's frame or T-16 implies it) and its source `rules/frames.toml` `[[frame.passive]]` (`FrameRuleRow.passives`). `ExtraHitParams.status_duration_add` kept (removing it would break for nothing) and documented as having no catalogue user; §16.8 Toxic Lash and the §16.13 test row no longer carry it. |
| DC-15 | `Per::AbilityDuration` (static Dur_e of the caster). |
| DC-17 | `TargetPred::Debuffed(Box<str>)` (named ability debuff, never simulated: uptime), `TargetPred::Eximus` (the scenario's Eximus switch: derived class GATE, not (a)), `EffectDef.uptime: Option<UptimeSpec { on: ValueExpr, period: ValueExpr }>` (only on conditional effects; EV u = on/period, MC on/off process). Simulated rule for `Marked`, `Incapacitated`, `Debuffed` in §12.2. |
| DC-19 | `rules/game_constants.toml` = `GameConstantsFile { [[constant]] GameConstantRow { key, value, source } }`, a `RULE_TABLES` entry and `CuratedRules.game_constants`. Owner proposed: T03b (owner of `rules/`, pipeline.md §6.1); the manager names it. `Rules.co_default` (04 §2.5 D2-D6) is not given a curated table: its defaults are qualitative rules of the spec, so the data area ingests 04 §2.5 as a fifth spec table (below). |
| DC-21 | `StackModel.duration` doc: 0 = no expiry in every class (E-4.9 with D = ∞). The pipeline.md §7 validator wording (`durations ≥ 0`) is the data area's. |
| DC-22 | `StackModel.default_count: Option<u16>` (default m_e of a user count; Piercing Navigator 0). §15.2: "+Nx" binds as `nK`. Fixture items are T03's. |
| DC-24 | `WeaponFilter.none_traits: WeaponTraits`, `WeaponFilter.weapons: SmallVec<[UniqueName; 1]>`, `RoleFilter.triggers: SmallVec<[Trigger; 2]>` (fire-mode trigger class). `TriggerId::Combo(n)` documented as M(c) ≥ n; `Cond::derived_class` gives it COMBO (not (b)), like `Per::ComboTier`. |
| DC-25 (model part) | `MultStage::Spot` doc corrected (inherited by DoT through `Part_dot = Part`, D-095). The `devouring_attrition` group in `mult_groups.toml` is T03b's (below); §16.13 states that `random_multiplier` names no group. |
| DC-26 | `CuratedItemFile.noncounting: Vec<UniqueName>` (set files). |
| DC-27 | One rule: binding fields are kept in the bundle form (generated JSON and `bundle.bin`) as provenance; no consumer outside the data build reads them. Matches the fixture as it is (no fixture change); `effect.rs` module doc and §15.1 updated. |
| API-6 | New module `wfd_model::sets`: `set_counts(&Bundle, &Build) -> SmallVec<[(SetId, u8); 4]>` (09 E-9.40: frame and active-weapon general/exilus/aura slots plus `companion_set_mods`, distinct identities, `noncounting`, member cap), with `set_slot_items(&Build)` and the pure `count_pieces(items, sets, ids)`; unit test `sets::tests`. `legality::set_counts` (T04) delegates to it and `compile/sets.rs` (T06) calls it. |
| API-12 (model part) | `RivenTraitDef.vintage_companion: bool` (`#[serde(default)]`): a vintage trait that generation must not produce on companion rivens (09 §12.3). |
| API-16 (model part) | `RankCurve::value(&self, max, r, max_rank) -> f64` (09 E-9.13; a rank above R reads as R, a short table takes its last entry); unit test `effect::tests::rank_curve_values`. |
| M-19 (model part) | `BuffGroup::Squad` doc: never Corrosive Projection or Shield Disruption (T-23 only; T19 rejects such a buff). `BuffGroup` doc: a UI grouping, not the 05 §13.1 source kind. |
| ENG-2 (model part) | domain-model.md §12.2: the `id` of `CondStacks(id)` indexes the engine's condition table (the table itself is the engine fixer's). |
| C-39 | domain-model.md §17 rows "DoT-only multipliers" and "Hit-outcome gate" reworded as R-2 constructs without a catalogue member; the stale "fixture item still has the gate form" text removed. Tokens unchanged, so the coverage-test exemption stays. |
| T03b-R2 (model part) | Docs: the sub-index of `arcane<n>` is n (`ElemOrder::Slot`, `SlotDef.element_order`, domain-model.md §13), UNVERIFIED until the 01 spec round words it. |
| T03b-R5 (model part) | `ToggleDescriptor.field` doc: an `opt.` path is never written by calculator panels, so such a toggle (T-36) is `custom`. |

### Rejected or without a model-core part

| Item | Reason |
|---|---|
| ENG-7, ENG-15, API-1, C-36 | Engine types only (`CompiledBuild`, `CompiledScenario`, `Engine::timeline`); no type in model-core files. |
| M-1 | Fixed on the weapon (`ExaltedScaling` per-rank variants, `Attack.exalted_s`; weapon.rs, model-records): with per-rank data on the record no rule table or DSL exalted-rank scalar is needed. |
| M-7 … M-18 (except M-19's doc) | Their types are in model-records' files (`weapon.rs`, `enemy.rs`, `enums.rs`, `scenario.rs`, `build.rs`, `api/**`); notes for them below where an item touches a model-core type. |
| API-3, API-5, API-7, API-8, API-9, API-11, API-14, API-15, API-17, API-20, API-23 | Crate APIs, `bundle.rs` or `api/**`: no part in model-core files. |
| DC-20 | `LIST_KEYS` (wfd-model `tests/fixtures.rs`) and `data/fixtures/README.md` belong to the fixture (T03, data area). |
| C-38 | Accepted with no model change (the reviewer's fix: four per-element value params, default 0); fixture and 04 §5.5 wording are T03's and the spec round's. |
| LINT-GLAIVE | No part in model-core files (`xtask/src/lint_engine.rs`: crates fixer; `MeleePlanKind::ALL`: model-records). `cargo xtask lint-engine` still reports the two known "Glaive" findings in this worktree. The new registry rows (`glaive_volatile_chance`, `tennokai_*`) are model vocabulary through `STAT_ROWS`, so they are exempt already. |
| T02c-R5, T02c-R6, T03c-3 | Manager and architecture rulings (AD-11, AD-13, manager-log); no model type. |
| T03c-2, T03c-TriggerScaling-Wave | `weapon.rs` doc changes (model-records). |
| M-3 (a), (b), (c) as written | Replaced by the item-level flag and the L1 gate (see "Applied", M-3, DC-16). |

### Edits outside the area's files (ruling 5)

- `crates/wfd-model/src/frame.rs` (no area lists it): `Frame.passives` (DC-14).
- `crates/wfd-model/src/sets.rs` (new) and `lib.rs` (`pub mod sets;`, crate doc names the shared pure rules): API-6.
- `crates/wfd-engine/src/mechanics/mod.rs` and new `mechanics/stack_present_statuses.rs` (T14's files): the exhaustive
  `mechanic_impl` match needs an arm for the new kind (DC-4; skeleton with `todo!("T14")` like the other kinds), and
  the dispatch test builds `ProcAfterTicksParams` with the new fields (DC-3).
- `crates/wfd-model/tests/documents.rs`: the bundle round trip sets the new fields (`ability_source`,
  `vintage_companion`, `Frame.passives`) and adds effects with every new DSL construct.
- `crates/wfd-model/tests/domain_examples.rs`: §16.13 rows follow the new contract (tests updated, none removed):
  Melee Doughty (`doughty_rank`, ruling C-37), Melee Animosity (`any_input`), Galvanized Scope (`gain_on`), Toxic Lash
  (no `status_duration_add`); 28 rows added for the new constructs.
- `crates/wfd-model/src/stats.rs` test `registry_shape`: `N_STATS` 296 → 308 (the count pinned the old registry).
- `frontend/src/api/generated/BuffGroup.ts`: regenerated (doc comment only); frontend lint, typecheck and tests pass.

### For model-records (exact changes; the merge applies them)

- `crates/wfd-model/src/api/codegen.rs` `CURATED_FILE_SCHEMAS` and `curated_schema_json()`: add
  `("overrides/weapons/**", "WeaponOverrideFile")`, `("overrides/attacks/*.toml", "AttackOverrideFile")`,
  `("overrides/enemies/*.toml", "EnemyOverrideFile")`, `("overrides/progenitor.toml", "ProgenitorFile")`,
  `("aliases/enemies.toml", "EnemyAliasFile")`, `("rules/game_constants.toml", "GameConstantsFile")` and the matching
  `g.subschema_for::<…>()` calls; then `cargo xtask gen-schema` (M-6, DC-18, DC-19).
- `weapon.rs` (DC-8): `Attack.compression: Option<Compression { eff: f64, mode: crate::effect::CompressionMode }>`
  (`#[serde(default)]`), keeping or retiring `compression_eff`; `Per::CompressionEff` reads it.
- `weapon.rs` or `enums.rs` (M-6, 06 §4.3 bow list): a bundle field for "a bow beyond `Class = Bow`" (Artemis Bow), e.g.
  `WeaponTraits::BOW`; `WeaponOverrideFile.bow` maps onto it. If M-9 adds `AttackRoles::STUN_GATED` and M-15 adds
  `Attack.zoom_level`, `AttackRecord.roles` already carries the role and `AttackRecord` gains `zoom_level: Option<u8>`.
- domain-model.md §5 `Frame`: add `pub passives: Vec<EffectDef>, // frame passive effects when this is the build's
  frame (04 §5.11; rules/frames.toml)`, and change "Frames carry identity, slot layout and Helminth facts only" to
  include the passives (DC-14).
- domain-model.md §6: `ItemKind::Buff` gains `inspiration_key` (existing) and `ability_source: bool` (05 §13.1);
  `SetDef` lists `noncounting` (existing); `RivenTraitDef` gains `vintage_companion: bool` (API-12) and the existing
  `coeff`, `sign`, `spliced`, `recipes`, `inspiration_key`.
- `scenario.rs` `SquadAuras` doc and domain-model.md §9.1 (M-19): squadmates' Corrosive Projection and Shield Disruption
  enter only through T-23 (`squad_cp`, `squad_shield_disruption`), never as buff items.
- `api/opt.rs` `RivenPolicy::Search.quality` doc (API-12): "u of 04 E-4.18 (0 … 1), applied to every trait".

### For other areas

- Engine (T06, T13, T14): lower the 05 §13.1 gate at L1 from `ItemKind::Buff.ability_source` and
  `EnemyFlags::ABILITY_DEBUFF_IMMUNE`; read `doughty_rank` in the crit stage (E-2.5a); implement
  `stack_present_statuses`; `compile/sets.rs` calls `wfd_model::sets::set_counts`; `RankCurve::value` is the per-rank
  evaluator.
- Crates (T04): `legality::set_counts` delegates to `wfd_model::sets::set_counts`; `riven_traits_for` filters
  `vintage_companion` on companion rivens.
- Data (T03 fixture, T03b rule tables, T18): fixture Melee Doughty → `stat = "doughty_rank"` (and the §17 row "Melee
  Doughty" with token `doughty_rank` once it exists; `encoding_check` needs the token); fixture custom item for
  `heatInheritFaction` (`heat_inherit_faction`); Toxic Lash fixture without `status_duration_add` and a Saryn
  `[[frame.passive]]`; Melee Animosity `per_event = "any_input"`; `ability_source = true` on Molecular Prime and the
  other ability debuffs (Magus Accelerant and Theorem Contagion stay false); Piercing Navigator `default_count = 0`;
  Arcane Crepuscular bound as `n2`; `mult_groups.toml`: remove or document `devouring_attrition` (DC-25) and add the
  `sc_vuln` groups when Pageflight or Dante are curated; `rules/game_constants.toml` (owner to be named) and 04 §2.5
  as a fifth spec table for `Rules.co_default` (DC-19); pipeline.md §7 "durations ≥ 0" (DC-21). New §17 rows for the
  constructs of this pass are added when the fixture has an item for each (the coverage test checks every token).
- Spec round: where `cc_set` and `sc_set` enter E-5 / E-8 (02); custom p.k placement (00 §6.3 vs 02 E-2.11); 01
  §4.10.1 sub-index of `arcane<n>`; 04 §5.4 Doughty wording (C-37); 04 source-kind column (OQ-523).

---

## 2026-11-27 · Contract review fixes (model-records)

Area: `crates/wfd-model/src/{weapon.rs, enemy.rs, enums.rs, scenario.rs, build.rs, bundle.rs, pick.rs, api/**,
message/**}` and the modules they use (here `frame.rs`, `lib.rs` and the new `sets.rs`), the generated
`frontend/src/api/generated/**` and `data/curated/schema.json` (regenerated, never edited), domain-model.md §1-§9 and
server-api.md. Input: `docs/project/inputs/contract-review-pass1.json`; the manager's rulings of the section above
apply. Every item below was checked against the code and the spec before the change. `BUNDLE_SCHEMA_VERSION` stays 1
for every bundle-layout change listed here (no bundle has been built; ruling on T01d, item T03c-3).

### Changes per item

| Id | Change (types, signatures, files) or reason for no change |
|---|---|
| M-1 | `weapon.rs`: `ExaltedScaling` is now the five values of 01 §2 `exalted_model` with the per-rank values of Table 4.3-A: `BaseXStr { s: [f64; 4] }` (JSON `base_x_str`), `MultInA { m: [f64; 4] }` (`mult_in_A`), `Peacemaker { p: [f64; 4] }`, `None`, `OutOfScope`; `deny_unknown_fields`. New `Attack.exalted_s: Option<[f64; 4]>` (serde default) for an attack whose model-1 ratios differ (Noctua's active attack). **Deviation from the reviewer's text** (same scope, recorded per ruling 1): the reviewer kept `BaseByStrength` and `None` "for compact-index stability" next to new `Model1/2/3`, which leaves two model-1 variants, one of them unable to evaluate T-31; with no bundle built (T01d ruling) index stability buys nothing, so the old `BaseByStrength` and `Partial(f64)` are replaced. Persisted-form change: the one fixture user, Glory in `data/fixtures/mini/weapons/exalted.json`, now reads `{ "base_x_str": { "s": [75, 100, 125, 150] } }` (Table 4.3-A). Tests: `weapon::tests::exalted_models_use_the_spec_names` (new); `tests/documents.rs` pinned `Partial(0.5)` and now uses `BaseXStr`. domain-model.md §4 updated. |
| M-2, M-3, M-4, M-5 | No part in these files: stat registry, effect DSL, rules and item kinds (model-core). |
| M-6 | The override-file types belong in `curated.rs` (model-core). The part in `api/codegen.rs` (mine) cannot land here, because the types do not exist in this tree; the exact change is under "For model-core" (1). |
| M-7 | `api/catalog.rs` `EnemyDetail`: new optional `spawn_level: Option<u16>`, `defaults: Option<EnemyDefaults>` (the model type; `None` when every default is off), `overguard_context: Option<Box<str>>`, all `skip_serializing_if`. Placed on the detail, not on `EnemySummary`: the target pane already needs the detail for T-03 parts and the Eximus kinds. server-api.md §5.5 updated; sample extended. |
| M-8 | `build.rs`: the four polarity maps (`WeaponItemDoc`, `WarframeDoc`, `WeaponItem`, `FrameLoadout`) are `BTreeMap<SlotId, Option<Polarity>>`: absent = innate, JSON `null` = Forma'd to no polarity (approved breaking change). TS `{ [k in SlotId]?: Polarity \| null }`. server-api.md §4 and domain-model.md §9.1-§9.2 updated; `tests/documents.rs` inserts `Some(..)` and checks a `null` entry. |
| M-9 | `enums.rs`: `AttackRoles::STUN_GATED = 1 << 28` (`stun_gated`), documented on the type (06 E-6.24, `W:Zymos` L33); `HEADSHOT_ONLY` alone has no stun gate. domain-model.md §3 updated. T18 sets it on Zymos "Headshot Explosion". |
| M-10, API-9 | Option (a), doc only: `OptRequest` (type doc), `Objective.window_s`, `.heavy`, `BuffPolicy.strength`, `.portfolio` and `RankedBuild.input` state that the server writes T-01, T-09, T-20, each ranked build's buff set and the other scenario inputs the search set into the scenario of every `RankedBuild.input`; `toggles.capacity` stays the input's; `/api/calc` on the code reproduces `score`. server-api.md §8.1 gains the paragraph. optimizer.md §1/§7 and `presets.rs` are not mine (below). |
| M-11 | Doc and ruling amendment, no type change. The T01d ruling row "Resolved `Scenario` not postcard-safe" lists two fields; the tree has **six** skipped fields: `CoStatusCount.n`, `Stealth.rank`, `MeleePlanSel::HeavyLoop.heavy_cancel`, `MeleePlanSel::BuildSpend.k` (new with M-14), `MeleePlanSel::DataAttackLoop.t_cyc`, `CondOverride::Uptime.stacks`; `Toggles::default()` already fails (verified: `co_status_count.n` is `None`). Stated on `scenario::Scenario` and in domain-model.md §9.2; whoever first postcard-encodes a resolved scenario applies `dual_serde!` or hashes another encoding. |
| M-12 | `api/calc.rs`: `CondClass::U` (`"u"`). server-api.md §5.6 (`conditions.classes[].id`) and §6.1 (`ConditionView.class`) widened; samples use `u`. |
| M-13 | `api/calc.rs` `ViewEv`: new optional `form_duration_s` (06 E-6.14, T-39 switches included) and `per_use` (06 E-6.14a; its presence flags the attack gauge-gated); `per_magazine` documented as the damage per form magazine for form attacks. server-api.md §6.1 updated. |
| M-14 | `scenario.rs`: `MeleePlanSel::BuildSpend { k: Option<f64> }` (`#[serde(default, skip_serializing_if)]`, TS `k?: number`); `None` = the weapon's M_cap; `{ "build_spend": {} }` parses (approved breaking change). `MeleePlanParam.default` documents `null` for it. Tests: `tests/documents.rs` pinned `k: 6.0`/`12.0` and now uses `Some(..)`, plus a `None` round trip and the `{}` form. server-api.md §4, domain-model.md §9.3 updated. |
| M-15 | `weapon.rs`: `ZoomLevel.enables: Vec<AttackIdx>` (serde default): the `ZOOM_ONLY` attacks that exist only at that level (Komorex "3.5x Zoom Radial Attack", TV-06-32); a `ZOOM_ONLY` attack no level lists exists at every zoom level. domain-model.md §4 updated. |
| M-16 | `enemy.rs`: `EnemyFlags::GUARDIAN_AURA_IMMUNE = 1 << 23` (set by data on the Demolisher family; E-5.27's `boss` and Guardian-Eximus exclusions stay on `BOSS` and the T-23 kind). Chosen over `DEMOLISHER`: the stage reads the effect, and a later 05 amendment (05 §7.6 also names capture targets) is data only. |
| M-17 | `enemy.rs`: `party_scaling: Option<Vec<f64>>` (was `[f64; 4]`; JSON and TOML form unchanged) so the open 05 question on P_5/P_6 can be carried either way: the last entry applies to larger squads until 05 answers (documented). New `Enemy.bolstered_og_factor: Option<f64>` (serde default; `None` = 1, Techrot Babau 0.25). The Archon convention (05 §4.5 "in Archon Hunts" rows are unconditional record data; marked UNVERIFIED) is on the `Enemy` doc and in domain-model.md §5. The curated enemy row (`curated.rs`, model-core) keeps `[f64; 4]`; T18 converts. |
| M-18 | Doc rule on `ScenarioDoc.custom` and domain-model.md §9.1/§9.3 (T-26 row): `abilityCombo` (`p.u`) is a T-11 alias; resolve (T04) moves it into `toggles.ability_combo` when T-11 is off and drops it when T-11 is set; import (T10) writes T-11. New optional `CustomStatSchema.toggle: Option<ToggleId>` marks the alias. server-api.md §4 and §5.6 updated. |
| M-19 | Doc rule on `SquadAuras` and domain-model.md §9.1: squadmates' Corrosive Projection and Shield Disruption enter only through `EnemySetup.squad_cp` / `.squad_shield_disruption`; T19 rejects a squad-aura buff of either. server-api.md §4 comment. |
| API-1 | Model part only: `TimelineRequest.seed` doc (the E-8.16 key of the replayed run; absent = `eval.seed`; required when that is `"auto"`, else 422) and `GroupRef.group` doc (units in `CalcReport.groups` order, `Engine::units`). server-api.md §6.2 comments. The engine signature is the engine fixer's. |
| API-3 | No part in these files: the feature-flag fix is superseded by the T02c-R5 ruling (mini fallback with a cargo warning, no feature flag). |
| API-5 | `bundle.rs`: new `Bundle.retired_keys: Vec<(StableKey, UniqueName)>` (serde default), disjoint from `stable_keys`, which now documents "every entry names a record". Chosen over "a key without a record is retired" because it keeps T02b's `every_stable_key_is_indexed` true and keeps the uniqueName, so a re-added uniqueName gets its old key. Test loaders set it (`tests/fixtures.rs` empty, `tests/documents.rs` one entry). domain-model.md §8 updated. |
| API-6 | New module `crates/wfd-model/src/sets.rs`: `pub fn set_counts(bundle: &Bundle, build: &Build) -> SmallVec<[(SetId, u8); 4]>` (09 E-9.40: distinct identities over fgen0-7, fexilus, aura0-1, gen0-7, exilus and `companion_set_mods`; `SetDef.noncounting` never counts; ranks ignored; sorted by set id). Choice recorded on the function: declared companion pieces count whatever `companion_kind` says (E-9.41 is a violation, 09 §1.5 rule 1); the T02b stub doc of `legality::set_counts` said "when `companion_kind` allows them". `lib.rs` documents the one-function exception to "types only". Test `set_counts_follow_tv_09_09` (TV-09-09's three rows on the mini fixture, plus another set, an arcane slot and the noncounting rule) in `tests/fixtures.rs`. domain-model.md §6 updated. |
| API-7 | Doc only: `RankedBuild.code` says it is produced by `wfd-buildcode` (AD-18), filled in every event and result, and that the encoder call versus a sink stamp is the optimizer tasks' choice; the dependency change is the crates fixer's. |
| API-8 | `api/opt.rs` `RankedBuild`: new optional `capacity: Option<CapacityReport>` (under `constraints.capacity.mode`) and `violations: Vec<Violation>` (skip if empty). `OptResult.capacity` documented as equal to `best.capacity`. server-api.md §8.1 sentence. |
| API-11 | `api/opt.rs`: `OptResult.riven_sets: Vec<RivenSetRow>` (skip if empty) and `RivenSetRow { traits: Vec<Box<str>>, best: RankedBuild }`. |
| API-12 | Model-records part: `RivenPolicy::Search.quality` documented as u of 04 E-4.18 (u = 1 gives e_1: v_hi for a bonus, v_lo for a curse). server-api.md §8.1 comment. `RivenTraitDef.vintage_companion` is item.rs (model-core). |
| API-14 | `api/calc.rs`: `CalcReport.riven: Vec<RivenTraitRange>` (skip if empty), `RivenTraitRange { slot, trait_id, role?: TraitRole, range?: ValueRange }`, `TraitRole { Positive, Negative }` (the fields of `legality::TraitRange`). `RivenTraitSchema.range` re-documented as a static hint, absent in API v1 (not removed: that would break the frozen DTO). server-api.md §5.6 and §6.1 updated. |
| API-15 | `api/calc.rs`: `CapacityReport.riven_defaults: Vec<RivenDefault>` (skip if empty), `RivenDefault { slot: SlotId, drain: bool, polarity: bool }` (09 E-9.59, TV-09-18). server-api.md §6.1 updated. |
| API-16 | No part in these files (`RankCurve::value` is effect.rs, model-core; the function list is T04's). |
| API-17 | Doc option: `OptRequest` (type doc), `input`, `Constraints.must_include`/`.prohibited`, `OwnedShard.item`, `BuffPolicy.allowed` state that `input.x` maps every local key of the whole request and that the server resolves the lists through it. server-api.md §8.1 paragraph. |
| API-20 | `api/opt.rs` `Score`: per-band table on the type and per-field docs (08 E-8.26: band 2 primary = removed pool fraction, tie = `dps_sustained[auto]`), matching `wfd-engine` `ev::score::Score`. |
| API-23 | `api/data.rs`: `DataEvent::Failed { error, report: Option<Box<DataReport>> }` (skip if none). server-api.md §9 now names `DataReport` for both events. |
| DC-1 … DC-5, DC-7, DC-9 … DC-13, DC-15 … DC-17, DC-19, DC-21, DC-22, DC-24 … DC-27 | No part in these files (effect DSL, mechanic kinds, stat registry, curated, rules, item kinds; fixtures and §10-§17 texts). |
| DC-6 | No part landed: the new `InstanceFlags` type belongs with `HitFilter` (effect.rs, model-core). If model-core puts it in `enums.rs` (mine), the merge keeps that. |
| DC-8 | Model-records part: `weapon.rs` `Attack.compression_mode: CompressionMode` (serde default `multiplies`; `Multiplies` → a `G_F` factor, `Adds` → A) next to the kept `compression_eff` (`None` = "Doesn't Work" or not listed). Smaller than the reviewer's `compression: Option<Compression { eff, mode }>` beside the old field, which would hold eff twice; every fixture file keeps parsing. `ResolvedStat::Radius` and `Per::CompressionEff` are effect.rs (model-core). Test `compression_mode_defaults_to_multiplies`. domain-model.md §4 updated. |
| DC-14 | Model-records part: `frame.rs` `Frame.passives: Vec<EffectDef>` (serde default), documented with the T-16 → Saryn and the "never a second time" rule of 04 E-4.14; `frame.rs` is a module of `bundle.rs`, not one of model-core's listed files, so it is done here. domain-model.md §5 Frame block and paragraph updated; `tests/documents.rs` fills it. The Toxic Lash `status_duration_add` removal (§16.8, fixture) is model-core's and data's. |
| DC-18 | As M-6: types in `curated.rs` (model-core), codegen rows under "For model-core" (1). |
| DC-20 | `tests/fixtures.rs` `LIST_KEYS`: `rules.melee.finishers` keyed by (`class`, `weapons`), and `data/fixtures/README.md` rule 3 table row split accordingly (outside my files, below). |
| LINT-GLAIVE | Model-records part: `api/schema.rs` `pub const MeleePlanKind::ALL: &'static [MeleePlanKind]` (11 kinds, declaration order) with test `melee_plan_kind_all_is_complete`. The lint itself (`xtask/src/lint_engine.rs`) is the crates fixer's; `cargo xtask lint-engine` still reports the two "Glaive" problems here. |
| C-36, ENG-2, ENG-7, ENG-15, ENG-19 | No part in these files (compiled engine tables, `CritResolved`, `CompiledScenario`); ENG-15's TimelineRequest part is API-1. |
| C-37, C-38, C-39 | No part in these files (stat registry row, fixture item, §17 text). |
| T02c-R5 | No part in these files (overview.md AD-11, T04 build.rs, T39). |
| T02c-R6 | server-api.md §10: the `refresh` feature embeds `data/curated/`, `data/spec_tables.toml` and `data/generated/spec/` through `OUT_DIR` with an empty placeholder while the spec directory is missing (a refresh then fails with `DatagenError::Config`). overview.md, pipeline.md and work-breakdown.md T11 are the manager's. |
| T03b-R2 | No part in these files (domain-model.md §13 is model-core's). |
| T03b-R5 | server-api.md §5.6: `control.kind` gains `"custom"`, and a table documents the `field` forms (`ScenarioDoc` path, `\|`-joined paths, `buffs`, `build.`, `opt.`). `api/tests/shapes.rs`: the `ControlSchema.kind` entry of `TYPE_DIFFERENCES` removed (no longer needed). Code fix (smallest, chosen over re-marking T-36 in data): `frontend/src/schema/visibility.ts` `isGenericField` returns false for `opt.` fields and `custom` controls, with a test (below). |
| T03c-2 | Reviewer's rejection of a model change accepted; `HeavyClassRow.total` doc amended (a "–" class has no row: σ_h = 1, E-7.40 rule 2). domain-model.md §4 note. |
| T03c-3 | Nothing to do: the T01d ruling covers it; the same ruling covers every layout change above. |
| T03c-TriggerScaling-Wave | `TriggerScaling::Wave` doc: light and slide waves E-7.20 with S_j, heavy waves E-7.20b with H_j. |

### Edits outside this area's files (ruling 5)

- `data/fixtures/mini/weapons/exalted.json`: Glory's `exalted` in the new M-1 form (the only fixture user; without it
  the fixture no longer parses).
- `data/fixtures/README.md`: rule 3 table, `melee.finishers` keyed by (`class`, `weapons`) (DC-20).
- `crates/wfd-model/tests/documents.rs`: constructions follow the new fields and types (M-1, M-8, M-14, M-15, M-17,
  API-5, DC-8, DC-14), plus the `build_spend: {}` and polarity-`null` checks. Tests that pinned the old contract and
  were updated to the new one: the bundle sample's `ExaltedScaling::Partial(0.5)`, `party_scaling: Some([..])`,
  `BuildSpend { k: 6.0 }` and `{ k: 12.0 }`, and the polarity maps.
- `crates/wfd-model/tests/fixtures.rs`: `load_mini` sets `retired_keys`; `LIST_KEYS` finishers key (DC-20); new test
  `set_counts_follow_tv_09_09` (API-6).
- `crates/wfd-model/src/lib.rs` (`pub mod sets;` and the crate doc), `crates/wfd-model/src/frame.rs` (DC-14).
- `frontend/src/schema/visibility.ts` and `frontend/src/schema/SchemaControl.test.tsx` (T03b-R5).
- Regenerated, not edited: `frontend/src/api/generated/**` (`cargo xtask gen-ts`), `data/curated/schema.json`
  (`cargo xtask gen-schema`: the new flag names and two doc strings).

### For model-core (the merge reconciles)

1. M-6, DC-18: once `curated.rs` defines the override-file types, add to `api/codegen.rs` `CURATED_FILE_SCHEMAS` one
   row per file pattern, `("overrides/weapons/**", "WeaponOverrideFile")`, `("overrides/attacks/**",
   "AttackOverrideFile")`, `("overrides/enemies/**", "EnemyOverrideFile")`, `("overrides/progenitor.toml",
   "ProgenitorFile")`, `("aliases/enemies.toml", "EnemyAliasFile")` (with the names model-core chooses), one
   `g.subschema_for::<…>()` line each in `curated_schema_json`, then `cargo xtask gen-schema`.
2. The weapon override type should be able to author what this pass added to `Weapon`/`Attack`: `exalted`
   (`ExaltedScaling`, Table 4.3-A per weapon), `Attack.exalted_s`, `Attack.compression_eff` with
   `Attack.compression_mode`, `ZoomLevel.enables`, and the role `stun_gated`.
3. Curated enemy data: `Enemy.bolstered_og_factor` and the flag `guardian_aura_immune` need an authoring field;
   `curated.rs`'s `party_scaling: Option<[f64; 4]>` may become `Option<Vec<f64>>` to match `Enemy.party_scaling`
   (T18 converts otherwise).
4. Curated frame data (`rules/frames.toml` or wherever frames are curated): an effect list for `Frame.passives`
   (DC-14), and the §16.8 Toxic Lash encoding without `status_duration_add`.
5. M-18: an optional `toggle: Option<ToggleId>` on `ItemKind::CustomStat` would let the server fill
   `CustomStatSchema.toggle` from data instead of matching the inspiration key `abilityCombo`.
6. domain-model.md §6 (this area's section) shows `Item`, `ItemKind`, `SetDef` and `RivenTraitDef`: model-core's
   additions to `item.rs` in this pass (DC-16 `ability_source`, API-12 `vintage_companion`, …) need their lines there
   at the merge.
7. API-6: `sets::set_counts` reads `Item.set` and `SetDef.noncounting`; DC-26's curated `noncounting` feeds the
   latter.

### For the other areas

- crates (T04, T18 code): `legality::set_counts` delegates to `wfd_model::sets::set_counts` and its doc loses "when
  `companion_kind` allows them" (API-6); `GameData::is_retired` reads `Bundle.retired_keys`, and the in-app refresh
  (`StableKeySource::Embedded`) reads both key lists (API-5); `legality::TraitRange`/`TraitRole` may reuse
  `api::{RivenTraitRange, TraitRole}`; the LINT-GLAIVE lint can enumerate `MeleePlanKind::ALL`.
- engine: `Engine::timeline` takes the seed and `Engine::units` returns units in `CalcReport.groups` order, as the
  `TimelineRequest` and `GroupRef` docs now state (API-1, ruling C-32).
- manager / T41: optimizer.md §1/§7 and `wfd-opt/src/presets.rs` docs for the M-10/API-9 rule, §6.2 step 6 for
  `OptResult.riven_sets`, §7 step 4 for `RankedBuild.capacity`/`violations`, and the `quality` definition (API-12).
  frontend.md (PAR-16 range hint) can point at `CalcReport.riven`.
- T18: emits `Weapon.exalted` from Table 4.3-A (Noctua's active attack `exalted_s`), `compression_mode`,
  `ZoomLevel.enables`, `stun_gated` on Zymos "Headshot Explosion", `guardian_aura_immune` on the Demolisher family,
  `bolstered_og_factor` (Techrot Babau 0.25), `Frame.passives`, `Bundle.retired_keys`.

### Checks

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`cargo xtask spec-sync --check`, `gen-ts --check`, `gen-schema --check`, `doc-check` pass; `npm --prefix frontend run
lint`, `typecheck` and `test -- --run` pass. `cargo xtask lint-engine` reports the two known "Glaive" problems
(`compile/melee.rs:30`, `mc/keys.rs:36`), left to the crates fixer's LINT-GLAIVE lint change.

---

## 2026-11-27 · Contract review fixes · merge integration (model-core + model-records)

The integrator's reconciliation of the two model sections above (their "For model-records" and "For model-core" lists).

- `sets.rs` (API-6, written by both fixers): model-core's version is kept (it adds the member cap `N_S ≤
  |SetDef.members|`, the shared helpers `set_slot_items` and `count_pieces`, and a unit test); model-records' note on
  declared companion pieces is added to its doc. Model-records' TV-09-09 test in `tests/fixtures.rs` runs against it.
- `CompressionMode` (DC-8): both fixers defined one (`effect.rs` and `weapon.rs`). One enum remains, in `effect.rs`
  (`Per::CompressionEff`), with `Default = multiplies`, `PartialOrd`, `Ord`; `Attack.compression_mode` uses it. The
  reviewer's `Attack.compression: Option<Compression>` is not added (model-records kept `compression_eff` and added
  `compression_mode` beside it).
- `Frame.passives` (DC-14, added by both): one field; the doc joins both texts (source `rules/frames.toml`
  `FrameRuleRow.passives`).
- `api/codegen.rs`: `CURATED_FILE_SCHEMAS` and `curated_schema_json` gain `GameConstantsFile`, `WeaponOverrideFile`,
  `AttackOverrideFile`, `EnemyOverrideFile`, `ProgenitorFile`, `EnemyAliasFile`; `data/curated/schema.json`
  regenerated (M-6, DC-18, DC-19).
- `WeaponTraits::BOW` (bit 15): the bundle field that `WeaponOverrideFile.bow` maps onto (06 §4.3 bow list, M-6);
  generated TS regenerated.
- `EnemyRuleRow` (`rules/special_dr.toml`): `party_scaling` is `Option<Vec<f64>>` like `Enemy.party_scaling` (M-17),
  and `bolstered_og_factor: Option<f64>` is the authoring field of `Enemy.bolstered_og_factor`.
  `EnemyFlags::GUARDIAN_AURA_IMMUNE` is authorable through the existing `flags` field.
- domain-model.md §6: `ItemKind::Buff` shows `inspiration_key` and `ability_source`, `SetDef` shows `noncounting`,
  `RivenTraitDef` shows `coeff`, `sign`, `spliced`, `recipes`, `inspiration_key`, `vintage_companion`; §4
  `WeaponTraits` lists `PSEUDO_EXALTED`, `NECRAMECH`, `BOW`.
- Not done (needs an owner decision): authoring fields in `WeaponOverrideFile` / `AttackFireOverride` for
  `Weapon.exalted`, `Attack.exalted_s`, `Attack.compression_eff`/`compression_mode` and `ZoomLevel.enables`
  (model-records' "For model-core" 2; T18 takes the first three from Table 4.3-A and `W:Primary_Compression`, so an
  override may not be needed), and `ItemKind::CustomStat.toggle` (M-18, optional).

---

## 2026-11-27 · Contract review fixes (data)

Area: `data/curated/rules/**` (T03b, T03c tables), `data/fixtures/**` and `crates/wfd-model/tests/{fixtures,
rules_tables,melee_tables}.rs`. Input: the data-area parts of `docs/project/inputs/contract-review-pass1.json` items M-2,
M-6, M-18, API-12, API-18, DC-1, DC-14, DC-16, DC-18 … DC-20, DC-22, DC-23, DC-25, DC-27, C-37, C-38, T02c-R2, T02c-R5,
T03b-R3, T03b-R5, T03b-R6, under the manager's rulings above, built on the merged model fixes (model-core,
model-records) and checked against `docs/mechanics/` as the contract-review spec round left it (fc67d67). The fixture
edits are recorded as a script, `data/fixtures/tools/history/contract_review_2026_11_27.py` (README.md "Tools"), so they
can be audited; `manifest.json` `spec_rev` is fc67d67 and `data_version` `fixture-mini.3`. No model type changed; two
doc comments did (below).

### Changes per item

| Id | Change (files) or reason |
|---|---|
| M-2 | Fixture custom stat `wfd:custom/heatinheritfaction` (key 3726, `inspiration_key` `heatInheritFaction`, `stat = "heat_inherit_faction"`, `value = { param = "value" }`; 00 §6.3, 03 §10.3, TV-03-33), name and stable key; 27 custom stats. domain-model.md §17: "Custom stats (27)", and the row "Heat inherit (D-028) and the inherited faction bonus F_inh (03 §10.3)" with tokens `heat_inherit`, `heat_inherit_faction` (COVERAGE.md and `encoding_check` follow). |
| M-6, DC-18 | No override file is a data-area file (owners T18, T20-T22, T29, work-breakdown.md §8 as the crates fixer amended it). `rules/special_dr.toml` header: the 05 §4.3 part rows, Scaldra `part_dr` and Juggernaut part immunity go to `overrides/enemies/*.toml` (`EnemyOverrideFile`, owner T29, read by T18), not "(T18)". |
| M-18 | Fixture `wfd:custom/abilitycombo` cites the alias rule (domain-model.md §9.3 T-26: resolve moves it into `toggles.ability_combo`); it stays `unmodelled` (an item without effects, `EffectStatus::Unmodelled` doc). `ItemKind::CustomStat.toggle` was not added by the model merge, so the server marks the alias by its `inspiration_key` `abilityCombo` (for T34/T04). |
| API-12 | `rules/riven_traits.toml`: `vintage_companion = true` on the nine companion vintage traits of 09 §12.3 (`zoom`, `recoil`, `ammo_maximum`, `combo_duration`, `critical_chance_slide`, `finisher_damage`, `heavy_attack_efficiency`, `initial_combo`, `chance_to_gain_combo_count`; `W:Riven_Mods` L921-922); header documents the field. Fixture `zoom` carries it too. Test `rules_tables::riven_traits` pins the set and that no spliced trait is flagged. |
| API-18 | No data part beyond a README line: `scenarios/real/*.json` are the documents CLI and smoke checks use; optimizer request fixtures live with their crate (`crates/wfd-opt/tests/fixtures/`, T33). The work-breakdown.md change is the crates fixer's. |
| DC-1 | Fixture Melee Animosity: `per_event = "any_input"`. domain-model.md §17 row "Counters per input …" gains the token `per_event = "any_input"` (checked). |
| DC-14 | Fixture Toxic Lash without `status_duration_add`. Fixture frames Saryn (8003; `passives = [{ stat = "status_dur", value = 0.25 }]`; its Toxic Lash ability's `self_buff` is `wfd:buff/toxic_lash`) and Rhino (8002, below). `rules/frames.toml`: `[[frame.passive]]` rows for Ash (`status_dur_type.SLA` 0.50, `status_dmg_type.SLA` 0.25; 03 §5, TV-03-36), Lavos (`status_dur` 1.00 × `ability_duration`) and Saryn (`status_dur` 0.25), on the base rows (new for Ash, Lavos, Saryn) and copied on their Prime rows ("Variants carry their own list", `Frame.passives`); sources 04 §5.11, `W:Universal_Weapon_Bonuses` L99-103, `W:Ash/Abilities`, `W:Saryn/Abilities/Passive`. Not rows (header says why): Dante (04 §5.6: a `sc_vuln` factor behind a user flag no input carries), and the other 04 §5.6 passives (T25). domain-model.md §17 row "Frame passives (04 §5.11, E-4.14)" (`Frame.passives`; Saryn with Toxic Lash). Tests: `rules_tables::helminth_and_frames` (values, variants equal their base, unconditional), `fixtures::contract_review_encodings`. |
| DC-16 | Fixture: `ability_source` on every buff per 04 §4.2 and §5.7 "Source kind" (true: Roar, ally Roar, Eclipse, Electric Shield, Molecular Prime, Nourish, Piercing Navigator (Ivara augment), Sonar, Terrify, Toxic Lash, Xata's Whisper; false: flat crit chance (T-18 input), Magus Accelerant, Mecha Empowered, Phoenix Talons, Smeeta Charm). 04 §4.2 marks the §5.6 buffs too (the gate reads only target debuffs, so it changes nothing there). Docs of `ItemKind::Buff.ability_source` and `CuratedItemFile.ability_source` cited "OQ-523: Source names a Warframe or Helminth"; they now cite 04 §4.2 (below). domain-model.md §17 row "Warframe-ability debuffs against `ability_debuff_immune` (05 §13.1)" (`ability_source`; Molecular Prime and Terrify gated, Magus Accelerant not). |
| DC-19 | New `rules/game_constants.toml` (T03b's, named by the crates fixer): E-5.4 `steel_path_mult` 2.5, `archon_empowered_base` 2, `archon_empowered_per_player` 0.5, `icebind_base` 3, `icebind_per_player` 1; E-5.5 the k_exi points `k_exi_d0..d4` = 15, 25, 35, 50, 100 and `k_exi_k0..k4` = 1, 1.25, 2.5, 4.5, 6 (piecewise linear, constant outside; the slopes are 05's), `c_exi_defended` 0.25, `c_exi_undefended` 0.375, `health_offset` 900, `base_floor_mult` 1.1; each row cites 05 and `G:slider` / the wiki. Keys `"<equation id>.<name>"`; a stage that needs another game value asks T03b for a row. Fixture `game_constants` equals it. Tests: `rules_tables::game_constants` (key form, sources, E-5.5 slopes), the OWNED list, `every_file_cites_sources`. |
| DC-20 | Applied by model-records (LIST_KEYS and README key (`class`, `weapons`)); verified. Added: `fixtures::rule_rows_follow_the_curated_tables` runs `check_list_keys` over the curated rows whose authoring row is the bundle row (multiplier groups, toggles, riven traits, slot layouts, special DR, faction heads, Helminth, melee heavy_class / attack_triggers / stance_co / finishers), so the real 9-E table passes the extension key rule. |
| DC-22 | Fixture Piercing Navigator `stacks.default_count = 0`; Arcane Crepuscular "+3x" bound as `n2`. |
| DC-23 | (a) fixture `molecular_prime` and `custom_vuln` without type lists (every type) and `dot = none` (with T03b-R6); (b) the fixture's 15 toggle descriptors are the `rules/toggles.toml` rows (T-01 max 600, T-07 and T-20 `percent`, T-13/T-15 max 10, T-24 and T-25 `custom` with T-24's v10 bounds and null default, groups, labels `toggle.<id>.label`); (c) slot layouts are the curated rows (primary and melee gain `inc2`-`inc5`; melee slot order as curated); the Braton, Laetum, Phenmor and Kunai perks (families 1-4) are documented as engine-only fixtures (README, SOURCES.md); (d) the body-part fallback row; (e) Helminth restricted list complete (Eclipse, Roar, Xata's Whisper) and the fixture frame Rhino (8002: abilities from the Public Export / WFCD, Roar's `self_buff` and `helminth_host` = `wfd:buff/roar`) with its Host row. Riven traits gain `inspiration_key`, `pe_tag` (and Zoom's `vintage_companion`) of the curated rows; Hemocyte `mdps_frac` 0.20. Test `fixtures::rule_rows_follow_the_curated_tables` keeps every such row equal to its curated row (citations, and the riven recipes the fixture cannot carry, aside). |
| DC-25 | `rules/mult_groups.toml`: the unused `devouring_attrition` group removed (the reviewer's first option: no `mult` encoding of the ×21 remains to be misused; `random_multiplier` places its factor in G itself, domain-model.md §16.13); the header says so; `rules_tables::multiplier_groups` asserts the absence. |
| DC-27 | No fixture change (the rule keeps bindings in the bundle form, as the fixture does); README "File shapes" states it. |
| C-37 | Fixture Melee Doughty: `stat = "doughty_rank"`, `value = { max = 1.0, rank = { table = [0.5 … 1.0] } }` (b_r), no `per`, status `verified` (the encoding is exact now that the crit stage computes 02 E-2.5a). Test `spec_encodings_of_the_review_fixes` pinned `draft` and `cd_abs` and now pins the new contract. domain-model.md §17 row "Melee Doughty, a per-instance crit-damage term (02 E-2.5a)" (`doughty_rank`); the "Resolved-stat conditions" row's COVERAGE fixture is Primary Plated Round instead of Doughty. |
| C-38 | Fixture Theorem Demulcent: four effects (HEA, COL, ELE, TOX), each `order = "warframe_arcane"`, `when = in_zone`, `value = { param = "<elem>_bonus" }`, and four `percent` params 0 … 1.80, default 0 (04 §5.5 as the spec round worded it; 01 §4.10.1: a zero holds no position); status `verified`. TV-01-38 reads `tox_bonus = 1.80`. |
| T02c-R2 | No part in data files (work-breakdown.md §8, crates fixer). |
| T02c-R5 | README: without `WFD_DATA_DIR` and `data/generated/`, `wfd-data/build.rs` embeds `mini/` with a `cargo:warning`; `data_version` starts with `fixture-` so `xtask dist` refuses the bundle (checked in `contract_review_encodings`). |
| T03b-R3 | As the spec round resolved it (05 §4.5): `corpus:Protea Specter` (the Deadlock Protocol quest boss, `M:enemies/corpus` L7150-7164) joins the `attenuation_unknown` row; "Coda Duets" are the members' spawn form, documented on the Technocyte Coda data-gap row. Spot check in `special_dr_rows`. |
| T03b-R5 | `rules/toggles.toml` T-36 is `custom` (its `opt.` field is written only by the optimizer page), as the `ToggleDescriptor.field` doc already says; the frontend change was model-records'. `toggle_controls_match_their_values`: the custom list (T-36 in, T-08/T-09 out) and "every `opt.` field is custom". |
| T03b-R6 | Ruling applied: fixture target_vuln groups `dot = none`, T-20 `percent`, T-24 and T-25 `custom` (DC-23 (b)); `rules/toggles.toml` T-08 and T-09 are `optional_number` with the one-member-object convention documented on `ControlType::OptionalNumber` (and in the toggles header and server-api.md §5.6), so T12's tested `NUMBER_MEMBER` binding stays. The test checks the two fields serialize as one-member objects and `{}` gives the model defaults. |

Beyond the listed items (the "new model fields of this pass" the rule tables need): `rules/special_dr.toml` sets
`guardian_aura_immune` (M-16) on the 22 Demolisher-family records (both Demolisher rows; 05 E-5.27, `W:Eximus` L116)
and `bolstered_og_factor = 0.25` (M-17) on both Techrot Babau records (05 §4.1, E-5.7, `W:Overguard` L149); the header
lists the flag and the field; `special_dr_rows` checks both and gained the field in its once-per-enemy list.

### Edits outside the area's files (ruling 5)

- `crates/wfd-model/src/rules.rs`: `ControlType` docs (the one-member-object convention on `OptionalNumber`, type doc,
  `Custom`'s example T-24 instead of T-08). Ruling T03b-R6 puts the convention there. Regenerated:
  `frontend/src/api/generated/ControlType.ts`, `data/curated/schema.json`.
- `crates/wfd-model/src/item.rs` (`ItemKind::Buff.ability_source`) and `curated.rs` (`CuratedItemFile.ability_source`):
  the docs cite 04 §4.2 instead of the closed OQ-523 wording, so the field's contract is the one the data follows.
  `data/curated/schema.json` regenerated.
- `docs/architecture/server-api.md` §5.6: one paragraph on the `optional_number` one-member object.
- `docs/architecture/domain-model.md` §17: three rows added (Melee Doughty, Warframe-ability debuffs, Frame passives),
  three changed (Counters: `per_event = "any_input"`; Custom stats (27); Heat inherit with `heat_inherit_faction`), as
  model-core's "For other areas" asked once the fixture had an item for each; each new row needs its hand-computed value
  in `wfd-engine/tests/effects_coverage.rs` (T08, testing.md §3).
- Frontend checks (`npm --prefix frontend ci`, `lint`, `typecheck`, `test -- --run`) pass with the regenerated TS.
- Not edited: pipeline.md and work-breakdown.md. The crates fixer's section already describes `rules/game_constants.toml`
  (owner T03b), the `[[frame.passive]]` rows and the fifth spec table; editing the same passages here would only
  conflict at the merge.

### For other areas

- Engine (T06, T14, T07): apply `Frame.passives` of the build's frame; T-16 implies Saryn, found without naming it (I-2)
  as the frame whose ability `self_buff` is the T-16 buff (fixture Saryn; T18 fills `self_buff` from data). T07 reads
  the E-5.4 / E-5.5 values from `Rules.game_constants` under the keys above, `Enemy.bolstered_og_factor` and
  `EnemyFlags::GUARDIAN_AURA_IMMUNE`.
- Frontend (T12, T30, T31): `frontend/src/i18n/en/schema.json` keys are `toggle.t01.label`, while the real
  `rules/toggles.toml` (and now the fixture) use `toggle.T-01.label` and `toggle.T-01.opt.<value>` as frontend.md §8
  names them; the catalogue should follow the data.
- T18: emit `Frame.passives` from `rules/frames.toml`, `Rules.game_constants` from `rules/game_constants.toml`, the
  special-DR rows' `bolstered_og_factor` and flags; the fixture and the curated tables now agree row for row.
- T25: Dante's passive (a `sc_vuln` group plus a user flag, default on when Dante is the frame), Excalibur's
  Swordsmanship and the other 04 §5.6 passives.
- Manager / spec round: (1) custom p.k: 00 §6.3 and 04 §10 now put `status_chance_mult` on the target side (`sc_vuln`,
  M_sc,t), but the fixture item `wfd:custom/status_chance_mult` still targets `sc_mul` (M_sc,w), the only `sc_mul` user
  of the fixture; moving it needs a `sc_vuln` group in `mult_groups.toml` and another `sc_mul` fixture user (Mandonel
  field) for the §17 `sc_*` row, so it is left for an owner (DC-12 had no data part in this pass). (2) Prelude of
  Might: 02 E-2.5 / 04 §5.12 now name CDM0c `cd_base_cond` with threshold θ_P; the registry has no such row and the
  fixture keeps `cd_base_add` gated by `Cond::Attack` (C-36 follow-up). (3) 09 §12.3: `W:Riven_Mods` L921 adds that the
  splices of the vintage traits are "also unobtainable on companion Rivens" (Reload While Holstered, Melee Damage On
  Heavy Attack and Heavy Attack Wind Up Speed have no recipe without one); 09 does not say so, so no spliced trait
  carries `vintage_companion`.

### Checks

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`cargo xtask spec-sync --check`, `gen-ts --check`, `gen-schema --check` and `doc-check` (6 warnings, none on lines this
change wrote) pass; `npm --prefix frontend run lint`, `typecheck` and `test -- --run` pass. `cargo xtask lint-engine`
reports the two known "Glaive" problems (`compile/melee.rs:30`, `mc/keys.rs:36`), LINT-GLAIVE being the crates
fixer's; the fixture's new names (Rhino, Saryn and their abilities) add no finding.
