//! The `wfd-datagen` binary (work-breakdown.md T02c acceptance: `cargo run -p wfd-datagen -- --help`): help lists every
//! subcommand of pipeline.md §2 the way `cargo xtask ci` detects them, and usage errors exit with status 2.

use std::process::{Command, Output};

fn datagen(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wfd-datagen"))
        .args(args)
        .output()
        .expect("run wfd-datagen")
}

/// The detection rule of `xtask/src/ci.rs` (`lists_subcommand`): a line of the `Commands:` section whose first word is
/// the subcommand.
fn listed(help: &str) -> Vec<String> {
    let mut in_commands = false;
    let mut out = Vec::new();
    for line in help.lines() {
        if !line.starts_with(' ') {
            in_commands = line.trim_end().eq_ignore_ascii_case("commands:");
            continue;
        }
        if in_commands && let Some(w) = line.split_whitespace().next() {
            out.push(w.to_owned());
        }
    }
    out
}

#[test]
fn help_lists_every_subcommand() {
    let out = datagen(&["--help"]);
    assert!(out.status.success(), "{out:?}");
    let help = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        listed(&help),
        [
            "fetch",
            "build",
            "check",
            "diff",
            "report",
            "spec-tables",
            "curate",
            "help"
        ],
        "{help}"
    );
    for opt in ["--root", "--cache-dir", "--curated", "--version"] {
        assert!(help.contains(opt), "{opt} missing from:\n{help}");
    }
}

#[test]
fn curate_help_lists_the_curator_tools() {
    let out = datagen(&["curate", "--help"]);
    assert!(out.status.success(), "{out:?}");
    let help = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        listed(&help),
        ["todo", "scaffold", "rebind", "render", "help"],
        "{help}"
    );
}

#[test]
fn every_subcommand_has_help() {
    for sub in [
        &["fetch"][..],
        &["build"],
        &["check"],
        &["diff"],
        &["report"],
        &["spec-tables"],
        &["curate", "todo"],
        &["curate", "scaffold"],
        &["curate", "rebind"],
        &["curate", "render"],
    ] {
        let mut args = sub.to_vec();
        args.push("--help");
        let out = datagen(&args);
        assert!(out.status.success(), "{args:?}: {out:?}");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("Usage: wfd-datagen"),
            "{args:?}"
        );
    }
}

#[test]
fn usage_errors_exit_with_status_2() {
    for args in [
        &[][..],
        &["fetch", "--from-lock", "--update-lock"],
        &["fetch", "--only", "github"],
        &["curate", "rebind"],
        &["frobnicate"],
    ] {
        let out = datagen(args);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {out:?}");
    }
}
