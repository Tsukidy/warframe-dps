//! The optimizer-facing kernel (engine.md §7.9; optimizer.md §4-§7): scores of build points at a fidelity, and upper
//! bounds of every completion of a partial build. No heap allocation in `score` and `bound` after warm-up. Owner:
//! T15c.

use smallvec::SmallVec;
use wfd_model::message::Message;
use wfd_model::{Attack, AttackIdx, View};

use crate::EvalUnit;
use crate::compile::CompiledScenario;
use crate::elements::ElemOutcome;
use crate::error::EngineError;
use crate::ev::score::Score;
use crate::stats::{Coeff, MechSet, StatVec};

/// The optimizer metric (08 §10). The model's DTO enum, re-exported: it includes `TtkNoarmor` (ruling C-28) and
/// spells the in-fight hit metric `HitInfight` (engine.md §7.9 wrote `HitInFight`).
pub use wfd_model::api::Metric;

/// What the optimizer maximizes (engine.md §7.9; 08 §10).
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveSpec {
    /// The metric.
    pub metric: Metric,
    /// The view (`Layered` for the TTK metrics).
    pub view: View,
    /// The weighted objective `Σ_u w_u·M_u` over evaluation units u with weights w_u ≥ 0 (08 §10): single attacks
    /// (`combined = false`, one member), T-22 combined units (`combined = true`) or the members of a family. The TTK
    /// metrics (`ttk`, `ttk_noarmor`, `ammo_to_kill`) take exactly one unit, a single attack or a combined unit (08
    /// §10: no group TTK exists for a T-22-off sum). Replaces `attacks: [(AttackIdx, f64)]`, which could not tell a
    /// combined group from a weighted sum (contract review ENG-24).
    pub units: SmallVec<[(EvalUnit, f64); 2]>,
    /// Search and ranking window (s; default T-01).
    pub window: f64,
}

impl ObjectiveSpec {
    /// Every attack of the objective's units, in unit order, without repeats: the objective attacks whose stat
    /// vectors `CompiledBuild.sv` holds (engine.md §3).
    pub fn attacks(&self) -> SmallVec<[AttackIdx; 4]> {
        let mut out: SmallVec<[AttackIdx; 4]> = SmallVec::new();
        for (u, _) in &self.units {
            for a in &u.attacks {
                if !out.contains(a) {
                    out.push(*a);
                }
            }
        }
        out
    }
}

/// EV fidelities (engine.md §7.2). `Bound` is [`Kernel::bound`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fidelity {
    /// One point at the steady state (+ the Heat window mean).
    Steady,
    /// Gauss-Legendre over segments; ≤ 0.5 % from `WindowExact`, else falls back to it.
    WindowFast,
    /// Every shot and every DoT tick in [0, t_win]: the definition of `dps_window`.
    WindowExact,
}

/// A build as the optimizer holds it: borrowed compiled parts (engine.md §7.9), per build attack k as in
/// `CompiledBuild` (`attacks[k]` is the attack of `sv[k]`, `elements[k]`, `coeffs[k]`, `mech[k]`; contract review
/// ENG-7). `CompiledBuild::point` borrows one from a compiled build.
#[derive(Debug, Clone, Copy)]
pub struct BuildPoint<'p> {
    /// The build attacks.
    pub attacks: &'p [AttackIdx],
    /// One stat vector per build attack.
    pub sv: &'p [StatVec],
    /// The element outcome per build attack (01 §4.11; scope per attack, 01 Table 4.10-B, E-1.13).
    pub elements: &'p [ElemOutcome],
    /// Dynamic terms per build attack.
    pub coeffs: &'p [SmallVec<[Coeff; 4]>],
    /// Mechanics per build attack.
    pub mech: &'p [MechSet],
    /// Active condition ids of the build (`CompiledBuild.conditions`, contract review ENG-2).
    pub conditions: &'p [u16],
}

/// Static per-attack data the kernel reuses across build points (attack records, schedule skeletons). T15c.
#[derive(Debug, Clone, Default)]
pub struct AttackStatics<'a> {
    /// The objective attacks and their records.
    pub attacks: SmallVec<[(AttackIdx, &'a Attack); 2]>,
}

/// The scoring kernel of one compiled scenario and objective (engine.md §7.9).
#[derive(Debug, Clone)]
pub struct Kernel<'a> {
    cs: &'a CompiledScenario,
    objective: &'a ObjectiveSpec,
    statics: AttackStatics<'a>,
}

impl<'a> Kernel<'a> {
    /// A kernel over prepared statics (`Engine::kernel` builds them).
    pub fn new(
        cs: &'a CompiledScenario,
        objective: &'a ObjectiveSpec,
        statics: AttackStatics<'a>,
    ) -> Self {
        Kernel {
            cs,
            objective,
            statics,
        }
    }

