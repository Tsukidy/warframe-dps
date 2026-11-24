//! `cargo xtask gen-ts [--check]` (T01b; overview.md AD-02, frontend.md §1): writes the TypeScript declarations of
//! every API and document type to `frontend/src/api/generated/` (ts-rs, one file per type, plus `messageKeys.ts` and
//! `index.ts`). The directory holds generated output only: files the model no longer produces are removed.
//! `--check` writes nothing and fails when a file is missing, stale or extra (CI, testing.md §10).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Output directory, relative to the workspace root.
pub const OUT_DIR: &str = "frontend/src/api/generated";

/// Entry point (`cargo xtask gen-ts [--check]`): the arguments after the command name.
pub fn run(args: &[String]) -> anyhow::Result<()> {
    match check_flag("gen-ts", args)? {
        Some(check) => generate(check).map_err(anyhow::Error::msg),
        None => Ok(()),
    }
}

/// Writes (or with `check` compares) the generated directory.
pub fn generate(check: bool) -> Result<(), String> {
    let files = wfd_model::api::codegen::typescript_files()?;
    if let Some((path, _)) = files.iter().find(|(_, text)| text.contains("bigint")) {
        // domain-model.md §1: every u64 of an API type carries #[ts(type = "number")].
        return Err(format!("{path} uses bigint"));
    }
    let dir = workspace_root().join(OUT_DIR);
    sync_dir(&dir, &files, check, "cargo xtask gen-ts")
}

/// Parses `[--check]`: `Some(check)`, or `None` after printing the usage for `--help`.
pub(crate) fn check_flag(command: &str, args: &[String]) -> anyhow::Result<Option<bool>> {
    match args {
        [] => Ok(Some(false)),
        [a] if a == "--check" => Ok(Some(true)),
        [a] if a == "--help" || a == "-h" => {
            println!("usage: cargo xtask {command} [--check]");
            Ok(None)
        }
        _ => anyhow::bail!("usage: cargo xtask {command} [--check] (got {args:?})"),
    }
}

/// The workspace root (the parent of the xtask crate).
pub(crate) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_path_buf()
}

/// Compares or writes a directory of generated files: `files` maps paths relative to `dir` to their contents.
pub(crate) fn sync_dir(
    dir: &Path,
    files: &BTreeMap<String, String>,
    check: bool,
    command: &str,
) -> Result<(), String> {
    let existing = list_files(dir)?;
    let mut problems = Vec::new();
    for (rel, text) in files {
        let path = dir.join(rel);
        match existing.get(rel) {
            Some(old) if old == text => {}
            Some(_) if check => problems.push(format!("stale: {}", path.display())),
            None if check => problems.push(format!("missing: {}", path.display())),
            _ => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("{}: {e}", parent.display()))?;
                }
                std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
    }
    for rel in existing.keys() {
        if !files.contains_key(rel) {
            let path = dir.join(rel);
            if check {
                problems.push(format!("not generated: {}", path.display()));
            } else {
                std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
    }
    if problems.is_empty() {
        if !check {
            println!("{command}: {} files in {}", files.len(), dir.display());
        }
        Ok(())
    } else {
        Err(format!(
            "{} out of date ({} problems); run `{command}`:\n  {}",
            dir.display(),
            problems.len(),
            problems.join("\n  ")
        ))
    }
}

/// Every file below `dir` (relative path with `/` → content); an absent directory is empty.
fn list_files(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    if !dir.exists() {
        return Ok(out);
    }
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let entries = std::fs::read_dir(&d).map_err(|e| format!("{}: {e}", d.display()))?;
        for e in entries {
            let p = e.map_err(|e| format!("{}: {e}", d.display()))?.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p
                    .strip_prefix(dir)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                let text =
                    std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
                out.insert(rel, text);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("xtask-gen-ts-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn sync_writes_checks_and_prunes() {
        let dir = tmp("sync");
        let mut files = BTreeMap::new();
        files.insert("A.ts".to_string(), "a\n".to_string());
        files.insert("sub/B.ts".to_string(), "b\n".to_string());
        assert!(
            sync_dir(&dir, &files, true, "x")
                .unwrap_err()
                .contains("missing")
        );
        sync_dir(&dir, &files, false, "x").unwrap();
        sync_dir(&dir, &files, true, "x").unwrap();
        std::fs::write(dir.join("A.ts"), "changed\n").unwrap();
        assert!(
            sync_dir(&dir, &files, true, "x")
                .unwrap_err()
                .contains("stale")
        );
        std::fs::write(dir.join("Old.ts"), "old\n").unwrap();
        let err = sync_dir(&dir, &files, true, "x").unwrap_err();
        assert!(err.contains("not generated"), "{err}");
        sync_dir(&dir, &files, false, "x").unwrap();
        assert!(!dir.join("Old.ts").exists());
        sync_dir(&dir, &files, true, "x").unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_files_are_current() {
        // The same check CI runs (`cargo xtask gen-ts --check`).
        generate(true).unwrap();
    }

    #[test]
    fn arguments() {
        assert_eq!(check_flag("gen-ts", &[]).unwrap(), Some(false));
        assert_eq!(
            check_flag("gen-ts", &["--check".into()]).unwrap(),
            Some(true)
        );
        assert_eq!(check_flag("gen-ts", &["--help".into()]).unwrap(), None);
        assert!(check_flag("gen-ts", &["--write".into()]).is_err());
        assert!(check_flag("gen-ts", &["--check".into(), "x".into()]).is_err());
    }
}
