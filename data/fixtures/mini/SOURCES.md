# Sources of the mini fixture bundle

Status: hand-written test fixture (T03). Every record cites its source here; records whose type has a
citation field (`Weapon.provenance`, `Item.sources`, `StatusRule.source`, `MultGroupDef.sources`, …) carry
the same citation inline. Wiki citations name the data module, the revision timestamp of the cached copy
(`research/external/cache/wiki-modules/revs.json`) and the entry key. `pe:` is the DE Public Export
(`research/external/cache/public-export/manifests`). Spec citations (`00 §3.8`, `E-7.40`, `TV-05-05`) point
at `docs/mechanics/`. Nothing is taken from the inspiration bundle.

## Weapons

| Key | uniqueName | Name | Sources |
|---|---|---|---|
| 1212 | `/Lotus/Weapons/Corpus/Bow/Longbow/CrpBow` | Lenz | wiki:Module:Weapons/data/primary#Lenz @2026-11-14T12:42:25Z; 06 §9.3-9.4 (delayed explosion: Bubble Collapse ExplosionDelay 1.3 s); 04 §2.5 D3 (radial children of a projectile: none) |
| 1208 | `/Lotus/Weapons/Corpus/LongGuns/ChainLightningGun/ChainLightningRifle` | Amprex | wiki:Module:Weapons/data/primary#Amprex @2026-11-14T12:42:25Z; 06 E-6.15 (ramp start 0.2, 0.6 s); TV-06-07 |
| 1209 | `/Lotus/Weapons/Corpus/LongGuns/CrpShotgun/CrpShotgun` | Arca Plasmor | wiki:Module:Weapons/data/primary#Arca Plasmor @2026-11-14T12:42:25Z; 04 §2.14 row 7 |
| 1205 | `/Lotus/Weapons/Grineer/KuvaLich/LongGuns/Karak/KuvaKarak` | Kuva Karak | wiki:Module:Weapons/data/primary#Kuva Karak @2026-11-14T12:42:25Z; W:Progenitor (Kuva weapons 25-60 %); 01 E-1.2; T-24; 09 E-9.70 |
| 1206 | `/Lotus/Weapons/Tenno/Bows/HuntingBow` | Paris | wiki:Module:Weapons/data/primary#Paris @2026-11-14T12:42:25Z; 04 §2.14 row 59 (Paris Charged Shot adding 0.5 uncharged); Incarnon attack omitted from the fixture |
| 1210 | `/Lotus/Weapons/Tenno/LongGuns/FiveShotSniper/FiveShotSniper` | Rubico | wiki:Module:Weapons/data/primary#Rubico @2026-11-14T12:42:25Z; 06 E-6.16; TV-06-08; 04 D-406 (zoom) |
| 1207 | `/Lotus/Weapons/Tenno/LongGuns/SapientPrimary/SapientPrimaryWeapon` | Acceltra | wiki:Module:Weapons/data/primary#Acceltra @2026-11-14T12:42:25Z; 04 §2.14 row 1; 04 §2.5 D3 (radial child of a projectile: none); 06 E-6.23 |
| 1204 | `/Lotus/Weapons/Tenno/Rifle/BratonPrime` | Braton Prime | wiki:Module:Weapons/data/primary#Braton Prime @2026-11-14T12:42:25Z; 00 §3.8 (Normal Attack only; the Incarnon attacks are omitted from the fixture) |
| 1211 | `/Lotus/Weapons/Tenno/Rifle/HeavyRifle` | Gorgon | wiki:Module:Weapons/data/primary#Gorgon @2026-11-14T12:42:25Z; 06 §7.3 E-6.17 (Gorgon 9, 0.20, 0.10667); TV-06-06; Incarnon attacks omitted from the fixture |
| 1300 | `/Lotus/Weapons/Tenno/Pistol/AutoPistol` | Furis | wiki:Module:Weapons/data/secondary#Furis @2026-10-22T19:15:19Z; W:Furis_Incarnon_Genesis; 04 §2.14 row 119; 06 E-6.15 (Incarnon held form ramp); only the perks of the fixture are listed per tier |
| 1401 | `/Lotus/Weapons/Syndicates/ArbitersOfHexis/Melee/AHBoltace` | Telos Boltace | wiki:Module:Weapons/data/melee#Telos Boltace @2026-10-22T19:15:30Z; 07 E-7.0b (unique table by name: Telos Boltace); 07 E-7.40 rule 2, table 9-A Tonfa 5× (250 % Bleed + 250 % Bleed); 07 §9.7 Stormpath: data_attack_loop only |
| 1400 | `/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana` | Nikana Prime | wiki:Module:Weapons/data/melee#Nikana Prime @2026-10-22T19:15:30Z; 07 E-7.40 rule 2, table 9-A Nikana 5× Bleed / 5× Bleed (σ = 1); 07 §9.1 synthesized heavy |
| 1500 | `/Lotus/Powersuits/Choir/ChoirEruptWeapon` | Glory | wiki:Module:Weapons/data/secondary#Glory @2026-10-22T19:15:19Z; 01 Table 4.3-A (Glory on High, model 1, s = 75/100/125/150); 04 §2.5 D5b (standalone AoE: none) |
| 1601 | `/Lotus/Weapons/Ostron/Melee/ModularMelee01/Tip/TipOne` | Balla | wiki:Module:Weapons/data/modular#Balla @2026-10-24T14:30:08Z; wiki:Module:Modular/data#Zaw.Strike.Balla @2026-06-22T05:27:50Z; 07 E-7.42 (zaw assembly; class, FR and base from the parts, TV-07-07); 07 E-7.40 rule 2: stanceless Dagger heavy entries (2 direct strikes each, as many as table 9-A lists) scaled by σ = 5/5 = 1; typical wind-up 0.4 of table 9-A (the zaw data has no WindUp) |
| 1600 | `/Lotus/Weapons/SolarisUnited/Secondary/SUModularSecondarySet1/Barrel/SUModularSecondaryBarrelAPart` | Catchmoon | wiki:Module:Weapons/data/modular#Catchmoon (Secondary) @2026-10-24T14:30:08Z; wiki:Module:Modular/data#KitgunSecondary.Chamber.Catchmoon @2026-06-22T05:27:50Z; 04 §2.14 row 107; 06 §11.1 (assembled at evaluation time) |

