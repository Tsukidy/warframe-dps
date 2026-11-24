//! `cargo xtask lint-engine` (work-breakdown.md T01c; overview.md §6.1 I-1, I-2; testing.md §2.5).
//!
//! Scans the library sources of `wfd-engine` and `wfd-opt` (`crates/<crate>/src/**/*.rs`; integration tests and
//! benches live outside `src/` and are not scanned) with a small Rust lexer, skipping comments and test code
//! (`#[cfg(test)]` and `#[test]` items, and the files of `#[cfg(test)] mod name;` declarations):
//!
//! - **I-1**: a float literal whose value is not 0, 1, 2 or 0.5 is an error outside
//!   `crates/wfd-engine/src/spec_constants.rs` (game numbers come from the bundle; spec formula constants live in that
//!   file, each with its E-number). Integer literals with an `f32`/`f64` suffix are float literals.
//! - **I-2**: no `/Lotus/` in code or string literals, and no item name of the mini fixture (`data/fixtures/mini/`,
//!   the display names of every record with a `unique_name`, resolved through `names/*.json`): in a string literal as
//!   a whole word, or as a run of identifier words (`braton_prime`, `BratonPrime`). Comments are not scanned for I-2:
//!   code cites the spec, whose examples name items; a comment is not a branch.
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
        "lint-engine: {} source file(s) in {} ({} excluded as test code), {} item name(s) from {MINI_FIXTURE}",
        report.files,
        LINTED_CRATES.join(", "),
        report.test_files,
        report.names
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
    /// `path:line: message`.
    pub problems: Vec<String>,
}

