Status: reviewed (architecture)

# Optimizer (`wfd-opt`)

The optimizer searches the build space of one weapon under a fixed scenario (enemy, toggles, buffs, conditions) for
the builds with the best objective score. It runs locally on all cores, may take a while, reports progress, can be
cancelled at any time, and returns the best builds found with an honest statement of how good the search was
(proven optimum, bound gap, or heuristic). It does no I/O; the server runs it as a job (server-api.md §8).

Its scoring is the engine's EV kernel (engine.md §7.9) with the metrics, TTK ordering and tie-breaks of spec 08 §10
(E-8.26); finalists are confirmed with Monte Carlo. The inspiration's
optimizer (`research/findings/inspiration-optimizer.md`, "WFOpt") is the parity reference; section 10 shows how each
of its weaknesses is addressed.

Contents: 1 API · 2 Problem compilation · 3 Decomposition · 4 Monotonicity and bounds · 5 Main-slot search ·
6 Context search · 7 Scoring and finalists · 8 Parallelism and determinism · 9 Progress and cancellation ·
10 Comparison with the inspiration · 11 Defaults

---

## 1. API

```rust
// crates/wfd-opt/src/lib.rs
pub fn optimize(engine: &Engine, data: &GameData, req: &OptRequest, ctl: &JobControl, sink: &dyn EventSink)
    -> Result<OptResult, OptError>;
pub fn validate(engine: &Engine, data: &GameData, req: &OptRequest) -> Result<(), OptError>;  // 422 before a job (API-19)

pub struct JobControl { pub cancel: CancelToken /* engine.md §1, shared with Engine::mc (T02d-4) */,
                        pub deadline: Option<Instant>, pub node_budget: Option<u64> }
// node_budget: deterministic per-work-item budgets (section 8); deadline: wall clock, not reproducible
pub trait EventSink: Send + Sync { fn emit(&self, e: OptEvent); }
pub enum OptError { Invalid(Vec<Message>), Objective(Message), Infeasible(Message), Engine(Message),
                    Cancelled /* before the first scored build: no anytime result (API-19) */ }
```

`validate` resolves the request and checks the objective without searching; the server calls it in
`POST /api/optimize` and answers 422 `api.invalid` for `OptError::Invalid` (server-api.md §8.1), and `optimize` runs
the same checks first. `optimize` scores the input build right after compiling the scenario (the first incumbent,
§5.2 "Ordering"), so every later cancel has an anytime result; a cancel before that is `OptError::Cancelled`, which
the caller ends with `Cancelled { result: None }` (§9).

**Keys and scenario of the results** (contract review API-17, M-10, API-9, API-7). `input.x` maps every session-local
key of the whole request, the lists outside `input` included (`constraints.must_include`, `constraints.prohibited`,
`buffs.allowed`, `groups.shards`), and the optimizer resolves them through it (AD-18). Every `RankedBuild` carries the
scenario it was scored in: the input scenario with the search's own options applied (`objective.window_s` as T-01,
`objective.heavy` as T-09, `buffs.strength` as T-20, the buff set of its portfolio row, T-03 `body` under
`USE_WEAK_POINT` off, the inspiration-compatible preset's window and T-02 `all_max`), and nothing else changed; only
`toggles.capacity` stays the input's (`constraints.capacity` governs the search, server-api.md §8.1). The calculator
on `RankedBuild.input` (or its share code) therefore reproduces `score` (08 §10; the one exception is an effect
filter, `NO_FACTION`, `NO_ON_KILL`, `ALLOW_NEXT_SHOT` off, which has no scenario input). `RankedBuild.code` is
`wfd_buildcode::encode` of that input made by `GameData::to_doc`, filled by `optimize` itself (`wfd-opt` depends on
`wfd-buildcode`, overview.md §3.1).

Request and result types are API DTOs in `wfd-model::api::opt` (JSON shapes in server-api.md §7):

