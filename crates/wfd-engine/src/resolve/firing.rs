//! Firing stats (06 §5, §10): fire rate and its locks, charge time, bursts, magazine, ammo cost and efficiency, reload,
//! duty, beam ramp, spool, gauge-gated attacks (E-6.14a with T-42), effective fire rate; and the
//! [`ScheduleInputs`], every other per-attack input of the ranged schedule generator (06 E-6.11 and the variants it
//! names), because `ranged_schedule(ctx, ra, horizon)` reads no bundle. Owner: T08a.

use wfd_model::{Attack, AttackIdx, Trigger};

use crate::StageCtx;
use crate::assemble::AssembledWeapon;
use crate::stats::StatVecF;

/// Largest number of child links of one firing attack (06 §9.1-§9.4). The longest chains of the data have three
/// links (Kuva Bramma, Zymos), a volley member one; a firing attack with more is a data error that the data build
/// reports (06 E-6.23 data-build check).
pub const MAX_CHILD_LINKS: usize = 8;

/// Largest number of members of one volley group (06 E-6.14b; the largest group of the data has six, Arbucep's
/// `1st Attack` … `6th Attack`); a group with more is a data error that the data build reports.
pub const MAX_VOLLEY_MEMBERS: usize = 8;

/// Largest number of own-event timing buffs of 06 §5.9 on one attack (Spring-Loaded Chamber family, Ready
/// Retaliation, …); a build with more is rejected at L2 (`compile_build`: `EngineError::UnsupportedMechanic` naming
/// the extra effect).
pub const MAX_TIMING_BUFFS: usize = 8;

/// Firing stats of one attack (06 §5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiringResolved {
    /// Trigger class (06 §4.2).
    pub trigger: Trigger,
    /// FR after mods and locks (06 §5.1, E-6.2).
    pub fire_rate: f64,
    /// FR_trig after the semi-auto cap (06 E-6.3). Addition to engine.md §4 (CONTRACTS.md T02a).
    pub fire_rate_trig: f64,
    /// Effective fire rate EFR (06 §10).
    pub efr: f64,
    /// Charge time CT (06 §5.3); 0 for attacks without charge.
    pub charge_time: f64,
    /// Shots per burst (1 without bursts).
    pub burst_count: u16,
    /// Delay between the shots of a burst (s).
    pub burst_delay: f64,
    /// Magazine N_mag (06 §5.5); `f64::INFINITY` without a magazine.
    pub magazine: f64,
    /// Ammo cost per shot (06 §5.6).
    pub ammo_cost: f64,
    /// Ammo efficiency AE (clamped, 06 §5.6).
    pub ammo_eff: f64,
    /// Reload time T_rel (06 §5.7) of a reload from empty with the static stats; the schedule recomputes it per
    /// reload from [`ScheduleInputs::reload`] (the by_round and regenerate styles depend on a_left, E-6.9).
    pub reload_time: f64,
    /// Magazine time T_mag (06 §10).
    pub t_mag: f64,
    /// Duty ρ = T_mag / (T_mag + T_rel) (06 §10).
    pub duty: f64,
    /// Beam ramp time (s; 0 = none).
    pub ramp_time: f64,
    /// Spool shots (0 = none, 06 §5.2).
    pub spool_shots: u32,
    /// Gauge-gated attack (06 §6.7, E-6.14a: no magazine cost, no steady metrics).
    pub gauge_gated: bool,
    /// T-42 gauge uses per minute u (set at L3 from the scenario; 0 = the first use only).
    pub gauge_uses_per_min: f64,
    /// Every other input of the ranged schedule generator. Addition to engine.md §4-§5 (CONTRACTS.md T02a).
    pub sched: ScheduleInputs,
}

