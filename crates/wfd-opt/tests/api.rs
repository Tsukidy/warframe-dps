//! The frozen interface of `wfd-opt` (work-breakdown.md T02d; optimizer.md §1, §5, §8, §9): signatures, job control,
//! event sinks, batch contexts and errors. The acceptance tests of T32 and T33 that need the engine are listed at the
//! end, ignored with the task that implements them.

use std::sync::Arc;
use std::time::{Duration, Instant};

use wfd_data::GameData;
use wfd_engine::{Engine, Kernel};
use wfd_model::api::{OptEvent, OptRequest, OptResult, Plan};
use wfd_model::message::{ApiKey, EngineKey, Message, OptKey};
use wfd_opt::batch::BATCH_SIZE;
use wfd_opt::bnb::{KeptLeaf, NodeStats};
use wfd_opt::events::{CollectSink, JOB_UNSET, NullSink, PROGRESS_MIN_INTERVAL, stamp_job};
use wfd_opt::{
    BatchCtx, CANCEL_CHECK_EVALS, CANCEL_CHECK_NODES, CancelToken, EventSink, JobControl,
    MainProblem, MainSolution, OptError, Score, optimize, solve_main, validate,
};

/// `optimize` as a function pointer (work-breakdown.md T02d).
type OptimizeFn =
    fn(&Engine, &GameData, &OptRequest, &JobControl, &dyn EventSink) -> Result<OptResult, OptError>;

/// The exact signatures of work-breakdown.md T02d, checked by coercion to function pointers.
#[test]
fn frozen_signatures() {
    let _optimize: OptimizeFn = optimize;
    let _solve_main: fn(&Kernel, &MainProblem, &BatchCtx, &JobControl) -> MainSolution = solve_main;
    let ctx = BatchCtx {
        theta: Score::WORST,
        budget: Some(7),
    };
    let BatchCtx { theta, budget } = ctx;
    assert_eq!(theta, Score::WORST);
    assert_eq!(budget, Some(7));
    // optimizer.md §1 with `cancel` as the engine's `CancelToken` (CONTRACTS.md T02d): `JobControl { cancel,
    // deadline: Option<Instant>, node_budget: Option<u64> }`.
    let ctl = JobControl::default();
    let JobControl {
        cancel,
        deadline,
        node_budget,
    } = ctl;
    let _: &CancelToken = &cancel;
    assert!(!cancel.is_cancelled());
    assert_eq!((deadline, node_budget), (None, None));
}

/// `validate` (contract review API-19): the synchronous checks behind 422 `api.invalid` of `POST /api/optimize`.
#[test]
fn validate_signature() {
    let _validate: fn(&Engine, &GameData, &OptRequest) -> Result<(), OptError> = validate;
}

/// The finalists' Monte Carlo takes the engine's `&CancelToken` (engine.md §1 `Engine::mc`); the job's flag is that
/// token, so a clone handed to MC (or kept by the server's cancel route) sees the job's cancel, and the other way round.
#[test]
fn job_cancel_is_the_engine_cancel_token() {
    let ctl = JobControl::new(None, None);
    let for_mc: CancelToken = ctl.cancel.clone();
    assert!(!for_mc.is_cancelled());
    ctl.request_cancel();
    assert!(for_mc.is_cancelled());

    let ctl = JobControl::new(None, None);
    let route = ctl.cancel.clone();
    route.cancel();
    assert!(ctl.is_cancelled());
    assert!(ctl.should_stop(Instant::now()));
}

#[test]
fn job_control_cancel_is_seen_by_another_thread() {
    let ctl = Arc::new(JobControl::new(None, Some(1_000)));
    assert!(!ctl.is_cancelled());
    assert!(!ctl.should_stop(Instant::now()));
    let worker = {
        let ctl = Arc::clone(&ctl);
        std::thread::spawn(move || {
            let start = Instant::now();
            while !ctl.is_cancelled() {
                assert!(
                    start.elapsed() < Duration::from_secs(10),
                    "cancel never seen"
                );
                std::thread::yield_now();
            }
        })
    };
    ctl.request_cancel();
    worker.join().unwrap();
    assert!(ctl.is_cancelled());
    assert!(ctl.should_stop(Instant::now()));
    assert_eq!(ctl.node_budget, Some(1_000));
}

