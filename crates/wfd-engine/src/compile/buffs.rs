//! Buffs (04 §4.4: caster, Strength and Duration of 04 E-4.10, T-13 … T-19, T-33, T-37) into `CompiledScenario.fixed`.
//!
//! The 05 §13.1 ability gate is applied here, when the buff is lowered in the scenario context (the item lowering of
//! domain-model.md §18 and §12.2, which knows the enemy; contract review M-3, DC-16): a buff
//! whose `ItemKind::Buff.ability_source` is true contributes no target-side debuff against an enemy with
//! `EnemyFlags::ABILITY_DEBUFF_IMMUNE` (Eidolons, Archons in Archon Hunts, Entropic Eximus): its `vuln_add` effects,
//! its `mult` groups of stage `target_vuln` and its armor strips (`strip_ability`, `strip_points`) are dropped; its
//! other effects stay. Mods, arcanes, auras, focus and companions are never gated (`strip_aura`, `strip_companion`).
//! Owner: T06.
