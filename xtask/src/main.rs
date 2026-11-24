//! `cargo xtask <command>`: the project's task runner (overview.md §3, work-breakdown.md §2.2 and T01c).
//!
//! Every subcommand of overview.md §3 is registered in [`COMMANDS`] from the start. Each lives in its own module,
//! owned by the task of work-breakdown.md §8; until that task lands, the row's `run` is `None` and the command is a
//! stub that names its owner and exits with status 2 (a stub never passes a check silently).
//!
//! Wiring a command (the only edit of this file allowed to another task, CONTRACTS.md): add `mod <module>;` below and
//! set the row's `run` to `Some(<module>::run)`. A module's entry point is
//! `pub fn run(args: &[String]) -> anyhow::Result<()>`: `args` are the arguments after the command name, `Err` means
//! the command failed (exit status 1, the error chain printed to stderr). Modules parse their own arguments with
//! clap ([`parse_args`]).

mod ci;
mod doc_check;
mod lint_engine;
mod spec_sync;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Entry point of a subcommand: the arguments after the command name.
pub type RunFn = fn(&[String]) -> anyhow::Result<()>;

/// One registered `cargo xtask` subcommand.
pub struct XtaskCommand {
    /// Command name (`spec-sync`).
    pub name: &'static str,
    /// Owning task (work-breakdown.md §8).
    pub owner: &'static str,
    /// Source module, relative to the workspace root.
    pub module: &'static str,
    /// One-line description.
    pub about: &'static str,
    /// `None` until the owning task implements the command.
    pub run: Option<RunFn>,
}

/// Every subcommand of overview.md §3, in its order.
pub static COMMANDS: &[XtaskCommand] = &[
    XtaskCommand {
        name: "gen-ts",
        owner: "T01b",
        module: "xtask/src/gen_ts.rs",
        about: "export the TypeScript API types to frontend/src/api/generated/ [--check]",
        run: None,
    },
    XtaskCommand {
        name: "gen-schema",
        owner: "T01b",
        module: "xtask/src/gen_schema.rs",
        about: "write the JSON schemas, incl. data/curated/schema.json [--check]",
        run: None,
    },
    XtaskCommand {
        name: "spec-sync",
        owner: "T01c",
        module: "xtask/src/spec_sync.rs",
        about: "generate CompatFlags from the spec registers; check TOGGLE_MAP against spec 00 §6.1 [--check]",
        run: Some(spec_sync::run),
    },
    XtaskCommand {
        name: "spec-vectors",
        owner: "T09",
        module: "xtask/src/spec_vectors.rs",
        about: "extract the spec's testvector blocks into the twin files [--check]",
        run: None,
    },
    XtaskCommand {
        name: "lint-engine",
        owner: "T01c",
        module: "xtask/src/lint_engine.rs",
        about: "invariants I-1 (float literals) and I-2 (/Lotus/, fixture item names) on wfd-engine and wfd-opt",
        run: Some(lint_engine::run),
    },
    XtaskCommand {
        name: "ci",
        owner: "T01c",
        module: "xtask/src/ci.rs",
        about: "run the CI gates of testing.md §10 that exist, in order [--dry-run] [--keep-going]",
        run: Some(ci::run),
    },
    XtaskCommand {
        name: "effect-smoke",
        owner: "T19",
        module: "xtask/src/effect_smoke.rs",
        about: "evaluate every curated item on its reference scenario (testing.md §3)",
        run: None,
    },
    XtaskCommand {
        name: "oracle-record",
        owner: "T36",
        module: "xtask/src/oracle.rs",
        about: "record golden results from the inspiration oracle (testing.md §5)",
        run: None,
    },
    XtaskCommand {
        name: "oracle-audit",
        owner: "T40",
        module: "xtask/src/oracle_audit.rs",
        about: "compare against the oracle recordings and the accuracy-audit table [--check]",
        run: None,
    },
    XtaskCommand {
        name: "bench-check",
        owner: "T37",
        module: "xtask/src/bench.rs",
        about: "run the criterion benches in quick mode against the baselines (testing.md §9)",
        run: None,
    },
    XtaskCommand {
        name: "dist",
        owner: "T39",
        module: "xtask/src/dist.rs",
        about: "build the frontend and the release binary",
        run: None,
    },
    XtaskCommand {
        name: "dist-smoke",
        owner: "T39",
        module: "xtask/src/dist_smoke.rs",
        about: "smoke-test the release binary offline",
        run: None,
    },
    XtaskCommand {
        name: "doc-check",
        owner: "T01c",
        module: "xtask/src/doc_check.rs",
        about: "check paths, types, commands, task ids and status lines named in the architecture and data docs",
        run: Some(doc_check::run),
    },
];

/// Looks up a registered command by name.
pub fn command(name: &str) -> Option<&'static XtaskCommand> {
    COMMANDS.iter().find(|c| c.name == name)
}

/// Parses a subcommand's arguments with clap: `--help` prints and exits 0, a usage error prints and exits 2.
pub fn parse_args<T: clap::Parser>(name: &str, args: &[String]) -> T {
    T::parse_from(std::iter::once(format!("cargo xtask {name}")).chain(args.iter().cloned()))
}

/// The workspace root (the parent of the xtask crate).
pub fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap_or(manifest).to_path_buf()
}

fn cli() -> clap::Command {
    let mut cmd = clap::Command::new("xtask")
        .bin_name("cargo xtask")
        .about("warframe-dps task runner (overview.md §3)")
        .subcommand_required(true)
        .arg_required_else_help(true);
    for c in COMMANDS {
        let about = match c.run {
            Some(_) => format!("{} ({})", c.about, c.owner),
            None => format!("{} (not implemented yet: {})", c.about, c.owner),
        };
        cmd = cmd.subcommand(
            clap::Command::new(c.name)
                .about(about)
                .disable_help_flag(true)
                .arg(
                    clap::Arg::new("args")
                        .num_args(0..)
                        .trailing_var_arg(true)
                        .allow_hyphen_values(true),
                ),
        );
    }
    cmd
}

fn main() -> ExitCode {
    let matches = cli().get_matches();
    let Some((name, sub)) = matches.subcommand() else {
        return ExitCode::from(2);
    };
    let args: Vec<String> = sub
        .get_many::<String>("args")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    let Some(c) = command(name) else {
        return ExitCode::from(2);
    };
    match c.run {
        None => {
            eprintln!(
                "`cargo xtask {}` is not implemented yet: it is owned by {} ({}, work-breakdown.md §8)",
                c.name, c.owner, c.module
            );
            ExitCode::from(2)
        }
        Some(run) => match run(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::FAILURE
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_overview_command_is_registered_once() {
        // overview.md §3, the xtask line of the layout tree.
        let expected = [
            "gen-ts",
            "gen-schema",
            "spec-sync",
            "spec-vectors",
            "lint-engine",
            "ci",
            "effect-smoke",
            "oracle-record",
            "oracle-audit",
            "bench-check",
            "dist",
            "dist-smoke",
            "doc-check",
        ];
        let names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();
        assert_eq!(names, expected);
        cli().debug_assert();
    }

    #[test]
    fn stub_modules_are_the_owned_files_of_section_8() {
        for c in COMMANDS {
            assert!(
                c.module.starts_with("xtask/src/") && c.module.ends_with(".rs"),
                "{}",
                c.module
            );
            assert!(c.owner.starts_with('T'), "{}", c.owner);
            if c.run.is_some() {
                assert!(
                    workspace_root().join(c.module).is_file(),
                    "{} has no module file",
                    c.name
                );
            }
        }
    }
}
