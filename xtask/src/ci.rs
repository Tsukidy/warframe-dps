//! `cargo xtask ci [--dry-run] [--keep-going]`: the CI gates of testing.md §10, in order, each passing before the
//! next (work-breakdown.md T01c).
//!
//! A gate runs when it exists, so the command is useful from wave 0 and grows with the project:
//!
//! - `cargo …` gates always exist;
//! - `cargo xtask <cmd>` gates exist once the command's owner has wired it in `main.rs` (stubs are skipped, naming the
//!   owner); they run in this process;
//! - `cargo run -p wfd-datagen -- <sub>` gates exist once `wfd-datagen --help` lists `<sub>` under its commands;
//! - `npm` gates exist once `frontend/package.json` exists (and, for `run <script>` and `test`, defines the script);
//!   the Playwright gate once `frontend/playwright.config.*` exists.
//!
//! `--dry-run` prints every gate and whether it would run. `--keep-going` runs every existing gate and reports all
//! failures at the end instead of stopping at the first.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, anyhow};

#[derive(clap::Parser, Debug)]
#[command(
    name = "cargo xtask ci",
    about = "Run the CI gates of testing.md §10 that exist (T01c)"
)]
struct Args {
    /// List the gates and whether each would run; run nothing.
    #[arg(long)]
    dry_run: bool,
    /// Run every existing gate and report all failures at the end.
    #[arg(long)]
    keep_going: bool,
}

