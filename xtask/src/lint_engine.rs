//! `cargo xtask lint-engine` (work-breakdown.md T01c; overview.md §6.1 I-1, I-2; testing.md §2.5).
//!
//! Scans the library sources of `wfd-engine` and `wfd-opt` (`crates/<crate>/src/**/*.rs`; integration tests and
//! benches live outside `src/` and are not scanned) with a small Rust lexer, skipping test code: items under an outer
//! attribute that holds only in test builds (`#[test]`, `#[tokio::test]`, `#[cfg(test)]`, `#[cfg(all(test, …))]`; not
//! `#[cfg(any(test, feature = "x"))]`, which also ships), files that start with `#![cfg(test)]`, and the files of
//! test-only `mod name;` declarations (honouring `#[path = "…"]`).
//!
//! - **I-1**: a float literal whose value is not 0, 1, 2 or 0.5 is an error outside
//!   `crates/wfd-engine/src/spec_constants.rs` (game numbers come from the bundle; spec formula constants live in that
//!   file, each with its E-number). Integer literals with an `f32`/`f64` suffix are float literals.
//! - **I-2**: no `/Lotus/` anywhere (code, string literals, comments), and no item name of the mini fixture
//!   (`data/fixtures/mini/`, the display names of every record with a `unique_name`, resolved through `names/*.json`)
//!   spelled as a run of words, case and separators ignored, inside one string literal (`"braton prime"`) or across an
//!   identifier path (`braton_prime`, `BratonPrime`, `Braton::Prime`). Two exemptions (CONTRACTS.md T01c, request
//!   C-22; widened by LINT-GLAIVE of the contract review, CONTRACTS.md "Contract review fixes (crates)"): an
//!   identifier that is model or spec vocabulary does not count on its own, and custom-stat records (`kind`
//!   `custom_stat`, whose names are the labels of spec 00 §6.3 such as "Multishot") are not item names. Vocabulary is
//!   [`model_vocabulary`] (a generated `CompatFlags` name such as `D007_PHOENIX_TALONS`, a stat-registry row or variant
//!   such as `phoenix_talons`/`PhoenixTalons`, a resolved-stat name such as `multishot`, a melee class name such as
//!   `Glaive`/`Machete`, a melee plan kind such as `GlaiveCycle`) and [`spec_vocabulary`] (read from
//!   `docs/mechanics/` at lint time: every code identifier of the spec symbol tables, such as `glaive_throw_ct` or
//!   `part_sonar_mult`, and every mechanic id string of 08 E-8.16/E-8.17, such as `"glaive_volatile"`). Item names in
//!   comments are allowed: code cites the spec, whose examples name items; a comment is not a branch.
//!
//! The lint passes on empty crates.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

/// Crates whose `src/` is linted.
pub const LINTED_CRATES: &[&str] = &["crates/wfd-engine", "crates/wfd-opt"];
/// The one file allowed to hold spec formula constants (I-1).
pub const SPEC_CONSTANTS: &str = "crates/wfd-engine/src/spec_constants.rs";
/// Fixture whose item names are forbidden (I-2).
pub const MINI_FIXTURE: &str = "data/fixtures/mini";
/// Float values allowed anywhere (overview.md §6.1 I-1).
pub const ALLOWED_FLOATS: &[f64] = &[0.0, 1.0, 2.0, 0.5];
/// The spec whose identifiers are vocabulary ([`spec_vocabulary`]).
pub const MECHANICS_DIR: &str = crate::spec_sync::MECHANICS_DIR;

#[derive(clap::Parser, Debug)]
#[command(
    name = "cargo xtask lint-engine",
    about = "Invariants I-1 and I-2 on wfd-engine and wfd-opt (T01c)"
)]
struct Args {
    /// Workspace root to lint (default: this workspace).
    #[arg(long)]
    root: Option<PathBuf>,
}

/// Entry point (`cargo xtask lint-engine`).
pub fn run(args: &[String]) -> Result<()> {
    let args: Args = crate::parse_args("lint-engine", args);
    let root = args.root.unwrap_or_else(crate::workspace_root);
    let report = lint_workspace(&root)?;
    println!(
        "lint-engine: {} source file(s) in {} ({} excluded as test code), {} item name(s) from {MINI_FIXTURE} \
         ({} custom-stat label(s) exempt), {} model identifier(s) and {} spec identifier(s) from {MECHANICS_DIR} \
         exempt",
        report.files,
        LINTED_CRATES.join(", "),
        report.test_files,
        report.names,
        report.custom_stats,
        report.vocabulary,
        report.spec_vocabulary
    );
    if report.problems.is_empty() {
        return Ok(());
    }
    for p in &report.problems {
        eprintln!("  - {p}");
    }
    Err(anyhow!("lint-engine: {} problem(s)", report.problems.len()))
}

/// Result of a workspace lint.
#[derive(Debug, Default)]
pub struct Report {
    /// Files scanned.
    pub files: usize,
    /// Files skipped as test modules.
    pub test_files: usize,
    /// Item names checked.
    pub names: usize,
    /// Custom-stat labels of the fixture, not checked (spec 00 §6.3).
    pub custom_stats: usize,
    /// Model identifiers exempt as a whole ([`model_vocabulary`]).
    pub vocabulary: usize,
    /// Spec identifiers exempt as a whole ([`spec_vocabulary`]; normalized, so some may equal a model identifier).
    pub spec_vocabulary: usize,
    /// `path:line: message`.
    pub problems: Vec<String>,
}

