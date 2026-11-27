//! Counter-based random numbers (08 E-8.16): Philox4x32-10 keyed by the seed, with counter (run, mechanic id, attack
//! and shot index, sub-instance and block index); the k-th draw of a (run, anchor, mechanic) uses block k >> 1 and
//! half k & 1, mapped to u = m·2^−53 ∈ [0, 1). Known answers TV-08-01 and TV-08-02 are unit tests here. Owner: T16.

use crate::error::EngineError;
use crate::mc::events::Anchor;
use crate::mc::keys::MechanicId;

/// Philox4x32-10 round multipliers (08 E-8.16).
pub const PHILOX_M: [u32; 2] = [0xD251_1F53, 0xCD9E_8D57];
/// Philox4x32-10 Weyl key increments (08 E-8.16).
pub const PHILOX_W: [u32; 2] = [0x9E37_79B9, 0xBB67_AE85];

/// 08 E-8.16: Philox4x32-10 of counter `ctr` under key `key`. T16.
pub fn philox4x32_10(ctr: [u32; 4], key: [u32; 2]) -> [u32; 4] {
    let _ = (ctr, key);
    todo!("T16")
}

/// The next ordinal per (anchor, mechanic) of one run: reusable scratch of a worker thread.
#[derive(Debug, Clone, Default)]
pub struct OrdinalTable {
    entries: Vec<(Anchor, MechanicId, u32)>,
}

impl OrdinalTable {
    /// An empty table.
    pub fn new() -> Self {
        OrdinalTable::default()
    }

    /// Forgets every ordinal (start of a run).
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// The keyed draw source of one run (engine.md §8.2): the Philox key from the seed, the run, the current anchor, the
/// per-mechanic ordinal counters and a sticky range error.
///
/// An index outside the ranges of 08 E-8.16 is an error, never wrapped and never a panic: [`McRng::set_anchor`]
/// returns it, and [`McRng::next_u`] (whose callers, the mechanics, return nothing) records the first one, e.g. a
/// block index b ≥ 65536 after 131,072 draws of one mechanic at one anchor. The simulator calls [`McRng::check`]
/// after each event and fails the evaluation with that error (`Engine::mc`, `Engine::timeline`).
#[derive(Debug)]
pub struct McRng<'r> {
    key: [u32; 2],
    run: u32,
    anchor: Anchor,
    ordinals: &'r mut OrdinalTable,
    error: Option<EngineError>,
}

impl<'r> McRng<'r> {
    /// A draw source for run `run` under `seed` (key (seed mod 2^32, floor(seed / 2^32))), anchored at the run's
    /// first shot; `ordinals` is cleared.
    pub fn new(seed: u64, run: u32, ordinals: &'r mut OrdinalTable) -> Self {
        ordinals.clear();
        McRng {
            key: [seed as u32, (seed >> 32) as u32],
            run,
            anchor: Anchor::default(),
            ordinals,
            error: None,
        }
    }

    /// The run index (counter word c0).
    pub fn run(&self) -> u32 {
        self.run
    }

    /// The Philox key.
    pub fn key(&self) -> [u32; 2] {
        self.key
    }

    /// The current anchor.
    pub fn anchor(&self) -> Anchor {
        self.anchor
    }

    /// Moves to another anchor; an index outside the E-8.16 ranges is `EngineError::RngIndexRange`, never wrapped.
    /// T16.
    pub fn set_anchor(&mut self, anchor: Anchor) -> Result<(), EngineError> {
        let _ = (anchor, &self.ordinals);
        todo!("T16")
    }

    /// The next uniform u ∈ [0, 1) of mechanic `mech` at the current anchor (E-8.16 draw rule: ordinal k uses block
    /// b = k >> 1, half k & 1). When b would be ≥ 65536, or after an earlier range error, it records
    /// `EngineError::RngIndexRange { what: "block", .. }` (the first error is kept), draws nothing and returns 0.0;
    /// the run's results are then discarded through [`McRng::check`]. T16.
    pub fn next_u(&mut self, mech: MechanicId) -> f64 {
        let _ = mech;
        todo!("T16")
    }

    /// The first range error of this run, if any (08 E-8.16; engine.md §8.2).
    pub fn error(&self) -> Option<&EngineError> {
        self.error.as_ref()
    }

