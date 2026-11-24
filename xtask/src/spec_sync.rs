//! `cargo xtask spec-sync [--check]` (work-breakdown.md T01c; domain-model.md §9.3, §9.5; testing.md §2.5; I-7).
//!
//! 1. Parses the discrepancy register of spec 00 §5.2 and the "Inspiration differences" register of every topic
//!    document 01-09, and generates `crates/wfd-model/src/compat_generated.rs`: exactly one `CompatFlags` flag per
//!    register row whose Toggle column has the switch kind `Compat` (spec 00 §5.1 rule 3, below; domain-model.md
//!    §9.5 words it as "contains `Compat`", request C-20), each row once even when several documents restate it
//!    (register order: 00 first, then the rows only 01 … 09 list), plus `INSPIRATION` (every flag).
//! 2. Checks `wfd_model::scenario::TOGGLE_MAP` against the toggle table of spec 00 §6.1: every T-nn of the spec present
//!    exactly once; `Proposed` entries are allowed for ids the spec does not list.
//!
//! Without `--check` the generated file is written when it differs; with `--check` nothing is written and a stale file
//! is an error. Both modes fail on a toggle mismatch. The flag set is data-driven: whatever `Compat` rows the
//! committed registers hold.
//!
//! Toggle column (spec 00 §5.1 rule 3): the switch kinds are the first words of the `+`-separated parts of the cell
//! before any parenthesis, `:`, `;` or `,`: `User`, `Compat` or `—`. `User (T-19) + Compat` and `Compat: …` are Compat rows;
//! `— (Compat path of D-065, …)` and `User (T-26); a compat flag is proposed …` are not. When registers disagree on
//! whether a row is Compat, the row gets a flag (any register marks it) and the disagreement is printed as a warning.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use wfd_model::scenario::{TOGGLE_MAP, TogglePlace, ToggleStatus};

/// Path of the generated file, relative to the workspace root.
pub const GENERATED_PATH: &str = "crates/wfd-model/src/compat_generated.rs";
/// Directory of the spec documents, relative to the workspace root.
pub const MECHANICS_DIR: &str = "docs/mechanics";
/// Capacity of `CompatFlags` (u128, domain-model.md §9.5).
pub const MAX_FLAGS: usize = 128;

#[derive(clap::Parser, Debug)]
#[command(
    name = "cargo xtask spec-sync",
    about = "Generate CompatFlags and check TOGGLE_MAP (T01c)"
)]
struct Args {
    /// Do not write; fail when the generated file is stale.
    #[arg(long)]
    check: bool,
    /// Print the generated flags (bit, id, name, registers).
    #[arg(long)]
    list: bool,
    /// Workspace root to read (default: this workspace). Tests use a copy.
    #[arg(long)]
    root: Option<PathBuf>,
}

/// Entry point (`cargo xtask spec-sync`).
pub fn run(args: &[String]) -> Result<()> {
    let args: Args = crate::parse_args("spec-sync", args);
    let root = args.root.unwrap_or_else(crate::workspace_root);
    let outcome = sync(&root, args.check)?;
    if args.list {
        for (bit, f) in outcome.flags.iter().enumerate() {
            println!(
                "{bit:>3}  {:<7} {:<40} owner {:<7} registers {}",
                f.id,
                f.flag,
                f.owner,
                f.docs.join(", ")
            );
        }
    }
    for w in &outcome.warnings {
        eprintln!("warning: {w}");
    }
    println!("{}", outcome.summary);
    let mut problems = outcome.compat_problems.clone();
    problems.extend(outcome.toggle_problems.iter().cloned());
    if problems.is_empty() {
        return Ok(());
    }
    for p in &problems {
        eprintln!("  - {p}");
    }
    Err(anyhow!(
        "spec-sync: {} problem(s): {} with the compat flags, {} with TOGGLE_MAP vs spec 00 §6.1",
        problems.len(),
        outcome.compat_problems.len(),
        outcome.toggle_problems.len()
    ))
}

/// Result of one spec-sync pass.
#[derive(Debug, Default)]
pub struct Outcome {
    /// One line: rows parsed, flags generated, file status.
    pub summary: String,
    /// The `Compat` rows, in bit order.
    pub flags: Vec<CompatRow>,
    /// Stale or missing generated file, rows without a flag, flags without a row.
    pub compat_problems: Vec<String>,
    /// `TOGGLE_MAP` against spec 00 §6.1.
    pub toggle_problems: Vec<String>,
    /// Register disagreements (not failures).
    pub warnings: Vec<String>,
}