/// Lints the workspace at `root`.
pub fn lint_workspace(root: &Path) -> Result<Report> {
    let fixture = fixture_names(&root.join(MINI_FIXTURE))?;
    let mut vocabulary = model_vocabulary();
    let spec = spec_vocabulary(root)?;
    let names = fixture.items;
    let mut report = Report {
        names: names.len(),
        custom_stats: fixture.custom_stats.len(),
        vocabulary: vocabulary.len(),
        spec_vocabulary: spec.len(),
        ..Report::default()
    };
    vocabulary.extend(spec);
    let mut files = Vec::new();
    for krate in LINTED_CRATES {
        collect_rs(&root.join(krate).join("src"), &mut files)?;
    }
    files.sort();
    // Test-only files: the files of test-only `mod name;` declarations and files that start with `#![cfg(test)]`
    // (all files are scanned first, so the order of parents and children does not matter).
    let mut test_files: BTreeSet<PathBuf> = BTreeSet::new();
    let mut scans = Vec::new();
    for file in &files {
        let text =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        let scan = scan_source(&text);
        if scan.test_only {
            test_files.insert(file.clone());
        }
        for m in &scan.test_modules {
            for child in module_files(file, m) {
                test_files.insert(child);
            }
        }
        scans.push((file.clone(), scan));
    }
    for (file, scan) in scans {
        // A file under a test module's directory is test code too.
        if test_files
            .iter()
            .any(|t| file == *t || file.starts_with(module_dir(t)))
        {
            report.test_files += 1;
            continue;
        }
        report.files += 1;
        let rel = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let exempt_floats = rel == SPEC_CONSTANTS;
        for p in check_scan(&scan, &names, &vocabulary, exempt_floats) {
            report.problems.push(format!("{rel}:{p}"));
        }
    }
    Ok(report)
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_rs(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// The candidate files of a `mod name;` declared in `parent`: with `#[path = "p"]`, `p` relative to the directory of
/// `parent`; else `name.rs` and `name/mod.rs` in the module directory of `parent` ([`module_dir`]).
pub fn module_files(parent: &Path, m: &TestModule) -> Vec<PathBuf> {
    if let Some(p) = &m.path {
        return vec![parent.parent().unwrap_or(Path::new(".")).join(p)];
    }
    let base = module_dir(parent);
    vec![
        base.join(format!("{}.rs", m.name)),
        base.join(&m.name).join("mod.rs"),
    ]
}

/// The directory holding the files of the modules a file declares: its own directory for `lib.rs`, `main.rs` and
/// `mod.rs`, else the directory named after it (`src/ev/kernel.rs` → `src/ev/kernel/`).
pub fn module_dir(file: &Path) -> PathBuf {
    let dir = file.parent().unwrap_or(Path::new("."));
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if matches!(stem, "lib" | "main" | "mod") {
        dir.to_path_buf()
    } else {
        dir.join(stem)
    }
}

// ------------------------------------------------------------------------------------------------------------------
// Fixture names
// ------------------------------------------------------------------------------------------------------------------

/// The display names of a fixture.
#[derive(Debug, Default)]
pub struct FixtureNames {
    /// Item names (I-2).
    pub items: BTreeSet<String>,
    /// Names of custom-stat records: the labels of spec 00 §6.3, not checked.
    pub custom_stats: BTreeSet<String>,
}

/// Display names of the fixture's records (every JSON object with a string `unique_name`/`uniqueName` and `name`),
/// resolved through the string maps of `names/*.json`; records whose `kind` is `custom_stat` (`"custom_stat"`,
/// `{"custom_stat": …}`, or the curated slug `"custom"`) go to [`FixtureNames::custom_stats`]. Missing directory: no
/// names.
pub fn fixture_names(dir: &Path) -> Result<FixtureNames> {
    let mut jsons = Vec::new();
    collect_ext(dir, "json", &mut jsons)?;
    let mut table: BTreeMap<String, String> = BTreeMap::new();
    let mut keys: Vec<(String, bool)> = Vec::new();
    for file in &jsons {
        let text =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).with_context(|| format!("parsing {}", file.display()))?;
        let in_names = file.components().any(|c| c.as_os_str() == "names");
        if in_names {
            flatten_strings(&value, &mut table);
        } else {
            collect_named(&value, &mut keys);
        }
    }
    let mut names = FixtureNames::default();
    for (key, custom) in keys {
        let name = table.get(&key).cloned().unwrap_or(key);
        if !name.starts_with('/') && normalize(&name).len() >= 3 {
            if custom {
                names.custom_stats.insert(name);
            } else {
                names.items.insert(name);
            }
        }
    }
    Ok(names)
}

fn collect_ext(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_ext(&path, ext, out)?;
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
    Ok(())
}

fn flatten_strings(v: &serde_json::Value, out: &mut BTreeMap<String, String>) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, v) in m {
                match v {
                    serde_json::Value::String(s) => {
                        out.insert(k.clone(), s.clone());
                    }
                    other => flatten_strings(other, out),
                }
            }
        }
        serde_json::Value::Array(a) => a.iter().for_each(|v| flatten_strings(v, out)),
        _ => {}
    }
}

fn collect_named(v: &serde_json::Value, out: &mut Vec<(String, bool)>) {
    match v {
        serde_json::Value::Object(m) => {
            let unique = m.get("unique_name").or_else(|| m.get("uniqueName"));
            if let (Some(serde_json::Value::String(_)), Some(serde_json::Value::String(name))) =
                (unique, m.get("name"))
            {
                let custom = match m.get("kind") {
                    Some(serde_json::Value::String(k)) => k == "custom_stat" || k == "custom",
                    Some(serde_json::Value::Object(k)) => k.contains_key("custom_stat"),
                    _ => false,
                };
                out.push((name.clone(), custom));
            }
            m.values().for_each(|v| collect_named(v, out));
        }
        serde_json::Value::Array(a) => a.iter().for_each(|v| collect_named(v, out)),
        _ => {}
    }
}

/// Lower-case ASCII alphanumerics only: `"Braton Prime"`, `braton_prime` and `BratonPrime` → `"bratonprime"`.
pub fn normalize(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Identifiers generated from `wfd-model` that are exempt from I-2 as a whole identifier, normalized ([`normalize`]):
/// the `CompatFlags` names (`D007_PHOENIX_TALONS`: domain-model.md §9.5 requires
/// `if ctx.compat.contains(CompatFlags::D0xx_…)` in the stage that owns the row), the stat-registry rows and variants
/// (`phoenix_talons`, `PhoenixTalons`, `heat_inherit`, `multishot`; domain-model.md §10), the resolved-stat names
/// (`multishot`, `fire_rate`), the melee class names (`MeleeClass::ALL`, variant and wiki `Class` string: `Glaive`,
/// `Machete`, `"Heavy Blade"`; 07 E-7.40 branches on the class) and the melee plan kinds (`MeleePlanKind::ALL`:
/// `GlaiveCycle`; 07 §8.1). Spec rows, registry stats, classes and plans named like an item are mechanics of the
/// spec, not item branches. An identifier that only contains such a name (`phoenix_talons_bonus`, `glaive_prime`) is
/// not exempt.
pub fn model_vocabulary() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    out.extend(wfd_model::CompatFlags::all_names());
    for row in wfd_model::STAT_ROWS.iter() {
        out.insert(row.toml.to_owned());
    }
    for key in wfd_model::StatKey::all() {
        let debug = format!("{key:?}");
        out.insert(debug.split('(').next().unwrap_or(&debug).to_owned());
    }
    for name in wfd_model::ResolvedStat::all_names() {
        out.insert(name.split('.').next().unwrap_or(&name).to_owned());
    }
    for class in wfd_model::MeleeClass::ALL {
        out.insert(format!("{class:?}"));
        out.insert(class.wiki_name().to_owned());
    }
    for kind in wfd_model::api::MeleePlanKind::ALL {
        out.insert(format!("{kind:?}"));
    }
    out.iter()
        .map(|n| normalize(n))
        .filter(|n| !n.is_empty())
        .collect()
}