/// The per-attack inputs of 06 E-6.11 and of the variants it names (06 §5.5-§5.9, §6.4-§6.8, §7, §8.4-§8.5, §9),
/// besides the resolved values of [`FiringResolved`]: `ranged_schedule(ctx, ra, horizon)` receives no bundle, so
/// everything it reads is here (fixed size, `Copy`).
///
/// Who fills what: [`firing_stats`] (T08a) fills the fields that come from the attack and weapon data and the stat
/// vector; L3 (`Engine::resolve`, T13) then sets the fields that come from the scenario's toggles (marked "L3, T-xx",
/// like `FiringResolved::gauge_uses_per_min`) and from the compiled build ([`ScheduleInputs::timing_buffs`]). The
/// roles in `ResolvedAttack.roles` mark the variants (`WHOLE_MAGAZINE`, `RETURNING`, `RELOAD_TRIGGERED`,
/// `GAUGE_GATED`, `INCARNON_FORM`), and `ResolvedAttack.hit_fraction` carries T-41.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScheduleInputs {
    /// Unmodded timing data and the static bucket sums, from which the generator re-evaluates FR, FR_trig, CT, BD,
    /// Mag, AE, c and T_rel at the points of 06 §5.9 while a timing buff is on.
    pub timing: TimingBase,
    /// 06 §5.9 own-event buffs of timing stats under `simulated` (on reload, on reload from empty, first shot, last
    /// shot), `timing_buffs[..n_timing_buffs]`, in canonical (item, effect) order. Filled at L3 from the compiled
    /// build's class (b) coefficients on timing slots; empty under `all_max` and `none` (their values are then in the
    /// static sums) and as returned by [`firing_stats`].
    pub timing_buffs: [TimingBuff; MAX_TIMING_BUFFS],
    /// Number of live entries of `timing_buffs`.
    pub n_timing_buffs: u8,
    /// Expected cost factor of an infinite-ammo state (06 §5.6, §5.9): 0 while the state is always on (exalted
    /// infinite weapons, Epitaph, a buff on under `all_max`), `1 − u` for a random or external trigger with uptime
    /// u (the first and last flags then follow the HUD counter of that expected cost, E-6.13), 1 otherwise.
    pub cost_factor: f64,
    /// Reload model (06 E-6.9) and reload lead (06 E-6.10).
    pub reload: ReloadInputs,
    /// Incarnon form (06 E-6.14); `None` for every other attack.
    pub form: Option<FormInputs>,
    /// Returning projectile (06 §6.4, role `RETURNING`): retrieval time T_ret, s_{i+1} = s_i + T_ret, no cost, no
    /// charge, no 1/FR lead; `None` otherwise.
    pub t_ret: Option<f64>,
    /// Volley group (06 E-6.14b); `None` outside a group.
    pub volley: Option<VolleyInputs>,
    /// Reload-triggered attacks fired at each reload start of this attack's schedule (06 §9.5).
    pub on_reload: ReloadTriggered,
    /// Beam ramp start s_rmp (06 E-6.15; 0.2 default, per-weapon values); the ramp time is
    /// [`FiringResolved::ramp_time`].
    pub ramp_start: f64,
    /// Auto-spool (06 E-6.17); `None` without a spool. The spool shot count is [`FiringResolved::spool_shots`].
    pub spool: Option<SpoolInputs>,
    /// Kohm family pellet cap P_max (06 E-6.18; 12 or 5): n_k = min(P_max, 1 + k), c_k = max(1, n_k/3)·(1 − AE);
    /// 0 for every other attack.
    pub kohm_max_pellets: u8,
    /// Peacemaker (06 E-6.19); `None` otherwise.
    pub peacemaker: Option<PeacemakerInputs>,
    /// Sniper combo (06 E-6.16, E-6.16a); `None` for weapons without `SniperComboMin`.
    pub sniper: Option<SniperInputs>,
    /// Flight time h_i − s_i = d / v_proj with v_proj = ShotSpeed·(1 + ΣPS) at the T-27 distance d (06 E-6.22); 0
    /// for hitscan attacks, beams and d = 0. L3, T-27.
    pub flight_time: f64,
    /// A hitscan or beam attack beyond its `Range` at the T-27 distance (06 §8.4): still scheduled and spending ammo,
    /// no damage instance. L3, T-27.
    pub out_of_range: bool,
    /// Child links (06 E-6.23-E-6.25), `children[..n_children]`, every link after its parent link. Zoom-only
    /// children whose zoom level is not active (T-38) are left out at L3.
    pub children: [ChildLink; MAX_CHILD_LINKS],
    /// Number of live entries of `children`.
    pub n_children: u8,
}

