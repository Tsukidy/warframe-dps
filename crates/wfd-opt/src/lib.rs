//! `wfd-opt`: the build optimizer (docs/architecture/optimizer.md). It searches the build space of one weapon under a
//! fixed scenario for the builds with the best objective score, on all cores, with progress events, cancellation and an
//! honest statement of how good the search was. It does no I/O and holds no global state; the server runs it as a job
//! (server-api.md §8).
//!
//! Skeleton by T02d (work-breakdown.md T02d). The frozen interface is [`optimize`], [`solve_main`], [`BatchCtx`],
//! [`JobControl`] and [`EventSink`] (optimizer.md §1, §5, §8); every body that needs the engine is `todo!` naming its
//! task, and the modules are handed over per work-breakdown.md §8:
//!
//! | Module | Contents | Owner |
//! |---|---|---|
//! | [`problem`] | problem compilation (§2): scenario, positions, filters, L1 compile, relevance; [`MainProblem`] | T32 |
//! | [`pool`] | positions and per-position pools through `wfd-data::legality` (§2 step 2) | T32 |
//! | [`dominance`] | dominance inside a pool (§2 step 7) | T32 |
//! | [`composition`] | composition sets and `ElementSolver` outcomes (§3, §5.1) | T32 |
//! | [`bnb`] | branch-and-bound over the monotone remainder: [`solve_main`], [`MainSolution`] (§5.2) | T32 |
//! | [`capacity`] | capacity, polarity and Forma pruning and leaf checks (§5.2, spec 09 §6) | T32 |
//! | [`keys`] | canonical build keys and the f64 memo cache (§8) | T32 |
//! | [`batch`] | deterministic batches, barrier thresholds, per-item budgets: [`BatchCtx`] (§8) | T32 |
//! | `lib.rs` | [`optimize`], [`JobControl`], [`OptError`] (§1, §9) | T33 |
//! | [`context`] | context groups, block-coordinate ascent with a context beam (§6.1, §6.2) | T33 |
//! | [`local`] | cross-group local search and the per-mix pass (§6.2 steps 4-5) | T33 |
//! | [`portfolio`] | buff portfolio (§6.3) | T33 |
//! | [`riven`] | riven search (§6.2 step 6) | T33 |
//! | [`finalists`] | finalists, MC confirmation, diagnostics (§7) | T33 |
//! | [`events`] | [`EventSink`] and progress throttling (§9) | T33 |
//! | [`presets`] | search presets, `Preset::InspirationCompatible` (§10.2, §11) | T33 |
//!
//! Scores are the engine's ordered [`Score`] keys (engine.md §7.9): the greater key is the better build, ordered by
//! band (lower is better), then `primary`, `tie`, `tie2` (spec 08 E-8.26). Results are bit-identical for any thread
//! count under a node budget (§8, I-5 for the optimizer).

#![warn(missing_docs)]

use std::fmt;
use std::time::Instant;

use wfd_data::GameData;
use wfd_engine::Engine;
use wfd_model::api::{OptRequest, OptResult};
use wfd_model::message::{ApiKey, EngineKey, Message};

pub use wfd_engine::{CancelToken, Score};

pub mod batch;
pub mod bnb;
pub mod capacity;
pub mod composition;
pub mod context;
pub mod dominance;
pub mod events;
pub mod finalists;
pub mod keys;
pub mod local;
pub mod pool;
pub mod portfolio;
pub mod presets;
pub mod problem;
pub mod riven;

pub use batch::BatchCtx;
pub use bnb::{MainSolution, solve_main};
pub use events::EventSink;
pub use problem::MainProblem;

