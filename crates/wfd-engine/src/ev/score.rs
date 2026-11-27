//! Ordered score keys (engine.md §7.9; 08 §10, E-8.26). Owner: T15a (constructors per metric); the order itself is
//! part of the T02a contract and implemented here.

use std::cmp::Ordering;

/// Ordered score key, compared lexicographically: lower `band` is better, then higher `primary`, then higher `tie`,
/// then higher `tie2`. `a > b` means `a` is better.
///
/// DPS and hit metrics: band 0, primary = metric value, tie = −ammo_window (08 §10 tie-break). TTK (E-8.26): band 0
/// killed: primary = −T_EV, tie = κ (larger overkill first), tie2 = −ammo_to_kill; band 1 extrapolated: primary =
/// −T_EV; band 2 never: primary = `dps_sustained` on the layered view's last layer. Bounds set the tie fields to +∞.
/// Floats are compared with `f64::total_cmp`, so the order is total (equality is bitwise on the four fields).
#[derive(Debug, Clone, Copy)]
pub struct Score {
    /// 0 killed or DPS/hit metric, 1 extrapolated, 2 never (E-8.26).
    pub band: u8,
    /// Metric value, or −T_EV.
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
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .band
            .cmp(&self.band)
            .then_with(|| self.primary.total_cmp(&other.primary))
            .then_with(|| self.tie.total_cmp(&other.tie))
            .then_with(|| self.tie2.total_cmp(&other.tie2))
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
    fn converts_to_and_from_the_dto() {
        let a = s(1, -12.5, f64::INFINITY, 0.0);
        let dto: wfd_model::api::Score = a.into();
        assert_eq!(Score::from(dto), a);
    }
}
