//! Ordered score keys (engine.md §7.9; 08 §10, E-8.26). Owner: T15a (constructors per metric); the order itself is
//! part of the T02a contract and implemented here.

use std::cmp::Ordering;

/// Ordered score key, compared lexicographically: lower `band` is better, then higher `primary`, then higher `tie`,
/// then higher `tie2`. `a > b` means `a` is better.
///
/// DPS and hit metrics: band 0, primary = metric value, tie = −ammo_window (08 §10 tie-break). TTK and
/// `ttk_noarmor` (E-8.26): band 0 killed by t_ttk_max: primary = −T_EV, tie = κ (larger overkill first), tie2 =
/// −ammo_to_kill (08 §10 tie-break); band 1 extrapolated: primary = −T_EV; band 2 never: primary = the fraction of
/// the total pool (OG + SH + HP) removed by t_ttk_max on the expected path of E-8.21, tie = `dps_sustained[auto]`
/// (E-8.26; engine.md §7.9 says "primary = dps_sustained on the layered view's last layer", a conflict recorded as
/// C-33 in CONTRACTS.md, T02a). Unused fields are 0. Bounds set the tie fields to +∞.
///
/// Floats are compared by `f64::total_cmp` after [`Score::key`]: −0.0 is read as +0.0 (a one-shot kill has T_EV = 0,
/// EC-08-33, and builds with equal T_EV are then ordered by κ, not by the sign of zero), and NaN as −∞ (the worst
/// value: a NaN score never becomes the incumbent). A NaN is a bug in the constructor: [`Score::new`] asserts in
/// debug builds that none occurs, which matters most for bounds (a NaN bound read as −∞ would prune). `PartialEq`
/// and `Eq` agree with this order.
#[derive(Debug, Clone, Copy)]
pub struct Score {
    /// 0 killed or DPS/hit metric, 1 extrapolated, 2 never (E-8.26).
    pub band: u8,
    /// Metric value, −T_EV, or the band-2 removed fraction.
    pub primary: f64,
    /// First tie-break.
    pub tie: f64,
    /// Second tie-break.
    pub tie2: f64,
}

impl Score {
    /// The worst key: band 2, every float −∞ (the initial incumbent).
    pub const WORST: Score = Score {
        band: 2,
        primary: f64::NEG_INFINITY,
        tie: f64::NEG_INFINITY,
        tie2: f64::NEG_INFINITY,
    };

    /// A score from its fields, with −0.0 stored as +0.0; panics in debug builds on NaN (a constructor bug).
    pub fn new(band: u8, primary: f64, tie: f64, tie2: f64) -> Score {
        debug_assert!(
            !(primary.is_nan() || tie.is_nan() || tie2.is_nan()),
            "NaN in a score (band {band}): {primary}, {tie}, {tie2}"
        );
        Score {
            band,
            primary: Score::key(primary),
            tie: Score::key(tie),
            tie2: Score::key(tie2),
        }
    }

