//! Run state of one pass (08 §6.2; engine.md §8.1): the target state (per-status stack entries ordered by
//! application, independent timers and replacement beyond the cap, CD-07, 03 §6.2; DoT instances with snapshotted
//! seeds, CD-05; the shared Heat episode, pool and strip counter; Blast fuses; Frozen, ragdoll and the single-instance
//! statuses; external stacks; pools per layer with the shield gate, adaptation and health-threshold flags; the
//! permanent flat armor strip) and the attacker state (schedule cursors, melee combo, sniper combo, condition-effect
//! states of 04 E-4.8, on/off processes of class (c)/(u) effects, `counter_bonus` counters). Mechanics read and change
//! it through [`crate::mechanics::MechanicMc`]. The accumulators and the event heap of 08 §6.2 belong to the
//! simulator itself (`mc/sim.rs`): no mechanic reads them. Every time is on the µs grid of 08 E-8.0a. Owner: T16.

use smallvec::SmallVec;
use wfd_model::{AttackIdx, N_ST, StatusType};

use crate::enemy::Pools;
use crate::resolve::N_MECH;
use crate::status::ProcSnapshot;

/// The state of one pass of one run (08 §6.2). Reused across runs (clear, keep capacity).
#[derive(Debug, Clone, Default)]
pub struct McState {
    /// Current time τ̂ (µs grid, 08 E-8.0a).
    pub t_us: i64,
    /// `og_present` of this status timeline in the view pass (03 E-3.2): true in the `overguard` view's timeline on
    /// an enemy that spawns with overguard, false in the other views' timeline; the layered passes read the live OG
    /// pool instead (08 §4.5, §6.2).
    pub og_view: bool,

    // ---- target ----
    /// Own live entries per status (index `StatusType::index`), ordered by application `seq` (08 §6.2, 03 §6.2): the
    /// first entry is the oldest live stack (smallest application time, ties by application order), the one 03 §6.2
    /// replaces at the cap. DoT statuses (SLA, TOX, ELE, GAS) list their live instances here as well, with
    /// [`StackEntry::dot`] pointing into [`McState::dots`]; HEA and BLA use [`McState::heat`] and
    /// [`McState::blast_fuses`]; VOI, KND, LIF and MIC use [`McState::single`]. A `Vec`, because the base cap of SLA,
    /// ELE and TOX is ∞ (03 §4.1) and COR's is 10 + β_COR: an uncapped status grows without bound and never panics.
    pub stacks: [Vec<StackEntry>; N_ST],
    /// External stacks N_ext,s of T-43 / T-06 (03 §6.3): never expire, never replaced, no damage, no triggers.
    pub external: [u8; N_ST],
    /// DoT instances (slab; `None` = free slot), referenced by [`StackEntry::dot`] and by `DotTick` events.
    pub dots: Vec<Option<DotInst>>,
    /// Heat episode, pool and strip counter (03 §10).
    pub heat: HeatState,
    /// Live Blast fuses in application order (03 §11).
    pub blast_fuses: Vec<BlastFuse>,
    /// Frozen until (µs grid), when frozen (03 §8.6).
    pub frozen_until: Option<i64>,
    /// Ragdoll state RAG until (µs grid), when ragdolled (03 E-3.13, §8.10).
    pub ragdoll_until: Option<i64>,
    /// The single-instance statuses VOI, KND, LIF, MIC (03 §8.7-§8.9), indexed by [`SingleStatus`].
    pub single: [Option<SingleState>; SingleStatus::N],
    /// Flat armor strip s_flat (05 E-5.11): permanent, never decays.
    pub armor_strip_flat: f64,
    /// Pools of the layered passes: OG, SH, HP, shield gate, adaptation, health-threshold flags (05 E-5.23, E-5.25).
    pub pools: Pools,

