//! Stat vectors and compiled contributions (engine.md §3; domain-model.md §10, §18). Bucket sums are fixed point
//! (`i64` in 1e-9 units), so the order of items never changes a result (I-4) and adding or removing one item is an
//! exact delta (optimizer.md §5); [`StatVec::to_f64`] converts once at L3, applying each row's aggregate
//! (`sum`, `sum→(1+x)`, `sum→Π(1+x)`, `max`, `any`, clamps). Owner: T06.

use smallvec::SmallVec;
use wfd_model::{ElemOrder, ItemId, Mechanic, N_STATS, Per, StatKey};

use crate::state::StateVar;

/// Fixed-point scale of [`StatVec`] and [`SparseContrib`] entries: 1 = 10^9 units (domain-model.md §18).
pub const FIXED_SCALE: i64 = 1_000_000_000;

/// Words of a [`StatMask`].
pub const STAT_MASK_WORDS: usize = N_STATS.div_ceil(64);

/// Bucket sums of one attack, one slot per `StatKey` (fixed point, 10^-9 units). Copy-sized for incremental search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatVec(pub [i64; N_STATS]);

impl StatVec {
    /// Every bucket empty.
    pub const ZERO: StatVec = StatVec([0; N_STATS]);

    /// The raw fixed-point sum of one slot.
    pub fn raw(&self, k: StatKey) -> i64 {
        self.0[k.index()]
    }

    /// Adds a contribution's constant entries (exact, associative).
    pub fn add(&mut self, c: &SparseContrib) {
        let _ = c;
        todo!("T06")
    }

    /// Removes a contribution's constant entries (the exact inverse of [`StatVec::add`]).
    pub fn sub(&mut self, c: &SparseContrib) {
        let _ = c;
        todo!("T06")
    }

    /// The f64 view at L3 with every row's aggregate applied (domain-model.md §10 `aggregate` column).
    pub fn to_f64(&self) -> StatVecF {
        todo!("T06")
    }
}

impl Default for StatVec {
    fn default() -> Self {
        StatVec::ZERO
    }
}

/// The f64 form of a [`StatVec`] after aggregation (what the stage functions read).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatVecF(pub [f64; N_STATS]);

impl StatVecF {
    /// The value of one slot.
    pub fn get(&self, k: StatKey) -> f64 {
        self.0[k.index()]
    }
}

/// A set of stat slots (dominance and relevance, optimizer.md §2; `CompiledItem.touched`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StatMask(pub [u64; STAT_MASK_WORDS]);

impl StatMask {
    /// No slot.
    pub const EMPTY: StatMask = StatMask([0; STAT_MASK_WORDS]);

    /// Adds a slot.
    pub fn insert(&mut self, k: StatKey) {
        let i = k.index();
        self.0[i / 64] |= 1 << (i % 64);
    }

    /// Whether the set holds a slot.
    pub fn contains(&self, k: StatKey) -> bool {
        let i = k.index();
        self.0[i / 64] & (1 << (i % 64)) != 0
    }

    /// The union of two sets.
    pub fn union(&self, other: &StatMask) -> StatMask {
        let mut out = *self;
        for (a, b) in out.0.iter_mut().zip(other.0.iter()) {
            *a |= *b;
        }
        out
    }

    /// Whether no slot is set.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|w| *w == 0)
    }
}

/// Reference to one effect of one item, for traces and condition views (engine.md §11.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectRef {
    /// The item.
    pub item: ItemId,
    /// Index of the effect in `Item.effects`.
    pub effect: u16,
}

/// The compiled contribution of one item (or one fixed scenario source) to one attack (domain-model.md §18).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SparseContrib {
    /// Constants: (`StatKey` index, value in 10^-9 units).
    pub entries: SmallVec<[(u16, i64); 6]>,
    /// Dynamic terms: value · Π state variables.
    pub coeffs: SmallVec<[Coeff; 2]>,
    /// Element contributions; their positions are fixed at L2 (domain-model.md §13).
    pub elements: SmallVec<[ElemEntry; 2]>,
    /// Parameters of mechanic kinds (magnitudes are in `entries`).
    pub mechanics: SmallVec<[MechanicInst; 1]>,
    /// Set-piece, shard-count, Strength and resolved-stat terms, finalized at L2.
    pub statics: SmallVec<[StaticTerm; 1]>,
    /// The effects that produced the entries (traces).
    pub sources: SmallVec<[EffectRef; 4]>,
}

/// A dynamic term `coef · Π vars` on one slot (domain-model.md §12.2, §18).
#[derive(Debug, Clone, PartialEq)]
pub struct Coeff {
    /// `StatKey` index of the slot.
    pub slot: u16,
    /// The coefficient (model units).
    pub coef: f64,
    /// The state variables multiplied in.
    pub vars: SmallVec<[StateVar; 2]>,
}

/// One element contribution: its ordering class, the resolved type and the value (domain-model.md §13, 01 §4.10).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElemEntry {
    /// Ordering class (`Slot`, `Riven`, `WarframeArcane`, `Innate`, `NonCombining`).
    pub order: ElemOrder,
    /// Line index within the source (riven lines bottom-up, E-1.8 sub-index).
    pub line: u8,
    /// The element type, resolved for the scenario (selectors and item parameters applied at L1).
    pub ty: wfd_model::DamageType,
    /// Value in 10^-9 units.
    pub value: i64,
}

/// A mechanic kind with its parameters as compiled for one attack (domain-model.md §11.2).
#[derive(Debug, Clone, PartialEq)]
pub struct MechanicInst {
    /// The mechanic and its typed parameters.
    pub mechanic: Mechanic,
    /// `StatKey` index of its magnitude slot (`xh.<k>` or `mech.<n>`); `None` for kinds without a magnitude.
    pub slot: Option<u16>,
    /// The effect it came from.
    pub source: EffectRef,
}

/// A static scalar term resolved by the L2 finalizers (engine.md §3): `value · per` on one slot.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticTerm {
    /// `StatKey` index of the slot.
    pub slot: u16,
    /// The static scalar (`SetPiece`, `ShardCount`, `Strength`, `Resolved`).
    pub per: Per,
    /// The value per unit of the scalar.
    pub value: f64,
    /// The effect it came from.
    pub source: EffectRef,
}

/// The mechanics of a compiled build (engine.md §3 `CompiledBuild.mechanics`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MechSet {
    /// Every mechanic instance, in canonical order (item order, then effect order).
    pub insts: SmallVec<[MechanicInst; 4]>,
}

/// Which effects contributed to which slots, for traces (engine.md §3 `CompiledBuild.sources`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SourceIndex {
    /// (`StatKey` index, effect, contribution in 10^-9 units).
    pub entries: Vec<(u16, EffectRef, i64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_insert_contains_union() {
        let mut a = StatMask::EMPTY;
        assert!(a.is_empty());
        let k = StatKey::parse("forced.SLA").expect("registry key");
        a.insert(k);
        assert!(a.contains(k));
        let mut b = StatMask::EMPTY;
        let last = StatKey::parse("display.ammo_pickup").expect("registry key");
        b.insert(last);
        let u = a.union(&b);
        assert!(u.contains(k) && u.contains(last));
        assert_eq!(StatVec::default(), StatVec::ZERO);
    }

    #[test]
    #[ignore = "T06"]
    fn add_then_sub_is_exact() {
        let mut v = StatVec::ZERO;
        let c = SparseContrib {
            entries: smallvec::smallvec![(0, 123), (5, -7)],
            ..Default::default()
        };
        v.add(&c);
        v.sub(&c);
        assert_eq!(v, StatVec::ZERO);
    }
}
