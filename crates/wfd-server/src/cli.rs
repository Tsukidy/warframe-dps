//! Command line (server-api.md §1.1, clap 4):
//!
//! ```text
//! warframe-dps [serve] [--port N] [--no-browser] [--listen ADDR] [--data-dir DIR] [--bundle FILE] [--threads N]
//!                      [--log-level LEVEL]
//! warframe-dps calc --code <wfd1-…> | --input calc.json [--mc-runs N] [--json]
//! warframe-dps optimize --request req.json [--node-budget N] [--json]
//! warframe-dps import <warframe-damage.com URL> [--json]
//! warframe-dps data status | check | refresh [--curated DIR] | list | activate <id> | verify
//! ```
//!
//! Owner: T11 (skeleton by T02d: the whole synopsis parses; `serve` runs [`crate::serve`]; the other commands report
//! the task that implements them: T34 for `calc`, `optimize`, `import` and `data`).

use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::{Context, bail};
use clap::{ArgGroup, Args, Parser, Subcommand};

use crate::config::Config;

/// `warframe-dps`: the local Warframe DPS calculator and optimizer.
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(name = "warframe-dps", version, args_conflicts_with_subcommands = true)]
pub struct Cli {
    /// The command; none = `serve`.
    #[command(subcommand)]
    pub command: Option<Command>,
    /// `serve` options given without the word `serve`.
    #[command(flatten)]
    pub serve: ServeArgs,
}

/// Subcommands of §1.1.
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Command {
    /// Serve the API and the frontend on localhost and open the browser (default).
    Serve(ServeArgs),
    /// Headless calculation (tests, scripts).
    Calc(CalcArgs),
    /// Headless optimizer; prints NDJSON `OptEvent`s.
    Optimize(OptimizeArgs),
    /// Import a warframe-damage.com link; prints `CalcInput` and `ImportReport`.
    Import(ImportArgs),
    /// Data bundles: status, update check, refresh, list, activate, verify.
    #[command(subcommand)]
    Data(DataCommand),
}

/// Options of `serve` (§1.1, §12.1).
#[derive(Args, Debug, Clone, Default, PartialEq)]
pub struct ServeArgs {
    /// First port tried (default 7878; 0 = an OS-assigned port).
    #[arg(long, value_name = "N")]
    pub port: Option<u16>,
    /// Do not open the browser.
    #[arg(long)]
    pub no_browser: bool,
    /// Bind this address explicitly (LAN use is opt-in; a warning is printed).
    #[arg(long, value_name = "ADDR")]
    pub listen: Option<IpAddr>,
    /// Data directory (bundles, images, logs, cache).
    #[arg(long, value_name = "DIR")]
    pub data_dir: Option<PathBuf>,
    /// Load this bundle file instead of the newest user bundle or the embedded one.
    #[arg(long, value_name = "FILE")]
    pub bundle: Option<PathBuf>,
    /// Optimizer pool size (config `optimizer_threads`).
    #[arg(long, value_name = "N")]
    pub threads: Option<usize>,
    /// Log level (config `log.level`; `RUST_LOG` also honoured).
    #[arg(long, value_name = "LEVEL")]
    pub log_level: Option<String>,
}

impl ServeArgs {
    /// Applies the options over a configuration (CLI wins, §12.1).
    pub fn apply(&self, cfg: &mut Config) {
        if let Some(port) = self.port {
            cfg.port = port;
        }
        if self.no_browser {
            cfg.open_browser = false;
        }
        if let Some(listen) = self.listen {
            cfg.listen = Some(listen);
        }
        if let Some(dir) = &self.data_dir {
            cfg.data_dir = Some(dir.clone());
        }
        if let Some(bundle) = &self.bundle {
            cfg.bundle = Some(bundle.clone());
        }
        if let Some(threads) = self.threads {
            cfg.optimizer_threads = threads.max(1);
        }
        if let Some(level) = &self.log_level {
            cfg.log_level = level.clone();
        }
    }
}

/// `warframe-dps calc`.
#[derive(Args, Debug, Clone, PartialEq)]
#[command(group(ArgGroup::new("source").required(true).args(["code", "input"])))]
pub struct CalcArgs {
    /// A share code (`wfd1-…`).
    #[arg(long, value_name = "CODE")]
    pub code: Option<String>,
    /// A `CalcInput` JSON file.
    #[arg(long, value_name = "FILE")]
    pub input: Option<PathBuf>,
    /// Monte Carlo runs.
    #[arg(long, value_name = "N")]
    pub mc_runs: Option<u32>,
    /// Print the `CalcReport` as JSON.
    #[arg(long)]
    pub json: bool,
}

