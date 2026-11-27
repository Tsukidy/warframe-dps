//! `wfd-datagen fetch` (pipeline.md §2, §3). Owner: T05.

use crate::cli::CommonArgs;
use crate::{DatagenError, LockMode, PipelineConfig, SourceKind};

/// `wfd-datagen fetch [--offline] [--update-lock] [--only wiki|pe|wfcd]` and `wfd-datagen fetch --from-lock`.
#[derive(clap::Args, Debug, Clone, Default, PartialEq, Eq)]
pub struct FetchArgs {
    /// No network: report what the cache lacks instead of downloading it.
    #[arg(long)]
    pub offline: bool,
    /// Fetch the newest inputs and write their revids, Public Export build label and WFCD tag to
    /// data/sources.lock.toml.
    #[arg(long)]
    pub update_lock: bool,
    /// Fetch one source only.
    #[arg(long, value_enum, value_name = "SOURCE")]
    pub only: Option<SourceKind>,
    /// Restore exactly the inputs of data/sources.lock.toml into an empty cache (wiki by revid, Public Export by
    /// content hash, WFCD at the locked tag).
    #[arg(long, conflicts_with_all = ["update_lock", "offline"])]
    pub from_lock: bool,
}

impl FetchArgs {
    /// The pipeline configuration of this invocation: `--from-lock` is [`LockMode::FromLock`]; otherwise the newest
    /// inputs ([`LockMode::Latest`]), written to the lock file with `--update-lock` (plain `fetch` refreshes the cache
    /// and leaves the lock alone).
    pub fn config(&self, common: &CommonArgs) -> PipelineConfig {
        let mut cfg = common.config();
        cfg.offline = self.offline;
        cfg.only = self.only;
        cfg.lock = if self.from_lock {
            LockMode::FromLock
        } else {
            LockMode::Latest
        };
        cfg.update_lock = self.update_lock;
        cfg
    }
}

/// Runs `wfd-datagen fetch`: [`crate::fetch()`] with [`FetchArgs::config`], progress on stderr, a summary of the
/// [`crate::FetchReport`] on stdout.
pub fn run(common: &CommonArgs, args: &FetchArgs) -> Result<(), DatagenError> {
    let _ = (common, args);
    todo!("T05: wfd-datagen fetch")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_map_to_lock_modes() {
        let common = CommonArgs::default();
        let plain = FetchArgs::default().config(&common);
        assert_eq!(
            (plain.lock, plain.update_lock, plain.offline, plain.only),
            (LockMode::Latest, false, false, None)
        );
        let update = FetchArgs {
            update_lock: true,
            only: Some(SourceKind::Wfcd),
            ..Default::default()
        }
        .config(&common);
        assert_eq!(
            (update.lock, update.update_lock, update.only),
            (LockMode::Latest, true, Some(SourceKind::Wfcd))
        );
        let restore = FetchArgs {
            from_lock: true,
            ..Default::default()
        }
        .config(&common);
        assert_eq!(
            (restore.lock, restore.update_lock),
            (LockMode::FromLock, false)
        );
        assert!(restore.lock_file.is_some());
        let offline = FetchArgs {
            offline: true,
            ..Default::default()
        }
        .config(&common);
        assert!(offline.offline);
    }
}