/// Identifiers of the spec that are exempt from I-2 as a whole identifier, normalized ([`normalize`]), read from
/// `docs/mechanics/0N-*.md` at lint time so that no hand-copied list can drift (LINT-GLAIVE of the contract review):
///
/// - the code identifiers of every symbol table (a table whose header starts `| ID | Symbol |`, overview 0.3 and the
///   "symbols added" tables of 01-09): the leading identifier of each code span of the first column (`base_listed[t]`
///   → `base_listed`; `glaive_throw_ct`, `glaive_volatile_chance`, `part_sonar_mult`, `sonar_mult`);
/// - the mechanic id strings of 08 E-8.16 and E-8.17 (the quoted strings that are whole identifiers, from the
///   `E-8.16` line to the next heading, which holds both blocks and the draw list of 02 §4.8: `"glaive_volatile"`,
///   `"crit_roll"`, `"hit_frac"`); the engine hashes them (TV-08-02), so they appear in its code as literals.
///
/// Without `docs/mechanics/` (a test workspace) the set is empty; a spec without the E-8.16 block is an error.
pub fn spec_vocabulary(root: &Path) -> Result<BTreeSet<String>> {
    if !root.join(MECHANICS_DIR).is_dir() {
        return Ok(BTreeSet::new());
    }
    let mut out = BTreeSet::new();
    for (doc, text) in crate::spec_sync::read_mechanics(root)? {
        let lines: Vec<&str> = text.lines().collect();
        out.extend(symbol_table_ids(&lines));
        if doc == "08" {
            let ids = mechanic_id_strings(&lines);
            if ids.is_empty() {
                return Err(anyhow!(
                    "{MECHANICS_DIR}/08: no quoted mechanic id after the E-8.16 line (08 §6.1)"
                ));
            }
            out.extend(ids);
        }
    }
    Ok(out
        .iter()
        .map(|n| normalize(n))
        .filter(|n| !n.is_empty())
        .collect())
}

/// The leading identifier of every code span in the first column of the symbol tables (header `| ID | Symbol | …`)
/// of one spec document; fenced code is skipped.
pub fn symbol_table_ids(lines: &[&str]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut in_code = false;
    let mut in_table = false;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim_start();
        if t.starts_with("```") {
            in_code = !in_code;
            in_table = false;
            continue;
        }
        if in_code {
            continue;
        }
        if !t.starts_with('|') {
            in_table = false;
            continue;
        }
        let cells = crate::spec_sync::split_cells(t);
        if !in_table {
            let separator = lines.get(i + 1).is_some_and(|l| {
                let sep = crate::spec_sync::split_cells(l);
                !sep.is_empty()
                    && sep
                        .iter()
                        .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':')))
            });
            in_table = separator
                && cells.first().is_some_and(|c| c == "ID")
                && cells.get(1).is_some_and(|c| c == "Symbol");
            continue;
        }
        let Some(first) = cells.first() else {
            continue;
        };
        if first.chars().all(|ch| matches!(ch, '-' | ':')) {
            continue; // the separator row
        }
        for span in first.split('`').skip(1).step_by(2) {
            let id: String = span
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if id
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            {
                out.insert(id);
            }
        }
    }
    out
}

/// The quoted strings that are whole identifiers (`"glaive_volatile"`, not `"xh/<source>/<name>"`) from the line of
/// 08 that starts `E-8.16` up to the next Markdown heading: the E-8.16 and E-8.17 blocks and the draw list after them.
pub fn mechanic_id_strings(lines: &[&str]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let Some(start) = lines
        .iter()
        .position(|l| l.trim_start().starts_with("E-8.16 "))
    else {
        return out;
    };
    for line in &lines[start..] {
        if line.starts_with('#') {
            break;
        }
        for quoted in line.split('"').skip(1).step_by(2) {
            let whole = quoted
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && quoted
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_');
            if whole {
                out.insert(quoted.to_owned());
            }
        }
    }
    out
}

// ------------------------------------------------------------------------------------------------------------------
// Lexer
// ------------------------------------------------------------------------------------------------------------------

/// A token of the non-test code.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// An identifier or keyword (`r#` stripped).
    Ident(String),
    /// Literal text and whether it is a float literal.
    Num(String, bool),
    /// String literal contents (raw text between the quotes).
    Str(String),
    /// A character or byte literal.
    Char,
    /// A lifetime or loop label.
    Lifetime,
    /// Any other character.
    Punct(char),
    /// A line or block comment, doc comments included (its text, delimiters included).
    Comment(String),
}

impl Tok {
    /// Is this a comment token?
    pub fn is_comment(&self) -> bool {
        matches!(self, Tok::Comment(_))
    }
}

