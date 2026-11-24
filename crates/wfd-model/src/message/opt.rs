//! Optimizer keys (optimizer.md §9; server-api.md §8): conflicts, excluded items, phases, logs and failures.

use super::message_keys;

message_keys! {
    /// Keys of the optimizer (area `opt`).
    pub enum OptKey, area = "opt" {
        /// 409: an optimizer job is already running (server-api.md §8.1, `args.job`).
        JobRunning = "opt.job_running" [job],
        /// Excluded: the item's effects are not encoded (I-6, optimizer.md §2 step 3).
        ExcludedUnmodelled = "opt.excluded.unmodelled" [],
        /// Excluded: the item has no damage effect.
        ExcludedNoDamage = "opt.excluded.no_damage_effect" [],
        /// Excluded: on the prohibited list.
        ExcludedProhibited = "opt.excluded.prohibited" [],
        /// Excluded: draft effects and rule `exclude_draft`.
        ExcludedDraft = "opt.excluded.draft" [],
        /// Excluded: stale effects and rule `exclude_stale`.
        ExcludedStale = "opt.excluded.stale" [],
        /// Excluded: every effect is zero after a rule filter (`rule` = `"no_faction"` …, optimizer.md §2 step 4).
        ExcludedRuleFilter = "opt.excluded.rule_filter" [rule],
        /// Excluded: no contribution to any objective attack (optimizer.md §2 step 6).
        ExcludedIrrelevant = "opt.excluded.irrelevant" [],
        /// Excluded: dominated by another candidate of its pool (optimizer.md §2 step 7).
        ExcludedDominated = "opt.excluded.dominated" [by],
        /// Excluded: not legal in any free slot of the weapon (spec 09 legality).
        ExcludedIllegal = "opt.excluded.illegal" [rule],
        /// Phase message of `OptPhase::Prepare` (optimizer.md §9; one key per phase).
        PhasePrepare = "opt.phase.prepare" [],
        /// Phase message of `OptPhase::Composition` (`sets` composition sets).
        PhaseComposition = "opt.phase.composition" [sets],
        /// Phase message of `OptPhase::MainSearch`.
        PhaseMainSearch = "opt.phase.main_search" [],
        /// Phase message of `OptPhase::Context` (`sweep` = 1-based sweep number).
        PhaseContext = "opt.phase.context" [sweep],
        /// Phase message of `OptPhase::LocalSearch`.
        PhaseLocalSearch = "opt.phase.local_search" [],
        /// Phase message of `OptPhase::PerMix` (`mixes` searched).
        PhasePerMix = "opt.phase.per_mix" [mixes],
        /// Phase message of `OptPhase::Riven` (`trait_sets` searched).
        PhaseRiven = "opt.phase.riven" [trait_sets],
        /// Phase message of `OptPhase::Buffs` (`scenarios` of the portfolio).
        PhaseBuffs = "opt.phase.buffs" [scenarios],
        /// Phase message of `OptPhase::Finalists` (`count` finalists).
        PhaseFinalists = "opt.phase.finalists" [count],
        /// Phase message of `OptPhase::McConfirm` (`finalists` × `runs`).
        PhaseMcConfirm = "opt.phase.mc_confirm" [finalists, runs],
        /// Log: a new best build.
        LogIncumbent = "opt.log.incumbent" [score],
        /// Log: the time limit ended the search (anytime result, optimizer.md §9).
        LogTimeLimit = "opt.log.time_limit" [seconds],
        /// Log: the node budget ended the search (optimizer.md §8).
        LogNodeBudget = "opt.log.node_budget" [nodes],
        /// Log: the keep-set exceeded 100,000 builds; its tail is heuristic (optimizer.md §7 step 2).
        LogKeepSetCapped = "opt.log.keep_set_capped" [kept],
        /// Log: the EV-vs-MC flag fired for a finalist and the finalists are re-ranked by the MC mean (08 §10).
        LogMcRerank = "opt.log.mc_rerank" [gap_classes],
        /// Log: a user-supplied item violates a rule; the optimizer keeps it and repeats the violation (09 §1.5 rule 3).
        LogUserViolation = "opt.log.user_violation" [rule, slot],
        /// Failed: a TTK metric was requested for a group sum (08 §10: no group TTK exists with T-22 off).
        FailedTtkGroupSum = "opt.failed.ttk_group_sum" [],
        /// Failed: the objective names no attack of the weapon, or a weight is negative (08 §10).
        FailedObjective = "opt.failed.objective" [attack],
        /// Failed: the weighted attacks do not share one metric and view (08 §10).
        FailedMixedMetric = "opt.failed.mixed_metric" [],
        /// Failed: no build satisfies the constraints (locks, element rules, capacity).
        FailedInfeasible = "opt.failed.infeasible" [],
        /// Failed: the job panicked (server-api.md §11); `error_id` names the log entry.
        FailedInternal = "opt.failed.internal" [error_id],
    }
}