## Items

| Key | uniqueName | Name | Status | Sources |
|---|---|---|---|---|
| 3342 | `/Lotus/Upgrades/Mods/Melee/DualStat/FocusEnergyMod` | Focus Energy | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Focus Energy; 07 §6.6 |
| 3332 | `/Lotus/Upgrades/Mods/Melee/Event/ComboCritChanceMod` | Blood Rush | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Blood Rush; domain-model.md §16.11; F:addmul §2; TV-07-02 |
| 3333 | `/Lotus/Upgrades/Mods/Melee/Event/ComboStatusChanceMod` | Weeping Wounds | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Weeping Wounds; 04 §5.3; W:Weeping_Wounds |
| 3344 | `/Lotus/Upgrades/Mods/Melee/Event/Nightwave/NightwaveVenatoAugmentMod` | Sentient Incision | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Sentient Incision; 01 §4.10.4; EC-01-39; D-102 |
| 3331 | `/Lotus/Upgrades/Mods/Melee/WeaponCritChanceMod` | True Steel | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#True Steel; domain-model.md §16.11 |
| 3334 | `/Lotus/Upgrades/Mods/Melee/WeaponDamageIfVictimProcActive` | Condition Overload | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Condition Overload; domain-model.md §16.3; F:addmul §1 |
| 3337 | `/Lotus/Upgrades/Mods/Melee/WeaponFireRateMod` | Fury | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Fury; 07 §5.2 |
| 3343 | `/Lotus/Upgrades/Mods/Melee/WeaponGlaivePowerthrowMod` | Power Throw | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Power Throw; 04 §3.4; EC-07-42; domain-model.md §16.13 |
| 3330 | `/Lotus/Upgrades/Mods/Melee/WeaponMeleeDamageMod` | Pressure Point | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Pressure Point |
| 3335 | `/Lotus/Upgrades/Mods/Melee/WeaponMeleeHeavyDamageMod` | Killing Blow | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Killing Blow; 04 §5.1; domain-model.md §16.13 |
| 3336 | `/Lotus/Upgrades/Mods/Melee/WeaponMeleeStealthLethalMod` | Covert Lethality | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Covert Lethality; 04 §3.2; W:Covert_Lethality |
| 3341 | `/Lotus/Upgrades/Mods/Melee/WeaponProcTimeMod` | Lasting Sting | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Lasting Sting; 03 §5 |
| 3345 | `/Lotus/Upgrades/Mods/Nightwave/NightwaveTC2024AK47AugmentMod` | Biotic Rounds | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Biotic Rounds; 01 §4.10.3 |
| 3350 | `/Lotus/Upgrades/Mods/Pistol/WeaponDamageAmountMod` | Hornet Strike | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Hornet Strike |
| 3351 | `/Lotus/Upgrades/Mods/Pistol/WeaponFireIterationsMod` | Barrel Diffusion | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Barrel Diffusion |
| 3311 | `/Lotus/Upgrades/Mods/Rifle/EventSniperReloadDamageMod` | Primed Chamber | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Primed Chamber; 04 §3.2; W:Primed_Chamber |
| 3307 | `/Lotus/Upgrades/Mods/Rifle/Expert/PrimedWeaponFactionDamageGrineer` | Primed Bane of Grineer | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Primed Bane of Grineer; 00 §3.8; 01 §4.15; D-008 |
| 3318 | `/Lotus/Upgrades/Mods/Rifle/RifleSemiAutoFantasyMod` | Semi-Rifle Cannonade | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Semi-Rifle Cannonade; 06 §5.1; EC-06-04; domain-model.md §16.13 |
| 3317 | `/Lotus/Upgrades/Mods/Rifle/RifleStatusDamageDuoMod` | Rifle Elementalist | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Rifle Elementalist; 03 §9.4 |
| 3312 | `/Lotus/Upgrades/Mods/Rifle/SniperReloadDamageMod` | Charged Chamber | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Charged Chamber; 04 §3.2; W:Charged_Chamber |
| 3316 | `/Lotus/Upgrades/Mods/Rifle/WeaponBleedOnImpactProcRifleMod` | Internal Bleeding | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Internal Bleeding; domain-model.md §16.5; 02 §4.7.3; D-070 |
| 3303 | `/Lotus/Upgrades/Mods/Rifle/WeaponCritChanceMod` | Point Strike | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Point Strike; 00 §3.8 |
| 3304 | `/Lotus/Upgrades/Mods/Rifle/WeaponCritDamageMod` | Vital Sense | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Vital Sense; 00 §3.8 |
| 3301 | `/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod` | Serration | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Serration; 00 §3.8; F:core §2 |
| 3306 | `/Lotus/Upgrades/Mods/Rifle/WeaponElectricityDamageMod` | Stormbringer | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Stormbringer; 00 §3.8; 01 §4.10 |
| 3302 | `/Lotus/Upgrades/Mods/Rifle/WeaponFireIterationsMod` | Split Chamber | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Split Chamber; 00 §3.8 |
| 3308 | `/Lotus/Upgrades/Mods/Rifle/WeaponFireRateMod` | Speed Trigger | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Speed Trigger; 00 §3.8; 06 §5.1 |
| 3309 | `/Lotus/Upgrades/Mods/Rifle/WeaponImpactDamageMod` | Rupture | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Rupture; 01 §4.6 |
| 3310 | `/Lotus/Upgrades/Mods/Rifle/WeaponStatusChanceSPMod` | Galvanized Aptitude | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Galvanized Aptitude; domain-model.md §16.2; F:mods §6.1; F:addmul §1.1-1.4; 04 §4.2 |
| 3319 | `/Lotus/Upgrades/Mods/Rifle/WeaponStunChanceMod` | Rifle Aptitude | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Rifle Aptitude |
| 3305 | `/Lotus/Upgrades/Mods/Rifle/WeaponToxinDamageMod` | Infected Clip | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Infected Clip; 00 §3.8; 01 §4.10 |
| 3314 | `/Lotus/Upgrades/Mods/Rifle/WeaponWeakpointCriticalChanceMod` | Primary Acuity | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Primary Acuity; 02 §4.5; domain-model.md §16.13 |
| 3338 | `/Lotus/Upgrades/Mods/Sets/Gladiator/MeleeGladiatorMightMod` | Gladiator Might | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Gladiator Might |
| 3340 | `/Lotus/Upgrades/Mods/Sets/Gladiator/MeleeGladiatorRushMod` | Gladiator Rush | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Gladiator Rush; 07 E-7.12 |
| 3339 | `/Lotus/Upgrades/Mods/Sets/Gladiator/MeleeGladiatorViceMod` | Gladiator Vice | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Gladiator Vice |
| 3315 | `/Lotus/Upgrades/Mods/Sets/Hunter/PrimaryHunterMunitionsMod` | Hunter Munitions | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Hunter Munitions; domain-model.md §16.5; 02 §4.7.4; D-016 |
| 3313 | `/Lotus/Upgrades/Mods/Sets/Synth/PistolSynthChargeMod` | Synth Charge | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Synth Charge; 04 §3.2; W:Synth_Charge; domain-model.md §16.13 |
| 3320 | `/Lotus/Upgrades/Mods/Sets/Vigilante/PrimaryVigilanteArmamentsMod` | Vigilante Armaments | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Vigilante Armaments |
| 3321 | `/Lotus/Upgrades/Mods/Sets/Vigilante/PrimaryVigilanteFervorMod` | Vigilante Fervor | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Vigilante Fervor |
| 3322 | `/Lotus/Upgrades/Mods/Sets/Vigilante/PrimaryVigilanteOffenseMod` | Vigilante Offense | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Vigilante Offense |
| 3323 | `/Lotus/Upgrades/Mods/Sets/Vigilante/PrimaryVigilanteSuppliesMod` | Vigilante Supplies | no_damage_effect | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Vigilante Supplies |
| 3406 | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/CritDamageForPunctureStatus` | Melee Doughty | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Melee Doughty; 04 §5.12; domain-model.md §16.13 |
| 3404 | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/HeavyCritOnRegularMelee` | Melee Animosity | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Melee Animosity; 04 §5.12; EC-07-28; domain-model.md §16.13 |
| 3403 | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/OverguardSteal` | Secondary Fortifier | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Secondary Fortifier; 04 §3.2; overview E-12; domain-model.md §16.13 |
| 3400 | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnNoMelee` | Primary Deadhead | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Primary Deadhead; domain-model.md §16.4; F:addmul §5; 04 §4.2 |
| 3401 | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamagePerAmmoOnReload` | Primary Plated Round | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Primary Plated Round; 04 §5.4; TV-04-16; EC-04-87; domain-model.md §16.13 |
| 3405 | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/ReplayLightCritHits` | Melee Duplicate | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Melee Duplicate; 04 §6.4; 04 E-4.15; 08 E-8.12; domain-model.md §16.13 |
| 3402 | `/Lotus/Upgrades/CosmeticEnhancers/Offensive/SecondaryExtraDamagePerFreezeStack` | Secondary Shiver | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Secondary Shiver; D-060; domain-model.md §16.13 |
| 3360 | `/Lotus/Powersuits/Excalibur/SwordOfDoomAugmentCard` | Chromatic Blade | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Chromatic Blade; domain-model.md §16.13; T-25 |
| 3324 | `/Lotus/Upgrades/Mods/Sets/Vigilante/WarframeVigilantePursuitMod` | Vigilante Pursuit | no_damage_effect | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Vigilante Pursuit |
| 3325 | `/Lotus/Upgrades/Mods/Sets/Vigilante/WarframeVigilanteVigorMod` | Vigilante Vigor | no_damage_effect | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Vigilante Vigor |
| 3361 | `/Lotus/Upgrades/Mods/Aura/EnemyArmorReductionAuraMod` | Corrosive Projection | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Corrosive Projection; 05 §9.3; E-15; TV-05-10 |
| 3362 | `/Lotus/Upgrades/Mods/Aura/PlayerMeleeStartingComboAuraMod` | Ready Steel | verified | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Ready Steel; 07 §6.5 |
| 3410 | `/Lotus/Upgrades/CosmeticEnhancers/Defensive/CritChanceOnDamage` | Arcane Avenger | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Arcane Avenger; domain-model.md §16.12; F:core §8 |
| 3411 | `/Lotus/Upgrades/CosmeticEnhancers/Utility/AbilityStrengthAndCritDamageWhenInvisible` | Arcane Crepuscular | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Arcane Crepuscular; 04 §5; 02 E-2.4 |
| 3412 | `/Lotus/Upgrades/CosmeticEnhancers/Utility/GolemArcaneBonusDamageOnWallLatch` | Arcane Arachne | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Arcane Arachne; 04 §3.4; 04 catalogue Arcane Arachne |
| 3451 | `wfd:shard/emerald/corrosive_stack` | +1 Max Corrosion Stacks | verified | W:Emerald_Archon_Shard; 03 §4.1 (COR cap 10 + β_COR); domain-model.md §16.13 |
| 3452 | `wfd:shard/emerald/toxin_status_damage` | +30% Toxin Status Damage | verified | W:Emerald_Archon_Shard; 03 §9.4 |
| 3450 | `wfd:shard/violet/primary_electricity` | +30% Primary Electricity Damage | verified | W:Violet_Archon_Shard; domain-model.md §16.9; D-006; OQ-024 |
| 3370 | `/Lotus/Weapons/Tenno/Melee/MeleeTrees/KatanaCmbThreeMeleeTree` | Blind Justice | no_damage_effect | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Blind Justice; 07 §4.1; wiki:Module:Stances/data#Blind Justice |
| 3501 | `wfd:perk/furis/extended_volley` | Extended Volley | verified | W:Furis_Incarnon_Genesis; F:mods §9; domain-model.md §16.10 |
| 3500 | `wfd:perk/furis/haven_foray` | Haven Foray | verified | W:Furis_Incarnon_Genesis; F:mods §9; domain-model.md §16.10 |
| 3502 | `wfd:perk/furis/prelude_of_might` | Prelude of Might | verified | W:Furis_Incarnon_Genesis; F:mods §9; domain-model.md §16.10 |
| 3514 | `wfd:perk/kunai/swift_conclusion` | Swift Conclusion | verified | W:Kunai_Incarnon_Genesis L49-57; 04 §2.1 catalogue (2.00 while target health < 50 %); domain-model.md §16.13 |
| 3510 | `wfd:perk/laetum/overwhelming_attrition` | Overwhelming Attrition | verified | W:Laetum L145-149; 04 §5.12; D-410; domain-model.md §16.13 |
| 3511 | `wfd:perk/phenmor/devouring_attrition` | Devouring Attrition | verified | W:Phenmor L126-131; 04 §3.2; TV-04-38; domain-model.md §16.13 |
| 3513 | `wfd:perk/phenmor/lingering_judgement` | Lingering Judgement | verified | W:Phenmor L140-145; 04 §5.12; domain-model.md §16.13 |
| 3512 | `wfd:perk/phenmor/spiteful_defilement` | Spiteful Defilement | verified | W:Phenmor L133-138; D-067; domain-model.md §16.13 |
| 3614 | `wfd:buff/ally_roar` | Roar (ally cast, Strength-scaled) | verified | W:Roar; 04 E-4.10; 04 §4.4 |
| 3601 | `wfd:buff/eclipse` | Eclipse | verified | W:Eclipse; 04 §3.1; T-14; D-058 |
| 3609 | `wfd:buff/electric_shield` | Electric Shield | verified | W:Electric_Shield; 04 catalogue Electric Shield |
| 3605 | `wfd:buff/flat_crit_chance` | Flat critical chance | verified | T-18; 00 §6.1 |
| 3608 | `wfd:buff/magus_accelerant` | Magus Accelerant | verified | wiki:Module:Arcane/data@2026-11-21T02:03:16Z#Magus Accelerant; 04 §5.5; domain-model.md §16.13 |
| 3613 | `wfd:buff/mecha_empowered` | Mecha Empowered | verified | W:Mecha_Empowered; 04 §3.2; D-041; domain-model.md §16.13 |
| 3607 | `wfd:buff/molecular_prime` | Molecular Prime | verified | W:Molecular_Prime; 04 §5.5 (+1.00 fixed, OG SH HP); TV-05-13 |
| 3604 | `wfd:buff/nourish` | Nourish | verified | W:Nourish; T-17; 01 §4.10.3 |
| 3606 | `wfd:buff/phoenix_talons` | Phoenix Talons | verified | W:Phoenix_Talons; 01 §4.9; D-007; T-19 |
| 3611 | `wfd:buff/piercing_navigator` | Piercing Navigator | verified | W:Piercing_Navigator; 04 catalogue Piercing Navigator |
| 3600 | `wfd:buff/roar` | Roar | verified | W:Roar; F:buffs §1.1; T-13; domain-model.md §16.7 |
| 3610 | `wfd:buff/smeeta_charm_crit` | Smeeta Kavat Charm (crit) | verified | W:Charm; 04 catalogue; EC-04-54; 02 E-2.4 CC_min |
| 3612 | `wfd:buff/sonar` | Sonar | verified | W:Sonar; 04 catalogue Sonar; 02 E-2.10 |
| 3603 | `wfd:buff/toxic_lash` | Toxic Lash | verified | W:Toxic_Lash; E-14; D-037; domain-model.md §16.8 |
| 3602 | `wfd:buff/xatas_whisper` | Xata's Whisper | verified | W:Xata's_Whisper; W:Extra_Hit; F:buffs §1.3; E-14; D-071; domain-model.md §16.8 |
| 3725 | `wfd:custom/abilitycombo` | Ability combo | unmodelled | 04 §10; 00 §6.3 |
| 3711 | `wfd:custom/absolute_status_chance` | Absolute status chance | verified | 04 §10; 00 §6.3 |
| 3700 | `wfd:custom/base` | base | verified | 04 §10; 00 §6.3 |
| 3701 | `wfd:custom/base_per_status` | Base damage per status | verified | 04 §10; 00 §6.3 |
| 3703 | `wfd:custom/crit_chance` | Critical chance | verified | 04 §10; 00 §6.3 |
| 3706 | `wfd:custom/crit_chance_weakp` | Weak-point critical chance | verified | 04 §10; 00 §6.3 |
| 3707 | `wfd:custom/crit_mult` | Critical damage | verified | 04 §10; 00 §6.3 |
| 3708 | `wfd:custom/crit_mult_add` | Additive critical damage | verified | 04 §10; 00 §6.3 |
| 3709 | `wfd:custom/crit_mult_mult` | Critical damage multiplier | verified | 04 §10; 00 §6.3 |
| 3722 | `wfd:custom/dmgvulnerability` | Damage vulnerability | verified | 04 §10; 00 §6.3 |
| 3702 | `wfd:custom/flat_change_dmg` | Flat damage | verified | 04 §10; 00 §6.3 |
| 3704 | `wfd:custom/flat_crit_chance` | Flat critical chance | verified | 04 §10; 00 §6.3 |
| 3720 | `wfd:custom/headshot_mult` | Weak-point damage | verified | 04 §10; 00 §6.3 |
| 3724 | `wfd:custom/heatadd` | Heat (non-combining) | verified | 04 §10; 00 §6.3 |
| 3723 | `wfd:custom/heatinherit` | Heat inherit | verified | 04 §10; 00 §6.3 |
| 3721 | `wfd:custom/initialcombo` | Initial combo | verified | 04 §10; 00 §6.3 |
| 3717 | `wfd:custom/magazinesize` | Magazine capacity | verified | 04 §10; 00 §6.3 |
| 3705 | `wfd:custom/mult_crit_chance` | Critical chance multiplier | verified | 04 §10; 00 §6.3 |
| 3719 | `wfd:custom/mult_for_head` | Headshot multiplier | verified | 04 §10; 00 §6.3 |
| 3716 | `wfd:custom/multishot` | Multishot | verified | 04 §10; 00 §6.3 |
| 3718 | `wfd:custom/reloadtime` | Reload speed | verified | 04 §10; 00 §6.3 |
| 3715 | `wfd:custom/speed` | Fire rate / attack speed | verified | 04 §10; 00 §6.3 |
| 3710 | `wfd:custom/status_chance` | Status chance | verified | 04 §10; 00 §6.3 |
| 3712 | `wfd:custom/status_chance_mult` | Status chance multiplier | verified | 04 §10; 00 §6.3 |
| 3713 | `wfd:custom/status_damage` | Status damage | verified | 04 §10; 00 §6.3 |
| 3714 | `wfd:custom/vuln_status_damage` | Status damage vulnerability | verified | 04 §10; 00 §6.3 |