/// Runs one optimizer job (optimizer.md §1, §6-§11): compiles the problem, searches composition sets, main slots and
/// context groups, re-scores the finalists with `WindowExact`, confirms them with Monte Carlo, and returns the ranked
/// builds with the proof statement and diagnostics. Events go to `sink` in order (§9: `started`, `phase`, `progress`,
/// `incumbent`, `finalist`, `log`); the terminal event (`done`, `cancelled`, `failed`) is the caller's, which knows
/// whether `ctl` was cancelled.
///
/// The job id belongs to the caller, not to the optimizer: `optimize` emits `OptEvent::Started` first, with `job`
/// left empty ([`events::JOB_UNSET`]), and the sink, which belongs to one job, fills it in
/// ([`events::stamp_job`]; the server's sink with its job id, server-api.md §8.2).
///
/// The request is resolved through `req.input.x`, which maps every session-local key of the whole request: the lists
/// outside `input` (`constraints.must_include`, `constraints.prohibited`, `buffs.allowed`, `groups.shards`) included
/// (AD-18; contract review API-17). Every [`RankedBuild`](wfd_model::api::RankedBuild) carries the scenario it was
/// scored in: the input scenario with the search's own options applied (`objective.window_s` as T-01,
/// `objective.heavy` as T-09, `buffs.strength` as T-20, its buff-portfolio row's buffs, T-03 `body` under the rule
/// `use_weak_point` off, the `inspiration_compatible` preset's window and T-02 `all_max`; `toggles.capacity` stays the
/// input's), so `/api/calc` on its `input` reproduces its `score` (08 §10; contract review M-10, API-9). `optimize`
/// fills `RankedBuild.code` itself with `wfd_buildcode::encode` of that `input`, made by `GameData::to_doc` so it
/// carries `x` for every local key (AD-18; contract review API-7, API-21).
///
/// Cancellation and the deadline of `ctl` end the search early with a valid anytime result (`Ok` with
/// `Proof::Gap` or `Proof::Heuristic`; cancel never loses the best build found, §9). `optimize` scores the input
/// build right after compiling the scenario (the first incumbent, §5.2 "Ordering"), so every later cancel has a
/// result; a cancel before that returns [`OptError::Cancelled`], which the caller ends with `cancelled` without a
/// result (`OptEvent::Cancelled { result: None }`; contract review API-19). The finalists' Monte Carlo confirmation
/// (§7 step 3) passes `ctl.cancel` to `Engine::mc`, so a cancel also stops MC; `Err(EngineError::Cancelled)` from it
/// keeps the EV-ranked finalists (C-35). With `ctl.node_budget` the result is bit-identical for any thread count (§8).
///
/// Owner: T33.
pub fn optimize(
    engine: &Engine,
    data: &GameData,
    req: &OptRequest,
    ctl: &JobControl,
    sink: &dyn EventSink,
) -> Result<OptResult, OptError> {
    let _ = (engine, data, req, ctl, sink);
    todo!("T33: optimizer.md §1, §6-§11")
}

/// Checks a request before a job starts, without searching: resolves `req.input` with its `x` map and the key lists
/// outside it (`GameData::resolve`, AD-18), checks the objective (08 §10) and the request's ranges. Every problem
/// found is one issue of [`OptError::Invalid`]: the `doc.*` issues of resolution and the `opt.failed.objective`,
/// `opt.failed.mixed_metric` and `opt.failed.ttk_group_sum` messages. The server calls it in `POST /api/optimize` and
/// answers 422 `api.invalid` with those issues (server-api.md §8.1); [`optimize`] runs the same checks first, so a job
/// started without it ends with `failed`. Addition of the contract review (API-19).
///
/// Owner: T33.
pub fn validate(engine: &Engine, data: &GameData, req: &OptRequest) -> Result<(), OptError> {
    let _ = (engine, data, req);
    todo!("T33: request checks before a job (server-api.md §8.1)")
}

/// Workers check [`JobControl`] every this many DFS nodes (optimizer.md §9: ≤ 100 ms cancel latency).
pub const CANCEL_CHECK_NODES: u64 = 1024;

/// Workers check [`JobControl`] every this many evaluations outside the DFS (optimizer.md §9).
pub const CANCEL_CHECK_EVALS: u64 = 64;

/// Control of one job (optimizer.md §1): the cancel flag the server sets on `DELETE /api/optimize/{job}`
/// (server-api.md §8.3), the wall-clock deadline (`SearchParams.time_limit_s`; not reproducible) and the deterministic
/// node budget (`SearchParams.node_budget`, divided over work items per §8).
///
/// The server keeps it in an `Arc` shared by the job thread and the route that cancels it (or keeps a clone of
/// `cancel`, which shares the flag).
#[derive(Debug, Default)]
pub struct JobControl {
    /// Set to stop the search; the job ends with its anytime result. The engine's shared flag (engine.md §1), so the
    /// finalists' Monte Carlo (`Engine::mc(.., cancel: &CancelToken)`, §7 step 3) observes the same cancel as the
    /// search (§9). optimizer.md §1 writes `AtomicBool`; CONTRACTS.md T02d.
    pub cancel: CancelToken,
    /// Wall-clock end of the search; `None` = no time limit.
    pub deadline: Option<Instant>,
    /// Deterministic node budget of the whole job; `None` = unbounded.
    pub node_budget: Option<u64>,
}

