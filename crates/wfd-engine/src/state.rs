//! State variables (domain-model.md §12.2): the quantities a dynamic term multiplies by. EV reads expectations over
//! their distributions (engine.md §7.1), MC reads live values (engine.md §8.1). Owner: T06 (the table and the
//! distribution helpers; T15a and T16 read them).

use smallvec::SmallVec;
use wfd_model::StatusType;

/// One state variable (domain-model.md §12.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateVar {
    /// n, the distinct-status count (T-05; 03 E-3.12, 08 E-8.10).
    Distinct,
    /// n_s, live stacks of a status (03 §6.3, external stacks included).
    Stacks(StatusType),
    /// 1{n_s ≥ 1}.
    Present(StatusType),
    /// The target is Frozen (03 §8.6).
    Frozen,
    /// Health fraction below the threshold (false in non-depleting views, live in the layered pass).
    HealthBelow(f64),
    /// M(c̄), the melee combo multiplier (07 §6; T-10 fixes it).
    ComboTier,
    /// The sniper combo multiplier (06 §7.2; T-12 fixes it).
    SniperCombo,
    /// Stacks of a class (b) conditional effect (04 E-4.8 / E-4.9); index into the scenario's condition table.
    CondStacks(u16),
    /// A `counter_bonus` counter; `StatKey` index of its `mech.<n>` slot.
    Counter(u16),
    /// (Mag − left)/Mag of the latest reload while it lasts (06 ReloadRec, `Per::ReloadedFraction`).
    ReloadedFraction,
    /// The shot is the first of its magazine (06 E-6.13).
    FirstShot,
    /// The shot is the last of its magazine (06 E-6.13).
    LastShot,
}

/// The state variables that exist in one scenario (`CompiledScenario.state_vars`), in canonical order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StateVarTable {
    /// The variables.
    pub vars: SmallVec<[StateVar; 8]>,
}

impl StateVarTable {
    /// Index of a variable, if the scenario has it.
    pub fn position(&self, v: StateVar) -> Option<usize> {
        self.vars.iter().position(|x| *x == v)
    }
}
