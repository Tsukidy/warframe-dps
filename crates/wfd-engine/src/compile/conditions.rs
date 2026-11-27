//! Condition classes and the T-02 compile table (domain-model.md §12; 00 §6.2): derived class, mode, overrides,
//! constants versus state-variable coefficients; and the scenario's **condition table** (contract review ENG-2), the
//! one record of every condition that stays dynamic in a scenario, which the evaluators index. Owner: T06.
//!
//! Under the scenario's T-02 mode and overrides most conditional effects compile to constants (domain-model.md
//! §12.2). The rest stay dynamic and get one [`CompiledCondition`] each in `CompiledScenario.conditions`, allocated
//! at L0 like the `mech.<n>` slots, in canonical (`EffectRef`, rank) order, with the rank applied (09 E-9.13,
//! `RankCurve::value`). **Allocation rule** (contract review ENG-R8, pass 3): L0 sees no candidate pool (the
//! optimizer builds its pools after L0, optimizer.md §2, and `compile_pick` cannot add entries), so it allocates from
//! the scenario's fixed sources plus, without an objective (the calculator), the build's own picks at their ranks,
//! and with an objective (the optimizer) every bundle item compatible with the build's weapon (09 legality) at every
//! rank the optimizer may pick (maximum rank; every rank with T-36): a superset of the build and of any pool, so a
//! candidate's ids exist before it is compiled and stay stable across candidates. The
//! index of an entry is its **condition id**, the `u16` of `StateVar::CondStacks`, `StateVar::Switch`,
//! `GateSet::ids`, `McEventKind::ConditionExpiry` and `McEventKind::Switch`; a contribution names the ids it brings
//! in `SparseContrib.conditions`, and a compiled build lists its active ids in `CompiledBuild.conditions`. The slot
//! and coefficient a condition feeds are the `Coeff` terms that read it (domain-model.md §18), not a field here.

use wfd_model::{ClassSet, Cond, EffectKey, StackModel, TargetPred};

use crate::stats::EffectRef;

/// One dynamic condition of a scenario (contract review ENG-2): everything 04 E-4.8 (MC), 04 E-4.9 and 08 E-8.15,
/// E-8.15b (EV), the on/off processes of 08 E-8.9 step 4c and E-8.18, the 06 §5.9 timing buffs and
/// `CalcReport.conditions` need.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledCondition {
    /// The effect whose condition this is.
    pub source: EffectRef,
    /// Its key: the MC mechanic ids `"eff/<key>"` (trigger draws) and `"eff/<key>/phase"` (the per-run phase of a
    /// switch) of 08 E-8.16.
    pub key: EffectKey,
    /// The derived classes of the effect (domain-model.md §12.1), (u) added for buffs.
    pub class: ClassSet,
    /// What varies.
    pub kind: CondKind,
}

/// The three kinds of dynamic condition (domain-model.md §12.2 under `simulated`).
#[derive(Debug, Clone, PartialEq)]
pub enum CondKind {
    /// Class (b) stacks gained on the attack's own events from 0 stacks: 04 E-4.8 (MC), 04 E-4.9 and 08 E-8.15,
    /// E-8.15b (EV); also an effect without a maximum under `all_max` (08 E-8.15). Read through
    /// `StateVar::CondStacks(id)`.
    Stacks(Box<StackCond>),
    /// A class (c) or (u) effect with uptime u_e < 1 that a nonlinear use reads (08 E-8.9 step 4c, A-12; 04 §4.3):
    /// on for `on` s of every `period` s, with a random phase per MC run (08 E-8.18, mechanic `"eff/<key>/phase"`);
    /// EV mixes the on and off branches with u_e = on/period. Read through `StateVar::Switch(id)`.
    Switch {
        /// On-time of one cycle (s).
        on: f64,
        /// Period (s).
        period: f64,
    },
    /// A class (a) threshold predicate used as a gate or a nonlinear condition (domain-model.md §12.2:
    /// `HasStatus`, `Frozen`, `HealthBelow`, `DistinctBelow`, `StatusCountAtLeast`): the probability of the
    /// predicate (EV, 08 E-8.9 steps 2 and 4a) or its live value (MC) before the instance (CD-04). A linear count
    /// term reads `StateVar::Distinct`, `Stacks` or `Present` directly and needs no entry.
    Target(TargetPred),
}

/// The class (b) stack condition of [`CondKind::Stacks`].
#[derive(Debug, Clone, PartialEq)]
pub struct StackCond {
    /// The gain event with its event filters, read as domain-model.md §12.1 says: the effect's `stacks.gain_on` when
    /// set, else its `when`; the chance of its `Trigger` at the picked rank (`chance_rank` applied).
    pub gain: Cond,
    /// The activity condition evaluated at use: the effect's `when` when `stacks.gain_on` is set, else
    /// `Cond::Always`.
    pub active: Cond,
    /// The stack model with `max` and `duration` at the picked rank (`max_rank`, `duration_rank` applied); `None` = a
    /// single stack (domain-model.md §12.2 "class (b), single stack": m = 1, g = 1, no expiry).
    pub stacks: Option<StackModel>,
}
