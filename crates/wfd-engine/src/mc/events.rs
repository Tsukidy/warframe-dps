//! MC events (08 §6.3, E-8.18; engine.md §8.1): a binary heap keyed `(τ̂, phase, seq)` (08 §4.3, E-8.0a) with the
//! phases P1-P4 of 03 §7.1, P4 split into (a) instances in order, (b) all procs of the shot, (c) zero-delay ticks,
//! (d) DoT detonation; and the anchors of 08 E-8.16. Owner: T16.

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

/// Processing phase at one time (03 §7.1; 08 E-8.18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// P1: state transitions due (expiries, Frozen end, ragdoll end, Heat strip steps, condition expiry, switches).
    Transitions,
    /// P2: scheduled damage due (DoT tick groups, Heat pulses, Blast fuses, Magnetic-break ticks).
    Damage,
    /// P3: Heat episode end.
    HeatEnd,
    /// P4a: the shot's instances in order.
    Instances,
    /// P4b: all procs of the shot.
    Procs,
    /// P4c: zero-delay ticks.
    ZeroDelayTicks,
    /// P4d: DoT detonation (03 §9.7).
    Detonation,
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
    /// Creation order.
    pub seq: u64,
    /// What happens.
    pub kind: McEventKind,
    /// The anchor of the instance or tick the event belongs to.
    pub anchor: Anchor,
}

impl McEvent {
    /// The heap key (τ̂, phase, seq).
    pub fn key(&self) -> (i64, Phase, u64) {
        (self.t_us, self.phase, self.seq)
    }

    /// Order by the heap key (earliest first).
    pub fn cmp_key(&self, other: &McEvent) -> Ordering {
        self.key().cmp(&other.key())
    }
}

/// Event kinds of E-8.18.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum McEventKind {
    /// A shot (index into `Schedule.shots`), inserted lazily.
    Shot {
        /// Attack of the shot.
        attack: AttackIdx,
        /// Index of the shot record.
        shot: u32,
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
    /// Procs to apply in P4b, with the anchor of the instance that made them.
    pub procs: SmallVec<[(StatusType, Anchor); 4]>,
}