    /// The comparison value of one field: −0.0 → +0.0, NaN → −∞, every other value unchanged.
    pub fn key(x: f64) -> f64 {
        if x.is_nan() {
            f64::NEG_INFINITY
        } else if x == 0.0 {
            0.0
        } else {
            x
        }
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .band
            .cmp(&self.band)
            .then_with(|| Score::key(self.primary).total_cmp(&Score::key(other.primary)))
            .then_with(|| Score::key(self.tie).total_cmp(&Score::key(other.tie)))
            .then_with(|| Score::key(self.tie2).total_cmp(&Score::key(other.tie2)))
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Score {}

impl From<Score> for wfd_model::api::Score {
    fn from(s: Score) -> Self {
        wfd_model::api::Score {
            band: s.band,
            primary: s.primary,
            tie: s.tie,
            tie2: s.tie2,
        }
    }
}

impl From<wfd_model::api::Score> for Score {
    fn from(s: wfd_model::api::Score) -> Self {
        Score {
            band: s.band,
            primary: s.primary,
            tie: s.tie,
            tie2: s.tie2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(band: u8, primary: f64, tie: f64, tie2: f64) -> Score {
        Score {
            band,
            primary,
            tie,
            tie2,
        }
    }

    #[test]
    fn lower_band_wins_then_higher_fields() {
        assert!(s(0, -100.0, 0.0, 0.0) > s(1, -1.0, 0.0, 0.0));
        assert!(s(1, -5.0, 0.0, 0.0) > s(2, 1e9, 0.0, 0.0));
        assert!(s(0, 10.0, -3.0, 0.0) > s(0, 9.0, 0.0, 0.0));
        assert!(s(0, 10.0, -3.0, 0.0) > s(0, 10.0, -4.0, 0.0));
        assert!(s(0, 10.0, -3.0, -1.0) > s(0, 10.0, -3.0, -2.0));
        assert_eq!(s(0, 10.0, -3.0, -1.0), s(0, 10.0, -3.0, -1.0));
        assert!(Score::WORST < s(2, 0.0, 0.0, 0.0));
        // A bound with +∞ tie fields is not below any completion of its band and primary.
        assert!(s(0, 10.0, f64::INFINITY, f64::INFINITY) >= s(0, 10.0, 0.9, -40.0));
    }

    #[test]
    fn sorting_descending_puts_the_best_first() {
        let mut v = vec![
            s(2, 5.0, 0.0, 0.0),
            s(0, 1.0, 0.0, 0.0),
            s(1, -2.0, 0.0, 0.0),
            s(0, 3.0, 0.0, 0.0),
        ];
        v.sort_by(|a, b| b.cmp(a));
        assert_eq!(
            v,
            vec![
                s(0, 3.0, 0.0, 0.0),
                s(0, 1.0, 0.0, 0.0),
                s(1, -2.0, 0.0, 0.0),
                s(2, 5.0, 0.0, 0.0)
            ]
        );
    }

    #[test]
    fn negative_zero_equals_positive_zero() {
        // EC-08-33: two one-shot kills have T_EV = 0; primary −T_EV may be −0.0 for one and +0.0 for the other.
        // They must tie on primary and be ordered by κ (E-8.26 band 0).
        let a = s(0, -0.0, 0.9, 0.0);
        let b = s(0, 0.0, 0.4, 0.0);
        assert!(a > b, "larger κ wins when T_EV is equal");
        assert_eq!(s(0, -0.0, 0.5, 0.0), s(0, 0.0, 0.5, 0.0));
        assert_eq!(
            Score::new(0, -0.0, 0.0, -0.0).primary.to_bits(),
            0.0_f64.to_bits()
        );
    }

    #[test]
    fn nan_ranks_worst() {
        // A NaN never becomes an unbeatable incumbent: it compares as −∞.
        let nan = s(0, f64::NAN, 0.0, 0.0);
        let neg_nan = s(0, -f64::NAN, 0.0, 0.0);
        assert!(nan < s(0, -1e300, 0.0, 0.0));
        assert!(neg_nan < s(0, -1e300, 0.0, 0.0));
        assert!(nan < s(0, f64::INFINITY, 0.0, 0.0));
        assert_eq!(nan, s(0, f64::NEG_INFINITY, 0.0, 0.0));
        // The band still dominates.
        assert!(nan > s(1, 5.0, 0.0, 0.0));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "NaN in a score")]
    fn constructor_rejects_nan_in_debug_builds() {
        let _ = Score::new(0, f64::NAN, 0.0, 0.0);
    }

    #[test]
    fn band_two_orders_by_removed_fraction_then_sustained_dps() {
        // E-8.26 band 2: larger fraction of OG + SH + HP removed by t_ttk_max first, then higher dps_sustained[auto].
        let more_removed = s(2, 0.8, 10.0, 0.0);
        let faster_but_less_removed = s(2, 0.6, 500.0, 0.0);
        assert!(more_removed > faster_but_less_removed);
        assert!(s(2, 0.6, 600.0, 0.0) > faster_but_less_removed);
    }

    #[test]
    fn converts_to_and_from_the_dto() {
        let a = s(1, -12.5, f64::INFINITY, 0.0);
        let dto: wfd_model::api::Score = a.into();
        assert_eq!(Score::from(dto), a);
    }
}
