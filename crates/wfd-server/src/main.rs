//! Binary `warframe-dps` (server-api.md §1): parses the command line and runs it. Owner: T11 (skeleton by T02d).

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = wfd_server::cli::Cli::parse();
    match wfd_server::cli::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("warframe-dps: {e:#}");
            ExitCode::FAILURE
        }
    }
}
