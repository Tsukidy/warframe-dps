//! Pipeline configuration (pipeline.md §2): where the inputs are and how the lock file pins them. Frozen API of T02c.

use crate::DatagenError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use wfd_model::curated::{CuratedTree, load_curated_dir, load_curated_files};

/// Default source cache, relative to the repository root (pipeline.md §3.3; gitignored; `--cache-dir` overrides, the
/// server uses `<data_dir>/cache`).
pub const CACHE_DIR: &str = ".cache/wfd-sources";
/// The lock file that pins the inputs of the committed `data/generated/` (pipeline.md §3.3).
pub const LOCK_FILE: &str = "data/sources.lock.toml";
/// The curated tree (pipeline.md §6.1).
pub const CURATED_DIR: &str = "data/curated";
/// The canonical JSON output (pipeline.md §8.1).
pub const GENERATED_DIR: &str = "data/generated";
/// The list of spec tables to ingest (pipeline.md §5.2).
pub const SPEC_TABLES_FILE: &str = "data/spec_tables.toml";
/// Validation thresholds and allow-lists (pipeline.md §7: `base.toml`, `coverage/<task>.toml`).
pub const VALIDATION_DIR: &str = "data/validation";

/// Everything a pipeline run reads (pipeline.md §2). Build one with [`PipelineConfig::developer`] (the CLI) or
/// [`PipelineConfig::in_app`] (the server's refresh) and adjust the public fields; the struct is `#[non_exhaustive]`
/// so that a field added later (a `CONTRACTS.md` entry) does not break callers.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PipelineConfig {
    /// The source cache (pipeline.md §3.3): `index.json`, `wiki/`, `pe/`, `wfcd/<tag>/`.
    pub cache_dir: PathBuf,
    /// `data/curated/`: a directory (developer, `--curated DIR`) or the tree embedded in the binary (in-app refresh).
    pub curated: InputTree,
    /// Where the tier-2 spec tables come from (pipeline.md §5.2).
    pub spec: SpecSource,
    /// No network: every input must already be in the cache (`--offline`).
    pub offline: bool,
    /// Which upstream revisions a run uses.
    pub lock: LockMode,
    /// The lock file (`data/sources.lock.toml`): read by [`LockMode::Pinned`] and [`LockMode::FromLock`], written by a
    /// fetch with [`update_lock`](Self::update_lock). `None` in the in-app refresh, which has no lock file. Addition
    /// to pipeline.md §2 (CONTRACTS.md T02c).
    pub lock_file: Option<PathBuf>,
    /// `fetch --update-lock`: with [`LockMode::Latest`], write the newest revids, Public Export build label and WFCD tag
    /// to [`lock_file`](Self::lock_file). Addition to pipeline.md §2 (CONTRACTS.md T02c).
    pub update_lock: bool,
    /// `fetch --only`: the one source a fetch touches; `None` = all three. Addition to pipeline.md §2 (CONTRACTS.md
    /// T02c).
    pub only: Option<SourceKind>,
}

impl PipelineConfig {
    /// The developer layout under a repository root (pipeline.md §3.3, §6.1): cache [`CACHE_DIR`], curated
    /// [`CURATED_DIR`] on disk, spec tables read from the Markdown ([`SpecSource::Docs`]), inputs pinned by
    /// [`LOCK_FILE`], network allowed.
    pub fn developer(root: &Path) -> PipelineConfig {
        PipelineConfig {
            cache_dir: root.join(CACHE_DIR),
            curated: InputTree::Dir(root.join(CURATED_DIR)),
            spec: SpecSource::Docs(root.to_owned()),
            offline: false,
            lock: LockMode::Pinned,
            lock_file: Some(root.join(LOCK_FILE)),
            update_lock: false,
            only: None,
        }
    }

    /// The in-app refresh (pipeline.md §9.2): newest inputs into `cache_dir`, curated data and the ingested spec tables
    /// from the trees the server embeds (AD-13), no lock file, never `docs/`.
    pub fn in_app(
        cache_dir: PathBuf,
        curated: InputTree,
        ingested_spec: InputTree,
    ) -> PipelineConfig {
        PipelineConfig {
            cache_dir,
            curated,
            spec: SpecSource::Ingested(ingested_spec),
            offline: false,
            lock: LockMode::Latest,
            lock_file: None,
            update_lock: false,
            only: None,
        }
    }
}

