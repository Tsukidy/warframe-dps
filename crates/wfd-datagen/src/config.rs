//! Pipeline configuration (pipeline.md §2): where the inputs are and how the lock file pins them. Frozen API of T02c.

use crate::DatagenError;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};
use wfd_model::curated::{CuratedTree, UNLOADED_DIRS, load_curated_dir, load_curated_files};
use wfd_model::{StableKey, UniqueName};

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
/// The append-only global stable-key table (pipeline.md §5.3, AD-18).
pub const STABLE_KEYS_FILE: &str = "data/generated/stable_keys.json";

/// Everything a pipeline run reads (pipeline.md §2). Build one with [`PipelineConfig::developer`] (the CLI) or
/// [`PipelineConfig::in_app`] (the server's refresh) and adjust the public fields; the struct is `#[non_exhaustive]`
/// so that a field added later (a `CONTRACTS.md` entry) does not break callers.
///
/// Network use: only [`crate::fetch()`] touches the network; [`crate::build()`] reads the source cache and nothing
/// else (pipeline.md §2 "the only stage that touches the network"). A caller that wants fresh inputs runs `fetch()`
/// first with the same configuration (the CLI's `build` without `--offline`, the in-app refresh of §9.2). The fields
/// [`offline`](Self::offline), [`update_lock`](Self::update_lock) and [`only`](Self::only) apply to `fetch()` only;
/// [`check_fetch`](Self::check_fetch) and [`check_build`](Self::check_build) reject contradictory settings.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PipelineConfig {
    /// The source cache (pipeline.md §3.3): `index.json`, `wiki/`, `pe/`, `wfcd/<tag>/`.
    pub cache_dir: PathBuf,
    /// `data/curated/`: a directory (developer, `--curated DIR`) or the tree embedded in the binary (in-app refresh).
    pub curated: InputTree,
    /// Where the tier-2 spec tables come from (pipeline.md §5.2).
    pub spec: SpecSource,
    /// `fetch()` only: no network; every input must already be in the cache (`fetch --offline` reports what it
    /// lacks). `build()` never uses the network, whatever this says.
    pub offline: bool,
    /// Which upstream revisions a run uses.
    pub lock: LockMode,
    /// The lock file (`data/sources.lock.toml`): read by [`LockMode::Pinned`] and [`LockMode::FromLock`], written by a
    /// fetch with [`update_lock`](Self::update_lock). `None` in the in-app refresh, which has no lock file. Addition
    /// to pipeline.md §2 (CONTRACTS.md T02c).
    pub lock_file: Option<PathBuf>,
    /// `fetch()` only (`fetch --update-lock`): with [`LockMode::Latest`] and the network, write the newest revids,
    /// Public Export build label and WFCD tag to [`lock_file`](Self::lock_file). Addition to pipeline.md §2
    /// (CONTRACTS.md T02c).
    pub update_lock: bool,
    /// `fetch()` only (`fetch --only`): the one source a fetch touches; `None` = all three. A build always reads all
    /// three. Addition to pipeline.md §2 (CONTRACTS.md T02c).
    pub only: Option<SourceKind>,
    /// `data/spec_tables.toml` (pipeline.md §5.2): the list of spec tables, read by
    /// [`crate::spec_tables::SpecTablesConfig::read`] for [`crate::ingest_spec_tables`] in both [`SpecSource`] modes (the
    /// in-app refresh reads `data/generated/spec/<id>.json` for each of its ids). Addition to pipeline.md §2
    /// (CONTRACTS.md T02c).
    pub spec_tables: InputFile,
    /// `data/validation/` (pipeline.md §7: `base.toml`, `coverage/<task>.toml`), read by every build and check. `None`
    /// when the run has no such tree (the in-app refresh unless the server embeds one, CONTRACTS.md T02c request
    /// T02c-R6): the checks of §7 then run without allow-lists and without coverage minimums. Addition to pipeline.md
    /// §2.
    pub validation: Option<InputTree>,
    /// The stable-key table the run starts from (pipeline.md §5.3, §9.2, AD-18): keys are never renumbered. Addition to
    /// pipeline.md §2 (CONTRACTS.md T02c).
    pub stable_keys: StableKeySource,
}

