//! `cargo xtask doc-check [--strict]` (work-breakdown.md T01c, T37, T41; testing.md §10).
//!
//! Checks that every path, type, command and task id named in `docs/architecture/` and `docs/data/` exists or is
//! listed as planned, and that every document's first line is a status line.
//!
//! - **Status lines**: the first line of every `.md` in `docs/architecture/`, `docs/data/`, `docs/mechanics/` and
//!   `research/findings/` is `Status: draft|reviewed|cross-checked|superseded …`.
//! - **Task ids** (`T01c`, `T20`): defined by a `### Tnn …` heading of work-breakdown.md (`### T20-T29` defines the
//!   range).
//! - **Commands** (in code spans and code blocks): `cargo xtask <cmd>` is a registered xtask command (`main.rs`);
//!   `-p/--package <crate>` is a workspace package; `--test <name>` / `--bench <name>` with `-p <crate>` is checked as
//!   the path `crates/<crate>/tests/<name>.rs` / `benches/<name>.rs`; `wfd-datagen <sub>` and `warframe-dps <sub>`
//!   are subcommands a CLI synopsis defines (a code-block line starting with the binary: pipeline.md §3,
//!   server-api.md §1) or the plan names (checked against the documents only, also under `--strict`).
//!   A bare `Tnn` names the task family `Tnna`, `Tnnb` … when no `### Tnn` exists.
//! - **Paths** (code spans starting with a top-level directory of the repository or naming a root file): exist in the
//!   repository, or are planned. `{a,b}` and `{01..05}` expand, `(x)` is optional, `<name>`, `NN`/`nn` and `*` are
//!   wildcards, a `NN-MM` segment is the document range `[0-9][0-9]-*`. Gitignored local paths (`research/external/`,
//!   `.cache/`, `frontend/node_modules`, `frontend/dist`, `data/cache/`, `target/`) are not checked.
//! - **Types** (code spans whose head is a CamelCase identifier: `CalcReport`, `Profile::Compat`, `Option<u8>`): the
//!   workspace sources define the name ([`defined_names`]: Rust items, enum variants and macro-defined types, lexed
//!   without comments and string literals; TypeScript declarations once the frontend exists), or an [`EXTERNAL`] list
//!   names it (std, third-party crates, the web platform, upstream data keys), or it is planned. A name the sources
//!   only mention does not exist.
//!
//! **Planned** (default mode): a path the plan names (task sections, acceptance commands and the ownership map of §8;
//! a relative span after an absolute one on the same line resolves in its package: `tests/x.rs` after
//! `crates/wfd-engine/src/…` is `crates/wfd-engine/tests/x.rs`, [`resolve_relative`]), an ancestor directory of one, or
//! a path under a directory the plan assigns (`frontend/`, `data/curated/**`) that a module layout tree of the
//! documents lists ([`layout_paths`]: engine.md §2, frontend.md §2, pipeline.md §6.1, overview.md §3) or that is a
//! pattern or a conventional Cargo entry. A path under an assigned directory that nothing names is a **warning**
//! (printed, not a failure: likely a misspelling). A type is planned when work-breakdown.md names it or the design
//! defines it (a code block of the scanned documents, or a prose `Name { … }` / `Name(…)` shape). `--strict` (for the
//! final documentation pass, T41) accepts only what exists.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{Context, Result, anyhow};
use regex::Regex;

/// Directories whose documents' names are checked.
pub const SCANNED_DIRS: &[&str] = &["docs/architecture", "docs/data"];
/// Directories whose documents must start with a status line.
pub const STATUS_DIRS: &[&str] = &[
    "docs/architecture",
    "docs/data",
    "docs/mechanics",
    "research/findings",
];
/// The plan.
pub const PLAN: &str = "docs/architecture/work-breakdown.md";
/// Top-level directories a repository path starts with.
pub const TOP_DIRS: &[&str] = &[
    "crates/",
    "xtask/",
    "data/",
    "frontend/",
    "docs/",
    "research/",
    "tools/",
    ".cargo/",
];
/// Root files checked when named alone.
pub const ROOT_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "CONTRACTS.md",
    "CLAUDE.md",
    "rust-toolchain.toml",
    ".gitignore",
];
/// Local-only (gitignored) paths: never checked, never indexed.
pub const IGNORED: &[&str] = &[
    "research/external",
    ".cache",
    "frontend/node_modules",
    "frontend/dist",
    "data/cache",
    "target",
    ".git",
];

/// Names that are not ours: std and core, third-party crates, the web platform and TypeScript, and the keys of the
/// upstream data sources (wiki Lua modules, Public Export) that docs/data/pipeline.md quotes.
pub const EXTERNAL: &[&str] = &[
    // Rust std / core
    "Option",
    "Some",
    "None",
    "Result",
    "Ok",
    "Err",
    "Vec",
    "Box",
    "String",
    "Arc",
    "Rc",
    "Cow",
    "HashMap",
    "HashSet",
    "BTreeMap",
    "BTreeSet",
    "Mutex",
    "RwLock",
    "PathBuf",
    "Path",
    "Duration",
    "Instant",
    "Ordering",
    "Default",
    "Clone",
    "Copy",
    "Debug",
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "Hash",
    "Send",
    "Sync",
    "Iterator",
    "From",
    "Into",
    "TryFrom",
    "Display",
    "FromStr",
    "AtomicBool",
    "AtomicU64",
    "OnceLock",
    "LazyLock",
    "Infinity",
    // third-party crates
    "FxHashMap",
    "FxHashSet",
    "SmallVec",
    "ArrayVec",
    "ArcSwap",
    "ServiceExt",
    "TraceLayer",
    "EnvFilter",
    "ProjectDirs",
    "Router",
    "Json",
    "Sse",
    "Serialize",
    "Deserialize",
    "JsonSchema",
    "TS",
    // web platform, TypeScript, React
    "AbortSignal",
    "AbortController",
    "Intl",
    "Promise",
    "Record",
    "Partial",
    "Array",
    "EventSource",
    "URL",
    "URLSearchParams",
    "Worker",
    "Map",
    "Set",
    "Number",
    "Math",
    "JSON",
    "Date",
    "Origin", // the HTTP request header (server-api.md §9)
    // upstream data keys (wiki modules, Public Export, WFCD)
    "Ammocost",
    "AttackParentIndex",
    "EximusSHield",
    "ExportAbilities",
    "ExportModSet",
    "ForcedProc",
    "Hitscan",
    "Incompatible",
    "IsSlilent",
    "Multis",
    "ProcResists",
    "UpgradeType",
    "UpgradeTypes",
];

#[derive(clap::Parser, Debug)]
#[command(
    name = "cargo xtask doc-check",
    about = "Check what the architecture and data docs name (T01c)"
)]
struct Args {
    /// Accept only what exists (no "planned" names): the final documentation pass (T41).
    #[arg(long)]
    strict: bool,
    /// Workspace root (default: this workspace).
    #[arg(long)]
    root: Option<PathBuf>,
}

/// Entry point (`cargo xtask doc-check`).
pub fn run(args: &[String]) -> Result<()> {
    let args: Args = crate::parse_args("doc-check", args);
    let root = args.root.unwrap_or_else(crate::workspace_root);
    let report = check(&root, args.strict)?;
    for w in &report.warnings {
        eprintln!("warning: {w}");
    }
    println!("{}", report.summary);
    if report.problems.is_empty() {
        return Ok(());
    }
    for p in &report.problems {
        eprintln!("  - {p}");
    }
    Err(anyhow!("doc-check: {} problem(s)", report.problems.len()))
}

