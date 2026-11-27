//! Element combination (01 §4.10-§4.11; domain-model.md §13) and the [`ElementSolver`] of the optimizer
//! (optimizer.md §5.1). Owner: T06.
//!
//! Each element contribution holds a position [`OrderKey`] (01 §4.10.1: main slots 0-7, exilus 8, weapon arcane and
//! Incarnon perk slots 9 with their sub-index, warframe arcanes 10 with the HCET sub-index, the innate group, then
//! non-combining additions); [`combine`] consumes the contributions sorted by key and returns the final elemental
//! sums with the combinations formed (slot order, riven bottom-up, innate group HCET, non-combining added after,
//! 01 §4.11). A contribution holds its position only while it is active with a value > 0 (EC-01-53).

use smallvec::SmallVec;
use wfd_model::{DamageType, DamageVec};

use crate::stats::ElemEntry;

/// Position of an element contribution (domain-model.md §13), ordered lexicographically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderKey {
    /// Position class: the slot position 0-9 of 01 §4.10.1, 10 for warframe arcanes, then the innate group, then
    /// non-combining additions (the classes after 10 are T06's constants).
    pub class_rank: u8,
    /// Slot index within the class (riven: its slot).
    pub slot_index: u8,
    /// Sub-index (E-1.8): riven lines bottom-up, Incarnon evolution t, HCET index of a warframe-arcane element.
    pub sub_index: u8,
}

/// One positioned element contribution: (position, type, value in model units).
pub type OrderedElem = (OrderKey, DamageType, f64);

/// The result of 01 §4.11: the elemental part `E_t` of E-2 per final type, and which secondary elements formed from
/// which primaries.
#[derive(Debug, Clone, PartialEq)]
pub struct ElemOutcome {
    /// `E_t` of E-2 per final type (combining and non-combining contributions; innate bases are E-1's B'_t).
    pub sums: DamageVec,
    /// Secondary elements formed, in formation order.
    pub combined: SmallVec<[Combination; 3]>,
}

impl Default for ElemOutcome {
    fn default() -> Self {
        ElemOutcome {
            sums: [0.0; wfd_model::N_DT],
            combined: SmallVec::new(),
        }
    }
}

/// One combination of two primaries into a secondary element (01 §4.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Combination {
    /// The secondary element.
    pub result: DamageType,
    /// The two primaries, in the order they met.
    pub from: [DamageType; 2],
}

/// 01 §4.11: combine positioned contributions (sorted by key) into the final elemental outcome. T06.
pub fn combine(entries: &[OrderedElem]) -> ElemOutcome {
    let _ = entries;
    todo!("T06")
}

/// One candidate's element contributions for [`ElementSolver::outcomes`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElemCandidate<'a> {
    /// The caller's identifier of the candidate.
    pub id: u16,
    /// Its element contributions.
    pub entries: &'a [ElemEntry],
}

/// Which free slot realizes an outcome for one candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotAssignment {
    /// The candidate's identifier ([`ElemCandidate::id`]).
    pub candidate: u16,
    /// The slot position (an [`OrderKey`] of `slots`).
    pub slot: OrderKey,
}

/// Every distinct element outcome an unordered set of element sources can reach (engine.md §3, optimizer.md §5.1).
#[derive(Debug, Clone, Copy, Default)]
pub struct ElementSolver;

impl ElementSolver {
    /// Every distinct outcome of placing the candidates of `set` into the free slot positions `slots`, with the
    /// already positioned contributions `fixed` (exilus, arcanes, innate, non-combining), and one slot assignment
    /// realizing each (at most 3 outcomes for 3-4 base elements). T06.
    pub fn outcomes(
        fixed: &[OrderedElem],
        set: &[ElemCandidate<'_>],
        slots: &[OrderKey],
    ) -> SmallVec<[(ElemOutcome, Vec<SlotAssignment>); 3]> {
        let _ = (fixed, set, slots);
        todo!("T06")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_keys_sort_by_class_then_slot_then_sub() {
        let k = |c, s, u| OrderKey {
            class_rank: c,
            slot_index: s,
            sub_index: u,
        };
        let mut v = vec![k(10, 0, 1), k(9, 0, 3), k(0, 7, 0), k(0, 2, 0), k(9, 0, 0)];
        v.sort();
        assert_eq!(
            v,
            vec![k(0, 2, 0), k(0, 7, 0), k(9, 0, 0), k(9, 0, 3), k(10, 0, 1)]
        );
    }

    #[test]
    #[ignore = "T06"]
    fn heat_then_cold_combine_to_blast() {
        let key = OrderKey {
            class_rank: 0,
            slot_index: 0,
            sub_index: 0,
        };
        let key2 = OrderKey {
            class_rank: 0,
            slot_index: 1,
            sub_index: 0,
        };
        let out = combine(&[(key, DamageType::HEA, 0.9), (key2, DamageType::COL, 0.9)]);
        assert_eq!(out.combined.len(), 1);
        assert_eq!(out.combined[0].result, DamageType::BLA);
        assert!((out.sums[DamageType::BLA.index()] - 1.8).abs() < 1e-12);
    }
}
