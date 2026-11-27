//! DoT ticks (00 E-13; 03 E-3.4, E-3.5): weapon-side terms snapshotted at proc creation (CD-05), the "+1"
//! accumulator (D-025), tick counts `floor(6·(1 + SD + SD_s) + ε)` (CD-06), ELE/GAS tick groups, the Heat pool, Blast
//! fuses. Owner: T14.

use wfd_model::StatusType;

use crate::StageCtx;

/// Weapon-side terms of one DoT proc, snapshotted at its creation (E-13, CD-05).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcSnapshot {
    /// seed_p = Y·(1 + F)·G·b·c_k·Part_dot (not quantized).
    pub seed: f64,
    /// E^dot_s, the elemental DoT scaling of s (01 E-1.15).
    pub elem_dot: f64,
    /// F_tick,p (the inducing instance's F; Heat inherit overrides it, 03 §10.3).
    pub f_tick: f64,
    /// A_sd.
    pub a_sd: f64,
    /// A_sd,s.
    pub a_sd_type: f64,
    /// V_sd,p, the status-damage vulnerability live at creation.
    pub v_sd: f64,
    /// M_OG,p of the source attack (overguard only).
    pub m_og: f64,
    /// Proc time t_p (s).
    pub t_proc: f64,
    /// Number of ticks N_s (CD-06).
    pub ticks: u32,
    /// The proc takes the "+1" accumulator a = `dot_acc` (03 E-3.5; 1 in the game profile, D-025). `false` for the
    /// forced proc of an extra hit (00 E-14: seed S_p = XH, no "+1"; 04 E-4.14, Toxic Lash) and for Primary
    /// Debilitate's proc (03 §9.8); a tick group whose oldest proc p₀ has `false` uses a = 0 (03 E-3.5).
    pub plus_one: bool,
}

/// E-13 for a single proc of status `s`: the weapon-side tick `(seed_p + a)·X_p` with a = `dot_acc` when
/// `snap.plus_one`, else 0 (03 E-3.5, 00 E-14), and
/// `X_p = K_s·(1 + E^dot_s)·(1 + F_tick,p)·(1 + A_sd + A_sd,s)·V_sd,p` (K_s from `ctx.rules`), before the
/// target-condition dips M_tick(τ) and the enemy side evaluated at tick time. T14.
pub fn dot_tick(ctx: &StageCtx, snap: &ProcSnapshot, s: StatusType) -> f64 {
    let _ = (ctx, snap, s);
    todo!("T14")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::mini_rules;
    use wfd_model::CompatFlags;

    fn snap(seed: f64, plus_one: bool) -> ProcSnapshot {
        ProcSnapshot {
            seed,
            elem_dot: 0.0,
            f_tick: 0.0,
            a_sd: 0.0,
            a_sd_type: 0.0,
            v_sd: 1.0,
            m_og: 1.0,
            t_proc: 0.0,
            ticks: 6,
            plus_one,
        }
    }

    #[test]
    #[ignore = "T14"]
    fn plus_one_only_for_ordinary_procs() {
        // 03 §9.5 (W:Damage/Calculation line 635): a Slash stack with seed 40 ticks (40 + 1)·0.35 = 14.35.
        // 00 E-14 / 03 §9.8: the forced proc of an extra hit has no "+1": 40·0.35 = 14.
        let rules = mini_rules();
        let ctx = StageCtx::new(&rules, CompatFlags::empty());
        let ordinary = dot_tick(&ctx, &snap(40.0, true), StatusType::SLA);
        let extra_hit = dot_tick(&ctx, &snap(40.0, false), StatusType::SLA);
        assert!((ordinary - 14.35).abs() < 1e-12);
        assert!((extra_hit - 14.0).abs() < 1e-12);
    }
}