/// `warframe-dps optimize`.
#[derive(Args, Debug, Clone, PartialEq)]
pub struct OptimizeArgs {
    /// An `OptRequest` JSON file.
    #[arg(long, value_name = "FILE")]
    pub request: PathBuf,
    /// Deterministic node budget (optimizer.md §8).
    #[arg(long, value_name = "N")]
    pub node_budget: Option<u64>,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// `warframe-dps import`.
#[derive(Args, Debug, Clone, PartialEq)]
pub struct ImportArgs {
    /// A warframe-damage.com URL.
    pub url: String,
    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// `warframe-dps data …` (server-api.md §9, pipeline.md §9).
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum DataCommand {
    /// Active bundle and the bundles on disk.
    Status,
    /// Check the upstream sources for changes.
    Check,
    /// Build a new user bundle from the sources.
    Refresh {
        /// Curated tables to use instead of the embedded ones.
        #[arg(long, value_name = "DIR")]
        curated: Option<PathBuf>,
    },
    /// List the bundles on disk.
    List,
    /// Activate a bundle (`embedded` or a bundle id).
    Activate {
        /// The bundle id.
        id: String,
    },
    /// Verify the bundles on disk.
    Verify,
}

/// Runs a parsed command line.
pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        None => serve_blocking(&cli.serve),
        Some(Command::Serve(args)) => serve_blocking(&args),
        Some(Command::Calc(_)) => not_yet("calc", "T34"),
        Some(Command::Optimize(_)) => not_yet("optimize", "T34"),
        Some(Command::Import(_)) => not_yet("import", "T34"),
        Some(Command::Data(_)) => not_yet("data", "T34"),
    }
}

fn not_yet(command: &str, task: &str) -> anyhow::Result<()> {
    bail!("`warframe-dps {command}` is not implemented yet ({task})")
}

fn serve_blocking(args: &ServeArgs) -> anyhow::Result<()> {
    let mut cfg = Config::default();
    args.apply(&mut cfg);
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("starting the async runtime")?
        .block_on(crate::serve(cfg))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("warframe-dps").chain(args.iter().copied()))
    }

    #[test]
    fn serve_is_the_default_command() {
        let cli = parse(&["--no-browser", "--port", "0"]).unwrap();
        assert_eq!(cli.command, None);
        assert!(cli.serve.no_browser);
        assert_eq!(cli.serve.port, Some(0));
        let explicit = parse(&[
            "serve",
            "--port",
            "9000",
            "--listen",
            "0.0.0.0",
            "--threads",
            "3",
        ])
        .unwrap();
        let Some(Command::Serve(args)) = explicit.command else {
            panic!("serve expected")
        };
        let mut cfg = Config::default();
        args.apply(&mut cfg);
        assert_eq!(cfg.port, 9000);
        assert_eq!(cfg.listen, Some("0.0.0.0".parse().unwrap()));
        assert_eq!(cfg.optimizer_threads, 3);
        assert!(cfg.open_browser);
        // Serve options and a subcommand do not mix.
        assert!(parse(&["--port", "1", "calc", "--code", "wfd1-x"]).is_err());
    }

    #[test]
    fn every_synopsis_line_parses() {
        let calc = parse(&["calc", "--code", "wfd1-abc", "--mc-runs", "10", "--json"]).unwrap();
        assert!(matches!(
            calc.command,
            Some(Command::Calc(CalcArgs {
                mc_runs: Some(10),
                json: true,
                ..
            }))
        ));
        assert!(parse(&["calc", "--input", "calc.json"]).is_ok());
        assert!(parse(&["calc"]).is_err(), "calc needs --code or --input");
        assert!(parse(&["calc", "--code", "a", "--input", "b"]).is_err());
        let opt = parse(&[
            "optimize",
            "--request",
            "req.json",
            "--node-budget",
            "2000000",
            "--json",
        ])
        .unwrap();
        assert!(matches!(
            opt.command,
            Some(Command::Optimize(OptimizeArgs {
                node_budget: Some(2_000_000),
                json: true,
                ..
            }))
        ));
        assert!(parse(&["optimize"]).is_err());
        assert!(parse(&["import", "https://warframe-damage.com/?p=x&n=1", "--json"]).is_ok());
        for data in [
            &["data", "status"][..],
            &["data", "check"],
            &["data", "list"],
            &["data", "verify"],
        ] {
            assert!(parse(data).is_ok(), "{data:?}");
        }
        assert_eq!(
            parse(&["data", "refresh", "--curated", "dir"])
                .unwrap()
                .command,
            Some(Command::Data(DataCommand::Refresh {
                curated: Some(PathBuf::from("dir"))
            }))
        );
        assert_eq!(
            parse(&["data", "activate", "embedded"]).unwrap().command,
            Some(Command::Data(DataCommand::Activate {
                id: "embedded".into()
            }))
        );
    }

    #[test]
    fn unimplemented_commands_name_their_task() {
        let err = run(parse(&["calc", "--code", "wfd1-x"]).unwrap()).unwrap_err();
        assert!(err.to_string().contains("T34"));
    }
}