impl PipelineConfig {
    /// The developer layout under a repository root (pipeline.md §3.3, §6.1): cache [`CACHE_DIR`], curated
    /// [`CURATED_DIR`] on disk, spec tables read from the Markdown ([`SpecSource::Docs`]) by the list
    /// [`SPEC_TABLES_FILE`], validation [`VALIDATION_DIR`], stable keys [`STABLE_KEYS_FILE`] (appended to), inputs
    /// pinned by [`LOCK_FILE`].
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
            spec_tables: InputFile::Path(root.join(SPEC_TABLES_FILE)),
            validation: Some(InputTree::Dir(root.join(VALIDATION_DIR))),
            stable_keys: StableKeySource::File(root.join(STABLE_KEYS_FILE)),
        }
    }

    /// The in-app refresh (pipeline.md §9.2): newest inputs into `cache_dir`; curated data, the ingested spec tables
    /// and their list from what the server embeds (AD-13); the global keys of the embedded bundle
    /// (`Bundle.stable_keys`, never renumbered); no lock file, never `docs/`. [`validation`](Self::validation) is
    /// `None`; the server sets it when it embeds `data/validation/` (CONTRACTS.md T02c request T02c-R6).
    pub fn in_app(
        cache_dir: PathBuf,
        curated: InputTree,
        ingested_spec: InputTree,
        spec_tables: InputFile,
        stable_keys: Vec<(StableKey, UniqueName)>,
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
            spec_tables,
            validation: None,
            stable_keys: StableKeySource::Embedded(stable_keys),
        }
    }

    /// Rejects settings [`crate::fetch()`] cannot honour ([`DatagenError::Config`]); `fetch()` calls it first.
    /// `update_lock` needs [`LockMode::Latest`], the network and a lock file; [`LockMode::FromLock`] needs the network
    /// and a lock file; [`LockMode::Pinned`] needs a lock file.
    pub fn check_fetch(&self) -> Result<(), DatagenError> {
        let bad = |detail: &str| {
            Err(DatagenError::Config {
                detail: detail.to_owned(),
            })
        };
        if self.update_lock {
            if self.lock != LockMode::Latest {
                return bad("update_lock writes the newest inputs: it needs LockMode::Latest");
            }
            if self.offline {
                return bad("update_lock needs the network: it cannot be combined with offline");
            }
            if self.lock_file.is_none() {
                return bad("update_lock needs a lock file to write");
            }
        }
        if self.lock == LockMode::FromLock && self.offline {
            return bad(
                "LockMode::FromLock downloads the locked inputs: it cannot be combined with offline",
            );
        }
        self.check_build()
    }

    /// Rejects settings [`crate::build()`] cannot honour ([`DatagenError::Config`]); `build()` calls it first.
    /// [`LockMode::Pinned`] and [`LockMode::FromLock`] read the lock file, so they need one.
    pub fn check_build(&self) -> Result<(), DatagenError> {
        if self.lock != LockMode::Latest && self.lock_file.is_none() {
            return Err(DatagenError::Config {
                detail: format!(
                    "LockMode::{:?} reads the lock file, but there is none",
                    self.lock
                ),
            });
        }
        Ok(())
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
///
/// An `Embedded` tree may be the root of an `include_dir!` or a sub-directory of one (`DATA.get_dir("curated")`):
/// paths are relative to the given directory either way. Both variants give the same results for the same files.
#[derive(Debug, Clone)]
pub enum InputTree {
    /// A directory on disk.
    Dir(PathBuf),
    /// A tree embedded in the binary.
    Embedded(&'static include_dir::Dir<'static>),
}

impl InputTree {
    /// Every file of the tree as `(path relative to the tree root, UTF-8 content)`, sorted by path, hidden files and
    /// the files of every directory included: a raw listing for inputs whose consumer decides what to skip. A file that
    /// is not UTF-8 is [`DatagenError::Parse`] in both variants (the first in path order). `data/curated/` is loaded
    /// with [`load_curated`](Self::load_curated) instead, which reads only what the curated loader reads.
    pub fn files(&self) -> Result<Vec<(PathBuf, String)>, DatagenError> {
        let mut raw = Vec::new();
        match self {
            InputTree::Dir(root) => walk_dir(root, root, &mut raw)?,
            InputTree::Embedded(dir) => walk_embedded(dir.path(), dir, &mut raw),
        }
        // Decoded in path order, so both variants report the same first non-UTF-8 file.
        raw.sort_by(|a, b| a.0.cmp(&b.0));
        raw.into_iter()
            .map(|(path, bytes)| {
                let text = utf8(&path, &bytes)?;
                Ok((path, text))
            })
            .collect()
    }

    /// The content of one file by its path relative to the tree root; `None` when the tree has no such file (or the
    /// path names a directory). A path that leaves the tree (absolute, or with a `..` component) is
    /// [`DatagenError::Io`] with kind `InvalidInput` in both variants; `.` components are ignored. On disk, an error
    /// other than "not found" while looking the file up is [`DatagenError::Io`] too.
    pub fn read(&self, rel: &Path) -> Result<Option<String>, DatagenError> {
        let rel = normal_relative(rel)?;
        match self {
            InputTree::Dir(root) => {
                let path = root.join(&rel);
                match std::fs::metadata(&path) {
                    Ok(m) if m.is_file() => {}
                    Ok(_) => return Ok(None),
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                        ) =>
                    {
                        return Ok(None);
                    }
                    Err(source) => return Err(DatagenError::Io { path, source }),
                }
                std::fs::read_to_string(&path)
                    .map(Some)
                    .map_err(|source| DatagenError::Io { path, source })
            }
            InputTree::Embedded(dir) => dir
                .get_file(dir.path().join(&rel))
                .map(|f| utf8(&rel, f.contents()))
                .transpose(),
        }
    }

    /// Loads the tree as `data/curated/` ([`wfd_model::curated`]): [`load_curated_dir`] for a directory; for an
    /// embedded tree the same walk over the embedded entries feeding [`load_curated_files`], so both apply the same
    /// rules. Hidden directories and the top-level [`UNLOADED_DIRS`] are not entered; every other file is passed to the
    /// loader, which skips hidden files and the unloaded top-level files and reports unexpected ones. A file that is
    /// not UTF-8 is passed by name only (empty content), as `load_curated_dir` passes the files it does not read; the
    /// one difference left is the message for a non-UTF-8 file the loader parses: "cannot read" on disk, a schema
    /// error embedded (both fail on that path).
    pub fn load_curated(&self) -> Result<CuratedTree, DatagenError> {
        match self {
            InputTree::Dir(root) => load_curated_dir(root).map_err(DatagenError::Curated),
            InputTree::Embedded(dir) => {
                let mut files = Vec::new();
                curated_walk_embedded(dir.path(), dir, &mut files);
                load_curated_files(files).map_err(DatagenError::Curated)
            }
        }
    }
}