```rust
pub struct OptRequest {
    pub input: CalcInput,                // current build (start point, locks, riven, variant) + scenario
    pub objective: Objective,
    pub constraints: Constraints,
    pub groups: GroupSwitches,
    pub buffs: BuffPolicy,
    pub search: SearchParams,
}
pub struct Objective {
    pub metric: Metric,                  // DpsWindow (default) | DpsBurst | DpsSustained | Ttk | HitInFight | HitFresh
                                         // (08 §10; ordering by engine `Score` keys, ties by ammo_window / ammo_to_kill)
    pub view: ViewSel,                   // Auto (armor if armored, else health) | Health | Armor | Shield | Overguard | Layered
    pub attacks: Vec<(AttackKey, f64)>,  // weighted multi-attack objective; default: first attack, weight 1
    pub heavy: Option<Heavy>,            // T-09 for the search (start combo); disabled for explicit heavy attacks;
                                         // written into every RankedBuild.input
    pub window_s: Option<f64>,           // search and ranking window; default T-01 of the scenario; written into every
                                         // RankedBuild.input as T-01
}
pub struct Constraints {
    pub locked: Vec<SlotId>,             // per slot (the inspiration's "keep installed" = lock every occupied slot)
    pub must_include: Vec<StableKey>,    // local keys resolved through input.x (API-17)
    pub prohibited: Vec<StableKey>,      // the "prohibited mods" list; persisted by the frontend (local keys with their
                                         // uniqueNames, sent through input.x)
    pub elements: ElementRule,           // { require: Vec<DamageType>, forbid: Vec<DamageType> } (final types)
    pub rules: RuleFlags,                // USE_WEAK_POINT, NO_FACTION, NO_ON_KILL, KEEP_PROGENITOR, USE_INCARNON,
                                         // ALLOW_NEXT_SHOT, EXCLUDE_DRAFT, EXCLUDE_STALE
    pub capacity: CapacityRule,          // { mode: Ignore | Current | AnyForma (default) | Budget(n), umbra_forma, ranks: Max | AllowLower }
                                         // spec 09 E-9.15 (T-35) and OQ-915 (T-36)
    pub riven: RivenPolicy,              // UseGiven (default) | None | Search { traits: 2..=4, allow_negative, quality }
                                         // quality = u of 04 E-4.18 (0 … 1), applied to every trait (API-12)
}
pub struct GroupSwitches {               // which slot groups the search may change (others stay as in the input)
    pub exilus: bool, pub weapon_arcanes: bool, pub stance: bool, pub melee_plan: bool /* T-30 plan search */, pub incarnon: bool,
    pub frame_mods: u8 /* 0..=8 general frame slots, 09 E-9.2 */, pub frame_exilus: bool, pub aura: bool, pub frame_arcanes: bool,
    pub shards: ShardPolicy /* Off | Owned(Vec<(StableKey, tauforged, count)>) | Any */,
    pub modular_parts: bool, pub progenitor: bool,
}
pub struct BuffPolicy { pub allowed: Vec<BuffPick>, pub portfolio: bool,
                        pub strength: Option<f64> /* T-20, written into every RankedBuild.input */ }
pub struct SearchParams {
    pub mode: SearchMode,                // Exact (default) | Balanced | Quick
    pub certify: bool,                   // default false: leaves scored with WindowExact, proof without the ε assumption (§5.2)
    pub time_limit_s: Option<f64>,       // anytime; default 120
    pub node_budget: Option<u64>,        // deterministic budget for tests and shared results (section 8)
    pub keep: u16,                       // finalists, default 32
    pub mc_confirm: Option<McConfirm>,   // default { finalists: 12, runs: 1000, seed: 0 }
    pub threads: Option<u16>,            // default: optimizer pool size (cores − 1)
    pub context_beam: u8,                // default 4 (the inspiration's "beam width" maps here)
    pub preset: Preset,                  // Default | InspirationCompatible (§10.2)
}
pub struct OptResult {
    pub best: RankedBuild, pub alternatives: Vec<RankedBuild>,
    pub per_mix: Vec<RankedBuild>,       // best build per final damage-type mix
    pub proof: Proof,                    // ProvenOptimal { scope, assumes_fast_error: Option<f64> /* ε, None when certified */ }
                                         // | Gap { upper_bound: Score, relative } | Heuristic
    pub leave_one_out: Vec<SlotValue>,   // score without each slot's item
    pub next_best: Vec<SlotAlternative>, // best replacement per slot and its score
    pub condition_sensitivity: Option<f64>, // best build re-scored with T-02 = none
    pub buff_table: Option<BuffTable>,   // per allowed buff scenario: best build, score, value of the buff
    pub excluded: Vec<ExcludedItem>,     // items left out and why (unmodelled, prohibited, rule filter, irrelevant)
    pub stats: SearchStats,              // evaluations, bounds, prunes, leaves, per-phase time, threads, reproducible
    pub capacity: Option<CapacityReport>,// = best.capacity
    pub riven_sets: Vec<RivenSetRow>,    // riven search: best build per trait set (§6.2 step 6)
}
pub struct RankedBuild { pub input: CalcInput,          // the scenario the build was scored in (above)
                         pub code: Box<str>,             // wfd1 code of `input` (wfd-buildcode, AD-18)
                         pub score: f64,                 // metric value in its unit (DPS, damage, or T_EV seconds for TTK)
                         pub key: Score,                 // engine.md §7.9 ordered key (band, primary, tie, tie2) used for ranking
                         pub ttk: Option<TtkEv>,         // TTK metric: band, T_EV, κ, shots and ammo to kill
                         pub mc: Option<Summary>, pub ev_mc_deviation: bool, pub gap_classes: Vec<Box<str>>,
                         pub ev_rank: Option<u16>,       // EV rank when MC re-ranked the finalists (§7 step 3)
                         pub mix: Vec<DamageType>,
                         pub draft_items: Vec<StableKey> /* items with draft/stale effects in this build */,
                         pub capacity: Option<CapacityReport>, pub violations: Vec<Violation> /* §7 step 4 */ }
```

