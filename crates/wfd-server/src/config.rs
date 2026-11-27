//! Configuration (server-api.md §12.1): `<config_dir>/warframe-dps/config.toml`, precedence CLI > environment
//! (`WFD_*`) > file > defaults.
//!
//! Owner: T11 (skeleton by T02d: the keys and defaults of §12.1 and the CLI's serve options; reading the file and the
//! environment is T11's).

use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// First port tried (server-api.md §1.2 step 4, §12.1).
pub const DEFAULT_PORT: u16 = 7878;

/// Default `eval.mc_runs` of new scenarios (§12.1 `mc_runs_default`).
pub const DEFAULT_MC_RUNS: u32 = 1000;

/// Server configuration (server-api.md §12.1 keys; `listen` and `bundle` are CLI-only, §1.1).
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// First port tried (`port`, default 7878); 0 = an OS-assigned port.
    pub port: u16,
    /// Address bound instead of loopback (`--listen ADDR`, opt-in LAN use, §1.2 step 4); `None` = loopback.
    pub listen: Option<IpAddr>,
    /// Open the browser on start (`open_browser`, default true; `--no-browser` clears it).
    pub open_browser: bool,
    /// Optimizer pool size (`optimizer_threads`, default max(1, cores − 1), AD-15).
    pub optimizer_threads: usize,
    /// Interactive pool size (`interactive_threads`, default cores, AD-15).
    pub interactive_threads: usize,
    /// Default `eval.mc_runs` in new scenarios (`mc_runs_default`, default 1000).
    pub mc_runs_default: u32,
    /// Bundles, images, logs, cache (`data_dir`); `None` = the platform data directory.
    pub data_dir: Option<PathBuf>,
    /// A bundle file forced at startup (`--bundle FILE`, §1.2 step 2).
    pub bundle: Option<PathBuf>,
    /// Proxy images on first use (`images.fetch`, default true; tests run with false, §10).
    pub images_fetch: bool,
    /// Log filter (`log.level`, default `info`; `RUST_LOG` also honoured).
    pub log_level: String,
    /// Daily rolling log file in `<data_dir>/logs` (`log.file`, default true).
    pub log_file: bool,
}

impl Default for Config {
    fn default() -> Self {
        let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        Config {
            port: DEFAULT_PORT,
            listen: None,
            open_browser: true,
            optimizer_threads: cores.saturating_sub(1).max(1),
            interactive_threads: cores,
            mc_runs_default: DEFAULT_MC_RUNS,
            data_dir: None,
            bundle: None,
            images_fetch: true,
            log_level: "info".to_owned(),
            log_file: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_section_12_1() {
        let c = Config::default();
        assert_eq!(c.port, 7878);
        assert!(c.open_browser && c.images_fetch && c.log_file);
        assert_eq!(c.mc_runs_default, 1000);
        assert_eq!(c.log_level, "info");
        assert!(c.listen.is_none() && c.data_dir.is_none() && c.bundle.is_none());
        let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        assert_eq!(c.interactive_threads, cores);
        assert_eq!(c.optimizer_threads, (cores - 1).max(1));
    }
}