/// Lexes Rust source into tokens with 1-based line numbers (a comment carries the line it starts on).
pub fn lex(src: &str) -> Vec<(usize, Tok)> {
    let c: Vec<char> = src.chars().collect();
    let n = c.len();
    let mut i = 0;
    let mut line = 1;
    let mut out: Vec<(usize, Tok)> = Vec::new();
    let is_ident_start = |ch: char| ch.is_alphabetic() || ch == '_';
    let is_ident = |ch: char| ch.is_alphanumeric() || ch == '_';
    let count_lines = |s: &[char]| s.iter().filter(|&&ch| ch == '\n').count();
    while i < n {
        let ch = c[i];
        if ch == '\n' {
            line += 1;
            i += 1;
        } else if ch.is_whitespace() {
            i += 1;
        } else if ch == '/' && c.get(i + 1) == Some(&'/') {
            let start = i;
            while i < n && c[i] != '\n' {
                i += 1;
            }
            out.push((line, Tok::Comment(c[start..i].iter().collect())));
        } else if ch == '/' && c.get(i + 1) == Some(&'*') {
            let (start, start_line) = (i, line);
            let mut depth = 0;
            while i < n {
                if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if c[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
            out.push((start_line, Tok::Comment(c[start..i].iter().collect())));
        } else if let Some((len, content_start, content_end)) = string_at(&c, i) {
            out.push((
                line,
                Tok::Str(c[content_start..content_end].iter().collect()),
            ));
            line += count_lines(&c[i..i + len]);
            i += len;
        } else if ch == '\'' || (ch == 'b' && c.get(i + 1) == Some(&'\'')) {
            let q = if ch == 'b' { i + 1 } else { i };
            if c.get(q + 1) == Some(&'\\') {
                // Escaped char literal: skip the escaped character, then up to the closing quote.
                let mut j = q + 3;
                while j < n && c[j] != '\'' {
                    j += 1;
                }
                out.push((line, Tok::Char));
                i = j + 1;
            } else if c.get(q + 2) == Some(&'\'') {
                out.push((line, Tok::Char));
                i = q + 3;
            } else {
                // Lifetime or label.
                let mut j = q + 1;
                while j < n && is_ident(c[j]) {
                    j += 1;
                }
                out.push((line, Tok::Lifetime));
                i = j;
            }
        } else if ch.is_ascii_digit() {
            let mut code = out.iter().rev().filter(|(_, t)| !t.is_comment());
            let tuple_index = matches!(code.next(), Some((_, Tok::Punct('.'))))
                && !matches!(code.next(), Some((_, Tok::Punct('.'))));
            let (len, is_float) = number_at(&c, i, tuple_index);
            out.push((line, Tok::Num(c[i..i + len].iter().collect(), is_float)));
            i += len;
        } else if is_ident_start(ch) {
            let mut j = i;
            if ch == 'r'
                && c.get(i + 1) == Some(&'#')
                && c.get(i + 2).is_some_and(|&x| is_ident_start(x))
            {
                j += 2; // raw identifier r#name
            }
            let start = j;
            while j < n && is_ident(c[j]) {
                j += 1;
            }
            out.push((line, Tok::Ident(c[start..j].iter().collect())));
            i = j;
        } else {
            out.push((line, Tok::Punct(ch)));
            i += 1;
        }
    }
    out
}

/// A string literal starting at `i` (`"…"`, `b"…"`, `c"…"`, `r#"…"#`, `br"…"`, `cr#"…"#`): total length and the
/// content range.
fn string_at(c: &[char], i: usize) -> Option<(usize, usize, usize)> {
    let mut j = i;
    if matches!(c.get(j), Some('b' | 'c')) {
        j += 1;
    }
    let raw = c.get(j) == Some(&'r');
    if raw {
        j += 1;
    }
    let mut hashes = 0;
    if raw {
        while c.get(j) == Some(&'#') {
            hashes += 1;
            j += 1;
        }
    }
    if c.get(j) != Some(&'"') {
        return None;
    }
    // A prefix letter must start the token (not the tail of an identifier such as `attr"`).
    if j > i && i > 0 && (c[i - 1].is_alphanumeric() || c[i - 1] == '_') {
        return None;
    }
    let content_start = j + 1;
    let mut k = content_start;
    while k < c.len() {
        if !raw && c[k] == '\\' {
            k += 2;
            continue;
        }
        if c[k] == '"' && (0..hashes).all(|h| c.get(k + 1 + h) == Some(&'#')) {
            return Some((k + 1 + hashes - i, content_start, k));
        }
        k += 1;
    }
    Some((c.len() - i, content_start, c.len()))
}

/// A number literal at `i`: its length and whether it is a float literal. `tuple_index`: the number follows a field
/// access dot (`t.0.1`), so no fraction is lexed.
fn number_at(c: &[char], i: usize, tuple_index: bool) -> (usize, bool) {
    let n = c.len();
    let at = |k: usize| c.get(k).copied().unwrap_or('\0');
    let mut j = i;
    if at(j) == '0' && matches!(at(j + 1), 'x' | 'o' | 'b') {
        j += 2;
        while j < n && (c[j].is_ascii_hexdigit() || c[j] == '_') {
            j += 1;
        }
        while j < n && (c[j].is_alphanumeric() || c[j] == '_') {
            j += 1; // suffix
        }
        return (j - i, false);
    }
    while j < n && (c[j].is_ascii_digit() || c[j] == '_') {
        j += 1;
    }
    if tuple_index {
        return (j - i, false);
    }
    let mut is_float = false;
    if at(j) == '.' && at(j + 1) != '.' && !(at(j + 1).is_alphabetic() || at(j + 1) == '_') {
        is_float = true;
        j += 1;
        while j < n && (c[j].is_ascii_digit() || c[j] == '_') {
            j += 1;
        }
    }
    if matches!(at(j), 'e' | 'E') {
        let mut k = j + 1;
        if matches!(at(k), '+' | '-') {
            k += 1;
        }
        while at(k) == '_' {
            k += 1;
        }
        if at(k).is_ascii_digit() {
            is_float = true;
            j = k;
            while j < n && (c[j].is_ascii_digit() || c[j] == '_') {
                j += 1;
            }
        }
    }
    if j < n && (c[j].is_alphabetic() || c[j] == '_') {
        let s = j;
        while j < n && (c[j].is_alphanumeric() || c[j] == '_') {
            j += 1;
        }
        let suffix: String = c[s..j].iter().filter(|&&ch| ch != '_').collect();
        if suffix == "f32" || suffix == "f64" {
            is_float = true;
        }
    }
    (j - i, is_float)
}

/// Numeric value of a float literal (`1_000.5e-3f64`).
pub fn float_value(text: &str) -> Option<f64> {
    let t: String = text.chars().filter(|&c| c != '_').collect();
    let t = t
        .strip_suffix("f64")
        .or_else(|| t.strip_suffix("f32"))
        .unwrap_or(&t);
    t.parse().ok()
}

/// A test-only `mod name;` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestModule {
    /// Module name.
    pub name: String,
    /// Its `#[path = "…"]` attribute, if any.
    pub path: Option<String>,
}

/// The tokens of a file outside test code, and its test-only module declarations.
#[derive(Debug, Default)]
pub struct Scan {
    /// Tokens outside test code (comments included), with their lines.
    pub tokens: Vec<(usize, Tok)>,
    /// `mod name;` declarations under a test-only attribute (their files are test code).
    pub test_modules: Vec<TestModule>,
    /// The file starts with a test-only inner attribute (`#![cfg(test)]`): all of it is test code.
    pub test_only: bool,
}

/// Three-valued truth of a `cfg` predicate in a build without `test` (Kleene logic: `Unknown` for every other
/// option, such as `feature = "parallel"` or `unix`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tri {
    True,
    False,
    Unknown,
}

/// Evaluates the `cfg` predicate at `toks[*i..]` with `test` false; advances `*i` past it.
fn eval_cfg(toks: &[&Tok], i: &mut usize) -> Tri {
    let Some(Tok::Ident(head)) = toks.get(*i) else {
        *i += 1;
        return Tri::Unknown;
    };
    *i += 1;
    match (head.as_str(), toks.get(*i)) {
        ("all" | "any" | "not", Some(Tok::Punct('('))) => {
            *i += 1;
            let mut args = Vec::new();
            while *i < toks.len() && toks[*i] != &Tok::Punct(')') {
                if toks[*i] == &Tok::Punct(',') {
                    *i += 1;
                    continue;
                }
                args.push(eval_cfg(toks, i));
            }
            *i += 1; // `)`
            match head.as_str() {
                "all" if args.contains(&Tri::False) => Tri::False,
                "all" if args.contains(&Tri::Unknown) => Tri::Unknown,
                "all" => Tri::True,
                "any" if args.contains(&Tri::True) => Tri::True,
                "any" if args.contains(&Tri::Unknown) => Tri::Unknown,
                "any" => Tri::False,
                _ => match args.first() {
                    Some(Tri::True) => Tri::False,
                    Some(Tri::False) => Tri::True,
                    _ => Tri::Unknown,
                },
            }
        }
        (_, Some(Tok::Punct('='))) => {
            *i += 2; // `= "value"`
            Tri::Unknown
        }
        ("test", _) => Tri::False,
        _ => Tri::Unknown,
    }
}

/// Is the attribute (the tokens between `#[` or `#![` and `]`) test-only? `cfg(p)` when `p` is false in every
/// build without `test` (`cfg(test)`, `cfg(all(test, …))`, not `cfg(any(test, feature = "x"))` or `cfg(not(test))`);
/// any other attribute whose path ends in `test` (`test`, `tokio::test(flavor = …)`).
fn is_test_attr(attr: &[(usize, Tok)]) -> bool {
    let toks: Vec<&Tok> = attr
        .iter()
        .map(|(_, t)| t)
        .filter(|t| !t.is_comment())
        .collect();
    if matches!(toks.as_slice(), [Tok::Ident(cfg), Tok::Punct('('), ..] if cfg == "cfg") {
        let mut i = 2;
        return eval_cfg(&toks, &mut i) == Tri::False;
    }
    // The attribute path: identifiers joined by `::`, up to the arguments.
    let mut last = None;
    for t in &toks {
        match t {
            Tok::Ident(s) => last = Some(s.as_str()),
            Tok::Punct(':') => {}
            _ => break,
        }
    }
    last == Some("test")
}

