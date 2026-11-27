# Coverage of the mini fixture bundle

Every row of domain-model.md §17 (column 1, verbatim) and the fixture records that encode it (column 2,
uniqueNames). `crates/wfd-model/tests/fixtures.rs` checks that the rows equal the §17 rows and that every named
record exists in the bundle. Toggle-only parts of a row (T-06 external Viral, T-07 external strip, the toggle set
itself) live in scenarios and `rules.json`, not in records. Rows whose §17 fixture names a weapon that is not in
the mini bundle are covered by another record of the same mechanic (for example Lenz for delayed explosions; the
Torid cloud is not included).

| Spec mechanic (domain-model.md §17) | Fixtures |
|---|---|
| Base bucket A incl. negatives (00 §3.1a) | `/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod`, `wfd:custom/base` |
| Melee-projectile second bucket (04 §3.4) | `/Lotus/Upgrades/CosmeticEnhancers/Utility/GolemArcaneBonusDamageOnWallLatch`, `/Lotus/Upgrades/Mods/Melee/WeaponGlaivePowerthrowMod` |
| Physical P_t, stance physical | `/Lotus/Upgrades/Mods/Rifle/WeaponImpactDamageMod`, `/Lotus/Weapons/Tenno/Melee/MeleeTrees/KatanaCmbThreeMeleeTree` |
| Elements with slot order, riven bottom-up, exilus, innate HCET | `/Lotus/Upgrades/Mods/Rifle/WeaponToxinDamageMod`, `/Lotus/Upgrades/Mods/Rifle/WeaponElectricityDamageMod`, `/Lotus/Weapons/Grineer/KuvaLich/LongGuns/Karak/KuvaKarak`, `wfd:shard/violet/primary_electricity` |
| Non-combining and already-combined elements | `wfd:buff/nourish`, `/Lotus/Upgrades/Mods/Nightwave/NightwaveTC2024AK47AugmentMod` |
| Progenitor V_t (T-24) | `/Lotus/Weapons/Grineer/KuvaLich/LongGuns/Karak/KuvaKarak` |
| Incarnon flat X, excluded from B_ref | `wfd:perk/furis/haven_foray`, `/Lotus/Weapons/Tenno/Pistol/AutoPistol` |
| Exalted Strength scaling | `/Lotus/Powersuits/Choir/ChoirEruptWeapon` |
| Faction F, sub-factions, Roar, riven faction | `/Lotus/Upgrades/Mods/Rifle/Expert/PrimedWeaponFactionDamageGrineer`, `wfd:enemy/grineer/kuva_trokarian`, `wfd:buff/roar` |
| CO adding / multiplying / none, ρ_co, stance ratio | `/Lotus/Upgrades/Mods/Rifle/WeaponStatusChanceSPMod`, `/Lotus/Upgrades/Mods/Melee/WeaponDamageIfVictimProcActive`, `/Lotus/Weapons/Corpus/LongGuns/CrpShotgun/CrpShotgun`, `/Lotus/Weapons/Tenno/Bows/HuntingBow`, `/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana` |
| CO per Cold stack; CO under a health threshold | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/SecondaryExtraDamagePerFreezeStack`, `wfd:perk/kunai/swift_conclusion` |
| G_F and G multipliers, additive pairs | `wfd:buff/eclipse`, `/Lotus/Upgrades/Mods/Rifle/EventSniperReloadDamageMod`, `/Lotus/Upgrades/Mods/Rifle/SniperReloadDamageMod` |
| First and last shot | `/Lotus/Upgrades/Mods/Rifle/EventSniperReloadDamageMod`, `/Lotus/Upgrades/Mods/Sets/Synth/PistolSynthChargeMod` |
| Heavy-only, slam, finisher | `/Lotus/Upgrades/Mods/Melee/WeaponMeleeHeavyDamageMod`, `/Lotus/Upgrades/Mods/Melee/WeaponMeleeStealthLethalMod`, `/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana` |
| Body part ΣWP (1.5× on head), ΣHS, Sonar | `/Lotus/Upgrades/Mods/Rifle/WeaponWeakpointCriticalChanceMod`, `/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnNoMelee`, `wfd:buff/sonar` |
| CC relative, absolute, floor, base add, custom final factor | `/Lotus/Upgrades/Mods/Rifle/WeaponCritChanceMod`, `/Lotus/Upgrades/CosmeticEnhancers/Defensive/CritChanceOnDamage`, `wfd:buff/smeeta_charm_crit`, `wfd:buff/piercing_navigator`, `wfd:custom/mult_crit_chance` |
| Combo-scaled CC/SC, Gladiator | `/Lotus/Upgrades/Mods/Melee/Event/ComboCritChanceMod`, `/Lotus/Upgrades/Mods/Melee/Event/ComboStatusChanceMod`, `/Lotus/Upgrades/Mods/Sets/Gladiator/GladiatorSetMod` |
| CD relative, absolute before tiers (D-014), base add, field factor | `/Lotus/Upgrades/Mods/Rifle/WeaponCritDamageMod`, `/Lotus/Upgrades/CosmeticEnhancers/Utility/AbilityStrengthAndCritDamageWhenInvisible`, `wfd:perk/furis/prelude_of_might`, `wfd:buff/electric_shield` |
| Tier raise (Vigilante) | `/Lotus/Upgrades/Mods/Sets/Vigilante/VigilanteSetMod` |
| SC relative, absolute, base add, factor | `/Lotus/Upgrades/Mods/Rifle/WeaponStunChanceMod`, `wfd:custom/absolute_status_chance`, `wfd:custom/status_chance_mult` |
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
| Vulnerability per layer and type (multiplicative and additive group) | `wfd:buff/molecular_prime`, `wfd:custom/dmgvulnerability`, `wfd:buff/magus_accelerant` |
| Critical promotion on body parts | `wfd:enemy/themurmur/rogue_culverin` |
| Armor strip (ability, aura), external Viral | `/Lotus/Upgrades/Mods/Aura/EnemyArmorReductionAuraMod` |
| Overguard-only multiplier | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/OverguardSteal` |
| Fire rate, lock, charge, magazine, reload, ammo efficiency | `/Lotus/Upgrades/Mods/Rifle/WeaponFireRateMod`, `/Lotus/Upgrades/Mods/Rifle/RifleSemiAutoFantasyMod`, `wfd:perk/furis/extended_volley`, `wfd:custom/reloadtime` |
| Melee timing stats | `/Lotus/Upgrades/Mods/Melee/WeaponFireRateMod`, `/Lotus/Upgrades/Mods/Aura/PlayerMeleeStartingComboAuraMod`, `/Lotus/Upgrades/Mods/Melee/DualStat/FocusEnergyMod`, `/Lotus/Upgrades/Mods/Melee/WeaponMeleeHeavyDamageMod` |
| Combo tables, unique overrides, strikes, synthesized heavy, plans | `/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana`, `/Lotus/Weapons/Syndicates/ArbitersOfHexis/Melee/AHBoltace`, `/Lotus/Weapons/Tenno/Melee/MeleeTrees/KatanaCmbThreeMeleeTree` |
| Lingering children, delayed explosions | `/Lotus/Weapons/Corpus/Bow/Longbow/CrpBow` |
| Zoom levels | `/Lotus/Weapons/Tenno/LongGuns/FiveShotSniper/FiveShotSniper` |
| Sniper combo, beam ramp, spool, falloff, semi cap | `/Lotus/Weapons/Tenno/LongGuns/FiveShotSniper/FiveShotSniper`, `/Lotus/Weapons/Corpus/LongGuns/ChainLightningGun/ChainLightningRifle`, `/Lotus/Weapons/Tenno/Rifle/HeavyRifle`, `/Lotus/Weapons/Corpus/LongGuns/CrpShotgun/CrpShotgun` |
| Set bonuses (count-based, member scale) | `/Lotus/Upgrades/Mods/Sets/Vigilante/VigilanteSetMod`, `/Lotus/Upgrades/Mods/Sets/Gladiator/GladiatorSetMod` |
| Resolved-stat conditions and stat-from-stat | `wfd:perk/furis/prelude_of_might`, `/Lotus/Upgrades/CosmeticEnhancers/Offensive/CritDamageForPunctureStatus` |
| Target-dependent element | `/Lotus/Upgrades/Mods/Melee/Event/Nightwave/NightwaveVenatoAugmentMod` |
| Item parameters | `/Lotus/Powersuits/Excalibur/SwordOfDoomAugmentCard` |
| Custom stats (26) and toggles (T-01 … T-43) | `wfd:custom/abilitycombo`, `wfd:custom/absolute_status_chance`, `wfd:custom/base`, `wfd:custom/base_per_status`, `wfd:custom/crit_chance`, `wfd:custom/crit_chance_weakp`, `wfd:custom/crit_mult`, `wfd:custom/crit_mult_add`, `wfd:custom/crit_mult_mult`, `wfd:custom/dmgvulnerability`, `wfd:custom/flat_change_dmg`, `wfd:custom/flat_crit_chance`, `wfd:custom/headshot_mult`, `wfd:custom/heatadd`, `wfd:custom/heatinherit`, `wfd:custom/initialcombo`, `wfd:custom/magazinesize`, `wfd:custom/mult_crit_chance`, `wfd:custom/mult_for_head`, `wfd:custom/multishot`, `wfd:custom/reloadtime`, `wfd:custom/speed`, `wfd:custom/status_chance`, `wfd:custom/status_chance_mult`, `wfd:custom/status_damage`, `wfd:custom/vuln_status_damage` |
| Buff caster Strength | `wfd:buff/ally_roar` |
| Kitgun and zaw assembly | `/Lotus/Weapons/SolarisUnited/Secondary/SUModularSecondarySet1/Barrel/SUModularSecondaryBarrelAPart`, `/Lotus/Weapons/SolarisUnited/Secondary/SUModularSecondarySet1/Handle/SUModularSecondaryHandleAPart`, `/Lotus/Weapons/SolarisUnited/Secondary/SUModularSecondarySet1/Clip/SUModularCritICapIClipPart`, `/Lotus/Weapons/Ostron/Melee/ModularMelee01/Tip/TipOne`, `/Lotus/Weapons/Ostron/Melee/ModularMelee01/Handle/HandleOne`, `/Lotus/Weapons/Ostron/Melee/ModularMelee01/Balance/BalanceSpeedI` |
| Incarnon form attack and perks per form | `/Lotus/Weapons/Tenno/Pistol/AutoPistol`, `wfd:perk/furis/haven_foray`, `wfd:perk/furis/extended_volley` |
| Heat inherit (D-028) | `wfd:custom/heatinherit` |
| Phoenix Talons (D-007) | `wfd:buff/phoenix_talons` |
