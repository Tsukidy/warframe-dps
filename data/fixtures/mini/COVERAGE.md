# Coverage of the mini fixture bundle

Every row of domain-model.md §17 (column 1, verbatim) and the fixture records that encode it (column 2,
uniqueNames). `crates/wfd-model/tests/fixtures.rs` checks that the rows equal the §17 rows and that every named
record exists in the bundle (`every_coverage_row_has_a_fixture`), and it checks every backticked token of the §17
Encoding column against the bundle and the scenarios (`every_coverage_encoding_is_exercised`: `cc_*` needs all of
`cc_rel`, `cc_abs`, `cc_floor`, `cc_base_add`, `cc_final_mul`; `HeavyDef` needs a heavy with σ ≠ 1; `SetDef` needs a
count-based bonus and a member scale; and so on). Scenario-only parts of a row live in `../scenarios/`: T-06 external
Viral and T-07 external strip with an ability strip and an ally caster
(`gorgon_heavy_gunner_l150_strip_viral.json`), the zoom level (`rubico_eximus_heavy_gunner_l58_overguard.json`), the
melee plan (`nikana_prime_blind_justice_light_loop.json`), the progenitor (`kuva_karak_toxin_kuva_trokarian_l100.json`);
`rules.json` holds a descriptor for every toggle a scenario sets: each `toggles.*` field that differs from its
default, and each buff `caster` (T-33 for a caster Strength or Duration other than 1.0 and for the caster of a
non-T-13 buff, T-37 for the caster of the T-13 Roar), checked by `toggle_descriptors_match_the_toggle_map_and_the_scenarios`.
`manifest.json` `coverage` (item count per kind and `EffectStatus`) is computed from the item files
(`manifest_coverage_counts_the_items`; `../tools/coverage.py`).

Four encoding tokens are not fixture data, and the test lists them by name:

- `DotOnly`: no sourced catalogue entry is a member of a `DotOnly`-stage group. The status-damage vulnerability
  V_sd (Burning Hate, Pageflight, custom p.o) is the `dot_vuln` slot (`wfd:custom/vuln_status_damage`, 03 §9.4), and
  the Mecha and Vauban second dips are `dot = "twice"` G groups (`wfd:buff/mecha_empowered`).
- `scope.hit.outcome`: since D-410 no 04 catalogue entry uses the hit-outcome gate (Overwhelming Attrition is the
  `on_plain_hit` buff now, domain-model.md §11.1 and §17); the construct is covered by unit tests in `effect.rs`.
- `assemble`: an engine function (kitgun and zaw assembly); its data are the `ModularTables` of the row.
- `Cond::Attack`: no catalogue user in the fixture. Prelude of Might, its user until contract review pass 3, is the
  registry rows `cd_base_cond` and `cd_base_cond_thr` of 02 E-2.5 (ENG-R7); the attack-role filter of Pain Points
  (DC-10) needs the Athodai (T21). Both predicate forms are covered by `wfd-model/tests/domain_examples.rs`.

