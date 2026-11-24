//! Error bodies (server-api.md §2): every non-2xx response has the body `{ "error": Message }`; 422 `api.invalid`
//! adds the document issues as `issues`; 500 `api.internal` carries `error_id` in `error.args`; 409 conflicts carry
//! the running job in `error.args.job`.

use crate::message::{ApiKey, Message};
use serde::{Deserialize, Serialize};

/// Body of a non-2xx response.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ErrorResponse {
    /// The error (`api.*`, `opt.job_running`, `data.refresh_running` …).
    pub error: Message,
    /// The reasons of a 422 `api.invalid` (doc and import keys); omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<Message>,
}

impl ErrorResponse {
    /// An error without issues.
    pub fn new(error: Message) -> Self {
        ErrorResponse {
            error,
            issues: Vec::new(),
        }
    }

    /// 422 `api.invalid` with the document issues.
    pub fn invalid(issues: Vec<Message>) -> Self {
        ErrorResponse {
            error: ApiKey::Invalid.message(),
            issues,
        }
    }
}