/// Which upstream revisions a run uses (pipeline.md §2, §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockMode {
    /// Exactly the inputs of the lock file (`wfd-datagen build`; reproducible `data/generated/`).
    Pinned,
    /// The newest inputs upstream (`fetch`, `fetch --update-lock`, the in-app refresh).
    Latest,
    /// Restore exactly the locked inputs into an empty cache by revid, content hash and tag (`fetch --from-lock`,
    /// §3.3); a build in this mode reads the same inputs as [`LockMode::Pinned`].
    FromLock,
}

/// The three upstream sources (pipeline.md §1). Readable form and CLI value: `wiki`, `pe`, `wfcd`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// wiki.warframe.com Lua data modules.
    Wiki,
    /// DE Public Export (content.warframe.com).
    Pe,
    /// WFCD warframe-items at a pinned tag.
    Wfcd,
}

impl SourceKind {
    /// Every source, in fetch order.
    pub const ALL: [SourceKind; 3] = [SourceKind::Wiki, SourceKind::Pe, SourceKind::Wfcd];

    /// `"wiki"`, `"pe"` or `"wfcd"`: the CLI value, the `source` argument of `data.*` messages and the lock-file
    /// section.
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Wiki => "wiki",
            SourceKind::Pe => "pe",
            SourceKind::Wfcd => "wfcd",
        }
    }
}

/// A directory on disk (developer, `--curated DIR`) or a tree embedded in the binary with `include_dir!` (in-app
/// refresh: `data/curated/` and `data/generated/spec/`, embedded by wfd-server behind feature `refresh`; AD-13).
#[derive(Debug, Clone)]
pub enum InputTree {
    /// A directory on disk.
    Dir(PathBuf),
    /// A tree embedded in the binary.
    Embedded(&'static include_dir::Dir<'static>),
}

impl InputTree {
    /// Every file of the tree as `(path relative to the tree root, UTF-8 content)`, sorted by path, hidden files
    /// included (consumers such as [`load_curated_files`] decide what to skip). Both variants give the same list for
    /// the same files.
    pub fn files(&self) -> Result<Vec<(PathBuf, String)>, DatagenError> {
        let mut out = Vec::new();
        match self {
            InputTree::Dir(root) => walk_dir(root, root, &mut out)?,
            InputTree::Embedded(dir) => walk_embedded(dir, &mut out)?,
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(out)
    }

    /// The content of one file by its path relative to the tree root; `None` when the tree has no such file.
    pub fn read(&self, rel: &Path) -> Result<Option<String>, DatagenError> {
        match self {
            InputTree::Dir(root) => {
                let path = root.join(rel);
                if !path.is_file() {
                    return Ok(None);
                }
                std::fs::read_to_string(&path)
                    .map(Some)
                    .map_err(|source| DatagenError::Io { path, source })
            }
            InputTree::Embedded(dir) => dir
                .get_file(rel)
                .map(|f| utf8(f.path(), f.contents()))
                .transpose(),
        }
    }

    /// Loads the tree as `data/curated/` ([`wfd_model::curated`]): [`load_curated_dir`] for a directory,
    /// [`load_curated_files`] over [`files`](Self::files) for an embedded tree, so both follow the same rules.
    pub fn load_curated(&self) -> Result<CuratedTree, DatagenError> {
        match self {
            InputTree::Dir(root) => load_curated_dir(root).map_err(DatagenError::Curated),
            InputTree::Embedded(_) => {
                load_curated_files(self.files()?).map_err(DatagenError::Curated)
            }
        }
    }
}

fn walk_dir(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, String)>) -> Result<(), DatagenError> {
    let io = |path: &Path| {
        let path = path.to_owned();
        move |source| DatagenError::Io { path, source }
    };
    for entry in std::fs::read_dir(dir).map_err(io(dir))? {
        let path = entry.map_err(io(dir))?.path();
        if path.is_dir() {
            walk_dir(root, &path, out)?;
        } else {
            let bytes = std::fs::read(&path).map_err(io(&path))?;
            let rel = path.strip_prefix(root).unwrap_or(&path);
            out.push((rel.to_owned(), utf8(rel, &bytes)?));
        }
    }
    Ok(())
}

fn walk_embedded(
    dir: &include_dir::Dir<'static>,
    out: &mut Vec<(PathBuf, String)>,
) -> Result<(), DatagenError> {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(d) => walk_embedded(d, out)?,
            include_dir::DirEntry::File(f) => {
                out.push((f.path().to_owned(), utf8(f.path(), f.contents())?))
            }
        }
    }
    Ok(())
}

fn utf8(path: &Path, bytes: &[u8]) -> Result<String, DatagenError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| DatagenError::Parse {
        file: path.display().to_string().into(),
        detail: "not UTF-8 text".to_owned(),
    })
}