---

## 2. Problem compilation

1. **Scenario.** `engine.compile_scenario(input.build, input.scenario, Some(&objective_spec))` once. The objective's
   view `Auto` resolves to `armor` if the enemy has armor, else `health` (inspiration parity).
2. **Positions and pools** (`pool.rs`). Positions: 8 main slots (one shared pool, no duplicates), exilus, weapon
   arcane(s), stance × combo, Incarnon tiers, frame mods, aura, frame arcanes, shard multiset, kitgun/zaw parts,
   progenitor element, buff scenario. Locked slots become fixed. Eligibility per position comes from
   `wfd-data::legality` (the same code the UI and the decoder use): slot kind, categories and tags, forbidding tags,
   Semi-only trigger, incompatibility (union of both directions of the wiki lists plus the hidden-parent rule,
   D-093), rank and capacity feasibility, `prohibited`, `must_include`.
3. **Status filter.** Items with `unmodelled` or `no_damage_effect` status are excluded (I-6); `draft` and `stale`
   items are included unless the rules say otherwise and are marked in the result.
4. **Rule filters act on effects, not whole items.** `NO_FACTION` zeroes `faction` targets; `NO_ON_KILL` zeroes
   effects whose trigger is a kill (`on_kill`, `on_weak_point_kill`, `on_melee_kill`, `on_primary_kill`);
   `USE_WEAK_POINT` off sets T-03 to `body`, so weak-point-scoped effects compile to zero; `ALLOW_NEXT_SHOT` off zeroes
   `next_shot_after` effects for DPS and TTK metrics. An item whose effects are all zero after filtering drops out.
   Status-duration-only mods are not excluded (they matter to `dps_window`; `I:opt` Q9).
5. **L1 compile.** Each remaining candidate is compiled once (`engine.compile_pick`) into a `CompiledItem` per
   objective attack.
6. **Relevance.** A candidate whose contribution to every objective attack is zero, with no element, no mechanic
   and no static term, is dropped (recoil, zoom, punch-through): derived from the engine, not from a hand list.
7. **Dominance.** Inside a pool, candidate B is dropped if some candidate A has `contrib(A) ≥ contrib(B)` component-
   wise on every `Inc` slot, equal `Unknown` slots and elements and mechanics, the same set membership, no larger drain
   under a capacity rule, and an incompatibility mask that is a subset of B's (so A is available wherever B is).

---

## 3. Decomposition

A complete build is `(context, main)`:

- **Context**: everything outside the 8 main slots: exilus, weapon arcane(s), stance and combo, Incarnon perk per
  tier, frame mods, aura, frame arcanes, shard multiset, kitgun/zaw parts, progenitor element, buff scenario.
- **Main**: the 8 main slots, with the riven as one candidate.

Main candidates split by the stat slots they touch (per-scenario classes of `CompiledScenario.slot_class`):

- **Composition candidates** touch an `Unknown` slot: elements, physical mods, conversions, forced procs,
  `phoenix_talons`, mechanics that change proc weights, and every candidate that touches a slot the compiled scenario
  downgraded to `Unknown` (section 4: CC items when Prelude of Might, Devouring Attrition or Melee Duplicate is in
  the context; magazine items when Primed Chamber is). They can raise or lower the score.
- **Monotone candidates** touch only `Inc` slots (and `None` display slots): base damage, multishot, crit, status
  chance, fire rate, magazine, reload, faction, status damage and duration, multiplier groups, extra-hit fractions.
- **Coupled candidates**: set members (their bonus depends on the count; bounded with the maximum count still
  reachable), items with `Dec` slots (locks: Primary Acuity, Cannonade), and candidates that are themselves
  non-monotone *readers* (`CompiledScenario.readers`: Primed Chamber, Synth Charge, Overwhelming Attrition, …). They are
  branched include/exclude first; in each branch the slot classes are those the scenario compiler derives for that
  branch's set of readers, so the branch without readers keeps its `Inc` slots.

---

## 4. Monotonicity and bounds

