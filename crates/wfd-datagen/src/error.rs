//! The pipeline error (frozen API of T02c). `Display` is the English diagnostic of the CLI; the server and the UI use
//! [`DatagenError::to_message`] (a `data.*` key, overview.md I-8).

use crate::{PipelineStep, Report, SourceKind};
use std::path::PathBuf;
use wfd_model::curated::CuratedErrors;
use wfd_model::message::{DataKey, Message, Severity};

/// Why a pipeline run failed. Warnings never fail a run; they go into the [`Report`].
#[derive(Debug, thiserror::Error)]
pub enum DatagenError {
    /// The run was cancelled (checked between requests and between stages, pipeline.md §3.1).
    #[error("cancelled")]
    Cancelled,
    /// `--offline`, and an input is not in the cache.
    #[error("offline: {what} is not in the source cache")]
    Offline {
        /// The missing input (module title, manifest name, file).
        what: Box<str>,
    },
    /// A request to a host outside the allow-list of pipeline.md §3.1.
    #[error("host {host} is not on the allow-list of pipeline.md §3.1")]
    HostNotAllowed {
        /// The host.
        host: Box<str>,
    },
    /// A request failed after the retries of pipeline.md §3.1.
    #[error("request to {url} failed: {status}")]
    Fetch {
        /// The host.
        host: Box<str>,
        /// The URL.
        url: Box<str>,
        /// The HTTP status (`"404"`) or the connection error.
        status: Box<str>,
    },
    /// `fetch --from-lock`: a locked input is no longer served (pipeline.md §3.3); the fix is `fetch --update-lock`.
    #[error(
        "{} input {file} at {locked} is no longer served; run `wfd-datagen fetch --update-lock` and commit the new lock \
         with the regenerated data",
        .kind.as_str()
    )]
    LockInputGone {
        /// The source.
        kind: SourceKind,
        /// The input (module title, manifest name, file name).
        file: Box<str>,
        /// Its locked revid, content hash or tag.
        locked: Box<str>,
    },
    /// The lock file is missing, unreadable, malformed, or lacks an input the run needs.
    #[error("lock file {}: {detail}", .path.display())]
    Lock {
        /// The lock file.
        path: PathBuf,
        /// What is wrong.
        detail: String,
    },
    /// A wiki Lua module failed to evaluate or tripped a sandbox limit (pipeline.md §4.1): the build fails with its title.
    #[error("Lua module {title}: {detail}")]
    Lua {
        /// The module title (`Module:Weapons/data/primary`).
        title: Box<str>,
        /// The Lua error or the limit that tripped.
        detail: String,
    },
    /// An input does not parse: a wire record, a Public Export manifest, a WFCD file, a JSON or TOML input.
    #[error("{file}: {detail}")]
    Parse {
        /// The input.
        file: Box<str>,
        /// The parser's diagnostic.
        detail: String,
    },
    /// The curated tree does not load (`wfd_model::curated`: schema, unknown keys, unexpected files).
    #[error("curated data:\n{0}")]
    Curated(CuratedErrors),
    /// A spec table cannot be ingested: missing heading, changed column header, unparsable row (pipeline.md §5.2).
    #[error("spec table {id}: {detail}")]
    SpecTable {
        /// The table id of `data/spec_tables.toml`.
        id: Box<str>,
        /// What is wrong.
        detail: String,
    },
    /// A `--check` mode found generated output that is out of date (`spec-tables --check`, a reproducibility check).
    #[error("{what} is stale: {detail}")]
    Stale {
        /// The generated file or directory.
        what: Box<str>,
        /// What differs.
        detail: String,
    },
    /// The validation checks of pipeline.md §7 found errors; the report holds them with every warning.
    #[error("{} validation error(s)", .report.errors.len())]
    Validation {
        /// The full report of the run.
        report: Box<Report>,
    },
    /// The bundle could not be encoded or decoded (`wfd-data`).
    #[error("bundle: {detail}")]
    Bundle {
        /// The codec's diagnostic.
        detail: String,
    },
    /// A file-system operation failed.
    #[error("I/O error on {}", .path.display())]
    Io {
        /// The file or directory.
        path: PathBuf,
        /// The I/O error.
        source: std::io::Error,
    },
}

impl DatagenError {
    /// The pipeline step the error belongs to, if any.
    pub fn step(&self) -> Option<PipelineStep> {
        use DatagenError as E;
        match self {
            E::Offline { .. }
            | E::HostNotAllowed { .. }
            | E::Fetch { .. }
            | E::LockInputGone { .. }
            | E::Lock { .. } => Some(PipelineStep::Fetch),
            E::Lua { .. } | E::Parse { .. } => Some(PipelineStep::Parse),
            E::Curated(_) | E::SpecTable { .. } => Some(PipelineStep::Effects),
            E::Validation { .. } => Some(PipelineStep::Validate),
            E::Stale { .. } | E::Bundle { .. } => Some(PipelineStep::Emit),
            E::Cancelled | E::Io { .. } => None,
        }
    }

    /// The error as a UI message (severity error): `data.host_not_allowed`, `data.fetch_failed` and
    /// `data.lock_input_gone` for their cases, otherwise `data.refresh_failed` with the step (`""` when none) and the
    /// English diagnostic with its causes as `detail` (the untranslated parser text, message/data.rs). A cancelled
    /// refresh is the `DataEvent::Cancelled` event, not a message; this gives `data.refresh_failed` with detail
    /// `cancelled`.
    pub fn to_message(&self) -> Message {
        let m = match self {
            DatagenError::HostNotAllowed { host } => {
                DataKey::HostNotAllowed.message().arg("host", host.clone())
            }
            DatagenError::Fetch { host, status, .. } => DataKey::FetchFailed
                .message()
                .arg("host", host.clone())
                .arg("status", status.clone()),
            DatagenError::LockInputGone { kind, file, locked } => DataKey::LockInputGone
                .message()
                .arg("source", kind.as_str())
                .arg("file", file.clone())
                .arg("locked", locked.clone()),
            other => DataKey::RefreshFailed
                .message()
                .arg("step", other.step().map_or("", step_name))
                .arg("detail", chain(other)),
        };
        m.with_severity(Severity::Error)
    }
}

