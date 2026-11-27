//! Field-level bundle diff (pipeline.md §2 `wfd-datagen diff`, §9). Owner: T18.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use wfd_model::Bundle;
use wfd_model::api::{DataDiffSummary, DiffSample};

/// The part of a bundle a record belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffSection {
    /// `Bundle.weapons`.
    Weapon,
    /// `Bundle.items`.
    Item,
    /// `Bundle.sets`.
    Set,
    /// `Bundle.stances`.
    Stance,
    /// `Bundle.combo_tables`.
    ComboTable,
    /// `Bundle.modular`.
    Modular,
    /// `Bundle.frames`.
    Frame,
    /// `Bundle.enemies`.
    Enemy,
    /// `Bundle.rules`.
    Rules,
    /// `Bundle.names`.
    Names,
}

/// A record present in only one of the two bundles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffRecord {
    /// Its section.
    pub section: DiffSection,
    /// Its uniqueName (or the table id for rules and combo tables).
    pub id: Box<str>,
}

/// One changed field of a record present in both bundles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldChange {
    /// The record's section.
    pub section: DiffSection,
    /// The record's uniqueName (or table id).
    pub id: Box<str>,
    /// The record's English name (`Furis`), for display.
    pub name: Box<str>,
    /// The field inside the record as the diff prints it (`normal_attack cc`).
    pub path: Box<str>,
    /// The old value as text.
    pub old: Box<str>,
    /// The new value as text.
    pub new: Box<str>,
}

impl fmt::Display for FieldChange {
    /// `Furis/normal_attack cc 0.05 → 0.07` (pipeline.md §2).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{} {} → {}", self.name, self.path, self.old, self.new)
    }
}

/// The difference of two bundles ([`crate::diff()`]).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataDiff {
    /// Records only in the new bundle.
    pub added: Vec<DiffRecord>,
    /// Records only in the old bundle.
    pub removed: Vec<DiffRecord>,
    /// Changed fields of records in both, in record order.
    pub changed: Vec<FieldChange>,
}

impl DataDiff {
    /// No record added, removed or changed.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// The summary of the in-app refresh report (server-api.md §9, `DataReport.diff`): weapons, items and enemies with
    /// at least one changed field, and the first `max_samples` changes.
    pub fn summary(&self, max_samples: usize) -> DataDiffSummary {
        let count = |section: DiffSection| {
            let ids: BTreeSet<&str> = self
                .changed
                .iter()
                .filter(|c| c.section == section)
                .map(|c| &*c.id)
                .collect();
            u32::try_from(ids.len()).unwrap_or(u32::MAX)
        };
        DataDiffSummary {
            weapons_changed: count(DiffSection::Weapon),
            items_changed: count(DiffSection::Item),
            enemies_changed: count(DiffSection::Enemy),
            samples: self
                .changed
                .iter()
                .take(max_samples)
                .map(|c| DiffSample {
                    path: format!("{}/{}", c.name, c.path).into(),
                    old: c.old.clone(),
                    new: c.new.clone(),
                })
                .collect(),
        }
    }
}

/// [`crate::diff()`]: compares two bundles record by record (matched by uniqueName, never by dense id).
pub fn diff_bundles(old: &Bundle, new: &Bundle) -> DataDiff {
    let _ = (old, new);
    todo!("T18: field-level bundle diff")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(
        section: DiffSection,
        id: &str,
        name: &str,
        path: &str,
        old: &str,
        new: &str,
    ) -> FieldChange {
        FieldChange {
            section,
            id: id.into(),
            name: name.into(),
            path: path.into(),
            old: old.into(),
            new: new.into(),
        }
    }

    #[test]
    fn display_is_the_cli_line() {
        let c = change(
            DiffSection::Weapon,
            "furis",
            "Furis",
            "normal_attack cc",
            "0.05",
            "0.07",
        );
        assert_eq!(c.to_string(), "Furis/normal_attack cc 0.05 → 0.07");
    }

    #[test]
    fn summary_counts_records_not_fields() {
        let d = DataDiff {
            added: vec![DiffRecord {
                section: DiffSection::Item,
                id: "new".into(),
            }],
            removed: vec![],
            changed: vec![
                change(
                    DiffSection::Weapon,
                    "w1",
                    "Furis",
                    "normal_attack cc",
                    "0.05",
                    "0.07",
                ),
                change(
                    DiffSection::Weapon,
                    "w1",
                    "Furis",
                    "normal_attack cd",
                    "1.5",
                    "1.7",
                ),
                change(
                    DiffSection::Weapon,
                    "w2",
                    "Lato",
                    "normal_attack sc",
                    "0.1",
                    "0.12",
                ),
                change(DiffSection::Enemy, "e1", "Lancer", "armor", "100", "150"),
                change(
                    DiffSection::Rules,
                    "factions",
                    "factions",
                    "grineer COR",
                    "1.5",
                    "1.75",
                ),
            ],
        };
        assert!(!d.is_empty() && DataDiff::default().is_empty());
        let s = d.summary(2);
        assert_eq!(
            (s.weapons_changed, s.items_changed, s.enemies_changed),
            (2, 0, 1)
        );
        assert_eq!(s.samples.len(), 2);
        assert_eq!(&*s.samples[1].path, "Furis/normal_attack cd");
        assert_eq!((&*s.samples[1].old, &*s.samples[1].new), ("1.5", "1.7"));
    }
}
