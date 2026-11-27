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
use wfd_model::{AttackIdx, N_ST};

use crate::compile::MeleePlan;
use crate::error::EngineError;
use crate::resolve::ResolvedAttack;

/// The largest schedule: `EngineError::ScheduleTooLong` beyond 10^7 events (horizon × rate, engine.md §15).
pub const MAX_SCHEDULE_EVENTS: u64 = 10_000_000;

/// engine.md §15: the number of events the schedule of `ra` (with `plan` for a melee attack) up to `horizon` can
/// hold, an upper bound from the resolved rates (06 E-6.11: shots × (1 + children); 07 E-7.30: strikes per loop over
/// T_loop), or `EngineError::ScheduleTooLong { shots }` when it exceeds [`MAX_SCHEDULE_EVENTS`]. An infinite rate
/// (`TimingBase.fr0 = ∞` with no charge, burst delay or other spacing: 1/r = 0 in E-6.11) is too long, so a schedule
/// generator never loops forever and never truncates silently. `Engine::try_schedule` calls it before scheduling.
/// Addition, contract review ENG-16. T08a (ranged), T08b (melee).
pub fn event_bound(
    ra: &ResolvedAttack,
    plan: Option<&MeleePlan>,
    horizon: f64,
) -> Result<u64, EngineError> {
    let _ = (ra, plan, horizon);
    todo!("T08a, T08b")
}

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
    /// A slam or heavy slam (07 §7.5 `slam`: Slam Damage, CO `none`, Melee Duplicate triggers). Addition, contract
    /// review ENG-3.
    pub const SLAM: ShotFlags = ShotFlags(1 << 7);
    /// A radial instance (07 §7.5 `radial`: the radial Part branch of 00 E-7, 02 E-2.10; rad_i' = 0).
    pub const RADIAL: ShotFlags = ShotFlags(1 << 8);
    /// A melee hit (07 §7.5 `melee_hit`: every melee instance except DoT ticks and extra hits; `on_melee_hit`).
    pub const MELEE_HIT: ShotFlags = ShotFlags(1 << 9);
    /// Stealth-eligible (07 §7.5 `stealth_ok`: T-08, 04 E-4.16; OQ-733).
    pub const STEALTH_OK: ShotFlags = ShotFlags(1 << 10);
    /// True-melee contact (07 §7.5 `melee_contact`: starts the 3 s stealth cooldown of 04 E-4.16).
    pub const MELEE_CONTACT: ShotFlags = ShotFlags(1 << 11);
    /// Reads n_anim without consuming it (07 §7.5 `anim_read`; OQ-740).
    pub const ANIM_READ: ShotFlags = ShotFlags(1 << 12);
    /// The first strike of an input (07 §7.5 `input_start`: Melee Animosity stacking, Tennokai qualifying events).
    pub const INPUT_START: ShotFlags = ShotFlags(1 << 13);

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
    /// Melee: the strike's damage multiplier S_j (light and slide strikes, light and slide waves: 07 E-7.20) or H_j
    /// (heavy strikes and heavy waves: E-7.20b; s_j of a gunblade shot, E-7.45); 1 for ranged shots.
    pub strike_mult: f64,
    /// Melee: the expected combo count c̄⁻ before the hit (07 §6.9); 0 for ranged shots.
    pub combo: f64,
    /// Expected weight of the hit (Volatile explosions: p_vol; 1 otherwise; 07 §8.2).
    pub weight: f64,
    /// The rest of the melee hit record of 07 §8.2; `None` for ranged shots. Addition, contract review ENG-3.
    pub melee: Option<MeleeHit>,
}

/// The instance kinds of 07 §7.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeleeKind {
    /// A strike of a Neutral, Forward, Block or Forward Block loop.
    #[default]
    Light,
    /// A strike of the shared Slide combo.
    Slide,
    /// A strike of the shared Aerial combo.
    Aerial,
    /// A strike of the shared Wall combo.
    Wall,
    /// A strike of a synthesized heavy attack (07 §9.1).
    Heavy,
    /// A `Ranged` stance strike: gunblade shot, wave, mapped weapon-unique attack (07 §9.5, §9.7, §9.8).
    Ranged,
    /// The `Slam Attack` data attack (07 §9.2).
    Slam,
    /// The `Heavy Slam Attack` data attack (07 §9.2).
    HeavySlam,
    /// A thrown-weapon hit (07 §9.4).
    Thrown,
    /// A thrown weapon's explosion (07 §9.4).
    Explosion,
    /// A `cycle_loop` strike (07 §8.1).
    Cycle,
    /// A `data_attack_loop` instance (07 §8.1).
    Data,
    /// A `Slam`-typed stance entry: no damage in the single-target metric (07 §7.1, OQ-710).
    SlamPart,
}

/// The melee fields of the hit record of 07 §8.2 that `ShotRec` does not carry (contract review ENG-3). `ShotRec`
/// holds the time, the attack, S_j or H_j (`strike_mult`), c̄⁻ (`combo`), the expected weight and the 07 §7.5 flags.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeleeHit {
    /// The instance kind (07 §7.1).
    pub kind: MeleeKind,
    /// Input index u of the combo.
    pub input: u8,
    /// Strike index e within the combo.
    pub strike: u8,
    /// S^phys_{j,t}: the strike's stance physical multipliers for IMP, PUN, SLA (01 E-1.6, EC-01-34; 07 §7.2), from
    /// which the composition is recomputed when they differ from the attack's (`resolve::compose`).
    pub s_phys: [f64; 3],
    /// The strike's forced procs per status (07 §7.7: its normalised proc list; heavies: E-7.40), counted by 02
    /// E-2.14 per instance.
    pub forced: [f64; N_ST],
    /// r_in, the stance CO ratio of the instance (07 E-7.21; 04 E-4.6).
    pub r_in: f64,
    /// Heavy index h ∈ {1, 2}; 0 when not a heavy strike.
    pub heavy: u8,
    /// M(c̄⁻), the combo multiplier the hit uses (07 E-7.10; M_h for heavies, E-7.16; T-10 per E-7.31).
    pub m: f64,
    /// M_b(c̄⁻), the combo multiplier of the combo-scaled bonuses (07 E-7.11, E-7.18).
    pub m_b: f64,
    /// g, the base combo gain of the instance (07 E-7.13); MC tracks the live count with `combo_acc` and `combo_rc`
    /// from it (07 §6.9).
    pub gain: f64,
    /// The expected extra gain (p_acc and Relentless Combination, 07 E-7.13, E-7.13b).
    pub gain_extra: f64,
    /// n_anim: Melee Animosity stacks carried (heavies and `anim_read` instances; 07 §7.5, 04 §5.4).
    pub n_anim: u8,
    /// n_thr: index of a throw in the current run of consecutive throws (Power Throw, 04 §3.4; `glaive_throw_index`
    /// of 07 §3); 0 otherwise.
    pub n_thr: u8,
    /// The gunblade pellet rule of 07 E-7.45: the shot fires floor(MS) + Bernoulli(frac(MS)) pellets of the Ranged
    /// Attack, each its own instance with g = 1 per pellet.
    pub gunblade_pellets: bool,
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
