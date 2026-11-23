Status: draft

# Damage types and status effects (game-truth research)

Scope: every damage type and its status effect (proc) as the game behaves in late 2026 (current live build is Update 44.x; the inspiration's changelog cites "Hotfix 44.1.3", 2026-10-23). Sources are wiki.warframe.com pages read via `?action=raw` on 2026-11-23 and cached in `research/external/cache/wiki/` (file name = page name with `/` replaced by `__`, extension `.wiki`). Patch-note quotes are the ones reproduced in each page's "Patch History" section; update numbers are given as `U<n>`.

Abbreviations: SC = status chance, SD = status duration bonus (e.g. +1.1 for +110%), CD = critical damage multiplier, OG = Overguard, IPS = Impact/Puncture/Slash, "page X §Y" = wiki page X, section Y.

Pages used (all wiki.warframe.com): Status_Effect, Damage, Damage/Calculation, Damage_over_Time, Damage/{Impact,Puncture,Slash,Heat,Cold,Electricity,Toxin,Blast,Corrosive,Gas,Magnetic,Radiation,Viral,Void,Tau,True,Cinematic}_Damage, Microwave, Overguard, Shield, Critical_Hit, Enemy_Body_Parts, Faction_Damage_Bonus, Damage_Vulnerability, Condition_Overload_(Mechanic), Archon_Shard, Lasting_Sting, Multishot, Bosses, Evensong.

Page freshness (from the MediaWiki revisions API, 2026-11-23): Status_Effect edited 2026-11-20, Heat 2026-11-14 (Heat-inherit rewrite), Damage/Calculation 2026-11-04, Blast 2026-11-03, Damage_over_Time 2026-10-12, Overguard 2026-10-01, Cold 2026-09-30. Several summary tables, on Status_Effect and Damage especially, are older than the per-type pages and contradict them. Where they conflict, this file follows the per-type page and records the conflict.

---

## 1. Summary table

| Type | Proc name (internal) | Base duration | Max stacks (normal enemy) | Stack timers | Damage per proc | Debuff / effect |
|---|---|---|---|---|---|---|
| Impact | Knockback / Stagger (`PT_KNOCKBACK`) | 6 s per stack (flinch about 1 s) | 5; 6th replaces oldest | independent | none | Stagger; heavy units: Parazon mercy threshold +8%/stack (max 80%) |
| Puncture | Weakened (`PT_FRAILTY`) | 10 s | 5; 6th replaces oldest | independent | none | Enemy damage −40% (+10%/stack, max −80%); **+5% flat crit chance per stack (max +25%)** on weapon hits received |
| Slash | Bleed (`PT_BLEEDING`) | 6 s (6 ticks, first at 1 s) | unlimited | independent | 0.35 × modded base per tick, Cinematic type | ignores armor, no faction type modifier |
| Heat | Ignite (`PT_IMMOLATION`) | 6 s (6 ticks, first at 1 s) | unlimited | **all stacks share one timer, refreshed by each new proc** | 0.5 × modded base × (1+Heat mods) per tick, all stacks summed into one tick | Armor −50% (ramps over 2 s, ramps back over 6 s after it ends); panic |
| Cold | Freeze (`PT_CHILLED`) | 6 s; Frozen 3 s | 10 (10th = Frozen) | independent | none | Slow 50% (+5%/stack to 90%); **+0.10 CD (+0.05/stack to +0.50) received**; Frozen: +1.0 CD |
| Electricity | Tesla Chain (`PT_ELECTROCUTION`) | 6 s (6 ticks, first at 0 s) | unlimited | independent | 0.5 × modded base × (1+Elec mods) per tick to target and enemies within 3 m | Stun of the primary target for 3 s (not extended by SD) |
| Toxin | Poison (`PT_POISONED`) | 6 s (6 ticks, first at 1 s) | unlimited | independent | 0.5 × modded base × (1+Toxin mods) per tick | Ticks bypass shields (not OG) |
| Blast | Detonate (`PT_FLASHBANG`) | 1.5 s fuse | 10 (10th detonates all) | independent | 0.3 × modded base when the fuse expires; at 10 stacks or on death, 3× per stack to others within 5 m | none |
| Corrosive | Corrosion (`PT_CAUSTIC_BURN`) | 8 s | 10 (+2 Emerald shard, +3 Tauforged); new stack replaces oldest | independent | none | Armor −26% at 1 stack, +6%/stack, −80% at 10 |
| Gas | Gas Cloud (`PT_ASPHYXIATION`) | 6 s (6 ticks, first at 0 s) | 10; new stack replaces oldest | independent | 0.5 × modded base × (1+Gas mods) per tick in a 3 m radius (grows with stacks to 6 m) | cloud persists if the host dies |
| Magnetic | Disrupt (`PT_MAGNETIZED`) | 6 s | 10; new stack replaces oldest | independent | none directly; forced Electricity proc when shields/OG break | Damage to shields **and OG** ×(2 + 0.25(n−1)), max ×4.25; no shield regeneration |
| Radiation | Confusion (`PT_RAD_TOX`) | 12 s | 10; new stack replaces oldest | independent | none | Enemy fights its allies; +100% (+50%/stack, max +550%) damage it deals to its allies |
| Viral | Virus (`PT_INFECTED`) | 6 s | 10; new stack replaces oldest | independent | none | Damage to **health** ×(2 + 0.25(n−1)), max ×4.25 (not shields, not OG) |
| Void | Bullet Attractor (`PT_RADIANT`) | 3 s | not documented | not documented | none | 2.5 m field that pulls projectiles to the body part where the proc landed |
| Tau | Status Vulnerability (`PT_SENTIENT`) | 8 s | 10; new stack replaces oldest | independent | none | SC of incoming hits ×(1 + 0.10n), max ×2.0 |
| True (`DT_HEALTH_DRAIN`) | none | – | – | – | – | Damages health and OG; ignores armor and shields; no faction modifier |
| Cinematic (`DT_CINEMATIC`) | none (it is what Bleed deals) | – | – | – | – | Damages health, shields and OG; ignores armor; no faction modifier |
| Finisher (`DT_FINISHER`) | none | – | – | – | – | Bypasses armor; damages health, shields and OG |
| Microwave (`PT_MICROWAVE_BURN`) | Microwave | infinite | – | – | none | Enlarges the body part hit; Nukor and Kuva Nukor only; counts as one status for Condition Overload |

Sources: per-type pages §"Status Effects"; Status_Effect §General; Damage §Damage Types; Damage_over_Time §Damage over Time Effects. Each row's detail and its sources are in §4.

---

## 2. How a proc is chosen

### 2.1 Number of procs per hit
- Each damage instance (each pellet or projectile) rolls SC independently (Multishot page, intro; Status_Effect §Multishot). Since U27.2 a shotgun's SC is the per-pellet chance shown in the Arsenal. Before U27.2 it was `1 − (1 − SC)^(1/pellets)` (Status_Effect §Trivia).
- SC above 100% gives extra procs. The type of each proc is drawn independently, so the same status can be applied more than once in one hit (Status_Effect §Status Chance: "200% Status with only Heat and Impact ... can still trigger two heat or two impact").
- Expected procs per shot = Multishot × (Forced procs + SC per projectile) (Status_Effect §Average Procs). The wiki gives no exact sampling rule for SC > 100%. The rule consistent with that expected value and with the critical-tier analogue is `procs = floor(SC) + Bernoulli(frac(SC))`. **UNVERIFIED** as a sampling rule; the expected value is sourced.
  - Example: SC 250% → 2 guaranteed procs plus a 50% chance of a third; expected 2.5 per pellet.
- Continuous (beam) weapons: beams from multishot that hit the same target merge into one tick. Its damage and SC are the sum of the beams' values, while crit chance stays at one beam's value. Example: 40% SC with multishot 2.5 gives a tick with 80% SC when multishot rolls 2 and 120% when it rolls 3 (Multishot page lines 19–29).
- Forced procs (stance combos, heavy attacks, weapon traits) are extra procs independent of SC, and a forced-proc attack can also roll a normal proc (Status_Effect §Forced Procs). Tau vulnerability does **not** add extra forced procs (Tau page).
- Multiple enemies hit by one attack each roll separately (Status_Effect §Status Chance).

### 2.2 Proc type weighting
- `P(type) = damage of that type / total damage` (Status_Effect §Damage Distribution).
  - Example (wiki): 20 Impact, 5 Puncture, 10 Slash, 25 Heat, 50 Corrosive (total 110) → 18.18% / 4.55% / 9.09% / 22.73% / 45.45%.
- The weights are the weapon's **modded** per-type damage after elemental combination. Heat and Cold mods raise the chance of Blast procs (Blast page §Notes), and the inspiration guide's Heat-versus-Viral example points the same way. A primary element that has been combined is gone: it deals no damage and has no proc (Damage §Elemental Damage). Physical mods change the IPS weights.
- The weights are **not** changed by enemy damage-type modifiers or by armor (Status_Effect §Status Immunity Interactions).
- Status immunity: if the enemy is immune to a proc type, that type is **removed and the remaining weights are renormalized**. Example (wiki): the same weapon against a Corrosion-immune enemy → 33.33 / 8.33 / 16.67 / 41.67 / N/A.
- **Exception, no renormalization:** against a ragdolled enemy, a proc that rolls Impact inflicts **nothing** (Impact page §General). Hemorrhage and Internal Bleeding can still trigger.
- Before U27.2, physical procs had 4× weight (Status_Effect §Trivia). That rule is obsolete.
- Open: whether the weights use the 1/32-quantized per-type values or the unquantized ones (see Open questions).

### 2.3 Status duration
- Status-duration mods (e.g. Lasting Sting, +110% at rank 10) scale each proc's duration. They do **not** extend crowd control: the Impact stagger animation, the Heat panic and the Electricity stun stay the same length. They **do** extend Cold's Frozen state (Lasting Sting §Notes; Status_Effect §Status Duration table).
- More elemental damage does not lengthen procs (Status_Effect §Damage Distribution).
- For DoTs, SD adds ticks. Lasting Sting §Notes: "Due to status duration always rounding down, there is no difference in performance between a 9th rank and 10th rank version" (+100% → 12 s, +110% → 12.6 s; both give 12 ticks). Working rule: `ticks = floor(6 × (1 + SD))` for Slash, Heat, Toxin, Electricity and Gas. **UNVERIFIED** for fractional durations; see the conflicting formulas below.
  - Damage_over_Time §Calculating gives `Total Ticks = floor(TickRate × (Duration − Delay)) + 1`. With Duration 6 and Delay 1 (Slash) this gives 6, which matches. With Delay 0 (Electricity, Gas) it gives 7, which does not match the tick tables (6 ticks, "last tick deals no damage").
  - Damage/Calculation §DoT final calculations uses `Total Ticks × (1 + Status Duration Bonuses)` without flooring. That version is an averaging approximation.
- Heat: SD **slows** the armor-strip ramp (+100% SD → 50% strip after 4 s instead of 2 s; −50% SD → after 1 s). See §4.4.
- Blast: SD lengthens the fuse, which delays the damage (Blast page).
- Negative SD (−100% or lower, only possible with Rivens): duration effects and DoTs are nullified, while instant effects still happen. Blast still deals its expiry damage to the target, and the AoE only happens if the triggering hit kills. Electricity's chain happens but not its stun (Status_Effect §Negative Status Duration). The wiki marks this table "outdated" and based on a 2016 forum post. **UNVERIFIED** for current behaviour.
- Whether non-DoT debuff durations (Corrosive 8 s, Viral 6 s, ...) are also floored to whole seconds is unknown. **UNVERIFIED.**

### 2.4 Global and per-enemy stack caps
- The wiki documents no global cap on total statuses per enemy. Caps are per status type (table above). Slash, Toxin, Electricity and Heat are unlimited on normal enemies. Only 10 Slash and Toxin tick numbers are *displayed*; damage is not affected (U27.2 note on the Slash and Toxin pages).
- "Status-capped" enemies (bosses, adversaries such as Kuva Liches, Sisters and Acolytes, and some VIPs) have per-status caps of 4:
  - U27.3 (Bosses page patch history): for Kuva Liches, "No Status Effect will exceed a maximum of 4 stacks, with the exception of Impact which can stack up to 6 times". That Impact exception predates the U33.6 cut of Impact to 5 maximum stacks, so it is outdated.
  - Cold: bosses and enemies with active OG max 4 stacks, so they cannot be Frozen (Cold page; U36).
  - Radiation: Liches, Acolytes, enemy Necramechs, Sisters and Hounds max 4 stacks (Radiation page).
  - Blast: bosses cap at 4 stacks, and reaching the cap detonates (Damage/Calculation §Blast).
  - Corrosive: the Emerald shard's +2/+3 "is applied to Status Capped enemies" (Archon_Shard §Emerald).
  - Evensong page: "Enemies that are usually capped at 4 stacks of a status, such as Acolytes and enemies with Overguard".
- **UNVERIFIED:** whether the boss cap of 4 also limits unlimited DoT stacks (Slash, Toxin, Electricity, Heat). The U27.3 wording ("No Status Effect") suggests it does. No current page confirms it.

---

## 3. Shared DoT rules (Slash, Heat, Toxin, Electricity, Gas, and Blast's delayed hit)

### 3.1 Tick formula
For a proc created by a hit (Damage_over_Time §DoT Damage Scaling; formulas on each type page):

```
ModdedBase   = BaseDamage(total of all types, unmodded) × (1 + BaseDamageBonuses) × (1 + Faction)
Tick         = C × ModdedBase × (1 + ElementBonus_X) × (1 + Faction) × (1 + StatusDamageBonuses)
               × CritMult(rolled tier of the inducing hit) × BodyPartMult(inducing hit)
               × [target-side multipliers at tick time]
C            = 0.35 Slash | 0.5 Heat, Toxin, Electricity, Gas | 0.3 Blast (single target), 3.0 Blast (AoE)
ElementBonus = 0 for Slash and Blast
```

What the tick inherits from the inducing hit, with sources:
- **Base-damage mods** (Serration, Vex Armor's Fury): yes, through ModdedBase.
- **Total-damage multipliers** (Furious Javelin, Eclipse) and **enemy debuffs** (Molecular Prime, Rest & Rage): yes (Damage_over_Time §DoT Damage Scaling). Damage_Vulnerability §Mechanics: a vulnerability applies to the target, so it does not double dip; it applies once, to the tick.
- **Elemental mods of the proc's own element:** yes, for Heat, Toxin, Electricity and Gas only. Physical mods (Sawtooth Clip, Buzz Kill) do **not** affect Slash ticks. Mods of a combined element's components do **not** affect Gas or Blast procs (Thermite + Infected Clip does nothing for the Gas DoT); only mods that add the combined element itself (Leaded Gas, Valence Formation set to Gas) affect the Gas DoT. Blast: see §4.8 (no elemental scaling at all).
- **Reverse case:** a forced Toxin proc (e.g. Toxic Lash) **is** buffed by Toxin mods even when those mods were combined into Corrosive on the weapon (Damage_over_Time §DoT Damage Scaling). So ElementBonus_X is the sum of all X-element mods, combined or not.
- **Faction damage (Bane, Expel, Roar):** applied **twice**, once in ModdedBase and once on the tick, so the effective bonus is `(1+F)^2` (+69% for 0.30, +140.25% for 0.55) (Faction_Damage_Bonus intro; every DoT page). Exception: Heat inherit, §4.4.
- **Status damage** (Elementalist mods, Emerald shard for Toxin only, Ash for Slash only, Conductive Sphere for Electricity only, Riven Splicer trait): additive with each other and multiplicative with everything else (Status_Effect §Status Damage). It applies to Slash, Heat, Toxin, Electricity, Gas and Blast.
- **Critical hit:** the tick uses the **rolled crit tier** of the inducing hit (Damage/Calculation §DoT: "Critical DoTs use the effective critical multiplier of the inducing hit", with critical-multiplier quantization). Tier k multiplier = `1 + k × (CD − 1)`.
- **Headshot and body-part multiplier** of the inducing hit: yes. **Sonar and Detect Vulnerability weak spots: no**, only the initial hit (Slash page; Damage_over_Time).
- **Stealth bonus, melee combo multiplier, sniper combo multiplier:** yes (Damage_over_Time).
- **Finisher-damage mods** (Finishing Touch, Savage Silence): no (Slash page).
- **Target-side, evaluated at tick time:** enemy damage-type modifier for the tick's own type (Heat, Toxin, Electricity, Gas and Blast are typed; Slash ticks are Cinematic and get **no** faction modifier); armor (Slash ignores it); Viral (health) and Magnetic (shields/OG) multipliers; Damage Vulnerability. The Toxin page lists "Damage Type Modifiers and Damage Vulnerability" among the multipliers. Viral is checked when each tick lands, not when the proc is created (Viral page §Status Effect).
- **Not covered by any source:** whether Cold's received CD bonus and Puncture's flat crit chance raise the crit of the inducing hit and so carry into its DoT. Puncture raises the inducing hit's crit chance, so it plausibly carries over; Cold applies "before critical tier is calculated". Cold's bonus "does not apply to AoE damage" and Puncture's does not either. **UNVERIFIED** for the Electricity and Gas AoE portions.

### 3.2 Worked examples
1. **Slash** (Slash page): base 100, Serration +165%, Bane of Grineer +30%, Rifle Elementalist +90%.
   - ModdedBase = 100 × 2.65 × 1.3 = 344.5
   - Tick = 0.35 × 344.5 × 1.3 × 1.9 = **297.82**; 6 ticks = 1786.9, ignoring armor.
   - On an orange crit (tier 2) with CD 3.0: ×(1 + 2 × 2) = ×5 → 1489.1 per tick.
2. **Heat** (Heat page): same mods plus Hellfire +90%: 0.5 × 344.5 × 1.9 × 1.3 × 1.9 = **808.37** per tick.
3. **Toxin** (Toxin page): base 100 Puncture, Serration, Infected Clip +90%, Rifle Elementalist, Bane of Infested: 0.5 × 344.5 × (1.9 × 1.9 × 1.3 = 4.693) = **808.37**.
4. **Gas** (Gas page, Vesper 77): base 180, Hornet Strike +220%, Leaded Gas +300%, Expel Grineer, Pistol Elementalist. ModdedBase = 180 × 3.2 × 1.3 = 748.8; tick = 0.5 × 748.8 × 4 × 1.3 × 1.9 = **3699.07**. With Scorch + Pathogen Rounds (Heat + Toxin mods) instead of Leaded Gas, the factor ×4 disappears.

### 3.3 Tick timing
(Status_Effect §Status Damage; Damage_over_Time tables)

| Type | Ticks at (s after proc) | Ticks (base) |
|---|---|---|
| Slash, Heat, Toxin | 1, 2, 3, 4, 5, 6 | 6 |
| Electricity, Gas | 0, 1, 2, 3, 4, 5 (the tick at 6 s "deals no damage") | 6 |
| Blast | one hit at 1.5 s (fuse) | 1 |

### 3.4 Exact-arithmetic details (recent, single contributor)
Damage/Calculation §DoT, added 2026-07-29 to 2026-08-31 by one editor (Wxihuann1225) and not yet confirmed by anyone else. **UNVERIFIED but specific.**
- DoT ticks are **not** quantized to 1/32. They are computed from the modded base damage, not from the quantized per-type values. Intermediate arithmetic is binary32 (f32).
- For weapon-generated Heat, Electricity, Toxin, Gas and Slash, each tick group's accumulator starts at **1**: `Tick = (Σ seeds + 1) × C × M`.
  - Examples: Heat with a 40 seed and +60% Heat → (40 + 1) × 0.5 × 1.6 = **32.8**. Slash with a 40 seed and Primed Bane +55% → (40 × 1.55 + 1) × 0.35 × 1.55 = 34.18.
  - Heat consolidates all its stacks into one tick, so +1 is added once. Electricity and Gas contributions landing on the same pulse are also consolidated, with +1 added once. Slash and Toxin stacks each add their own +1.
- The displayed number is `floor(raw + 0.5)`. This is display only.
- Blast has no +1 seed. Each stack records the damage H of the hit that applied it. Expiry deals `max(1, 0.3 × H)`. Detonation deals `3 × Σ H` as one radial instance.
- Effect on DPS: +0.5 (or +0.35 for Slash) per tick before multipliers. This matters only for low-damage procs.

---

## 4. Per-type detail

### 4.1 Impact: Knockback / Stagger (`PT_KNOCKBACK`)
- Stacks: max 5, each lasting 6 s; a 6th replaces the oldest. Stagger escalates with stacks. Ospreys, bosses and Tenno are immune to the stagger (Impact page §General; U33.6 cut max stacks from 10 to 5 and raised the mercy threshold from 4% to 8% per stack).
- Heavy units (humanoid Eximus, Heavy Gunner, Bombard, Scrambus, Comba, Nullifier, Ancient Disruptor, Ancient Healer, Toxic Ancient): Parazon mercy threshold +8% per stack, up to 80% (100% on Corpus and Eximus with shields removed).
- 5 stacks cause a Big Stagger (`PT_BIG_STAGGER`) (Status_Effect §Independent).
- No damage effect. It matters for DPS only as a Condition Overload status, and as the trigger for Hemorrhage and Internal Bleeding (Slash procs from Impact procs).
- Cannot coexist with Ragdoll, and a ragdolled target gets **no** status from an Impact roll (§2.2).
- Duration conflict: Status_Effect §General says "staggers movement for 1 second" (the flinch), while the Impact page and Damage page say each stack lasts 6 s. The Status_Effect duration table lists Stagger as 6 s. This file uses 6 s per stack.

### 4.2 Puncture: Weakened (`PT_FRAILTY`)
- Duration 10 s, max 5 stacks; new stacks replace the oldest even if the oldest has more time left (Puncture page; Status_Effect).
- Enemy damage dealt: −40% at 1 stack, then +10% per stack → 40 / 50 / 60 / 70 / 80%.
- **Crit chance received: +5% per stack, max +25%.** This is a **flat (absolute)** bonus added after relative crit-chance mods: `CC = BaseCC × (1 + ΣRelative) + ΣAbsolute` (Critical_Hit §Absolute Increases lists Weakened). It does **not** apply to AoE damage or to abilities.
  - Example: base CC 30% with Point Strike +150% → 75%. Against 5 Puncture stacks → 75% + 25% = **100%**.
- VIP enemies with OG: intended cap is **3** stacks (U36: "Fixed being able to apply 5 Puncture Stacks to VIP enemies with Overguard, instead of the intended 3"). Evensong can apply 5.
- History: U27.2 set 30% + 5%/stack with a 6 s duration. U33.6 changed it to 40% + 10%/stack, 5 maximum stacks, added the crit chance, and set the duration (now 10 s).
- Conflict: the Damage page §Physical Damage table still says 6 s. The Puncture page, the Status_Effect general table and the Status Duration table all say 10 s, so this file uses **10 s**.

### 4.3 Slash: Bleed (`PT_BLEEDING`)
- Formula: §3.1 with C = 0.35. Damage type **Cinematic** (`DT_CINEMATIC`): ignores armor, has no faction damage-type modifier (so no ×1.5 against Infested), and damages shields, health and OG (Cinematic page; Slash page). Since U27.2 Bleed no longer bypasses shields; it damages them (Slash page; Damage U27.2 notes).
- Independent stacks, unlimited, 6 ticks starting 1 s after the proc.
- Not affected by Slash or physical mods, nor by elemental mods. Affected by base damage, faction (twice), status damage, crit tier, headshot and Viral (health).
- Because armor does not reduce it, Bleed ticks deal the same damage at every enemy level. Shields and Viral still apply.
- Enemy-inflicted Bleed on players: 10% instead of 35%.
- Heavy attacks of Claws, Dual Daggers, Nikanas, Two-Handed Nikanas, Rapiers, Scythes, Tonfas, Warfans and Whips force Bleed procs (Slash page).

### 4.4 Heat: Ignite (`PT_IMMOLATION`)
- **DoT:** §3.1 with C = 0.5 and ElementBonus = Heat mods; Heat damage type, so armor and faction type modifiers apply (×1.5 against Infested, ×0.5 against Kuva Grineer).
- **Stacking (unique):** all Heat procs on a target merge into **one** tick per second. A new proc adds its per-second damage to the pool and **refreshes the duration of all stacks**, so Heat stacks indefinitely while it keeps being re-applied within 6 s. When a new stack arrives, the duration resets to the **longest** status duration among the existing stacks, so a high-SD source extends everyone's future Heat (Heat page §Stack Combining). The DoT table's "Refreshable? DoT instance can be refreshed / No limit" agrees. Introduced in U26.
- **Heat inherit** (Heat page §Heat Inherit; community term; rewritten 2026-11-14). When Heat is added to a target that already burns, the new proc's **Heat elemental bonus and faction bonus are taken from the source of the FIRST Heat stack**, overwriting the new source's own bonuses, until the Heat status ends.
  - An inherited faction bonus applies only **once** (no double dip). If the first proc had no faction bonus, later procs use their own faction bonus once. If both have one, each applies once.
  - Base-damage and status-damage bonuses are **not** inherited; each proc uses its own source's.
  - Example (wiki): Panthera (base 100, Hellfire +90%, Primed Bane of Corpus +55%) procs first: 50 × 1.9 × 1.55² = 228/s. An unmodded Silva & Aegis Prime (base 318) then procs: 159 × 1.9 × 1.55 = 468/s, for a total of 696/s. In the reverse order the total is 159 + 50 × 1.55 = 237/s.
  - Engine implication: Heat DoT depends on which source applied Heat first (abilities, companions, Manifold Bond). For a single-weapon calculation, inherit has no effect except the faction rule below.
  - **UNVERIFIED:** whether a single weapon's own later Heat procs double dip faction (the rule says an inherited faction applies once). The page's wording ("subsequent procs that inherit it") suggests a single weapon's second and later Heat procs get faction **once**, not twice. That would make Heat's faction scaling differ from the other DoTs after the first proc. Needs in-game testing; see Open questions.
- **Armor strip:** every 0.5 s after the proc, *current* armor is multiplied by 0.5^(1/4) ≈ 0.8409. Total strip goes 0 → 15.9% → 29.3% → 40.5% → 50.0% (full at 2 s). Re-proccing does not speed this up. After the Heat status ends, armor comes back in the same steps every 1.5 s over 6 s. SD scales the interval: interval = 0.5 s × (1 + SD).
  - Example: 500 armor → 420.4 (0.5 s), 353.6 (1 s), 297.3 (1.5 s), 250 (2 s).
- **Heat with Corrosive and Corrosive Projection** (Heat page; Corrosive page):
  - `Armor = Base × (1 − 0.5·h) × (1 − (0.20 + 0.06·n_corr)) × (1 − 0.18·n_CP)`, where h is Heat's current strip fraction (1 when fully ramped) and n_corr ≥ 1.
  - Example: 1000 armor, full Heat + 10 Corrosive stacks → 1000 × 0.5 × 0.2 = **100**.
  - The 18% per Corrosive Projection is taken from these pages; it is not checked against the mod's own page.
- **Crowd control:** panic about 4 s for humanoids, 3 s for most Infested, 2 s for MOAs and Chargers. Ospreys, bosses, Tenno and OG units are immune. Panic triggers when OG breaks while Heat is active.

### 4.5 Cold: Freeze (`PT_CHILLED`)
- Duration 6 s, independent stacks, max 10.
- Slow: 50% at 1 stack, +5% per stack to 90% at 9 (Cold page; U33.6, U36).
- **Critical damage received:** `ColdCD(n) = 0.10 + 0.05·(n − 1)` for n = 1..9 (+0.50 at 9). This is an **absolute** bonus added after relative CD mods and **before** the crit-tier calculation: `CD = BaseCD × (1 + ΣRel) + ΣAbs` (Critical_Hit §Absolute Increases).
  - Not applied to AoE damage.
  - Example 1 (wiki): Kunai (1.6 CD) + Primed Target Cracker (+110%) against 9 stacks → 1.6 × 2.1 + 0.5 = **3.86×**; an orange crit gives 1 + 2 × 2.86 = 6.72×.
  - Example 2 (wiki): Paris Prime with 112.5% CC, CD 2.0 + 0.5 → tier 1 = 2.5× (87.5% of hits), tier 2 = 4.0× (12.5%).
- **10th stack → Frozen** for 3 s (scaled by SD): no actions, no shield recharge, Cold CD bonus becomes **+1.0**. Afterwards 3 Cold stacks remain. A Frozen enemy cannot receive more Cold stacks, and Cold procs do not refresh it (Primary Frostbite cannot extend it).
  - The in-game text and U36 note say "receive double Critical Damage"; the wiki table interprets this as +1.0 (double the +0.5). This file uses +1.0 absolute.
- Bosses and enemies with active OG: max **4** stacks (+0.25 CD), so they can never be Frozen.
- Players (U42): 10% slow per stack, max 4 stacks, never frozen.
- History: U27.2: 25% + 5%; U32.3: 50% base, max 8 stacks; U33.6: 9 stacks, 5% per stack, crit damage bonus added; U36: 10 stacks and Freeze.
- Conflict: the Damage page's Cold row says "On Player: 25%", which is outdated by the U42 change.

### 4.6 Electricity: Tesla Chain (`PT_ELECTROCUTION`)
- DoT: §3.1 with C = 0.5 and ElementBonus = Electricity mods; Electricity type.
- Ticks at 0 to 5 s (6 ticks). Each tick hits the target and **all enemies within 3 m** (Electricity page). Independent stacks, unlimited.
- The chain has its own **1×** headshot multiplier; it can be raised by Target Acquired, Deadhead or sniper zoom. The proc also carries the inducing hit's headshot, crit and stealth multipliers (Electricity page). Enemy_Body_Parts lists "Electricity and Gas status procs" among attacks whose headshot behaviour is special.
- Stun of the primary target about 3 s; not extended by SD. Ospreys, bosses and Tenno are immune.
- Example (wiki): 344.5 ModdedBase, Stormbringer +90%, Bane +30%, Elementalist +90% → 808.37 per tick, applied to every enemy in the 3 m radius.
- Contributions landing on the same pulse merge into one damage number (Damage/Calculation §DoT).
- Magnetic's shield-break Electricity proc is a special forced proc with its own formula (§4.11).

### 4.7 Toxin: Poison (`PT_POISONED`)
- DoT: §3.1 with C = 0.5 and ElementBonus = Toxin mods; Toxin type. Independent stacks, unlimited, ticks at 1 to 6 s.
- **Bypasses shields** (both the direct Toxin damage and the proc ticks hit health) but **not OG** (Toxin page). Exceptions where shields are not bypassed: Treasurer and Hounds (Damage/Calculation, armor section).
- Toxin status damage is also raised by the Emerald Archon Shard: +30% (Tauforged +45%), additive with Elementalist mods.
- Toxin's armor bonus was removed in U27.2.

### 4.8 Blast: Detonate (`PT_FLASHBANG`)
Rework U36. Sources: Blast page, revised 2026-11-03, and Damage/Calculation §Blast.
- Each stack has a **1.5 s fuse** (scaled by SD). When the fuse runs out it deals **0.3 × ModdedBase × (1+F) × (1+StatusDmg) × crit × bodypart** Blast damage to the target.
- **10 stacks (4 on bosses), or the target dying with any stacks:** all active stacks detonate at once.
  - The target takes each stack's 0.3× (i.e. 10 × 0.3× at the cap).
  - Every **other** enemy within **5 m**, with no falloff, takes **3.0 × the same per-stack damage for each stack**, as a single instance. The radius is not affected by Firestorm or Fulmination.
  - A killing blow that applies at least one Blast proc triggers the full AoE even if the target had no stacks before.
- **No elemental scaling:** neither Heat/Cold mods nor Blast-element mods (Valence Formation, Thermal Transfer) raise Blast proc damage. Contested on the wiki:
  - Damage_over_Time §DoT Damage Scaling says literal Blast mods (Valence Formation) *do* buff it.
  - The Blast page's editor reverted that on 2026-10-27: "Blast status damage does not scale with blast elemental bonuses and never has".
  - This file follows the Blast page. **Contradiction recorded.**
- Faction applies **twice**; crit, headshot and headcrit multipliers apply to both the single-target and AoE parts.
- Status damage bonuses apply (Status_Effect §Status Damage lists Blast).
- Still deals damage with SD below −100% (Blast page §Notes).
- Example (wiki, Xata's Whisper section): base 100, Primed Bane of Grineer, no crit. Single stack = 0.3 × 100 × 1.55² = **72.075**. At 10 stacks each neighbour takes 3 × 240.25 × 10 = **7207.5**.
- Xata's Whisper triggers off each detonation, with faction applied a third time and elemental bonuses applied (Blast page). That is an ability interaction.

### 4.9 Corrosive: Corrosion (`PT_CAUSTIC_BURN`)
- Duration 8 s, independent stacks, max 10; new stacks replace the oldest.
- Armor reduction `= 0.20 + 0.06·n` (n ≥ 1): 26% at 1 stack, 80% at 10 (Corrosive page; U27.2).
- Emerald Archon Shard: max stacks +2 (Tauforged +3) → 92% / 98%. The Corrosive page says **14 stacks** remove all armor (0.20 + 0.84 ≥ 1, clamped), which implies shard bonuses from multiple shards stack. That is not stated on the Archon_Shard page. **UNVERIFIED.**
- The extra stacks only come from Corrosion applied by the shard holder's own weapons and abilities (Archon_Shard).
- Hydroid's passive: the first Corrosive proc strips 50%, allowing 100% at 10 stacks.
- Multiplicative with Heat and Corrosive Projection (§4.4).
- Effect against capped armor (the armor DR formula is on the Armor page and outside this file's scope): 1 stack ≈ ×2.3 effective damage, 10 stacks ≈ ×6.

### 4.10 Gas: Gas Cloud (`PT_ASPHYXIATION`)
- DoT: §3.1 with C = 0.5 and ElementBonus = **Gas** mods only (Leaded Gas, Valence Formation set to Gas). Heat and Toxin mods do nothing for it.
- Each tick hits every enemy in the radius, including the host.
- Stacks: max 10, independent; the 11th replaces the oldest. Ticks at 0 to 5 s.
- Radius: 3 m, +0.3 m per extra stack, "6 m at 10 stacks". As literally stated, 3 + 0.3 × 9 = 5.7 m, which conflicts with 6 m. **UNVERIFIED** exact formula.
- The cloud has its own 1× headshot multiplier, raised by headshot-bonus mods. It also carries the inducing hit's crit and headshot multipliers.
- If the host dies, the cloud keeps ticking on enemies inside the radius for its remaining duration (U27.3: "for the average of the remaining Effect duration").
- Example: §3.2 example 4.
- Damage/Calculation says Gas stacks are **consolidated into one tick** (e.g. nine seeds summed with +1 once). This conflicts with Damage_over_Time's "each instance having its own timer". Both can be true (own timers, merged display/tick). Engine: sum the active stacks' tick damage each second.

### 4.11 Magnetic: Disrupt (`PT_MAGNETIZED`)
- Duration 6 s, independent stacks, max 10; the 11th replaces the oldest.
- Damage to **shields and OG** ×`(2 + 0.25·(n − 1))`: 2.0× at 1 stack, 4.25× at 10 (Magnetic page; Overguard page). This is a Damage-Vulnerability-type multiplier, multiplicative with other bonuses. It also stops natural shield regeneration.
  - Example: 4 stacks → ×2.75 to shields.
- OG multiplier added in U36.
- **Shield/OG-break forced Electricity proc** (U36):
  - Total damage = **0.03 × n × target max shields (or max OG)**, n ≤ 10 (30% at 10 stacks), dealt over 6 s.
  - Base-damage mods have **no** effect. Status-damage mods apply **twice** (0.9 → ×3.61). Faction applies **twice**. Electricity mods apply even if combined into another element. SD multiplies the total.
  - Example: 1000 max shields, 10 stacks, Pistol Elementalist +90%, no faction → 300 × 3.61 = **1083** total Electricity damage over 6 s, before Electricity mods.
- Against players: energy drain and HUD static (out of scope).

### 4.12 Radiation: Confusion (`PT_RAD_TOX`)
- Duration 12 s, independent stacks, max 10; the 11th replaces the oldest.
- The confused enemy attacks its allies and is attacked by them, dealing +100% (+50% per extra stack, max +550%) damage to its allies. Auras from confused units stop working.
- No direct effect on the player's weapon DPS. It counts for Condition Overload.
- Bosses, Liches, Acolytes, Necramechs, Sisters and Hounds: max 4 stacks; +100% per stack damage *taken* from turned allies (max +250%); their allies are not turned against them.
- OG enemies receive the status but ignore it until OG is gone (Radiation page; Overguard page).
- Manics and Manic Bombards are immune.

### 4.13 Viral: Virus (`PT_INFECTED`)
- Duration 6 s, independent stacks, max 10; the 11th replaces the oldest.
- Damage to **health** ×`(2 + 0.25·(n − 1))`: 2.0× at 1 stack, 4.25× (+325%) at 10.
  - Applies to armored health too (armor still reduces the damage). Does **not** apply to shields or OG. Viral amplifying OG was a bug fixed in U31.6.
  - Multiplicative with other bonuses.
- Conflict: the U27.2 patch note says "can scale up to 4.5x"; the formula and in-game text give +325% = 4.25×. This file uses **4.25×**.
- **Timing rules** (Viral page; derived from Simulacrum testing, per the page's comments):
  - A hit deals its damage **before** applying its own Viral proc, so the proc does not boost that hit.
  - Pellets of the **same** multishot shot do not benefit from each other's Viral procs.
  - Shots within an Instant Burst (Quartakk) **do** benefit from earlier shots' Viral procs.
  - Electricity and Gas procs created in the same hit as a Viral proc **do** get the Viral bonus on their immediate first tick.
  - DoT ticks use the Viral stacks present **when each tick lands** (no snapshot, no double dip).
  - Example: a Bleed of 35/tick deals 70/tick while 1 Viral stack is active and 35/tick after it expires.
- Immune to Viral status: Ambulas, Deimos Carnis/Rex, Genetrix, Jugulus, Leaping Thrasher (Deimos), Therid, Saxum/Rex, Demolisher Boiler/Thrasher, Tusk Bolkor, Tusk Firbolg, Techrot Babau. Fully immune to Viral damage: Demolisher Charger and Demolisher Juggernaut.

### 4.14 Void: Bullet Attractor (`PT_RADIANT`)
- A field of 2.5 m radius for 3 s, centred on the body part hit (U29.5). Hitscan and projectiles entering it are redirected to that point. Punch-through projectiles orbit instead.
- Stack and SD behaviour: not documented. **UNVERIFIED.**
- Void damage: ×1.5 against OG (Overguard page: "50% vulnerability") and against Zariman; neutral against everything else since U29.1. Strips Sentient adaptation.
- Not physical and not elemental, so no physical or elemental bonuses apply. No DPS effect from the proc itself.

### 4.15 Tau: Status Vulnerability (`PT_SENTIENT`)
- Duration 8 s, independent stacks, max 10; the 11th replaces the oldest (Tau page; U37).
- Effect: `SC_effective = SC_source × (1 + 0.10·n)`, multiplicative, so ×2.0 at 10 stacks.
  - Example (wiki): 210% SC against 5 stacks → **315%**; against 10 stacks → **420%**.
- Forced procs gain nothing from it.
- Sources: Caliban abilities, Haalvu, Sentients, Corpus Amalgams, Vomval Trumpet pickup. Dante's Pageflight applies the same "Status Chance Vulnerability" effect.
- Tau damage is neutral against all factions.

### 4.16 True, Cinematic, Finisher and other non-proc types
- **True** (`DT_HEALTH_DRAIN`): damages health and OG; bypasses shields and armor DR (but not non-armor DR); no faction modifier; not raised by physical or elemental bonuses; no proc. The only weapon source is the Basmu pulse; otherwise abilities.
- **Cinematic** (`DT_CINEMATIC`): what Slash Bleed deals. See §4.3.
- **Finisher** (`DT_FINISHER`): finishers and some abilities (e.g. Ash). Damages health, shields and OG; bypasses armor DR; no proc (Damage §Unique Damage, which the page tags as `{{Speculation}}`).
- **Microwave** (`PT_MICROWAVE_BURN`): Nukor and Kuva Nukor only; infinite duration; enlarges the hit body part; no damage effect; counts as one CO status (Microwave page; Status_Effect §Independent).
- **Knockdown, Lifted, Stun:** non-damage procs, usually forced by stances, heavy attacks or abilities. Each counts as an individual status for Condition Overload; Knockdown and Lifted cannot coexist (Condition_Overload_(Mechanic) §What Counts; Status_Effect §Independent).
- **Shield Drain, Energy Drain:** ability or enemy only. Out of scope.

---

## 5. Shields, Overguard and status immunity

- **Shields:** absorb every damage type except Toxin, so Toxin proc ticks go straight to health. Slash Bleed, Heat, Electricity, Gas and Blast damage shields.
  - Magnetic amplifies all damage to shields.
  - Enemy shield gate: on full shield depletion, 0.1 s during which only 5% of damage reaches health. Weak-spot hits bypass it entirely. "Additional damage instance associated with the attack such as status effects ... can still damage the enemy" (Shield §Shield Gating). Whether DoT ticks landing inside the gate are cut to 5% is unclear. **UNVERIFIED.**
- **Overguard (enemies):**
  - Enemies **can receive status effects** while OG is up. They ignore the crowd-control part (Stagger, Knockdown, Stun, Confusion/Radiation, Slow, Ragdoll, Blind, Lifted, Heat panic).
  - Cold is capped at 4 stacks under OG. Puncture is intended to cap at 3 for VIPs with OG (U36 fix). Evensong's page speaks of a general "4 stacks" cap for OG enemies. These caps conflict; see Open questions.
  - DoT ticks damage OG: Toxin does not bypass OG, and Cinematic and True damage OG.
  - Viral does **not** amplify OG damage. Magnetic amplifies OG damage (×2 to ×4.25).
  - Armor and shields can still be stripped while OG is up (Corrosive and Heat work).
  - OG is not reduced by armor or by faction damage-type modifiers; only Void (×1.5) and Damage Vulnerability apply (Overguard §Mechanics).
  - Enemies have no invulnerability gate when OG breaks.
- **Historical note:** the U31.6 dev quote on the Viral page ("the only Status Effect that would work on Eximus while Overguard is present is Cold") is outdated by the current Overguard page.
- **Overguard (players):** any amount gives **full status immunity**, plus a 0.5 s invulnerability gate when it is depleted (Overguard §Player; Status_Effect intro).
- **Damage vs status immunities:** a status immunity removes that type from the proc weights (§2.2) whether or not the enemy is also immune to the damage.

---

## 6. Interaction with Condition Overload-type bonuses
(Brief; the full mechanic belongs to another research file.)
- CO counts **distinct status types** on the target, not stacks. Counted: Impact, Puncture, Slash, Cold, Electricity, Heat, Toxin, Blast, Corrosive, Gas, Magnetic, Radiation, Viral, Void, Tau, Lifted, Knockdown and Microwave (Condition_Overload_(Mechanic) §What Counts As A Status Effect). Frozen is a Cold state, not a separate type; Secondary Shiver counts Cold *stacks* (+45% per stack) instead.
- CO stacks additively with Serration-type bonuses on "intended" (mostly hitscan) attacks but multiplicatively on many projectile attacks. Several other weapon-specific behaviours exist (same page, §Application Behaviors).
- Blast, Electricity and Gas procs take the CO bonus of the initial target and carry that damage into their radius (same page, §Mechanics). This is one route by which CO reaches DoT damage: it raises the hit-derived proc seed. **UNVERIFIED** for Slash, Heat and Toxin. The page shows CO does not affect embedded DoTs of Tysis and Stinger; whether it enters ModdedBase for ordinary procs is not stated.

---

## 7. Edge cases (checklist for the engine)

1. Faction damage double-dips into every DoT and Blast proc (`(1+F)^2`), but **not** into Heat procs that inherit a faction bonus (once), and Magnetic's break proc takes it twice.
2. Status-damage bonuses apply **twice** to Magnetic's shield-break Electricity proc (×3.61 for +90%) and once to every other proc.
3. Slash ticks are Cinematic: no faction type modifier (no ×1.5 against Infested or Narmer), no armor, but they do damage shields.
4. Toxin ticks skip shields (except Treasurer and Hounds) but not OG.
5. Heat merges all stacks into one tick, refreshes all stacks on each proc, adopts the longest SD among stacks, and inherits the first source's Heat elemental and faction bonus.
6. Heat armor strip ramps over 2 s × (1 + SD) and is multiplicative with Corrosive and Corrosive Projection.
7. Gas and Blast ignore their component-element mods. Gas scales only with Gas mods; Blast scales with no elemental mod at all (wiki pages disagree; §4.8). A forced primary-element proc (Toxic Lash) **is** scaled by that element's mods even when they were combined on the weapon.
8. Electricity and Gas tick at t = 0, and their first tick can benefit from a Viral proc applied by the same hit. Slash, Heat and Toxin start at t = 1 s.
9. A Viral proc does not boost its own hit or sibling pellets of the same shot, but does boost later shots of an Instant Burst.
10. DoT damage uses the crit **tier rolled** by the inducing hit (yellow, orange or red), not the average.
11. Cold's CD bonus is absolute (after relative mods, before tier math). It does not apply to AoE. It becomes +1.0 when Frozen and caps at +0.25 (4 stacks) on bosses and OG units.
12. Puncture's crit chance bonus is flat, after mods; it does not apply to AoE or abilities. VIPs with OG cap at 3 stacks.
13. Magnetic multiplies damage to OG as well as shields; Viral does neither (health only, including armored health).
14. A ragdolled target turns an Impact roll into no proc at all (the weights are not renormalized). A status-immune target renormalizes the weights.
15. Tau multiplies status chance (×(1+0.1n)); it does not add to it, and it does not affect forced procs.
16. Blast at the cap or on kill: the host takes only 0.3× per stack; neighbours take 3× per stack with no falloff. The boss cap of 4 triggers detonation at 4.
17. SD below −100% removes DoTs and debuffs, but Blast still deals its hit.
18. SD adds DoT ticks, rounded down to whole seconds (Lasting Sting rank 9 and rank 10 are equal).
19. Weak spots from Sonar or Detect Vulnerability boost only the initial hit, never the DoT. Body-part headshot multipliers do carry into the DoT. Electricity and Gas AoE ticks have their own 1× headshot multiplier.
20. Continuous weapons: merged multishot beams sum SC on one tick (SC can exceed 100% even if the per-beam SC is lower).
21. Enemy-inflicted Slash is 10%, not 35% (irrelevant for our calculator, but don't reuse the constant).
22. Per-tick "+1 accumulator" and binary32 arithmetic (§3.4): matters for exact matching with in-game numbers on low-damage procs.
23. Display only shows up to 10 Slash and Toxin numbers; damage from more stacks is still dealt.
24. Combined elements remove their components' procs (e.g. Heat + Cold → Blast; no Ignite or Freeze from those mods). Innate secondary elements (Ogris Blast, Stug Corrosive, Nukor Radiation) never combine with mod elements.
25. Kuva and Tenet weapons can carry two primary elements (weapon + progenitor); the HCET order rule decides how they combine (Damage §Modding). This changes which procs exist.

---

## 8. Open questions (UNVERIFIED or contradictory)

1. **Exact proc count for SC > 100%:** is it `floor + Bernoulli(frac)`? The wiki only gives the expected value.
2. **Are proc-type weights computed on the 1/32-quantized per-type damage or the raw values?** This changes weights slightly for small types (e.g. Nagantaka Prime's Impact quantizes to 0).
3. **DoT tick count with fractional SD:** `floor(6(1+SD))` (Lasting Sting note), the DoT-page formula `floor(Duration − Delay) + 1`, or the Calculation page's continuous `6 × (1+SD)`? Also, does the same rounding apply to debuff durations (Corrosive 8 s, Viral 6 s)?
4. **Boss and Lich cap of 4 stacks:** does it limit the unlimited DoTs (Slash, Toxin, Electricity, Heat) and Viral, Corrosive and Magnetic? The U27.3 text says "No Status Effect will exceed 4" (Liches); the current per-type pages only mention Cold, Radiation and Blast.
5. **OG stack caps:** the Overguard page says only Cold is capped (4). The Puncture U36 fix says VIP OG enemies are meant to have 3 Puncture stacks. The Evensong page says OG enemies are "usually capped at 4 stacks of a status". Which statuses are capped, and at what count, on Eximus OG?
6. **Blast and Blast-element mods:** Damage_over_Time says literal Blast/Valence mods buff Blast procs; the Blast page (edited 2026-10-27) says they never have. Needs a test.
7. **Blast's H value:** the Calculation page says "damage H of the hit that applied it" (whole hit?), while the Blast page formula uses ModdedBase × faction² × status × crit × body part. These differ if H includes elemental damage.
8. **Heat inherit with a single weapon:** do a weapon's second and later Heat procs on the same target lose faction double-dipping (faction once)? Literal reading of the §Heat Inherit notes says yes.
9. **Gas radius formula:** 3 + 0.3·(n−1) gives 5.7 m at 10; 6 m as stated needs 0.333 per stack or 3 + 0.3·n.
10. **Cold CD bonus and Puncture CC bonus on DoT ticks:** do they affect the crit multiplier and crit roll captured by the proc? Are the Electricity and Gas AoE ticks on other enemies excluded ("does not apply to AoE")?
11. **"+1 accumulator" and binary32 rules** (Damage/Calculation §DoT): single editor (mid-2026), unconfirmed by others or by patch notes.
12. **Impact stack duration:** 6 s per stack (Impact page, Damage page, Status Duration table) vs "1 second" (Status_Effect general table, likely the flinch only).
13. **Void proc stacking and SD scaling:** undocumented.
14. **Emerald shard Corrosion cap with multiple shards:** "14 stacks" implies multiple shards stack; the Archon_Shard page gives only the per-shard +2/+3.
15. **Status Effect "Negative Status Duration" table** is marked outdated on the wiki (2016 forum source).
16. **Enemy shield gate and DoT:** are ticks landing during the 0.1 s enemy shield gate reduced to 5%?
17. **Does CO raise ModdedBase for ordinary Slash, Heat and Toxin procs?** Only Blast, Electricity and Gas carry-through is documented.

---

## 9. Out of scope, noted
- Empyrean/Railjack status effects (Concuss, Decompress, Tear, Immobilize, Scramble, Sear, Intoxicate) differ entirely (Damage §Empyrean).
- Effects of statuses on players (Magnetic energy drain, Radiation friendly fire, Cold U42 rework).

Source for the one external search performed (WebSearch, 2026-11-23): it found no official source for a global status stack cap. The only extra lead was a 2024 player comment on a Steam forum thread claiming bosses are capped at 4 statuses: https://steamcommunity.com/app/230410/discussions/0/4514379280326472301 (unverified player claim).
