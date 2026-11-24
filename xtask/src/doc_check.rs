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
//!   name occurs in the workspace sources (Rust, and the frontend's TypeScript once it exists), or in an
//!   [`EXTERNAL`] list (std, third-party crates, the web platform, upstream data keys), or is planned.
//!
//! **Planned** (default mode): a path named by work-breakdown.md (task sections and the ownership map of §8; a
//! relative span after an absolute one on the same line resolves against it, `tests/x.rs` after
//! `crates/wfd-engine/src/…`), or a path under a directory it assigns (`frontend/`, `data/curated/**`); a type named in
//! work-breakdown.md or defined by the design (a code block of the scanned documents, or a prose `Name { … }` /
//! `Name(…)` shape). `--strict` (for the final documentation pass, T41) accepts only what exists.

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
    // upstream data keys (wiki modules, Public Export, WFCD)
    "Ammocost",
    "AttackParentIndex",
    "EximusSHield",
    "ExportAbilities",
    "ForcedProc",
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
    pub summary: String,
    /// `doc:line: kind `name`: reason`.
    pub problems: Vec<String>,
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

/// The paths the plan names: concrete patterns and owned directories.
#[derive(Debug, Default)]
pub struct PlannedPaths {
    pub paths: BTreeSet<String>,
    pub dirs: BTreeSet<String>,
}