/// `rel` with its `.` components dropped; an error for an absolute path or a `..` component.
fn normal_relative(rel: &Path) -> Result<PathBuf, DatagenError> {
    let mut out = PathBuf::new();
    for c in rel.components() {
        match c {
            Component::Normal(n) => out.push(n),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(DatagenError::Io {
                    path: rel.to_owned(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "not a relative path inside the input tree",
                    ),
                });
            }
        }
    }
    Ok(out)
}

fn walk_dir(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), DatagenError> {
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
            out.push((rel.to_owned(), bytes));
        }
    }
    Ok(())
}

/// The path of an embedded entry relative to the embedded tree's own directory (`include_dir` paths are relative to
/// the `include_dir!` root, so a sub-directory's entries carry its path as a prefix).
fn embedded_rel(base: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(base).unwrap_or(path).to_owned()
}

fn walk_embedded(base: &Path, dir: &include_dir::Dir<'static>, out: &mut Vec<(PathBuf, Vec<u8>)>) {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(d) => walk_embedded(base, d, out),
            include_dir::DirEntry::File(f) => {
                out.push((embedded_rel(base, f.path()), f.contents().to_vec()))
            }
        }
    }
}

/// The walk of [`load_curated_dir`] over an embedded tree: hidden directories and the top-level [`UNLOADED_DIRS`] are
/// not entered; files go to the loader with their text, or by name only when they are not UTF-8.
fn curated_walk_embedded(
    base: &Path,
    dir: &include_dir::Dir<'static>,
    out: &mut Vec<(PathBuf, String)>,
) {
    for entry in dir.entries() {
        let rel = embedded_rel(base, entry.path());
        match entry {
            include_dir::DirEntry::Dir(d) => {
                let hidden = rel
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('.'));
                let unloaded = rel.components().count() == 1
                    && UNLOADED_DIRS.iter().any(|u| rel == Path::new(u));
                if !hidden && !unloaded {
                    curated_walk_embedded(base, d, out);
                }
            }
            include_dir::DirEntry::File(f) => {
                let text = std::str::from_utf8(f.contents()).unwrap_or_default();
                out.push((rel, text.to_owned()));
            }
        }
    }
}

