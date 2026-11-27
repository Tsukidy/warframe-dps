//! MC events (08 §6.3, E-8.18; engine.md §8.1): a binary heap keyed `(τ̂, phase, sub_key, seq)` (08 §4.3, §6.2,
//! E-8.0a) with the phases P1-P4 of 03 §7.1, and the anchors of 08 E-8.16. P4 is one heap phase: each shot S runs
//! its steps a-d (instances, procs, zero-delay ticks, DoT detonation) inside its own P4 step, in the shot order of
//! 08 §4.2, so a stack applied in P4b of S affects every later shot at τ (03 §7.2). Owner: T16.

use std::cmp::Ordering;

use smallvec::SmallVec;
use wfd_model::{AttackIdx, StatusType};

use crate::outputs::SourceKind;

/// The anchor (a, i, j) of a draw (08 E-8.16): attack a < 4096, shot index i < 2^20, sub-instance j < 65536.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Anchor {
    /// Attack index a.
    pub a: u16,
    /// Shot index i (reload number for reload-level draws).
    pub i: u32,
    /// Sub-instance index j.
    pub j: u16,
}

impl Anchor {
    /// j of shot-level draws (`"ms_pellets"`).
    pub const SHOT_LEVEL_J: u16 = 0xFFFF;
    /// j of reload-level draws.
    pub const RELOAD_J: u16 = 0xFFFE;
    /// j of per-run draws (with a = 0, i = [`Anchor::RUN_I`]).
    pub const RUN_J: u16 = 0xFFFD;
    /// i of per-run draws.
    pub const RUN_I: u32 = 0xF_FFFF;
}

/// Processing phase at one time τ̂ (03 §7.1; 08 §4.3: "phase P1 = 1, P2 = 2, P3 = 3, P4 = 4").
///
/// P4 is a single heap phase. The steps of one shot S (03 §7.1 P4): (a) its instances in order, with their extra
/// hits, Melee Duplicate copies and same-time children; (b) all procs of S; (c) the zero-delay ticks created in (b);
/// (d) DoT detonation. They run inside S's P4 step, before the next shot's step, and events that (b)-(d) create at
/// τ̂ for a phase ≤ P4 are processed inside that step (03 §7.1, 08 §4.3). They are not heap phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Phase {
    /// P1: state transitions due (expiries, Frozen end, ragdoll end, Heat strip steps, condition expiry, combo and
    /// sniper decay, on/off switches).
    Transitions = 1,
    /// P2: scheduled damage due (DoT tick groups, Heat pulses, Blast fuses, Magnetic-break ticks).
    Damage = 2,
    /// P3: Heat episode end.
    HeatEnd = 3,
    /// P4: one shot (or one delayed child event, 08 §4.2) with its steps a-d.
    Shot = 4,
}

/// The third heap-key component of 08 §4.3, which orders events of one (τ̂, phase).
///
/// - P4: the order key of 08 §4.2, (firing attack index, shot index, child attack index), packed so that the integer
///   order is the lexicographic order; a parent shot (no child component) sorts before its delayed children. Shots
///   inserted lazily from several firing attacks therefore follow 08 §4.2 whatever their creation order.
/// - P2: the creation sequence number of the oldest proc of the tick group (03 §7.1 "creation order of the oldest
///   proc in each group"), i.e. the `seq` of that proc's run-state entry (08 §6.2).
/// - P1, P3: 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SubKey(pub u64);

impl SubKey {
    /// The sub-key of P1 and P3 events.
    pub const NONE: SubKey = SubKey(0);

    /// P4: (firing attack index, shot index, child attack index) of 08 §4.2, packed as `a << 48 | i << 16 | c` with
    /// `c = 0` for the parent shot and `child + 1` for a delayed child event of `child`.
    pub fn shot(firing: AttackIdx, shot: u32, child: Option<AttackIdx>) -> SubKey {
        let c = child.map_or(0, |c| u64::from(c.0) + 1);
        SubKey((u64::from(firing.0) << 48) | (u64::from(shot) << 16) | c)
    }

