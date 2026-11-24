//! `cargo xtask`: skeleton created by T01a; T01c registers every subcommand of overview.md §3.
//!
//! Interim dispatcher of T01b for the two commands T01b owns (`gen-ts`, `gen-schema`). T01c's `main.rs` supersedes
//! this file; wiring the commands there is `mod gen_schema; mod gen_ts;` and `run: Some(gen_ts::run)`,
//! `run: Some(gen_schema::run)` in its command table (both modules have the entry point
//! `pub fn run(args: &[String]) -> anyhow::Result<()>`; CONTRACTS.md, T01b).

mod gen_schema;
mod gen_ts;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (name, rest) = match args.split_first() {
        Some((n, r)) => (n.as_str(), r),
        None => ("", &[][..]),
    };
    let run = match name {
        "gen-ts" => gen_ts::run,
        "gen-schema" => gen_schema::run,
        other => {
            eprintln!(
                "unknown command `{other}`: this interim xtask (T01b) knows `gen-ts [--check]` and \
                 `gen-schema [--check]`; the full command set is T01c's"
            );
            return ExitCode::from(2);
        }
    };
    match run(rest) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
