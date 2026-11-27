//! Fetch stage (pipeline.md §3): the polite client, the source cache and the lock file. Owner: T05
//! (work-breakdown.md §8); skeleton by T02c with the signatures the data tasks and the server share.
//!
//! T05 provides `PoliteClient` (§3.1: allowed hosts only, one request per second per host, retries on 429/503/509 with
//! `Retry-After`, `maxlag=5`, conditional requests, cancellation between requests), the cache layout and its
//! `index.json` (§3.3), lock-file read and write, the three fetchers (§3.2) and WFCD version discovery through
//! `registry.npmjs.org`.

pub mod cli;

use crate::{DatagenError, PipelineConfig, Progress};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use wfd_model::api::DataCheck;
use wfd_model::{Manifest, SourceStamp};

/// The User-Agent of every request (pipeline.md §3.1): no personal data, no contact address.
pub const USER_AGENT: &str = concat!(
    "warframe-dps-datagen/",
    env!("CARGO_PKG_VERSION"),
    " (local non-commercial tool)"
);

/// The only hosts the pipeline contacts (pipeline.md §3.1): wiki Lua modules, Public Export manifests and item images,
/// WFCD files at a tag, WFCD version discovery.
pub const ALLOWED_HOSTS: [&str; 4] = [
    "wiki.warframe.com",
    "content.warframe.com",
    "raw.githubusercontent.com",
    "registry.npmjs.org",
];

/// `true` when `host` is one of [`ALLOWED_HOSTS`] (exact name, ASCII case-insensitive).
pub fn is_allowed_host(host: &str) -> bool {
    ALLOWED_HOSTS.iter().any(|h| h.eq_ignore_ascii_case(host))
}

/// What a fetch did (CLI summary, server log). Its `inputs` become `Manifest.sources` of the bundle built from them.
/// `#[non_exhaustive]`: a field added later does not break callers outside this crate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FetchReport {
    /// Every input in the cache after the fetch with its revision: wiki module title + revid + timestamp, Public
    /// Export manifest name + content hash, WFCD file name + tag and SHA-256.
    pub inputs: Vec<SourceStamp>,
    /// Requests that downloaded a body.
    pub downloaded: u32,
    /// Inputs that were already current (same revid or hash, `304 Not Modified`).
    pub unchanged: u32,
    /// Bytes downloaded.
    pub bytes: u64,
    /// Public Export `buildLabel` of the index, when the index was read.
    pub build_label: Option<Box<str>>,
    /// The WFCD tag of the fetched files.
    pub wfcd_tag: Option<Box<str>>,
    /// The lock file was written (`fetch --update-lock`).
    pub lock_written: bool,
}

/// `fetch` with [`LockMode::Pinned`](crate::LockMode::Pinned) or [`LockMode::Latest`](crate::LockMode::Latest)
/// (called by [`crate::fetch()`]): fills the cache with the pinned or the newest inputs of the sources selected by
/// [`PipelineConfig::only`]; with `update_lock`, writes the new pins.
pub fn fetch_sources(
    cfg: &PipelineConfig,
    progress: &dyn Progress,
    cancel: &AtomicBool,
) -> Result<FetchReport, DatagenError> {
    let _ = (cfg, progress, cancel);
    todo!("T05: fetch the newest or pinned inputs (pipeline.md §3.2)")
}

/// `fetch --from-lock` (pipeline.md §3.3): fills an empty cache with exactly the inputs of the lock file, wiki modules
/// by revision id, Public Export manifests by content hash, WFCD files at the locked tag, each checked against the lock
/// (revid, hash, SHA-256). An input that is no longer served is [`DatagenError::LockInputGone`].
pub fn fetch_from_lock(
    cfg: &PipelineConfig,
    p: &dyn Progress,
    cancel: &AtomicBool,
) -> Result<FetchReport, DatagenError> {
    let _ = (cfg, p, cancel);
    todo!("T05: restore the locked inputs (pipeline.md §3.3)")
}

/// "Check for updates" (`POST /api/data/check`, pipeline.md §9.2): asks the Public Export index (ETag), the wiki
/// revids and the npm registry (2-3 requests) whether anything changed since the inputs of `active`. Addition to
/// pipeline.md §2 for the server (CONTRACTS.md T02c); proposed owner T05, called by T34's route.
pub fn check_updates(
    cfg: &PipelineConfig,
    active: &Manifest,
    cancel: &AtomicBool,
) -> Result<DataCheck, DatagenError> {
    let _ = (cfg, active, cancel);
    todo!("T05: check upstream revisions against the active bundle (pipeline.md §9.2)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{code_spans, pipeline_md, table_cell};

    /// The User-Agent is the one of the pipeline.md §3.1 row, with `<version>` the crate version.
    #[test]
    fn user_agent_is_that_of_pipeline_md() {
        let cell = table_cell(&pipeline_md(), "User-Agent");
        let spans = code_spans(&cell);
        assert_eq!(
            spans[0].replace("<version>", env!("CARGO_PKG_VERSION")),
            USER_AGENT
        );
        assert!(cell.contains("no contact address"), "{cell}");
        assert!(
            !USER_AGENT.contains('@'),
            "no contact address (pipeline.md §3.1)"
        );
    }

    /// The hosts are exactly those of the pipeline.md §3.1 "Hosts allowed" row before "nothing else"; the hosts the row
    /// names after it (WFCD's image CDN) are not allowed.
    #[test]
    fn hosts_are_those_of_pipeline_md() {
        let cell = table_cell(&pipeline_md(), "Hosts allowed");
        let (allowed, rest) = cell
            .split_once("nothing else")
            .expect("`nothing else` in the §3.1 hosts row");
        assert_eq!(code_spans(allowed), ALLOWED_HOSTS);
        let excluded = code_spans(rest);
        assert!(!excluded.is_empty(), "{rest}");
        for h in &excluded {
            assert!(!is_allowed_host(h), "{h}");
        }
    }

    #[test]
    fn host_allow_list_matches_exact_names() {
        for h in ALLOWED_HOSTS {
            assert!(is_allowed_host(h));
        }
        assert!(is_allowed_host("Wiki.Warframe.com"));
        for h in [
            "api.github.com",
            "origin.warframe.com",
            "warframe.com",
            "evil.wiki.warframe.com",
            "wiki.warframe.com.evil",
        ] {
            assert!(!is_allowed_host(h), "{h}");
        }
    }
}