/// Unmodded timing data and static bucket sums (06 E-6.1-E-6.9), the inputs of the re-evaluation at the points of
/// 06 §5.9.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimingBase {
    /// FR0, the data `FireRate` (`f64::INFINITY` for `math.huge`).
    pub fr0: f64,
    /// ΠM_fr, the static multiplicative fire-rate factors (06 §3); 1 under the lock.
    pub fire_mult: f64,
    /// FR_cap of SEMI (incl. Active) and DUPLEX (06 E-6.3); `f64::INFINITY` for the other classes.
    pub semi_cap: f64,
    /// Cannonade lock (06 E-6.1): ΣFR := 0 and ΠM_fr := 1, buff deltas included; CT = CT0, BD = BD0.
    pub fire_locked: bool,
    /// CT0 (s; 0 without charge).
    pub ct0: f64,
    /// Charge time not affected by mods (Phantasma, 06 E-6.4).
    pub charge_fixed: bool,
    /// BD0 (s).
    pub bd0: f64,
    /// Mag0 (rounds; kitguns: the assembled chamber value).
    pub mag0: f64,
    /// Magazine bonuses ignored: bows and Epitaph (Mag = 1) and Incarnon forms (Mag = Mag_inc), 06 E-6.6.
    pub mag_fixed: bool,
    /// cost0, the data `AmmoCost` (default 1).
    pub cost0: f64,
    /// The static bucket sums (every source active for the whole schedule).
    pub sums: TimingSums,
}

/// Bucket sums of the timing stats (06 E-6.1, E-6.4, E-6.6, E-6.7, E-6.9), static or as a buff's delta.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimingSums {
    /// ΣFR with the bow weighting (06 E-6.1) and the zoom level's change.
    pub fire: f64,
    /// Charge-rate bonus (`charge_rate`, 06 §5.3).
    pub charge: f64,
    /// ΣMag (fraction).
    pub magazine: f64,
    /// Mag+ (rounds).
    pub magazine_flat: f64,
    /// RS, the reload-speed bucket.
    pub reload: f64,
    /// ΣAE before the 0.75 clamp of E-6.7.
    pub ammo_eff: f64,
}

/// One 06 §5.9 timing buff: switched on by a schedule event of this attack, on for its duration.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimingBuff {
    /// The schedule event that switches it on.
    pub trigger: TimingTrigger,
    /// Duration (s); `f64::INFINITY` = until the magazine ends.
    pub duration: f64,
    /// Its additions to the timing sums while on (at max stacks).
    pub add: TimingSums,
}

/// The schedule events of 06 §5.9 that switch a timing buff on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimingTrigger {
    /// On reload: starts at reload start and counts for the T_rel of that reload (Spring-Loaded Chamber family).
    #[default]
    Reload,
    /// On a reload from empty (a_left ≤ ε), including the T_rel of that reload (Ready Retaliation).
    ReloadFromEmpty,
    /// The first-shot mark of E-6.13.
    FirstShot,
    /// The last-shot mark of E-6.13.
    LastShot,
}

/// Reload model (06 E-6.9) and reload lead (06 E-6.10).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReloadInputs {
    /// Reload style and its data.
    pub style: ReloadStyleInputs,
    /// δ_rel = 0 by data: data `ReloadDelay = 0` on a non-battery weapon (Vectis, Vectis Prime, Exergis) or a bow
    /// (06 E-6.10). A modded Mag = 1 also gives 0; the generator tests the evaluated Mag itself.
    pub no_lead: bool,
    /// `BurstReloadDelay`: the lead after a MAG_BURST that emptied the magazine (06 E-6.10); `None` without it.
    pub burst_reload_delay: Option<f64>,
    /// T-40 timed reload: the 1/r lead becomes 0 unless the style is regenerate (06 E-6.10). L3, T-40.
    pub timed_reload: bool,
}