/// Result of a doc-check.
#[derive(Debug, Default)]
pub struct Report {
    /// One line: documents, references per kind, warnings.
    pub summary: String,
    /// `doc:line: kind `name`: reason` (the check fails).
    pub problems: Vec<String>,
    /// Same form: paths covered only by an owned directory ([`Status::Warn`]; the check passes).
    pub warnings: Vec<String>,
}

// ------------------------------------------------------------------------------------------------------------------
// Documents
// ------------------------------------------------------------------------------------------------------------------

/// A Markdown document split into prose code spans and code-block lines.
#[derive(Debug, Default)]
pub struct Doc {
    /// `docs/architecture/engine.md`.
    pub name: String,
    /// Inline code spans outside code blocks, with their line.
    pub spans: Vec<(usize, String)>,
    /// Lines inside fenced code blocks.
    pub code_lines: Vec<(usize, String)>,
    /// Every line.
    pub lines: Vec<(usize, String)>,
}

/// Splits a document into spans and code lines.
pub fn parse_doc(name: &str, text: &str) -> Doc {
    let mut doc = Doc {
        name: name.to_owned(),
        ..Doc::default()
    };
    let mut in_code = false;
    for (k, line) in text.lines().enumerate() {
        let n = k + 1;
        doc.lines.push((n, line.to_owned()));
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            doc.code_lines.push((n, line.to_owned()));
        } else {
            doc.spans
                .extend(code_spans(line).into_iter().map(|s| (n, s)));
        }
    }
    doc
}

/// Inline code spans of a line (single backticks; a span does not cross lines).
pub fn code_spans(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(s) = rest.find('`') {
        let after = &rest[s + 1..];
        let Some(e) = after.find('`') else { break };
        if e > 0 {
            out.push(after[..e].to_owned());
        }
        rest = &after[e + 1..];
    }
    out
}

fn md_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if dir.is_dir() {
        for e in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
            let p = e?.path();
            if p.extension().is_some_and(|x| x == "md") {
                out.push(p);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

// ------------------------------------------------------------------------------------------------------------------
// Paths and globs
// ------------------------------------------------------------------------------------------------------------------

/// Every file and directory of the repository (relative, `/`-separated), ignored paths excluded.
pub fn index_repo(root: &Path) -> Result<BTreeSet<String>> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
            let p = e?.path();
            let r = rel(root, &p);
            if IGNORED
                .iter()
                .any(|i| r == *i || r.starts_with(&format!("{i}/")))
                || r.contains("node_modules")
            {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            }
            out.insert(r);
        }
    }
    Ok(out)
}

/// Expands `{a,b}`, `{01..05}` and an optional `(x)` suffix group.
pub fn expand(p: &str) -> Vec<String> {
    static OPT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\(([A-Za-z0-9_.-]+)\)").expect("static regex"));
    let p = OPT.replace_all(p, "{,$1}").into_owned();
    let Some(s) = p.find('{') else { return vec![p] };
    let Some(e) = p[s..].find('}').map(|e| s + e) else {
        return vec![p];
    };
    let mut alts = Vec::new();
    for a in p[s + 1..e].split(',') {
        match a.split_once("..") {
            Some((x, y)) if x.parse::<u32>().is_ok() && y.parse::<u32>().is_ok() => {
                let (x, y) = (x.parse::<u32>().unwrap_or(0), y.parse::<u32>().unwrap_or(0));
                alts.extend((x..=y).map(|i| {
                    format!(
                        "{i:0w$}",
                        w = a.split_once("..").map_or(0, |(x, _)| x.len())
                    )
                }));
            }
            _ => alts.push(a.to_owned()),
        }
    }
    alts.into_iter()
        .flat_map(|a| expand(&format!("{}{a}{}", &p[..s], &p[e + 1..])))
        .collect()
}

/// Turns placeholders into glob wildcards: `<name>` → `*`, `NN`/`nn` → `*`, a `NN-MM` segment → `[0-9][0-9]-*`.
pub fn to_glob(p: &str) -> String {
    static ANGLE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"<[^<>]*>").expect("static regex"));
    static NN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\b(NN|nn)\b").expect("static regex"));
    static RANGE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(^|/)\d\d-\d\d$").expect("static regex"));
    let p = ANGLE.replace_all(p, "*");
    let p = NN.replace_all(&p, "*");
    let p = RANGE.replace_all(&p, "${1}[0-9][0-9]-*");
    p.trim_end_matches('/').to_owned()
}

fn has_wildcard(p: &str) -> bool {
    p.contains(['*', '?', '['])
}

/// Glob match of one path segment (`*`, `?`, `[a-z0-9]`).
fn seg_match(pat: &[char], s: &[char]) -> bool {
    match pat.first() {
        None => s.is_empty(),
        Some('*') => (0..=s.len()).any(|k| seg_match(&pat[1..], &s[k..])),
        Some('?') => !s.is_empty() && seg_match(&pat[1..], &s[1..]),
        Some('[') => {
            let Some(close) = pat.iter().position(|&c| c == ']') else {
                return s.first() == Some(&'[') && seg_match(&pat[1..], &s[1..]);
            };
            let Some(&c) = s.first() else { return false };
            let class = &pat[1..close];
            let mut ok = false;
            let mut k = 0;
            while k < class.len() {
                if k + 2 < class.len() && class[k + 1] == '-' {
                    ok |= class[k] <= c && c <= class[k + 2];
                    k += 3;
                } else {
                    ok |= class[k] == c;
                    k += 1;
                }
            }
            ok && seg_match(&pat[close + 1..], &s[1..])
        }
        Some(&p) => s.first() == Some(&p) && seg_match(&pat[1..], &s[1..]),
    }
}

fn segs(p: &str) -> Vec<Vec<char>> {
    p.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().collect())
        .collect()
}

/// Segment-wise glob match; `ancestor`: the pattern may also match a leading part (an ancestor directory) of `path`.
fn match_segs(pat: &[Vec<char>], path: &[Vec<char>], ancestor: bool) -> bool {
    match pat.first() {
        None => path.is_empty() || ancestor,
        Some(s) if s.as_slice() == ['*', '*'] => {
            (0..=path.len()).any(|k| match_segs(&pat[1..], &path[k..], ancestor))
        }
        Some(s) => {
            !path.is_empty()
                && seg_match(s, &path[0])
                && match_segs(&pat[1..], &path[1..], ancestor)
        }
    }
}

/// Does `path` match the glob `pat` (segments; `**` spans any number of segments)?
pub fn glob_match(pat: &str, path: &str) -> bool {
    match_segs(&segs(pat), &segs(path), false)
}

/// Does `pat` match `path` or one of its ancestor directories?
pub fn glob_ancestor(pat: &str, path: &str) -> bool {
    match_segs(&segs(pat), &segs(path), true)
}

/// What the plan and the design documents name about paths that do not exist yet.
#[derive(Debug, Default)]
pub struct PlannedPaths {
    /// Paths (globs) the plan names: brace expansions, `<name>`/`NN` wildcards, relative spans resolved in their
    /// package ([`resolve_relative`]).
    pub paths: BTreeSet<String>,
    /// Directories the plan assigns to a task (spans ending in `/` or `/**`: `frontend/`, `data/curated/**`).
    pub dirs: BTreeSet<String>,
    /// Paths and directories the module layout trees of the documents name ([`layout_paths`]).
    pub layout: BTreeSet<String>,
}

