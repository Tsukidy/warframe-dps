//! Damage instances as the evaluators and the mechanics see them (contract review ENG-1, CONTRACTS.md "Contract review
//! fixes (engine)"): the identity of one instance of the schedule (08 §4.1 `Instance`, E-8.0), its outcome after its
//! own rolls (the crit tier, the delivered procs, the weapon side), the damage instances a mechanic derives from it
//! (extra hits E-14, Melee Duplicate copies 04 E-4.15, flat damage on proc, Blast-detonation extra hits 04 E-4.13a) and
//! the procs it applies in P4b (03 §7.1). The EV kernel ([`crate::ev::kernel::point`]), both mechanic traits
//! ([`crate::mechanics::MechanicEv`], [`crate::mechanics::MechanicMc`]) and the MC simulator use these types, so the
//! tasks that implement them (T14, T15a, T16) exchange instances through one contract. Owner: T14 (changes through
//! a CONTRACTS.md entry).

use wfd_model::{AttackIdx, DamageVec, N_ST, StatusType};

use crate::enemy::InstanceClass;
use crate::ev::kernel::ProcMass;
use crate::mc::events::Anchor;
use crate::outputs::SourceKind;
use crate::resolve::{HitPart, TierDist, XhProc};
use crate::schedule::ShotRec;
use crate::status::{HitWeaponSide, ProcSnapshot};

/// Kinds of damage instances (08 §4.1 `Instance.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceKind {
    /// A pellet, a merged beam tick (CD-10), a melee hit or a non-radial child instance (08 E-8.0).
    Direct,
    /// A radial instance: an explosion, a slam or heavy slam, a radial child (02 E-2.10 radial branch; 07 §7.5).
    Radial,
    /// An extra hit (00 E-14; 04 §6), including the Blast-detonation extra hit of 04 E-4.13a.
    ExtraHit,
    /// A Melee Duplicate copy (04 E-4.15; 08 §4.2).
    Duplicate,
    /// One tick of a lingering child (06 E-6.25; 08 E-8.0).
    LingeringTick,
}

/// The identity of one damage instance of a schedule (08 §4.1, E-8.0): what the EV kernel evaluates at one point and
/// what the MC simulator processes in P4a.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Instance<'s> {
    /// The kind.
    pub kind: InstanceKind,
    /// The anchor (a, i, j) of E-8.0 (derived instances: their parent's anchor, E-8.16). EV reads `j` for the
    /// sequential-pellet terms of 08 E-8.10; MC keys its draws by it.
    pub anchor: Anchor,
    /// The attack of the instance (a child or reload-triggered attack for their instances).
    pub attack: AttackIdx,
    /// The schedule record the instance belongs to (its shot or melee hit; a child instance: its parent shot), with
    /// the flags of 06 E-6.13 and 07 §7.5 and the melee hit record of 07 §8.2 (`ShotRec::melee`).
    pub shot: &'s ShotRec,
}

/// The crit outcome of an instance, as far as the evaluator knows it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TierOutcome {
    /// MC: the rolled tier k (02 E-2.6, Critical Promotion included) and its crit factor c_k (00 E-12, 02 E-2.9 at
    /// the live Puncture and Cold stacks).
    Rolled {
        /// Tier k.
        k: u32,
        /// c_k.
        c: f64,
    },
    /// EV: the tier distribution of 02 E-2.7 (marginal over the Puncture stacks) and, per tier of `dist`, the
    /// conditional expectation of the crit factor `c[i] = E[c | k = dist.base + i]` over the PUN and COL groups (08
    /// E-8.9 step 3).
    Expected {
        /// The tier distribution.
        dist: TierDist,
        /// E[c | k = dist.base + i], i = 0, 1, 2.
        c: [f64; 3],
    },
}

/// The procs an instance delivers by its own rolls (02 §4.7, E-2.12-E-2.17), before any mechanic adds procs.
// One evaluator fills each variant and both are read by reference; boxing the EV variant would cost a heap
// allocation per instance and the `Copy` bound (engine.md §13).
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcOutcome {
    /// MC: procs of each status the instance delivers (rolled, forced, exclusive groups resolved), applied in P4b.
    Rolled([u8; N_ST]),
    /// EV: the instance's expected procs (mass, presence, certain part; 08 E-8.2).
    Expected(ProcMass),
}

/// The outcome of one instance after its own rolls, which the mechanics react to (contract review ENG-1 (c)): the
/// crit tier (Melee Duplicate on k = 1, Devouring Attrition on k = 0, `counter_bonus.reset_on` tiers, `proc_on_event`
/// on crit), the delivered procs (`proc_on_event` on proc, `stack_present_statuses` on Knockdown or Lifted), the
/// weapon side (extra hits read H_w, 00 E-14), the part and the kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstanceOutcome<'s> {
    /// The instance.
    pub inst: Instance<'s>,
    /// The hit location's terms (02 E-2.10; the radial part for radial instances).
    pub part: HitPart,
    /// The weapon side of the instance at crit factor 1: `W_t(k) = c_k·w.w[t]` and `H_w = c_k·w.total` (00 E-12,
    /// E-14); F and Part as applied to the instance. With the instance's own `g_mult` of [`crate::mc::McSink`] not
    /// yet applied (MC). [`crate::status::extra_hit()`] expects the crit-scaled form, so a mechanic scales `w` and
    /// `total` by c_k (MC) before calling it, or calls it at c = 1 and marks the result `DerivedHit::crit_scaled`
    /// (EV; contract review ENG-R4).
    pub w: HitWeaponSide,
    /// The crit outcome.
    pub tier: TierOutcome,
    /// The delivered procs.
    pub procs: ProcOutcome,
}

