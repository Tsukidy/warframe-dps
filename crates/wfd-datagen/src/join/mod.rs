//! Join (pipeline.md §5.1): the wire records of the three sources → one catalogue keyed by uniqueName. Owner: T18.
//!
//! T18 adds the precedence table `join::PRECEDENCE` (printed into `REPORT.md`), frames and image paths, the conflict
//! and unmatched lists of the [`crate::Report`], and the asserted match rates (testing.md §4).

use crate::{DatagenError, PeRecords, WfcdRecords, WikiRecords};

/// The joined catalogue, keyed by uniqueName (names are never join keys, `DS:pe` §1.8): weapons with their attacks,
/// items, enemies, frames, stances, modular parts. Derive, effects and validation work on it; emit turns it into the
/// bundle. Created empty by T02c; T18 defines the fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Joined {}

/// One weapon of the [`Joined`] catalogue: identity (PE), attacks and melee fields (wiki), names (WFCD). Created empty
/// by T02c; T18 defines the fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JoinedWeapon {}

/// Joins the three sources by the precedence table of pipeline.md §5.1. Conflicts and unmatched records go into the
/// report of the caller's run (T18 decides how the report reaches it).
pub fn join(w: &WikiRecords, pe: &PeRecords, wfcd: &WfcdRecords) -> Result<Joined, DatagenError> {
    let _ = (w, pe, wfcd);
    todo!("T18: join by uniqueName (pipeline.md §5.1)")
}