/// Where the tier-2 spec tables come from (pipeline.md §5.2).
#[derive(Debug, Clone)]
pub enum SpecSource {
    /// Developer builds read the spec Markdown and write `data/generated/spec/*.json`. The path is the repository root:
    /// the `file` entries of `data/spec_tables.toml` (`docs/mechanics/04-….md`) are relative to it.
    Docs(PathBuf),
    /// The in-app refresh reads the already-ingested tables (`data/generated/spec/<id>.json`) and never needs `docs/`.
    Ingested(InputTree),
}

#[cfg(test)]
mod tests {
    use super::*;
    use include_dir::{Dir, include_dir};

    static TREE: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/input_tree");

    fn fixture_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/input_tree")
    }

    #[test]
    fn developer_layout() {
        let root = Path::new("/repo");
        let cfg = PipelineConfig::developer(root);
        assert_eq!(cfg.cache_dir, Path::new("/repo/.cache/wfd-sources"));
        assert!(matches!(&cfg.curated, InputTree::Dir(p) if p == Path::new("/repo/data/curated")));
        assert!(matches!(&cfg.spec, SpecSource::Docs(p) if p == root));
        assert_eq!(cfg.lock, LockMode::Pinned);
        assert_eq!(
            cfg.lock_file.as_deref(),
            Some(Path::new("/repo/data/sources.lock.toml"))
        );
        assert!(!cfg.offline && !cfg.update_lock && cfg.only.is_none());
    }

    #[test]
    fn in_app_layout_has_no_lock_and_reads_ingested_tables() {
        let cfg = PipelineConfig::in_app(
            "/data/cache".into(),
            InputTree::Embedded(&TREE),
            InputTree::Embedded(&TREE),
        );
        assert_eq!(cfg.lock, LockMode::Latest);
        assert!(cfg.lock_file.is_none() && !cfg.update_lock && !cfg.offline);
        assert!(matches!(
            cfg.spec,
            SpecSource::Ingested(InputTree::Embedded(_))
        ));
    }

    /// The same files through both variants (AD-13: the in-app refresh sees exactly what the developer build sees).
    #[test]
    fn dir_and_embedded_trees_agree() {
        let dir = InputTree::Dir(fixture_dir()).files().unwrap();
        let emb = InputTree::Embedded(&TREE).files().unwrap();
        assert_eq!(dir, emb);
        let paths: Vec<&Path> = dir.iter().map(|(p, _)| p.as_path()).collect();
        assert_eq!(
            paths,
            [
                Path::new(".hidden"),
                Path::new("a.toml"),
                Path::new("rules/b.json"),
                Path::new("rules/sub/c.txt")
            ]
        );
        assert_eq!(dir[1].1, "schema = 1\n");
    }

    #[test]
    fn read_by_relative_path() {
        for tree in [InputTree::Dir(fixture_dir()), InputTree::Embedded(&TREE)] {
            assert_eq!(
                tree.read(Path::new("rules/sub/c.txt")).unwrap().as_deref(),
                Some("c\n")
            );
            assert_eq!(tree.read(Path::new("missing.toml")).unwrap(), None);
            assert_eq!(tree.read(Path::new("rules")).unwrap(), None);
        }
    }

    #[test]
    fn missing_directory_is_an_io_error() {
        let err = InputTree::Dir(fixture_dir().join("nope"))
            .files()
            .unwrap_err();
        assert!(matches!(err, DatagenError::Io { .. }), "{err}");
    }

    /// Both variants go through the same curated loader rules: here, files at the top level and outside the known
    /// directories are errors, reported with the same relative paths.
    #[test]
    fn curated_loading_agrees() {
        let dir = InputTree::Dir(fixture_dir()).load_curated().unwrap_err();
        let emb = InputTree::Embedded(&TREE).load_curated().unwrap_err();
        let (DatagenError::Curated(d), DatagenError::Curated(e)) = (dir, emb) else {
            panic!("curated errors expected");
        };
        assert_eq!(d, e);
        assert!(!d.0.is_empty());
        assert!(d.0.iter().all(|e| e.path.is_relative()), "{d}");
    }

    #[test]
    fn source_kind_names() {
        for k in SourceKind::ALL {
            assert_eq!(serde_json::to_value(k).unwrap(), k.as_str());
            assert_eq!(
                <SourceKind as clap::ValueEnum>::from_str(k.as_str(), false),
                Ok(k)
            );
        }
    }
}
