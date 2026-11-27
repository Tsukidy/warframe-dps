//! Probe paths and the probe registry (engine.md §11.2-§11.3). A probe names one spec symbol of a scenario:
//!
//! ```text
//! probe     := "enemy." name | "attack[" slug "]." name | "attack[" slug "].view[" view "]." name | "group[" n "]." name
//! name      := segment ( "." segment )*          segment := ident ( "[" index "]" )?
//! index     := DamageType | StatusType | integer
//! state     := "@fresh" (default) | "@steady" | "@stacks(" STATUS "=" n ( "," STATUS "=" n )* ")"   (view probes)
//! ```
//!
//! An unknown probe is a test failure, so a topic document can force the engine to expose an intermediate value by
//! naming it in a vector; [`registry`] lists every supported path with its symbol. Owner: T17.

use std::fmt;

use smallvec::SmallVec;
use wfd_model::{DamageType, StatusType, View};

/// A parsed probe path (engine.md §11.2).
#[derive(Debug, Clone, PartialEq)]
pub struct ProbePath {
    /// What the path starts at.
    pub scope: ProbeScope,
    /// The dotted name after the scope (`ratio_q[COR]`, `hit.k0.total`, `dot_tick[SLA].k0`).
    pub name: SmallVec<[ProbeSegment; 3]>,
    /// The target-state selector of a view probe (spec 00 §3.8 states A and B); `Fresh` by default.
    pub state: StateSel,
}

/// The scope of a probe.
#[derive(Debug, Clone, PartialEq)]
pub enum ProbeScope {
    /// `enemy.` (05 quantities).
    Enemy,
    /// `attack[<slug>].` (E-1..E-11, 06).
    Attack(Box<str>),
    /// `attack[<slug>].view[<view>].` (E-12..E-14, spec 00 §4.1).
    AttackView(Box<str>, View),
    /// `group[<n>].` (08 §9).
    Group(u8),
}

/// One segment of a probe name.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSegment {
    /// The identifier.
    pub ident: Box<str>,
    /// Its optional index.
    pub index: Option<ProbeIndex>,
}

/// An index of a probe segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeIndex {
    /// A damage type (`ratio[COR]`).
    Type(DamageType),
    /// A status type (`proc_rate[COR]`, `dot_tick[SLA]`).
    Status(StatusType),
    /// An integer (`p_tier[2]`).
    Int(u32),
}

/// Target state of a view probe (engine.md §11.2).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StateSel {
    /// `@fresh`: the fresh state of spec 00 §4.1 (default).
    #[default]
    Fresh,
    /// `@steady`: the steady state (08 E-8.13).
    Steady,
    /// `@stacks(COR=10,PUN=5)`: fixed stack counts.
    Stacks(SmallVec<[(StatusType, u8); 4]>),
}

impl ProbePath {
    /// Parses a probe path (the grammar above). T17.
    pub fn parse(text: &str) -> Result<ProbePath, ProbeError> {
        let _ = text;
        todo!("T17")
    }
}

impl fmt::Display for ProbePath {
    /// The canonical text of the path (round trip of [`ProbePath::parse`]). T17.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!("T17")
    }
}

/// Why a probe failed.
#[derive(Debug, Clone, PartialEq)]
pub enum ProbeError {
    /// The text does not follow the grammar (`at` = byte offset).
    Syntax {
        /// The probe text.
        path: Box<str>,
        /// Byte offset of the error.
        at: usize,
    },
    /// The path parses but the registry does not know it (`engine.probe_unknown`).
    Unknown(Box<str>),
    /// The path names an attack, view or group the scenario does not have.
    NotInScenario(Box<str>),
    /// Evaluating the scenario failed.
    Engine(crate::EngineError),
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeError::Syntax { path, at } => {
                write!(f, "probe syntax error in {path:?} at byte {at}")
            }
            ProbeError::Unknown(p) => write!(f, "unknown probe {p:?}"),
            ProbeError::NotInScenario(p) => write!(f, "probe {p:?} names nothing in the scenario"),
            ProbeError::Engine(e) => write!(f, "engine error: {e}"),
        }
    }
}

impl std::error::Error for ProbeError {}

impl From<crate::EngineError> for ProbeError {
    fn from(e: crate::EngineError) -> Self {
        ProbeError::Engine(e)
    }
}

/// One supported probe pattern of the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeDef {
    /// Pattern with `*` for slugs and indices (`"attack[*].ratio_q[*]"`).
    pub pattern: &'static str,
    /// The spec symbol (`"q_t"`).
    pub symbol: &'static str,
    /// The equation (`"E-3"`).
    pub equation: &'static str,
}

/// Every supported probe path with its symbol (engine.md §11.2). Empty until T17 fills it; a probe that matches no
/// entry is [`ProbeError::Unknown`].
pub fn registry() -> &'static [ProbeDef] {
    &[]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "T17"]
    fn parse_round_trips_the_engine_md_examples() {
        for p in [
            "enemy.pool.HP",
            "enemy.type_mod[COR]",
            "attack[normal_attack].ratio_q[COR]",
            "attack[normal_attack].p_tier[2]",
            "attack[normal_attack].view[armor].hit.k0.total",
            "attack[normal_attack].view[armor].dot_tick[SLA].k0",
            "attack[normal_attack].view[armor].dps_burst@steady",
            "attack[normal_attack].view[armor].per_pellet@stacks(COR=10,PUN=5)",
            "group[0].dps_window",
        ] {
            let parsed = ProbePath::parse(p).expect(p);
            assert_eq!(parsed.to_string(), p);
        }
    }
}
