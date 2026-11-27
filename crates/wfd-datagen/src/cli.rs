//! Options shared by every `wfd-datagen` subcommand (pipeline.md §2; frozen API of T02c). Each subcommand's own
//! arguments and handler live in the module of its implementing task: `fetch` in [`crate::fetch::cli`] (T05),
//! `curate` in [`crate::curated::cli`] (T19), `build`, `check`, `diff`, `report` and `spec-tables` in `main.rs` (T18).
//! A handler has the form `fn run(common: &CommonArgs, args: &Args) -> Result<(), DatagenError>`; `main` prints an
//! error and exits with status 1.

use crate::PipelineConfig;
use std::path::PathBuf;

/// Global options (accepted before or after the subcommand).
#[derive(clap::Args, Debug, Clone, PartialEq, Eq)]
pub struct CommonArgs {
    /// Repository root; the default paths (`.cache/wfd-sources`, `data/…`, `docs/mechanics/`) are relative to it.
    #[arg(long, global = true, value_name = "ROOT", default_value = ".")]
    pub root: PathBuf,
    /// Source cache directory [default: ROOT/.cache/wfd-sources].
    #[arg(long, global = true, value_name = "DIR")]
    pub cache_dir: Option<PathBuf>,
    /// Curated tree to read instead of ROOT/data/curated (e.g. a newer checkout).
    #[arg(long, global = true, value_name = "DIR")]
    pub curated: Option<PathBuf>,
}

impl Default for CommonArgs {
    fn default() -> Self {
        CommonArgs {
            root: PathBuf::from("."),
            cache_dir: None,
            curated: None,
        }
    }
}

impl CommonArgs {
    /// [`PipelineConfig::developer`] under [`root`](Self::root) with the overrides applied.
    pub fn config(&self) -> PipelineConfig {
        let mut cfg = PipelineConfig::developer(&self.root);
        if let Some(dir) = &self.cache_dir {
            cfg.cache_dir = dir.clone();
        }
        if let Some(dir) = &self.curated {
            cfg.curated = crate::InputTree::Dir(dir.clone());
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InputTree;
    use std::path::Path;

    #[test]
    fn overrides_apply() {
        let common = CommonArgs {
            root: "/repo".into(),
            cache_dir: Some("/tmp/cache".into()),
            curated: Some("/other/curated".into()),
        };
        let cfg = common.config();
        assert_eq!(cfg.cache_dir, Path::new("/tmp/cache"));
        assert!(matches!(&cfg.curated, InputTree::Dir(p) if p == Path::new("/other/curated")));
        assert_eq!(
            cfg.lock_file.as_deref(),
            Some(Path::new("/repo/data/sources.lock.toml"))
        );
        let plain = CommonArgs::default().config();
        assert_eq!(plain.cache_dir, Path::new("./.cache/wfd-sources"));
    }
}