/// Runs the generator (and writes the file unless `check`) and both checks on the workspace at `root`.
pub fn sync(root: &Path, check: bool) -> Result<Outcome> {
    let docs = read_mechanics(root)?;
    let mut registers = Vec::new();
    for (doc, text) in &docs {
        registers.push((doc.clone(), parse_register(doc, text)?));
    }
    let (flags, warnings) = compat_rows(&registers)?;
    let generated = render_compat(&flags);
    let path = root.join(GENERATED_PATH);
    let current = std::fs::read_to_string(&path).ok();
    let mut compat_problems = Vec::new();
    let file_status;
    if check {
        compat_problems = check_compat(&flags, &generated, current.as_deref());
        file_status = if compat_problems.is_empty() {
            "up to date"
        } else {
            "STALE"
        };
    } else if current.as_deref() == Some(generated.as_str()) {
        file_status = "up to date";
    } else {
        std::fs::write(&path, &generated).with_context(|| format!("writing {}", path.display()))?;
        file_status = "written";
    }
    let spec_toggles = parse_toggle_table(&docs[0].1)?;
    let toggle_problems = check_toggles(&spec_toggles, TOGGLE_MAP);
    let rows: usize = registers.iter().map(|(_, r)| r.len()).sum();
    let summary = format!(
        "spec-sync: {} register rows in {} registers; {} Compat rows → {} flags (bits 0-{}), {GENERATED_PATH} {}; \
         spec 00 §6.1 lists {} toggles, TOGGLE_MAP has {} entries ({} mismatch(es))",
        rows,
        registers.len(),
        flags.len(),
        flags.len(),
        flags.len().saturating_sub(1),
        file_status,
        spec_toggles.len(),
        TOGGLE_MAP.len(),
        toggle_problems.len(),
    );
    Ok(Outcome {
        summary,
        flags,
        compat_problems,
        toggle_problems,
        warnings,
    })
}

/// Reads `docs/mechanics/0N-*.md` for N = 0 … 9: exactly one file per number, in order. Returns `("00", text)` ….
pub fn read_mechanics(root: &Path) -> Result<Vec<(String, String)>> {
    let dir = root.join(MECHANICS_DIR);
    let mut found: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for entry in std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let b = name.as_bytes();
        if name.ends_with(".md")
            && b.len() > 3
            && b[0] == b'0'
            && b[1].is_ascii_digit()
            && b[2] == b'-'
        {
            found
                .entry(name[..2].to_owned())
                .or_default()
                .push(path.clone());
        }
    }
    let mut out = Vec::new();
    for n in 0..10 {
        let doc = format!("0{n}");
        let files = found.remove(&doc).unwrap_or_default();
        let [file] = files.as_slice() else {
            bail!(
                "expected exactly one {}/{doc}-*.md, found {}",
                dir.display(),
                files.len()
            );
        };
        let text =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        out.push((doc, text));
    }
    Ok(out)
}

// ------------------------------------------------------------------------------------------------------------------
// Markdown tables
// ------------------------------------------------------------------------------------------------------------------

/// A Markdown table: the 1-based line of its header, the header cells and the data rows with their line numbers.
#[derive(Debug, Clone)]
pub struct Table {
    /// 1-based line of the header row.
    pub line: usize,
    /// Header cells, trimmed.
    pub header: Vec<String>,
    /// Data rows: 1-based line and cells (as many as the header).
    pub rows: Vec<(usize, Vec<String>)>,
}

/// Splits a table line into cells. `|` inside a code span or escaped as `\|` does not separate cells.
pub fn split_cells(line: &str) -> Vec<String> {
    let s = line.trim();
    let s = s.strip_prefix('|').unwrap_or(s);
    let s = s.strip_suffix('|').unwrap_or(s);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut in_code = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                cur.push('|');
                chars.next();
            }
            '`' => {
                in_code = !in_code;
                cur.push(c);
            }
            '|' if !in_code => cells.push(std::mem::take(&mut cur).trim().to_owned()),
            _ => cur.push(c),
        }
    }
    cells.push(cur.trim().to_owned());
    cells
}

/// The tables of `lines[start..end]` (`lines` 0-based; reported line numbers are 1-based).
pub fn tables_in(lines: &[&str], start: usize, end: usize) -> Result<Vec<Table>> {
    let mut tables = Vec::new();
    let mut i = start;
    while i < end {
        if !lines[i].trim_start().starts_with('|') {
            i += 1;
            continue;
        }
        let header = split_cells(lines[i]);
        let sep = lines.get(i + 1).map(|l| split_cells(l)).unwrap_or_default();
        let is_sep = !sep.is_empty()
            && sep
                .iter()
                .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':')));
        if !is_sep {
            bail!("line {}: table header without a separator row", i + 1);
        }
        let mut table = Table {
            line: i + 1,
            header,
            rows: Vec::new(),
        };
        let mut j = i + 2;
        while j < end && lines[j].trim_start().starts_with('|') {
            let cells = split_cells(lines[j]);
            if cells.len() != table.header.len() {
                bail!(
                    "line {}: {} cells, the header at line {} has {}",
                    j + 1,
                    cells.len(),
                    i + 1,
                    table.header.len()
                );
            }
            table.rows.push((j + 1, cells));
            j += 1;
        }
        tables.push(table);
        i = j;
    }
    Ok(tables)
}

/// Heading level (number of leading `#`) of a Markdown heading line outside code blocks.
fn heading_level(line: &str) -> Option<usize> {
    let n = line.bytes().take_while(|&b| b == b'#').count();
    (n > 0 && line.as_bytes().get(n) == Some(&b' ')).then_some(n)
}

