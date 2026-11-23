Status: draft

# Ranged weapon firing mechanics: from stats to hits over time (game-truth side)

Scope: how each ranged weapon class turns its stats (fire rate, burst, charge, magazine, reload, ammo cost, multishot, AoE) into damage instances per second, and the burst/sustained DPS formulas built on them. Per-hit damage (base damage, elements, crit, faction, CO) is in `wiki-core-damage-formula.md`; enemy-side effects in `wiki-enemy-defenses-and-scaling.md`. This file only touches per-hit damage where a weapon class changes it (beam ramp-up, sniper combo, AoE falloff, first/last-shot bonuses, exalted strength scaling).

Research date: 2026-11-23. Sources: wiki.warframe.com raw wikitext (`?action=raw`), cached in `research/external/cache/wiki/` (page name with `/` → `__`), and the wiki Lua modules cached in `research/external/cache/wiki-modules/raw/` (`Weapons.lua`, `Weapons_data_*.lua`, `Modular_data.lua`, `Mods_data.lua`). "U" = game update as tagged by the wiki `{{ver|…}}` template. The wiki pages read do not give release dates for most updates, so "last changed" is usually just the update number.

Abbreviations: FR = fire rate (shots/s, or bursts/s for burst triggers, or ticks/s for beams), ΣFR = summed fire-rate bonus, CT = charge time, BC = burst count, BD = burst delay, RS = summed reload-speed bonus, MS = multishot, AE = ammo efficiency, N = shots per magazine, EFR = effective fire rate (attacks per second actually achieved), T_mag = time to empty one magazine, T_rel = time spent reloading.

Data-field names in `code` are the keys of the wiki weapon data modules (`Module:Weapons/data/*`), which is what the engine will most likely ingest (see `data-source-wiki-modules.md`).

---

## 0. Summary formula set (proposed engine model)

All of the following are derived in the sections below; each line cites the section.

```
FR_mod   = max(0.05, FR_base × (1 + ΣFR_eff))                      §1.1, §1.4
           ΣFR_eff: bow-tagged mod fire-rate bonuses count ×2 on bows   §1.5
           Cannonade mod installed → FR_mod = FR_base (all bonuses ignored) §1.3
           Semi-auto: EFR = min(FR_mod, 10)  (UNVERIFIED for exact cap)  §1.2
CT_mod   = min(10 × CT_base, CT_base / (1 + ΣFR_eff))              §3.1
BD_mod   = BD_base / (1 + max(0, ΣFR_eff))                          §2
Mag_mod  = max(1, round(Mag_base × (1 + ΣMag)) [+ flat])            §8.1
cost_eff = AmmoCost × (1 − min(ΣAE, 0.75))   (infinite ammo → no reload) §8.3
N        = Mag_mod / cost_eff   (average; actual alternates floor/ceil) §8.3
Reload   = Reload_base / (1 + RS)                                   §9.1
           per-shell: (t_startend + t_shell × Mag_mod) / (1 + RS)   §9.2
           battery:   Delay_empty / (1 + RS) + Mag_mod / RechargeRate §9.3

Per-attack timing (one "tap" = one trigger input):
  Auto / Held / Auto-Spool(full) / Semi / Duplex:  T_mag = N / FR_mod
  Burst (incl. auto-burst, mag-burst):              T_mag = ceil(N/BC) × (1/FR_mod + (BC−1)·BD_mod)   (last burst may be partial)
  Charge (non-bow, Mag > 1):                        T_mag = N × (CT_mod + 1/FR_mod)
  Lanka:                                            T_mag = N × CT_mod
  Bow / Mag = 1:                                    cycle = CT_mod + Reload  (no separate reload term)
  Vectis (Mag 1, no reload delay):                  cycle = Reload  (shot is instantaneous)

BurstDPS     = AvgShotDamage × N / T_mag            (= AvgShot × EFR)
SustainedDPS = BurstDPS × T_mag / (T_mag + T_rel)
```

The task's generic `sustained = burst × mag_time / (mag_time + reload_time)` is correct **if** mag_time is defined as `N / EFR` (it already contains the one 1/FR "reload delay" after the last shot, see §9.4) and the weapon is not one of the deviations: bows / magazine-1 weapons (§5, §9.4), Vectis (§9.4), Epitaph (§5.3), battery weapons (§9.3), per-shell reloaders (§9.2), Incarnon forms (§13), beams with ramp decay during reload (§4.3), spool-up weapons (§6), and infinite-ammo states (§8.3).

---

## 1. Fire rate

### 1.1 Definition and linear scaling
- FR is "the number of times per second a weapon discharges a projectile, or applies a discrete instance of damage … (normally one round per shot, and for continuous weapons 0.5 rounds per tick)" (wiki `Fire Rate` intro).
- `FR_mod = FR_base × (1 + ΣFR)` (wiki `Damage/Calculation` § Modded Stats: "Modded Stat = Base Stat × (1 + Stat Bonuses)"; exceptions listed there are accuracy, reload time and charge time only).
- "The Fire Rate bonus increases fire rate linearly, regardless of the Trigger Type" — on burst weapons it affects both burst speed and time between bursts; it also affects Spool-up Time and Charge Time (wiki `Fire Rate` § Mechanics). Principle in force since U17.4.3; before that, fire rate did not shorten the in-burst interval (wiki `Fire Rate` § Trivia).
- Shot delay = `1 / FR_mod` (wiki `Fire Rate` § Shot Delay).
- Floor: "Fire Rate cannot drop below 0.05 rounds per second, resulting in 20 seconds between shots" (wiki `Fire Rate` § Notes).
- Fire rate also shortens the delay before an automatic reload on an empty magazine, before a manual reload, and before weapon swap / quick melee (wiki `Fire Rate` § Notes).
- Sources of FR are additive in one bucket (mods, arcanes, abilities) — wiki `Fire Rate` lists sources but does not state stacking explicitly for every source. `Calculating Bonuses` (cached) states most same-stat bonuses stack additively; individual buff stacking (e.g. Wisp Haste Mote, Arcane Acceleration) not individually verified → UNVERIFIED per source.

Worked example (wiki `Reload` § Reload Speed vs Fire Rate): Viper FR 14.4, rank-3 Gunslinger (+48 %) → 14.4 × 1.48 = 21.31 shots/s.

### 1.2 Semi-auto cap
- "Increasing fire rate of Semi-auto weapons increases the cap on how quickly they can be fired … Notably, Semi-Auto weapons are capped at 10 rounds per second." (wiki `Fire Rate` § Mechanics). Arca Scisco page: "Fire rate is capped at 10 Shots/Sec due to the Semi-Auto trigger type."
- Komorex page (§ Notes) says its fire rate is "hard-capped around a value of 8.0, slightly below the typical semi-auto fire rate limit of 10", with a tooltip: "Fire rate caps are not well documented or understood. This value was measured at 60 FPS …". So the cap may be frame-rate or animation dependent → exact value UNVERIFIED; no patch note found that introduced it.
- The "Fire Manual Trigger Weapons Continuously" accessibility option makes Semi and Burst fire automatically (not Duplex or Charge) and does not alter fire rate (wiki `Trigger Type` § Accessibility Options; U38.5 fix). The wiki DPS model assumes the player reaches the modded fire rate on Semi/Duplex (wiki `Damage/Calculation` § Average Burst DPS table).
- Proposed: `EFR_semi = min(FR_mod, 10)`, with a per-weapon override (Komorex ≈ 8). Whether the cap also applies to the semi-auto modes of multi-mode weapons and to Semi kitguns is UNVERIFIED.

### 1.3 Locked fire rate (Cannonade mods, U36)
- Semi-Pistol/Rifle/Shotgun Cannonade: only on weapons whose every trigger mode is Semi; "lock the fire rate … preventing any changes to the fire rate regardless if they're positive or negative" (wiki `Fire Rate` § Locking Fire Rate; `Cannonade Mods`). Semi-Rifle Cannonade: "+240% Damage, +1.5 Punch Through" (Mods_data description); `Semi-Rifle Cannonade` § Notes: "set weapon's Fire Rate to its default ignoring other bonuses, even negative effects (e.g. Critical Delay, Vile Precision, Komorex's second zoom)". Cannonade's damage is described as a "large increase to base damage" (`Cannonade Mods`), i.e. it joins the Serration bucket (stacking not separately verified → UNVERIFIED).
- Latron and Vectis Incarnon are the only Incarnon weapons that can equip it (`Semi-Rifle Cannonade` § Notes). Nagantaka (Prime) cannot.
- Engine rule: if a Cannonade is installed, ΣFR = 0 for every attack of the weapon.

### 1.4 Fire-rate floor interactions
- Charge time ceiling 10 × base (≥ −90 % FR), see §3.1. Burst delay ignores net negative FR, see §2.

