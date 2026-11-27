//! Run-length form of a schedule (engine.md §5): maximal runs of shots with constant spacing and equal fields other
//! than time; a 20 s rifle window is about 3-10 segments. `WindowExact` visits every shot; `WindowFast` and `Bound`
//! work per segment (engine.md §7.2). Owner: T08a.

use crate::schedule::ShotRec;

/// A run of evenly spaced, otherwise equal shots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Index of the first shot in `Schedule.shots`.
    pub first: u32,
    /// Number of shots.
    pub len: u32,
    /// Fire time of the first shot (s).
    pub start: f64,
    /// Spacing between consecutive shots (s; 0 for a single shot).
    pub spacing: f64,
}

/// The maximal segments of a shot list. T08a.
pub fn segments(shots: &[ShotRec]) -> Vec<Segment> {
    let _ = shots;
    todo!("T08a")
}