    /// `Err` with the first range error of this run; the simulator calls it after each event.
    pub fn check(&self) -> Result<(), EngineError> {
        match &self.error {
            None => Ok(()),
            Some(e) => Err(e.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_splits_the_seed() {
        let mut ord = OrdinalTable::new();
        let rng = McRng::new(0xDEAD_BEEF_1234_5678, 3, &mut ord);
        assert_eq!(rng.key(), [0x1234_5678, 0xDEAD_BEEF]);
        assert_eq!(rng.run(), 3);
    }

    #[test]
    #[ignore = "T16"]
    fn tv_08_01_philox_known_answers() {
        assert_eq!(
            philox4x32_10([0; 4], [0; 2]),
            [0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8]
        );
        assert_eq!(
            philox4x32_10([0xffff_ffff; 4], [0xffff_ffff; 2]),
            [0x408f276d, 0x41c83b0e, 0xa20bc7c6, 0x6d5451fd]
        );
        assert_eq!(
            philox4x32_10(
                [0x243f6a88, 0x85a308d3, 0x13198a2e, 0x03707344],
                [0xa4093822, 0x299f31d0]
            ),
            [0xd16cfe09, 0x94fdcceb, 0x5001e420, 0x24126ea1]
        );
    }

    #[test]
    #[ignore = "T16"]
    fn tv_08_02_keyed_draws() {
        // (a) seed 0, run 3, anchor (a = 1, i = 5, j = 2), "crit_roll", ordinals 0 and 1
        let mut ord = OrdinalTable::new();
        let mut rng = McRng::new(0, 3, &mut ord);
        rng.set_anchor(Anchor { a: 1, i: 5, j: 2 })
            .expect("anchor in range");
        let crit = MechanicId(0xa4bc7899);
        assert!((rng.next_u(crit) - 0.5926430734280415).abs() < 1e-15);
        assert!((rng.next_u(crit) - 0.6150167835743386).abs() < 1e-15);
        // (b) seed 0xDEADBEEF12345678, run 0, shot-level draw (a = 0, i = 0, j = 0xFFFF), "ms_pellets", ordinal 0
        let mut ord = OrdinalTable::new();
        let mut rng = McRng::new(0xDEAD_BEEF_1234_5678, 0, &mut ord);
        rng.set_anchor(Anchor {
            a: 0,
            i: 0,
            j: Anchor::SHOT_LEVEL_J,
        })
        .expect("anchor in range");
        assert!((rng.next_u(MechanicId(0xf351e291)) - 0.8861288082231514).abs() < 1e-15);
    }

    #[test]
    fn a_fresh_draw_source_has_no_error() {
        let mut ord = OrdinalTable::new();
        let rng = McRng::new(7, 0, &mut ord);
        assert!(rng.error().is_none());
        assert!(rng.check().is_ok());
    }

    #[test]
    #[ignore = "T16"]
    fn block_index_overflow_is_a_sticky_error() {
        // E-8.16: 131,072 draws of one mechanic at one anchor use blocks 0 … 65535; the next needs b = 65536, an
        // error with no wrap-around. The draw source records it and the run reports it through `check`.
        let mut ord = OrdinalTable::new();
        let mut rng = McRng::new(0, 0, &mut ord);
        rng.set_anchor(Anchor { a: 0, i: 0, j: 0 })
            .expect("anchor in range");
        let m = MechanicId(0xa4bc7899);
        for _ in 0..131_072 {
            let _ = rng.next_u(m);
        }
        assert!(rng.check().is_ok());
        let _ = rng.next_u(m);
        assert!(matches!(
            rng.check(),
            Err(EngineError::RngIndexRange { .. })
        ));
    }

    #[test]
    #[ignore = "T16"]
    fn anchors_out_of_range_are_errors() {
        let mut ord = OrdinalTable::new();
        let mut rng = McRng::new(0, 0, &mut ord);
        assert!(
            rng.set_anchor(Anchor {
                a: 4096,
                i: 0,
                j: 0
            })
            .is_err()
        );
        assert!(
            rng.set_anchor(Anchor {
                a: 0,
                i: 1 << 20,
                j: 0
            })
            .is_err()
        );
    }
}