impl PlannedPaths {
    /// The status of a (glob) path that does not exist: [`Status::Planned`] when the plan names it (or it is an
    /// ancestor directory of a named path or an owned directory) or when it lies under an owned directory and a layout
    /// tree names it (or it is a pattern, or a conventional Cargo package entry: [`is_package_convention`]);
    /// [`Status::Warn`] when it lies under an owned directory but nothing names it
    /// (a likely misspelling); else [`Status::Unknown`].
    pub fn status(&self, p: &str) -> Status {
        let named = |set: &BTreeSet<String>| {
            set.iter()
                .any(|q| glob_match(q, p) || glob_match(p, q) || glob_ancestor(p, q))
        };
        if named(&self.paths) || self.dirs.iter().any(|d| glob_ancestor(p, d)) {
            return Status::Planned;
        }
        let Some(dir) = self.dirs.iter().find(|d| glob_ancestor(d, p)) else {
            return Status::Unknown(format!(
                "does not exist and {PLAN} does not plan it (no task names or owns it)"
            ));
        };
        if named(&self.layout) || has_wildcard(p) || is_package_convention(p) {
            Status::Planned
        } else {
            Status::Warn(format!(
                "does not exist; it lies under `{dir}/`, which {PLAN} assigns to a task, but no task names it and no \
                 layout tree of the documents lists it (misspelt?)"
            ))
        }
    }
}

/// A conventional entry of a Cargo package: `<pkg>/{src,tests,benches,examples}` directories, `<pkg>/build.rs`,
/// `<pkg>/Cargo.toml`, `<pkg>/src/lib.rs`, `<pkg>/src/main.rs`.
pub fn is_package_convention(p: &str) -> bool {
    let Some(pkg) = package_root(p) else {
        return false;
    };
    let rest = p[pkg.len()..].trim_matches('/');
    matches!(
        rest,
        "src"
            | "tests"
            | "benches"
            | "examples"
            | "build.rs"
            | "Cargo.toml"
            | "src/lib.rs"
            | "src/main.rs"
    )
}

fn is_repo_path(s: &str) -> bool {
    !s.contains(char::is_whitespace)
        && !s.contains("://")
        && (TOP_DIRS.iter().any(|t| s.starts_with(t)) || ROOT_FILES.contains(&s))
}

fn is_ignored(p: &str) -> bool {
    IGNORED
        .iter()
        .any(|i| p == *i || p.starts_with(&format!("{i}/")))
        || p.contains("node_modules")
}

fn looks_relative_path(s: &str) -> bool {
    static REL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[a-z_{][A-Za-z0-9_{},./*-]*(/|\.(rs|toml|json|ts|tsx|md))$|^[a-z_{][A-Za-z0-9_{},.*-]*/[A-Za-z0-9_{},./*-]+$")
            .expect("static regex")
    });
    !s.contains(char::is_whitespace) && REL.is_match(s)
}

/// The package directory of a repository path: `crates/<name>`, `tools/<name>`, `xtask` or `frontend`.
pub fn package_root(p: &str) -> Option<String> {
    let segs: Vec<&str> = p.split('/').collect();
    match segs.as_slice() {
        ["crates" | "tools", name, ..] if !name.is_empty() => Some(format!("{}/{name}", segs[0])),
        ["xtask" | "frontend", ..] => Some(segs[0].to_owned()),
        _ => None,
    }
}

/// Resolves a relative span against the absolute span before it on the same plan line (`base`: its directory): a
/// package-level name (`tests/…`, `benches/…`, `examples/…`, `src/…`, `build.rs`, `Cargo.toml`, `package.json`) in
/// the package root, any other name in the package's `src/` when the base lies there (`main.rs`, `resolve/firing.rs`,
/// `effect_smoke.rs`), else in `base` itself.
pub fn resolve_relative(base: &str, rel: &str) -> String {
    const PACKAGE_LEVEL: &[&str] = &[
        "tests/",
        "benches/",
        "examples/",
        "src/",
        "e2e/",
        "build.rs",
        "Cargo.toml",
        "package.json",
    ];
    match package_root(base) {
        Some(pkg) if PACKAGE_LEVEL.iter().any(|l| rel.starts_with(l)) => format!("{pkg}/{rel}"),
        Some(pkg) if base == format!("{pkg}/src") || base.starts_with(&format!("{pkg}/src/")) => {
            format!("{pkg}/src/{rel}")
        }
        _ => format!("{base}/{rel}"),
    }
}

/// Collects the planned paths: inline spans of the plan that are repository paths (a parenthesised remark inside a
/// span, `{compile/ (except melee.rs),stats.rs}`, is dropped), relative spans resolved against the last absolute span
/// on the same line ([`resolve_relative`]), and the layout trees of `design` (every scanned document). Spans ending in
/// `/` or `/**` are owned directories.
pub fn planned_paths(plan: &Doc, design: &[Doc]) -> PlannedPaths {
    static REMARK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\s+\([^()]*\)").expect("static regex"));
    let mut out = PlannedPaths::default();
    let mut by_line: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for (n, s) in &plan.spans {
        by_line
            .entry(*n)
            .or_default()
            .push(REMARK.replace_all(s, "").into_owned());
    }
    for spans in by_line.values() {
        let mut base: Option<String> = None;
        for s in spans {
            let cand = if is_repo_path(s) {
                let first = expand(s).into_iter().next().unwrap_or_default();
                let dir = if first.ends_with('/') || first.ends_with("/**") {
                    first
                        .trim_end_matches("/**")
                        .trim_end_matches('/')
                        .to_owned()
                } else {
                    first
                        .rsplit_once('/')
                        .map(|(d, _)| d.to_owned())
                        .unwrap_or_default()
                };
                base = Some(dir);
                s.clone()
            } else if let (Some(b), true) = (&base, looks_relative_path(s)) {
                resolve_relative(b, s)
            } else {
                continue;
            };
            for e in expand(&cand) {
                let g = to_glob(&e);
                if e.ends_with('/') || e.ends_with("/**") {
                    out.dirs.insert(g.trim_end_matches("/**").to_owned());
                } else {
                    out.paths.insert(g);
                }
            }
        }
    }
    // Test and bench targets of the plan's commands (`cargo test -p wfd-model --test fixtures`).
    for (_, l) in plan.spans.iter().chain(plan.code_lines.iter()) {
        out.paths.extend(cargo_target_paths(l));
    }
    for d in design {
        out.layout.extend(layout_paths(d));
    }
    out
}

static PKG_ARG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|\s)(?:-p|--package)\s+([A-Za-z0-9_-]+)").expect("static regex")
});
static TARGET_ARG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|\s)--(test|bench)\s+([A-Za-z0-9_]+)").expect("static regex")
});

/// The files of the `--test <name>` / `--bench <name>` targets of a `cargo … -p <crate>` command
/// (`crates/<crate>/tests/<name>.rs`, `…/benches/<name>.rs`); the text after ` # ` is a comment.
pub fn cargo_target_paths(command: &str) -> Vec<String> {
    let command = command.split(" # ").next().unwrap_or(command);
    if !command.contains("cargo ") {
        return Vec::new();
    }
    let Some(krate) = PKG_ARG.captures(command).map(|c| c[1].to_owned()) else {
        return Vec::new();
    };
    TARGET_ARG
        .captures_iter(command)
        .map(|c| {
            let dir = if &c[1] == "test" { "tests" } else { "benches" };
            format!("crates/{krate}/{dir}/{}.rs", &c[2])
        })
        .collect()
}

/// The paths named by the module layout trees in a document's code blocks (engine.md §2, frontend.md §2,
/// pipeline.md §6.1, overview.md §3). A tree starts at a code-block line whose first entry is a repository directory
/// (`crates/wfd-engine/src/`, `data/curated/`, `warframe-dps/` for the root); each later entry belongs to the nearest
/// directory entry indented less (box-drawing characters count as indentation). An entry line holds a directory
/// (`resolve/`) or file names (`base.rs`, `weapon_mod/<family>/<slug>.toml`, `generated/*.ts`), a directory followed
/// by its files on the same line, or CamelCase component names (`ModGrid` → `ModGrid.*`); parenthesised remarks
/// (which may span lines) and `# comments` are ignored, and other words are description.
pub fn layout_paths(doc: &Doc) -> BTreeSet<String> {
    static FILE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[A-Za-z0-9_.<>{}*-]+(/[A-Za-z0-9_.<>{}*-]+)*\.(rs|ts|tsx|js|mjs|cjs|json|toml|md|html|css|svg|txt|lock)$")
            .expect("static regex")
    });
    static DIR: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[A-Za-z0-9_.<>{}*-]+(/[A-Za-z0-9_.<>{}*-]+)*/$").expect("static regex")
    });
    static COMPONENT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[A-Z][a-z0-9]+(?:[A-Z][A-Za-z0-9]*)+$").expect("static regex")
    });
    let is_box = |c: char| {
        matches!(
            c,
            '│' | '├' | '└' | '─' | '┬' | '┴' | '┼' | '┌' | '┐' | '┘' | '┤'
        )
    };
    let mut out = BTreeSet::new();
    let add = |out: &mut BTreeSet<String>, p: &str| {
        for e in expand(p) {
            out.insert(to_glob(&e));
        }
    };
    // Code blocks, as runs of consecutive code lines.
    let mut blocks: Vec<Vec<&(usize, String)>> = Vec::new();
    for l in &doc.code_lines {
        match blocks.last_mut() {
            Some(b) if b.last().is_some_and(|p| p.0 + 1 == l.0) => b.push(l),
            _ => blocks.push(vec![l]),
        }
    }
    for block in blocks {
        let mut stack: Vec<(usize, String)> = Vec::new(); // (column, directory with a trailing `/`, "" = root)
        let mut depth = 0usize; // open parentheses, across lines
        for (_, line) in block {
            let col = line.chars().take_while(|&c| c == ' ' || is_box(c)).count();
            // The entry text: box characters, `# comments` and parenthesised remarks removed.
            let mut text = String::new();
            for c in line.chars().skip(col) {
                match c {
                    '(' => depth += 1,
                    ')' => depth = depth.saturating_sub(1),
                    '#' if depth == 0 => break,
                    _ if depth == 0 => text.push(if is_box(c) { ' ' } else { c }),
                    _ => {}
                }
            }
            let words: Vec<&str> = text
                .split(|c: char| c.is_whitespace() || c == ',')
                .filter(|w| !w.is_empty())
                .collect();
            let Some(first) = words.first() else { continue };
            while stack.last().is_some_and(|(c, _)| *c >= col) {
                stack.pop();
            }
            let parent = match stack.last() {
                Some((_, d)) => d.clone(),
                None if *first == "warframe-dps/" => {
                    stack.push((col, String::new()));
                    continue;
                }
                None if DIR.is_match(first) && is_repo_path(first) => {
                    stack.push((col, (*first).to_owned()));
                    add(&mut out, first.trim_end_matches('/'));
                    continue;
                }
                None => continue,
            };
            let mut dir = parent.clone();
            for (k, w) in words.iter().enumerate() {
                if DIR.is_match(w) {
                    add(&mut out, &format!("{parent}{}", w.trim_end_matches('/')));
                    if k == 0 {
                        dir = format!("{parent}{w}");
                        stack.push((col, dir.clone()));
                    }
                } else if FILE.is_match(w) {
                    add(&mut out, &format!("{dir}{w}"));
                } else if COMPONENT.is_match(w) {
                    add(&mut out, &format!("{dir}{w}.*"));
                }
            }
        }
    }
    out
}