fn utf8(path: &Path, bytes: &[u8]) -> Result<String, DatagenError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| DatagenError::Parse {
        file: path.display().to_string().into(),
        detail: "not UTF-8 text".to_owned(),
    })
}

/// One input file: on disk (developer) or embedded in the binary with `include_str!` (in-app refresh). Used for
/// `data/spec_tables.toml` ([`PipelineConfig::spec_tables`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputFile {
    /// A file on disk.
    Path(PathBuf),
    /// A file embedded in the binary.
    Embedded {
        /// Its repository path (`data/spec_tables.toml`), for diagnostics.
        name: &'static str,
        /// Its text.
        text: &'static str,
    },
}

impl InputFile {
    /// The path or the embedded name, for diagnostics.
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            InputFile::Path(p) => p.to_string_lossy(),
            InputFile::Embedded { name, .. } => Cow::Borrowed(name),
        }
    }

    /// The text of the file ([`DatagenError::Io`] when a file on disk cannot be read).
    pub fn read(&self) -> Result<Cow<'static, str>, DatagenError> {
        match self {
            InputFile::Path(p) => std::fs::read_to_string(p)
                .map(Cow::Owned)
                .map_err(|source| DatagenError::Io {
                    path: p.clone(),
                    source,
                }),
            InputFile::Embedded { text, .. } => Ok(Cow::Borrowed(text)),
        }
    }
}

/// The stable-key table a run starts from (pipeline.md §5.3, §9.2; AD-18). A key is never renumbered or reused; the
/// two variants differ in how they key new uniqueNames.
#[derive(Debug, Clone, PartialEq)]
pub enum StableKeySource {
    /// Developer build: `data/generated/stable_keys.json`, append-only and global. New uniqueNames get the next global
    /// key below `LOCAL_KEY_BASE`; removed ones keep theirs and are listed as retired. The build returns the extended
    /// table as `stable_keys.json` of [`crate::GeneratedJson`]. T18 defines the file format and whether a missing file
    /// is an error.
    File(PathBuf),
    /// In-app refresh: the global keys of the embedded bundle (`Bundle.stable_keys`). Never extended: new uniqueNames
    /// get session-local keys ≥ `LOCAL_KEY_BASE` in the order of their uniqueName, stored in the user bundle only.
    Embedded(Vec<(StableKey, UniqueName)>),
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
    /// A curated tree with binary files only where the curated loader never reads (a hidden file, a hidden directory,
    /// `aliases/`, `overrides/`) and an unloaded top-level file.
    static CURATED_BINARY: Dir<'static> =
        include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/curated_binary");
    /// A curated tree whose one item file is not UTF-8.
    static CURATED_BAD_TOML: Dir<'static> =
        include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/curated_bad_toml");

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    fn fixture_dir() -> PathBuf {
        fixture("input_tree")
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
        assert_eq!(
            cfg.spec_tables,
            InputFile::Path("/repo/data/spec_tables.toml".into())
        );
        assert!(
            matches!(&cfg.validation, Some(InputTree::Dir(p)) if p == Path::new("/repo/data/validation"))
        );
        assert_eq!(
            cfg.stable_keys,
            StableKeySource::File("/repo/data/generated/stable_keys.json".into())
        );
        assert!(cfg.check_fetch().is_ok() && cfg.check_build().is_ok());
    }