The bound needs one property: for a fixed context, composition and set of mechanics, the scored metric is
non-decreasing in every `Inc` slot. Each slot's class is declared in the stat registry (domain-model.md §10) with a
justification that cites the formula (for example `fire_rate` is `Inc` because a faster schedule fires every shot no
later, so shots and DoT ticks landing in [0, t_win] cannot decrease, 06; `cc_rel` is `Inc` because
`E[c] = 1 + E[k]·(h·CDM − 1)` with `h·CDM ≥ 1`, E-4..E-6).

The declared class holds only while nothing reads the slot's quantity in a non-monotone way. The downgrades are
derived from the compiled scenario, not from a list of cases: `compile::monotone` (engine.md §3, domain-model.md §10)
collects every *reader* of the scenario and of the candidate pool (a trigger, gate, condition, mechanic parameter or
`per` scalar that reads a crit-tier outcome, a proc outcome, a schedule mark or a resolved stat) and downgrades the
slots feeding that quantity to `Unknown`, unless the reader is monotone (upward-closed predicate, non-negative gated
value). Examples: Devouring Attrition (×21 on tier-0 instances), Overwhelming Attrition (instances with no crit and no
proc), Melee Duplicate and every `OnTier1Crit` trigger (P(tier = 1) falls above 100 % CC), Secondary Enervate (reset
after 6 hits of tier ≥ 2) downgrade the CC slots (and the SC slots for Overwhelming Attrition); Primed Chamber,
Charged Chamber, Synth Charge and Plated Round downgrade `magazine`, `magazine_flat`, `ammo_eff`, `fire_rate`,
`reload_speed` and `charge_rate`; Prelude of Might's `Cond::Attack` on CC downgrades CC; a `distinct_below` predicate
downgrades the status-chance slots. Readers held by fixed context items downgrade for the whole search; readers held by
candidates are coupled candidates (section 3). The claim is enforced four ways (testing.md §6):

1. **Property tests**: random weapon, scenario, build and a random increase of one `Inc` slot: the score must not
   decrease at any fidelity. Failing seeds are kept as regression fixtures.
2. **Bound tests**: random partial build + random completion: `bound(relaxed) ≥ WindowExact(completion)`.
3. **Exactness tests**: on small pools (≤ 18 candidates) branch-and-bound returns the same argmax and top-10 set as
   brute force.
4. **Reader fixtures**: one fixture per reader kind and per named example above (Devouring Attrition, Overwhelming
   Attrition, Melee Duplicate, Secondary Enervate, Primed Chamber, Synth Charge, Plated Round, Prelude of Might,
   `distinct_below`): the fixture shows that the declared class would be violated (a larger value lowers the score)
   and asserts that `slot_class` downgrades the slot, so random property tests are not relied on to find them.

If a mechanic is found to break monotonicity, its slot class becomes `Unknown`: the search then enumerates instead of
bounding; correctness degrades to "slower", never to "wrong".

---

## 5. Main-slot search

### 5.1 Composition sets

The search enumerates **composition sets**: subsets of composition candidates of size 0..k (k = free main slots),
respecting incompatibilities, deduplicated by their resulting element outcome and physical/forced vector. For each
set, `ElementSolver::outcomes` (engine.md §3) expands it to every distinct element outcome (at most 3 for 3-4 base
elements) with a slot ordering that realizes it, so element order is searched exactly, not heuristically. Progenitor
element (if not kept) multiplies the sets by up to 7. Element rules (`require`, `forbid`) are checked on outcomes
before any inner search. Sets are processed in descending order of their root bound.

### 5.2 Branch-and-bound over the monotone remainder

```text
solve_main(context, comp_outcome, θ):
    S0   = Σ contrib(context) + Σ contrib(comp_set)                  // fixed part, exact i64
    k    = free main slots − |comp_set|
    pool = monotone candidates, sorted by marginal value at S0 (descending), compacted to the active slots A
    TOP[i][r][a] = sum of the r largest positive values of slot a among pool[i..]      // suffix top-r table, r ≤ 8
    DFS(i, depth, S, mask, drain):
        if depth == k: leaf(S); return
        if Kernel::bound(S + TOP[i][k − depth]) < θ: return                          // no completion reaches θ
        if capacity and drain + MINDRAIN[i][k − depth] > cap: return
        for j in i .. len(pool) − (k − depth) + 1:
            if pool[j].incompat & mask != 0: continue
            DFS(j + 1, depth + 1, S + contrib(pool[j]), mask | pool[j].group, drain + pool[j].drain)
    leaf(S): if bound(S) < θ: return
             s = score(S, WindowFast); s_best = max(s_best, s); keep (s, build) if s ≥ θ
```