/// `path = "…"` of an attribute.
fn path_attr(attr: &[(usize, Tok)]) -> Option<String> {
    let toks: Vec<&Tok> = attr
        .iter()
        .map(|(_, t)| t)
        .filter(|t| !t.is_comment())
        .collect();
    match toks.as_slice() {
        [Tok::Ident(p), Tok::Punct('='), Tok::Str(v)] if p == "path" => Some(v.clone()),
        _ => None,
    }
}

/// The index of the next non-comment token at or after `i`.
fn skip_comments(toks: &[(usize, Tok)], mut i: usize) -> usize {
    while toks.get(i).is_some_and(|(_, t)| t.is_comment()) {
        i += 1;
    }
    i
}

/// Removes test items from a file's tokens.
pub fn scan_source(src: &str) -> Scan {
    let toks = lex(src);
    let mut scan = Scan::default();
    // Inner attributes at the top of the file: `#![cfg(test)]` makes the whole file test code.
    let mut i = skip_comments(&toks, 0);
    while let Some(end) = attribute_end(&toks, i, true) {
        if is_test_attr(&toks[i + 3..end]) {
            scan.test_only = true;
            return scan;
        }
        i = skip_comments(&toks, end + 1);
    }
    let mut i = 0;
    while i < toks.len() {
        // A run of outer attributes (comments between them allowed): test-only if any one is.
        let mut attrs = Vec::new();
        let mut j = i;
        while let Some(end) = attribute_end(&toks, j, false) {
            attrs.push((j + 2, end));
            j = skip_comments(&toks, end + 1);
        }
        if attrs.iter().any(|&(s, e)| is_test_attr(&toks[s..e])) {
            let path = attrs.iter().find_map(|&(s, e)| path_attr(&toks[s..e]));
            let (end, module) = skip_item(&toks, j);
            if let Some(name) = module {
                scan.test_modules.push(TestModule { name, path });
            }
            i = end;
            continue;
        }
        scan.tokens.push(toks[i].clone());
        i += 1;
    }
    scan
}

/// If `toks[i..]` starts an outer attribute `#[…]` (or, with `inner`, an inner attribute `#![…]`), the index of its
/// closing `]`.
fn attribute_end(toks: &[(usize, Tok)], i: usize, inner: bool) -> Option<usize> {
    let open = if inner { i + 2 } else { i + 1 };
    if toks.get(i).map(|t| &t.1) != Some(&Tok::Punct('#'))
        || (inner && toks.get(i + 1).map(|t| &t.1) != Some(&Tok::Punct('!')))
        || toks.get(open).map(|t| &t.1) != Some(&Tok::Punct('['))
    {
        return None;
    }
    let mut depth = 0;
    for (k, (_, t)) in toks.iter().enumerate().skip(open) {
        match t {
            Tok::Punct('[') => depth += 1,
            Tok::Punct(']') => {
                depth -= 1;
                if depth == 0 {
                    return Some(k);
                }
            }
            _ => {}
        }
    }
    None
}