/// Reload style (06 E-6.9; data `ReloadStyle`, absent = magazine, BATTERY with `ReloadRate` = regenerate).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReloadStyleInputs {
    /// Whole-magazine reload: T_rel = R0 / (1 + RS), 1 + RS clamped to ≥ 0.05.
    Magazine {
        /// R0 (s): the attack-level `Reload` when present, else the weapon's.
        r0: f64,
    },
    /// Per-shell reload: T_rel = (t_se + t_sh·n_load) / (1 + RS), n_load = Mag − max(0, floor(a_left + ε)).
    ByRound {
        /// t_se, the start and end time (s; 0 when the split is unknown).
        t_se: f64,
        /// t_sh, the time per shell (s; R0 / Mag0 when the split is unknown).
        t_sh: f64,
    },
    /// Battery: T_rel = δ_rech' + n_regen / ρ_rech, δ_rech' = max(min(δ0, 0.5), δ0 / (1 + RS)) with δ0 the empty or
    /// the normal delay, n_regen = Mag − a_left.
    Regenerate {
        /// δ0 after a shot with ammo left (`ReloadDelay`, falling back to the empty delay).
        delay: f64,
        /// δ0 from empty (`ReloadDelayEmpty`, falling back to the normal delay).
        delay_empty: f64,
        /// ρ_rech (rounds/s).
        rate: f64,
        /// Reload speed scales the delay (false for the Shedu exception: δ_rech' = δ0).
        rs_scales_delay: bool,
    },
    /// No reload (Epitaph, exalted infinite weapons, Incarnon forms).
    None,
}

/// Incarnon form (06 E-6.14): a separate magazine, no AE, no reload; the schedule ends when `can_fire` fails.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormInputs {
    /// Mag_inc, the shared form magazine.
    pub mag: f64,
    /// T_sw = T_rel of the weapon's normal form (E-6.9).
    pub t_sw: f64,
    /// T-39: the form's schedule starts at τ = T_sw. L3, T-39.
    pub switch_counted: bool,
}

/// Volley group (06 E-6.14b): one schedule for the group, each pull fires one instance of every member at the same
/// time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolleyInputs {
    /// The first member, whose trigger and fire rate time the group.
    pub lead: AttackIdx,
    /// Number of members.
    pub members: u8,
    /// The members in data order, `attacks[..members]` (`attacks[0]` = `lead`): the schedule emits one shot record
    /// per member and pull, anchored (member, i, j) (08 E-8.0), with each member's children. Addition, contract review
    /// ENG-20.
    pub attacks: [AttackIdx; MAX_VOLLEY_MEMBERS],
    /// Cost per pull: the sum of the members' costs.
    pub cost_per_pull: f64,
}

/// Reload-triggered attacks of a primary attack (06 §9.5): one instance at each reload start, "Reload From Empty" for
/// a reload from empty, "Partial Reload" otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReloadTriggered {
    /// The attack fired at a reload from empty.
    pub from_empty: Option<AttackIdx>,
    /// The attack fired at a partial reload.
    pub partial: Option<AttackIdx>,
}

/// Auto-spool cadence (06 E-6.17): r_k = FR·min(1, s_sp + k·Δ_sp), reset at every reload.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpoolInputs {
    /// s_sp (Kohm family: FR0(single) / FR0(full)).
    pub start: f64,
    /// Δ_sp, with the fallback (1 − s_sp)/(S_sp − 1) applied.
    pub step: f64,
}

/// Peacemaker (06 E-6.19): r_i = FR·(s_0 + (1 − s_0)·min(1, s_i / t_r)), b_pm(i) = floor(i / burst) + 1,
/// M_pm = 1 + step·min(b_pm, max_bursts); infinite ammo, no reload.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PeacemakerInputs {
    /// Fire-rate ramp start s_0 (0.25).
    pub ramp_start: f64,
    /// Fire-rate ramp duration t_r (3.5 s).
    pub ramp_time: f64,
    /// Shots per burst (2).
    pub burst: u8,
    /// Damage added per burst (0.075).
    pub step: f64,
    /// Bursts after which the damage ramp stops (20).
    pub max_bursts: u8,
}

