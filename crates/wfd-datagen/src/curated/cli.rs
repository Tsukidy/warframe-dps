//! `wfd-datagen curate …` (pipeline.md §2, §6.3). Owner: T19.

use crate::DatagenError;
use crate::cli::CommonArgs;
use std::path::PathBuf;
use wfd_model::api::ItemKindName;
use wfd_model::curated::CuratedKind;

/// The curator tools.
#[derive(clap::Subcommand, Debug, Clone, PartialEq)]
pub enum CurateCommand {
    /// List items whose status is unmodelled or draft, by the priority of data/curated/priority.toml, with their
    /// upstream text.
    Todo {
        /// Only this kind: a directory of data/curated/ (pipeline.md §6.1).
        #[arg(long, value_enum, value_name = "KIND")]
        kind: Option<TodoKind>,
        /// At most this many items.
        #[arg(long, value_name = "N", default_value_t = 50)]
        limit: u32,
    },
    /// Write a draft TOML for an item, prefilled from upstream text (every rank as comments, tier-1 lines as draft
    /// effects, other lines as `# TODO` naming the DSL construct).
    Scaffold {
        /// The item's English name or uniqueName.
        #[arg(value_name = "NAME|UNIQUENAME")]
        item: String,
    },
    /// Recompute the upstream text, its SHA-256 and the binding check of curated files.
    Rebind {
        /// Curated TOML files.
        #[arg(value_name = "FILE", required = true)]
        files: Vec<PathBuf>,
    },
    /// Print the canonical English rendering of a curated file next to its upstream text.
    Render {
        /// A curated TOML file.
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

/// What `curate todo --kind` selects: one curated directory of pipeline.md §6.1 (the kind directories of
/// [`CuratedKind::dir`] and `incarnon/`). The CLI value is the directory name; the item-kind names of the API
/// ([`ItemKindName`]: `incarnon_perk`, `custom_stat`) are accepted as aliases.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[value(rename_all = "snake_case")]
pub enum TodoKind {
    /// `weapon_mod/`.
    WeaponMod,
    /// `weapon_arcane/`.
    WeaponArcane,
    /// `frame_mod/`.
    FrameMod,
    /// `aura/`.
    Aura,
    /// `frame_arcane/`.
    FrameArcane,
    /// `shard/`.
    Shard,
    /// `stance/`.
    Stance,
    /// `set/`: set bonus effects.
    Set,
    /// `incarnon/`: Incarnon family files (perks).
    #[value(alias = "incarnon_perk")]
    Incarnon,
    /// `buff/`.
    Buff,
    /// `custom/`: the custom stats of spec 00 §6.3.
    #[value(name = "custom", alias = "custom_stat")]
    Custom,
}

impl TodoKind {
    /// Every kind, in the order of pipeline.md §6.1.
    pub const ALL: [TodoKind; 11] = [
        TodoKind::WeaponMod,
        TodoKind::WeaponArcane,
        TodoKind::FrameMod,
        TodoKind::Aura,
        TodoKind::FrameArcane,
        TodoKind::Shard,
        TodoKind::Stance,
        TodoKind::Set,
        TodoKind::Incarnon,
        TodoKind::Buff,
        TodoKind::Custom,
    ];

    /// The item-file kind of the directory; `None` for `incarnon/` (family files, not item files).
    pub fn curated_kind(self) -> Option<CuratedKind> {
        Some(match self {
            TodoKind::WeaponMod => CuratedKind::WeaponMod,
            TodoKind::WeaponArcane => CuratedKind::WeaponArcane,
            TodoKind::FrameMod => CuratedKind::FrameMod,
            TodoKind::Aura => CuratedKind::Aura,
            TodoKind::FrameArcane => CuratedKind::FrameArcane,
            TodoKind::Shard => CuratedKind::Shard,
            TodoKind::Stance => CuratedKind::Stance,
            TodoKind::Set => CuratedKind::Set,
            TodoKind::Incarnon => return None,
            TodoKind::Buff => CuratedKind::Buff,
            TodoKind::Custom => CuratedKind::CustomStat,
        })
    }

    /// The directory of `data/curated/` (`"weapon_mod"`, `"incarnon"`, `"custom"`).
    pub fn dir(self) -> &'static str {
        self.curated_kind().map_or("incarnon", CuratedKind::dir)
    }

    /// The kind whose directory holds the curated file of a bundle item of this kind.
    pub fn of_item_kind(k: ItemKindName) -> TodoKind {
        match k {
            ItemKindName::WeaponMod => TodoKind::WeaponMod,
            ItemKindName::Stance => TodoKind::Stance,
            ItemKindName::WeaponArcane => TodoKind::WeaponArcane,
            ItemKindName::FrameMod => TodoKind::FrameMod,
            ItemKindName::Aura => TodoKind::Aura,
            ItemKindName::FrameArcane => TodoKind::FrameArcane,
            ItemKindName::Shard => TodoKind::Shard,
            ItemKindName::IncarnonPerk => TodoKind::Incarnon,
            ItemKindName::Buff => TodoKind::Buff,
            ItemKindName::CustomStat => TodoKind::Custom,
        }
    }
}

/// Runs `wfd-datagen curate <command>`.
pub fn run(common: &CommonArgs, cmd: &CurateCommand) -> Result<(), DatagenError> {
    let _ = (common, cmd);
    todo!("T19: wfd-datagen curate")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::ValueEnum;

    fn parse(s: &str) -> Option<TodoKind> {
        TodoKind::from_str(s, false).ok()
    }

    /// The CLI values are exactly the directories of pipeline.md §6.1: every [`CuratedKind`] directory and `incarnon`.
    #[test]
    fn values_are_the_curated_directories() {
        let mut dirs: Vec<&str> = CuratedKind::ALL.iter().map(|k| k.dir()).collect();
        dirs.push("incarnon");
        dirs.sort_unstable();
        let mut values: Vec<String> = TodoKind::value_variants()
            .iter()
            .map(|k| k.to_possible_value().unwrap().get_name().to_owned())
            .collect();
        values.sort_unstable();
        assert_eq!(values, dirs);
        assert_eq!(TodoKind::value_variants(), TodoKind::ALL);
        for k in TodoKind::ALL {
            assert_eq!(parse(k.dir()), Some(k));
        }
        for k in CuratedKind::ALL {
            let t = parse(k.dir()).unwrap();
            assert_eq!(t.curated_kind(), Some(k));
        }
        assert_eq!(parse("set"), Some(TodoKind::Set));
        assert_eq!(TodoKind::Incarnon.curated_kind(), None);
    }

    /// Every item-kind name of the API is accepted too and selects the directory of that kind's files.
    #[test]
    fn item_kind_names_are_aliases() {
        for k in [
            ItemKindName::WeaponMod,
            ItemKindName::Stance,
            ItemKindName::WeaponArcane,
            ItemKindName::FrameMod,
            ItemKindName::Aura,
            ItemKindName::FrameArcane,
            ItemKindName::Shard,
            ItemKindName::IncarnonPerk,
            ItemKindName::Buff,
            ItemKindName::CustomStat,
        ] {
            let slug = serde_json::to_value(k).unwrap();
            let slug = slug.as_str().unwrap();
            assert_eq!(parse(slug), Some(TodoKind::of_item_kind(k)), "{slug}");
        }
        assert_eq!(parse("incarnon_perk"), Some(TodoKind::Incarnon));
        assert_eq!(parse("custom_stat"), Some(TodoKind::Custom));
        assert_eq!(parse("WeaponMod"), None);
        assert_eq!(parse("rifle"), None);
    }
}
