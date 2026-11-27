//! The binary bundle (pipeline.md §8.2): the embedded `bundle.bin` that `build.rs` writes and the user bundles that an
//! in-app refresh writes (`<data_dir>/bundles/*.bin`, pipeline.md §9.2).
//!
//! ```text
//! file   = magic "WFDPSB01" | u32 schema_version | u32 header_len | header | body
//! header = postcard(BundleHeader { schema_version, data_version, content_hash: [u8; 32] /* blake3(body) */,
//!                                  created_at, game_baseline, licences, counts })
//! body   = zstd frame of postcard(Bundle)
//! ```
//!
//! Compression (contract review API-4, DC-28; pipeline.md §8.2): `build.rs` compresses the embedded bundle at zstd
//! level 19 with its build-dependency `zstd`; [`encode_bundle`], which runs in the shipped code (`wfd-datagen build
//! --bundle`, the in-app refresh), compresses with the pure-Rust encoder of `ruzstd` (its `Fastest` level, the only
//! compressing level ruzstd 0.9 implements), so `wfd-data` links no C library at run time and a WebAssembly build
//! stays possible (overview.md §3.1). The file format is the same either way and [`decode_bundle`] reads both. T04
//! writes the framing once (in a module `build.rs` includes) and passes the compressor.
//!
//! Decoding uses `ruzstd` (pure Rust) and `postcard`, target < 100 ms on one core (the `load` benchmark of T04).
//! [`read_header`] reads the header alone (no decompression): listing user bundles (`DataStatus.bundles`, pipeline.md
//! §9.2 "Activate" and "Rollback", `warframe-dps data list`) and choosing the newest loadable bundle at startup
//! (server-api.md §1.2 step 2).

use crate::error::DataError;
use serde::{Deserialize, Serialize};
use wfd_model::{Bundle, LicenceNotice};

/// The header of a bundle file (pipeline.md §8.2), postcard-encoded between the fixed prefix and the body. The field
/// order is the postcard layout, so a change of it is a `BUNDLE_SCHEMA_VERSION` change; the prefix carries the schema
/// version before the header, so a reader rejects a foreign layout without decoding it. Addition of the contract
/// review (API-2, DC-28; CONTRACTS.md "Contract review fixes (crates)").
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BundleHeader {
    /// `wfd_model::BUNDLE_SCHEMA_VERSION` of the writer (equal to the prefix's).
    pub schema_version: u32,
    /// `Manifest.data_version` of the bundle.
    pub data_version: Box<str>,
    /// blake3 of the body bytes (the compressed frame), checked by [`decode_bundle`].
    pub content_hash: [u8; 32],
    /// `Manifest.created_at` (RFC 3339).
    pub created_at: Box<str>,
    /// `Manifest.game_baseline`.
    pub game_baseline: Box<str>,
    /// `Manifest.licences` (pipeline.md §10).
    pub licences: Vec<LicenceNotice>,
    /// Record counts of the body.
    pub counts: BundleCounts,
}

/// Record counts of a bundle (`BundleHeader.counts`): the lengths of the `Bundle` vectors a bundle listing shows.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BundleCounts {
    /// `Bundle.weapons`.
    pub weapons: u32,
    /// `Bundle.items`.
    pub items: u32,
    /// `Bundle.enemies`.
    pub enemies: u32,
    /// `Bundle.frames`.
    pub frames: u32,
    /// `Bundle.stable_keys`.
    pub stable_keys: u32,
}

/// Encodes a bundle into the pipeline.md §8.2 file format (magic, schema version, header with the blake3 content
/// hash, zstd-compressed postcard body) with the run-time compressor (`ruzstd`, module doc). Used by `wfd-datagen`
/// (`build --bundle`) and the in-app refresh; `build.rs` shares the framing and compresses at zstd level 19.
/// Deterministic: equal bundles give equal bytes. Implemented by T04.
pub fn encode_bundle(b: &Bundle) -> Vec<u8> {
    let _ = b;
    todo!("T04: pipeline.md §8.2 bundle encoder")
}

/// Reads the header of a pipeline.md §8.2 file without decompressing the body: checks the magic
/// ([`DataError::BadMagic`]) and the prefix's schema version against `wfd_model::BUNDLE_SCHEMA_VERSION`
/// ([`DataError::SchemaVersion`], whose `found` names the foreign version), then decodes the header
/// ([`DataError::Corrupt`] when it is truncated or does not decode). The content hash is not checked (it needs the
/// body). Implemented by T04; [`decode_bundle`] starts with it.
pub fn read_header(bytes: &[u8]) -> Result<BundleHeader, DataError> {
    let _ = bytes;
    todo!("T04: pipeline.md §8.2 bundle header")
}

/// Decodes a pipeline.md §8.2 file: [`read_header`] (magic, schema version), then the content hash
/// ([`DataError::HashMismatch`]), then decompresses and deserializes the body. Implemented by T04.
pub fn decode_bundle(bytes: &[u8]) -> Result<Bundle, DataError> {
    let _ = bytes;
    todo!("T04: pipeline.md §8.2 bundle decoder")
}
