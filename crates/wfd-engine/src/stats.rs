//! Stat vectors and compiled contributions (engine.md §3; domain-model.md §10, §18). Bucket sums are fixed point
//! (`i64` in 1e-9 units), so the order of items never changes a result (I-4) and adding or removing one item is an
//! exact delta (optimizer.md §5); [`StatVec::to_f64`] converts once at L3, applying each row's aggregate
//! (`sum`, `sum→(1+x)`, `sum→Π(1+x)`, `any`, clamps).
//!
//! **Rows with the `max` aggregate** (`cc_floor`, `external.*`; domain-model.md §10) are not sums, so they never
//! travel in [`SparseContrib::entries`] and [`StatVec::add`] / [`StatVec::sub`] never touch them: L1 puts their
//! values in [`SparseContrib::max_entries`], and the slot of a [`StatVec`] holds the maximum over the contributions
//! applied with [`StatVec::raise_max`] (L2 applies every contribution of the build, scenario-fixed sources included).
//! A maximum has no exact inverse: a caller that removes a contribution with `max_entries` (the optimizer's
//! incremental search, T32) recomputes those slots from the remaining contributions. `to_f64` reads them as stored.
//! Owner: T06.

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

    /// Adds a contribution's constant entries [`SparseContrib::entries`] (exact, associative; never a `max` row).
    pub fn add(&mut self, c: &SparseContrib) {
        let _ = c;
        todo!("T06")
    }

    /// Removes a contribution's constant entries (the exact inverse of [`StatVec::add`]; `max` rows untouched).
    pub fn sub(&mut self, c: &SparseContrib) {
        let _ = c;
        todo!("T06")
    }

    /// Raises each `max` row named in [`SparseContrib::max_entries`] to at least the contribution's value
    /// (domain-model.md §10 aggregate `max`; the empty value is 0). Order-independent, not invertible.
    pub fn raise_max(&mut self, c: &SparseContrib) {
        let _ = c;
        todo!("T06")
    }

    /// The f64 view at L3 with every row's aggregate applied (domain-model.md §10 `aggregate` column; `max` rows as
    /// stored by [`StatVec::raise_max`]).
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
    /// Constants on summed rows: (`StatKey` index, value in 10^-9 units). Never a row whose aggregate is `max`.
    pub entries: SmallVec<[(u16, i64); 6]>,
    /// Constants on `max` rows (`cc_floor`, `external.*`): (`StatKey` index, value in 10^-9 units), applied with
    /// [`StatVec::raise_max`]. Addition to domain-model.md §18 (CONTRACTS.md T02a).
    pub max_entries: SmallVec<[(u16, i64); 1]>,
    /// Dynamic terms: value · Π state variables.
    pub coeffs: SmallVec<[Coeff; 2]>,
    /// Element contributions; their positions are fixed at L2 (domain-model.md §13).
    pub elements: SmallVec<[ElemEntry; 2]>,
    /// Parameters of mechanic kinds (magnitudes are in `entries`).
    pub mechanics: SmallVec<[MechanicInst; 1]>,
    /// Set-piece, shard-count, Strength and resolved-stat terms, finalized at L2.
    pub statics: SmallVec<[StaticTerm; 1]>,
    /// The ids of the dynamic conditions this contribution brings (`CompiledScenario.conditions`, ascending): L2
    /// unions them into `CompiledBuild.conditions`. Addition, contract review ENG-2.
    pub conditions: SmallVec<[u16; 1]>,
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
    /// The element type, resolved for the scenario (selectors and item parameters applied at L1). An entry whose
    /// selector is `target_weakness` (01 §4.10.4) has `weakness` set and its type is chosen per target layer by the
    /// composition (Table 4.10-A; `ElemOutcome.weakness`), so `ty` is then a placeholder.
    pub ty: wfd_model::DamageType,
    /// The `target_weakness` selector (01 §4.10.4, Table 4.10-A): a non-combining entry whose type depends on the
    /// target layer. Addition, contract review ENG-4.
    pub weakness: bool,
    /// `dot_scaling` of 04 §4.2 resolved for the entry: `EffectDef.dot_scaling`, else the 03 §9.3 default (true,
    /// false for `target_weakness`): whether it enters `elem_dot_sum[t]` of 01 E-1.15 (`ElemOutcome.elem_dot`).
    /// Addition, contract review DC-11.
    pub dot_scaling: bool,
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
///
/// Order contract (contract review ENG-19, C-36): `entries` are sorted by slot, then by the source's position in the
/// slot layout (`Rules.slot_layouts` order, which puts the Incarnon perk slots `inc2` … `inc5` in ascending tier),
/// then by effect index; scenario-fixed sources (`CompiledScenario.fixed`) follow the build's entries of their slot.
/// That order is 02 E-2.5's "ascending Incarnon evolution tier" for the CDM0+ entries of `cd_base_add` (Incarnon
/// perks are the only CDM0+ source). L3 does not read them (it sees only a `BuildPoint`) and passes the slot's sum as
/// one entry, which is exact while at most one entry exists (02 E-2.5: the case with current data); T17 counts the
/// CDM0+ entries here and warns with `rule.unverified { row = "02 E-2.5" }` when a build has more than one (contract
/// review ENG-R1, pass 3).
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
    fn max_rows_take_the_maximum_and_are_not_summed() {
        // domain-model.md §10: `cc_floor` aggregates by max. Two sources 2.0 and 1.5 give 2.0, not 3.5.
        let floor = StatKey::parse("cc_floor").expect("registry key").index() as u16;
        let a = SparseContrib {
            max_entries: smallvec::smallvec![(floor, 2 * FIXED_SCALE)],
            ..Default::default()
        };
        let b = SparseContrib {
            max_entries: smallvec::smallvec![(floor, 3 * FIXED_SCALE / 2)],
            ..Default::default()
        };
        let mut v = StatVec::ZERO;
        v.add(&a);
        v.add(&b);
        assert_eq!(v.0[usize::from(floor)], 0, "add never touches max rows");
        v.raise_max(&b);
        v.raise_max(&a);
        assert_eq!(v.0[usize::from(floor)], 2 * FIXED_SCALE);
        let f = v.to_f64();
        assert!((f.0[usize::from(floor)] - 2.0).abs() < 1e-12);
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