/// The 0-based line range of the section that starts at the heading matching `is_start`, up to the next heading of the
/// same or a higher level. Exactly one such heading must exist (fenced code is skipped).
fn section(lines: &[&str], what: &str, is_start: impl Fn(&str) -> bool) -> Result<(usize, usize)> {
    let mut in_code = false;
    let mut start = None;
    let mut end = lines.len();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        let Some(level) = heading_level(line) else {
            continue;
        };
        match start {
            None if is_start(line) => start = Some((i, level)),
            Some((s, l)) if level <= l => {
                if is_start(line) {
                    bail!("two {what} headings (lines {} and {})", s + 1, i + 1);
                }
                end = i;
                break;
            }
            _ => {}
        }
    }
    // A second matching heading after the end would be a duplicate too.
    let (s, _) = start.ok_or_else(|| anyhow!("no {what} heading"))?;
    if lines[end..]
        .iter()
        .any(|l| heading_level(l).is_some() && is_start(l))
    {
        bail!("more than one {what} heading");
    }
    Ok((s + 1, end))
}

// ------------------------------------------------------------------------------------------------------------------
// Registers
// ------------------------------------------------------------------------------------------------------------------

/// Switch kinds of a register row's Toggle cell (spec 00 §5.1 rule 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Switch {
    /// A `User` part: a user toggle (T-nn) selects the behaviour.
    pub user: bool,
    /// A `Compat` part: the row gets a `CompatFlags` flag.
    pub compat: bool,
}

/// One row of a register (a combined `D-030, D-031` cell yields one row per id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterRow {
    /// `D-001`.
    pub id: String,
    /// Document number, `"00"` … `"09"`.
    pub doc: String,
    /// 1-based line in the document.
    pub line: usize,
    /// Topic cell as written.
    pub topic: String,
    /// Toggle cell as written.
    pub toggle: String,
    /// Switch kinds of the Toggle cell ([`classify_toggle`]).
    pub switch: Switch,
}

/// Classifies a Toggle cell; see the module documentation.
pub fn classify_toggle(cell: &str) -> Result<Switch> {
    let mut head = String::new();
    let mut depth = 0usize;
    let mut in_code = false;
    for c in cell.chars() {
        match c {
            '`' => in_code = !in_code,
            '(' if !in_code => depth += 1,
            ')' if !in_code => depth = depth.saturating_sub(1),
            ':' | ';' | ',' if depth == 0 && !in_code => break,
            _ if depth == 0 && !in_code => head.push(c),
            _ => {}
        }
    }
    let mut sw = Switch::default();
    for part in head.split('+') {
        // The first word of each part is the switch kind; free text may follow (`— for both halves`).
        match part.split_whitespace().next().unwrap_or("") {
            "User" => sw.user = true,
            "Compat" => sw.compat = true,
            "—" | "-" | "–" => {}
            other => bail!(
                "unrecognised Toggle cell `{cell}` (switch word `{other}`; expected User, Compat or —)"
            ),
        }
    }
    Ok(sw)
}

/// Parses the register of one document: 00 §5.2, or the `## N. Inspiration differences` section of 01-09. Every table
/// of the section whose header starts with `ID` and has a Toggle column is a register table.
pub fn parse_register(doc: &str, text: &str) -> Result<Vec<RegisterRow>> {
    let lines: Vec<&str> = text.lines().collect();
    let (start, end) = if doc == "00" {
        section(&lines, "00 §5.2 Register", |l| l.starts_with("### 5.2 "))
    } else {
        let re = Regex::new(r"^##\s+\d+\.\s+Inspiration differences\s*$").expect("static regex");
        section(&lines, "Inspiration differences", |l| re.is_match(l))
    }
    .with_context(|| format!("{doc}: locating the register"))?;
    let id_cell =
        Regex::new(r"^D-\d{3}[a-z]?(\s*(,|and)\s*D-\d{3}[a-z]?)*$").expect("static regex");
    let id_re = Regex::new(r"D-\d{3}[a-z]?").expect("static regex");
    let mut rows = Vec::new();
    let mut n_tables = 0;
    for table in tables_in(&lines, start, end).with_context(|| format!("{doc}: register tables"))? {
        if table.header.first().map(String::as_str) != Some("ID") {
            continue;
        }
        let Some(toggle_col) = table.header.iter().position(|h| h.starts_with("Toggle")) else {
            continue;
        };
        let topic_col = table
            .header
            .iter()
            .position(|h| h.starts_with("Topic"))
            .ok_or_else(|| {
                anyhow!(
                    "{doc}:{}: register table without a Topic column",
                    table.line
                )
            })?;
        n_tables += 1;
        for (line, cells) in &table.rows {
            let ids = &cells[0];
            if !id_cell.is_match(ids) {
                bail!("{doc}:{line}: register ID cell `{ids}` is not a D-nnn id list");
            }
            let toggle = &cells[toggle_col];
            let switch = classify_toggle(toggle).with_context(|| format!("{doc}:{line}"))?;
            for id in id_re.find_iter(ids) {
                rows.push(RegisterRow {
                    id: id.as_str().to_owned(),
                    doc: doc.to_owned(),
                    line: *line,
                    topic: cells[topic_col].clone(),
                    toggle: toggle.clone(),
                    switch,
                });
            }
        }
    }
    if n_tables == 0 {
        bail!("{doc}: the register section has no table with ID and Toggle columns");
    }
    Ok(rows)
}

