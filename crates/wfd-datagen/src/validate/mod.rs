//! Validation (pipeline.md §7): schema, references, class, element order, applicability, squad auras (no buff item for
//! a squadmate's Corrosive Projection or Shield Disruption, which are T-23 options; contract review M-19), binding,
//! unbound numbers, rank scaling, fingerprint, upgrade types, ranges and units (`duration` ≥ 0, 0 = no expiry; DC-21),
//! rule invariants, Incarnon completeness, cross-source, coverage, spec anchors. Owner: T19 (each check with a passing
//! and a failing fixture under `tests/fixtures/curated/`, testing.md §3).

use crate::{DatagenError, Joined, Report};

/// Thresholds and allow-lists of `data/validation/`: `base.toml` (T18: cross-source allow-lists, join thresholds) and
/// `coverage/<task>.toml` (ratcheting minimums per curation batch). Created empty by T02c; T18 and T19 define the
/// fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ValidationConfig {}

/// Runs every check of pipeline.md §7 on `j`: errors and warnings go into `report` (a check that finds an error does
/// not stop the others); `Err` only when the checks cannot run at all.
pub fn validate(j: &Joined, v: &ValidationConfig, report: &mut Report) -> Result<(), DatagenError> {
    let _ = (j, v, report);
    todo!("T19: validation checks (pipeline.md §7)")
}