/// Sniper combo (06 E-6.16): the counter decays by 1 every D_snp of idle time; M_snp(c) from C_min.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SniperInputs {
    /// C_min (`SniperComboMin`).
    pub min: u32,
    /// D_snp = `SniperComboReset` + the `sniper_combo_dur` additions (s; Harkonar Scope).
    pub reset: f64,
    /// T-12: M_snp fixed for every shot; `None` = simulated. L3, T-12.
    pub fixed: Option<f64>,
    /// T-38 aiming: off → the counter is not built and M_snp = 1. L3, T-38.
    pub aiming: bool,
}

/// One child link (06 E-6.23-E-6.25): a child attack hanging off every instance of its parent.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ChildLink {
    /// The child attack.
    pub attack: AttackIdx,
    /// Index of the parent link in `children`; `None` = the firing attack itself (a volley member, see `member`).
    pub parent: Option<u8>,
    /// With `parent = None` in a volley group: the index into `VolleyInputs::attacks` of the member whose instances
    /// the link hangs off; 0 otherwise. Addition, contract review ENG-20.
    pub member: u8,
    /// n_inst of a firing parent per shot (06 E-6.24): `true` = one per parent projectile (ShotType Projectile, p_i),
    /// `false` = one per shot (Hit-Scan or HELD). A grandchild hangs off every instance of its parent link.
    pub per_projectile: bool,
    /// f(a): the child's data multishot when it is the first split of its chain, else 1 (06 E-6.24).
    pub split: f64,
    /// child_needs_head (hand table, role `HEADSHOT_ONLY`): exists only for parent instances that hit a part with the
    /// headshot flag (06 §9.2; with T-41 only for parent instances that hit).
    pub needs_head: bool,
    /// The child also needs a stunnable target (Zymos: 05 `cc_immune` false and `og_present` false, 06 §9.2).
    pub needs_stun: bool,
    /// h_child − h_parent: data `ExplosionDelay` or `EmbedDelay` (s, 06 §9.3; 0 default).
    pub delay: f64,
    /// Lingering effect (06 E-6.25): ticks at h_spawn + m / rate, m = 0 .. ceil(D_e·rate) − 1; `None` = one instance
    /// at h_parent + delay.
    pub lingering: Option<LingerInputs>,
}

/// A lingering child's ticks (06 E-6.25).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LingerInputs {
    /// D_e, the data `EffectDuration` (s; Nightwatch Napalm: the mod rank's duration).
    pub duration: f64,
    /// FR_child, ticks per second (data `FireRate`; 1 when it is 0 or absent).
    pub rate: f64,
}

/// 06 §5: firing stats of `atk` on the assembled weapon with the stat vector `sv`, with the data-derived
/// [`ScheduleInputs`] (the scenario-derived fields and `timing_buffs` are set at L3). T08a.
pub fn firing_stats(
    ctx: &StageCtx,
    atk: &Attack,
    w: &AssembledWeapon,
    sv: &StatVecF,
) -> FiringResolved {
    let _ = (ctx, atk, w, sv);
    todo!("T08a")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_inputs_are_fixed_size_copy() {
        // `ranged_schedule(ctx, ra, horizon)` reads them from the `Copy` ResolvedAttack (engine.md §4).
        fn assert_copy<T: Copy + Send + Sync>() {}
        assert_copy::<ScheduleInputs>();
        assert_copy::<FiringResolved>();
        let links = [ChildLink::default(); MAX_CHILD_LINKS];
        assert!(links.iter().all(|l| l.parent.is_none()));
        let buffs = [TimingBuff::default(); MAX_TIMING_BUFFS];
        assert!(buffs.iter().all(|b| b.trigger == TimingTrigger::Reload));
    }
}