#[test]
fn job_control_deadline() {
    let t0 = Instant::now();
    let ctl = JobControl::new(Some(t0 + Duration::from_secs(60)), None);
    assert!(!ctl.past_deadline(t0));
    assert!(!ctl.should_stop(t0));
    assert!(ctl.past_deadline(t0 + Duration::from_secs(60)));
    assert!(ctl.should_stop(t0 + Duration::from_secs(61)));
    // No deadline never expires.
    let open = JobControl::new(None, None);
    assert!(!open.past_deadline(t0 + Duration::from_secs(1 << 30)));
}

#[test]
fn cancel_check_intervals_follow_section_9() {
    assert_eq!(CANCEL_CHECK_NODES, 1024);
    assert_eq!(CANCEL_CHECK_EVALS, 64);
    assert_eq!(PROGRESS_MIN_INTERVAL, Duration::from_millis(250));
    assert_eq!(BATCH_SIZE, 64);
}

fn log(n: u32) -> OptEvent {
    OptEvent::Log {
        message: OptKey::LogNodeBudget.message().arg("nodes", n),
    }
}

#[test]
fn sinks_receive_events_in_order() {
    let sink = CollectSink::new();
    assert!(sink.is_empty());
    let dyn_sink: &dyn EventSink = &sink;
    for n in 0..3 {
        dyn_sink.emit(log(n));
    }
    assert_eq!(sink.len(), 3);
    assert_eq!(sink.take(), vec![log(0), log(1), log(2)]);
    assert!(sink.is_empty());

    // A closure is a sink.
    let seen = std::sync::Mutex::new(Vec::new());
    let closure = |e: OptEvent| seen.lock().unwrap().push(e.name());
    let dyn_closure: &dyn EventSink = &closure;
    dyn_closure.emit(log(9));
    assert_eq!(*seen.lock().unwrap(), vec!["log"]);

    NullSink.emit(log(1));
}

