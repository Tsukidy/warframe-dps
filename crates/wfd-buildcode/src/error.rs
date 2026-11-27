//! Why a code or link could not be decoded, or a document could not be exported (server-api.md §7).

use std::fmt;
use wfd_model::message::{ImportKey, Message};

/// Error of [`crate::decode`], [`crate::migrate`], [`crate::import_inspiration`] and
/// [`crate::export_inspiration`]. Problems that do not stop a decode (unmapped link fields, unit conversions,
/// migrations, retired items) are not errors: they go into the report (`ImportReport`; the import never rejects a
/// link it could parse, 09 §2.4). [`CodeError::messages`] gives the i18n form the server returns (overview.md I-8).
///
/// `#[non_exhaustive]`, like `wfd_data::DataError`: T10 may add a kind (a new variant is the backward-compatible change
/// of work-breakdown.md §2.1), so matchers outside this crate keep a fallback arm, which [`CodeError::messages`] covers.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum CodeError {
    /// The text is neither a `wfd1-` code, our `?b=` URL, nor a warframe-damage.com link (`import.unrecognized`).
    #[error("not a share code or a supported link")]
    Unrecognized,
    /// A decoding stage failed (`import.corrupt [stage]`): bad base64, bad DEFLATE or LZString data, or JSON that is
    /// not a document of the code's version.
    #[error("corrupt code ({stage}): {detail}")]
    Corrupt {
        /// The stage that failed.
        stage: DecodeStage,
        /// The decoder's error text, for logs (not shown to the user).
        detail: Box<str>,
    },
    /// The document version is newer than this binary understands (`import.version_unsupported [v, max]`;
    /// `max` = `wfd_model::DOC_VERSION`).
    #[error("document version {v} is newer than the supported version {max}")]
    VersionUnsupported {
        /// The version found.
        v: u16,
        /// The newest version this binary reads.
        max: u16,
    },
    /// The document has issues that reject it (`doc.*` messages, `wfd_model::message::DocIssue`; the 422 `issues`
    /// of server-api.md §6.1), e.g. an unknown key in a document [`crate::export_inspiration`] must resolve.
    #[error("the document is invalid ({} issue(s))", .0.len())]
    Invalid(Vec<Message>),
    /// The scenario uses something the inspiration cannot express (`import.export_unsupported [feature]`;
    /// work-breakdown.md T10); `feature` names the document field or item.
    #[error("the inspiration cannot express {feature}")]
    ExportUnsupported {
        /// The document field or feature (`"toggles.melee_plans"`, `"build.variant.zaw"` …).
        feature: Box<str>,
    },
}

impl CodeError {
    /// The error as i18n messages: one `import.*` message, or the document issues of [`CodeError::Invalid`].
    pub fn messages(&self) -> Vec<Message> {
        match self {
            CodeError::Unrecognized => vec![Message::new(ImportKey::Unrecognized)],
            CodeError::Corrupt { stage, .. } => {
                vec![Message::new(ImportKey::Corrupt).arg("stage", stage.as_str())]
            }
            CodeError::VersionUnsupported { v, max } => vec![
                Message::new(ImportKey::VersionUnsupported)
                    .arg("v", u32::from(*v))
                    .arg("max", u32::from(*max)),
            ],
            CodeError::Invalid(issues) => issues.clone(),
            CodeError::ExportUnsupported { feature } => {
                vec![Message::new(ImportKey::ExportUnsupported).arg("feature", feature.clone())]
            }
        }
    }
}

/// The decoding stage of a [`CodeError::Corrupt`]; its name is the `stage` argument of `import.corrupt`.
/// `#[non_exhaustive]` for the same reason as [`CodeError`]: a new stage (e.g. URL parsing) is a new variant, and its
/// name joins the `stage` values that `import.corrupt` lists (`wfd_model::message::ImportKey`, a CONTRACTS.md entry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DecodeStage {
    /// base64url of a `wfd1-` code, or the legacy base64 of an inspiration link.
    Base64,
    /// Raw DEFLATE of a `wfd1-` code.
    Deflate,
    /// LZString of an inspiration link (`?p=…&n=1`).
    Lz,
    /// The JSON document (or the inspiration's JSON object).
    Json,
}

impl DecodeStage {
    /// `"base64"`, `"deflate"`, `"lz"` or `"json"` (the `stage` values of `import.corrupt`).
    pub fn as_str(self) -> &'static str {
        match self {
            DecodeStage::Base64 => "base64",
            DecodeStage::Deflate => "deflate",
            DecodeStage::Lz => "lz",
            DecodeStage::Json => "json",
        }
    }
}

impl fmt::Display for DecodeStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
