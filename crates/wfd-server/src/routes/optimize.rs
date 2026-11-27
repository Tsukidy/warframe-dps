//! Optimizer routes and their SSE stream (server-api.md §8): one job at a time on the optimizer pool, running
//! `wfd_opt::optimize` with a `JobControl` the `DELETE` route cancels; events go through the job's ring buffer (last
//! 1,000) to SSE subscribers, with `Last-Event-ID` replay, 15 s keep-alive and the stream closed after the terminal
//! event (`OptEvent::to_sse`). A second job is 409 `opt.job_running`; finished jobs are kept for one hour.
//!
//! Owner: T34 (skeleton by T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/optimize", post(start))
        .route("/api/optimize/{job}/events", get(events))
        .route("/api/optimize/{job}", get(status).delete(cancel))
}

/// `POST /api/optimize`: `OptRequest` → 202 `{ job }` · 409 `opt.job_running` · 422 `api.invalid` when
/// `wfd_opt::validate` rejects the request (§8.1; contract review API-19).
pub async fn start() -> ApiError {
    ApiError::not_implemented("POST /api/optimize")
}

/// `GET /api/optimize/{job}/events`: `text/event-stream` of `OptEvent` (§8.2).
pub async fn events() -> ApiError {
    ApiError::not_implemented("GET /api/optimize/{job}/events")
}

/// `GET /api/optimize/{job}`: `JobStatus<OptResult>` (§8.3).
pub async fn status() -> ApiError {
    ApiError::not_implemented("GET /api/optimize/{job}")
}

/// `DELETE /api/optimize/{job}`: 202; the job ends with `cancelled` carrying its anytime result (§8.3).
pub async fn cancel() -> ApiError {
    ApiError::not_implemented("DELETE /api/optimize/{job}")
}