#[test]
fn sinks_are_shared_across_threads() {
    let sink = Arc::new(CollectSink::new());
    let handles: Vec<_> = (0..4)
        .map(|t| {
            let sink = Arc::clone(&sink);
            std::thread::spawn(move || {
                for n in 0..25 {
                    sink.emit(log(t * 100 + n));
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(sink.len(), 100);
}

fn score(band: u8, primary: f64, tie: f64, tie2: f64) -> Score {
    Score {
        band,
        primary,
        tie,
        tie2,
    }
}

/// engine.md §7.9: lower band is better, then higher primary, tie, tie2; the greater key is the better build.
#[test]
fn score_order_used_by_the_search() {
    assert!(score(0, 1.0, 0.0, 0.0) > score(1, 1e9, 0.0, 0.0));
    assert!(score(1, -3.0, 0.0, 0.0) > score(2, 1e9, 1e9, 1e9));
    assert!(score(0, 2.0, -9.0, -9.0) > score(0, 1.0, 9.0, 9.0));
    assert!(score(0, 2.0, 1.0, -9.0) > score(0, 2.0, 0.0, 9.0));
    assert!(score(0, 2.0, 1.0, 1.0) > score(0, 2.0, 1.0, 0.0));
    let mut keys = [
        score(2, 5.0, 0.0, 0.0),
        score(0, 1.0, 0.0, 0.0),
        score(0, 3.0, 0.0, 0.0),
        score(1, -2.0, 0.0, 0.0),
    ];
    keys.sort_by(|a, b| b.cmp(a)); // (Score desc), §7 step 2
    assert_eq!(
        keys.iter().map(|k| (k.band, k.primary)).collect::<Vec<_>>(),
        vec![(0, 3.0), (0, 1.0), (1, -2.0), (2, 5.0)]
    );
}

/// A batch without a threshold prunes nothing: θ is the engine's `Score::WORST`, at most every key a kernel can
/// return (bands 0-2; §5.2 prunes `bound < θ` and keeps leaves with `s ≥ θ`).
#[test]
fn batch_without_threshold_prunes_nothing() {
    let ctx = BatchCtx::without_threshold(Some(5));
    assert_eq!(ctx.budget, Some(5));
    assert_eq!(ctx.theta, Score::WORST);
    assert_eq!(
        (Score::WORST.band, Score::WORST.primary),
        (2, f64::NEG_INFINITY)
    );
    let inf = f64::INFINITY;
    for k in [
        score(0, 0.0, 0.0, 0.0),
        score(0, -inf, -inf, -inf),
        score(1, -1e300, 0.0, 0.0),
        score(1, -inf, -inf, -inf),
        score(2, -inf, -inf, -inf),
        score(2, 0.0, -inf, -inf),
        score(2, -inf, inf, inf), // a bound sets the tie fields to +∞ (engine.md §7.9)
    ] {
        assert!(k >= ctx.theta, "{k:?}");
    }
    assert_eq!(BatchCtx::without_threshold(None).budget, None);
}

/// The frozen result types print and compare, so the implementing tasks can assert on them.
#[test]
fn search_types_are_debug_and_comparable() {
    let solution = MainSolution {
        keep: vec![KeptLeaf {
            score: score(0, 1.0, 0.0, 0.0),
            picks: vec![0, 3],
        }],
        best: Some(score(0, 1.0, 0.0, 0.0)),
        gap: None,
        stats: NodeStats::default(),
    };
    assert_eq!(solution.clone(), solution);
    assert!(format!("{solution:?}").contains("picks: [0, 3]"));
    let ctx = BatchCtx::without_threshold(None);
    assert!(format!("{ctx:?}").contains("budget: None"));
}

/// `optimize` does not know the job id: it emits `started` with `JOB_UNSET`, and the job's sink stamps the id
/// (server-api.md §8.2); other events pass unchanged.
#[test]
fn sinks_stamp_the_job_id_on_started() {
    let plan = Plan {
        positions: Vec::new(),
        composition_sets: 2154,
        excluded: Vec::new(),
    };
    let started = OptEvent::Started {
        job: JOB_UNSET.into(),
        plan: plan.clone(),
    };
    assert_eq!(
        stamp_job(started, "opt-7f3a9c"),
        OptEvent::Started {
            job: "opt-7f3a9c".into(),
            plan
        }
    );
    assert_eq!(stamp_job(log(4), "opt-7f3a9c"), log(4));
}

#[test]
fn opt_error_messages() {
    let issue = Message::new(ApiKey::NotFound)
        .arg("what", "item")
        .arg("id", 3301);
    let invalid = OptError::Invalid(vec![issue.clone()]);
    assert_eq!(invalid.message(), ApiKey::Invalid.message());
    assert_eq!(invalid.issues(), &[issue]);
    let infeasible = OptError::Infeasible(OptKey::FailedInfeasible.message());
    assert_eq!(infeasible.message().key.as_str(), "opt.failed.infeasible");
    assert!(infeasible.issues().is_empty());
    let objective = OptError::Objective(OptKey::FailedObjective.message().arg("attack", "x"));
    assert_eq!(objective.message().key.as_str(), "opt.failed.objective");
    assert!(objective.to_string().contains("opt.failed.objective"));
    let failed = OptEvent::Failed {
        error: infeasible.message(),
    };
    assert!(failed.is_terminal());
    // A cancel before any scored build has no result (API-19); the caller ends the job with `cancelled`.
    let cancelled = OptError::Cancelled;
    assert_eq!(cancelled.message(), EngineKey::Cancelled.message());
    assert!(cancelled.issues().is_empty());
    assert!(cancelled.to_string().contains("cancelled"));
    assert!(OptEvent::Cancelled { result: None }.is_terminal());
}

// ---- acceptance of the implementing tasks (work-breakdown.md T32, T33; testing.md §6) ----

#[test]
#[ignore = "T33: constraints proptests (locks, must-include, prohibited, incompatibility, capacity, element rules)"]
fn constraints_hold_on_every_ranked_build() {
    todo!("T33")
}

#[test]
#[ignore = "T33: cancellation returns a valid result within 100 ms (optimizer.md §9)"]
fn cancellation_returns_a_valid_result_quickly() {
    todo!("T33")
}

#[test]
#[ignore = "T33: identical results for 1, 2 and 8 threads with node_budget, repeated 5 times (optimizer.md §8)"]
fn results_identical_across_thread_counts() {
    todo!("T33")
}

#[test]
#[ignore = "T33: the inspiration-compatible preset is applied (optimizer.md §10.2)"]
fn inspiration_compatible_preset_applied() {
    todo!("T33")
}

#[test]
#[ignore = "T33: every RankedBuild.input, evaluated by the engine, reproduces its score (08 §10; M-10, API-9)"]
fn ranked_builds_reproduce_their_score() {
    todo!("T33")
}

#[test]
#[ignore = "T33: the request fixture tests/fixtures/opt_rifle.json runs on the mini bundle with a node budget (API-18)"]
fn rifle_request_fixture_runs() {
    todo!("T33")
}