- **Active-slot compaction**: only the 10-30 slots any pool candidate touches are carried in the DFS (`[i64; 32]`);
  `S + contrib` is a fixed-width vector add the compiler vectorizes. Fixed-point arithmetic makes every prefix sum
  bit-identical to a full recompute.
- **Suffix top-r table** makes the relaxation O(active slots) per node; ignoring incompatibilities and set coupling
  only loosens the bound (still valid). Set members contribute their bonus at the maximum still-reachable count.
- **Threshold and keep-set**: scores are the engine's ordered `Score` keys (engine.md §7.9). `s_best` is the best
  `WindowFast` leaf key so far and ε the tested bound on `|WindowFast − WindowExact| / WindowExact` (0.5 %, engine.md
  §7.2). `θ = relax(s_best, 4ε)`: same band, primary lowered by the relative margin (for TTK: T_EV × (1 + 4ε)), tie
  fields −∞. Nodes are pruned when `Bound < θ`; leaves with `WindowFast ≥ θ` enter the keep-set. Because
  `Bound ≥ WindowExact` (provable) and, **if the WindowFast error of every leaf is ≤ ε**, `s_best ≤ (1 + ε)·O` (O = the
  exact optimum), every build whose exact score is within 2ε of O survives and is kept. The ε bound is measured on the
  golden corpus and 10,000 random builds per weapon class, not proven, so the proof is reported as
  `ProvenOptimal { assumes_fast_error: Some(ε) }` ("optimal assuming the WindowFast error ≤ 0.5 %"). With
  `search.certify = true` leaves are scored with `WindowExact` instead (slower, about 4× per leaf for long windows),
  pruning uses `Bound` only, and the proof carries no assumption (`assumes_fast_error: None`). For metrics whose
  search score is already exact (`Steady` metrics, hit metrics, windows with ≤ 64 shots) ε = 0 and the proof is
  unconditional in either mode.
- **TTK metric** (E-8.26): the key is lexicographic, `(band, −T_EV, κ, −ammo_to_kill)` with band 0 killed,
  1 extrapolated, 2 never (primary = layered `dps_sustained` in band 2). `Kernel::bound` returns a key that is ≥ the key
  of every completion in this order (engine.md §7.9 gives the argument: the damage envelope kills no later and has no
  worse band), so pruning by key comparison is valid across bands. DPS and hit metrics use band 0 with the ammo
  tie-break (`ammo_window`, lower is better).
- **Ordering**: children in descending marginal value; the first incumbent is a greedy completion of the best set or
  the previous context's solution (warm start), or the user's current build if better.