| Spec mechanic (domain-model.md §17) | Fixtures |
|---|---|
| Base bucket A incl. negatives (00 §3.1a) | `/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod`, `wfd:custom/base` |
| Melee-projectile second bucket (04 §3.4) | `/Lotus/Upgrades/CosmeticEnhancers/Utility/GolemArcaneBonusDamageOnWallLatch`, `/Lotus/Upgrades/Mods/Melee/WeaponGlaivePowerthrowMod`, `/Lotus/Weapons/Tenno/Melee/Glaives/LightGlaive/LightGlaiveWeapon` |
| Physical P_t, stance physical | `/Lotus/Upgrades/Mods/Rifle/WeaponImpactDamageMod`, `/Lotus/Weapons/Tenno/Melee/MeleeTrees/KatanaCmbThreeMeleeTree` |
| Elements with slot order, riven bottom-up, exilus, innate HCET | `/Lotus/Upgrades/Mods/Rifle/WeaponToxinDamageMod`, `/Lotus/Upgrades/Mods/Rifle/WeaponElectricityDamageMod`, `/Lotus/Weapons/Grineer/KuvaLich/LongGuns/Karak/KuvaKarak`, `wfd:shard/violet/primary_electricity`, `/Lotus/Upgrades/CosmeticEnhancers/Offensive/WeaponDamageOnResidualContact` |
| Non-combining and already-combined elements | `wfd:buff/nourish`, `/Lotus/Upgrades/Mods/Nightwave/NightwaveTC2024AK47AugmentMod` |
| Progenitor V_t (T-24) | `/Lotus/Weapons/Grineer/KuvaLich/LongGuns/Karak/KuvaKarak` |
| Incarnon flat X, excluded from B_ref | `wfd:perk/furis/haven_foray`, `/Lotus/Weapons/Tenno/Pistol/AutoPistol` |
| Exalted Strength scaling | `/Lotus/Powersuits/Choir/ChoirEruptWeapon` |
| Faction F, sub-factions, Roar, riven faction | `/Lotus/Upgrades/Mods/Rifle/Expert/PrimedWeaponFactionDamageGrineer`, `wfd:enemy/grineer/kuva_trokarian`, `wfd:buff/roar` |
| CO adding / multiplying / none, ρ_co, stance ratio | `/Lotus/Upgrades/Mods/Rifle/WeaponStatusChanceSPMod`, `/Lotus/Upgrades/Mods/Melee/WeaponDamageIfVictimProcActive`, `/Lotus/Weapons/Corpus/LongGuns/CrpShotgun/CrpShotgun`, `/Lotus/Weapons/Tenno/Bows/HuntingBow`, `/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana` |
| CO per Cold stack; CO under a health threshold | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/SecondaryExtraDamagePerFreezeStack`, `wfd:perk/kunai/swift_conclusion` |
| G_F and G multipliers, additive pairs | `wfd:buff/eclipse`, `/Lotus/Upgrades/Mods/Rifle/EventSniperReloadDamageMod`, `/Lotus/Upgrades/Mods/Rifle/SniperReloadDamageMod` |
| First and last shot | `/Lotus/Upgrades/Mods/Rifle/EventSniperReloadDamageMod`, `/Lotus/Upgrades/Mods/Sets/Synth/PistolSynthChargeMod` |
| Heavy-only, slam, finisher | `/Lotus/Upgrades/Mods/Melee/WeaponMeleeHeavyDamageMod`, `/Lotus/Upgrades/Mods/Melee/WeaponMeleeStealthLethalMod`, `/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana`, `/Lotus/Upgrades/Mods/Sets/Nira/NiraSetMod`, `/Lotus/Upgrades/Mods/Sets/Nira/NiraMeleeMod` |
| Body part ΣWP (1.5× on head), ΣHS, Sonar | `/Lotus/Upgrades/Mods/Rifle/WeaponWeakpointCriticalChanceMod`, `/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnNoMelee`, `wfd:buff/sonar` |
| CC relative, absolute, floor, base add, custom final factor | `/Lotus/Upgrades/Mods/Rifle/WeaponCritChanceMod`, `/Lotus/Upgrades/CosmeticEnhancers/Defensive/CritChanceOnDamage`, `wfd:buff/smeeta_charm_crit`, `wfd:buff/piercing_navigator`, `wfd:custom/mult_crit_chance`, `wfd:perk/braton/survivors_edge` |
| Combo-scaled CC/SC, Gladiator | `/Lotus/Upgrades/Mods/Melee/Event/ComboCritChanceMod`, `/Lotus/Upgrades/Mods/Melee/Event/ComboStatusChanceMod`, `/Lotus/Upgrades/Mods/Sets/Gladiator/GladiatorSetMod` |
| CD relative, absolute before tiers (D-014), base add, field factor | `/Lotus/Upgrades/Mods/Rifle/WeaponCritDamageMod`, `/Lotus/Upgrades/CosmeticEnhancers/Utility/AbilityStrengthAndCritDamageWhenInvisible`, `wfd:perk/braton/critical_parallel`, `wfd:perk/furis/prelude_of_might`, `wfd:buff/electric_shield`, `wfd:buff/mandonel_field` |
| Melee Doughty, a per-instance crit-damage term (02 E-2.5a) | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/CritDamageForPunctureStatus` |
| Tier raise (Vigilante) | `/Lotus/Upgrades/Mods/Sets/Vigilante/VigilanteSetMod` |
| SC relative, absolute, base add, factor | `/Lotus/Upgrades/Mods/Rifle/WeaponStunChanceMod`, `wfd:custom/absolute_status_chance`, `wfd:buff/mandonel_field`, `wfd:custom/status_chance_mult` |
| Forced procs, Hunter Munitions, Internal Bleeding | `/Lotus/Weapons/Tenno/LongGuns/SapientPrimary/SapientPrimaryWeapon`, `/Lotus/Upgrades/Mods/Sets/Hunter/PrimaryHunterMunitionsMod`, `/Lotus/Upgrades/Mods/Rifle/WeaponBleedOnImpactProcRifleMod` |
| Multishot and Acuity lock | `/Lotus/Upgrades/Mods/Rifle/WeaponFireIterationsMod`, `/Lotus/Upgrades/Mods/Rifle/WeaponWeakpointCriticalChanceMod` |
| Status damage global and per type | `/Lotus/Upgrades/Mods/Rifle/RifleStatusDamageDuoMod`, `wfd:shard/emerald/toxin_status_damage` |
| Status duration and caps | `/Lotus/Upgrades/Mods/Melee/WeaponProcTimeMod`, `wfd:shard/emerald/corrosive_stack` |
| DoT-only multipliers M_dot | `wfd:custom/vuln_status_damage`, `wfd:buff/mecha_empowered` |
| Extra hits | `wfd:buff/xatas_whisper`, `wfd:buff/toxic_lash` |
| Melee Duplicate, Devouring Attrition | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/ReplayLightCritHits`, `wfd:perk/phenmor/devouring_attrition` |
| Hit-outcome gate | `wfd:perk/laetum/overwhelming_attrition` |
| Stat from a resolved stat with a power curve, reloaded fraction | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamagePerAmmoOnReload` |
| Counters per input or throw, consumed by a heavy or another input; n events within a window | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/HeavyCritOnRegularMelee`, `/Lotus/Upgrades/Mods/Melee/WeaponGlaivePowerthrowMod`, `wfd:perk/phenmor/lingering_judgement` |
| Vulnerability per layer and type (multiplicative and additive group) | `wfd:buff/molecular_prime`, `wfd:custom/dmgvulnerability`, `wfd:buff/magus_accelerant`, `/Lotus/Upgrades/CosmeticEnhancers/Offensive/OrbsOnResidualContact` |
| Critical promotion on body parts | `wfd:enemy/themurmur/rogue_culverin` |
| Armor strip (ability, aura), external Viral | `/Lotus/Upgrades/Mods/Aura/EnemyArmorReductionAuraMod`, `wfd:buff/terrify` |
| Warframe-ability debuffs against `ability_debuff_immune` (05 §13.1) | `wfd:buff/molecular_prime`, `wfd:buff/terrify`, `wfd:buff/magus_accelerant` |
| Overguard-only multiplier | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/OverguardSteal` |
| Fire rate, lock, charge, magazine, reload, ammo efficiency | `/Lotus/Upgrades/Mods/Rifle/WeaponFireRateMod`, `/Lotus/Upgrades/Mods/Rifle/RifleSemiAutoFantasyMod`, `wfd:perk/furis/extended_volley`, `wfd:custom/reloadtime`, `wfd:custom/magazinesize`, `/Lotus/Upgrades/Mods/Archwing/Rifle/ArchwingRifleChargeSpeedMod`, `/Lotus/Upgrades/Mods/Pistol/Event/AkstilettoEfficiencyWhileAirborneMod` |
| Melee timing stats | `/Lotus/Upgrades/Mods/Melee/WeaponFireRateMod`, `/Lotus/Upgrades/Mods/Aura/PlayerMeleeStartingComboAuraMod`, `/Lotus/Upgrades/Mods/Melee/DualStat/FocusEnergyMod`, `/Lotus/Upgrades/Mods/Melee/WeaponMeleeHeavyDamageMod`, `/Lotus/Upgrades/Mods/Melee/DualStat/CorruptedHeavyDamageChargeSpeedMod` |
| Combo tables, unique overrides, strikes, synthesized heavy, plans | `/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana`, `/Lotus/Weapons/Syndicates/ArbitersOfHexis/Melee/AHBoltace`, `/Lotus/Weapons/Tenno/Melee/MeleeTrees/KatanaCmbThreeMeleeTree`, `/Lotus/Weapons/Grineer/Melee/GrineerMachetteAndCleaver/Machete` |
| Lingering children, delayed explosions | `/Lotus/Weapons/ClanTech/Bio/BioWeapon`, `/Lotus/Weapons/Corpus/Bow/Longbow/CrpBow`, `/Lotus/Weapons/Corpus/LongGuns/CrpRubanRifle/CrpRubanRifle` |
| Zoom levels | `/Lotus/Weapons/Tenno/LongGuns/FiveShotSniper/FiveShotSniper` |
| Sniper combo, beam ramp, spool, falloff, semi cap | `/Lotus/Weapons/Tenno/LongGuns/FiveShotSniper/FiveShotSniper`, `/Lotus/Weapons/Corpus/LongGuns/ChainLightningGun/ChainLightningRifle`, `/Lotus/Weapons/Tenno/Rifle/HeavyRifle`, `/Lotus/Weapons/Corpus/LongGuns/CrpShotgun/CrpShotgun` |
| Set bonuses (count-based, member scale) | `/Lotus/Upgrades/Mods/Sets/Vigilante/VigilanteSetMod`, `/Lotus/Upgrades/Mods/Sets/Gladiator/GladiatorSetMod`, `/Lotus/Upgrades/Mods/Sets/Sacrifice/SacrificeSetMod`, `/Lotus/Upgrades/Mods/Sets/Nira/NiraSetMod` |
| Resolved-stat conditions and stat-from-stat | `wfd:perk/furis/prelude_of_might`, `/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamagePerAmmoOnReload` |
| Target-dependent element | `/Lotus/Upgrades/Mods/Melee/Event/Nightwave/NightwaveVenatoAugmentMod` |
| Item parameters | `/Lotus/Powersuits/Excalibur/SwordOfDoomAugmentCard` |
| Custom stats (27) and toggles (T-01 … T-43) | `wfd:custom/abilitycombo`, `wfd:custom/absolute_status_chance`, `wfd:custom/base`, `wfd:custom/base_per_status`, `wfd:custom/crit_chance`, `wfd:custom/crit_chance_weakp`, `wfd:custom/crit_mult`, `wfd:custom/crit_mult_add`, `wfd:custom/crit_mult_mult`, `wfd:custom/dmgvulnerability`, `wfd:custom/flat_change_dmg`, `wfd:custom/flat_crit_chance`, `wfd:custom/headshot_mult`, `wfd:custom/heatadd`, `wfd:custom/heatinherit`, `wfd:custom/heatinheritfaction`, `wfd:custom/initialcombo`, `wfd:custom/magazinesize`, `wfd:custom/mult_crit_chance`, `wfd:custom/mult_for_head`, `wfd:custom/multishot`, `wfd:custom/reloadtime`, `wfd:custom/speed`, `wfd:custom/status_chance`, `wfd:custom/status_chance_mult`, `wfd:custom/status_damage`, `wfd:custom/vuln_status_damage` |
| Buff caster Strength | `wfd:buff/ally_roar` |
| Kitgun and zaw assembly | `/Lotus/Weapons/SolarisUnited/Secondary/SUModularSecondarySet1/Barrel/SUModularSecondaryBarrelAPart`, `/Lotus/Weapons/SolarisUnited/Secondary/SUModularSecondarySet1/Handle/SUModularSecondaryHandleAPart`, `/Lotus/Weapons/SolarisUnited/Secondary/SUModularSecondarySet1/Clip/SUModularCritICapIClipPart`, `/Lotus/Weapons/Ostron/Melee/ModularMelee01/Tip/TipOne`, `/Lotus/Weapons/Ostron/Melee/ModularMelee01/Handle/HandleOne`, `/Lotus/Weapons/Ostron/Melee/ModularMelee01/Balance/BalanceSpeedI` |
| Incarnon form attack and perks per form | `/Lotus/Weapons/Tenno/Pistol/AutoPistol`, `wfd:perk/furis/haven_foray`, `wfd:perk/furis/extended_volley`, `wfd:perk/furis/stormburst` |
| Heat inherit (D-028) and the inherited faction bonus F_inh (03 §10.3) | `wfd:custom/heatinherit`, `wfd:custom/heatinheritfaction` |
| Frame passives (04 §5.11, E-4.14) | `/Lotus/Powersuits/Saryn/Saryn`, `wfd:buff/toxic_lash` |
| Phoenix Talons (D-007) | `wfd:buff/phoenix_talons` |