    /// P2: the creation sequence number of the oldest proc of the tick group.
    pub fn tick_group(oldest_proc_seq: u64) -> SubKey {
        SubKey(oldest_proc_seq)
    }
}

/// One event of a pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct McEvent {
    /// Time (s, f64 as scheduled).
    pub t: f64,
    /// Time on the integer µs grid τ̂ (08 E-8.0a); the heap orders by it. Addition to engine.md §2.1.
    pub t_us: i64,
    /// Phase.
    pub phase: Phase,
    /// Order inside one (τ̂, phase) (08 §4.3). Addition to engine.md §2.1.
    pub sub_key: SubKey,
    /// Creation order: a per-run counter incremented on every event creation (the last tie-break, 08 §4.3).
    pub seq: u64,
    /// What happens.
    pub kind: McEventKind,
    /// The anchor of the instance or tick the event belongs to.
    pub anchor: Anchor,
}

impl McEvent {
    /// The heap key (τ̂, phase, sub_key, seq) of 08 §4.3 and §6.2.
    pub fn key(&self) -> (i64, Phase, SubKey, u64) {
        (self.t_us, self.phase, self.sub_key, self.seq)
    }

    /// Order by the heap key (earliest first).
    pub fn cmp_key(&self, other: &McEvent) -> Ordering {
        self.key().cmp(&other.key())
    }
}

/// Event kinds of E-8.18.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum McEventKind {
    /// A shot (index into `Schedule.shots`), inserted lazily (P4; sub-key [`SubKey::shot`] with no child).
    Shot {
        /// Attack of the shot.
        attack: AttackIdx,
        /// Index of the shot record.
        shot: u32,
    },
    /// The instances of one child attack of one parent shot at a later time than the parent's hit: a delayed
    /// explosion or one lingering tick (06 §9.3-§9.4, E-6.25; 08 §4.2 "a child instance at a later time forms its
    /// own event"). P4; sub-key [`SubKey::shot`] of (firing attack, parent shot index, child attack). Children at the
    /// parent's own hit time are instances of the parent's [`McEventKind::Shot`] (03 §7.1 P4a).
    Child {
        /// The firing attack whose schedule holds the record.
        attack: AttackIdx,
        /// Index into `Schedule.children`.
        entry: u32,
        /// Index into that record's hit times (lingering tick m).
        tick: u16,
    },
    /// A stack or DoT instance expires (presence only; never cancels a tick).
    Expiry {
        /// The status.
        status: StatusType,
        /// The stack or instance id.
        id: u32,
    },
    /// Frozen ends (03 §8.6).
    FrozenEnd,
    /// Ragdoll ends (03 E-3.13).
    RagdollEnd,
    /// Heat strip step (03 §10.4).
    HeatStrip,
    /// A condition stack expires or decays one step (04 E-4.8).
    ConditionExpiry {
        /// Condition-table index.
        cond: u16,
    },
    /// The melee combo timer expires (07 E-7.14).
    ComboExpiry,
    /// A sniper-combo decay step (06 E-6.16).
    SniperDecay,
    /// A DoT tick of an instance (ELE/GAS grouped at processing time).
    DotTick {
        /// The DoT instance id.
        id: u32,
    },
    /// A Heat pulse.
    HeatPulse,
    /// The Heat episode ends.
    HeatEnd,
    /// A Blast fuse expires.
    BlastFuse {
        /// The fuse id.
        id: u32,
    },
    /// A Magnetic-break tick (layered passes).
    MagneticBreakTick {
        /// The break proc id.
        id: u32,
    },
    /// On/off switch of a class (c)/(u) process.
    Switch {
        /// Condition-table index.
        cond: u16,
    },
}