/// How a gate runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A process: program and arguments, run in the workspace root.
    Process(&'static str, &'static [&'static str]),
    /// A registered `cargo xtask` command with arguments, run in this process.
    Xtask(&'static str, &'static [&'static str]),
    /// `cargo run -p wfd-datagen -- <sub> <args>`.
    Datagen(&'static str, &'static [&'static str]),
    /// `npm --prefix frontend <args>`; `script` names the package.json script it needs (`None`: only package.json).
    Npm(Option<&'static str>, &'static [&'static str]),
    /// `npx --prefix frontend playwright test`.
    Playwright,
}

/// One CI gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate {
    pub kind: Kind,
}

impl Gate {
    /// The command line as testing.md §10 writes it.
    pub fn label(&self) -> String {
        match self.kind {
            Kind::Process(p, a) => format!("{p} {}", a.join(" ")),
            Kind::Xtask(c, a) => format!("cargo xtask {c} {}", a.join(" "))
                .trim_end()
                .to_owned(),
            Kind::Datagen(s, a) => format!("cargo run -p wfd-datagen -- {s} {}", a.join(" "))
                .trim_end()
                .to_owned(),
            Kind::Npm(_, a) => format!("npm --prefix frontend {}", a.join(" ")),
            Kind::Playwright => "npx --prefix frontend playwright test".to_owned(),
        }
    }
}

/// The gates of testing.md §10, in order.
pub const GATES: &[Gate] = &[
    Gate {
        kind: Kind::Process("cargo", &["fmt", "--all", "--check"]),
    },
    Gate {
        kind: Kind::Process(
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
    },
    Gate {
        kind: Kind::Xtask("spec-sync", &["--check"]),
    },
    Gate {
        kind: Kind::Xtask("spec-vectors", &["--check"]),
    },
    Gate {
        kind: Kind::Xtask("gen-ts", &["--check"]),
    },
    Gate {
        kind: Kind::Xtask("lint-engine", &[]),
    },
    Gate {
        kind: Kind::Datagen("spec-tables", &["--check"]),
    },
    Gate {
        kind: Kind::Datagen("check", &[]),
    },
    Gate {
        kind: Kind::Process("cargo", &["test", "--workspace"]),
    },
    Gate {
        kind: Kind::Xtask("effect-smoke", &[]),
    },
    Gate {
        kind: Kind::Npm(None, &["ci"]),
    },
    Gate {
        kind: Kind::Npm(Some("lint"), &["run", "lint"]),
    },
    Gate {
        kind: Kind::Npm(Some("typecheck"), &["run", "typecheck"]),
    },
    Gate {
        kind: Kind::Npm(Some("test"), &["test", "--", "--run"]),
    },
    Gate {
        kind: Kind::Npm(Some("build"), &["run", "build"]),
    },
    Gate {
        kind: Kind::Process("cargo", &["build", "--release", "-p", "wfd-server"]),
    },
    Gate {
        kind: Kind::Playwright,
    },
    Gate {
        kind: Kind::Xtask("doc-check", &[]),
    },
];

/// Entry point (`cargo xtask ci`).
pub fn run(args: &[String]) -> Result<()> {
    let args: Args = crate::parse_args("ci", args);
    let root = crate::workspace_root();
    let mut env = Env::new(&root);
    let total = GATES.len();
    let mut failures = Vec::new();
    let mut ran = 0;
    for (k, gate) in GATES.iter().enumerate() {
        let label = gate.label();
        let skip = env.skip_reason(gate)?;
        if args.dry_run {
            match &skip {
                None => println!("[{:>2}/{total}] run   {label}", k + 1),
                Some(r) => println!("[{:>2}/{total}] skip  {label}  ({r})", k + 1),
            }
            continue;
        }
        if let Some(r) = skip {
            println!("==> [{:>2}/{total}] {label}: skipped ({r})", k + 1);
            continue;
        }
        println!("==> [{:>2}/{total}] {label}", k + 1);
        ran += 1;
        if let Err(e) = run_gate(&root, gate) {
            eprintln!("==> FAILED: {label}: {e:#}");
            failures.push(label);
            if !args.keep_going {
                break;
            }
        }
    }
    if args.dry_run {
        return Ok(());
    }
    if failures.is_empty() {
        println!("ci: {ran} gate(s) passed, {} skipped", total - ran);
        Ok(())
    } else {
        Err(anyhow!(
            "ci: {} gate(s) failed: {}",
            failures.len(),
            failures.join("; ")
        ))
    }
}

/// Cached facts about which gates exist.
struct Env<'a> {
    root: &'a Path,
    datagen_help: Option<String>,
}

impl<'a> Env<'a> {
    fn new(root: &'a Path) -> Self {
        Env {
            root,
            datagen_help: None,
        }
    }

    /// `None` when the gate exists, else why it is skipped.
    fn skip_reason(&mut self, gate: &Gate) -> Result<Option<String>> {
        Ok(match gate.kind {
            Kind::Process(..) => None,
            Kind::Xtask(name, _) => match crate::command(name) {
                Some(c) if c.run.is_some() => None,
                Some(c) => Some(format!("not implemented yet, owned by {}", c.owner)),
                None => Some("not a registered xtask command".to_owned()),
            },
            Kind::Datagen(sub, _) => {
                if self.datagen_help.is_none() {
                    self.datagen_help = Some(datagen_help(self.root)?);
                }
                let help = self.datagen_help.as_deref().unwrap_or("");
                (!lists_subcommand(help, sub))
                    .then(|| format!("`wfd-datagen --help` does not list `{sub}` yet"))
            }
            Kind::Npm(script, _) => {
                let pkg = self.root.join("frontend/package.json");
                if !pkg.is_file() {
                    Some("frontend/package.json does not exist yet (T12)".to_owned())
                } else if let Some(s) = script {
                    let text = std::fs::read_to_string(&pkg)?;
                    let json: serde_json::Value =
                        serde_json::from_str(&text).context("frontend/package.json")?;
                    json.get("scripts")
                        .and_then(|v| v.get(s))
                        .is_none()
                        .then(|| format!("frontend/package.json has no `{s}` script"))
                } else {
                    None
                }
            }
            Kind::Playwright => {
                let has = ["ts", "js", "mjs", "cjs"].iter().any(|e| {
                    self.root
                        .join(format!("frontend/playwright.config.{e}"))
                        .is_file()
                });
                (!has).then(|| "frontend/playwright.config.* does not exist yet".to_owned())
            }
        })
    }
}

/// Output of `cargo run -q -p wfd-datagen -- --help` (empty when it fails or prints nothing).
fn datagen_help(root: &Path) -> Result<String> {
    let out = Command::new("cargo")
        .args(["run", "-q", "-p", "wfd-datagen", "--", "--help"])
        .current_dir(root)
        .output()
        .context("running wfd-datagen --help")?;
    if !out.status.success() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Does a clap help text list `sub` in its commands section (a line whose first word is `sub`)?
pub fn lists_subcommand(help: &str, sub: &str) -> bool {
    let mut in_commands = false;
    for line in help.lines() {
        if !line.starts_with(' ') {
            in_commands = line.trim_end().eq_ignore_ascii_case("commands:")
                || line.starts_with("SUBCOMMANDS");
            continue;
        }
        if in_commands
            && line
                .split_whitespace()
                .next()
                .map(|w| w.trim_end_matches(','))
                == Some(sub)
        {
            return true;
        }
    }
    false
}

fn run_gate(root: &Path, gate: &Gate) -> Result<()> {
    let status = match gate.kind {
        Kind::Xtask(name, args) => {
            let c = crate::command(name).ok_or_else(|| anyhow!("unknown command {name}"))?;
            let run = c.run.ok_or_else(|| anyhow!("{name} is a stub"))?;
            let args: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
            return run(&args);
        }
        Kind::Process(p, a) => Command::new(p).args(a).current_dir(root).status(),
        Kind::Datagen(s, a) => Command::new("cargo")
            .args(["run", "-p", "wfd-datagen", "--", s])
            .args(a)
            .current_dir(root)
            .status(),
        Kind::Npm(_, a) => Command::new("npm")
            .args(["--prefix", "frontend"])
            .args(a)
            .current_dir(root)
            .status(),
        Kind::Playwright => Command::new("npx")
            .args(["--prefix", "frontend", "playwright", "test"])
            .current_dir(root)
            .status(),
    }
    .with_context(|| format!("starting `{}`", gate.label()))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("exit status {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The gate list is testing.md §10's code block, line for line (one gate per `&&`-separated command).
    #[test]
    fn gates_follow_testing_md_section_10() {
        let text =
            std::fs::read_to_string(crate::workspace_root().join("docs/architecture/testing.md"))
                .unwrap();
        let start = text.find("## 10. CI gates").expect("testing.md §10");
        let block = &text[start..];
        let block = &block[block.find("```text").unwrap() + 7..];
        let block = &block[..block.find("```").unwrap()];
        let expected: Vec<String> = block
            .lines()
            .map(|l| l.split('#').next().unwrap().trim())
            .filter(|l| !l.is_empty())
            .flat_map(|l| {
                l.split("&&")
                    .map(|c| c.trim().to_owned())
                    .collect::<Vec<_>>()
            })
            .collect();
        let labels: Vec<String> = GATES.iter().map(Gate::label).collect();
        assert_eq!(labels, expected);
    }

    #[test]
    fn subcommand_detection() {
        let help = "Usage: wfd-datagen <COMMAND>\n\nCommands:\n  fetch        Fetch sources\n  spec-tables  Ingest\n  \
                    help         Print help\n\nOptions:\n  -h, --help  Print help\n";
        assert!(lists_subcommand(help, "spec-tables"));
        assert!(lists_subcommand(help, "fetch"));
        assert!(!lists_subcommand(help, "check"));
        assert!(!lists_subcommand(help, "-h"));
        assert!(!lists_subcommand("", "check"));
    }

    #[test]
    fn stubs_are_skipped_with_their_owner() {
        let root = crate::workspace_root();
        let mut env = Env::new(&root);
        let r = env
            .skip_reason(&Gate {
                kind: Kind::Xtask("effect-smoke", &[]),
            })
            .unwrap();
        assert_eq!(r.as_deref(), Some("not implemented yet, owned by T19"));
        assert_eq!(
            env.skip_reason(&Gate {
                kind: Kind::Xtask("spec-sync", &["--check"])
            })
            .unwrap(),
            None
        );
        assert_eq!(env.skip_reason(&GATES[0]).unwrap(), None);
    }
}