impl PlannedPaths {
    /// Is the (glob) path `p` planned: under an owned directory or an ancestor of one, matched by a planned path,
    /// matching one, or an ancestor directory of one?
    pub fn covers(&self, p: &str) -> bool {
        self.dirs
            .iter()
            .any(|d| glob_ancestor(d, p) || glob_ancestor(p, d))
            || self
                .paths
                .iter()
                .any(|q| glob_match(q, p) || glob_match(p, q) || glob_ancestor(p, q))
    }
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

/// Collects the planned paths of the plan document: inline spans that are repository paths, and relative spans
/// resolved against the directory (and its ancestors) of the last absolute span on the same line. Spans ending in
/// `/` or `/**` are owned directories.
pub fn planned_paths(plan: &Doc) -> PlannedPaths {
    let mut out = PlannedPaths::default();
    let mut by_line: BTreeMap<usize, Vec<&String>> = BTreeMap::new();
    for (n, s) in &plan.spans {
        by_line.entry(*n).or_default().push(s);
    }
    for spans in by_line.values() {
        let mut base: Option<String> = None;
        for s in spans {
            let candidates: Vec<String> = if is_repo_path(s) {
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
                vec![(*s).clone()]
            } else if let (Some(b), true) = (&base, looks_relative_path(s)) {
                let mut c = Vec::new();
                let mut dir = b.as_str();
                loop {
                    c.push(format!("{dir}/{s}"));
                    match dir.rsplit_once('/') {
                        Some((d, _)) => dir = d,
                        None => break,
                    }
                }
                c
            } else {
                continue;
            };
            for cand in candidates {
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

/// Capitalised identifiers of the workspace sources: Rust under `crates/` and `xtask/`, TypeScript under
/// `frontend/src/`.
pub fn code_identifiers(root: &Path, index: &BTreeSet<String>) -> Result<BTreeSet<String>> {
    let mut out = BTreeSet::new();
    for p in index {
        let rust = (p.starts_with("crates/") || p.starts_with("xtask/")) && p.ends_with(".rs");
        let ts = p.starts_with("frontend/src/") && (p.ends_with(".ts") || p.ends_with(".tsx"));
        if rust || ts {
            let text =
                std::fs::read_to_string(root.join(p)).with_context(|| format!("reading {p}"))?;
            out.extend(IDENT.find_iter(&text).map(|m| m.as_str().to_owned()));
        }
    }
    Ok(out)
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
    let planned = planned_paths(&plan);
    let packages = workspace_packages(root)?;
    let code_idents = code_identifiers(root, &index)?;
    let mut docs = Vec::new();
    for dir in SCANNED_DIRS {
        for f in md_files(&root.join(dir))? {
            let text =
                std::fs::read_to_string(&f).with_context(|| format!("reading {}", f.display()))?;
            docs.push(parse_doc(&rel(root, &f), &text));
        }
    }
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
    let mut report = |doc: &str, line: usize, kind: &'static str, name: &str, ok: Status| {
        let e = counts.entry(kind).or_default();
        e.0 += 1;
        match ok {
            Status::Exists => {}
            Status::Planned if !strict => e.1 += 1,
            Status::Planned => {
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
    let pkg_re = Regex::new(r"(?:^|\s)(?:-p|--package)\s+([A-Za-z0-9_-]+)").expect("static regex");
    let test_re = Regex::new(r"(?:^|\s)--(test|bench)\s+([A-Za-z0-9_]+)").expect("static regex");
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
                let krate = pkg_re.captures(text).map(|c| c[1].to_owned());
                for c in pkg_re.captures_iter(text) {
                    let ok = if packages.contains(&c[1]) {
                        Status::Exists
                    } else {
                        Status::Unknown("not a workspace package".to_owned())
                    };
                    report(&d.name, *n, "package", &c[1], ok);
                }
                if let Some(k) = &krate {
                    for c in test_re.captures_iter(text) {
                        let dir = if &c[1] == "test" { "tests" } else { "benches" };
                        let p = format!("crates/{k}/{dir}/{}.rs", &c[2]);
                        let ok = path_status(&p, &index, &planned);
                        report(&d.name, *n, "path", &p, ok);
                    }
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
        "doc-check{}: {} status line(s); {} document(s) of {}: {}",
        if strict { " --strict" } else { "" },
        n_status,
        docs.len(),
        SCANNED_DIRS.join(", "),
        parts.join(", ")
    );
    Ok(Report { summary, problems })
}

/// Outcome of one name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Exists,
    Planned,
    Unknown(String),
}

/// Whether a (possibly wildcard) path exists in the index, is planned, or neither.
pub fn path_status(p: &str, index: &BTreeSet<String>, planned: &PlannedPaths) -> Status {
    let g = to_glob(p);
    let exists = if has_wildcard(&g) {
        index.iter().any(|i| glob_match(&g, i))
    } else {
        index.contains(&g)
    };
    if exists {
        Status::Exists
    } else if planned.covers(&g) {
        Status::Planned
    } else {
        Status::Unknown(format!(
            "does not exist and {PLAN} does not plan it (no task names or owns it)"
        ))
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

    #[test]
    fn plan_paths_resolve_relative_spans() {
        let plan = parse_doc(
            PLAN,
            "| `crates/wfd-engine/src/{lib,outputs}.rs`, `tests/{compat,golden_oracle}.rs` | T17 |\n\
             | `xtask/src/spec_vectors.rs` / `effect_smoke.rs` | T09 / T19 |\n\
             | `frontend/` (foundation) | T12 |\n| `data/curated/**` item directories | T20 |\n",
        );
        let p = planned_paths(&plan);
        assert!(p.covers("crates/wfd-engine/tests/golden_oracle.rs"));
        assert!(p.covers("crates/wfd-engine/src/outputs.rs"));
        assert!(p.covers("xtask/src/effect_smoke.rs"));
        assert!(p.covers("frontend/vite.config.ts"));
        assert!(p.covers("data/curated/weapon_mod/sets/hunter_munitions.toml"));
        assert!(p.covers("crates/*/tests"));
        assert!(!p.covers("crates/wfd-engine/tests/measured.rs"));
        assert!(!p.covers("research/test-cases/ingame/*.toml"));
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

    /// The documents of this worktree parse and every check runs (their findings are `cargo xtask doc-check`'s, not a
    /// test failure: the documents change independently of this crate).
    #[test]
    fn workspace_docs_are_checked() {
        let report = check(&crate::workspace_root(), false).unwrap();
        for kind in [
            "status line",
            "task id",
            "command",
            "package",
            "path",
            "type",
        ] {
            assert!(report.summary.contains(kind), "{kind}: {}", report.summary);
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
            "pub struct ResolvedAttack;\n",
        );
        w(
            PLAN,
            "Status: reviewed\n\n### T01a Workspace\n### T12 Frontend\n### T15a EV\n### T20-T29 Curation\n\n\
             | `crates/wfd-engine/src/ev/{kernel,steady}.rs`, `tests/mechanics.rs` | T15a |\n\
             | `frontend/` (foundation) | T12 |\n\nT15a provides `Kernel` and `wfd-datagen build`.\n",
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
             Types: `ResolvedAttack`, `Kernel`, `Designed`, `HashMap<K, V>`, `OrderKey { a, b }`, `Dangling::X`.\n",
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
                "docs/architecture/x.md:2 T07",
                "docs/architecture/x.md:3 cargo xtask frobnicate",
                "docs/architecture/x.md:4 wfd-datagen nope",
                "docs/architecture/x.md:4 wfd-engin",
                "docs/data/pipeline.md:1 Status: draft|reviewed|cross-checked|superseded …",
            ],
            "{:#?}",
            report.problems
        );
        // --strict: planned names fail too (the planned path, the plan-only and design-only types).
        for name in [
            "crates/wfd-engine/src/ev/kernel.rs",
            "frontend/vite.config.ts",
            "type `Kernel`",
            "type `Designed`",
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
