//! Deterministic schedules (engine.md §5): the shot, reload and child-hit records of 06 §6.1 (E-6.11) and the hit,
//! heavy and combo records of 07 §8.2 (E-7.30), identical for EV and MC; the schedule never depends on random draws.
//! [`Segment`]s are the run-length form the `WindowFast` and `Bound` fidelities walk. Owner: T08a; `melee.rs` T08b.

pub mod melee;
pub mod ranged;
pub mod segment;

pub use melee::{ComboRec, HeavyRec, MeleeTrack, melee_schedule};
pub use ranged::ranged_schedule;
pub use segment::Segment;

use smallvec::SmallVec;
use wfd_model::AttackIdx;

/// The schedule of one firing attack (with its children) or one melee plan up to the horizon (engine.md §5).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Schedule {
    /// Shot records (melee: one per hit) in time order.
    pub shots: Vec<ShotRec>,
    /// Reload records in time order.
    pub reloads: Vec<ReloadRec>,
    /// Maximal runs of shots with constant spacing and equal fields other than time.
    pub segments: SmallVec<[Segment; 16]>,
    /// Child hits per shot and child attack (06 §9.3-§9.4, E-6.24, E-6.25).
    pub children: Vec<ChildHits>,
    /// Sniper-combo decay times (06 §7.2, E-6.16).
    pub sniper_decay: Vec<f64>,
    /// Heavy and combo records of 07 §8.2; `None` for ranged attacks.
    pub melee: Option<MeleeTrack>,
    /// Steady quantities of the cycle.
    pub cycle: CycleStats,
}

/// Flags of a shot record (06 §6.1, E-6.13; 07 §7.5 hit flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ShotFlags(pub u16);

impl ShotFlags {
    /// First shot of a magazine by the HUD counter (06 E-6.13).
    pub const FIRST: ShotFlags = ShotFlags(1);
    /// The shot ends its magazine (06 E-6.13).
    pub const LAST: ShotFlags = ShotFlags(1 << 1);
    /// First shot after a reload (06 §6.1).
    pub const AFTER_RELOAD: ShotFlags = ShotFlags(1 << 2);
    /// DUPLEX release shot (06 E-6.12).
    pub const RELEASE: ShotFlags = ShotFlags(1 << 3);
    /// A heavy-attack hit (07 §7.1).
    pub const HEAVY: ShotFlags = ShotFlags(1 << 4);
    /// A slide-attack hit (07 §7.5).
    pub const SLIDE: ShotFlags = ShotFlags(1 << 5);
    /// A Tennokai heavy (07 §6.7).
    pub const TENNOKAI: ShotFlags = ShotFlags(1 << 6);

    /// Whether every flag of `other` is set.
    pub fn contains(self, other: ShotFlags) -> bool {
        self.0 & other.0 == other.0
    }

    /// The union.
    pub fn union(self, other: ShotFlags) -> ShotFlags {
        ShotFlags(self.0 | other.0)
    }
}

/// One shot (06 §6.1) or one melee hit (07 §8.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShotRec {
    /// The attack fired.
    pub attack: AttackIdx,
    /// Shot index i (0-based, the RNG anchor's i, 08 E-8.16).
    pub index: u32,
    /// Cycle start τ_i (charge start or fire time).
    pub cycle_start: f64,
    /// Fire time s_i.
    pub fire: f64,
    /// Hit time h_i (06 §8.5; flight time at T-27 distance).
    pub hit: f64,
    /// Magazine number (1-based).
    pub mag: u32,
    /// Index within the magazine (0-based).
    pub mag_index: u32,
    /// Ammo before the shot.
    pub ammo_before: f64,
    /// Ammo after the shot.
    pub ammo_after: f64,
    /// Cost c_i.
    pub cost: f64,
    /// Flags.
    pub flags: ShotFlags,
    /// Burst number (or DUPLEX sequence number).
    pub burst: u32,
    /// Position in the burst (DUPLEX: 0 press, 1 release).
    pub burst_pos: u16,
    /// Spool index k (06 E-6.17).
    pub spool_index: u32,
    /// Rate after the shot r_k (shots/s).
    pub rate: f64,
    /// Kohm base pellets n_k (06 E-6.18; 1 otherwise).
    pub pellets: f64,
    /// Peacemaker burst b_pm (06 E-6.19; 0 otherwise).
    pub peacemaker_burst: u32,
    /// Beam ramp level L_i (06 §7.1; 1 when not ramping).
    pub ramp: f64,
    /// Melee: the strike's damage multiplier S_j or heavy H_j (07 E-7.20, E-7.20b); 1 for ranged shots.
    pub strike_mult: f64,
    /// Melee: the expected combo count c̄⁻ before the hit (07 §6.9); 0 for ranged shots.
    pub combo: f64,
    /// Expected weight of the hit (Volatile explosions: p_vol; 1 otherwise; 07 §8.2).
    pub weight: f64,
}

/// One reload (06 §6.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReloadRec {
    /// The attack whose magazine reloads.
    pub attack: AttackIdx,
    /// Reload number (1-based; the RNG anchor's i of reload-level draws, 08 E-8.16).
    pub index: u32,
    /// Start (s).
    pub start: f64,
    /// End (s).
    pub end: f64,
    /// a_left ≤ 1e-9 (HUD counter 0).
    pub from_empty: bool,
    /// Ammo before the reload.
    pub ammo_before: f64,
    /// Reloaded fraction (Mag − left)/Mag (`Per::ReloadedFraction`).
    pub reloaded_fraction: f64,
}

/// Child hits of one parent shot for one child attack (06 §9.2-§9.4, E-6.23-E-6.25).
#[derive(Debug, Clone, PartialEq)]
pub struct ChildHits {
    /// Index of the parent shot in `Schedule.shots`.
    pub parent: u32,
    /// The child attack.
    pub attack: AttackIdx,
    /// n_child per parent instance (E-6.24), before the T-41 fraction.
    pub count: f64,
    /// Headshot-only children (exist only for parent instances that hit a headshot part).
    pub headshot_gate: bool,
    /// Hit times (delayed explosions, lingering ticks).
    pub times: SmallVec<[f64; 4]>,
}

/// Steady quantities of a schedule (06 §10, 07 E-7.37).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CycleStats {
    /// Effective fire rate (shots/s; melee: hits/s).
    pub efr: f64,
    /// Shots per magazine.
    pub shots_per_mag: f64,
    /// Duty ρ = T_mag / (T_mag + T_rel).
    pub duty: f64,
    /// T_mag (s).
    pub t_mag: f64,
    /// T_rel (s).
    pub t_rel: f64,
    /// Melee loop time T_loop (s; 0 for ranged attacks).
    pub t_loop: f64,
}
