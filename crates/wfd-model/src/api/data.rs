//! Data routes and events (server-api.md §9; pipeline.md §9.2): `GET /api/data/status`, `POST /api/data/check`,
//! `/api/data/refresh` (202 [`super::JobAccepted`]), the SSE stream of [`DataEvent`]s, `/api/data/activate`.

use super::catalog::BundleSource;
use crate::ids::StableKey;
use crate::message::Message;
use serde::{Deserialize, Serialize};

/// `GET /api/data/status`, also the response of `POST /api/data/activate`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DataStatus {
    /// The active bundle.
    pub active: ActiveBundle,
    /// User bundles on disk (newest first).
    pub bundles: Vec<BundleEntry>,
    /// Id of a running refresh job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub refresh_job: Option<Box<str>>,
}

/// The active bundle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ActiveBundle {
    /// `"embedded"` or the user bundle's id.
    pub id: Box<str>,
    /// Where it came from.
    pub source: BundleSource,
    /// Data version.
    pub data_version: Box<str>,
    /// Game version the data describes.
    pub game_baseline: Box<str>,
    /// Creation time (RFC 3339).
    pub created_at: Box<str>,
}

/// A user bundle on disk.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BundleEntry {
    /// Bundle id (`wfdata-<build_label>-<hash8>`).
    pub id: Box<str>,
    /// Data version.
    pub data_version: Box<str>,
    /// Creation time (RFC 3339).
    pub created_at: Box<str>,
    /// File size (bytes).
    #[serde(with = "super::int53")]
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    pub size: u64,
    /// The bundle is active.
    pub active: bool,
}

/// `POST /api/data/check` response.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DataCheck {
    /// Something changed upstream since the active bundle.
    pub changed: bool,
    /// Public Export build label.
    pub pe_build_label: Box<str>,
    /// Wiki modules with a newer revision.
    pub wiki_changed_modules: Vec<Box<str>>,
    /// Newest WFCD tag.
    pub wfcd_latest_tag: Box<str>,
    /// Check time (RFC 3339).
    pub checked_at: Box<str>,
}

/// `POST /api/data/refresh` body (may be empty: `{}`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RefreshRequest {
    /// Curated input; absent = the default of the binary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub curated: Option<CuratedSource>,
}

/// Curated input of a refresh.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CuratedSource {
    /// The `data/curated/` embedded in the binary.
    Embedded,
}

/// `POST /api/data/activate` body.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ActivateRequest {
    /// `"embedded"` or a bundle id from `DataStatus.bundles`.
    pub bundle: Box<str>,
}

/// A data-refresh event. JSON internally tagged (`{ "event": "step", "step": "fetch", "done": 3, "total": 60 }`); on
/// the SSE stream the `event:` field carries the tag ([`DataEvent::to_sse`], [`DataEvent::from_sse`]).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DataEvent {
    /// Progress of a pipeline step.
    Step {
        /// The step.
        step: DataStep,
        /// Units done.
        done: u32,
        /// Units in the step.
        total: u32,
    },
    /// A log line.
    Log {
        /// The message.
        message: Message,
    },
    /// The report of the new bundle (before activation).
    Report {
        /// The report.
        report: Box<DataReport>,
        /// Id of the written bundle.
        bundle_id: Box<str>,
    },
    /// The refresh finished.
    Done {
        /// Id of the written bundle.
        bundle_id: Box<str>,
    },
    /// The refresh was cancelled.
    Cancelled,
    /// The refresh failed.
    Failed {
        /// The reason.
        error: Message,
    },
}

impl DataEvent {
    /// The SSE `event:` name (the variant in `snake_case`).
    pub fn name(&self) -> &'static str {
        match self {
            DataEvent::Step { .. } => "step",
            DataEvent::Log { .. } => "log",
            DataEvent::Report { .. } => "report",
            DataEvent::Done { .. } => "done",
            DataEvent::Cancelled => "cancelled",
            DataEvent::Failed { .. } => "failed",
        }
    }

    /// `true` for the events after which the server closes the stream.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            DataEvent::Done { .. } | DataEvent::Cancelled | DataEvent::Failed { .. }
        )
    }

    /// The SSE `event:` name and `data:` JSON.
    pub fn to_sse(&self) -> Result<(&'static str, String), serde_json::Error> {
        super::sse::to_sse(self, self.name())
    }

    /// Parses an SSE message.
    pub fn from_sse(event: &str, data: &str) -> Result<Self, serde_json::Error> {
        super::sse::from_sse(event, data)
    }
}

/// Pipeline steps (pipeline.md §2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DataStep {
    Fetch,
    Parse,
    Join,
    Derive,
    Effects,
    Validate,
    Emit,
}

/// Report of a refresh (pipeline.md §9.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DataReport {
    /// Field diffs against the active bundle.
    pub diff: DataDiffSummary,
    /// New items (`unmodelled` until curated).
    pub new_items: Vec<StableKey>,
    /// Stale curated items.
    pub stale: Vec<StableKey>,
    /// Auto-patched curated items.
    pub auto_patched: Vec<StableKey>,
    /// Source conflicts.
    pub conflicts: Vec<Message>,
    /// Validation errors.
    pub errors: Vec<Message>,
    /// Validation warnings.
    pub warnings: Vec<Message>,
}

/// Summary of the field diffs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DataDiffSummary {
    /// Weapons with changed fields.
    pub weapons_changed: u32,
    /// Items with changed fields.
    pub items_changed: u32,
    /// Enemies with changed fields.
    pub enemies_changed: u32,
    /// Sample changes.
    pub samples: Vec<DiffSample>,
}

/// One changed field.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiffSample {
    /// Path of the field.
    pub path: Box<str>,
    /// Old value (text).
    pub old: Box<str>,
    /// New value (text).
    pub new: Box<str>,
}
