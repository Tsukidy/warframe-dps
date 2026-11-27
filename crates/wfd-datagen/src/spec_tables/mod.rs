//! Spec-table ingestion (pipeline.md §5.2): the tables `docs/mechanics/` marks as normative or ingestable, read by
//! file, heading and expected column header (a mismatch is an error, so a reformatted table is noticed, never
//! misread). Owner: T18.
//!
//! T18 adds a typed row parser per table (CO catalogue rows expanded per weapon and mapped to bundle attacks, with
//! `data/curated/overrides/co_names.toml` for names; status summary; faction modifiers; scaling coefficients), the
//! generated `data/generated/spec/*.json` (also the embedded refresh inputs, AD-13), `spec_rev`, and
//! `wfd-datagen spec-tables --check`.

use crate::{DatagenError, SpecSource};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// `data/spec_tables.toml`: the tables to ingest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecTablesConfig {
    /// `[[table]]` entries.
    #[serde(rename = "table")]
    pub tables: Vec<SpecTableDef>,
}

/// One `[[table]]` entry of `data/spec_tables.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecTableDef {
    /// Table id; also the generated file `data/generated/spec/<id>.json`.
    pub id: Box<str>,
    /// The Markdown file, relative to the repository root ([`SpecSource::Docs`]).
    pub file: PathBuf,
    /// The heading text of the section that holds the table (without the `#`s).
    pub heading: Box<str>,
    /// The expected column header, cell by cell; absent = not pinned here (T18 decides whether every table pins it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<Box<str>>>,
}

impl SpecTablesConfig {
    /// Parses the TOML text of `data/spec_tables.toml`; table ids must be unique.
    pub fn parse(text: &str, origin: &Path) -> Result<SpecTablesConfig, DatagenError> {
        let cfg: SpecTablesConfig = toml::from_str(text).map_err(|e| DatagenError::Parse {
            file: origin.display().to_string().into(),
            detail: e.to_string(),
        })?;
        for (i, t) in cfg.tables.iter().enumerate() {
            if cfg.tables[..i].iter().any(|u| u.id == t.id) {
                return Err(DatagenError::Parse {
                    file: origin.display().to_string().into(),
                    detail: format!("duplicate table id `{}`", t.id),
                });
            }
        }
        Ok(cfg)
    }

    /// Reads and parses a file.
    pub fn load(path: &Path) -> Result<SpecTablesConfig, DatagenError> {
        let text = std::fs::read_to_string(path).map_err(|source| DatagenError::Io {
            path: path.to_owned(),
            source,
        })?;
        Self::parse(&text, path)
    }
}

/// The ingested tables: one typed row list per table of [`SpecTablesConfig`], ready to become `CoSpec`s on attacks and
/// data in `Rules`. Created empty by T02c; T18 defines the fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpecTables {}

/// Reads every table of `cfg` from the spec Markdown ([`SpecSource::Docs`], checking headings and column headers) or
/// from the ingested JSON ([`SpecSource::Ingested`], the in-app refresh).
pub fn ingest_spec_tables(
    spec: &SpecSource,
    cfg: &SpecTablesConfig,
) -> Result<SpecTables, DatagenError> {
    let _ = (spec, cfg);
    todo!("T18: spec-table ingestion (pipeline.md §5.2)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::pipeline_md;

    /// The example of pipeline.md §5.2 is a valid `data/spec_tables.toml`, and its ids are the generated files of §8.1.
    #[test]
    fn pipeline_md_example_parses() {
        let md = pipeline_md();
        let block = md
            .split("```toml")
            .skip(1)
            .map(|b| b.split("```").next().unwrap())
            .find(|b| b.contains("[[table]]"))
            .expect("the [[table]] example of §5.2");
        let cfg = SpecTablesConfig::parse(block, Path::new("pipeline.md")).unwrap();
        let ids: Vec<&str> = cfg.tables.iter().map(|t| &*t.id).collect();
        assert_eq!(
            ids,
            [
                "co_catalogue",
                "status_summary",
                "faction_modifiers",
                "scaling_coefficients"
            ]
        );
        assert!(md.contains(&format!("spec/{{{}}}.json", ids.join(","))));
        let co = &cfg.tables[0];
        assert_eq!(
            co.file,
            Path::new("docs/mechanics/04-conditional-bonuses-and-buffs.md")
        );
        // Only that the example pins a header: its 11 cells lack the "Data attack" column the 04 §2.14 table has now
        // (CONTRACTS.md T02c, request to the manager; T18 writes data/spec_tables.toml from the spec).
        assert!(co.columns.is_some());
        assert!(cfg.tables[1].columns.is_none());
        // Every file named exists and contains its heading.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for t in &cfg.tables {
            let text = std::fs::read_to_string(root.join(&t.file)).unwrap();
            assert!(
                text.lines()
                    .any(|l| l.starts_with('#') && l.trim_start_matches('#').trim() == &*t.heading),
                "{}",
                t.heading
            );
        }
    }

    #[test]
    fn duplicates_and_unknown_keys_are_errors() {
        let dup = "[[table]]\nid = \"a\"\nfile = \"x.md\"\nheading = \"h\"\n[[table]]\nid = \"a\"\nfile = \"y.md\"\nheading = \"g\"\n";
        assert!(matches!(
            SpecTablesConfig::parse(dup, Path::new("t")),
            Err(DatagenError::Parse { .. })
        ));
        let unknown = "[[table]]\nid = \"a\"\nfile = \"x.md\"\nheading = \"h\"\ncolumn = []\n";
        assert!(SpecTablesConfig::parse(unknown, Path::new("t")).is_err());
    }
}
