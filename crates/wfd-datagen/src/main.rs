//! `wfd-datagen`: the CLI of the data pipeline (pipeline.md §2). Owner: T18 (work-breakdown.md §8); T02c registered
//! every subcommand of pipeline.md §2.
//!
//! The global options are [`CommonArgs`]; `fetch` is handled by `wfd_datagen::fetch::cli` (T05) and `curate` by
//! `wfd_datagen::curated::cli` (T19), so those tasks never edit this file. Exit status: 0 on success, 1 when a command
//! fails (the error and its causes on stderr), 2 on a usage error (clap).

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;
use wfd_datagen::DatagenError;
use wfd_datagen::cli::CommonArgs;
use wfd_datagen::curated::cli::CurateCommand;
use wfd_datagen::fetch::cli::FetchArgs;

/// The data pipeline of warframe-dps: fetch upstream sources, build data/generated/ and the bundle, check, diff,
/// report, ingest spec tables, curate (docs/data/pipeline.md).
#[derive(Parser, Debug)]
#[command(name = "wfd-datagen", version)]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Refresh the source cache (the only stage that uses the network); --from-lock restores the locked inputs
    Fetch(FetchArgs),
    /// Build data/generated/ (and optionally a bundle) from the locked inputs
    Build(BuildArgs),
    /// Parse, join, attach effects and validate without writing anything (CI gate)
    Check,
    /// Field-level difference of two generated directories or bundles
    Diff(DiffArgs),
    /// Write the coverage, conflict and stale-entry report (REPORT.md, report.json)
    Report,
    /// (Re)ingest the spec tables of docs/mechanics/ into data/generated/spec/
    SpecTables(SpecTablesArgs),
    /// Curator tools: todo, scaffold, rebind, render
    #[command(subcommand)]
    Curate(CurateCommand),
}

/// `wfd-datagen build [--offline] [--out data/generated] [--bundle out.bin]`.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
struct BuildArgs {
    /// Use the cache only (no network)
    #[arg(long)]
    offline: bool,
    /// Output directory of the canonical JSON [default: ROOT/data/generated]
    #[arg(long, value_name = "DIR")]
    out: Option<PathBuf>,
    /// Also write the binary bundle to this file
    #[arg(long, value_name = "FILE")]
    bundle: Option<PathBuf>,
}

/// `wfd-datagen diff <old dir|bundle> <new dir|bundle>`.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
struct DiffArgs {
    /// The old data: a generated directory or a bundle file
    #[arg(value_name = "OLD")]
    old: PathBuf,
    /// The new data: a generated directory or a bundle file
    #[arg(value_name = "NEW")]
    new: PathBuf,
}

/// `wfd-datagen spec-tables [--check]`.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
struct SpecTablesArgs {
    /// Fail when data/generated/spec/ is stale against the Markdown instead of rewriting it
    #[arg(long)]
    check: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("wfd-datagen: error: {e}");
            let mut cause = std::error::Error::source(&e);
            while let Some(c) = cause {
                eprintln!("  caused by: {c}");
                cause = c.source();
            }
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), DatagenError> {
    let common = &cli.common;
    match &cli.command {
        Command::Fetch(args) => wfd_datagen::fetch::cli::run(common, args),
        Command::Build(args) => build(common, args),
        Command::Check => check(common),
        Command::Diff(args) => diff(common, args),
        Command::Report => report(common),
        Command::SpecTables(args) => spec_tables(common, args),
        Command::Curate(cmd) => wfd_datagen::curated::cli::run(common, cmd),
    }
}

/// `wfd-datagen build`: [`wfd_datagen::build()`], then write the canonical JSON (and the bundle).
fn build(common: &CommonArgs, args: &BuildArgs) -> Result<(), DatagenError> {
    let _ = (common, args);
    todo!("T18: wfd-datagen build")
}

/// `wfd-datagen check`: parse + join + effects + validate, no emit; exit 1 on a validation error.
fn check(common: &CommonArgs) -> Result<(), DatagenError> {
    let _ = common;
    todo!("T18: wfd-datagen check (validators: T19)")
}

/// `wfd-datagen diff`: [`wfd_datagen::diff()`] of two directories or bundles, one line per change.
fn diff(common: &CommonArgs, args: &DiffArgs) -> Result<(), DatagenError> {
    let _ = (common, args);
    todo!("T18: wfd-datagen diff")
}

/// `wfd-datagen report`: REPORT.md and report.json.
fn report(common: &CommonArgs) -> Result<(), DatagenError> {
    let _ = common;
    todo!("T18: wfd-datagen report")
}

