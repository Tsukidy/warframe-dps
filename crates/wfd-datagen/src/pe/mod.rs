//! DE Public Export: the LZMA index and the manifests (pipeline.md §3.2, §4.2). Owner: T05.
//!
//! T05 adds the index parsing (legacy LZMA-alone with `lzma-rs`, CRLF lines used verbatim in manifest URLs), the
//! `buildLabel`, and the manifest wire structs (strict `serde_json` with a lenient fallback that strips raw control
//! characters; float32 noise rounded to 6 significant digits on import), incl. `ExportWarframes` and `ExportManifest`.

/// Base URL of the Public Export (pipeline.md §3.2): the index is `<base>index_en.txt.lzma`, a manifest
/// `<base>Manifest/<index line>`, an image `<base><textureLocation>`.
pub const PE_BASE_URL: &str = "https://content.warframe.com/PublicExport/";

/// The index file (pipeline.md §11).
pub const PE_INDEX: &str = "index_en.txt.lzma";

/// The manifests the pipeline reads (pipeline.md §11), by the name before `!<hash>` in the index.
/// `ExportWarframes_en.json` also holds `ExportAbilities` (Helminth abilities, for buff ids).
pub const PE_MANIFESTS: [&str; 6] = [
    "ExportWeapons_en.json",
    "ExportUpgrades_en.json",
    "ExportRelicArcane_en.json",
    "ExportWarframes_en.json",
    "ExportSentinels_en.json",
    "ExportManifest.json",
];

/// The typed wire records of the Public Export manifests (weapons, upgrades and mod sets with riven `upgradeEntries`,
/// arcanes, warframes and abilities, sentinels, `textureLocation`s) and the build label. Created empty by T02c; T05
/// defines the fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PeRecords {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{code_spans, pipeline_md, table_cell};

    #[test]
    fn files_are_those_of_pipeline_md_section_11() {
        let spans = code_spans(&table_cell(&pipeline_md(), "Public Export"));
        assert_eq!(spans[0], PE_INDEX);
        let manifests: Vec<&str> = spans
            .iter()
            .map(String::as_str)
            .filter(|s| s.ends_with(".json"))
            .collect();
        assert_eq!(manifests, PE_MANIFESTS);
        assert!(pipeline_md().contains(&format!("{PE_BASE_URL}{PE_INDEX}")));
    }
}