/// A damage instance a mechanic derives from the current instance (00 E-14 extra hits, 04 E-4.15 Melee Duplicate
/// copies, flat damage on proc, 04 E-4.13a Blast-detonation extra hits), on the weapon side only.
///
/// The mechanic never applies the enemy side itself: the evaluator applies 05 E-5.24 once per view of MC pass V (on
/// both `og_present` timelines, 08 §4.5), routes it once in the layered passes (05 E-5.25), or, in EV, takes its
/// expectation jointly with the parent's state as a term of the parent instance (08 E-8.9 step 1). So a mechanic is
/// called once per instance whatever the number of views, and its draws keep their E-8.16 ordinals (contract review
/// ENG-1 (b)).
///
/// The procs of a derived instance belong to the mechanic that emits it (contract review ENG-R4, pass 3): it rolls
/// them itself with the derived ids of 08 E-8.16 at the parent's anchor (`"xh/<source>/<name>"`, `"dup/<name>"`,
/// `"xd/<name>"`, which repeat the ids of 02 4.8 such as `"sc_count"`) and emits them as [`ProcApply`] in
/// `McSink.procs` (MC), or adds their expectation through `MechanicEv::ev_procs` into the event's `ProcMass` and its
/// DoTs into `EvAcc.dots` (EV). The evaluators never roll, add or apply procs from a `DerivedHit`: they evaluate its
/// damage only, so no proc is applied twice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DerivedHit {
    /// Its breakdown source (08 §8.2).
    pub source: SourceKind,
    /// `ExtraHit` or `Duplicate` (a flat damage on proc is an `ExtraHit` of source `FlatOnProc`).
    pub kind: InstanceKind,
    /// Weapon-side damage per type. MC: final. EV: at the parent's crit factor 1 when `crit_scaled` (the evaluator
    /// multiplies by the parent's c), else final.
    pub w: DamageVec,
    /// EV: the damage scales with the parent's crit factor (00 E-14: H_w = Σ_t W_t(k)); MC: always false.
    pub crit_scaled: bool,
    /// Existence weight: EV the probability that the instance exists (Melee Duplicate `p_dup·P(k = 1)`, 08 §4.2;
    /// TV-08-28), MC 1.
    pub weight: f64,
    /// Its status behaviour (rolls the weapon's SC, a forced proc, or none; 04 §6). Informational (breakdown, traces,
    /// Explain): the emitting mechanic has already rolled or added these procs (type doc; ENG-R4).
    pub status: XhProc,
    /// Its class for the enemy side: inherited from the parent instance (05 E-5.20).
    pub class: InstanceClass,
    /// Its shield-gate factor g (05 E-5.25: 1 for extra hits and special instances; a Duplicate copy takes its
    /// parent's).
    pub gate_factor: f64,
}

/// One application of a status in P4b of the current shot (03 §7.1), emitted by a mechanic (contract review ENG-1
/// (d)): the procs of an extra hit or a Melee Duplicate copy (rolled by the mechanic with its derived ids, see
/// [`DerivedHit`]), `proc_on_event`, `proc_after_ticks`, `stack_present_statuses`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcApply {
    /// The status.
    pub status: StatusType,
    /// The anchor of the instance that made it (E-8.16: derived draws of its tick groups and fuses).
    pub anchor: Anchor,
    /// The attack whose proc applies it (`StackEntry.source`, 08 §6.2).
    pub source: AttackIdx,
    /// Stacks applied (1; N of 04 E-4.20 for `stack_present_statuses`).
    pub stacks: u8,
    /// Its duration D_s (s, before the grid of 08 E-8.0a): 03 E-3.1 with the source's SD, SD_s and an extra hit's
    /// `status_duration_add`.
    pub duration: f64,
    /// The snapshotted weapon-side terms of a damaging status (SLA, TOX, ELE, GAS DoTs, HEA pool, BLA fuse; 00 E-13,
    /// CD-05), with `plus_one` false for the forced proc of an extra hit (00 E-14) and Primary Debilitate (03 §9.8);
    /// `None` for the other statuses.
    pub snap: Option<ProcSnapshot>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_contract_types_are_copy() {
        fn assert_copy<T: Copy + Send + Sync>() {}
        assert_copy::<Instance<'static>>();
        assert_copy::<InstanceOutcome<'static>>();
        assert_copy::<DerivedHit>();
        assert_copy::<ProcApply>();
    }
}
