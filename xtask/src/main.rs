//! `cargo xtask <command>`: the project's task runner (overview.md §3, work-breakdown.md §2.2 and T01c).
//!
//! Every subcommand of overview.md §3 is registered in [`COMMANDS`] from the start. Each lives in its own module,
//! owned by the task of work-breakdown.md §8; until that task lands, the row's `run` is `None` and the command is a
//! stub that names its owner and exits with status 2 (a stub never passes a check silently).
//!
//! Wiring a command: add `mod <module>;` below and set the row's `run` to `Some(<module>::run)`. This two-line edit is
//! the only change another task makes to this file (CONTRACTS.md T01c; work-breakdown.md §2.2 says only the owner
//! edits a file, so request C-21 asks the manager to allow it there). A module's entry point is
//! `pub fn run(args: &[String]) -> anyhow::Result<()>`: `args` are the arguments after the command name, `Err` means
//! the command failed (exit status 1, the error chain printed to stderr). Modules parse their own arguments with
//! clap ([`parse_args`]).

mod ci;
mod doc_check;
mod gen_schema;
mod gen_ts;
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
        run: Some(gen_ts::run),
    },
    XtaskCommand {
        name: "gen-schema",
        owner: "T01b",
        module: "xtask/src/gen_schema.rs",
        about: "write the JSON schemas, incl. data/curated/schema.json [--check]",
        run: Some(gen_schema::run),
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
    use std::collections::BTreeMap;

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

    /// The `xtask/src/` rows of work-breakdown.md §8: module file → owning task. A row lists paths separated by ` / `
    /// with as many owners (or one owner for all); a relative span is in `xtask/src/`; braces expand.
    fn section_8_xtask_owners() -> BTreeMap<String, String> {
        let text =
            std::fs::read_to_string(workspace_root().join("docs/architecture/work-breakdown.md"))
                .unwrap();
        let start = text
            .find("## 8. Ownership map")
            .expect("work-breakdown.md §8");
        let mut out = BTreeMap::new();
        for line in text[start..].lines().filter(|l| l.starts_with('|')) {
            let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
            let [paths, owners] = cells.as_slice() else {
                continue;
            };
            if !paths.contains("xtask/src/") {
                continue;
            }
            let groups: Vec<&str> = paths.split(" / ").collect();
            let owners: Vec<&str> = owners.split(" / ").collect();
            assert!(
                owners.len() == groups.len() || owners.len() == 1,
                "§8 row with {} path groups and {} owners: {line}",
                groups.len(),
                owners.len()
            );
            for (k, group) in groups.iter().enumerate() {
                let owner = owners[if owners.len() == 1 { 0 } else { k }];
                for span in crate::doc_check::code_spans(group) {
                    let path = if span.starts_with("xtask/") {
                        span
                    } else {
                        format!("xtask/src/{span}")
                    };
                    for p in crate::doc_check::expand(&path) {
                        out.insert(p, owner.to_owned());
                    }
                }
            }
        }
        out
    }

    #[test]
    fn stub_modules_are_the_owned_files_of_section_8() {
        let owners = section_8_xtask_owners();
        assert_eq!(
            owners.get("xtask/src/main.rs").map(String::as_str),
            Some("T01c")
        );
        for c in COMMANDS {
            assert_eq!(
                owners.get(c.module),
                Some(&c.owner.to_owned()),
                "`{}`: module {} owned by {} here, §8 says {:?}",
                c.name,
                c.module,
                c.owner,
                owners.get(c.module)
            );
            if c.run.is_some() {
                assert!(
                    workspace_root().join(c.module).is_file(),
                    "{} has no module file",
                    c.name
                );
            }
        }
        // The manifest shares the T01c row (ruling C-21); it is not a command module.
        assert_eq!(
            owners.get("xtask/Cargo.toml").map(String::as_str),
            Some("T01c")
        );
        // Every xtask module (`.rs` file) of §8 except main.rs is a registered command's module.
        for (module, owner) in &owners {
            assert!(
                module == "xtask/src/main.rs"
                    || !module.ends_with(".rs")
                    || COMMANDS.iter().any(|c| c.module == module),
                "§8 assigns {module} to {owner}, but no command uses it"
            );
        }
    }
}
