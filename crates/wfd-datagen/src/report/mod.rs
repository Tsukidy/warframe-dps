//! The data report (pipeline.md §7, §8.1: `REPORT.md`, `report.json`, the UI's data page) and the bundle diff.
//! Owner: T18 (the [`Report`] fields are shared: T05 counts unknown fields, T18 adds joins and corrections, T19
//! validation results).

mod diff;

pub use diff::{DataDiff, DiffRecord, DiffSection, FieldChange, diff_bundles};

use crate::SourceKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wfd_model::message::{Message, Severity};
use wfd_model::{Coverage, UniqueName};

/// What a pipeline run found. Errors stop a build (pipeline.md §7); everything else is information for the curator and
/// the UI (`DataReport` of the in-app refresh).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// Validation errors (severity error); a run with any fails.
    pub errors: Vec<Message>,
    /// Validation warnings (severity warning).
    pub warnings: Vec<Message>,
    /// Conflicts the join resolved by precedence (`data.conflict`, pipeline.md §5.1).
    pub conflicts: Vec<Message>,
    /// Corrections made while deriving, one line each (stance normalisation, 07 E-7.0; pipeline.md §5.3).
    pub corrections: Vec<Message>,
    /// Records of one source without a partner in another (pipeline.md §5.1).
    pub unmatched: Vec<Unmatched>,
    /// Upstream fields the wire structs do not know (kept in their `rest` maps, pipeline.md §4.1): wire type → field →
    /// count, so a new upstream field is noticed.
    pub unknown_fields: BTreeMap<Box<str>, BTreeMap<Box<str>, u32>>,
    /// Counts per item kind and effect status (also `Manifest.coverage`).
    pub coverage: Coverage,
    /// Items without a global stable key before this run (`unmodelled` until curated).
    pub new_items: Vec<UniqueName>,
    /// Curated items whose upstream text changed (fingerprint check, pipeline.md §7).
    pub stale: Vec<UniqueName>,
    /// Stale curated items whose numbers were patched from the new text (pipeline.md §7).
    pub auto_patched: Vec<UniqueName>,
}

impl Report {
    /// Adds a validation error (severity set to error).
    pub fn error(&mut self, m: Message) {
        self.errors.push(m.with_severity(Severity::Error));
    }

    /// Adds a warning (severity set to warning).
    pub fn warning(&mut self, m: Message) {
        self.warnings.push(m.with_severity(Severity::Warning));
    }

    /// `true` when the run must fail.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Counts one occurrence of an unknown upstream field.
    pub fn count_unknown_field(&mut self, wire_type: &str, field: &str) {
        *self
            .unknown_fields
            .entry(wire_type.into())
            .or_default()
            .entry(field.into())
            .or_default() += 1;
    }
}

/// A record of one source that has no partner in another (pipeline.md §5.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unmatched {
    /// The source of the record.
    pub source: SourceKind,
    /// Its uniqueName, or its name when the source has none.
    pub id: Box<str>,
    /// Why it has no partner.
    pub reason: UnmatchedReason,
}

/// Reason classes of unmatched records (pipeline.md §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnmatchedReason {
    /// Beast (companion) weapons.
    BeastWeapon,
    /// Innate upgrades (`DefaultUpgrades`).
    InnateUpgrade,
    /// Antique arcanes.
    AntiqueArcane,
    /// Railjack items.
    Railjack,
    /// None of the known classes: a join-threshold candidate.
    Unexplained,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wfd_model::message::DataKey;

    #[test]
    fn errors_and_warnings_get_their_severity() {
        let mut r = Report::default();
        assert!(!r.has_errors());
        r.warning(
            DataKey::CheckCrossSource
                .message()
                .arg("weapon", "Kuva Bramma")
                .arg("field", "fire_rate"),
        );
        assert!(!r.has_errors());
        r.error(DataKey::CheckSpecAnchors.message().arg("item", "Serration"));
        assert!(r.has_errors());
        assert_eq!(r.errors[0].severity, Some(Severity::Error));
        assert_eq!(r.warnings[0].severity, Some(Severity::Warning));
    }

    #[test]
    fn unknown_fields_are_counted() {
        let mut r = Report::default();
        r.count_unknown_field("WikiAttack", "NewField");
        r.count_unknown_field("WikiAttack", "NewField");
        r.count_unknown_field("WikiEnemy", "Other");
        assert_eq!(r.unknown_fields["WikiAttack"]["NewField"], 2);
        assert_eq!(r.unknown_fields["WikiEnemy"]["Other"], 1);
    }

    /// `report.json` round-trips.
    #[test]
    fn json_round_trip() {
        let mut r = Report::default();
        r.error(
            DataKey::CheckSchema
                .message()
                .arg("file", "aura/x.toml")
                .arg("detail", "unknown field"),
        );
        r.unmatched.push(Unmatched {
            source: SourceKind::Wiki,
            id: "Example Claws".into(),
            reason: UnmatchedReason::BeastWeapon,
        });
        r.new_items
            .push(UniqueName::from("/Lotus/Upgrades/Mods/Example"));
        let text = serde_json::to_string(&r).unwrap();
        assert!(
            text.contains("\"beast_weapon\"") && text.contains("\"wiki\""),
            "{text}"
        );
        assert_eq!(serde_json::from_str::<Report>(&text).unwrap(), r);
    }
}