    /// The compiled scenario.
    pub fn scenario(&self) -> &'a CompiledScenario {
        self.cs
    }

    /// The objective.
    pub fn objective(&self) -> &'a ObjectiveSpec {
        self.objective
    }

    /// The per-attack statics.
    pub fn statics(&self) -> &AttackStatics<'a> {
        &self.statics
    }

    /// The score of `x` at fidelity `fid` (08 §10, E-8.26 ordering); an error (engine.md §15) is recorded in `ws`
    /// ([`Workspace::fail`]) and the score is then [`Score::WORST`]. T15c.
    pub fn score(&self, x: &BuildPoint, fid: Fidelity, ws: &mut Workspace) -> Score {
        let _ = (x, fid, ws);
        todo!("T15c")
    }

    /// Upper bound, in the order of [`Score`], of `score(y, WindowExact)` for every y whose Inc slots are ≤ `upper`'s
    /// and whose Unknown slots, elements and mechanics equal `upper`'s (optimizer.md §4; engine.md §7.9 gives the
    /// argument for TTK). An error is recorded in `ws` as for [`Kernel::score`]. T15c.
    pub fn bound(&self, upper: &BuildPoint, ws: &mut Workspace) -> Score {
        let _ = (upper, ws);
        todo!("T15c")
    }
}

/// Per-thread scratch arrays of the kernel; no heap allocation in `score` and `bound` after warm-up (engine.md §13).
///
/// It is also the warning and error channel of the EV evaluations (contract review ENG-16): `Engine::ev`,
/// [`Kernel::score`] and [`Kernel::bound`] return plain values, so they report warnings (`ev.steady.no_convergence`,
/// `ev.approx.gate_split_limit`, `ttk.extrapolated`; engine.md §15) with [`Workspace::warn`] and an error (a schedule
/// over the event bound, `EngineError::ScheduleTooLong`) with [`Workspace::fail`], keeping the first error as
/// `McRng` does. A failed evaluation's value is meaningless (a score is [`Score::WORST`]); the caller drains the
/// channel with [`Workspace::check`] and [`Workspace::take_warnings`] (T17 puts the warnings into
/// `CalcReport.warnings`).
#[derive(Debug, Clone, Default)]
pub struct Workspace {
    scratch: Vec<f64>,
    warnings: Vec<Message>,
    error: Option<EngineError>,
}

impl Workspace {
    /// An empty workspace (grows on first use).
    pub fn new() -> Self {
        Workspace::default()
    }

    /// Scratch space of at least `n` values (for the owning task's walks).
    pub fn scratch(&mut self, n: usize) -> &mut [f64] {
        if self.scratch.len() < n {
            self.scratch.resize(n, 0.0);
        }
        &mut self.scratch[..n]
    }

    /// Records a warning of the current evaluation; an equal message already recorded is not repeated.
    pub fn warn(&mut self, m: Message) {
        if !self.warnings.contains(&m) {
            self.warnings.push(m);
        }
    }

    /// The warnings recorded since the last [`Workspace::take_warnings`].
    pub fn warnings(&self) -> &[Message] {
        &self.warnings
    }

    /// Takes the recorded warnings, in recording order.
    pub fn take_warnings(&mut self) -> Vec<Message> {
        std::mem::take(&mut self.warnings)
    }

    /// Records an error of the current evaluation; the first error is kept.
    pub fn fail(&mut self, e: EngineError) {
        if self.error.is_none() {
            self.error = Some(e);
        }
    }

    /// The recorded error, if any.
    pub fn error(&self) -> Option<&EngineError> {
        self.error.as_ref()
    }

    /// `Err` with the recorded error, which is cleared; `Ok` when none was recorded.
    pub fn check(&mut self) -> Result<(), EngineError> {
        match self.error.take() {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_grows_and_reuses() {
        let mut ws = Workspace::new();
        assert_eq!(ws.scratch(4).len(), 4);
        ws.scratch(4)[0] = 1.5;
        assert_eq!(ws.scratch(2).len(), 2);
        assert!((ws.scratch(2)[0] - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn workspace_carries_warnings_and_the_first_error() {
        use wfd_model::message::EngineKey;
        let mut ws = Workspace::new();
        assert!(ws.check().is_ok());
        ws.warn(EngineKey::TtkExtrapolated.message());
        ws.warn(EngineKey::TtkExtrapolated.message());
        ws.warn(EngineKey::GateSplitLimit.message());
        assert_eq!(ws.warnings().len(), 2, "an equal warning is recorded once");
        ws.fail(EngineError::ScheduleTooLong { shots: 20_000_000 });
        ws.fail(EngineError::Cancelled);
        assert!(matches!(
            ws.error(),
            Some(EngineError::ScheduleTooLong { .. })
        ));
        assert!(matches!(
            ws.check(),
            Err(EngineError::ScheduleTooLong { .. })
        ));
        assert!(ws.check().is_ok(), "check clears the error");
        assert_eq!(ws.take_warnings().len(), 2);
        assert!(ws.warnings().is_empty());
    }

    #[test]
    fn objective_attacks_are_the_units_members_once() {
        let unit = |a: &[u8], combined| EvalUnit {
            attacks: a.iter().map(|i| AttackIdx(*i)).collect(),
            combined,
        };
        let o = ObjectiveSpec {
            metric: Metric::DpsWindow,
            view: View::Armor,
            units: smallvec::smallvec![(unit(&[0, 2], true), 1.0), (unit(&[2], false), 0.5)],
            window: 20.0,
        };
        assert_eq!(o.attacks().as_slice(), &[AttackIdx(0), AttackIdx(2)]);
    }
}