/// Lints the workspace at `root`.
pub fn lint_workspace(root: &Path) -> Result<Report> {
    let names = fixture_names(&root.join(MINI_FIXTURE))?;
    let mut report = Report {
        names: names.len(),
        ..Report::default()
    };
    let mut files = Vec::new();
    for krate in LINTED_CRATES {
        collect_rs(&root.join(krate).join("src"), &mut files)?;
    }
    files.sort();
    // Test-only module files, found while scanning their parents (parents sort before their children).
    let mut test_files: BTreeSet<PathBuf> = BTreeSet::new();
    let mut scans = Vec::new();
    for file in &files {
        let text =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        let scan = scan_source(&text);
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
            .any(|t| file == *t || file.starts_with(t.with_extension("")))
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
        for p in check_scan(&scan, &names, exempt_floats) {
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

/// The candidate files of `mod name;` declared in `parent`: `name.rs` and `name/mod.rs` next to `parent` for
/// `lib.rs`, `main.rs` and `mod.rs`, else under the directory named after `parent`.
pub fn module_files(parent: &Path, name: &str) -> Vec<PathBuf> {
    let dir = parent.parent().unwrap_or(Path::new("."));
    let stem = parent.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let base = if matches!(stem, "lib" | "main" | "mod") {
        dir.to_path_buf()
    } else {
        dir.join(stem)
    };
    vec![
        base.join(format!("{name}.rs")),
        base.join(name).join("mod.rs"),
    ]
}

// ------------------------------------------------------------------------------------------------------------------
// Fixture names
// ------------------------------------------------------------------------------------------------------------------

/// Display names of the fixture's records (every JSON object with a string `unique_name`/`uniqueName` and `name`),
/// resolved through the string maps of `names/*.json`. Missing directory: no names.
pub fn fixture_names(dir: &Path) -> Result<BTreeSet<String>> {
    let mut jsons = Vec::new();
    collect_ext(dir, "json", &mut jsons)?;
    let mut table: BTreeMap<String, String> = BTreeMap::new();
    let mut keys: Vec<String> = Vec::new();
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
    let mut names = BTreeSet::new();
    for key in keys {
        let name = table.get(&key).cloned().unwrap_or(key);
        if !name.starts_with('/') && normalize(&name).len() >= 3 {
            names.insert(name);
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

fn collect_named(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(m) => {
            let unique = m.get("unique_name").or_else(|| m.get("uniqueName"));
            if let (Some(serde_json::Value::String(_)), Some(serde_json::Value::String(name))) =
                (unique, m.get("name"))
            {
                out.push(name.clone());
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

// ------------------------------------------------------------------------------------------------------------------
// Lexer
// ------------------------------------------------------------------------------------------------------------------

/// A token of the non-test code.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    /// Literal text and whether it is a float literal.
    Num(String, bool),
    /// String literal contents (raw text between the quotes).
    Str(String),
    Char,
    Lifetime,
    Punct(char),
}

/// Lexes Rust source into tokens with 1-based line numbers. Comments are dropped.
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
            while i < n && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '/' && c.get(i + 1) == Some(&'*') {
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
            let tuple_index = matches!(out.last(), Some((_, Tok::Punct('.'))))
                && !matches!(
                    out.len().checked_sub(2).map(|k| &out[k].1),
                    Some(Tok::Punct('.'))
                );
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

/// The tokens of a file outside test code, and the names of `#[cfg(test)] mod name;` declarations.
#[derive(Debug, Default)]
pub struct Scan {
    pub tokens: Vec<(usize, Tok)>,
    pub test_modules: Vec<String>,
}

/// Is the attribute token list (between `#[` and `]`) test-only: `test`, `cfg(test)`, `cfg(all(test, …))`,
/// `tokio::test` …? `cfg(not(test))` is not.
fn is_test_attr(attr: &[(usize, Tok)]) -> bool {
    let idents: Vec<&str> = attr
        .iter()
        .filter_map(|(_, t)| {
            if let Tok::Ident(s) = t {
                Some(s.as_str())
            } else {
                None
            }
        })
        .collect();
    match idents.first() {
        Some(&"cfg") => idents.contains(&"test") && !idents.contains(&"not"),
        _ => idents.last() == Some(&"test"),
    }
}

/// Removes test items from a file's tokens.
pub fn scan_source(src: &str) -> Scan {
    let toks = lex(src);
    let mut scan = Scan::default();
    let mut i = 0;
    while i < toks.len() {
        if let Some(attr_end) = attribute_end(&toks, i)
            && is_test_attr(&toks[i + 2..attr_end])
        {
            let (end, module) = skip_item(&toks, attr_end + 1);
            if let Some(m) = module {
                scan.test_modules.push(m);
            }
            i = end;
            continue;
        }
        scan.tokens.push(toks[i].clone());
        i += 1;
    }
    scan
}

/// If `toks[i..]` starts an outer attribute `#[…]`, the index of its closing `]`.
fn attribute_end(toks: &[(usize, Tok)], i: usize) -> Option<usize> {
    if toks.get(i).map(|t| &t.1) != Some(&Tok::Punct('#'))
        || toks.get(i + 1).map(|t| &t.1) != Some(&Tok::Punct('['))
    {
        return None;
    }
    let mut depth = 0;
    for (k, (_, t)) in toks.iter().enumerate().skip(i + 1) {
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
    while let Some(end) = attribute_end(toks, i) {
        i = end + 1;
    }
    let start = i;
    let mut depth = 0i32;
    while i < toks.len() {
        match &toks[i].1 {
            Tok::Punct('(' | '[') => depth += 1,
            Tok::Punct(')' | ']') => depth -= 1,
            Tok::Punct(';') if depth == 0 => {
                let module = match (
                    toks.get(i.wrapping_sub(2)).map(|t| &t.1),
                    toks.get(i - 1).map(|t| &t.1),
                ) {
                    (Some(Tok::Ident(kw)), Some(Tok::Ident(name)))
                        if kw == "mod" && i >= start + 2 =>
                    {
                        Some(name.clone())
                    }
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

/// Does `hay` contain `needle` as a whole word (no alphanumeric neighbour)?
fn contains_word(hay: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(pos) = hay[from..].find(needle) {
        let s = from + pos;
        let e = s + needle.len();
        let before = hay[..s]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        let after = hay[e..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        if !before && !after {
            return true;
        }
        from = s + needle.len().max(1);
    }
    false
}

/// I-1 and I-2 problems of one scanned file, as `line: message`.
pub fn check_scan(scan: &Scan, names: &BTreeSet<String>, exempt_floats: bool) -> Vec<String> {
    let mut problems = Vec::new();
    let keys: Vec<(String, &String)> = names.iter().map(|n| (normalize(n), n)).collect();
    // Identifier words in token order, for runs of words across identifiers and paths (`braton_prime`, `Braton::Prime`).
    let mut words: Vec<(usize, String)> = Vec::new();
    for (line, tok) in &scan.tokens {
        match tok {
            Tok::Num(text, true) if !exempt_floats => {
                let v = float_value(text);
                if !v.is_some_and(|v| ALLOWED_FLOATS.contains(&v)) {
                    problems.push(format!(
                        "{line}: float literal `{text}` (I-1: only 0, 1, 2 and 0.5 outside {SPEC_CONSTANTS}; game \
                         numbers come from the bundle, spec constants go to spec_constants.rs with their E-number)"
                    ));
                }
            }
            Tok::Str(s) => {
                if s.contains("/Lotus/") {
                    problems.push(format!(
                        "{line}: `/Lotus/` in a string literal (I-2: no item-specific code)"
                    ));
                }
                for (_, name) in &keys {
                    if contains_word(s, name) {
                        problems.push(format!(
                            "{line}: fixture item name \"{name}\" in a string literal (I-2: no item-specific code)"
                        ));
                    }
                }
            }
            Tok::Ident(id) => {
                for w in ident_words(id) {
                    words.push((*line, w));
                }
            }
            _ => {}
        }
    }
    let mut reported = BTreeSet::new();
    for start in 0..words.len() {
        let mut acc = String::new();
        for (line, w) in &words[start..] {
            acc.push_str(&normalize(w));
            if let Some((_, name)) = keys.iter().find(|(k, _)| *k == acc)
                && reported.insert((*line, (*name).clone()))
            {
                problems.push(format!(
                    "{line}: identifier spells the fixture item name \"{name}\" (I-2: no item-specific code)"
                ));
            }
            if !keys.iter().any(|(k, _)| k.starts_with(acc.as_str())) {
                break;
            }
        }
    }
    problems
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
        check_scan(&scan_source(src), &names, false)
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
            #[cfg(not(test))]
            const LIVE: f64 = 0.6;
        ";
        let scan = scan_source(src);
        assert_eq!(scan.test_modules, ["more"]);
        assert_eq!(floats(src), ["0.0", "0.6"]);
    }

    #[test]
    fn lotus_paths_and_item_names() {
        let names = ["Braton Prime", "Ash"];
        assert!(lint("let p = \"/Lotus/Weapons/Tenno/Rifle\";", &names)[0].contains("/Lotus/"));
        assert!(lint("// /Lotus/ in a comment is a citation\nlet x = 1;", &names).is_empty());
        assert_eq!(lint("if name == \"Braton Prime\" {}", &names).len(), 1);
        assert_eq!(lint("let braton_prime = 1;", &names).len(), 1);
        assert_eq!(lint("struct BratonPrime;", &names).len(), 1);
        assert_eq!(lint("let ash_shield = 1;", &names).len(), 1);
        // Whole words only: `hash`, "Cashier" and `dash` are not Ash; comments are not scanned.
        assert!(
            lint(
                "let hash = dash(\"Cashier\"); // Braton Prime (spec example)",
                &names
            )
            .is_empty()
        );
    }

    #[test]
    fn module_file_resolution() {
        let p = Path::new("/r/src/lib.rs");
        assert_eq!(module_files(p, "tests")[0], Path::new("/r/src/tests.rs"));
        let p = Path::new("/r/src/ev/kernel.rs");
        assert_eq!(
            module_files(p, "tests")[0],
            Path::new("/r/src/ev/kernel/tests.rs")
        );
        assert_eq!(
            module_files(p, "tests")[1],
            Path::new("/r/src/ev/kernel/tests/mod.rs")
        );
    }

    /// A fixture workspace on disk: spec_constants.rs exemption, test-module files, fixture names via names/en.json.
    #[test]
    fn workspace_fixture() {
        let root = std::env::temp_dir().join(format!("wfd-lint-engine-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let w = |rel: &str, text: &str| {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        w(
            "crates/wfd-engine/src/lib.rs",
            "mod spec_constants;\n#[cfg(test)]\nmod tests;\npub fn f() -> f64 { 1.0 }\n",
        );
        w(
            "crates/wfd-engine/src/spec_constants.rs",
            "/// E-5.2\npub const SMOOTH: f64 = 0.4375;\n",
        );
        w("crates/wfd-engine/src/tests.rs", "const X: f64 = 0.123;\n");
        w(
            "crates/wfd-opt/src/lib.rs",
            "pub fn g() -> f64 {\n    0.3\n}\nconst N: &str = \"Lanka\";\n",
        );
        w(
            "data/fixtures/mini/weapons/primary.json",
            r#"[{"unique_name": "/Lotus/Weapons/Lanka", "name": "/Lotus/Weapons/Lanka"}]"#,
        );
        w(
            "data/fixtures/mini/names/en.json",
            r#"{"/Lotus/Weapons/Lanka": "Lanka"}"#,
        );
        let report = lint_workspace(&root).unwrap();
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!((report.files, report.test_files, report.names), (3, 1, 1));
        assert_eq!(report.problems.len(), 2, "{:?}", report.problems);
        assert!(report.problems[0].starts_with("crates/wfd-opt/src/lib.rs:2: float literal `0.3`"));
        assert!(
            report.problems[1]
                .starts_with("crates/wfd-opt/src/lib.rs:4: fixture item name \"Lanka\"")
        );
    }

    /// Empty crates pass (work-breakdown.md T01c acceptance).
    #[test]
    fn empty_crates_pass() {
        let root = std::env::temp_dir().join(format!("wfd-lint-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for k in LINTED_CRATES {
            std::fs::create_dir_all(root.join(k).join("src")).unwrap();
            std::fs::write(root.join(k).join("src/lib.rs"), "//! skeleton\n").unwrap();
        }
        let report = lint_workspace(&root).unwrap();
        let _ = std::fs::remove_dir_all(&root);
        assert!(report.problems.is_empty());
        assert_eq!(report.files, 2);
    }
}