### 1.5 Bows: fire-rate mods count double
- Mod cards for rifle fire-rate mods read "(x2 for Bows)": Speed Trigger "+60% Fire Rate (x2 for Bows)", Vile Acceleration "+90% Fire Rate (x2 for Bows), −15% Damage", Critical Delay "−20% Fire Rate (x2 for Bows)", plus four more entries in `Mods_data.lua` (lines 5968, 10731, 20647, 24837, 25512, 29028, 29180, 29216 — includes the Shred/Primed Shred/Vigilante Fervor/Creeping Bullseye family). `Speed Trigger` stat table has separate Rifle / Bow columns (+60 % / +120 % at rank 5) and § Notes: "330% (660% for Bows)". `Paris`, `Daikyu`, `Kuva Bramma`, `Evensong`, `Cernos` pages: "Fire rate mods apply double their bonuses".
- So for a bow: `CT_mod = CT_base / (1 + 2 × ΣFR_mods + ΣFR_other)`.
- Whether non-mod fire-rate sources (Arcane Velocity/Acceleration, Wisp, Gauss, Harrow) are also doubled on bows is UNVERIFIED (only the mod descriptions carry "x2"). Which weapons count as "Bows" for this rule (class `Bow` in data, including crossbows such as Zhuge/Attica?) is UNVERIFIED; the data module puts crossbows under class "Bow" (`Weapons.lua` line 785 maps "Crossbow" → "Bow").

Worked example: Paris CT 0.5 s, Speed Trigger rank 5 → 0.5 / (1 + 1.2) = 0.227 s (rifle would be 0.5/1.6 = 0.3125 s).

---

## 2. Burst weapons

**Formula** (wiki `Fire Rate` § Fire Rate and Burst Weapons; same in `Damage/Calculation` § Average Burst DPS):
```
EFR_burst (shots/s) = BC / ( 1/FR_mod + (BC − 1) × BD )
```
- FR is "number of bursts per second if Burst Delay is 0"; its reciprocal is the wait after a burst completes before the next one. No delay after the last shot of a burst (also `Weapons.lua` `BurstsPerSec`: "There is no delay after last shot in burst").
- BD is the hidden delay between shots in a burst (`BurstDelay`), BC the shot count (`BurstCount`; shown in the arsenal as "BURST (x2)" since U37, `Trigger Type` patch history).
- **Fire rate mods affect burst delay** (linear-scaling principle, `Fire Rate` § Mechanics: "on Burst weapons … it affects both the speed of the burst as well as the time between bursts"), but "Burst Delay is **not** affected by net negative Fire Rate bonuses" (`Fire Rate` § Burst). Proposed: `BD_mod = BD / (1 + max(0, ΣFR))`.
  - Conflict: the displayed formula in both wiki pages and `Weapons.lua` uses the unmodded `Burst Delay`; only the prose says FR affects it. The inspiration's changelog (guide.txt, 2026-01-28: "Fixed an issue where rate of fire mods would not apply to the burst delay") agrees with the prose. Proposed rule: divide BD by (1+ΣFR) when ΣFR > 0 (recorded here as the wiki-prose reading; numeric in-game confirmation not found → partly UNVERIFIED).
- Simultaneous bursts (BD = 0: Quartakk, Kuva Quartakk, Quatz, Akjagara, Sepulcrum, Morgha) → EFR = BC × FR_mod; differs from innate multishot only in ammo cost (`Trigger Type` § List).
- Auto-Burst (Cyngas, Battacor, Morgha) fires bursts while held (`Trigger Type`).
- Mag Burst (Pandero alt-fire, Nagantaka alt, Cyanex alt, Kuva Kraken alt): "use the full magazine size as the Burst Count" (`Damage/Calculation` § Effective FR table). Data `BurstReloadDelay` (Nagantaka 0.2, Cyanex 0.21, Pandero/Kuva Kraken 0.24, Tenet Diplos 0.1 s) is defined in `Weapons.lua` but not used in its DPS calc; meaning (extra delay before reload after a mag burst?) UNVERIFIED.
- Partial last burst: if Mag_mod is not a multiple of BC, the last burst fires the remainder (Burston Prime § Notes: "you might be reduced to only 1 or 2 rounds in your last shot"). So bursts per mag = ceil(N / BC).

Worked example (Burston: FR 5, BD 0.061, BC 3, Mag 45, Reload 2.0 — data `Weapons_data_primary.lua`):
- Unmodded: EFR = 3 / (0.2 + 2 × 0.061) = 3 / 0.322 = 9.317 shots/s.
- Speed Trigger +60 %: FR 8, BD 0.061/1.6 = 0.038125 → 3 / (0.125 + 0.07625) = 14.907 shots/s = 1.6 × 9.317 (linear, as the wiki prose requires). If BD were left unmodded (wiki formula as displayed): 3 / (0.125 + 0.122) = 12.146 — a 19 % difference, so this choice matters.
- Unmodded sustained: T_mag = 15 bursts × 0.322 = 4.83 s; ratio = 4.83 / (4.83 + 2.0) = 0.707.

Worked example (mag-1/2 burst remainder): Burston Prime mag 54 / BC 3 = 18 full bursts (page example).

---

## 3. Charge weapons

### 3.1 Charge time
- Arsenal "Charge Rate" is actually charge time (s) (`Fire Rate` § Fire Rate and Charge Time).
- `CT_mod = CT_base / (1 + ΣFR)`; equivalently charge rate × (1 + ΣFR). "Charge time cannot go above 10 times the base charge time (achieved by having at least −90% fire rate bonus)." (`Fire Rate`). `Damage/Calculation` § Modded Stats lists charge time under the "Base / (1 + speed bonus)" rule.
- Charge-time list (selected; `Fire Rate` § Charge Times): Opticor 2.0, Opticor Vandal 0.6, Lanka 1.0, Ogris 0.3, Ferrox 0.5, Tenet Ferrox 0.4, Scourge 0.5, Kuva Drakgoon 0.3, Phantasma 1.0 ("not affected by mods"), Ballistica 1.0, Stug 3.0, Balefire Charger 2.0, Angstrum 0.5/1.5 (scales with magazine size), bows 0.25–1.5 (Paris 0.5, Lenz 1.2, Kuva Bramma 0.4, Cinta 1.5). Data field `ChargeTime` per attack.

### 3.2 Effective fire rate (three cases, `Fire Rate` § Fire Rate and Charge Time; `Damage/Calculation`)
```
Charge weapons (not bows, Epitaph, Lanka): EFR = 1 / (CT_mod + 1/FR_mod)
Bows:                                       EFR = 1 / (CT_mod + Reload_mod)
Lanka (no delay between charged shots):     EFR = 1 / CT_mod
```
The wiki assumes the player releases at full charge (`Damage/Calculation`). The data module gives Lanka `FireRate = math.huge`, consistent with the third case.

Worked example (Opticor: CT 2, FR 1, Mag 5, Reload 2):
- Unmodded EFR = 1 / (2 + 1) = 0.333/s. With +60 % FR: 1 / (1.25 + 0.625) = 0.533/s.
- Modded sustained ratio: T_mag = 5 / 0.533 = 9.375 s → 9.375 / (9.375 + 2) = 0.824.

Worked example (Lanka: CT 1.0, Mag 10, Reload 2): EFR 1.0; ratio 10 / 12 = 0.833.

### 3.3 Partial charges ("quick shots")
Charge trigger subtypes (`Trigger Type` § Charge Trigger Subtypes table):
- Full quickshot (release any time): Drakgoon/Kuva Drakgoon, Dread, Miter, Paris family, Cernos family, Evensong, Artemis Bow, Ballistica family, Epitaph, Stug alt, Balefire Charger, Velocitus, Corvas, Mandonel; auto-release variants Tombfinger primary, Phantasma alt; delayed auto-release Staticor, Nataruk (2 s).
- Partial quickshot (needs a minimum charge %): Lanka (33 %: "Can be fired after charging for at least 33% of maximum charge, but with reduced damage and critical chance", `Lanka`), Daikyu Prime, Nunchasa; Opticor/Opticor Vandal ("Can be fired at 40% of maximum charge, but with less damage", `Opticor`).
- No quickshot: Ogris, Proboscis Cernos, Kuva Bramma, Daikyu, Paracyst alt, Lenz (uncharges after 5 s), Ferrox, Tenet Ferrox (auto-charges next shot), Larkspur alt.
- Autocharge (tap = full charge): Javlok, Stahlta alt, Battacor alt, Tenora alt, Tenet Plinx alt, Ambassador alt, Aeolak alt, Vadarya Prime; Angstrum "autocharged quickshot".
- Damage of a partial charge: the wiki data models quick shots as **separate attacks with fixed stats**, not as an interpolation: e.g. Paris "Uncharged Shot" 160 total vs "Charged Shot" 320; Lanka "Partially Charged Shot" (CT 0.33, 200 Electricity, CC 0.20) vs "Charged Shot" (525, CC 0.25); Opticor "Quick Shot" 500 + 200 AoE vs charged 1000 + 400 AoE (`Weapons_data_primary.lua`). Whether damage scales continuously between the minimum and full charge is UNVERIFIED; the wiki exposes only the two end points.
- Since U25.7 bow quick shots share crit/status stats with charged shots ("Normalized quick shots to have the same critical chance/damage and status chance as charged shots", `Bow` patch history). The current Paris data agrees (CC 0.3/CD 2/SC 0.1 for both), but Epitaph does not (charged CC 0.48 vs uncharged 0.02) — the rule is bow-only.
- Charge-trigger accessibility toggle does not auto-fire charge weapons (`Trigger Type`).