## Sets

| uniqueName | Name | Sources |
|---|---|---|
| `/Lotus/Upgrades/Mods/Sets/Gladiator/GladiatorSetMod` | Gladiator Set | pe:ExportUpgrades.ExportModSet (uniqueName; the wiki module's InternalName points at the Augur set); 04 E-4.19; domain-model.md §16.13 |
| `/Lotus/Upgrades/Mods/Sets/Hunter/HunterSetMod` | Hunter Set | 04 §9 (companion only) |
| `/Lotus/Upgrades/Mods/Sets/Synth/SynthSetMod` | Synth Set | 04 §9 (reload while holstered: none) |
| `/Lotus/Upgrades/Mods/Sets/Vigilante/VigilanteSetMod` | Vigilante Set | wiki:Module:Mods/data@2026-11-01T23:35:48Z#Vigilante Set; 04 §9; domain-model.md §16.13 |

## Enemies

| Key | uniqueName | Sources |
|---|---|---|
| 9005 | `wfd:enemy/corpus/elite_crewman` | wiki:Module:Enemies/data/corpus@2026-07-18T17:29:47Z#Elite Crewman; 05 §4.3 Multis "Head: 3.0x", Corpus h = 1; TV-05-08 |
| 9002 | `wfd:enemy/grineer/heavy_gunner` | wiki:Module:Enemies/data/grineer@2026-08-19T14:46:16Z#Heavy Gunner; W:Enemy_Body_Parts L310; TV-05-05 (Eximus, L 58) |
| 9003 | `wfd:enemy/grineer/kuva_trokarian` | wiki:Module:Enemies/data/grineer@2026-08-19T14:46:16Z#Kuva Trokarian; 05 §4.3 rule F (faction default: Kuva Grineer head 3.0, h = 2); 05 §8 (overguard pool) |
| 9001 | `wfd:enemy/grineer/lancer` | wiki:Module:Enemies/data/grineer@2026-08-19T14:46:16Z#Lancer; W:Enemy_Body_Parts L322 (Body 1x, Head 3x); 05 §4.3 faction default h = 2; TV-05-03; 00 §3.8 |
| 9006 | `wfd:enemy/infestation/hemocyte` | wiki:Module:Enemies/data/infestation@2026-09-13T04:26:29Z#Hemocyte; 05 §4.3 hand row (W:Hemocyte L19-35); 05 §4.4 status ALL; 05 §4.5 attenuation 0.05 {HP}; TV-05-15 |
| 9004 | `wfd:enemy/orokin/corrupted_heavy_gunner` | wiki:Module:Enemies/data/orokin@fetched-2026-11-23#Corrupted Heavy Gunner; 05 §4.3 Multis "Head: 3.0x", faction default h = 2; TV-05-01 |
| 9007 | `wfd:enemy/themurmur/rogue_culverin` | wiki:Module:Enemies/data/themurmur@fetched-2026-11-23#Rogue Culverin; W:Enemy_Body_Parts L887 (Body 1x, Arm Canisters¹² 1.5x; ¹ promotes hits to next crit tier); 05 §4.3 rule T; TV-02-04 |

## Frames

| Key | uniqueName | Sources |
|---|---|---|
| 8000 | `/Lotus/Powersuits/Excalibur/Excalibur` | wiki:Module:Warframes/data@2026-11-10T01:27:04Z#Excalibur; pe:ExportWarframes#/Lotus/Powersuits/Excalibur/Excalibur; 09 §2 |
| 8001 | `/Lotus/Powersuits/Choir/Choir` | wiki:Module:Warframes/data@2026-11-10T01:27:04Z#Jade; pe:ExportWarframes#/Lotus/Powersuits/Choir/Choir; 09 §2.2 (Jade: two aura slots) |

## Combo tables and stances

Converted from `wiki:Module:Stances/data@2026-10-02T11:57:23Z` (evaluated with `addSharedCombos`, cached
`research/external/cache/poc/out/Stances_data.json`) by the normalisation E-7.0 of `07` §4.1 (steps 1-6);
`from_weapon` is kept. Table ids: 0 Blind Justice (stance), 1 Nikana (stanceless), 2 Tonfa (stanceless),
3 Telos Boltace (UniqueToWeapon, matched by name, `07` E-7.0b), 4 Dagger (stanceless). The stance mod is
`wiki:Module:Mods/data#Blind Justice`.

## Modular parts

Kitgun chamber Catchmoon (secondary), grips Ramble and Haymaker, loader Bashrack, zaw strike Balla, grip Peye,
link Jai: stats from `wiki:Module:Modular/data@2026-06-22T05:27:50Z` (`KitgunSecondary`, `Zaw`), uniqueNames from
`pe:ExportWeapons` (each name checked against the export).

## Rules (`rules.json`)

- `status`: `03` §4.1 summary table (all 18 rows).
- `factions`: `05` §5.1 (type modifiers), `01` Table 4.10-A (weakness), `01` §4.15 / D-008 (bane_of: Kuva Grineer
  takes Grineer banes).
- `scaling`: `05` §6.1 (health and shield classes, armor and overguard curves, smoothstep bounds).
- `mult_groups`: `04` §3.1-3.2, `05` §13, domain-model.md §14 (each row cites its source inline).
- `slot_layouts`: `09` §2.1-2.2 (primary, secondary with Incarnon tiers, melee with stance, warframe).
- `co_default`: `04` §2.5 rules D2-D6.
- `body_parts`: `05` §4.3 faction defaults (table F) and `02` §4.5 (k_wp = 1.5).
- `special_dr`: `05` §4.5 Hemocyte attenuation (p_att 0.05 on health).
- `riven_traits`, `riven_formula`: `04` §8.1-8.3 (E-4.18), `09` E-9.59 (default drain).
- `toggles`: `00` §6.1 (only the toggles the scenarios use; T03b transcribes the full table).
- `melee`: `07` table 9-A (Nikana, Tonfa, Dagger rows), table 9-D (`W:Condition_Overload_(Mechanic)` lines 592,
  642, 644, 682).
- `helminth`: `09` E-9.32 (subset).
- `categories`, `tags`: wiki mod `Type` strings
  (`Module:Mods/itemCompatibility/data`) and weapon `CompatibilityTags` used by the fixture.
- `game_constants`: `05` §7.2 (Steel Path ×2.5).
