//! WFCD warframe-items at a pinned tag (pipeline.md §3.2, §4.2). Owner: T05.
//!
//! T05 adds the file fetcher (raw files at the tag, each verified by SHA-256 from `data/sources.lock.toml`), the
//! version discovery through the npm registry (`dist-tags.latest`; the git tag is `v` + version) and the wire structs
//! (plain JSON; `levelStats` lines keep DE colour tokens such as `<DT_FIRE_COLOR>`).

/// Raw files: `<WFCD_RAW_BASE><tag>/data/json/<file>` (pipeline.md §3.2).
pub const WFCD_RAW_BASE: &str = "https://raw.githubusercontent.com/WFCD/warframe-items/";

/// The npm registry document whose `dist-tags.latest` is the newest WFCD release (pipeline.md §3.2).
pub const WFCD_REGISTRY_URL: &str = "https://registry.npmjs.org/@wfcd/items";

/// The files the pipeline reads (pipeline.md §11; `i18n/<locale>.json` later).
pub const WFCD_FILES: [&str; 10] = [
    "Primary.json",
    "Secondary.json",
    "Melee.json",
    "Arch-Gun.json",
    "Arch-Melee.json",
    "SentinelWeapons.json",
    "Warframes.json",
    "Mods.json",
    "Arcanes.json",
    "Misc.json",
];

/// The typed wire records of the WFCD files (English names, descriptions, image names, per-rank `levelStats`, the
/// catalogue cross-check). Created empty by T02c; T05 defines the fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WfcdRecords {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{code_spans, pipeline_md, table_cell};

    #[test]
    fn files_are_those_of_pipeline_md_section_11() {
        let md = pipeline_md();
        let cell = table_cell(&md, "WFCD");
        let (now, _later) = cell
            .split_once("; later")
            .expect("`; later` in the §11 WFCD row");
        assert_eq!(code_spans(now), WFCD_FILES);
        // §3.2 lists the same files by category name (the Markdown wraps lines: compare with whitespace collapsed).
        let flat = md.split_whitespace().collect::<Vec<_>>().join(" ");
        let listed = WFCD_FILES.map(|f| f.trim_end_matches(".json")).join(", ");
        assert!(flat.contains(&format!("({listed})")), "{listed}");
        assert!(flat.contains(&format!("{WFCD_RAW_BASE}<tag>/data/json/<Category>.json")));
        assert!(flat.contains(&format!("GET {WFCD_REGISTRY_URL}")));
    }
}