---

## 4. Continuous (beam / "Held") weapons

### 4.1 Ticks, ammo, crit and status
- Each tick is one damage instance every `1/FR_mod` s; "Consumes 0.5 ammo per tick of damage (although not true for some continuous weapons)" (`Continuous Weapon` § Characteristics). Data: `AmmoCost = 0.5` on all held attacks checked (Amprex, Ignis, Ignis Wraith, Flux Rifle, Synapse, Glaxion, Phage, Quanta beam, Spectra, Nukor, Gammacor, Cycron, Catabolyst, Embolist, Larkspur, Cortege, Artax, Tazicor). U22.13.3 said "unless they are Flamethrowers", but current data gives Ignis 0.5 too → the flamethrower exception appears obsolete (note: old patch note vs current data).
- Status chance is rolled per tick since U22.15 ("now have a chance to Status proc for every tick … based on Fire Rate, instead of only ticking once per second", `Continuous Weapon` patch history). Arsenal shows "STATUS" not "STATUS/SEC" since U22.13.3.
- Crit is rolled once per merged tick (see 4.2).
- Hitscan, pinpoint accuracy, no recoil, limited range (`Range`, modifiable by Sinister Reach / Ruinous Extension). Projectile-speed mods change beam damage falloff but not beam range (`Damage Falloff` § Range Falloff).

### 4.2 Multishot on beams
- "Multiple hits on the same target from Multishot are combined, producing a damage instance with damage and Status Chance equal to the sum of all hits" (`Continuous Weapon`); "the Critical Chance is still equal to that of a single beam", damaging status (Slash, Heat, Toxin, Electricity, Blast, Gas) output is affected **twice** by multishot; forced procs (Hunter Munitions, Seeking Talons) apply after merging, once per tick (`Multishot` § Continuous Weapons). Holds for innate-multishot beams (Quanta, 2 beams that can hit separate targets).
- Example (`Multishot`): SC 40 %, MS 2.5 → rolls of 2 or 3 beams give 80 % or 120 % for that tick. Amprex 22/tick with +150 % MS: always 44, 50 % chance of 66; still 1 instance per tick.
- Multishot does **not** affect the spherical blast of continuous weapons (Ignis/Ignis Wraith, Glaxion Vandal, Gaze primary, Embolist, Catabolyst, Cortege) (`Multishot`).
- Expected values per tick (derived): E[dmg] = D_beam × MS × E[crit mult]; E[status chance of tick] = SC × MS (same mean proc count as normal weapons, but procs > 100 % chance roll extra procs per `Status Effect` § Status Chance); damage per proc ∝ MS (merged damage).

### 4.3 Damage ramp-up
- "Initial damage starts at a lower percentage, and ramps up to 100% of its damage over 0.6 seconds of hitting a target. 0.8 seconds after the weapon stops hitting a target, the damage decays back to its initial point over 2 seconds. For most weapons, this lower percentage is 20%." (`Continuous Weapon` § Characteristics). Ramp is per target (U22.13.4 fix).
- Last changed U22.14: ramp 0.6 s, decay delay 0.8 s, decay 2 s, start 20 % except Convectrix 60 % (alt 80 %), Phage 70 %, Embolist 30 % (`Continuous Weapon` patch history). Per-weapon start values may have changed since U22.14; no newer note found and the data modules carry no ramp field → current per-weapon values UNVERIFIED.
- Shape of ramp (linear?) is not stated → UNVERIFIED (linear assumed below).
- Wiki DPS formulas assume full ramp and note that "reloading will reset/decay the damage ramp up so Burst DPS will be lower" (`Damage/Calculation`).
- Basmu alt-fire also "requires a damage ramp-up" (wiki search snippet, `Basmu`); Tenet Flux Rifle is an automatic (no ramp, no beam multishot merging) despite its name (wiki search snippet, `Tenet Flux Rifle`).

Proposed ramp model for sustained DPS (assumes linear ramp and decay, single target, continuous firing apart from reloads; UNVERIFIED shape):
```
s = start fraction (0.2), up = 0.6 s, hold = 0.8 s, down = 2.0 s
level after a reload of length R: L = 1                          if R ≤ 0.8
                                  L = max(s, 1 − (1−s)(R−0.8)/2)  otherwise
time to re-ramp: t_r = (1 − L) × up / (1 − s)
damage lost per magazine (in "seconds of full damage"): (1 − L) × t_r / 2
```
Worked example (Amprex: FR 12, cost 0.5, Mag 100, Reload 2.6): N = 200 ticks, T_mag = 16.67 s, base ratio 16.67/19.27 = 0.865. After a 2.6 s reload: L = 1 − 0.8 × 1.8/2 = 0.28; t_r = 0.72 × 0.6/0.8 = 0.54 s; loss = 0.72 × 0.54 / 2 = 0.194 s → in-magazine factor (16.67 − 0.194)/16.67 = 0.988. On the first engagement (L = 0.2): loss = 0.8 × 0.6 / 2 = 0.24 s.

### 4.4 Chaining and other beam specials (`Continuous Weapon` § Unique Mechanics)
- Linear chain beams (Amprex, Atomos, Basmu alt, Gaze secondary, Larkspur, Tenet Glaxion, Torid/Boar Incarnon): each further target receives "a reduced factor of the previous target's damage"; Kuva Nukor, Tenet Cycron, Vermisplicer primary make several single-target chains at reduced damage. Chain range not affected by beam-length mods (U22.14); chains use no ammo; punch through multiplies chains. Per-weapon chain factors not on this page → UNVERIFIED / weapon pages.

### 4.5 Beam DPS formula
`BurstDPS = AvgTick × FR_mod × ramp_factor`; `N = Mag_mod / (0.5 × (1 − AE))`; `T_mag = N / FR_mod`. Example: Ignis Mag 150, FR 8 → N = 300 ticks, T_mag = 37.5 s, Reload 2.0 → ratio 0.949.

---

## 5. Bows, single-shot weapons, snipers

### 5.1 Bows
- All non-crossbow bows: Charge trigger; "effective magazine of 1 that cannot be increased with Magazine Size mods, but also lack a Reload delay after firing" (`Bow` § Characteristics; magazine-UI fix U38.5). `Ammo` § Magazine Capacity: magazine bonuses don't affect bows or Epitaph (they draw from reserve).
- EFR = 1 / (CT_mod + Reload_mod) (`Fire Rate`); no separate reload term in sustained DPS (the reload is the inter-shot time). `Weapons.lua` `InterShotTime`: for `Magazine == 1`, inter-shot time = `Reload + ReloadDelayEmpty`, else 1/FR; `EffectiveFireRate = 1/(ChargeTime + InterShotTime)`.
- Fire-rate mods ×2 on bows (§1.5). Reload-speed mods: bows use rifle reload mods (`Reload` § Mods).
- Split Flights: "On Hit: +100% Multishot, −180% Accuracy for 2s, stacks 4×, (Non-AOE Bows)" (Mods_data).
- Worked example (Paris: CT 0.5, Reload 0.65): EFR = 1/1.15 = 0.870/s; with Speed Trigger (bow +120 %) → 1/(0.227 + 0.65) = 1.140/s; with +30 % reload as well → 1/(0.227 + 0.5) = 1.375/s.

### 5.2 Magazine-1 non-bow weapons
- `Reload` § Weapons With No Reload Delay: "Weapons that have a Magazine Size of 1 will not have Reload Delay." → cycle = CT + Reload (Tonkor: 1.7 s, so FR 3.17 is irrelevant unless Mag is raised).
- Vectis/Vectis Prime: "not affected by this shot delay before reloading and reloads instantly after the last shot" (`Fire Rate` § Shot Delay); `Damage/Calculation`: subtract 1 from the denominator for Vectis. Data `ReloadDelay = 0` on both; U-history "Removed .25 second delay for auto reloading the Vectis" (`Vectis`).
- Worked example (Vectis: FR 1.5, Mag 1, Reload 1.0): Vectis rule ratio = 1/(1.5 × 1 + 1 − 1) = 0.667 → sustained = AvgShot × 1.5 × 0.667 = AvgShot × 1.0/s (one shot per 1.0 s reload). Generic formula would give 1/(1.5 + 1) = 0.4 → AvgShot × 0.6/s.
- CONFLICT: `Damage/Calculation` applies the "−1" only to Vectis, implying other magazine-1 weapons do pay a 1/FR delay, while `Reload` says all magazine-1 weapons have no reload delay. Open question.

