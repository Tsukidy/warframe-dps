//! Emit (pipeline.md §8): canonical JSON of `data/generated/`, `REPORT.md`, `report.json`, licences, the optional
//! bundle; and the build driver behind [`crate::build()`]. Owner: T18.
//!
//! Canonical JSON (§8.1): pretty-printed, keys sorted, two-space indent, floats in the shortest round-trip form,
//! records sorted by uniqueName, so a game patch shows as a small, readable diff.

use crate::{DatagenError, PipelineConfig, Progress, Report};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use wfd_model::Bundle;

/// The result of [`crate::build()`] (pipeline.md §2).
#[derive(Debug, Clone)]
pub struct BuildOutput {
    /// The bundle (`wfd_data::encode_bundle` writes `bundle.bin`).
    pub bundle: Bundle,
    /// Its canonical JSON form.
    pub json: GeneratedJson,
    /// Coverage, unmatched records, conflicts, stale entries, warnings.
    pub report: Report,
}

/// The files of `data/generated/` (pipeline.md §8.1): path relative to the output directory → canonical text
/// (`manifest.json`, `weapons/primary.json`, …, `stable_keys.json`, `spec/*.json`, `REPORT.md`, `report.json`,
/// `LICENSE`). Writing them is the caller's choice (`wfd-datagen build --out DIR`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneratedJson {
    /// Relative path → file content.
    pub files: BTreeMap<PathBuf, String>,
}

/// The build driver of [`crate::build()`]: parse (from the source cache only, never the network), join, derive,
/// effects, validate, emit, reporting each step to `progress` and checking `cancel` between steps. Its inputs are the
/// fields of [`PipelineConfig`]: the cache and lock, `curated`, `spec` with the list `spec_tables`, `validation`, and
/// `stable_keys` (pipeline.md §5.3: extended and emitted as `stable_keys.json` for
/// [`crate::StableKeySource::File`], session-local keys for [`crate::StableKeySource::Embedded`]).
pub fn run_build(
    cfg: &PipelineConfig,
    progress: &dyn Progress,
    cancel: &AtomicBool,
) -> Result<BuildOutput, DatagenError> {
    let _ = (cfg, progress, cancel);
    todo!("T18: the build driver (pipeline.md §2, §8)")
}