/// The error and its causes on one line (`a: b: c`).
fn chain(e: &dyn std::error::Error) -> String {
    let mut text = e.to_string();
    let mut cause = e.source();
    while let Some(c) = cause {
        text.push_str(": ");
        text.push_str(&c.to_string());
        cause = c.source();
    }
    text
}

/// The readable name of a step (`"fetch"`, the JSON form of `DataStep`).
fn step_name(step: PipelineStep) -> &'static str {
    match step {
        PipelineStep::Fetch => "fetch",
        PipelineStep::Parse => "parse",
        PipelineStep::Join => "join",
        PipelineStep::Derive => "derive",
        PipelineStep::Effects => "effects",
        PipelineStep::Validate => "validate",
        PipelineStep::Emit => "emit",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wfd_model::curated::CuratedError;

    fn every_variant() -> Vec<DatagenError> {
        vec![
            DatagenError::Cancelled,
            DatagenError::Offline {
                what: "Module:Stances/data".into(),
            },
            DatagenError::HostNotAllowed {
                host: "example.com".into(),
            },
            DatagenError::Fetch {
                host: "wiki.warframe.com".into(),
                url: "https://wiki.warframe.com/api.php".into(),
                status: "503".into(),
            },
            DatagenError::LockInputGone {
                kind: SourceKind::Pe,
                file: "ExportWeapons_en.json".into(),
                locked: "abc123".into(),
            },
            DatagenError::Lock {
                path: "data/sources.lock.toml".into(),
                detail: "no [wfcd] tag".into(),
            },
            DatagenError::Lua {
                title: "Module:Weapons/data/melee".into(),
                detail: "instruction limit".into(),
            },
            DatagenError::Parse {
                file: "Primary.json".into(),
                detail: "EOF".into(),
            },
            DatagenError::Curated(CuratedErrors(vec![CuratedError {
                path: "aura/x.toml".into(),
                message: "unknown field".into(),
            }])),
            DatagenError::SpecTable {
                id: "co_catalogue".into(),
                detail: "column header changed".into(),
            },
            DatagenError::Stale {
                what: "data/generated/spec/".into(),
                detail: "co_catalogue.json differs".into(),
            },
            DatagenError::Validation {
                report: Box::default(),
            },
            DatagenError::Bundle {
                detail: "schema_version 2".into(),
            },
            DatagenError::Io {
                path: "data/curated".into(),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            },
        ]
    }

    /// Every message carries exactly the arguments its key declares (message/data.rs), so the UI can render it.
    #[test]
    fn messages_carry_the_declared_arguments() {
        for e in every_variant() {
            let m = e.to_message();
            let key = DataKey::parse(m.key.as_str())
                .unwrap_or_else(|| panic!("{} is not a data key", m.key.as_str()));
            let mut declared: Vec<&str> = key.args().to_vec();
            declared.sort_unstable();
            let given: Vec<&str> = m.args.keys().map(|k| &**k).collect();
            assert_eq!(given, declared, "{e:?}");
            assert_eq!(m.severity, Some(Severity::Error));
        }
    }

    #[test]
    fn specific_keys_and_steps() {
        let gone = DatagenError::LockInputGone {
            kind: SourceKind::Wiki,
            file: "Module:Stances/data".into(),
            locked: "123456".into(),
        };
        assert_eq!(gone.to_message().key, DataKey::LockInputGone);
        assert!(gone.to_string().contains("fetch --update-lock"), "{gone}");
        assert_eq!(gone.step(), Some(PipelineStep::Fetch));
        let lua = DatagenError::Lua {
            title: "Module:Stances/data".into(),
            detail: "memory limit".into(),
        };
        let m = lua.to_message();
        assert_eq!(m.key, DataKey::RefreshFailed);
        assert_eq!(m.get("step"), Some(&"parse".into()));
        assert!(
            matches!(m.get("detail"), Some(wfd_model::MsgArg::Str(s)) if s.contains("Module:Stances/data"))
        );
        assert_eq!(
            DatagenError::Cancelled.to_message().get("step"),
            Some(&"".into())
        );
    }

    /// The causes reach the UI's detail text; the CLI prints them as "caused by" lines.
    #[test]
    fn detail_includes_the_causes() {
        let io = DatagenError::Io {
            path: "data/curated".into(),
            source: std::io::Error::other("disk full"),
        };
        assert_eq!(io.to_string(), "I/O error on data/curated");
        assert!(matches!(io.to_message().get("detail"),
            Some(wfd_model::MsgArg::Str(s)) if &**s == "I/O error on data/curated: disk full"));
    }

    /// `step_name` is the JSON form of `DataStep` (what `DataEvent::Step` sends).
    #[test]
    fn step_names_are_the_json_form() {
        use PipelineStep as S;
        for s in [
            S::Fetch,
            S::Parse,
            S::Join,
            S::Derive,
            S::Effects,
            S::Validate,
            S::Emit,
        ] {
            assert_eq!(serde_json::to_value(s).unwrap(), step_name(s));
        }
    }
}