// ------------------------------------------------------------------------------------------------------------------
// The check
// ------------------------------------------------------------------------------------------------------------------

static TASK_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bT\d{2}[a-z]?\b").expect("static regex"));
static TYPE_HEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:[a-z_][a-z0-9_-]*::)*([A-Z][a-z0-9]+(?:[A-Z][A-Za-z0-9]*)*)(?:$|::|<|\s*\{|\(|\[|\s)",
    )
    .expect("static regex")
});
static DEF_SHAPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Z][A-Za-z0-9]*)\s*[{(]").expect("static regex"));
static IDENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[A-Z][A-Za-z0-9_]*\b").expect("static regex"));

/// Task ids defined by the plan's `### Tnn` headings (a `Tnn-Tmm` heading defines the range).
pub fn planned_tasks(plan: &Doc) -> BTreeSet<String> {
    let re = Regex::new(r"^###\s+(T(\d{2})[a-z]?)(?:-T(\d{2}))?\b").expect("static regex");
    let mut out = BTreeSet::new();
    for (_, line) in &plan.lines {
        if let Some(c) = re.captures(line) {
            out.insert(c[1].to_owned());
            if let (Ok(a), Some(Ok(b))) = (
                c[2].parse::<u32>(),
                c.get(3).map(|m| m.as_str().parse::<u32>()),
            ) {
                out.extend((a..=b).map(|i| format!("T{i:02}")));
            }
        }
    }
    out
}

/// Names the workspace sources define ([`rust_definitions`] for Rust under `crates/` and `xtask/`,
/// [`ts_definitions`] for TypeScript under `frontend/src/`). A name that is only mentioned (in a comment, a string
/// literal, a use or a call) does not count.
pub fn defined_names(root: &Path, index: &BTreeSet<String>) -> Result<BTreeSet<String>> {
    let mut out = BTreeSet::new();
    for p in index {
        let rust = (p.starts_with("crates/") || p.starts_with("xtask/")) && p.ends_with(".rs");
        let ts = p.starts_with("frontend/src/") && (p.ends_with(".ts") || p.ends_with(".tsx"));
        if rust || ts {
            let text =
                std::fs::read_to_string(root.join(p)).with_context(|| format!("reading {p}"))?;
            if rust {
                rust_definitions(&text, &mut out);
            } else {
                ts_definitions(&text, &mut out);
            }
        }
    }
    Ok(out)
}

/// The names a Rust file defines, from its tokens without comments and string literals: the names after `struct`,
/// `enum`, `union`, `trait`, `type`, `fn`, `const`, `static`, `mod` and `macro_rules!`, enum variants, `use … as Name`
/// aliases, and the first identifier of an item-level macro invocation (`sub_enum!(PhysType { … })`,
/// `rule_file!(MultGroupsFile { … })`), which is how wfd-model defines some types.
pub fn rust_definitions(src: &str, out: &mut BTreeSet<String>) {
    use crate::lint_engine::{Tok, lex};
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Frame {
        Enum,
        Mod,
        Other,
    }
    let toks: Vec<Tok> = lex(src)
        .into_iter()
        .map(|(_, t)| t)
        .filter(|t| !t.is_comment())
        .collect();
    let ident = |k: usize| match toks.get(k) {
        Some(Tok::Ident(s)) => Some(s.as_str()),
        _ => None,
    };
    // Brace frames with the open parentheses/brackets inside each.
    let mut frames: Vec<(Frame, i32)> = Vec::new();
    let mut next_frame = Frame::Other;
    let mut expect_variant = false;
    let mut k = 0;
    while k < toks.len() {
        let in_enum_list = matches!(frames.last(), Some((Frame::Enum, 0)));
        match &toks[k] {
            Tok::Punct('#') if expect_variant => {
                // An attribute before a variant: skip it.
                let mut depth = 0;
                while k < toks.len() {
                    match toks[k] {
                        Tok::Punct('[') => depth += 1,
                        Tok::Punct(']') => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    k += 1;
                }
                k += 1;
                continue;
            }
            Tok::Ident(name) if expect_variant => {
                out.insert(name.clone());
                expect_variant = false;
            }
            Tok::Ident(kw)
                if matches!(
                    kw.as_str(),
                    "struct"
                        | "enum"
                        | "union"
                        | "trait"
                        | "type"
                        | "fn"
                        | "const"
                        | "static"
                        | "mod"
                        | "as"
                ) =>
            {
                if let Some(name) = ident(k + 1) {
                    out.insert(name.to_owned());
                    match kw.as_str() {
                        "enum" => next_frame = Frame::Enum,
                        "mod" => next_frame = Frame::Mod,
                        "struct" | "union" | "trait" | "fn" => next_frame = Frame::Other,
                        _ => {}
                    }
                }
            }
            Tok::Ident(m)
                if toks.get(k + 1) == Some(&Tok::Punct('!'))
                    && matches!(frames.last(), None | Some((Frame::Mod, _))) =>
            {
                if m == "macro_rules" {
                    if let Some(name) = ident(k + 2) {
                        out.insert(name.to_owned());
                    }
                } else if matches!(toks.get(k + 2), Some(Tok::Punct('(' | '{' | '['))) {
                    // The first identifier of the arguments, after attributes.
                    let mut j = k + 3;
                    while toks.get(j) == Some(&Tok::Punct('#')) {
                        let mut depth = 0;
                        while j < toks.len() {
                            match toks[j] {
                                Tok::Punct('[') => depth += 1,
                                Tok::Punct(']') => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                _ => {}
                            }
                            j += 1;
                        }
                        j += 1;
                    }
                    if let Some(name) = ident(j).filter(|n| n.starts_with(char::is_uppercase)) {
                        out.insert(name.to_owned());
                    }
                }
            }
            Tok::Punct('{') => {
                expect_variant = next_frame == Frame::Enum;
                frames.push((next_frame, 0));
                next_frame = Frame::Other;
                k += 1;
                continue;
            }
            Tok::Punct('}') => {
                frames.pop();
            }
            Tok::Punct('(' | '[') => {
                if let Some(f) = frames.last_mut() {
                    f.1 += 1;
                }
            }
            Tok::Punct(')' | ']') => {
                if let Some(f) = frames.last_mut() {
                    f.1 -= 1;
                }
            }
            Tok::Punct(',') if in_enum_list => {
                expect_variant = true;
                k += 1;
                continue;
            }
            Tok::Punct(';') => next_frame = Frame::Other,
            _ => {}
        }
        if !matches!(toks[k], Tok::Ident(_)) {
            expect_variant = false;
        }
        k += 1;
    }
}

/// The names a TypeScript file declares (`type`, `interface`, `class`, `enum`, `function`, `const`, `let`, `var`,
/// `namespace`), comments removed.
pub fn ts_definitions(src: &str, out: &mut BTreeSet<String>) {
    static COMMENT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/|//[^\n]*").expect("static regex"));
    static DECL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\b(?:type|interface|class|enum|function|const|let|var|namespace)\s+([A-Za-z_$][A-Za-z0-9_$]*)")
            .expect("static regex")
    });
    let code = COMMENT.replace_all(src, " ");
    out.extend(DECL.captures_iter(&code).map(|c| c[1].to_owned()));
}

fn workspace_packages(root: &Path) -> Result<BTreeSet<String>> {
    let text = std::fs::read_to_string(root.join("Cargo.toml")).context("reading Cargo.toml")?;
    let members = Regex::new(r#""((?:crates/)?[A-Za-z0-9_-]+)""#).expect("static regex");
    let start = text.find("members").unwrap_or(0);
    let end = text[start..].find(']').map_or(text.len(), |e| start + e);
    let package_name = Regex::new(r#"(?m)^name\s*=\s*"([^"]+)""#).expect("static regex");
    let mut out = BTreeSet::new();
    for c in members.captures_iter(&text[start..end]) {
        let dir = root.join(&c[1]);
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        let name = package_name.captures(&manifest).map(|m| m[1].to_owned());
        out.insert(name.unwrap_or_else(|| c[1].rsplit('/').next().unwrap_or("").to_owned()));
    }
    Ok(out)
}

/// Runs every check on the workspace at `root`.
pub fn check(root: &Path, strict: bool) -> Result<Report> {
    let mut problems = Vec::new();

    // Status lines.
    let status_line =
        Regex::new(r"^Status: (draft|reviewed|cross-checked|superseded)\b").expect("static regex");
    let mut n_status = 0;
    for dir in STATUS_DIRS {
        for f in md_files(&root.join(dir))? {
            n_status += 1;
            let text =
                std::fs::read_to_string(&f).with_context(|| format!("reading {}", f.display()))?;
            let first = text.lines().next().unwrap_or("");
            if !status_line.is_match(first) {
                problems.push(format!(
                    "{}:1: the first line is not a status line (`Status: draft|reviewed|cross-checked|superseded …`): \
                     `{first}`",
                    rel(root, &f)
                ));
            }
        }
    }

    let index = index_repo(root)?;
    let plan_text =
        std::fs::read_to_string(root.join(PLAN)).with_context(|| format!("reading {PLAN}"))?;
    let plan = parse_doc(PLAN, &plan_text);
    let tasks = planned_tasks(&plan);
    let packages = workspace_packages(root)?;
    let code_idents = defined_names(root, &index)?;
    let mut docs = Vec::new();
    for dir in SCANNED_DIRS {
        for f in md_files(&root.join(dir))? {
            let text =
                std::fs::read_to_string(&f).with_context(|| format!("reading {}", f.display()))?;
            docs.push(parse_doc(&rel(root, &f), &text));
        }
    }
    let planned = planned_paths(&plan, &docs);
    // Names the plan and the design define.
    let mut plan_words: BTreeSet<String> = BTreeSet::new();
    for (_, s) in &plan.spans {
        plan_words.extend(IDENT.find_iter(s).map(|m| m.as_str().to_owned()));
    }
    let mut design: BTreeSet<String> = BTreeSet::new();
    let mut plan_subs: BTreeSet<(String, String)> = BTreeSet::new();
    let sub_re = Regex::new(r"\b(wfd-datagen|warframe-dps)\s+(?:--\s+)?([a-z][a-z-]*)")
        .expect("static regex");
    for d in &docs {
        for (_, l) in &d.code_lines {
            design.extend(IDENT.find_iter(l).map(|m| m.as_str().to_owned()));
        }
        for (_, s) in &d.spans {
            design.extend(DEF_SHAPE.captures_iter(s).map(|c| c[1].to_owned()));
        }
    }
    // Subcommands of our binaries: the CLI synopses (code-block lines starting with the binary, `[serve]` included)
    // and the plan define them.
    let synopsis =
        Regex::new(r"^\s*(wfd-datagen|warframe-dps)\s+\[?([a-z][a-z-]*)").expect("static regex");
    for d in &docs {
        for (_, l) in &d.code_lines {
            if let Some(c) = synopsis.captures(l) {
                plan_subs.insert((c[1].to_owned(), c[2].to_owned()));
            }
        }
    }
    for (_, l) in plan.spans.iter().chain(plan.code_lines.iter()) {
        for c in sub_re.captures_iter(l) {
            plan_subs.insert((c[1].to_owned(), c[2].to_owned()));
        }
    }

    let mut counts = BTreeMap::<&str, (usize, usize)>::new(); // kind → (checked, planned-only)
    let mut seen = BTreeSet::new();
    let mut warnings = Vec::new();
    let mut report = |doc: &str, line: usize, kind: &'static str, name: &str, ok: Status| {
        let e = counts.entry(kind).or_default();
        e.0 += 1;
        match ok {
            Status::Exists => {}
            Status::Planned if !strict => e.1 += 1,
            Status::Warn(why) if !strict => {
                e.1 += 1;
                if seen.insert((kind, name.to_owned())) {
                    warnings.push(format!("{doc}:{line}: {kind} `{name}`: {why}"));
                }
            }
            Status::Planned | Status::Warn(_) => {
                if seen.insert((kind, name.to_owned())) {
                    problems.push(format!(
                        "{doc}:{line}: {kind} `{name}` does not exist yet (--strict)"
                    ));
                }
            }
            Status::Unknown(why) => {
                if seen.insert((kind, name.to_owned())) {
                    problems.push(format!("{doc}:{line}: {kind} `{name}`: {why}"));
                }
            }
        }
    };

    let xtask_re = Regex::new(r"\bxtask\s+([a-z][a-z0-9-]*)").expect("static regex");
    for d in &docs {
        // Task ids: every line.
        for (n, line) in &d.lines {
            for m in TASK_ID.find_iter(line) {
                // A bare `T01` names the family T01a, T01b, … when no `### T01` exists.
                let family =
                    m.as_str().len() == 3 && tasks.iter().any(|t| t.starts_with(m.as_str()));
                let ok = if tasks.contains(m.as_str()) || family {
                    Status::Exists
                } else {
                    Status::Unknown(format!("no `### {}` task in {PLAN}", m.as_str()))
                };
                report(&d.name, *n, "task id", m.as_str(), ok);
            }
        }
        // Commands: code spans and code-block lines.
        for (n, text) in d.spans.iter().chain(d.code_lines.iter()) {
            let text = text.split(" # ").next().unwrap_or(text);
            for c in xtask_re.captures_iter(text) {
                let name = &c[1];
                let ok = match crate::command(name) {
                    Some(cmd) if cmd.run.is_some() => Status::Exists,
                    Some(_) => Status::Planned,
                    None => Status::Unknown(
                        "not a registered `cargo xtask` command (xtask/src/main.rs)".to_owned(),
                    ),
                };
                report(&d.name, *n, "command", &format!("cargo xtask {name}"), ok);
            }
            if text.contains("cargo ") {
                for c in PKG_ARG.captures_iter(text) {
                    let ok = if packages.contains(&c[1]) {
                        Status::Exists
                    } else {
                        Status::Unknown("not a workspace package".to_owned())
                    };
                    report(&d.name, *n, "package", &c[1], ok);
                }
                for p in cargo_target_paths(text) {
                    let ok = path_status(&p, &index, &planned);
                    report(&d.name, *n, "path", &p, ok);
                }
            }
            for c in sub_re.captures_iter(text) {
                let key = (c[1].to_owned(), c[2].to_owned());
                let ok = if plan_subs.contains(&key) {
                    Status::Planned
                } else {
                    Status::Unknown(format!(
                        "neither a CLI synopsis (a code-block line starting with `{}`) nor {PLAN} defines it",
                        &c[1]
                    ))
                };
                // Subcommands of the binaries are checked against the plan only: report them as planned.
                let ok = if strict && matches!(ok, Status::Planned) {
                    Status::Exists
                } else {
                    ok
                };
                report(&d.name, *n, "command", &format!("{} {}", &c[1], &c[2]), ok);
            }
        }
        // Paths and types: prose code spans.
        for (n, s) in &d.spans {
            if is_repo_path(s) {
                if is_ignored(s.trim_start_matches('/')) {
                    continue;
                }
                for e in expand(s) {
                    let ok = path_status(&e, &index, &planned);
                    report(&d.name, *n, "path", &e, ok);
                }
            } else if let Some(c) = TYPE_HEAD.captures(s) {
                let name = &c[1];
                let ok = if code_idents.contains(name) || EXTERNAL.contains(&name) {
                    Status::Exists
                } else if design.contains(name) || plan_words.contains(name) {
                    Status::Planned
                } else {
                    Status::Unknown(
                        "named nowhere else: not in the workspace sources, a design code block, the plan, or the \
                         external list of doc_check.rs"
                            .to_owned(),
                    )
                };
                report(&d.name, *n, "type", name, ok);
            }
        }
    }
    let mut parts = Vec::new();
    for (kind, (checked, planned_only)) in &counts {
        parts.push(format!(
            "{checked} {kind} reference(s) ({planned_only} planned, not yet present)"
        ));
    }
    let summary = format!(
        "doc-check{}: {} status line(s); {} document(s) of {}: {}; {} warning(s)",
        if strict { " --strict" } else { "" },
        n_status,
        docs.len(),
        SCANNED_DIRS.join(", "),
        parts.join(", "),
        warnings.len()
    );
    Ok(Report {
        summary,
        problems,
        warnings,
    })
}

/// Outcome of one name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// It exists in the repository (or is external).
    Exists,
    /// It does not exist yet and the plan or the design names it (a problem under `--strict`).
    Planned,
    /// It does not exist and only an owned directory covers it: a warning (a problem under `--strict`).
    Warn(String),
    /// It does not exist and nothing plans it: a problem.
    Unknown(String),
}

/// Whether a (possibly wildcard) path exists in the index, is planned, or neither ([`PlannedPaths::status`]).
pub fn path_status(p: &str, index: &BTreeSet<String>, planned: &PlannedPaths) -> Status {
    let g = to_glob(p);
    let exists = if has_wildcard(&g) {
        index.iter().any(|i| glob_match(&g, i))
    } else {
        index.contains(&g)
    };
    if exists {
        Status::Exists
    } else {
        planned.status(&g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_blocks() {
        let d = parse_doc(
            "x.md",
            "a `b` c `d e`\n```text\n`not a span` T01a\n```\n`f`",
        );
        assert_eq!(
            d.spans,
            vec![(1, "b".into()), (1, "d e".into()), (5, "f".into())]
        );
        assert_eq!(d.code_lines, vec![(3, "`not a span` T01a".into())]);
    }

    #[test]
    fn expansion_and_globs() {
        assert_eq!(expand("a/{b,c}.rs"), ["a/b.rs", "a/c.rs"]);
        assert_eq!(
            expand("PAR-{01,03..05}-*.ts"),
            ["PAR-01-*.ts", "PAR-03-*.ts", "PAR-04-*.ts", "PAR-05-*.ts"]
        );
        assert_eq!(
            expand("src/**/*.test.ts(x)"),
            ["src/**/*.test.ts", "src/**/*.test.tsx"]
        );
        assert_eq!(to_glob("data/x/<task>.toml"), "data/x/*.toml");
        assert_eq!(
            to_glob("tests/spec_vectors/NN.toml"),
            "tests/spec_vectors/*.toml"
        );
        assert_eq!(
            to_glob("docs/mechanics/00-09"),
            "docs/mechanics/[0-9][0-9]-*"
        );
        assert_eq!(
            to_glob("frontend/e2e/parity/PAR-nn-*.spec.ts"),
            "frontend/e2e/parity/PAR-*-*.spec.ts"
        );
        assert!(glob_match(
            "docs/mechanics/0[1-9]-*.md",
            "docs/mechanics/04-x.md"
        ));
        assert!(!glob_match(
            "docs/mechanics/0[1-9]-*.md",
            "docs/mechanics/00-x.md"
        ));
        assert!(glob_match("crates/**/x.rs", "crates/a/b/x.rs"));
        assert!(glob_match("crates/**/x.rs", "crates/x.rs"));
        assert!(glob_ancestor(
            "crates/*/benches",
            "crates/wfd-engine/benches/mc.rs"
        ));
        assert!(glob_ancestor("crates/**", "crates"));
        assert!(!glob_ancestor("crates/*/benches", "crates/wfd-engine"));
        assert!(!glob_match(
            "crates/*/benches",
            "crates/wfd-engine/benches/mc.rs"
        ));
    }

    fn planned(p: &PlannedPaths, path: &str) -> bool {
        p.status(&to_glob(path)) == Status::Planned
    }

    #[test]
    fn plan_paths_resolve_relative_spans() {
        let plan = parse_doc(
            PLAN,
            "| `crates/wfd-engine/src/{lib,outputs}.rs`, `tests/{compat,golden_oracle}.rs` | T17 |\n\
             | `crates/wfd-engine/src/ev/{bound,ttk}.rs`, `tests/properties.rs`, `benches/{steady,window}.rs` | T15c |\n\
             | `crates/wfd-engine/src/schedule/{mod,ranged}.rs`, `resolve/firing.rs` | T08a |\n\
             | `crates/wfd-engine/src/{compile/ (except melee.rs),stats.rs}` | T06 |\n\
             | `crates/wfd-datagen/src/{join,emit}/**`, `main.rs`, `tests/spec_vectors_07.rs` | T18 |\n\
             | `xtask/src/spec_vectors.rs` / `effect_smoke.rs` | T09 / T19 |\n\
             | `frontend/` (foundation) | T12 |\n| `data/curated/**` item directories | T20 |\n\
             ```text\ncargo test -p wfd-model --test fixtures   # T03\n```\n",
        );
        let p = planned_paths(&plan, &[]);
        for path in [
            "crates/wfd-engine/tests/golden_oracle.rs",
            "crates/wfd-engine/src/outputs.rs",
            "crates/wfd-engine/tests/properties.rs",
            "crates/wfd-engine/benches/window.rs",
            "crates/wfd-engine/src/resolve/firing.rs",
            "crates/wfd-engine/src/stats.rs",
            "crates/wfd-engine/src/compile",
            "crates/wfd-datagen/src/main.rs",
            "crates/wfd-datagen/tests/spec_vectors_07.rs",
            "xtask/src/effect_smoke.rs",
            "crates/wfd-model/tests/fixtures.rs",
            "frontend",
            "crates/*/tests",
        ] {
            assert!(planned(&p, path), "{path}: {:?}", p.status(path));
        }
        // A relative span resolves in its package only, not against every ancestor of the base.
        for path in [
            "crates/tests/properties.rs",
            "crates/benches/window.rs",
            "crates/wfd-engine/src/ev/tests/properties.rs",
            "crates/wfd-engine/tests/measured.rs",
            "research/test-cases/ingame/*.toml",
        ] {
            assert!(
                matches!(p.status(path), Status::Unknown(_)),
                "{path}: {:?}",
                p.status(path)
            );
        }
        // Under an owned directory, a file no task and no layout tree names is a warning; a pattern is planned.
        assert!(matches!(
            p.status("frontend/src/features/nothing/typo.tsx"),
            Status::Warn(_)
        ));
        assert!(matches!(
            p.status("data/curated/mods/xx/typo.toml"),
            Status::Warn(_)
        ));
        assert!(matches!(
            p.status("crates/wfd-engine/src/compile/scop_typo.rs"),
            Status::Warn(_)
        ));
        assert!(planned(&p, "data/curated/weapon_mod/<family>/<slug>.toml"));
    }

    #[test]
    fn layout_trees() {
        let doc = parse_doc(
            "x.md",
            "```text\n\
             crates/wfd-engine/src/\n  lib.rs  error.rs\n  spec_constants.rs     formula constants (I-1)\n\
             \x20 compile/              scenario.rs (L0), pick.rs (L1, T-02\n\
             \x20                       table), melee.rs (E-7.0b table resolution, MeleePlanSel → MeleePlan)\n\
             \x20                       riven.rs, buffs.rs\n\
             \x20 enemy/                armor.rs (E-15, see notes/x.rs)\n\
             ```\n\
             ```text\nwarframe-dps/\n├── Cargo.toml                 # [workspace]\n├── crates/\n\
             │   ├── wfd-model/             # types only: enums/x.rs\n│   └── wfd-data/\n\
             ├── data/\n│   ├── fixtures/mini/         # tiny bundle\n└── .cache/\n```\n\
             ```text\nfrontend/\n  src/\n    api/         client.ts (fetch wrapper)  generated/*.ts (ts-rs; do not edit)\n\
             \x20   features/\n      shell/     TopBar  AppLayout  PaneTabs (narrow screens)\n\
             \x20 e2e/\n    keyboard.spec.ts\n```\n\
             ```text\ndata/curated/\n  weapon_mod/<family>/<slug>.toml      # family = rifle, shotgun\n  rules/\n\
             \x20   melee/heavy_class.toml  toggles.toml\n  aliases/\n    inspiration.toml   # names\n```\n\
             ```text\nwfd-model ◄── wfd-data ◄── wfd-opt\n```\n",
        );
        let l = layout_paths(&doc);
        for p in [
            "crates/wfd-engine/src/lib.rs",
            "crates/wfd-engine/src/spec_constants.rs",
            "crates/wfd-engine/src/compile",
            "crates/wfd-engine/src/compile/pick.rs",
            "crates/wfd-engine/src/compile/melee.rs",
            "crates/wfd-engine/src/compile/buffs.rs",
            "crates/wfd-engine/src/enemy/armor.rs",
            "Cargo.toml",
            "crates/wfd-model",
            "crates/wfd-data",
            "data/fixtures/mini",
            "frontend/src/api/client.ts",
            "frontend/src/api/generated/*.ts",
            "frontend/src/features/shell/TopBar.*",
            "frontend/src/features/shell/PaneTabs.*",
            "frontend/e2e/keyboard.spec.ts",
            "data/curated/weapon_mod/*/*.toml",
            "data/curated/rules/melee/heavy_class.toml",
            "data/curated/rules/toggles.toml",
            "data/curated/aliases/inspiration.toml",
        ] {
            assert!(l.contains(p), "{p}: {l:#?}");
        }
        // Parenthesised remarks (also across lines), `#` comments and description words are not entries.
        for p in [
            "crates/wfd-engine/src/compile/table.rs",
            "crates/wfd-engine/src/enemy/notes/x.rs",
            "crates/wfd-model/enums/x.rs",
            "crates/wfd-engine/src/compile/MeleePlan.*",
            "data/curated/weapon_mod/rifle",
        ] {
            assert!(!l.contains(p), "{p}");
        }
        assert!(!l.iter().any(|p| p.contains("wfd-opt")));
    }

    #[test]
    fn definitions_not_mentions() {
        let mut out = BTreeSet::new();
        rust_definitions(
            r#"
            //! `CalcReport` is mentioned in a comment only.
            use crate::x::{Imported, Other as Renamed};
            /// Doc of `DocMention`.
            pub struct Defined { field: Mentioned }
            pub enum Choice { #[serde(rename = "a")] First, Second(Inner), Third { x: u8 }, Fourth = 4 }
            pub trait Tr { type Assoc; const LIMIT: u8; fn method(&self); }
            type Alias = Vec<u8>;
            sub_enum!(
                /// Doc.
                PhysType { IMP => "IMP" }
            );
            bitflags::bitflags! { pub struct Flags: u8 { const A = 1; } }
            macro_rules! my_macro { () => {} }
            fn body() { let s = "StringMention"; vec![CallMention::new()]; }
            mod inner { rule_file!(InnerFile { x: u8 }); }
            "#,
            &mut out,
        );
        for name in [
            "Defined",
            "Choice",
            "First",
            "Second",
            "Third",
            "Fourth",
            "Tr",
            "Assoc",
            "LIMIT",
            "method",
            "Alias",
            "PhysType",
            "Flags",
            "A",
            "my_macro",
            "body",
            "inner",
            "InnerFile",
            "Renamed",
        ] {
            assert!(out.contains(name), "{name}: {out:?}");
        }
        for name in [
            "CalcReport",
            "DocMention",
            "Imported",
            "Mentioned",
            "Inner",
            "StringMention",
            "CallMention",
            "IMP",
            "x",
        ] {
            assert!(!out.contains(name), "{name}: {out:?}");
        }
        let mut ts = BTreeSet::new();
        ts_definitions(
            "// type Commented = 1;\nexport type CalcReport = { a: Mentioned };\nexport function ModGrid() {}\n\
             export const LIMIT = 3; /* interface Blocked {} */ interface Props {}\n",
            &mut ts,
        );
        assert_eq!(
            ts.into_iter().collect::<Vec<_>>(),
            ["CalcReport", "LIMIT", "ModGrid", "Props"]
        );
    }

    #[test]
    fn tasks_from_headings() {
        let plan = parse_doc(
            PLAN,
            "### T01a Workspace\n### T20-T29 Curation batches\ntext T99\n",
        );
        let t = planned_tasks(&plan);
        assert!(t.contains("T01a") && t.contains("T20") && t.contains("T25") && t.contains("T29"));
        assert!(!t.contains("T99") && !t.contains("T30"));
    }

    #[test]
    fn type_heads() {
        let head = |s: &str| TYPE_HEAD.captures(s).map(|c| c[1].to_owned());
        assert_eq!(head("CalcReport").as_deref(), Some("CalcReport"));
        assert_eq!(
            head("Profile::Compat(CompatFlags)").as_deref(),
            Some("Profile")
        );
        assert_eq!(head("wfd_model::scenario::TOGGLE_MAP"), None);
        assert_eq!(head("Option<u8>").as_deref(), Some("Option"));
        assert_eq!(head("HEA"), None);
        assert_eq!(head("Status:"), None);
    }

    /// The documents of this worktree: every check runs over a realistic number of references, and every finding is
    /// well formed (`doc:line: kind `name`: reason` with an existing document and line). Which findings there are is
    /// `cargo xtask doc-check`'s business, not a test failure: the documents change independently of this crate.
    #[test]
    fn workspace_docs_are_checked() {
        let root = crate::workspace_root();
        let report = check(&root, false).unwrap();
        let count = |kind: &str| -> usize {
            let re = Regex::new(&format!(r"(\d+) {kind} reference")).unwrap();
            re.captures(&report.summary)
                .map_or(0, |c| c[1].parse().unwrap())
        };
        let n_docs: usize = STATUS_DIRS
            .iter()
            .map(|d| md_files(&root.join(d)).unwrap().len())
            .sum();
        assert!(
            n_docs >= 30
                && report
                    .summary
                    .contains(&format!(": {n_docs} status line(s)")),
            "{}",
            report.summary
        );
        for (kind, min) in [
            ("task id", 300),
            ("command", 50),
            ("package", 30),
            ("path", 300),
            ("type", 300),
        ] {
            assert!(count(kind) >= min, "{kind}: {}", report.summary);
        }
        let finding =
            Regex::new(r"^(docs/(?:architecture|data|mechanics)/[a-z0-9-]+\.md|research/findings/[^:]+\.md):(\d+): ")
                .unwrap();
        for f in report.problems.iter().chain(report.warnings.iter()) {
            let c = finding
                .captures(f)
                .unwrap_or_else(|| panic!("malformed finding: {f}"));
            let lines = std::fs::read_to_string(root.join(&c[1]))
                .unwrap()
                .lines()
                .count();
            let line: usize = c[2].parse().unwrap();
            assert!(line >= 1 && line <= lines, "{f}");
        }
        // Names that exist are found: a path, a defined type, a registered command, a package.
        let index = index_repo(&root).unwrap();
        assert!(index.contains("crates/wfd-model/src/stats.rs"));
        let defined = defined_names(&root, &index).unwrap();
        for t in [
            "StatKey",
            "CompatFlags",
            "Ips",
            "NcElem",
            "ItemKind",
            "MultGroupsFile",
        ] {
            assert!(defined.contains(t), "{t}");
        }
    }

    /// A fixture workspace with one good and one bad reference of every kind.
    #[test]
    fn fixture_workspace() {
        let root = std::env::temp_dir().join(format!("wfd-doc-check-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let w = |rel: &str, text: &str| {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        w(
            "Cargo.toml",
            "[workspace]\nmembers = [\n    \"crates/wfd-engine\",\n]\n",
        );
        w(
            "crates/wfd-engine/Cargo.toml",
            "[package]\nname = \"wfd-engine\"\n",
        );
        w(
            "crates/wfd-engine/src/lib.rs",
            "pub struct ResolvedAttack;\n// CommentOnly is mentioned here, StrictOnly too.\n\
             const S: &str = \"StringOnly\";\n",
        );
        w(
            PLAN,
            "Status: reviewed\n\n### T01a Workspace\n### T12 Frontend\n### T15a EV\n### T20-T29 Curation\n\n\
             | `crates/wfd-engine/src/ev/{kernel,steady}.rs`, `tests/mechanics.rs` | T15a |\n\
             | `frontend/` (foundation) | T12 |\n\nT15a provides `Kernel`, `StrictOnly` and `wfd-datagen build`.\n",
        );
        w("docs/data/pipeline.md", "# no status line\n");
        w(
            "docs/architecture/x.md",
            "Status: draft\n\
             T01a and T01 and T25 exist; T07 does not.\n\
             `cargo xtask spec-sync --check`, `cargo xtask frobnicate`, `cargo test -p wfd-engine --test mechanics`,\n\
             `cargo test -p wfd-engin`, `wfd-datagen build`, `wfd-datagen check`, `wfd-datagen nope`.\n\
             ```text\nwfd-datagen check   # synopsis\npub struct Designed { x: u8 }\n```\n\
             Paths: `crates/wfd-engine/src/lib.rs`, `crates/wfd-engine/src/ev/kernel.rs`, `frontend/vite.config.ts`,\n\
             `crates/wfd-engine/tests/measured.rs`, `research/external/cache/x`, `crates/*/src/lib.rs`.\n\
             Types: `ResolvedAttack`, `Kernel`, `Designed`, `HashMap<K, V>`, `OrderKey { a, b }`, `Dangling::X`,\n\
             `CommentOnly`, `StringOnly`, `StrictOnly`.\n\
             ```text\nfrontend/\n  vite.config.ts\n```\n",
        );
        let report = check(&root, false).unwrap();
        let strict = check(&root, true).unwrap();
        let _ = std::fs::remove_dir_all(&root);
        let mut p: Vec<String> = report
            .problems
            .iter()
            .map(|p| {
                p.split(": ").next().unwrap().to_owned() + " " + p.split('`').nth(1).unwrap_or("")
            })
            .collect();
        p.sort();
        assert_eq!(
            p,
            [
                "docs/architecture/x.md:10 crates/wfd-engine/tests/measured.rs",
                "docs/architecture/x.md:11 Dangling",
                "docs/architecture/x.md:12 CommentOnly",
                "docs/architecture/x.md:12 StringOnly",
                "docs/architecture/x.md:2 T07",
                "docs/architecture/x.md:3 cargo xtask frobnicate",
                "docs/architecture/x.md:4 wfd-datagen nope",
                "docs/architecture/x.md:4 wfd-engin",
                "docs/data/pipeline.md:1 Status: draft|reviewed|cross-checked|superseded …",
            ],
            "{:#?}",
            report.problems
        );
        assert!(report.warnings.is_empty(), "{:#?}", report.warnings);
        // --strict: planned names fail too (the planned path, the plan-only and design-only types, and a type the
        // sources only mention in a comment).
        for name in [
            "crates/wfd-engine/src/ev/kernel.rs",
            "frontend/vite.config.ts",
            "type `Kernel`",
            "type `Designed`",
            "type `StrictOnly`",
        ] {
            assert!(
                strict.problems.iter().any(|p| p.contains(name)),
                "{name}: {:#?}",
                strict.problems
            );
        }
        assert!(
            !strict
                .problems
                .iter()
                .any(|p| p.contains("ResolvedAttack") || p.contains("HashMap"))
        );
    }
}