/// Skips the item starting at `i` (further attributes, then up to a `;` before any block, or a balanced `{…}`).
/// Returns the index after the item and, for `mod name;`, the module name.
fn skip_item(toks: &[(usize, Tok)], mut i: usize) -> (usize, Option<String>) {
    i = skip_comments(toks, i);
    while let Some(end) = attribute_end(toks, i, false) {
        i = skip_comments(toks, end + 1);
    }
    let mut code: Vec<&Tok> = Vec::new(); // the item's tokens before its `;`, comments dropped
    let mut depth = 0i32;
    while i < toks.len() {
        match &toks[i].1 {
            Tok::Punct('(' | '[') => depth += 1,
            Tok::Punct(')' | ']') => depth -= 1,
            Tok::Punct(';') if depth == 0 => {
                let module = match code.as_slice() {
                    [.., Tok::Ident(kw), Tok::Ident(name)] if kw == "mod" => Some(name.clone()),
                    _ => None,
                };
                return (i + 1, module);
            }
            Tok::Punct('{') if depth == 0 => {
                let mut braces = 0;
                while i < toks.len() {
                    match toks[i].1 {
                        Tok::Punct('{') => braces += 1,
                        Tok::Punct('}') => {
                            braces -= 1;
                            if braces == 0 {
                                return (i + 1, None);
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                return (i, None);
            }
            _ => {}
        }
        if !toks[i].1.is_comment() {
            code.push(&toks[i].1);
        }
        i += 1;
    }
    (i, None)
}

/// Words of an identifier: `braton_prime` → [braton, prime]; `BratonPrime` → [braton, prime].
fn ident_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (k, &ch) in chars.iter().enumerate() {
        if ch == '_' {
            if !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
            }
            continue;
        }
        let boundary = ch.is_uppercase()
            && k > 0
            && (chars[k - 1].is_lowercase()
                || chars[k - 1].is_ascii_digit()
                || (chars[k - 1].is_uppercase()
                    && chars.get(k + 1).is_some_and(|c| c.is_lowercase())));
        if boundary && !cur.is_empty() {
            words.push(std::mem::take(&mut cur));
        }
        cur.extend(ch.to_lowercase());
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

/// One identifier (or one identifier-like piece of a string literal) for the I-2 word runs.
struct Piece {
    line: usize,
    /// Normalized words.
    words: Vec<String>,
    /// Model vocabulary: a match inside this piece alone does not count.
    exempt: bool,
}

impl Piece {
    fn new(line: usize, text: &str, vocabulary: &BTreeSet<String>) -> Piece {
        Piece {
            line,
            words: ident_words(text)
                .iter()
                .map(|w| normalize(w))
                .filter(|w| !w.is_empty())
                .collect(),
            exempt: vocabulary.contains(&normalize(text)),
        }
    }
}

/// Item names spelled by a run of consecutive words of `chain` (several pieces may contribute): `(line, name)` of each
/// match, unless the whole run lies inside one exempt piece.
fn word_runs<'a>(chain: &[Piece], keys: &'a BTreeMap<String, String>) -> Vec<(usize, &'a str)> {
    let flat: Vec<(usize, &str)> = chain
        .iter()
        .enumerate()
        .flat_map(|(p, piece)| piece.words.iter().map(move |w| (p, w.as_str())))
        .collect();
    let mut out = Vec::new();
    for start in 0..flat.len() {
        let mut acc = String::new();
        for &(p, w) in &flat[start..] {
            acc.push_str(w);
            let first = flat[start].0;
            if let Some(name) = keys.get(&acc)
                && !(first == p && chain[p].exempt)
            {
                out.push((chain[first].line, name.as_str()));
            }
            // Stop when no name starts with the words so far.
            if !keys
                .range(acc.clone()..)
                .next()
                .is_some_and(|(k, _)| k.starts_with(acc.as_str()))
            {
                break;
            }
        }
    }
    out
}

/// I-1 and I-2 problems of one scanned file, as `line: message` in line order. `names`: fixture item names;
/// `vocabulary`: exempt model identifiers ([`model_vocabulary`]).
pub fn check_scan(
    scan: &Scan,
    names: &BTreeSet<String>,
    vocabulary: &BTreeSet<String>,
    exempt_floats: bool,
) -> Vec<String> {
    let keys: BTreeMap<String, String> = names
        .iter()
        .map(|n| (normalize(n), n.clone()))
        .filter(|(k, _)| !k.is_empty())
        .collect();
    let mut problems: Vec<(usize, String)> = Vec::new();
    let item = |line: usize, name: &str, what: &str| {
        (
            line,
            format!("{what} spells the fixture item name \"{name}\" (I-2: no item-specific code)"),
        )
    };
    // Identifiers chained by `::`, `.` or nothing (`braton_prime`, `Braton::Prime`, `weapon.braton.prime`); comments
    // inside a chain are transparent.
    let mut chain: Vec<Piece> = Vec::new();
    let toks = &scan.tokens;
    let mut i = 0;
    while i < toks.len() {
        let (line, tok) = (toks[i].0, &toks[i].1);
        let joins = match tok {
            Tok::Ident(id) => {
                chain.push(Piece::new(line, id, vocabulary));
                1
            }
            Tok::Comment(_) | Tok::Punct('.') => 1,
            Tok::Punct(':') if matches!(toks.get(i + 1), Some((_, Tok::Punct(':')))) => 2,
            _ => 0,
        };
        if let Tok::Comment(text) = tok
            && text.contains("/Lotus/")
        {
            problems.push((
                line,
                "`/Lotus/` in a comment (I-2: no uniqueName paths in engine or optimizer sources)"
                    .to_owned(),
            ));
        }
        if joins > 0 {
            i += joins;
            continue;
        }
        for (l, name) in word_runs(&chain, &keys) {
            problems.push(item(l, name, "identifier"));
        }
        chain.clear();
        match tok {
            Tok::Num(text, true) if !exempt_floats => {
                let v = float_value(text);
                if !v.is_some_and(|v| ALLOWED_FLOATS.contains(&v)) {
                    problems.push((
                        line,
                        format!(
                            "float literal `{text}` (I-1: only 0, 1, 2 and 0.5 outside {SPEC_CONSTANTS}; game numbers \
                             come from the bundle, spec constants go to spec_constants.rs with their E-number)"
                        ),
                    ));
                }
            }
            Tok::Str(s) => {
                if s.contains("/Lotus/") {
                    problems.push((
                        line,
                        "`/Lotus/` in a string literal (I-2: no item-specific code)".to_owned(),
                    ));
                }
                // The identifier-like pieces of the literal, as one chain (`"Braton Prime"`, `"braton-prime"`).
                let pieces: Vec<Piece> = s
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .filter(|p| !p.is_empty())
                    .map(|p| Piece::new(line, p, vocabulary))
                    .collect();
                for (l, name) in word_runs(&pieces, &keys) {
                    problems.push(item(l, name, "string literal"));
                }
            }
            _ => {}
        }
        i += 1;
    }
    for (l, name) in word_runs(&chain, &keys) {
        problems.push(item(l, name, "identifier"));
    }
    problems.sort();
    problems.dedup();
    problems
        .into_iter()
        .map(|(l, m)| format!("{l}: {m}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floats(src: &str) -> Vec<String> {
        scan_source(src)
            .tokens
            .into_iter()
            .filter_map(|(_, t)| {
                if let Tok::Num(s, true) = t {
                    Some(s)
                } else {
                    None
                }
            })
            .collect()
    }

    fn lint(src: &str, names: &[&str]) -> Vec<String> {
        let names: BTreeSet<String> = names.iter().map(|s| (*s).to_owned()).collect();
        check_scan(&scan_source(src), &names, &model_vocabulary(), false)
    }

    #[test]
    fn float_literals() {
        assert_eq!(
            floats("let a = 0.25 + 1.0 + 2. + 3f32 + 1e5 + 1E-9_f64 + 2_f64 + 0.5;"),
            [
                "0.25", "1.0", "2.", "3f32", "1e5", "1E-9_f64", "2_f64", "0.5"
            ]
        );
        // Not floats: integers, hex with an `e` digit, ranges, tuple fields, method calls on integers.
        assert!(floats("let b = 10 + 0x1e5 + 0b1 + 7u8; for i in 0..5 {} let c = t.0.1; 1.max(2); x[1..=2];").is_empty());
        assert_eq!(floats("let r = 0..1.5;"), ["1.5"]);
        // A comment between a field access and its index does not make the index a float.
        assert!(floats("let c = t. /* field */ 0;").is_empty());
    }

    #[test]
    fn allowed_values_pass() {
        assert!(
            lint(
                "const A: f64 = 0.0; let b = -1.0 * 2.0 + 0.5 + 1f64 + 2_f32 + 1e0 + 0.;",
                &[]
            )
            .is_empty()
        );
        let p = lint("fn f() -> f64 { 0.25 }\nfn g() -> f64 { 1e-9 }", &[]);
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(
            p[0].starts_with("1: float literal `0.25`")
                && p[1].starts_with("2: float literal `1e-9`")
        );
    }

    #[test]
    fn comments_strings_and_chars_are_not_code() {
        let src = r##"
            // 0.25 in a line comment
            /* 0.3 in /* a nested */ block comment 0.7 */
            /// doc comment 0.9
            let s = "0.25"; let r = r#"0.3 "quoted""#; let b = b"0.4"; let c = '.'; let d = '\''; let e = b'x';
            fn f<'a>(x: &'a str) -> &'a str { x }
            let tail = 0.75;
        "##;
        assert_eq!(floats(src), ["0.75"]);
        let toks = lex(src);
        assert_eq!(toks.iter().filter(|(_, t)| *t == Tok::Lifetime).count(), 3);
        assert_eq!(toks.iter().filter(|(_, t)| t.is_comment()).count(), 3);
        assert_eq!(
            toks.last().unwrap().0,
            7,
            "line numbers survive multi-line comments and raw strings"
        );
    }

    #[test]
    fn test_code_is_skipped() {
        let src = "
            fn engine() -> f64 { 0.0 }
            #[cfg(test)]
            mod tests { #[test] fn t() { assert_eq!(0.25, 0.25); } }
            #[test]
            fn loose_test() { let x = 0.3; }
            #[cfg(all(test, feature = \"x\"))]
            const T: f64 = 0.4;
            #[cfg(test)] #[path = \"t.rs\"] mod more;
            /// Docs.
            #[path = \"u.rs\"]
            // a comment between the attributes
            #[cfg(test)]
            mod other;
            #[tokio::test(flavor = \"multi_thread\")]
            async fn async_test() { let y = 0.35; }
            #[cfg(not(not(test)))]
            const NN: f64 = 0.45;
            #[cfg(not(test))]
            const LIVE: f64 = 0.6;
            #[cfg(any(test, feature = \"parallel\"))]
            pub fn shipped() -> f64 { 4.2 }
            #[cfg(any(test, debug_assertions))]
            const DBG: f64 = 0.7;
            #[cfg_attr(test, derive(Debug))]
            struct S { x: f64 }
        ";
        let scan = scan_source(src);
        assert_eq!(
            scan.test_modules,
            [
                TestModule {
                    name: "more".into(),
                    path: Some("t.rs".into())
                },
                TestModule {
                    name: "other".into(),
                    path: Some("u.rs".into())
                }
            ]
        );
        assert!(!scan.test_only);
        // `any(test, …)` also holds in builds without `test`: that code ships and is linted.
        assert_eq!(floats(src), ["0.0", "0.6", "4.2", "0.7"]);
    }

    #[test]
    fn inner_test_attribute_makes_the_file_test_code() {
        let scan = scan_source(
            "//! Test helpers.\n#![allow(dead_code)]\n#![cfg(test)]\nconst X: f64 = 3.7;\n",
        );
        assert!(scan.test_only && scan.tokens.is_empty());
        let scan = scan_source("#![cfg(any(test, feature = \"x\"))]\nconst X: f64 = 3.7;\n");
        assert!(!scan.test_only);
        assert_eq!(
            floats("#![cfg(any(test, feature = \"x\"))]\nconst X: f64 = 3.7;\n"),
            ["3.7"]
        );
    }

    #[test]
    fn lotus_paths_and_item_names() {
        let names = ["Braton Prime", "Ash"];
        assert!(lint("let p = \"/Lotus/Weapons/Tenno/Rifle\";", &names)[0].contains("/Lotus/"));
        // testing.md §2.5, overview.md I-2: no `/Lotus/` in the sources, comments included.
        let p = lint("// see /Lotus/Weapons/Tenno/Rifle\nlet x = 1;", &names);
        assert!(
            p.len() == 1 && p[0].starts_with("1: `/Lotus/` in a comment"),
            "{p:?}"
        );
        assert_eq!(lint("if name == \"Braton Prime\" {}", &names).len(), 1);
        // String literals: case and separators do not matter.
        for s in [
            "\"braton prime\"",
            "\"BRATON PRIME\"",
            "\"braton_prime\"",
            "\"braton-prime\"",
            "\"BratonPrime\"",
            "\"the Braton Prime riven\"",
        ] {
            assert_eq!(lint(&format!("let s = {s};"), &names).len(), 1, "{s}");
        }
        assert_eq!(lint("let braton_prime = 1;", &names).len(), 1);
        assert_eq!(lint("struct BratonPrime;", &names).len(), 1);
        assert_eq!(lint("let x = Braton::Prime;", &names).len(), 1);
        assert_eq!(lint("let x = weapon.braton.prime;", &names).len(), 1);
        assert_eq!(lint("let ash_shield = 1;", &names).len(), 1);
        // Whole words only: `hash`, "Cashier" and `dash` are not Ash; item names in comments are citations.
        assert!(
            lint(
                "let hash = dash(\"Cashier\"); // Braton Prime (spec example)",
                &names
            )
            .is_empty()
        );
        // Separate statements and other punctuation break an identifier run.
        assert!(lint("let braton = 1; prime(braton, prime);", &names).is_empty());
    }

    /// Generated `CompatFlags` names and stat-registry identifiers spell items (D-007 Phoenix Talons, 01 §4.9 `pt`):
    /// they are model vocabulary, exempt as whole identifiers (domain-model.md §9.5, §10).
    #[test]
    fn model_vocabulary_is_exempt() {
        let vocabulary = model_vocabulary();
        for id in [
            "D007_PHOENIX_TALONS",
            "PhoenixTalons",
            "phoenix_talons",
            "HeatInherit",
            "multishot",
        ] {
            assert!(vocabulary.contains(&normalize(id)), "{id}");
        }
        let names = [
            "Phoenix Talons",
            "Heat Inherit",
            "Multishot",
            "Secondary Shiver",
            "Primary Plated Round",
            "Hunter Munitions",
            "Spring-Loaded Broadhead",
            "Dreamer's Wrath",
        ];
        let src = "
            use wfd_model::{CompatFlags, StatKey};
            pub mod multishot;
            pub fn multishot(ms0: f64) -> f64 { ms0 }
            fn stage(ctx: &StageCtx, sv: &StatVecF) {
                if ctx.compat.contains(CompatFlags::D007_PHOENIX_TALONS) {}
                if ctx.compat.contains(CompatFlags::D060_SECONDARY_SHIVER) {}
                let pt = sv[StatKey::PhoenixTalons] + sv[StatKey::HeatInherit];
                let k = StatKey::parse(\"heat_inherit\");
            }
        ";
        let flagged = |src: &str| lint(src, &names);
        assert_eq!(flagged(src), Vec::<String>::new());
        for flag in wfd_model::CompatFlags::all_names() {
            assert!(
                flagged(&format!("if c.contains(CompatFlags::{flag}) {{}}")).is_empty(),
                "{flag}"
            );
        }
        // Only the whole identifier is exempt: a longer identifier that spells the item is item code.
        assert_eq!(flagged("let phoenix_talons_hack = 1;").len(), 1);
        assert_eq!(flagged("let x = \"Secondary Shiver\";").len(), 1);
        assert_eq!(flagged("fn hunter_munitions() {}").len(), 1);
    }

    /// LINT-GLAIVE (contract review): melee class names, plan kinds, spec symbols and mechanic ids are vocabulary,
    /// read from the model and from `docs/mechanics/` at lint time; an identifier that spells an item around them still
    /// fails.
    #[test]
    fn melee_classes_plans_and_spec_identifiers_are_exempt() {
        let model = model_vocabulary();
        for id in [
            "Glaive",
            "Machete",
            "HeavyBlade",
            "Heavy Blade",
            "GlaiveCycle",
            "glaive_cycle",
        ] {
            assert!(model.contains(&normalize(id)), "{id}");
        }
        let spec = spec_vocabulary(&crate::workspace_root()).unwrap();
        for id in [
            "glaive_throw_ct",
            "glaive_throw_index",
            "glaive_volatile_chance",
            "glaive_volatile",
            "part_sonar_mult",
            "sonar_mult",
            "crit_roll",
            "hit_frac",
            "base_listed",
        ] {
            assert!(spec.contains(&normalize(id)), "{id}");
        }
        // Only whole identifiers: the parts of a derived id ("xh/xata/sc_count") are not vocabulary.
        assert!(!spec.contains("xata"));
        let mut vocabulary = model;
        vocabulary.extend(spec);
        let names: BTreeSet<String> = ["Glaive", "Glaive Prime", "Machete", "Sonar"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        let flagged = |src: &str| check_scan(&scan_source(src), &names, &vocabulary, false);
        let clean = "
            fn plan(c: MeleeClass, k: PlanKind) -> bool {
                matches!(c, MeleeClass::Glaive | MeleeClass::Machete) || k == PlanKind::GlaiveCycle
            }
            fn keys(ct: f64) { let glaive_throw_ct = ct; let id = rng_mech(\"glaive_volatile\"); }
            fn parts(sv: &StatVecF) -> f64 { sv.part_sonar_mult * sv.sonar_mult }
        ";
        assert_eq!(flagged(clean), Vec::<String>::new());
        // `glaive_prime` spells both "Glaive" and "Glaive Prime"; neither run lies inside an exempt identifier.
        for src in ["let glaive_prime = 1;", "let n = \"Glaive Prime\";"] {
            let p = flagged(src);
            assert!(
                p.iter().any(|m| m.contains("\"Glaive Prime\"")),
                "{src}: {p:?}"
            );
        }
        assert_eq!(flagged("fn sonar_bonus() {}").len(), 1);
    }

    #[test]
    fn spec_tables_and_draw_ids_are_read() {
        let doc = [
            "| ID | Symbol | Unit | Meaning |",
            "|---|---|---|---|",
            "| `base_listed[t]` | B0_t | HP | x |",
            "| `heavy_windup_base`, `heavy_windup` | WU0, WU | s | y |",
            "",
            "| ID | Rule | Source |",
            "|---|---|---|",
            "| `not_a_symbol` | r | s |",
            "```text",
            "| ID | Symbol | Unit | Meaning |",
            "|---|---|---|---|",
            "| `in_code` | – | – | – |",
            "```",
        ];
        let ids = symbol_table_ids(&doc);
        assert_eq!(
            ids.into_iter().collect::<Vec<_>>(),
            ["base_listed", "heavy_windup", "heavy_windup_base"]
        );
        let doc = [
            "intro \"before\"",
            "E-8.16 Counter-based RNG",
            "  melee draws (07: \"combo_acc\", \"glaive_volatile\")",
            "  derived: \"xh/<source>/<name>\" and \"xh/xata/sc_count\"",
            "draw order \"crit_roll\", \"st\"",
            "### 6.2 Run state",
            "\"after\"",
        ];
        assert_eq!(
            mechanic_id_strings(&doc).into_iter().collect::<Vec<_>>(),
            ["combo_acc", "crit_roll", "glaive_volatile", "st"]
        );
    }

    #[test]
    fn module_file_resolution() {
        let m = |name: &str, path: Option<&str>| TestModule {
            name: name.into(),
            path: path.map(Into::into),
        };
        let p = Path::new("/r/src/lib.rs");
        assert_eq!(
            module_files(p, &m("tests", None))[0],
            Path::new("/r/src/tests.rs")
        );
        let p = Path::new("/r/src/ev/kernel.rs");
        assert_eq!(
            module_files(p, &m("tests", None)),
            [
                Path::new("/r/src/ev/kernel/tests.rs"),
                Path::new("/r/src/ev/kernel/tests/mod.rs")
            ]
        );
        // `#[path]` is relative to the directory of the declaring file.
        assert_eq!(
            module_files(p, &m("more", Some("t.rs"))),
            [Path::new("/r/src/ev/t.rs")]
        );
        assert_eq!(
            module_dir(Path::new("/r/src/ev/mod.rs")),
            Path::new("/r/src/ev")
        );
    }

    fn write_tree(root: &Path, files: &[(&str, &str)]) {
        let _ = std::fs::remove_dir_all(root);
        for (rel, text) in files {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
    }

    /// A fixture workspace on disk: spec_constants.rs exemption, test-module files (incl. `#[path]` and
    /// `#![cfg(test)]`), fixture names via names/en.json, custom-stat records.
    #[test]
    fn workspace_fixture() {
        let root = std::env::temp_dir().join(format!("wfd-lint-engine-{}", std::process::id()));
        write_tree(
            &root,
            &[
                (
                    "crates/wfd-engine/src/lib.rs",
                    "mod spec_constants;\n#[cfg(test)]\nmod tests;\n#[cfg(test)]\n#[path = \"testdata/helpers.rs\"]\n\
                     mod helpers;\nmod only_tests;\npub fn f() -> f64 { 1.0 }\n",
                ),
                (
                    "crates/wfd-engine/src/spec_constants.rs",
                    "/// E-5.2\npub const SMOOTH: f64 = 0.4375;\n",
                ),
                ("crates/wfd-engine/src/tests.rs", "const X: f64 = 0.123;\n"),
                (
                    "crates/wfd-engine/src/tests/deep.rs",
                    "const Y: f64 = 0.124;\n",
                ),
                (
                    "crates/wfd-engine/src/testdata/helpers.rs",
                    "const Z: f64 = 0.125;\n",
                ),
                (
                    "crates/wfd-engine/src/only_tests.rs",
                    "#![cfg(test)]\nconst W: f64 = 0.126;\n",
                ),
                (
                    "crates/wfd-opt/src/lib.rs",
                    "pub fn g() -> f64 {\n    0.3\n}\nconst N: &str = \"Lanka\";\npub mod multishot;\n",
                ),
                (
                    "data/fixtures/mini/weapons/primary.json",
                    r#"[{"unique_name": "/Lotus/Weapons/Lanka", "name": "/Lotus/Weapons/Lanka"}]"#,
                ),
                (
                    "data/fixtures/mini/items.json",
                    r#"[{"unique_name": "wfd:custom/multishot", "name": "custom.multishot",
                         "kind": {"custom_stat": {"inspiration_key": "p.q"}}}]"#,
                ),
                (
                    "data/fixtures/mini/names/en.json",
                    r#"{"/Lotus/Weapons/Lanka": "Lanka", "custom.multishot": "Multishot"}"#,
                ),
            ],
        );
        let report = lint_workspace(&root).unwrap();
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            (
                report.files,
                report.test_files,
                report.names,
                report.custom_stats
            ),
            (3, 4, 1, 1)
        );
        assert_eq!(report.problems.len(), 2, "{:?}", report.problems);
        assert!(report.problems[0].starts_with("crates/wfd-opt/src/lib.rs:2: float literal `0.3`"));
        assert!(report.problems[1].starts_with(
            "crates/wfd-opt/src/lib.rs:4: string literal spells the fixture item name \"Lanka\""
        ));
    }

    /// Empty crates pass (work-breakdown.md T01c acceptance).
    #[test]
    fn empty_crates_pass() {
        let root = std::env::temp_dir().join(format!("wfd-lint-empty-{}", std::process::id()));
        let files: Vec<(String, &str)> = LINTED_CRATES
            .iter()
            .map(|k| (format!("{k}/src/lib.rs"), "//! skeleton\n"))
            .collect();
        let files: Vec<(&str, &str)> = files.iter().map(|(p, t)| (p.as_str(), *t)).collect();
        write_tree(&root, &files);
        let report = lint_workspace(&root).unwrap();
        let _ = std::fs::remove_dir_all(&root);
        assert!(report.problems.is_empty());
        assert_eq!(report.files, 2);
    }
}