/// One generated flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatRow {
    /// `D-001`.
    pub id: String,
    /// Constant name, `D001_QUANTIZATION_GRID`.
    pub flag: String,
    /// Topic without the owner parenthesis (from the first register listing the row).
    pub topic: String,
    /// Owner documents (`"01, 04"`): the topic's owner parenthesis in 00, else the first register listing the row.
    pub owner: String,
    /// Every register that lists the row, in order.
    pub docs: Vec<String>,
}

/// Splits a trailing owner parenthesis `(01, 04)` / `(03; …)` off a 00 topic.
pub fn split_owner(topic: &str) -> (String, Option<String>) {
    let re =
        Regex::new(r"^(.*?)\s*\((\d\d(?:,\s*\d\d)*)(?:[;,][^()]*)?\)\s*$").expect("static regex");
    match re.captures(topic) {
        Some(c) => (c[1].to_owned(), Some(c[2].to_owned())),
        None => (topic.to_owned(), None),
    }
}

/// Flag name of a row: the id without the hyphen, then up to four words of the topic (parenthesised parts,
/// apostrophes, quotes, code marks and the words a, an, and, at, by, for, from, in, of, on, or, the, to, vs, with, without removed),
/// upper case, joined by `_`: `D-001` "Quantization grid (01)" → `D001_QUANTIZATION_GRID`.
pub fn flag_name(id: &str, topic: &str) -> String {
    const STOP: &[&str] = &[
        "a", "an", "and", "at", "by", "for", "from", "in", "of", "on", "or", "the", "to", "vs",
        "with", "without",
    ];
    let mut base = String::new();
    let mut depth = 0usize;
    for c in topic.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => base.push(c),
            _ => {}
        }
    }
    let base: String = base.chars().filter(|c| !matches!(c, '\'' | '’')).collect();
    let words: Vec<String> = base
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty() && !STOP.contains(&w.to_ascii_lowercase().as_str()))
        .take(4)
        .map(str::to_ascii_uppercase)
        .collect();
    let mut name: String = id
        .chars()
        .filter(|c| *c != '-')
        .collect::<String>()
        .to_ascii_uppercase();
    for w in words {
        name.push('_');
        name.push_str(&w);
    }
    name
}

/// The `Compat` rows over all registers (in register order, each id once) and the register disagreements.
pub fn compat_rows(
    registers: &[(String, Vec<RegisterRow>)],
) -> Result<(Vec<CompatRow>, Vec<String>)> {
    let mut order: Vec<&str> = Vec::new();
    let mut by_id: BTreeMap<&str, Vec<&RegisterRow>> = BTreeMap::new();
    for (_, rows) in registers {
        for r in rows {
            let list = by_id.entry(&r.id).or_default();
            if list.is_empty() {
                order.push(&r.id);
            }
            list.push(r);
        }
    }
    let mut out = Vec::new();
    let mut warnings = Vec::new();
    for id in order {
        let rows = &by_id[id];
        let compat: Vec<&&RegisterRow> = rows.iter().filter(|r| r.switch.compat).collect();
        if compat.is_empty() {
            continue;
        }
        if compat.len() != rows.len() {
            let desc: Vec<String> = rows
                .iter()
                .map(|r| format!("{}:{} `{}`", r.doc, r.line, r.toggle))
                .collect();
            warnings.push(format!(
                "{id}: the registers disagree on Compat (flag generated because one marks it): {}",
                desc.join("; ")
            ));
        }
        let first = rows[0];
        let (topic, owner) = split_owner(&first.topic);
        let mut docs: Vec<String> = rows.iter().map(|r| r.doc.clone()).collect();
        docs.dedup();
        out.push(CompatRow {
            id: id.to_owned(),
            flag: flag_name(id, &topic),
            owner: owner.unwrap_or_else(|| first.doc.clone()),
            topic,
            docs,
        });
    }
    if out.len() > MAX_FLAGS {
        bail!(
            "{} Compat rows exceed the {MAX_FLAGS} bits of CompatFlags: the type must become a bitset (a contract \
             change, domain-model.md §9.5)",
            out.len()
        );
    }
    let mut seen = BTreeSet::new();
    for r in &out {
        if !seen.insert(&r.flag) {
            bail!("duplicate flag name {}", r.flag);
        }
    }
    Ok((out, warnings))
}

/// Wraps `text` into `///` lines of at most `width` columns, indented by `indent`.
fn doc_lines(out: &mut String, indent: &str, text: &str, width: usize) {
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && indent.len() + 4 + line.len() + 1 + word.len() > width {
            let _ = writeln!(out, "{indent}/// {line}");
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        let _ = writeln!(out, "{indent}/// {line}");
    }
}

