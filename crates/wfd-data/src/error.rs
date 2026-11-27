//! Errors of loading game data (pipeline.md §8.2, §8.3; testing.md §4 "Bundle").

use std::path::PathBuf;

/// Why a bundle or a JSON data directory could not be loaded.
///
/// A broken embedded bundle fails the build in `build.rs`, not the user (pipeline.md §8.2); a user bundle that fails
/// to load is skipped and the server starts on the embedded one (pipeline.md §9.2 "Startup"). New variants may be
/// added by T04 (`#[non_exhaustive]`).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DataError {
    /// A file or directory could not be read.
    #[error("{}: {source}", .path.display())]
    Io {
        /// The file or directory.
        path: PathBuf,
        /// The I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A JSON file of the pipeline.md §8.1 layout does not deserialize into the `wfd-model` types (the schema check of
    /// pipeline.md §8.2).
    #[error("{}: {message}", .path.display())]
    Json {
        /// The file.
        path: PathBuf,
        /// The serde_json error text (line and column included).
        message: Box<str>,
    },
    /// The bytes do not start with the magic `WFDPSB01` (pipeline.md §8.2 file layout).
    #[error("not a bundle file (magic WFDPSB01 missing)")]
    BadMagic,
    /// The bundle's `schema_version` is not this binary's `wfd_model::BUNDLE_SCHEMA_VERSION`; postcard is not
    /// self-describing, so such a bundle cannot be read (pipeline.md §8.2; §9.2 "Startup"; testing.md §4 "Bundle").
    #[error("bundle schema version {found}, this binary reads {expected}")]
    SchemaVersion {
        /// The version the bundle header carries.
        found: u32,
        /// The version this binary reads.
        expected: u32,
    },
    /// `blake3(body)` differs from the header's `content_hash` (pipeline.md §8.2).
    #[error("bundle content hash mismatch")]
    HashMismatch,
    /// The header or the body failed to decode (truncated file, zstd or postcard error).
    #[error("bundle could not be decoded: {0}")]
    Corrupt(Box<str>),
    /// The reference validator rejected the data (pipeline.md §8.2: stable keys unique, ids dense, effect targets
    /// exist); one entry per problem.
    #[error("game data failed validation: {}", .0.join("; "))]
    Invalid(Vec<Box<str>>),
}