/// What a mechanic's MC side emits for one instance: derived events, damage by source, procs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct McSink {
    /// Events to schedule.
    pub events: SmallVec<[McEvent; 4]>,
    /// Damage dealt now, by source.
    pub damage: SmallVec<[(SourceKind, f64); 4]>,
    /// Procs to apply in step b of the current shot (03 §7.1 P4b), with the anchor of the instance that made them.
    pub procs: SmallVec<[(StatusType, Anchor); 4]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(t_us: i64, phase: Phase, sub_key: SubKey, seq: u64) -> McEvent {
        McEvent {
            t: t_us as f64 / 1e6,
            t_us,
            phase,
            sub_key,
            seq,
            kind: McEventKind::HeatEnd,
            anchor: Anchor::default(),
        }
    }

    #[test]
    fn phases_are_p1_to_p4() {
        // 08 §4.3: P1 = 1, P2 = 2, P3 = 3, P4 = 4; steps a-d of a shot are not heap phases.
        assert_eq!(Phase::Transitions as u8, 1);
        assert_eq!(Phase::Damage as u8, 2);
        assert_eq!(Phase::HeatEnd as u8, 3);
        assert_eq!(Phase::Shot as u8, 4);
        assert!(Phase::Transitions < Phase::Damage && Phase::Damage < Phase::HeatEnd);
        assert!(Phase::HeatEnd < Phase::Shot);
    }

    #[test]
    fn shots_at_one_time_follow_the_4_2_order_not_creation_order() {
        // Attack 1's shot was inserted first (seq 0), attack 0's second (seq 1): 08 §4.3 orders by the 4.2 key.
        let a = AttackIdx(0);
        let b = AttackIdx(1);
        let late = ev(500_000, Phase::Shot, SubKey::shot(b, 0, None), 0);
        let early = ev(500_000, Phase::Shot, SubKey::shot(a, 7, None), 1);
        assert_eq!(early.cmp_key(&late), Ordering::Less);
        // Same firing attack: shot index, then child attack index; a parent shot precedes its own delayed children,
        // a delayed child of an earlier shot precedes a later shot.
        assert!(SubKey::shot(a, 3, None) < SubKey::shot(a, 3, Some(AttackIdx(0))));
        assert!(SubKey::shot(a, 3, Some(AttackIdx(2))) < SubKey::shot(a, 3, Some(AttackIdx(5))));
        assert!(SubKey::shot(a, 2, Some(AttackIdx(9))) < SubKey::shot(a, 3, None));
        assert!(SubKey::shot(a, u32::MAX, Some(AttackIdx(255))) < SubKey::shot(b, 0, None));
    }

    #[test]
    fn key_is_time_then_phase_then_sub_key_then_seq() {
        let shot = ev(
            1_000_000,
            Phase::Shot,
            SubKey::shot(AttackIdx(0), 0, None),
            0,
        );
        let tick = ev(1_000_000, Phase::Damage, SubKey::tick_group(40), 9);
        let expiry = ev(1_000_000, Phase::Transitions, SubKey::NONE, 12);
        let earlier_shot = ev(
            999_999,
            Phase::Shot,
            SubKey::shot(AttackIdx(3), 0, None),
            30,
        );
        let mut v = [shot, tick, expiry, earlier_shot];
        v.sort_by(McEvent::cmp_key);
        assert_eq!(v, [earlier_shot, expiry, tick, shot]);
        // P2 tick groups: the group whose oldest proc was created first lands first, whatever the event seq.
        let g_old = ev(2_000_000, Phase::Damage, SubKey::tick_group(3), 50);
        let g_new = ev(2_000_000, Phase::Damage, SubKey::tick_group(8), 20);
        assert_eq!(g_old.cmp_key(&g_new), Ordering::Less);
        // The event seq breaks the last tie.
        let x = ev(2_000_000, Phase::Transitions, SubKey::NONE, 1);
        let y = ev(2_000_000, Phase::Transitions, SubKey::NONE, 2);
        assert_eq!(x.cmp_key(&y), Ordering::Less);
    }
}