/// The text of `compat_generated.rs` for these rows.
pub fn render_compat(rows: &[CompatRow]) -> String {
    let mut s = String::new();
    s.push_str(
        "//! Compatibility flags: exactly one flag per register row whose Toggle column has the switch kind `Compat` (spec 00\n\
         //! §5.1 rule 3), over spec 00 §5.2 and the \"Inspiration differences\" register of every topic document 01-09\n\
         //! (domain-model.md §9.5, AD-20).\n\
         //!\n\
         //! GENERATED by `cargo xtask spec-sync` (xtask/src/spec_sync.rs, owner T01c) from docs/mechanics; never edit by\n\
         //! hand. Bits follow register order (00 §5.2 first, then the rows only 01 … 09 list); a flag's name is the row id\n\
         //! and up to four words of its topic. Serde, helpers and docs of the type live in `compat.rs` (T01a).\n",
    );
    let _ = writeln!(s, "//!\n//! {} Compat rows.\n", rows.len());
    s.push_str(
        "bitflags::bitflags! {\n    \
         /// One bit per `Compat` register row (u128: room for 128 rows; past that the type becomes a bitset, a contract\n    \
         /// change).\n    \
         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]\n    \
         pub struct CompatFlags: u128 {\n",
    );
    for (bit, r) in rows.iter().enumerate() {
        let restated = if r.docs.len() > 1 {
            "registers"
        } else {
            "register"
        };
        let text = format!(
            "{} {} (owner {}; {restated} {}).",
            r.id,
            r.topic,
            r.owner,
            r.docs.join(", ")
        );
        doc_lines(&mut s, "        ", &text, 120);
        let _ = writeln!(s, "        const {} = 1 << {bit};", r.flag);
    }
    s.push_str(
        "        /// Every flag above: the inspiration-compatible profile (spec 00 §4.6).\n",
    );
    if rows.is_empty() {
        s.push_str("        const INSPIRATION = 0;\n");
    } else {
        let _ = writeln!(
            s,
            "        const INSPIRATION = u128::MAX >> {};",
            MAX_FLAGS - rows.len()
        );
    }
    s.push_str("    }\n}\n");
    s
}

/// Flag constants declared in a generated file (`const NAME =` lines), `INSPIRATION` excluded.
pub fn declared_flags(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^\s*const\s+([A-Z0-9_]+)\s*=").expect("static regex");
    re.captures_iter(text)
        .map(|c| c[1].to_owned())
        .filter(|n| n != "INSPIRATION")
        .collect()
}

/// `--check` of the generated file: rows without a flag, flags without a row, and any other difference.
pub fn check_compat(rows: &[CompatRow], generated: &str, current: Option<&str>) -> Vec<String> {
    let Some(current) = current else {
        return vec![format!(
            "{GENERATED_PATH} is missing: run `cargo xtask spec-sync`"
        )];
    };
    if current == generated {
        return Vec::new();
    }
    let declared = declared_flags(current);
    let declared_set: BTreeSet<&str> = declared.iter().map(String::as_str).collect();
    let wanted: BTreeSet<&str> = rows.iter().map(|r| r.flag.as_str()).collect();
    let mut problems = Vec::new();
    for r in rows {
        if !declared_set.contains(r.flag.as_str()) {
            problems.push(format!(
                "{} \"{}\" (registers {}) is a Compat row without a flag (expected `{}`)",
                r.id,
                r.topic,
                r.docs.join(", "),
                r.flag
            ));
        }
    }
    for f in &declared {
        if !wanted.contains(f.as_str()) {
            problems.push(format!("flag `{f}` has no Compat row in the registers"));
        }
    }
    if problems.is_empty() {
        problems.push("the flag set matches but the file differs from the generated text (bit order or doc comments)".into());
    }
    problems.push(format!(
        "{GENERATED_PATH} is stale: run `cargo xtask spec-sync`"
    ));
    problems
}

// ------------------------------------------------------------------------------------------------------------------
// Toggles
// ------------------------------------------------------------------------------------------------------------------

/// One row of spec 00 §6.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecToggle {
    /// `T-01`.
    pub id: String,
    /// Toggle name without the parenthesised inspiration control (`Fight window`).
    pub name: String,
}

/// Parses the table of spec 00 §6.1.
pub fn parse_toggle_table(text: &str) -> Result<Vec<SpecToggle>> {
    let lines: Vec<&str> = text.lines().collect();
    let (start, end) = section(&lines, "00 §6.1 Toggle definitions", |l| {
        l.starts_with("### 6.1 ")
    })?;
    let id_re = Regex::new(r"^T-\d{2}$").expect("static regex");
    let mut out = Vec::new();
    for table in tables_in(&lines, start, end)? {
        if table.header.first().map(String::as_str) != Some("ID") {
            continue;
        }
        for (line, cells) in &table.rows {
            let id = cells[0].trim();
            if !id_re.is_match(id) {
                bail!("00:{line}: toggle id `{id}` is not T-nn");
            }
            let raw = cells.get(1).map(String::as_str).unwrap_or("");
            let name = raw.split(" (").next().unwrap_or(raw).trim().to_owned();
            out.push(SpecToggle {
                id: id.to_owned(),
                name,
            });
        }
    }
    if out.is_empty() {
        bail!("00 §6.1 has no toggle table");
    }
    Ok(out)
}

fn place(p: &TogglePlace) -> String {
    match p {
        TogglePlace::Scenario(path) => format!("ScenarioDoc `{path}`"),
        TogglePlace::Build(path) => format!("BuildDoc `{path}`"),
        TogglePlace::Buffs => "buffs[] (ItemKind::Buff.toggle)".to_owned(),
        TogglePlace::OptRequest(path) => format!("OptRequest `{path}`"),
    }
}