impl JobControl {
    /// A control that is not cancelled.
    pub fn new(deadline: Option<Instant>, node_budget: Option<u64>) -> Self {
        JobControl {
            cancel: CancelToken::new(),
            deadline,
            node_budget,
        }
    }

    /// Requests cancellation (`DELETE /api/optimize/{job}`); workers and Monte Carlo runs observe it at their next
    /// check.
    pub fn request_cancel(&self) {
        self.cancel.cancel();
    }

    /// Whether cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// Whether the deadline has passed at `now` (never without a deadline).
    pub fn past_deadline(&self, now: Instant) -> bool {
        self.deadline.is_some_and(|d| now >= d)
    }

    /// Whether a worker must stop at `now`: cancelled or past the deadline (the node budget is per work item and
    /// checked by the item itself, §8).
    pub fn should_stop(&self, now: Instant) -> bool {
        self.is_cancelled() || self.past_deadline(now)
    }
}

/// Why a job could not produce a result. Cancellation after the first scored build is not an error: it ends the job
/// with `Ok` and an anytime result (§9); only a cancel before any build was scored is [`OptError::Cancelled`]. The
/// server maps an error to `OptEvent::Failed { error: self.message() }` (`Cancelled` to `OptEvent::Cancelled { result:
/// None }`), or, for [`OptError::Invalid`] found by [`validate`] before the job starts, to 422 `api.invalid` with the
/// issues (server-api.md §2, §8.1). Non-exhaustive: T33 may add failure kinds; callers use [`OptError::message`] and
/// [`OptError::issues`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum OptError {
    /// The request does not resolve against the data (`GameData::resolve` issues, `doc.*` keys) or a request field is
    /// out of range.
    Invalid(Vec<Message>),
    /// The objective cannot be scored (`opt.failed.objective`, `opt.failed.mixed_metric`,
    /// `opt.failed.ttk_group_sum`; spec 08 §10).
    Objective(Message),
    /// No build satisfies the constraints (`opt.failed.infeasible`).
    Infeasible(Message),
    /// The engine failed (engine.md §15 `EngineError`, carried as its `{ key, args }` message).
    Engine(Message),
    /// Cancelled before any build was scored, so there is no anytime result (`OptEvent::Cancelled { result: None }`).
    /// Addition of the contract review (API-19).
    Cancelled,
}

impl OptError {
    /// The message of `OptEvent::Failed`: the variant's message, `api.invalid` for [`OptError::Invalid`] (its issues
    /// travel separately), `engine.cancelled` for [`OptError::Cancelled`] (for a log line only: its terminal event is
    /// `cancelled`, not `failed`).
    pub fn message(&self) -> Message {
        match self {
            OptError::Invalid(_) => ApiKey::Invalid.message(),
            OptError::Objective(m) | OptError::Infeasible(m) | OptError::Engine(m) => m.clone(),
            OptError::Cancelled => EngineKey::Cancelled.message(),
        }
    }

    /// The document issues of [`OptError::Invalid`]; empty otherwise.
    pub fn issues(&self) -> &[Message] {
        match self {
            OptError::Invalid(issues) => issues,
            _ => &[],
        }
    }
}

impl fmt::Display for OptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptError::Invalid(issues) => {
                write!(f, "invalid optimizer request ({} issue(s))", issues.len())
            }
            OptError::Objective(m) => write!(f, "objective cannot be scored: {}", m.key.as_str()),
            OptError::Infeasible(m) => {
                write!(f, "no build satisfies the constraints: {}", m.key.as_str())
            }
            OptError::Engine(m) => write!(f, "engine error: {}", m.key.as_str()),
            OptError::Cancelled => f.write_str("cancelled before any build was scored"),
        }
    }
}

impl std::error::Error for OptError {}
