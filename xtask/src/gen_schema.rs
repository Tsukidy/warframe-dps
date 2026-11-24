//! `cargo xtask gen-schema [--check]` (T01b; pipeline.md §6.1): writes `data/curated/schema.json`, the JSON schema of
//! the curated TOML files for editor validation (Taplo). `--check` writes nothing and fails when the file is missing
//! or stale.

/// Output file, relative to the workspace root.
pub const OUT_FILE: &str = "data/curated/schema.json";

/// Entry point (`cargo xtask gen-schema [--check]`): the arguments after the command name.
pub fn run(args: &[String]) -> anyhow::Result<()> {
    match crate::gen_ts::check_flag("gen-schema", args)? {
        Some(check) => generate(check).map_err(anyhow::Error::msg),
        None => Ok(()),
    }
}

/// Writes (or with `check` compares) `data/curated/schema.json`.
pub fn generate(check: bool) -> Result<(), String> {
    let text = wfd_model::api::codegen::curated_schema_json();
    let path = crate::gen_ts::workspace_root().join(OUT_FILE);
    let old = std::fs::read_to_string(&path).ok();
    if old.as_deref() == Some(text.as_str()) {
        return Ok(());
    }
    if check {
        let what = if old.is_some() { "stale" } else { "missing" };
        return Err(format!(
            "{} is {what}; run `cargo xtask gen-schema`",
            path.display()
        ));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    println!("cargo xtask gen-schema: wrote {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_file_is_current() {
        // The same check CI runs (`cargo xtask gen-schema --check`).
        super::generate(true).unwrap();
    }
}