/// Checks a toggle map against the spec table (domain-model.md §9.3): every spec id exactly once; `Spec` entries must
/// be spec rows; `Proposed` entries must not use an id the spec lists. Each problem names the toggle on both sides.
pub fn check_toggles(
    spec: &[SpecToggle],
    map: &[(&str, TogglePlace, ToggleStatus)],
) -> Vec<String> {
    let mut problems = Vec::new();
    let spec_by_id: BTreeMap<&str, &SpecToggle> = spec.iter().map(|t| (t.id.as_str(), t)).collect();
    if spec_by_id.len() != spec.len() {
        problems.push("spec 00 §6.1 lists a toggle id twice".to_owned());
    }
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for (id, _, _) in map {
        *count.entry(id).or_default() += 1;
    }
    for (id, n) in &count {
        if *n > 1 {
            problems.push(format!(
                "{id} appears {n} times in TOGGLE_MAP (exactly once required)"
            ));
        }
    }
    for (id, p, status) in map {
        match (spec_by_id.get(id), status) {
            (Some(_), ToggleStatus::Spec) | (None, ToggleStatus::Proposed) => {}
            (None, ToggleStatus::Spec) => problems.push(format!(
                "{id}: TOGGLE_MAP marks it Spec at {} but spec 00 §6.1 has no {id}",
                place(p)
            )),
            (Some(t), ToggleStatus::Proposed) => problems.push(format!(
                "{id}: spec 00 §6.1 lists {id} as \"{}\", TOGGLE_MAP still has {id} as a Proposed entry at {} \
                 (flip it to Spec if it is the same toggle, else renumber the entry: the spec numbering wins)",
                t.name,
                place(p)
            )),
        }
    }
    for t in spec {
        if !count.contains_key(t.id.as_str()) {
            problems.push(format!(
                "{} \"{}\" (spec 00 §6.1) has no TOGGLE_MAP entry",
                t.id, t.name
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        crate::workspace_root()
    }

    #[test]
    fn toggle_cells() {
        let c = |s: &str| classify_toggle(s).unwrap();
        assert_eq!(
            c("Compat"),
            Switch {
                user: false,
                compat: true
            }
        );
        assert_eq!(c("—"), Switch::default());
        assert_eq!(
            c("User (T-19) + Compat"),
            Switch {
                user: true,
                compat: true
            }
        );
        assert_eq!(
            c("Compat: `Q = x`"),
            Switch {
                user: false,
                compat: true
            }
        );
        assert_eq!(
            c("Compat, k ≥ 1: `Part·c_k` of E-12"),
            Switch {
                user: false,
                compat: true
            }
        );
        assert_eq!(c("— (Compat path of D-065, OQ-739)"), Switch::default());
        assert_eq!(
            c("User (T-26); a compat flag is proposed (OQ-335)"),
            Switch {
                user: true,
                compat: false
            }
        );
        assert_eq!(
            c("User (T-35, T-36) + Compat (capacity mode `ignore`)"),
            Switch {
                user: true,
                compat: true
            }
        );
        assert_eq!(
            c("Compat (`03` §10.3: F_tick,p = F_i)"),
            Switch {
                user: false,
                compat: true
            }
        );
        assert_eq!(
            c("— for both halves: shielded Toxin scenarios are not oracle-safe"),
            Switch::default()
        );
        assert!(classify_toggle("maybe").is_err());
    }

    #[test]
    fn names_and_owners() {
        assert_eq!(
            split_owner("Quantization grid (01)"),
            ("Quantization grid".into(), Some("01".into()))
        );
        assert_eq!(
            split_owner("Phoenix Talons (\"Madurai\") (01, 04)"),
            ("Phoenix Talons (\"Madurai\")".into(), Some("01, 04".into()))
        );
        assert_eq!(
            split_owner("Sonar on DoT (03; wiki-versus-wiki, section 1.4 rule 3)")
                .1
                .as_deref(),
            Some("03")
        );
        assert_eq!(split_owner("Prelude of Might (`x`)").1, None);
        assert_eq!(
            flag_name("D-001", "Quantization grid"),
            "D001_QUANTIZATION_GRID"
        );
        assert_eq!(
            flag_name("D-007", "Phoenix Talons (\"Madurai\")"),
            "D007_PHOENIX_TALONS"
        );
        assert_eq!(
            flag_name("D-025", "DoT \"+1\" accumulator"),
            "D025_DOT_1_ACCUMULATOR"
        );
        assert_eq!(
            flag_name("D-504", "Health and armor panels on shielded targets"),
            "D504_HEALTH_ARMOR_PANELS_SHIELDED"
        );
        assert_eq!(
            flag_name("D-030a", "Own-hit timing"),
            "D030A_OWN_HIT_TIMING"
        );
        assert_eq!(
            flag_name("D-706", "Dreamer's Wrath gating"),
            "D706_DREAMERS_WRATH_GATING"
        );
    }

    #[test]
    fn tables_split_code_spans_and_escapes() {
        assert_eq!(
            split_cells("| a | `x | y` | b \\| c |"),
            vec!["a", "`x | y`", "b | c"]
        );
    }

    /// Every register of docs/mechanics parses (committed versions of this worktree): known rows and switch kinds.
    #[test]
    fn every_register_parses() {
        let docs = read_mechanics(&root()).unwrap();
        assert_eq!(docs.len(), 10);
        let mut registers = Vec::new();
        for (doc, text) in &docs {
            let rows = parse_register(doc, text).unwrap();
            assert!(!rows.is_empty(), "{doc}: empty register");
            for r in &rows {
                assert!(r.id.starts_with("D-") && !r.topic.is_empty(), "{r:?}");
            }
            registers.push((doc.clone(), rows));
        }
        let find = |doc: &str, id: &str| {
            registers
                .iter()
                .find(|(d, _)| d == doc)
                .unwrap()
                .1
                .iter()
                .find(|r| r.id == id)
                .cloned()
        };
        // 00 §5.2: D-001 is Compat, D-004 has no switch.
        assert!(find("00", "D-001").unwrap().switch.compat);
        assert_eq!(find("00", "D-004").unwrap().switch, Switch::default());
        // 05 restates D-030 and D-031 in one row.
        assert_eq!(
            find("05", "D-030").unwrap().line,
            find("05", "D-031").unwrap().line
        );
        let (flags, warnings) = compat_rows(&registers).unwrap();
        assert!(!flags.is_empty() && flags.len() <= MAX_FLAGS);
        assert_eq!(flags[0].id, "D-001");
        // Rows only a topic document lists are included (e.g. 01's D-103).
        assert!(flags.iter().any(|f| f.id == "D-103" && f.docs == ["01"]));
        // 07 D-708 `— (Compat path of D-065, OQ-739)` names Compat but has no switch (00 §5.1 rule 3, request C-20).
        let d708 = find("07", "D-708").unwrap();
        assert!(d708.toggle.contains("Compat") && !d708.switch.compat);
        assert!(!flags.iter().any(|f| f.id == "D-708"));
        // D-028 is the one register disagreement (request C-17).
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].starts_with("D-028:"));
    }

    /// The committed registers, pinned: rows (a combined `D-030, D-031` cell counts twice) and Compat rows per
    /// register, distinct ids, flags. A spec change that adds or drops register rows changes these numbers together
    /// with `compat_generated.rs` (`cargo xtask spec-sync`); update them in the same change. A table that silently
    /// stops parsing (05 has two register tables, the second with 13 rows) fails here.
    #[test]
    fn register_counts_are_pinned() {
        const EXPECTED: [(&str, usize, usize); 10] = [
            // (document, rows, Compat rows)
            ("00", 95, 44),
            ("01", 15, 5),
            ("02", 27, 10),
            ("03", 27, 15),
            ("04", 41, 8),
            ("05", 33, 18),
            ("06", 24, 7),
            ("07", 14, 9),
            ("08", 16, 7),
            ("09", 14, 1),
        ];
        let docs = read_mechanics(&root()).unwrap();
        let mut registers = Vec::new();
        let mut got = Vec::new();
        for (doc, text) in &docs {
            let rows = parse_register(doc, text).unwrap();
            let compat = rows.iter().filter(|r| r.switch.compat).count();
            got.push((doc.clone(), rows.len(), compat));
            registers.push((doc.clone(), rows));
        }
        let expected: Vec<(String, usize, usize)> = EXPECTED
            .iter()
            .map(|(d, r, c)| ((*d).to_owned(), *r, *c))
            .collect();
        assert_eq!(got, expected, "register rows / Compat rows per document");
        assert_eq!(got.iter().map(|g| g.1).sum::<usize>(), 306);
        let ids: BTreeSet<&str> = registers
            .iter()
            .flat_map(|(_, rows)| rows.iter().map(|r| r.id.as_str()))
            .collect();
        assert_eq!(ids.len(), 197, "distinct register ids");
        // 05: the second register table (rows after the first table's last line + 1) has 13 rows.
        let r05 = &registers[5].1;
        let lines: Vec<usize> = r05.iter().map(|r| r.line).collect();
        let gap = lines.windows(2).position(|w| w[1] > w[0] + 1).unwrap();
        let second: BTreeSet<usize> = lines[gap + 1..].iter().copied().collect();
        assert_eq!(second.len(), 13, "05 second register table");
        let (flags, _) = compat_rows(&registers).unwrap();
        assert_eq!(
            flags.len(),
            73,
            "Compat flags (work-breakdown.md T01c says 71: request C-20)"
        );
    }

    /// The committed generated file matches the committed registers, and the compiled type has exactly these flags.
    #[test]
    fn committed_file_is_current() {
        let outcome = sync(&root(), true).unwrap();
        assert_eq!(
            outcome.compat_problems,
            Vec::<String>::new(),
            "{}",
            outcome.summary
        );
        let mut names: Vec<String> = outcome.flags.iter().map(|f| f.flag.clone()).collect();
        names.push("INSPIRATION".into());
        assert_eq!(wfd_model::CompatFlags::all_names(), names);
        assert_eq!(
            wfd_model::CompatFlags::INSPIRATION.bits().count_ones() as usize,
            outcome.flags.len()
        );
    }

    /// A copy of the spec with one extra Compat row in a topic register fails the check (work-breakdown.md T01c).
    #[test]
    fn extra_compat_row_fails_the_check() {
        let src = root();
        let tmp = std::env::temp_dir().join(format!("wfd-spec-sync-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join(MECHANICS_DIR)).unwrap();
        std::fs::create_dir_all(tmp.join(GENERATED_PATH).parent().unwrap()).unwrap();
        std::fs::copy(src.join(GENERATED_PATH), tmp.join(GENERATED_PATH)).unwrap();
        for (doc, text) in read_mechanics(&src).unwrap() {
            let text = if doc == "04" {
                // Insert the extra row after the register's separator line.
                let lines: Vec<&str> = text.lines().collect();
                let (start, _) = section(&lines, "", |l| {
                    l.starts_with("## ") && l.contains("Inspiration differences")
                })
                .unwrap();
                let sep = (start..lines.len())
                    .find(|&i| lines[i].starts_with("|---"))
                    .unwrap();
                let mut out: Vec<String> = lines.iter().map(|l| (*l).to_owned()).collect();
                out.insert(
                    sep + 1,
                    "| D-499 | Test-only extra row (04) | a | b | c | Compat |".to_owned(),
                );
                out.join("\n") + "\n"
            } else {
                text
            };
            std::fs::write(tmp.join(MECHANICS_DIR).join(format!("{doc}-copy.md")), text).unwrap();
        }
        let baseline = sync(&src, true).unwrap();
        let outcome = sync(&tmp, true).unwrap();
        let _ = std::fs::remove_dir_all(&tmp);
        assert_eq!(outcome.flags.len(), baseline.flags.len() + 1);
        assert!(
            outcome
                .compat_problems
                .iter()
                .any(|p| p.contains("D-499") && p.contains("without a flag")),
            "{:?}",
            outcome.compat_problems
        );
        assert!(outcome.compat_problems.iter().any(|p| p.contains("stale")));
    }

    #[test]
    fn missing_and_unknown_flags_are_named() {
        let rows = vec![CompatRow {
            id: "D-001".into(),
            flag: "D001_X".into(),
            topic: "X".into(),
            owner: "01".into(),
            docs: vec!["00".into()],
        }];
        let generated = render_compat(&rows);
        assert!(check_compat(&rows, &generated, Some(&generated)).is_empty());
        let other = generated.replace("D001_X", "D002_Y");
        let p = check_compat(&rows, &generated, Some(&other));
        assert!(
            p.iter()
                .any(|p| p.contains("D-001") && p.contains("without a flag")),
            "{p:?}"
        );
        assert!(
            p.iter()
                .any(|p| p.contains("D002_Y") && p.contains("no Compat row")),
            "{p:?}"
        );
        assert!(!check_compat(&rows, &generated, None).is_empty());
        assert!(render_compat(&[]).contains("const INSPIRATION = 0;"));
        assert!(generated.contains("const INSPIRATION = u128::MAX >> 127;"));
    }

    #[test]
    fn toggle_map_check() {
        let spec = vec![
            SpecToggle {
                id: "T-01".into(),
                name: "Fight window".into(),
            },
            SpecToggle {
                id: "T-02".into(),
                name: "Melee plan".into(),
            },
            SpecToggle {
                id: "T-03".into(),
                name: "Strength sources".into(),
            },
        ];
        let ok: Vec<(&str, TogglePlace, ToggleStatus)> = vec![
            (
                "T-01",
                TogglePlace::Scenario("eval.window_s"),
                ToggleStatus::Spec,
            ),
            (
                "T-02",
                TogglePlace::Scenario("toggles.melee_plan"),
                ToggleStatus::Spec,
            ),
            ("T-03", TogglePlace::Buffs, ToggleStatus::Spec),
            (
                "T-04",
                TogglePlace::Scenario("toggles.x"),
                ToggleStatus::Proposed,
            ),
        ];
        assert!(check_toggles(&spec, &ok).is_empty());
        let drift: Vec<(&str, TogglePlace, ToggleStatus)> = vec![
            (
                "T-01",
                TogglePlace::Scenario("eval.window_s"),
                ToggleStatus::Spec,
            ),
            (
                "T-02",
                TogglePlace::Scenario("toggles.capacity"),
                ToggleStatus::Proposed,
            ),
            (
                "T-05",
                TogglePlace::Scenario("toggles.y"),
                ToggleStatus::Spec,
            ),
            (
                "T-01",
                TogglePlace::Scenario("eval.window_s"),
                ToggleStatus::Spec,
            ),
        ];
        let p = check_toggles(&spec, &drift);
        assert!(p.iter().any(|p| p.starts_with("T-02")
            && p.contains("Melee plan")
            && p.contains("toggles.capacity")));
        assert!(p.iter().any(|p| p.starts_with("T-03")
            && p.contains("Strength sources")
            && p.contains("no TOGGLE_MAP")));
        assert!(
            p.iter()
                .any(|p| p.starts_with("T-05") && p.contains("has no T-05"))
        );
        assert!(p.iter().any(|p| p.starts_with("T-01 appears 2 times")));
    }

    #[test]
    fn spec_toggle_table_parses() {
        let docs = read_mechanics(&root()).unwrap();
        let t = parse_toggle_table(&docs[0].1).unwrap();
        assert_eq!(
            t[0],
            SpecToggle {
                id: "T-01".into(),
                name: "Fight window".into()
            }
        );
        for (i, row) in t.iter().enumerate() {
            assert_eq!(
                row.id,
                format!("T-{:02}", i + 1),
                "spec 00 §6.1 ids are consecutive"
            );
        }
    }
}
