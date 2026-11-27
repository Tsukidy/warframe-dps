//! Wiki Lua module sources and their typed wire records (pipeline.md §3.2, §4.1). Owner: T05.
//!
//! T05 adds the typed wire structs (`wiki/*.rs`): `#[serde(default)]`, `#[serde(alias)]` for the known typos
//! (`Hitscan`, `Ammocost`, `ForcedProc`, `EximusSHield`, `IsSlilent`; `DS:wiki` §11), entries with
//! `_IgnoreEntry = true` skipped, unknown fields kept in a `rest: BTreeMap` and counted in the [`crate::Report`].

/// The wiki modules the pipeline reads (pipeline.md §11; railjack weapons excluded). Not used: `Weapons/ppdata`,
/// `Enemies/data/pre-U36`, `Flaicher/dispoData`, `Maximization/data`, `Ability/data`.
pub const WIKI_MODULES: [&str; 29] = [
    "Module:Weapons/data/primary",
    "Module:Weapons/data/secondary",
    "Module:Weapons/data/melee",
    "Module:Weapons/data/archwing",
    "Module:Weapons/data/companion",
    "Module:Weapons/data/modular",
    "Module:Weapons/data/misc",
    "Module:Enemies/data/grineer",
    "Module:Enemies/data/corpus",
    "Module:Enemies/data/infestation",
    "Module:Enemies/data/orokin",
    "Module:Enemies/data/sentient",
    "Module:Enemies/data/stalker",
    "Module:Enemies/data/narmer",
    "Module:Enemies/data/themurmur",
    "Module:Enemies/data/techrot",
    "Module:Enemies/data/scaldra",
    "Module:Enemies/data/anarchs",
    "Module:Enemies/data/unaffiliated",
    "Module:DamageTypes/data",
    "Module:Stances/data",
    "Module:Modular/data",
    "Module:Modular/data/kitgun",
    "Module:Mods/data",
    "Module:Mods/itemCompatibility/data",
    "Module:Arcane/data",
    "Module:Warframes/data",
    "Module:Upgrades/data",
    "Module:Enum/data",
];

/// One wiki module as fetched: cached as `wiki/<Module_Name>@<revid>.lua` (pipeline.md §3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiSource {
    /// The title (`Module:Weapons/data/primary`).
    pub title: Box<str>,
    /// The MediaWiki revision id (also `LicenceNotice.modules`).
    pub revid: u32,
    /// The Lua source.
    pub text: String,
}

/// The typed wire records of every wiki module (weapons and attacks, enemies, damage-type table, stances, modular
/// parts, mods, item compatibility, arcanes, warframes, enums). Created empty by T02c; T05 defines the fields (the
/// name and module path are the contract the join of T18 codes against).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WikiRecords {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{code_spans, pipeline_md, table_cell};

    /// Every module of the pipeline.md §11 row, in its order, and nothing it lists as not used.
    #[test]
    fn modules_are_those_of_pipeline_md_section_11() {
        let cell = table_cell(&pipeline_md(), "wiki modules");
        let (used, _not_used) = cell
            .split_once("Not used:")
            .expect("`Not used:` in the §11 wiki row");
        let doc: Vec<String> = code_spans(used)
            .into_iter()
            .map(|t| format!("Module:{t}"))
            .collect();
        assert_eq!(doc, WIKI_MODULES);
    }
}
