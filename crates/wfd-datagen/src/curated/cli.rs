//! `wfd-datagen curate …` (pipeline.md §2, §6.3). Owner: T19.

use crate::DatagenError;
use crate::cli::CommonArgs;
use std::path::PathBuf;
use wfd_model::api::ItemKindName;

/// The curator tools.
#[derive(clap::Subcommand, Debug, Clone, PartialEq)]
pub enum CurateCommand {
    /// List items whose status is unmodelled or draft, by the priority of data/curated/priority.toml, with their
    /// upstream text.
    Todo {
        /// Only this item kind (`weapon_mod`, `weapon_arcane`, `frame_mod`, `aura`, …).
        #[arg(long, value_name = "KIND", value_parser = parse_item_kind)]
        kind: Option<ItemKindName>,
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

/// Parses an item kind slug (`weapon_mod`, the JSON form of [`ItemKindName`]).
pub fn parse_item_kind(s: &str) -> Result<ItemKindName, String> {
    serde_json::from_value(serde_json::Value::String(s.to_owned()))
        .map_err(|_| format!("unknown item kind `{s}`"))
}

/// Runs `wfd-datagen curate <command>`.
pub fn run(common: &CommonArgs, cmd: &CurateCommand) -> Result<(), DatagenError> {
    let _ = (common, cmd);
    todo!("T19: wfd-datagen curate")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_kind_slugs() {
        assert_eq!(parse_item_kind("weapon_mod"), Ok(ItemKindName::WeaponMod));
        assert_eq!(
            parse_item_kind("incarnon_perk"),
            Ok(ItemKindName::IncarnonPerk)
        );
        assert!(parse_item_kind("WeaponMod").is_err());
    }
}