    #[test]
    fn in_app_layout_has_no_lock_and_reads_ingested_tables() {
        let keys = vec![(
            StableKey(7),
            UniqueName::from("/Lotus/Weapons/Tenno/Rifle/BratonPrime"),
        )];
        let list = InputFile::Embedded {
            name: "data/spec_tables.toml",
            text: "[[table]]\n",
        };
        let cfg = PipelineConfig::in_app(
            "/data/cache".into(),
            InputTree::Embedded(&TREE),
            InputTree::Embedded(&TREE),
            list.clone(),
            keys.clone(),
        );
        assert_eq!(cfg.lock, LockMode::Latest);
        assert!(cfg.lock_file.is_none() && !cfg.update_lock && !cfg.offline);
        assert!(matches!(
            cfg.spec,
            SpecSource::Ingested(InputTree::Embedded(_))
        ));
        assert_eq!(cfg.spec_tables, list);
        assert!(cfg.validation.is_none());
        assert_eq!(cfg.stable_keys, StableKeySource::Embedded(keys));
        assert!(cfg.check_fetch().is_ok() && cfg.check_build().is_ok());
    }

    /// Contradictory settings are rejected before anything runs (`fetch --offline --update-lock`, a pinned run without
    /// a lock file, …).
    #[test]
    fn contradictory_settings_are_config_errors() {
        let base = PipelineConfig::developer(Path::new("/repo"));
        let with = |f: &dyn Fn(&mut PipelineConfig)| {
            let mut c = base.clone();
            f(&mut c);
            c
        };
        let fetch_errors = [
            // update_lock with Pinned
            with(&|c| c.update_lock = true),
            with(&|c| {
                c.update_lock = true;
                c.lock = LockMode::FromLock;
            }),
            with(&|c| {
                c.update_lock = true;
                c.lock = LockMode::Latest;
                c.offline = true;
            }),
            with(&|c| {
                c.update_lock = true;
                c.lock = LockMode::Latest;
                c.lock_file = None;
            }),
            with(&|c| {
                c.lock = LockMode::FromLock;
                c.offline = true;
            }),
            // Pinned without a lock file
            with(&|c| c.lock_file = None),
        ];
        for c in &fetch_errors {
            assert!(
                matches!(c.check_fetch(), Err(DatagenError::Config { .. })),
                "{c:?}"
            );
        }
        for lock in [LockMode::Pinned, LockMode::FromLock] {
            let c = with(&|c| {
                c.lock = lock;
                c.lock_file = None;
            });
            assert!(matches!(c.check_build(), Err(DatagenError::Config { .. })));
        }
        let fine = [
            with(&|c| {
                c.update_lock = true;
                c.lock = LockMode::Latest;
            }),
            // Pinned and offline: check that the cache holds the locked inputs.
            with(&|c| c.offline = true),
            with(&|c| c.lock = LockMode::FromLock),
            with(&|c| {
                c.lock = LockMode::Latest;
                c.lock_file = None;
                c.only = Some(SourceKind::Wfcd);
            }),
        ];
        for c in &fine {
            assert!(c.check_fetch().is_ok(), "{c:?}");
            assert!(c.check_build().is_ok(), "{c:?}");
        }
        // A build ignores the fetch-only settings.
        let c = with(&|c| {
            c.update_lock = true;
            c.offline = true;
        });
        assert!(c.check_build().is_ok());
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

    /// An embedded sub-directory (`DATA.get_dir("rules")`) behaves like that directory on disk: paths are relative to
    /// it, not to the `include_dir!` root.
    #[test]
    fn embedded_sub_directory_is_its_own_root() {
        let sub = TREE.get_dir("rules").unwrap();
        let dir = InputTree::Dir(fixture_dir().join("rules")).files().unwrap();
        let emb = InputTree::Embedded(sub).files().unwrap();
        assert_eq!(dir, emb);
        let paths: Vec<&Path> = emb.iter().map(|(p, _)| p.as_path()).collect();
        assert_eq!(paths, [Path::new("b.json"), Path::new("sub/c.txt")]);
        for tree in [
            InputTree::Dir(fixture_dir().join("rules")),
            InputTree::Embedded(sub),
        ] {
            assert_eq!(
                tree.read(Path::new("sub/c.txt")).unwrap().as_deref(),
                Some("c\n")
            );
            assert_eq!(tree.read(Path::new("rules/sub/c.txt")).unwrap(), None);
        }
    }

    #[test]
    fn read_by_relative_path() {
        for tree in [InputTree::Dir(fixture_dir()), InputTree::Embedded(&TREE)] {
            assert_eq!(
                tree.read(Path::new("rules/sub/c.txt")).unwrap().as_deref(),
                Some("c\n")
            );
            assert_eq!(
                tree.read(Path::new("./rules/./sub/c.txt"))
                    .unwrap()
                    .as_deref(),
                Some("c\n")
            );
            assert_eq!(tree.read(Path::new("missing.toml")).unwrap(), None);
            assert_eq!(tree.read(Path::new("rules")).unwrap(), None);
            assert_eq!(tree.read(Path::new("a.toml/x")).unwrap(), None);
        }
    }

    /// Paths that leave the tree are errors in both variants, never a file outside the root.
    #[test]
    fn read_rejects_paths_outside_the_tree() {
        let outside = fixture_dir().join("a.toml");
        for tree in [
            InputTree::Dir(fixture_dir().join("rules")),
            InputTree::Embedded(&TREE),
        ] {
            for bad in [
                Path::new("../a.toml"),
                Path::new("sub/../../a.toml"),
                outside.as_path(),
            ] {
                let err = tree.read(bad).unwrap_err();
                assert!(
                    matches!(&err, DatagenError::Io { source, .. }
                        if source.kind() == std::io::ErrorKind::InvalidInput),
                    "{bad:?}: {err}"
                );
            }
        }
    }

    #[test]
    fn missing_directory_is_an_io_error() {
        let err = InputTree::Dir(fixture_dir().join("nope"))
            .files()
            .unwrap_err();
        assert!(matches!(err, DatagenError::Io { .. }), "{err}");
    }

    /// A non-UTF-8 file in a raw listing fails both variants the same way.
    #[test]
    fn files_require_utf8_in_both_variants() {
        let dir = InputTree::Dir(fixture("curated_binary"))
            .files()
            .unwrap_err();
        let emb = InputTree::Embedded(&CURATED_BINARY).files().unwrap_err();
        assert!(matches!(&dir, DatagenError::Parse { .. }), "{dir}");
        assert_eq!(dir.to_string(), emb.to_string());
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

    /// Binary files where the curated loader never reads (hidden files and directories, `aliases/`, `overrides/`, an
    /// unloaded top-level file) load on disk and embedded alike.
    #[test]
    fn curated_loading_ignores_binary_files_it_does_not_read() {
        let dir = InputTree::Dir(fixture("curated_binary"))
            .load_curated()
            .unwrap();
        let emb = InputTree::Embedded(&CURATED_BINARY).load_curated().unwrap();
        assert_eq!(dir, emb);
        assert_eq!(dir, CuratedTree::default());
    }

    /// A non-UTF-8 file the loader parses fails both variants on that path (the message differs, see
    /// `InputTree::load_curated`).
    #[test]
    fn curated_loading_fails_on_a_binary_item_file() {
        for tree in [
            InputTree::Dir(fixture("curated_bad_toml")),
            InputTree::Embedded(&CURATED_BAD_TOML),
        ] {
            let Err(DatagenError::Curated(errs)) = tree.load_curated() else {
                panic!("curated error expected for {tree:?}");
            };
            let paths: Vec<&Path> = errs.0.iter().map(|e| e.path.as_path()).collect();
            assert_eq!(paths, [Path::new("aura/bad.toml")], "{errs}");
        }
    }

    #[test]
    fn input_file_reads_disk_and_embedded_text() {
        let disk = InputFile::Path(fixture_dir().join("a.toml"));
        assert_eq!(disk.read().unwrap(), "schema = 1\n");
        assert!(disk.name().ends_with("a.toml"));
        let emb = InputFile::Embedded {
            name: "data/spec_tables.toml",
            text: "x",
        };
        assert_eq!(emb.name(), "data/spec_tables.toml");
        assert_eq!(emb.read().unwrap(), "x");
        let missing = InputFile::Path(fixture_dir().join("nope.toml"));
        assert!(matches!(missing.read(), Err(DatagenError::Io { .. })));
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