    // ---- attacker ----
    /// Schedule cursor per firing attack of the unit: index of the next shot record to insert (shots are inserted
    /// lazily, 08 E-8.18). The magazine state is read from the schedule records (08 §6.2).
    pub cursor: SmallVec<[u32; 3]>,
    /// Melee combo counter and its timer (07 §6).
    pub combo: ComboState,
    /// Sniper combo counter c_snp (06 E-6.16).
    pub sniper_combo: u32,
    /// Condition-effect states of 04 E-4.8 (stacks, timers, use and reset counters), per condition-table index.
    pub conditions: Vec<CondState>,
    /// On/off processes of class (c)/(u) effects with u_e < 1 used nonlinearly (A-12; 08 E-8.18 start).
    pub switches: Vec<SwitchState>,
    /// `counter_bonus` counters per `mech.<n>` slot.
    pub counters: [f64; N_MECH],
}

/// One live own stack or DoT instance of a status (08 §6.2: `(t_apply, t_expire, source attack, seq)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackEntry {
    /// Application time t̂_a (µs grid).
    pub t_apply_us: i64,
    /// Expiry t̂_a + D̂_s (µs grid; 03 §7.2: present on `[t̂_a, t̂_a + D̂_s)`).
    pub t_expire_us: i64,
    /// The attack whose proc applied it.
    pub source: AttackIdx,
    /// Application sequence number (per run, increasing): the replacement order of 03 §6.2.
    pub seq: u64,
    /// The DoT instance of SLA, TOX, ELE, GAS (index into [`McState::dots`]); `None` for the other statuses.
    pub dot: Option<u32>,
}

/// One DoT instance (08 §6.2: `(seq, s, S_p incl. c_k and Part_dot, X_p, N, next tick m, cancelled flag)`; E-13).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DotInst {
    /// Its status.
    pub status: StatusType,
    /// Creation sequence number (the order of 03 §7.1 P2 and E-3.5's oldest proc p₀).
    pub seq: u64,
    /// Snapshotted weapon-side terms (CD-05).
    pub snap: ProcSnapshot,
    /// Next tick index m (tick m lands at t̂_p + (δ_s + m)·10^6, 03 §7).
    pub next_tick: u32,
    /// Replaced (03 §6.2) or detonated (03 §9.7): its ticks not yet landed are cancelled. Expiry never sets it.
    pub cancelled: bool,
    /// A Magnetic-break instance (03 §13): forms its own tick group (03 E-3.5).
    pub magnetic_break: bool,
}

/// The shared Heat episode (03 §10; 08 §6.2 `t_ign, pool, t_exp, next pulse; strip counter k_h, pending steps`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeatState {
    /// The episode is live.
    pub live: bool,
    /// Ignition time t̂_ign (µs grid); pulse m lands at t̂_ign + m·10^6.
    pub t_ign: i64,
    /// Episode end t̂_exp (µs grid).
    pub t_exp: i64,
    /// Next pulse index m.
    pub next_pulse: u32,
    /// Procs in the pool, in application order (the oldest leaves first beyond cap_HEA, 03 §10.2).
    pub pool: Vec<HeatProc>,
    /// Strip counter k_h (0 … 4, 03 §10.4); outlives the episode.
    pub strip: u8,
    /// Pending strip steps: (grid time, +1 up or −1 down), in time order (03 E-3.8).
    pub strip_steps: SmallVec<[(i64, i8); 4]>,
}

/// One Heat proc in the pool (08 §6.2 `(seq, S_q·X_q, D_q, SD_q)`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatProc {
    /// Application sequence number.
    pub seq: u64,
    /// Snapshotted weapon-side terms.
    pub snap: ProcSnapshot,
    /// Its duration D̂_q (µs).
    pub dur_us: i64,
    /// Its status duration SD_q (the down-step ramp of 03 §10.4 reads SD* of the pool).
    pub sd: f64,
}

/// One live Blast fuse (08 §6.2 `(seq, H_p, expiry)`; 03 §11).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlastFuse {
    /// Application sequence number.
    pub seq: u64,
    /// Snapshotted host-hit terms H_p.
    pub snap: ProcSnapshot,
    /// Fuse expiry (µs grid).
    pub t_expire_us: i64,
    /// The attack whose proc applied it.
    pub source: AttackIdx,
}