/// `wfd-datagen spec-tables [--check]` (pipeline.md §5.2).
fn spec_tables(common: &CommonArgs, args: &SpecTablesArgs) -> Result<(), DatagenError> {
    let _ = (common, args);
    todo!("T18: wfd-datagen spec-tables")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::collections::BTreeSet;

    /// The CLI synopsis block of pipeline.md §2: every line that starts with `wfd-datagen`.
    fn synopsis() -> Vec<String> {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/data/pipeline.md");
        let md = std::fs::read_to_string(path).unwrap();
        let (_, after) = md
            .split_once("CLI (`crates/wfd-datagen/src/main.rs`, clap):")
            .expect("the §2 CLI block");
        let block = after
            .split("```")
            .nth(1)
            .expect("a code block after the CLI heading");
        block
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("wfd-datagen "))
            .map(str::to_owned)
            .collect()
    }

    /// One concrete invocation of a synopsis line: every optional flag given, `<…>` placeholders filled, the first
    /// alternative of `a|b|c`, comments dropped.
    fn invocation(line: &str) -> Vec<String> {
        let line = line.split('#').next().unwrap();
        let mut s = String::new();
        let mut placeholder = false;
        for ch in line.chars() {
            match ch {
                '<' => {
                    placeholder = true;
                    s.push('x');
                }
                '>' => placeholder = false,
                _ if placeholder => {}
                '[' | ']' => s.push(' '),
                _ => s.push(ch),
            }
        }
        s.split_whitespace()
            .map(|t| {
                t.trim_end_matches("...")
                    .split('|')
                    .next()
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }

    #[test]
    fn invocation_of_a_synopsis_line() {
        assert_eq!(
            invocation("wfd-datagen fetch   [--offline] [--only wiki|pe|wfcd]   # refresh; --x"),
            ["wfd-datagen", "fetch", "--offline", "--only", "wiki"]
        );
        assert_eq!(
            invocation("wfd-datagen diff <old dir|bundle> <new>"),
            ["wfd-datagen", "diff", "x", "x"]
        );
        assert_eq!(
            invocation("wfd-datagen curate rebind <file>..."),
            ["wfd-datagen", "curate", "rebind", "x"]
        );
    }

    /// Every synopsis line of pipeline.md §2 parses, with every option it shows, into the subcommand it names; and the
    /// CLI has no subcommand the document does not list.
    #[test]
    fn every_documented_invocation_parses() {
        let lines = synopsis();
        assert!(lines.len() >= 11, "{lines:?}");
        let mut documented = BTreeSet::new();
        for line in &lines {
            let args = invocation(line);
            let cli =
                Cli::try_parse_from(&args).unwrap_or_else(|e| panic!("`{line}` → {args:?}:\n{e}"));
            let name = match &cli.command {
                Command::Fetch(a) => {
                    assert_eq!(a.from_lock, line.contains("--from-lock"));
                    "fetch"
                }
                Command::Build(a) => {
                    assert!(a.offline && a.out.is_some() && a.bundle.is_some(), "{a:?}");
                    "build"
                }
                Command::Check => "check",
                Command::Diff(_) => "diff",
                Command::Report => "report",
                Command::SpecTables(a) => {
                    assert!(a.check);
                    "spec-tables"
                }
                Command::Curate(c) => match c {
                    CurateCommand::Todo { kind, limit } => {
                        assert_eq!((kind.is_some(), *limit), (true, 50));
                        "curate todo"
                    }
                    CurateCommand::Scaffold { .. } => "curate scaffold",
                    CurateCommand::Rebind { files } => {
                        assert_eq!(files.len(), 1);
                        "curate rebind"
                    }
                    CurateCommand::Render { .. } => "curate render",
                },
            };
            assert!(
                args[1..].join(" ").starts_with(name),
                "{line} parsed as {name}"
            );
            documented.insert(name.split(' ').next().unwrap().to_owned());
        }
        let cmd = Cli::command();
        let registered: BTreeSet<String> = cmd
            .get_subcommands()
            .map(|c| c.get_name().to_owned())
            .collect();
        assert_eq!(registered, documented);
        let curate = cmd.find_subcommand("curate").unwrap();
        let tools: BTreeSet<&str> = curate.get_subcommands().map(|c| c.get_name()).collect();
        assert_eq!(
            tools,
            BTreeSet::from(["todo", "scaffold", "rebind", "render"])
        );
    }

    #[test]
    fn defaults_and_global_options() {
        let cli = Cli::try_parse_from(["wfd-datagen", "curate", "todo"]).unwrap();
        assert_eq!(
            cli.command_debug(),
            "Curate(Todo { kind: None, limit: 50 })"
        );
        let cli = Cli::try_parse_from([
            "wfd-datagen",
            "build",
            "--root",
            "/repo",
            "--cache-dir",
            "/c",
        ])
        .unwrap();
        assert_eq!(cli.common.root, PathBuf::from("/repo"));
        assert_eq!(cli.common.config().cache_dir, PathBuf::from("/c"));
        let Command::Build(b) = &cli.command else {
            panic!()
        };
        assert!(!b.offline && b.out.is_none() && b.bundle.is_none());
    }

    #[test]
    fn usage_errors() {
        for bad in [
            &["wfd-datagen"][..],
            &["wfd-datagen", "fetch", "--from-lock", "--update-lock"],
            &["wfd-datagen", "fetch", "--only", "github"],
            &["wfd-datagen", "diff", "a"],
            &["wfd-datagen", "curate", "rebind"],
            &["wfd-datagen", "curate", "todo", "--kind", "rifle"],
            &["wfd-datagen", "frobnicate"],
        ] {
            assert!(Cli::try_parse_from(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn clap_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    impl Cli {
        fn command_debug(&self) -> String {
            format!("{:?}", self.command)
        }
    }
}