### 5.3 Epitaph
- "Ignore Modded Reload Time since it draws ammo directly from the full ammo pool similar to bows, but does not 'reload' between shots like them" (`Damage/Calculation` § Average Sustained DPS). `Fire Rate` excludes Epitaph from the charge EFR formula but gives none for it. Data: Mag 1, Reload 0.6, FR 1.5, charged CT 0.36. Proposed EFR = 1/(CT + 1/FR) for charged shots, no reload → UNVERIFIED.

### 5.4 Sniper rifles
- Shot Combo Counter (`Sniper Rifle` § Shot Combo Counter):
  ```
  Combo required for multiplier M = MinCombo × 3^(2M − 3)
  M(count) = 1.5 + 0.5 × floor(log3(count / MinCombo))   for count ≥ MinCombo, else 1.0
  ```
  It is "a bonus to their total damage" (multiplicative, final; ordering relative to other buckets is in `wiki-core-damage-formula.md`). Each multishot projectile that hits counts as a hit; punch-through and ricochet hits count; AoE and DoT do not count; must be scoped (except Collective Curse links). Decay: −1 count per miss and −1 per expiry of the combo duration (2 s; Lanka 6 s; Harkonar Scope +12 s). U22 last changed decay rules. Max count 2^32 − 1 (trivia).
- Data `SniperComboMin` / `SniperComboReset`: Rubico/Rubico Prime/Perigale/Sporothrix/Coda Sporothrix/Vectis 1; Lanka 2 (reset 6); Vulkar/Vulkar Wraith 2; Snipetron/Snipetron Vandal/Vadarya Prime 3; Komorex/Vectis Prime 5. (`Sniper Rifle` table agrees.) Arca Scisco and Knell have reset 2 with no min (secondaries with sniper-style combo; behaviour UNVERIFIED).
- Worked example (Rubico, Min 1): 1.5× at 1 hit, 2.0× at 3, 2.5× at 9, 3.0× at 27, 3.5× at 81. Vectis Prime (Min 5): 5 / 15 / 45 / 135. With 2 bullets from multishot landing on the first trigger pull, a Lanka (Min 2) gets 1.5× immediately (`Lanka` § Notes).
- Zoom buffs (`Sniper Rifle` § Zoom Buffs table): crit chance, crit multiplier or headshot damage per zoom level, "generally stack additively with similar buffs from mods" — **except Lanka and Komorex**. Lanka: "The zoom bonus adds a flat +20/30/50 critical chance, applied after mods. With Point Strike at max zoom … (25 × (1 + 1.5)) + 50 = 112.5%"; applies if scoped when the shot hits (`Lanka` § Notes). Komorex 3.5× zoom: +100 % Damage, +3 m explosion radius, −75 % Fire Rate; fire rate hard-capped ≈ 8 (§1.2).
- Range: hitscan sniper limit 1000 m; sniper falloff 100 % → 50 % between 400 m and 600 m (`Damage Falloff`, `Sniper Rifle` U22).

---

## 6. Auto-spool weapons

- Spool cadence (shots to full, start fraction, increment per shot), from weapon pages (all "§ Characteristics"):

| Weapon | Shots | Start | +per shot | Data `Spool` |
|---|---|---|---|---|
| Gorgon | 9 | 20 % | 10.667 % | 9 |
| Gorgon Wraith | 6 | 20 % | 16 % | 6 |
| Prisma Gorgon | 7 | 20 % | 13.33 % | 7 |
| Soma | 6 | 25 % | 15 % | 6 |
| Soma Prime | 4 | 25 % | 30 % | 4 |
| Supra | 5 | 10 % | 22.5 % | 5 |
| Supra Vandal | 4 | 40 % | 20 % | 4 |
| Tenora | 9 | 40 % | 7.5 % | 9 |
| Aksomati | 6 | 25 % | 15 % | 6 |
| Cestra | 6 | 60 % | 8 % | — |
| Secura Dual Cestra | 5 (page) | 60 % | 8 % | 5 |
| Kohmak | 4 (FR) / 5 (pellets) | 20 % | 32 % | 4 |
| Twin Kohmak | 11 / 5 | 20 % | 8 % | 11 |
| Grattler | 4 | 40 % | 20 % | 4 |
| Phaedra | 11 | 50 % | 5 % | 11 |
| Imperator Vandal | 16 | 53.33 % | 3.11 % | 16 |

  Consistency check: start + (shots − 1) × inc ≥ 100 % for every row (Gorgon 20 + 8 × 10.667 = 105 → capped). Secura Dual Cestra page says 5 shots but 60 + 4 × 8 = 92 % → it would need 6; mismatch on the page → UNVERIFIED. Aksomati Prime data `Spool = 11`, page numbers not found. Kuva Grattler "No longer auto-spool and reaches peak fire rate instantly" (`Kuva Weapons`).
- Model: `rate_k = FR_mod × min(1, start + k × inc)`, k = 0,1,…; fire-rate mods scale the whole curve (`Fire Rate` § Mechanics: FR bonus "also affects Spool-up Time"). Whether the interval *after* shot k is 1/rate_k (assumed) or 1/rate_{k+1} is UNVERIFIED.
- "Burst firing maintains spool-up" (Gorgon, Prisma Gorgon); the spool is kept while sprinting with the trigger held (Gorgon § Tips). When spool resets (on reload? after a time without firing?) is not stated → UNVERIFIED. The wiki DPS formula assumes fully spooled (`Damage/Calculation`).
- Worked example (Gorgon, FR 12.5, Mag 90, Reload 4.2, assuming reset every reload): first 8 intervals at rates 0.2, 0.307, 0.413, 0.52, 0.627, 0.733, 0.84, 0.947 × 12.5 → Σ(1/rate) = 17.81/12.5 = 1.425 s vs 0.64 s at full rate → +0.785 s per magazine. T_mag = 7.2 + 0.785 = 7.985 s; sustained ratio 7.985/(7.985 + 4.2) = 0.655 (vs 0.632 if T_mag were 7.2, but with fewer shots per second: average EFR over mag = 90/7.985 = 11.27).
- **Kohm family (spooling multishot)**: each consecutive shot adds a pellet up to 12 (Kohm/Kuva Kohm) or 5 (Kohmak/Twin Kohmak); full spool uses 4 ammo/shot (Kohm) or 1.67 (Kohmak). "The listed status chance is for the first shot only, and is divided between the pellets the longer it is fired" (`Kohm` § Notes). Data: Kohm "Single Pellet" SC 0.75, FR 0.733, cost 1 vs "Fully Spooled" MS 12, SC 0.0625 (= 0.75/12), FR 3.67, cost 4. So per-pellet status chance = SC_listed / pellet count — unlike every other shotgun (§7). The Kohm attempts to reload when the magazine cannot pay the next shot's ammo cost (`Kohm` § Notes).

---

## 7. Shotguns, multishot, pellets