- **Capacity, polarity and Forma** (spec 09 §6, D-092): the capacity mode follows 09 E-9.15 (`ignore`, `current`,
  `any_forma` (default, the game's own limit), `budget(N)`). Drain is a knapsack constraint pruned with the suffix
  minimum-drain table (drain after the best possible polarity, a valid lower bound: `SlotCost::min_charge` of each
  candidate's `wfd-data::legality::slot_cost`, 09 E-9.12, E-9.14, E-9.59); at a leaf, feasibility and the minimum Forma
  count are computed by `legality::CapacityModel::report` over the work item's fixed context (09 E-9.16-E-9.19: exact DP
  over the remaining innate polarities, at most 2^8 states, plus the exilus and stance options), built with the job's
  capacity mode and `CapacityRule::effective` (server-api.md §8.1). It is the computation of `legality::min_forma_with`,
  which the calculator's capacity report uses, so the two cannot diverge (AD-24; contract review API-13, request
  T02b-1); the optimizer has no capacity DP of its own. Under-ranked versions of a mod (`ranks = AllowLower`, T-36) are
  extra candidates in one incompatibility group. User-supplied items (riven, valence, locked slots) are never rewritten;
  if they violate a rule the search still runs and the result repeats the violation (09 §1.5).

### 5.3 Cost

A rifle with 14 composition and 22 monotone candidates has C(36, 8) = 30.3 M complete main fillings; at the `Bound`
target of 1.5 µs that is 45 core-seconds with no pruning, under 10 s on 8 cores even in the worst case; pruning
typically removes one to two orders of magnitude. A melee weapon with 16 + 40 candidates has C(56, 8) = 1.42 G
fillings and relies on pruning; the time limit turns it into an anytime search with a reported gap. These are
estimates; task T37 measures them and records the numbers in `docs/architecture/performance.md`.

---

## 6. Context search

### 6.1 Groups

| Group | Options (typical) | Strategy |
|---|---|---|
| weapon arcane (arch-guns: 2) | 5-15 | enumerate |
| exilus | 5-20 | enumerate; element exilus mods join the composition set at the exilus position |
| stance × combo × melee plan | 2-5 stances × 2-6 combos × the plans of 07 §8.1 allowed by the objective (T-30) | enumerate (fixes the inspiration's "first combo only"); the plan is searched only when the user leaves it on `auto` and enables plan search |
| Incarnon perks | ≤ 3 per tier, ≤ 4 tiers → ≤ 81 | enumerate jointly |
| frame arcanes | pairs of ~10 → ≤ 45 | enumerate pairs |
| aura | 5-10 | enumerate |
| frame mods | up to 8 general + exilus from ~10 damage-relevant ones (09 E-9.2) | small branch-and-bound with the same machinery |
| archon shards | multisets of 5 from the allowed types (14 types → 8,568) | enumerate; each multiset is one precompiled vector (the shard finalizer resolves Violet scaling) |
| kitgun grip × loader, zaw grip × link | ≤ 100 | enumerate (the inspiration cannot search these) |
| progenitor element | 7 | joins the composition enumeration; value from the build (default 60 %), as in the inspiration, whose optimizer reads the user's value field (`tb()`, `JS L17746-17755`) |
| buff scenario | allowed buffs | portfolio (§6.3) |

### 6.2 Algorithm (`context.rs`)

1. **Seed**: the user's current context (locked groups stay fixed); empty groups get the option with the best
   marginal value on a greedy main build.
2. **Block-coordinate ascent**: for each group in turn, score every option with the current main fixed
   (`WindowFast`); for the top `m = 3` options, re-solve main exactly (warm-started branch-and-bound); accept the best.
   Sweeps repeat until no group changes.
3. **Context beam**: keep the `B = context_beam` best distinct contexts after each sweep and continue from each,
   which removes the order dependence of a single ascent path.
4. **Cross-group local search**: from the best `B` full builds, apply 1-swaps and 2-swaps across any two slots of any
   groups (including main ↔ exilus moves and element re-orderings), with exact fixed-point deltas and `WindowFast`
   scoring, until no move improves by more than 1e-9 relative. Moves are deterministic (seeded restarts only when the
   user asks).
5. **Best per damage mix**: the global threshold prunes whole mixes that cannot compete, so the per-mix table is a
   second, cheaper pass: for the 15 mixes with the highest root bounds, the main search reruns with a mix-local
   threshold, warm-started.
6. **Riven search** (`RivenPolicy::Search`): an outer loop over trait combinations legal for the weapon class (04 §8,
   09 §12: the class from `legality::riven_class`, E-9.50; the traits from `legality::riven_traits_for`, which applies
   T2 and T5 of E-9.55 and never offers a vintage companion trait on a companion riven, §12.3) with values at the
   chosen quality u of the value formula (04 E-4.18, disposition 1.2; u = 1 gives e_1 of E-9.56, v_hi for a bonus and
   v_lo for a curse); each riven is one main candidate; pruned by the same bound; reports the best riven per trait set
   (`OptResult.riven_sets`).

**Proof scope.** `ProvenOptimal { scope: MainGivenContext }` when the branch-and-bound finished for the final
context; the context is proven optimal only for groups enumerated jointly. Otherwise the result carries the bound gap
of the unfinished search or `Heuristic`.

### 6.3 Buff portfolio

`BuffPolicy { allowed, portfolio: true }`: the scenarios of interest ("none", each allowed buff alone, all allowed
buffs) are solved as separate jobs in parallel. `OptResult.buff_table` gives, per scenario, the best build, its score,
the value of the buff on the no-buff best build, and the gain of rebuilding for it; the user picks the scenario. With
`portfolio: false` all allowed buffs are on for the whole search (inspiration behaviour).

---

## 7. Scoring and finalists

1. **Search scores**: `Bound` for pruning, `WindowFast` (= `WindowExact` for windows ≤ 64 shots) for leaves,
   `Steady` for burst and sustained metrics, EV TTK for the TTK metric, one kernel point for hit metrics. The search
   window equals the ranking window, the T-01 of the scenario the results carry (`objective.window_s` when given, §1),
   which fixes the inspiration's window mismatch (`I:opt` Q10).
2. **Finalists**: the merged keep-set (every build above the final θ, plus the per-mix bests) is re-scored with
   `WindowExact` (TTK: EV TTK); the top `keep` (default 32) are the finalists, sorted by `(Score desc, canonical key
   asc)`. If the
   keep-set exceeds 100,000 builds (a flat optimum), θ is raised to the 100,000-th score and the tail is marked
   `Heuristic`.
3. **Monte Carlo confirmation**: the top `mc_confirm.finalists` (default 12) run `runs` MC runs each with the same
   seed (common random numbers, 08 E-8.16), in parallel. The result shows mean ± 95 % CI per finalist and flags any
   finalist that fails 08 E-8.24 (`|EV − x̄| > 2.5758293035489004·SE + 0.02·|x̄|`), listing the active gap classes
   (`RankedBuild.gap_classes`). When the metric is `dps_window` and the flag fires for any finalist, the finalists are
   re-ranked by the MC mean (log `opt.log.mc_rerank`) and each keeps its EV rank in `RankedBuild.ev_rank` (08 §10: MC is
   the game-faithful evaluator); for every other metric the flag is reported and the EV order stays, and "rank by MC
   mean" is a UI option that re-sorts the finalists (contract review API-10). A cancel during MC
   (`Err(EngineError::Cancelled)` from `Engine::mc`) keeps the EV-ranked finalists (C-35).
4. **Diagnostics of the best build**: leave-one-out score per slot, next-best replacement per slot, score with
   T-02 = `none` (condition sensitivity), capacity and Forma report (09 E-9.16) unless the mode is `ignore`. Every
   ranked build carries its capacity report and the violations of its user-supplied items (`RankedBuild.capacity`,
   `.violations`; 09 §1.5 rule 3); `OptResult.capacity` equals `best.capacity`.

---

## 8. Parallelism and determinism

- One rayon pool for optimizer jobs (`max(1, cores − 1)` threads by default, AD-15); one job at a time.
- **Work items** are generated in a canonical order: (composition outcome × first two DFS levels) for the main search,
  options of a group for context ascent, finalists for MC. They are processed in **batches** of a fixed size W = 64
  (independent of the thread count); the items of a batch run in parallel.
- **Threshold**: θ is published only at batch barriers. Every item of batch b starts from `θ_b`, computed from the
  completed results of batches 0 .. b − 1 merged in canonical item order; inside an item, the item's own leaves raise
  its local threshold as usual, but no item sees another item's leaves of the same batch. Which nodes an item visits
  therefore depends only on the item and `θ_b`, never on thread timing.
- **Budgets**: with `node_budget = N`, each item gets a deterministic share (`N` divided over the items of the batch
  in proportion to their subtree-size estimates, remainders in canonical order; budget an item does not use goes to
  the next batch). An item that exhausts its share stops and records its subtree's bound as the gap. The result is
  therefore bit-identical for any thread count and machine (I-5 for the optimizer).
- **Wall-clock limits** (`time_limit_s`, cancellation) end the search at a batch boundary or inside the running
  batch; the result is valid (anytime) but reproducible only with `threads = 1`, which the result states
  (`SearchStats.reproducible = false`).
- Keep-sets are per item, merged in canonical item order, filtered with the final θ and sorted by
  `(Score desc, canonical key asc)`.
- Canonical build key: xxh3-128 of (sorted non-element item ids, element outcome with realized order, context
  positions). The memo cache (`FxHashMap<u128, f64>`, per item, LRU 1 M entries) stores f64 scores (no rounding, so
  near-ties keep their order) and is scoped to one `CompiledScenario`, so it can never serve a stale score
  (`I:opt` Q12). A cache hit returns exactly what a recomputation would, so caching never changes a result.
- Test (testing.md §6): the same job with 1, 2 and 8 threads and a `node_budget`, repeated 5 times, gives identical
  results.

---

## 9. Progress and cancellation

```rust
// wfd-model/src/api/opt.rs
pub enum OptEvent {
    Started   { job: JobId, plan: Plan },          // positions, pool sizes after pruning, composition sets, excluded items
    Phase     { phase: OptPhase, index: u8, total: u8, message: Message },
    Progress  { phase: OptPhase, fraction: Option<f64>, evaluated: u64, bounded: u64, pruned: u64,
                rate_per_s: f64, elapsed_ms: u64, eta_s: Option<f64>, gap: Option<f64> },
    Incumbent { rank: u16, build: RankedBuild },   // every change of the top 1; top 5 changes throttled
    Finalist  { build: RankedBuild },              // after MC confirmation
    Log       { message: Message },                // i18n keys + args, never English text (I-8)
    Done      { result: OptResult },
    Cancelled { result: Option<OptResult> },       // anytime result
    Failed    { error: Message },
}
pub enum OptPhase { Prepare, Composition, MainSearch, Context, LocalSearch, PerMix, Riven, Buffs, Finalists, McConfirm }
```

- `Progress` is throttled to 4 per second. `fraction` is exact in the main-search phase (work items done / total,
  weighted by subtree size estimates) and absent in local search.
- Workers check `cancel` every 1,024 DFS nodes or 64 evaluations (≤ 100 ms latency at the performance targets).
- **Anytime**: on cancel or time limit the current incumbents and keep-set go through §7 steps 2-3 quickly (finalists
  only, MC with fewer runs if needed) and the result carries `Proof::Gap` or `Heuristic`. Cancel never loses the best
  build found. The input build is scored first, so only a cancel before that has no result (`OptError::Cancelled`,
  ended as `Cancelled { result: None }`; API-19).
- **Job id**: `optimize` does not know it; it emits `Started` with `job` empty (`events::JOB_UNSET`), and the job's sink
  stamps the id (`events::stamp_job`) before storing or sending the event (request T02d-5). The terminal event
  (`done`, `cancelled`, `failed`) is the caller's.

---

## 10. Comparison with the inspiration

### 10.1 Weaknesses addressed (`I:opt` §8-§9)

| Inspiration weakness | This design |
|---|---|
| Full stochastic simulator as score, 3 seeds, a few thousand evaluations | deterministic EV kernel, 10^5-10^6 evaluations per second per core |
| Biased fast estimate, winner's curse, 20 % trust tolerance (80 before the 2026-11-27 bundle, `min.pretty.js@7bb6` L15957-16023) | no noise in the search; MC only confirms, with CRN and CIs; EV-vs-MC deviation flagged per build |
| Elements chosen on a bare weapon; element order searched once | composition sets solved jointly with every other main mod; `ElementSolver` enumerates every distinct outcome |
| Beam pruning by marginal value drops synergies | exact branch-and-bound with valid bounds |
| Greedy perks, stance (first combo), warframe slots | joint enumeration of perks, stance × combo, shards; coordinate ascent with a context beam elsewhere |
| Order dependence of sweeps | context beam + cross-group local search to a fixed point; determinism tests |
| TTK quantization hack | continuous EV TTK; MC TTK distribution for finalists |
| Single-threaded UI-thread JavaScript | all cores via rayon in a separate process; the UI stays responsive |
| Regex exclusions on tooltip text, hard-coded lists | rule filters act on typed effects; relevance derived from the engine |
| Buff phase decides under "all allowed" only | buff portfolio with a per-scenario table |
| No capacity, polarity, rank, riven search, set awareness, per-slot locks, kitgun/zaw parts, multi-attack objective | all available as options |
| Cache key omits context; search window differs from final window | caches scoped to a compiled scenario; one window for search and ranking |

### 10.2 Inspiration-compatible preset

`Preset::InspirationCompatible` sets the inspiration's defaults (`I:feat` §17.1, `I:opt` §2): metric DPS, view
`Auto`, `NO_FACTION` on, conditions `all_max`, prohibited defaults (Mecha Empowered, Proton Snap, Arcane Blade
Charger, Arcane Primary Charger, Arcane Arachne), status-duration-only items excluded, search window
`clamp(round(100 / attack speed), 5, 20)` s, frame mods limited to one slot, capacity mode `ignore` (D-092). The
window and T-02 `all_max` are scenario inputs and are written into every `RankedBuild.input` (§1), so a ranked build's
share code reproduces its score; the capacity mode is `constraints.capacity` and is not. The audit task (T40) uses it
to show that our best build scores at least as high as WFOpt's best under our own `WindowExact`, on the oracle weapon
set.

---

## 11. Defaults

| Setting | Default | Inspiration control it maps |
|---|---|---|
| metric | `dps_window` | Metric radio (DPS, Minimum TTK, Maximum hit; `hit_fresh` exposed too) |
| attacks | first attack, weight 1 | Attack select |
| view | `auto` | Defense layer select |
| heavy | off; start combo 12 | Heavy attack + combo counter |
| window | T-01 of the scenario (20 s); the UI offers the inspiration's auto value | Search window |
| context beam | 4 (1-8) | Beam width |
| element rule | none | Exclude elements (free text, parsed in the UI) |
| rules | weak point off, no faction off (on in the inspiration preset), no on-kill off, conditions from the scenario | Build rules |
| groups | exilus and arcane on; stance on for melee; Incarnon on for Incarnon weapons; frame slots, aura, shards off | Warframe slots |
| capacity | `any_forma` (09 E-9.15); `ignore` in the inspiration preset | — (the inspiration has no capacity model) |
| strength | the scenario's T-20 | Ability strength |
| buffs | none allowed | Allowed external buffs with values |
| time limit | 120 s | — |
| finalists, MC | 32 kept, 12 confirmed with 1,000 runs, seed 0 | — |
| certify | off (proof stated as "assuming WindowFast error ≤ 0.5 %"); on in tests and when the user asks | — |
| melee plan search | off (the scenario's T-30 plan is used) | — (the inspiration has no plans) |
