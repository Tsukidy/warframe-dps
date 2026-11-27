//! The binary bundle (pipeline.md §8.2): the embedded `bundle.bin` that `build.rs` writes and the user bundles that an
//! in-app refresh writes (`<data_dir>/bundles/*.bin`, pipeline.md §9.2).
//!
//! ```text
//! file   = magic "WFDPSB01" | u32 schema_version | u32 header_len | header | body
//! header = postcard(BundleHeader { schema_version, data_version, content_hash: [u8; 32] /* blake3(body) */,
//!                                  created_at, game_baseline, licences, counts })
//! body   = zstd(level 19, postcard(Bundle))
//! ```
//!
//! Decoding uses `ruzstd` (pure Rust) and `postcard`, target < 100 ms on one core (the `load` benchmark of T04).

use crate::error::DataError;
use wfd_model::Bundle;

/// Encodes a bundle into the pipeline.md §8.2 file format (magic, schema version, header with the blake3 content
/// hash, zstd-compressed postcard body). Used by `build.rs`, `wfd-datagen` and the in-app refresh. Implemented by T04.
pub fn encode_bundle(b: &Bundle) -> Vec<u8> {
    let _ = b;
    todo!("T04: pipeline.md §8.2 bundle encoder")
}

/// Decodes a pipeline.md §8.2 file: checks the magic ([`DataError::BadMagic`]), the schema version against
/// `wfd_model::BUNDLE_SCHEMA_VERSION` ([`DataError::SchemaVersion`]) and the content hash
/// ([`DataError::HashMismatch`]), then decompresses and deserializes the body. Implemented by T04.
pub fn decode_bundle(bytes: &[u8]) -> Result<Bundle, DataError> {
    let _ = bytes;
    todo!("T04: pipeline.md §8.2 bundle decoder")
}