/// The single-instance statuses (03 §8.7-§8.9): the index of [`McState::single`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SingleStatus {
    /// Void (Bullet Attractor): a new proc restarts the timer.
    Voi = 0,
    /// Knockdown: a re-proc restarts; removes LIF.
    Knd = 1,
    /// Lifted: a LIF proc on a Lifted target ends Lifted and ragdolls (03 §8.10); removes KND.
    Lif = 2,
    /// Microwave: permanent once applied.
    Mic = 3,
}

impl SingleStatus {
    /// Number of single-instance statuses.
    pub const N: usize = 4;
}

/// A live single-instance status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleState {
    /// Application time (µs grid).
    pub t_apply_us: i64,
    /// End (µs grid); `i64::MAX` for MIC (permanent).
    pub until_us: i64,
    /// Application sequence number.
    pub seq: u64,
}

/// Melee combo counter and timer (07 §6, E-7.12-E-7.17).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ComboState {
    /// Combo count.
    pub count: f64,
    /// Expiry of the combo timer (µs grid); `None` while no timer runs.
    pub expires_us: Option<i64>,
}

/// The state of one conditional effect (04 E-4.8): stacks with their timers, uses and resets.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CondState {
    /// Live stacks: expiry per stack (µs grid; `i64::MAX` without duration), in application order.
    pub stacks: SmallVec<[i64; 4]>,
    /// Next step of an `idle` decay (µs grid), when one is pending.
    pub decay_at: Option<i64>,
    /// Uses consumed (`consume_on`).
    pub uses: u32,
    /// Resets seen (`reset_on`).
    pub resets: u32,
}

/// An on/off process of a class (c)/(u) effect with u_e < 1 (A-12; 08 E-8.18): on during
/// `[φ + k·T_period, φ + k·T_period + u_e·T_period)`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SwitchState {
    /// The effect is on now.
    pub on: bool,
    /// Phase φ (µs grid), drawn per run.
    pub phase_us: i64,
    /// Period T_period (µs).
    pub period_us: i64,
    /// On-time u_e·T_period (µs).
    pub on_us: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncapped_statuses_hold_any_number_of_entries() {
        // 03 §4.1: SLA, ELE, TOX have cap ∞; COR 10 + β_COR. The store grows; it never panics at a fixed size.
        let mut st = McState::default();
        let sla = StatusType::SLA.index();
        for k in 0..100u32 {
            st.stacks[sla].push(StackEntry {
                t_apply_us: i64::from(k),
                t_expire_us: i64::from(k) + 6_000_000,
                source: AttackIdx(0),
                seq: u64::from(k),
                dot: Some(k),
            });
        }
        assert_eq!(st.stacks[sla].len(), 100);
        assert_eq!(st.stacks[sla][0].seq, 0, "oldest first");
    }

    #[test]
    fn oldest_by_application_is_not_first_to_expire() {
        // 03 §6.2: replacement removes the smallest application time, even when a later stack expires sooner
        // (different SD, a Cold re-application after Frozen). The entries keep both times.
        let older_long = StackEntry {
            t_apply_us: 0,
            t_expire_us: 12_000_000,
            source: AttackIdx(0),
            seq: 0,
            dot: None,
        };
        let newer_short = StackEntry {
            t_apply_us: 1_000_000,
            t_expire_us: 7_000_000,
            source: AttackIdx(1),
            seq: 1,
            dot: None,
        };
        let v = [older_long, newer_short];
        let oldest = v
            .iter()
            .min_by_key(|e| (e.t_apply_us, e.seq))
            .expect("two entries");
        assert_eq!(oldest.seq, 0);
        let first_to_expire = v.iter().min_by_key(|e| e.t_expire_us).expect("two entries");
        assert_eq!(first_to_expire.seq, 1);
    }
}