- Multishot: `MS = MS_base × (1 + ΣMS)`; integer part = guaranteed projectiles, fractional part = chance of one more (`Multishot` § Calculating; Lex +180 % → 2.8; Hek + Hell's Chamber 7 × 2.2 = 15.4). Each projectile rolls crit and status independently. Projectiles per unit of ammo (no extra ammo cost). Data `Damage` is **per projectile** (`Weapons.lua` CalcDamage multiplies hitDamage by multishot).
- Status per pellet: since U27.2 the arsenal SC is the per-pellet chance; before U27.2 `per pellet = 1 − (1 − SC)^(1/pellets)` (`Shotgun` § Trivia, patch notes U27.2 which also tripled shotgun SC). Exception: Kohm family (§6).
- Multishot effectiveness depends on accuracy/spread (`Multishot` § Mechanics → `Accuracy`); the UI assumes all pellets hit (`Multishot`). Not modelled; a "pellets hitting %" user input is a reasonable engine option.
- Riven multishot curse cannot push MS below 1; Acuity mods lock MS to base (see `wiki-core-damage-formula.md` §7).
- All shotguns have damage falloff (`Shotgun`); see §10.2.
- Spearguns: multishot has no effect on the thrown component (`Multishot` § Spearguns); spearguns reload 33 % of the magazine per second while thrown or holstered (U25, `Speargun`).

Worked example (Hek, data: 75 dmg/pellet × MS 7, FR 2.17, Mag 4, Reload 2.0): with Hell's Chamber (+120 %) MS 15.4 → 15.4 × 75 = 1155 per shot before crit; T_mag = 4/2.17 = 1.843 s; ratio 1.843/3.843 = 0.480.

---

## 8. Magazine, ammo cost and ammo efficiency

### 8.1 Magazine size
- `Mag_mod = Mag_base × (1 + ΣMag%) + flat` — percentages first, flat additions after (`Calculating Bonuses` line 76: Viper × (1 + Slip Magazine 0.3) + Stinging Truth 40).
- Rounding: "rounded to the nearest whole number" (`Magazine Warp`: Braton 45 × 1.3 = 58.5 → 59; `Wildfire`: 9 × 1.2 = 10.8 → 11; `Tainted Mag`, `Slip Magazine`, `Burdened Magazine`, `Depleted Reload`); `Ammo Stock` says "rounded up" (6 × 1.6 = 9.6 → 10; 1 × 1.6 = 1.6 → 2). Every example given is consistent with both "nearest, half up" and "ceil" → which one is UNVERIFIED (distinguishing case: fractional part < 0.5, e.g. Tigris 2 × 1.2 = 2.4). Minimum 1 (`Depleted Reload`: "can not be reduce to 0"; Vectis Prime 2 × 0.4 = 0.8 → 1).
- Bows and Epitaph ignore magazine bonuses (§5). Incarnon-form magazines ignore mods (§13). Battery weapons: magazine increases recharge time (§9.3).
- Ammo Maximum bonuses do not change magazine (`Ammo`).

### 8.2 Ammo cost per shot
- `N = Mag_mod / AmmoCost` (`Damage/Calculation` § Number of Shots Per Magazine; `Weapons.lua` `ShotsPerMag = floor(Magazine / AmmoCost)` — note the Lua floors, the article doesn't). Non-1 costs from data: beams 0.5/tick; Kohm full spool 4; Kohmak 1.67; Panthera 2; Twin Grakatas 2; Stradavar Prime semi 2; Zenith semi 3; Kuva Hind semi 3; Fusilai semi 3; Bubonico burst 3; Kuva Hek alt 4; Corinth Prime air-burst 4; Ballistica Prime 4; Staticor 5; Mutalist Quanta orb 5; Tenora/Tenora Prime charged 10; Aeolak alt 10; Quanta cube 10; Larkspur alt 10; Fulmin semi 10; Tenet Plinx alt 10; EFV-5 Jupiter 15; Ambassador charge 16; Stahlta alt 20; Onos incarnon charge 50; Quellor alt 50; Tenet Tetra 80 ("Total damage is now based on ammo in mag on fire", U32); Azima turret 75; Balefire Charger 100 (energy/shields, exalted); Artemis Bow 15 energy; Regulators 0.
- For burst weapons whether `AmmoCost` is per shot or per burst (Bubonico "Burst" cost 3, BC 3) is UNVERIFIED from data alone.
- Mag-burst alt-fires consume the whole magazine.

### 8.3 Ammo efficiency (AE)
- `cost_eff = cost × (1 − AE)`; sources add; "ammo efficiency is hard capped at +75%" (since U44.1.3); some sources grant **Infinite Ammo** instead (Void relic buff, Magnus/Akmagnus Prime Chamber Cascade, Athodai Overdrive, Dual Toxocyst Frenzy, Knell Death Knell, Perigale Gale Force, Vasto Incarnon Reaper's Plenty, Arcane Pistoleer after U44.1.3, Brain Storm, Skull Shots) (`Ammo` § Ammo Efficiency and U44.1.3 notes). Infinite ammo ⇒ no reloads (sustained = burst while active).
- "Ammo efficiency and infinite ammo buffs do not affect the ammo usage of transformed Incarnon weapons" (`Ammo`).
- Fractional ammo is tracked internally; HUD shows the ceiling; a shot is allowed with 0 < ammo < 1, carrying "ammo debt" into the next magazine. Shots per magazine therefore alternate between floor and ceil of `Mag / cost_eff`, averaging exactly `Mag / cost_eff` (`Ammo` § Ammo Efficiency; Vectis +25 % AE worked example: 4 shots per 3 magazines). Example: Mag 8, +65 % AE → 22.86 → 22 or 23 shots.
- Use the unfloored average `N = Mag_mod / cost_eff` for long-run sustained DPS (contrary to `Weapons.lua`'s floor).
- Interaction with first-shot mods: Primed/Charged Chamber apply while "the magazine counter is at Max Magazine − 1 after a shot is fired" (Vulkar + Energized Munitions: shots 4–7 buffed) (`Primed Chamber`, `Charged Chamber` § Notes).

Worked example: Strun Mag 6 (+Ammo Stock → 10), 0 AE, FR 2.5 → T_mag = 4 s.

---

## 9. Reload

### 9.1 Standard reload
`Reload_mod = Reload_base / (1 + RS)` (`Reload` § Calculating Reload Time): Braton 2.0 s + Fast Hands (+30 %) → 1.54 s; Tainted Mag (−30 %) → 2.0/0.7 = 2.857 s.

### 9.2 Per-shell reload (data `ReloadStyle = "ByRound"`: Strun, Mk1-Strun, Strun Prime, Strun Wraith, Corinth, Felarx, Zarr, Kuva Zarr)
```
Reload = t_startend / (1 + RS) + t_shell / (1 + RS) × Mag_mod      (`Reload` § Weapons that reload per shell)
```
- Strun: 0.5 s start + 0.25 s end + 0.5 s/shell → 0.75 + 0.5 × 6 = 3.75 s (= data `Reload`) (`Strun` § Notes). Corinth: 1.2 + 0.22 × 5 = 2.3 s. Zarr: 0.75 + 0.5 per cannonball.
- Data stores only the full-empty reload time; the start/end and per-shell split are on weapon pages only (needed per weapon).
- Reload can be interrupted (fire partially loaded), so the engine's "full reload every magazine" is the worst case.
- Worked example: Strun with Ammo Stock (Mag 10) and Tactical Pump (+30 %): (0.75 + 0.5 × 10)/1.3 = 4.42 s; the `Reload` page notes Tactical Pump beats Ammo Stock on Strun/Corinth for this reason.

### 9.3 Battery (regenerating) weapons (data `ReloadStyle = "Regenerate"`, `ReloadDelay`, `ReloadDelayEmpty`, `ReloadRate`)
```
Reload (from empty) = ReloadDelayEmpty / (1 + RS) + Mag_mod / ReloadRate
```
- "Reload speed bonuses only affect the Recharge Delay while the Recharge Rate cannot be changed" and "Recharge Delay for battery weapons cannot go below 0.5s" (`Reload` § Rechargeable weapons; `Battery Weapon`). Data `Reload` for batteries equals delay + Mag/Rate: Flux Rifle 1 + 50/40 = 2.25 ✓.
- `ReloadDelayEmpty` differs from `ReloadDelay` (delay when not empty) on some weapons: Basmu 2 vs 0.2, Shedu 1 vs 0.4, Tenet Plinx 2 vs 0.8; Cycron delay halved when the magazine is not empty (U20.4). For "fire until empty" sustained DPS use `ReloadDelayEmpty` (falls back to `ReloadDelay`, `Weapons.lua` line 378).
- Batteries cannot regenerate between shots (U30.7 note), recharge while holstered/rolling/knocked down, ignore Cold slow, have no reserve or ammo pickup; Pax Charge turns kitguns into batteries with base delay = base reload (`Battery Weapon`; `Kitgun` § Tips). All Archwing / Necramech archguns are batteries; deployed (atmospheric) archguns use normal reload and reserve (`Battery Weapon`, `Archgun`).
- Worked example: Flux Rifle (FR 12, cost 0.5, Mag 50 → N 100, T_mag 8.33 s) with +30 % RS: Reload = 1/1.3 + 50/40 = 2.019 s; ratio 8.33/10.35 = 0.805. With +60 % magazine (80): Reload = 1 + 2 = 3.0 s, T_mag 13.33 s, ratio 0.816 (magazine barely helps a battery weapon).
- CONFLICT: U30.7 "Fixed Reload Speed Mods affecting the Shedu's ammo regeneration delay" suggests reload speed should NOT affect at least Shedu's delay, contradicting the general rule → open question.

### 9.4 Reload delay after the last shot
- "When firing a weapon until it is empty … there is a short Reload Delay equal to the reciprocal of the weapon's Fire Rate" (can be skipped by pressing reload; unavoidable on battery weapons) (`Reload` § Manual Reload). This is why `T_mag = N / FR` (N shots take (N−1)/FR, plus 1/FR before the reload starts). `Reload` page example: Viper "spends its entire Magazine in 0.97 seconds" = 14/14.4.
- `Damage/Calculation` sustained ratio: `N / (EFR × Reload + N)` ⇔ `T_mag / (T_mag + Reload)` with `T_mag = N/EFR`. "Reload time includes reload delay which is more prevalent for Battery Weapons."
- The wiki Lua (`Weapons.lua` CalcDamage) instead computes `avgDmgPerMag / (magDepletionTime + ReloadDelayEmpty + Reload) / tapShots` with its own `magDepletionTime` — a different (and for bursts, dubious) construction; not recommended as reference.
- Exceptions: magazine-1 weapons (§5.2), Vectis (§5.2).
- Worked example (from `Reload` § Reload Speed vs Fire Rate; note the page's Viper stats are outdated): Viper 14 rounds, 16 dmg, FR 14.4 → 21.31 with Gunslinger: T_mag 0.657 s, reload 1.1 → 224 dmg / 1.757 s = 127.5 DPS; Quickdraw (reload 0.74 s): 224/(0.972 + 0.74) = 130.8. Current data: Viper 17 dmg, FR 14.38, Reload 0.7 → the page example is stale.

### 9.5 Other reload mechanics
- Holster reload (fixed rate, not affected by reload mods); instant-reload effects; Incarnon form switching takes "an animation equal to the weapon's reload speed" and refills the normal magazine (`Lex Incarnon Genesis` § Overview; `Incarnon` § Mechanics).
- Reload-triggered buffs: Primary Plated Round `max bonus = 15 × sqrt(5 × Mag)` % (additive to Serration), scaled by fraction reloaded (`Primary Plated Round` § Damage Bonus).

---

## 10. AoE / radial attacks and multi-part attacks

### 10.1 Structure
- Data models a weapon as several attacks; an AoE child has `ShotType = "AoE"` and `AttackParentIndex` pointing to the direct-hit attack (Tonkor "Grenade Impact" 75 Puncture + "Grenade Explosion" 650 Blast; Opticor charged 1000 + 400 Magnetic AoE). Each component has its **own** base damage, CC, CD, SC, falloff.
- On a direct hit the target takes both: `AvgShot = AvgShot_direct + AvgShot_AoE` (`Damage/Calculation` § Single Target Direct AoE Hits). Since U26.0.7 AoE damage is a separate arsenal stat (previously merged into the hit) (`Launcher`).
- Weapon mods apply to each component on its own base (damage, elements, crit, status, faction). Exceptions:
  - Condition Overload / Galvanized "direct damage" bonuses do not apply to AoE (except the directly-hit target on hitscan explosions); details in `Condition Overload (Mechanic)` and `wiki-core-damage-formula.md` §11.
  - Synth Charge's last-shot bonus *does* affect the AoE (`Synth Charge` § Notes).
  - Hitscan explosions are not multiplied by multishot: Opticor ("Explosion isn't affected by multishot", listed under Bugs), Trumna ("Explosion is unaffected by multishot"), and the CO table notes "AoE does not scale off multishot" for Ambassador alt, Braton/Burston Incarnon radial, Ferrox, Tenet Ferrox, Glaxion Vandal, Opticor Vandal (`Condition Overload (Mechanic)` table). Projectile launchers create extra projectiles per multishot (Ogris: "difficult to see additional rockets generated by Multishot") — each presumably with its own explosion (implied, not stated → UNVERIFIED).
  - Continuous weapons' spherical AoE is unaffected by multishot (§4.2).
  - Some explosion radii cannot be increased by radius mods (Trumna: "cannot be increased with Firestorm").
- Each enemy in an AoE gets its own status roll (`Area of Effect` § Other Traits). Crit per enemy is not stated → UNVERIFIED (assumed independent).

### 10.2 Falloff
Data `Falloff = { StartRange, EndRange, Reduction }`; `Damage Falloff` § General:
```
f(d) = 1                                              d ≤ Start
f(d) = 1 − Reduction × (d − Start) / (End − Start)    Start < d < End
f(d) = 1 − Reduction                                  d ≥ End
```
- Range falloff (shotguns etc.): page example 200 dmg, 10–30 m, 80 % → 120 at 20 m. Projectile-speed mods scale falloff distances for weapons with falloff (hitscan without falloff ignore projectile speed).
- AoE falloff: Start = 0, End = radius ("linear from central impact"). Since U27.2 all AoE got 90 % falloff, then U27.2.2 set per-weapon values (e.g. Ogris 80 %, Tonkor 70 %, Penta 50 %, Opticor 60 %, Zarr 50 %, Kuva Bramma unchanged) (`Area of Effect` patch history); current values are in data (Tonkor `Reduction 0.7`, `EndRange 7`). Radius mods increase End (`Damage Falloff` § AoE Falloff).
- Bugs that change ordering: additive-CO bonus ignores range falloff; Incarnon base-damage evolutions ignore damage falloff (`Damage Falloff` § Bugs); Kuva/Tenet/Coda progenitor bonus always falls off 100 % → 0 % over the weapon's falloff range (see `wiki-core-damage-formula.md` §3.2).
- Worked example (Tonkor explosion 650 Blast, radius 7 m, 70 %): enemy at 3.5 m → 650 × (1 − 0.7 × 0.5) = 422.5; at the edge → 195.

### 10.3 Headshots and self-damage
- Since U32: "Radial damage no longer gains extra headshot damage or triggers headshot conditions" (aimed at Ignis Wraith); "Explosive projectiles themselves can still land headshots"; base headshot multiplier raised 2× → 3× (`Area of Effect` U32 notes; `Area of Effect` § Characteristics: "1x headshot multiplier … does not apply to the projectile itself"; `Damage/Calculation`: "AoE damage also cannot headshot"). Some weapons have no headshot bonus at all (Arca Plasmor, `Damage/Calculation`).
- Self-damage removed in U27.2, replaced by self-stagger (`Area of Effect` U27.2). No DPS impact.
- Hitscan range limit 300 m (snipers 1000 m) since U22 (`Damage Falloff` § Range Falloff).

---

## 11. Kitguns

- Three parts (`Kitgun` § Components): Chamber (projectile type, trigger, base damage types, base CC/SC, magazine table, falloff, multishot), Grip (primary vs secondary; damage and fire rate — for beam chambers grip sets range instead of fire rate), Loader (reload time, magazine tier, flat CC/CD/SC offsets).
- Stat assembly as stored in `Module:Modular/data` (`Modular_data.lua` `KitgunSecondary` / `KitgunPrimary`):
  - Damage: per chamber, a hard-coded damage table per grip (`Damage[grip]`). Observed pattern: each grip adds the **same flat amount to every damage type** of the chamber (Catchmoon secondary: Gibber {Heat 119, Impact 41} = Ramble {143, 65} − 24 each; Haymaker {269, 191} = +102 each vs Base {167, 89}). The module header says "according to Hotfix 28.0.5 … grips damage reduction is expressed as a vector rather than a scalar multiplier", matching U28.0.5 notes ("Brash: damage reduction lowered from −38 to −24 when used with Catchmoon"). Use the per-grip tables, not a formula.
  - Fire rate: per chamber per grip (`FireRate[grip]`); for Tombfinger primary the grip also sets charge time (U28.0.5: Brash 0.5 s, Steadyslam 1.1 s, Tremor 1.4 s, default 0.8 s); for Gaze/Vermisplicer the grip sets beam range (`Range[grip]`).
  - CC, CD, SC: `chamber value + loader offset` (additive, absolute). Check: Catchmoon CC 0.21 + Arcroid 0.03 = 0.24 = `Modular_data_kitgun.lua` "Catchmoon-Arcroid-Gibber" CC ✓; SC 0.21 + 0.03 = 0.24 ✓.
  - Magazine: loader gives a tier (Lowest/Low/Med/High/Highest/SuperHighest/MegaHighest/GigaHighest) looked up in the chamber's `Magazine` table (Catchmoon Med = 9).
  - Reload: loader value (0.9 / 1.3 / 1.5 / 1.7 / 2.1 / 2.3 / 2.9 / 3.1 s).
- `Modular_data_kitgun.lua` contains all assembled combinations (e.g. "Catchmoon-Arcroid-Gibber"), usable directly.
- Historical per-chamber loader exceptions existed (U-note: Thunderdrum on Catchmoon CC 13→17 %, SC 35→28 %), so the additive model may not hold for every combination → verify against the assembled table rather than computing.
- Pax Charge makes a kitgun a battery weapon (§9.3). Secondary Semi kitguns can equip Cannonade since U37.
- Worked example: Catchmoon (secondary) + Gibber + Arcroid → 160 dmg (119 Heat + 41 Impact), FR 3.17, CC 24 %, CD 2.0, SC 24 %, Mag 9, Reload 1.5 s, falloff 8–16 m to 6.25 %.

---

## 12. Arch-guns

- Archwing (space) mode: regenerating magazine, no reserve; "almost all zero-gravity Archguns have Damage Falloff and projectile travel time" (`Archgun` § Archwing). Deployed / atmospheric (Gravimag) mode: normal magazine + reserve, Heavy Ammo only, "lack damage falloff, and most of them are hit-scan"; separate stats — the data module has separate entries "X (Atmosphere)" (e.g. Grattler vs Grattler (Atmosphere); Corvas atmospheric multishot 11). U35 doubled most Heavy Weapon damage (atmospheric only).
- Necramech archguns use the regenerating system (`Archgun` § Necramech).
- Arch-gun mods are a separate pool (`Archgun Mods`: Rubedo-Lined Barrel, Dual Rounds, Critical Focus, Parallax Scope, Automatic Trigger, Magazine Extension, Ammo Chain, elemental, Primed variants…); some rifle mods are also archgun-compatible (marked in `Multishot` lists). Formulas are the same as other guns.
- Arcanes (since U40, Archgun Arcane Adapter): one Primary and one Secondary arcane slot, but "Archguns are not considered either primary or secondary weapons, and will thus not be affected by Arcanes such as Arcane Precision or Arcane Acceleration"; universal weapon arcanes (Arcane Avenger, Hot Shot) do apply (`Archgun` § Archwing).
- Charged archguns (Corvas, Mandonel, Velocitus, Larkspur alt) follow the charge rules (§3). Spooling archguns: Grattler, Phaedra, Imperator Vandal (§6).
- Profit-Taker has 50 % archgun damage resistance since U35 (enemy-side; see enemy findings).

---

## 13. Incarnon forms

- Incarnon Form is a separate attack in data (`AttackName = "Incarnon Form"`, own base damage/CC/CD/SC/FR/trigger, `IncarnonCharges` = its magazine). Often with a child radial attack ("Incarnon Form Radial Attack", AoE).
- Ammo: separate "magazine" filled by Transmutation charges (e.g. Braton: 10 rounds per charge, max 200; Lex: 2 per charge, max 20), "not affected by mods" and "not affected by Ammo Efficiency"; when empty the weapon reverts; manual exit wastes remaining charges; exiting refills the normal magazine "as if it were reloaded" (`Braton Incarnon Genesis`, `Lex Incarnon Genesis`, `Incarnon` § Mechanics, `Ammo`). There is no reload inside Incarnon form. `Damage/Calculation`: "For Incarnon Form, use Max Incarnon Charge [as shots per magazine]. Ammo Cost is 1 per shot".
- Proposed DPS: burst DPS from the form's own attack; "sustained" over one form activation = burst (no reload term); optionally add switch time (= reload time) at both transitions.
- Charge gain: weakpoint hits (most primaries/secondaries; individual multishot projectiles count) or direct hits (Angstrum, Torid); melee: 6× combo + heavy attack, 180 s (90 s for some) (`Incarnon` § Mechanics). `IncarnonChargeGain` in data. Uptime modelling is out of scope of the wiki.
- Evolutions: one perk per tier (2–3 choices). Typical wording since the description clean-up: "Increase Base Critical Chance by +X%", "Increase Base Status Chance by +X%", "Increase Base Critical Damage Multiplier by +X", "Increase Base Damage by +X" (`Incarnon` patch history). "The critical damage increase is added as base critical damage and will be affected by mods" (`Braton Incarnon Genesis` notes), i.e. `CC = (CC_base + X) × (1 + ΣCC_mods)`. Status bonuses apply per projectile (Elemental Balance changed to "each projectile instead of being divided by the Multishot count").
  - Some perks apply only to the normal form ("The bonus does not apply to the Incarnon form", Braton Evolution II/III notes) — per-perk mode flags required.
  - Conditional perks (on headshot, while channeling, below 50 % CC, on reload from empty, last shot in magazine (Torid "+3 Multishot on last shot in magazine")) need toggles.
  - Braton "Multishot consumes ammo … does +Y Damage": "a unique modifier that is multiplicative to other damage buffs", only for multishot-generated shots; its multishot bonus "stacks additively with multishot mods" (`Braton Incarnon Genesis` lines 72–75).
  - Base-damage evolutions ignore damage falloff (bug, `Damage Falloff`).
- Cannonade on Incarnon only if both modes are Semi (§1.3). Synth Charge has no effect on Incarnon forms (`Synth Charge`).
- Worked example (Lex Incarnon: FR 0.67, 1000 dmg/shot, 20 rounds): one activation = 20 shots over 20/0.67 = 29.9 s; sustained = burst = 1000 × avg_crit × 0.67 per second; plus 2 × 2.35 s switch time if modelled.

---

## 14. Exalted weapons (ability-strength scaling)

Three distinct models found:
1. **Base damage × Ability Strength** (fully multiplicative with weapon damage mods): Artemis Bow (75/120/180/240 per arrow, 3/4/6/7 arrows, 15 energy per shot), Balefire Charger (100–500 uncharged, up to 300–1500 at 2 s charge), Dex Pixia (50/80/120/160), Noctua (175–250 primary, 2000–2750 alt), Glory (75–150, alt 750–1500), Exalted Blade (melee). `Razorwing`: "Because Ability Strength modifies the base damage properties of the weapon, damage bonuses like Hornet Strike … are fully multiplicative with the effects of Ability Strength mods." Bug: Galvanized Shot on Noctua scales from 200/2500 regardless of strength (`Noctua`).
   - Worked example: Artemis Bow rank 4 at 200 % strength: 240 × 2.0 = 480 base per arrow × 7 arrows = 3360 per shot before mods.
2. **Strength-scaled damage multiplier additive with base-damage mods**: Neutralizer (Cyte-09) and Lizzie (Temple): multiplier 1/1.08/1.16(1.15)/1.25 × Strength, "additive to base damage mods such as Serration and Galvanized Aptitude" (`Neutralize`, `Exalted Solo`). Exact bucket form (`1 + (m·S − 1) + Σmods`?) not given → UNVERIFIED.
3. **Peacemaker (Regulators)**: fixed base 50 (50 % Impact / 25 % Puncture / 25 % Slash), innate bonus +100/115/133/150 % × Strength in the base-damage bucket, plus a multiplicative ramp of +7.5 % per burst up to +150 % at 20 bursts; fire rate ramps 25 % → 100 % over 3.5 s (multiplied with other FR). Formula (`Peacemaker`):
   `Damage = 50 × (1 + 1.5 × min(bursts,20)/20) × (1 + 1.5 × Strength + DamageMods) × (1 + ElementalMods)`.
   Worked example (page): rank 3, Intensify (+30 %), Hornet Strike (+220 %), Convulsion (+90 %), 8th burst: 50 × 1.6 × (1 + 1.95 + 2.2) × 1.9 = 782.8. Peacemaker snapshots some buffs at cast. Regulators count as secondaries for arcanes/shards.
- General: "Damage dealt by Exalted Weapons depends on the rank of the weapon's ability. The weapon's rank only affects mod capacity"; no ammo (energy instead; Cyte-09 uses both); Riven mods not generated (`Exalted Weapon` § Notes). Artemis Bow is a primary for mod/arcane purposes; charging only reorients arrows (no damage change without Concentrated Arrow); charge speed affected by fire rate (`Artemis Bow`). U44.0.3 changed only damage-number colours.
- Neutralizer has sniper-style zoom crit multiplier bonuses (+40/60/80 %, `Neutralize`).

---

## 15. Companion (robotic) weapons

- Each robotic weapon is typed (Rifle, Sniper, Shotgun, Pistol, Melee) and uses that type's mods; "Most mods work as normal, including … Fire Rate, base damage, Multishot, Critical Hits, Elemental Damage, Magazine Size, Reload Speed and Status Chance"; ammo-maximum mods do nothing because reserves are unlimited; no Exilus, no arcanes (`Sentinel` § Robotic Weapons). Charged Chamber gives no bonus on sentinel weapons (`Charged Chamber`).
- Data in `Weapons_data_companion.lua` uses the same schema (e.g. Helstrum burst BC 4, BD 0.1; Multron BC 3, BD 0.4; Artax/Tazicor beams cost 0.5; Deth Machine Rifle auto-spool).
- AI firing behaviour (target acquisition, pauses) is not documented → any sentinel DPS is a theoretical upper bound (UNVERIFIED).

---

## 16. First-shot and last-shot effects

| Effect | Value | Rule | Source |
|---|---|---|---|
| Primed Chamber (sniper) | +100 % damage on first shot in magazine | multiplicative after all other mods/abilities; applies to all multishot hits and status damage; additive with Charged Chamber (max +140 %) | `Primed Chamber` § Notes; Mods_data |
| Charged Chamber (sniper) | +40 % | same; not on sentinel weapons | `Charged Chamber` |
| Synth Charge (set mod) | +200 % "Bonus Damage on final shot", requires base magazine ≥ 6 | multiplicative with Hornet Strike; affects AoE; not on beams (except Catabolyst reload grenade), not on Incarnon forms, not on whole-mag alt-fires (U32.3, U33) | `Synth Charge` |
| Torid Incarnon | +3 multishot on last shot in magazine | conditional perk | `Torid Incarnon Genesis` |
| "On reload from empty" perks (Ready Retaliation etc.) | per perk | trigger once per magazine | `Incarnon` patch history |

- For DPS: per magazine of N shots, first-shot multiplier contributes `(1 + b)` on 1 shot and 1 on N − 1 → average factor `1 + b/N`; Vectis (N = 1) gets it on every shot (`Charged Chamber` § Tips). With AE, "first shot" = every shot fired while the counter reads Mag − 1 after firing (§8.3).

---

## Edge cases

1. Burst delay scales with positive fire rate but not with net negative fire rate; the wiki's displayed formula omits the scaling (§2).
2. Bows get ×2 from fire-rate *mods*; unknown for non-mod sources (§1.5).
3. Semi-auto cap 10/s (Komorex ≈ 8/s); Cannonade locks fire rate to base, ignoring negatives (§1.2–1.3).
4. Charge time can't exceed 10× base; FR floor 0.05/s (§1.1, §3.1).
5. Lanka has no inter-shot delay (EFR = 1/CT); bows use reload as inter-shot time; Epitaph ignores reload (§3.2, §5).
6. Vectis has no reload delay (cycle = reload); other mag-1 weapons may or may not (conflict) (§5.2).
7. Lanka zoom crit is flat, added after mods (exception to additive zoom buffs) (§5.4).
8. Sniper combo counts each multishot projectile; AoE/DoT don't count (§5.4).
9. Beams: multishot merges into one instance — summed damage and status chance, single crit roll; DoT scales twice with multishot; forced procs once per tick (§4.2).
10. Beam AoE (Ignis, Glaxion Vandal…) not affected by multishot; hitscan explosions (Opticor, Trumna, Ferrox, Braton/Burston Incarnon) not affected by multishot; launcher projectiles are (§4.2, §10.1).
11. Beam ramp 20 % → 100 % over 0.6 s; decays after 0.8 s idle over 2 s; reloads longer than 0.8 s cost damage (§4.3).
12. Beams cost 0.5 ammo per tick; ramp start differs (Convectrix 60/80 %, Phage 70 %, Embolist 30 %) (§4.1, §4.3).
13. Kohm family: pellets spool up and per-pellet status chance = listed / pellet count; ammo per shot rises to 4 (§6).
14. Shotgun status per pellet changed in U27.2 (old formula `1 − (1−SC)^(1/n)`) (§7).
15. Spearguns: multishot doesn't affect the throw (§7).
16. Magazine rounding: nearest vs ceil ambiguous; minimum 1; percentages before flat additions (§8.1).
17. Ammo efficiency capped at 75 % (U44.1.3); infinite-ammo sources separate; neither affects Incarnon forms; fractional "ammo debt" (§8.3).
18. Per-shell reloaders: magazine mods increase reload time (§9.2).
19. Battery weapons: reload speed affects only the delay (min 0.5 s); delay may differ when not empty; Shedu note conflicts (§9.3).
20. Mag-burst alt-fires: burst count = remaining magazine (§2).
21. AoE cannot headshot (since U32), projectiles can; headshot base 3× (§10.3).
22. Direct + AoE are separate instances with separate stats; CO/Galvanized not on AoE; Synth Charge is (§10.1).
23. Some AoE radii cannot be modded (Trumna) (§10.1).
24. Incarnon forms: own magazine (no mods, no AE), no reload, switch time = reload time; some perks apply to one mode only (§13).
25. Exalted weapons: three different strength-scaling models (§14).
26. Archguns: Archwing vs atmospheric stats differ; atmospheric lack falloff; archguns aren't primary/secondary for arcanes (§12).
27. Kitguns: grip damage is a per-type flat offset (per-chamber table), loader CC/CD/SC are additive offsets (§11).
28. Spool weapons: fire-rate mods scale the spool curve; Kuva Grattler has no spool (§6).
29. Incarnon "base damage +X" ignores falloff; progenitor bonus falls off 100 % → 0 % (§10.2).
30. Quick shots are separate attacks with separate stats (Paris, Lanka, Opticor); bow quick shots share crit/status with charged shots since U25.7 (§3.3).

## Open questions

1. Burst delay: does positive ΣFR divide BD exactly by (1+ΣFR)? (§2) UNVERIFIED numerically.
2. Bow ×2 fire-rate rule: does it apply to arcanes/abilities, and to crossbows? (§1.5)
3. Semi-auto cap: exact value, frame-rate dependence, which multi-mode weapons it applies to (§1.2).
4. Duplex (Tigris, Zylok): `Fire Rate` says FR caps "how quickly each two-shot sequence can be initiated", `Damage/Calculation` says EFR = FR per shot — which? (Tigris FR 2, Mag 2: 2 shots per 0.5 s vs 1 s.)
5. Magazine-1 weapons other than Vectis: reload delay (1/FR) or not? (§5.2)
6. Epitaph effective fire rate formula (§5.3).
7. Magazine rounding: nearest (half up) or ceil? (§8.1)
8. Beam ramp curve shape (linear?) and current per-weapon start values (§4.3).
9. Spool: interval after shot k uses rate_k or rate_{k+1}; when the spool resets (reload, idle time) (§6). Secura Dual Cestra cadence mismatch.
10. Shedu/battery: does reload speed affect regeneration delay? (U30.7 note vs general rule) (§9.3)
11. `BurstReloadDelay` meaning (§2); AmmoCost per shot vs per burst for burst weapons (§8.2).
12. Partial-charge damage: interpolated or fixed quick-shot stats (§3.3).
13. Do launcher multishot projectiles each create a full explosion? Is crit rolled per enemy for AoE? (§10.1)
14. Exalted model 2 exact formula (Neutralizer, Lizzie) (§14).
15. Kitgun loader offsets: universally additive or chamber-specific in some combinations (§11).
16. The `Reload` page's Viper example is stale (reload 1.1 s vs data 0.7 s); other examples on mechanic pages may be similarly outdated.

## Sources

wiki.warframe.com pages (raw wikitext cached in `research/external/cache/wiki/`): Fire Rate, Trigger Type, Continuous Weapon, Multishot, Damage/Calculation, Reload, Ammo, Battery Weapon, Area of Effect, Damage Falloff, Self-Damage, Shotgun, Bow, Launcher, Speargun, Sniper Rifle, Kitgun, Archgun, Archgun Mods, Exalted Weapon, Incarnon, Braton Incarnon Genesis, Lex Incarnon Genesis, Torid Incarnon Genesis, Calculating Bonuses, Cannonade Mods, Semi-Rifle Cannonade, Speed Trigger, Magazine Warp, Ammo Stock, Wildfire, Tainted Mag, Slip Magazine, Burdened Magazine, Depleted Reload, Primed Chamber, Charged Chamber, Synth Charge, Primary Plated Round, Condition Overload (Mechanic), Galvanized Shot/Aptitude/Savvy, Sentinel, Robotic Mods, Artemis Bow, Peacemaker, Balefire, Razorwing, Noctua, Glory on High, Neutralize, Exalted Solo, weapon pages (Gorgon, Gorgon Wraith, Prisma Gorgon, Soma, Soma Prime, Supra, Supra Vandal, Tenora, Aksomati, Kohm, Kohmak, Twin Kohmak, Cestra, Secura Dual Cestra, Grattler, Phaedra, Imperator Vandal, Lanka, Opticor, Paris, Tigris, Vectis, Strun, Corinth, Zarr, Felarx, Komorex, Arca Scisco, Ferrox, Trumna, Ogris, Tonkor, Pandero, Nagantaka, Cyanex, Kuva Kraken, Burston Prime, Kuva Weapons); search snippets for Tenet Flux Rifle and Basmu.
Wiki Lua modules (cached in `research/external/cache/wiki-modules/raw/`): `Weapons.lua` (StatObject: BurstsPerSec, InterShotTime, EffectiveFireRate, ShotsPerMag, CalcDamage), `Weapons_data_primary/secondary/archwing/companion/misc.lua`, `Modular_data.lua`, `Modular_data_kitgun.lua`, `Mods_data.lua`.
Inspiration `guide.txt` was glanced at only to confirm which mechanics it models (burst-delay fix 2026-01-28); no claims here rest on it.
