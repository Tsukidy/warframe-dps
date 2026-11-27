//! `wfd-buildcode`: build documents as text (overview.md §3; AD-18; server-api.md §7).
//!
//! - [`encode`], [`decode`], [`migrate`]: our `wfd1-` share codes and `?b=` URLs; Rust is their only codec (AD-18).
//! - [`import_inspiration`], [`export_inspiration`]: warframe-damage.com links (`?p=…&n=1` LZString, legacy base64),
//!   mapped as spec 09 §2.4.
//!
//! Decoding reports what it changed or could not map in an [`ImportReport`] ([`DecodeReport`] for our codes);
//! document issues are `doc.*` messages (`wfd_model::message::DocIssue`, ruling C-18), import problems `import.*`
//! messages (overview.md I-8). Depends on `wfd-model` and `wfd-data` (overview.md §3.1).
//!
//! The public API was fixed by T02b (work-breakdown.md T02b) and is implemented by T10; every body T10 owns is
//! `todo!("T10 …")`, and the tests that need one are `#[ignore = "T10 …"]`. A change to these signatures is a
//! `CONTRACTS.md` entry (work-breakdown.md §2.1).

#![warn(missing_docs)]

mod error;
mod inspiration;
mod wfd1;

pub use error::{CodeError, DecodeStage};
pub use inspiration::{export_inspiration, import_inspiration};
pub use wfd_model::api::{CodeSource, DecodeReport, ImportReport};
pub use wfd1::{CODE_PREFIX, decode, encode, migrate};
