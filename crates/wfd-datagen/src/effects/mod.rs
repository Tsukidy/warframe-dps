//! Effects (pipeline.md §6.2): tier 1 stat-line import, tier 2 spec-table data, tier 3 curated TOML → `EffectDef`
//! lists on the joined catalogue, overrides merged. Owner: T19.
//!
//! T19 adds the tier-1 importer (`data/curated/rules/stat_lines.toml`, the 04 §5.1 table as anchored patterns over
//! wiki `Description` and WFCD `levelStats` lines), binding and number checks, per-rank checks, fingerprints, stale
//! detection and auto-patching (§7), and the `draft` cap for constructs without a spec vector (R-2).

use crate::{DatagenError, Joined, Report};

/// The inputs of the three tiers: the stat-line rules (tier 1), the ingested spec tables (tier 2) and the loaded
/// curated tree (tier 3). Created empty by T02c; T19 defines the fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EffectInputs {}

/// Attaches effects to every item of `j`: curated > tier 2 > tier 1, a curated entry replacing the automatic effects
/// of its item entirely; records which tier produced each item's effects and its `EffectStatus`.
pub fn attach_effects(
    j: &mut Joined,
    tiers: &EffectInputs,
    report: &mut Report,
) -> Result<(), DatagenError> {
    let _ = (j, tiers, report);
    todo!("T19: effect tiers (pipeline.md §6.2)")
}
