//! Engine errors (engine.md §15). An error is never a panic; it travels as a [`Message`] keyed by an `EngineKey`
//! (`{ key, args }`, server-api.md §4). Complete in T02a (no implementing task owns this file).

use std::fmt;

use wfd_model::EffectKey;
use wfd_model::message::{EngineKey, Message};

/// Why an evaluation failed (engine.md §15).
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// The build cannot be evaluated; the reasons are document or legality messages.
    InvalidBuild(Vec<Message>),
    /// A record the build or scenario names is missing from the bundle.
    MissingData {
        /// What is missing (an id, a table name).
        what: Box<str>,
    },
    /// An effect uses a construct the engine does not implement.
    UnsupportedMechanic {
        /// The effect.
        effect: EffectKey,
    },
    /// The schedule would exceed 10^7 events (horizon × rate).
    ScheduleTooLong {
        /// Events the schedule would hold.
        shots: u64,
    },
    /// Two distinct mechanic id strings hash to one FNV-1a value (engine.md §8.2, 08 E-8.16).
    MechanicIdCollision {
        /// First id string.
        a: Box<str>,
        /// Second id string.
        b: Box<str>,
    },
    /// An anchor index outside the ranges of 08 E-8.16 (a ≥ 4096, i ≥ 2^20, j or b ≥ 65536; engine.md §8.2).
    RngIndexRange {
        /// Which index (`"attack"`, `"shot"`, `"sub_instance"`, `"block"`).
        what: Box<str>,
        /// The value out of range.
        value: u64,
    },
    /// The calculation was cancelled through its [`crate::CancelToken`] (a superseded request, server-api.md §11).
    /// Addition to engine.md §15 (CONTRACTS.md T02a): `Engine::calculate` returns `EngineError`.
    Cancelled,
}

impl EngineError {
    /// The error as a message: `{ key, args }` with the `EngineKey` of the variant; `InvalidBuild` carries its
    /// reasons after the head message in [`EngineError::messages`].
    pub fn message(&self) -> Message {
        match self {
            EngineError::InvalidBuild(_) => EngineKey::InvalidBuild.message(),
            EngineError::MissingData { what } => {
                EngineKey::MissingData.message().arg("what", what.clone())
            }
            EngineError::UnsupportedMechanic { effect } => EngineKey::UnsupportedMechanic
                .message()
                .arg("effect", effect.as_str()),
            EngineError::ScheduleTooLong { shots } => EngineKey::ScheduleTooLong
                .message()
                .arg("shots", *shots as f64),
            EngineError::MechanicIdCollision { a, b } => EngineKey::MechanicIdCollision
                .message()
                .arg("a", a.clone())
                .arg("b", b.clone()),
            EngineError::RngIndexRange { what, value } => EngineKey::RngIndexRange
                .message()
                .arg("what", what.clone())
                .arg("value", *value as f64),
            EngineError::Cancelled => EngineKey::Cancelled.message(),
        }
    }

    /// The head message followed by the reasons of `InvalidBuild` (the 422 `issues` shape, server-api.md §6.1).
    pub fn messages(&self) -> Vec<Message> {
        let mut out = vec![self.message()];
        if let EngineError::InvalidBuild(reasons) = self {
            out.extend(reasons.iter().cloned());
        }
        out
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::InvalidBuild(reasons) => {
                write!(f, "invalid build ({} reason(s))", reasons.len())
            }
            EngineError::MissingData { what } => write!(f, "missing data: {what}"),
            EngineError::UnsupportedMechanic { effect } => {
                write!(f, "unsupported mechanic in effect {effect}")
            }
            EngineError::ScheduleTooLong { shots } => {
                write!(f, "schedule too long: {shots} events")
            }
            EngineError::MechanicIdCollision { a, b } => {
                write!(f, "mechanic ids {a:?} and {b:?} share one hash")
            }
            EngineError::RngIndexRange { what, value } => {
                write!(f, "RNG anchor index {what} = {value} out of range")
            }
            EngineError::Cancelled => f.write_str("cancelled"),
        }
    }
}

impl std::error::Error for EngineError {}

/// A Monte Carlo run set was cancelled between runs (engine.md §8.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cancelled;

impl Cancelled {
    /// `engine.cancelled`.
    pub fn message(self) -> Message {
        EngineKey::Cancelled.message()
    }
}

impl fmt::Display for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cancelled")
    }
}

impl std::error::Error for Cancelled {}

impl From<Cancelled> for EngineError {
    fn from(_: Cancelled) -> Self {
        EngineError::Cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_carry_engine_keys_and_args() {
        let e = EngineError::MechanicIdCollision {
            a: "x".into(),
            b: "y".into(),
        };
        let m = e.message();
        assert_eq!(m.key.as_str(), "engine.mechanic_id_collision");
        assert_eq!(m.args.len(), 2);
        let e = EngineError::RngIndexRange {
            what: "shot".into(),
            value: 1 << 20,
        };
        assert_eq!(e.message().key.as_str(), "engine.rng_index_range");
        assert_eq!(
            EngineError::from(Cancelled).message().key.as_str(),
            "engine.cancelled"
        );
        let inv = EngineError::InvalidBuild(vec![Cancelled.message()]);
        assert_eq!(inv.messages().len(), 2);
        assert!(!inv.to_string().is_empty());
    }
}
