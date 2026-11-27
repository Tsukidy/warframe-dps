//! The route table of server-api.md §3: one module per route group (work-breakdown.md §2.2), assembled here.
//!
//! Owner: T11, except the modules `calc`, `timeline`, `explain`, `build`, `optimize`, `data` and `img`, which T34
//! owns (work-breakdown.md §8). Skeleton by T02d: every route answers 501 `api.not_implemented` with its
//! `"METHOD path"` until its task wires it ([`RouteSpec::wired_by`]).

use std::sync::Arc;

use axum::Router;

use crate::state::AppState;

pub mod build;
pub mod calc;
pub mod catalog;
pub mod data;
pub mod explain;
pub mod img;
pub mod meta;
pub mod optimize;
pub mod timeline;

/// One row of the route table of server-api.md §3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteSpec {
    /// HTTP method (`"GET"`, `"POST"`, `"DELETE"`).
    pub method: &'static str,
    /// Path template in axum syntax (`/api/weapons/{key}`); `/*` is the fallback of [`crate::web`].
    pub path: &'static str,
    /// Module serving it (`routes::<module>`, or `web`).
    pub module: &'static str,
    /// Owner of the module (work-breakdown.md §8).
    pub owner: &'static str,
    /// The task that makes the route answer (work-breakdown.md T11: routes backed by `wfd-data`; T34: the rest).
    pub wired_by: &'static str,
}

impl RouteSpec {
    /// `"METHOD path"`, the `route` argument of `api.not_implemented`.
    pub fn name(&self) -> String {
        format!("{} {}", self.method, self.path)
    }
}

const fn row(
    method: &'static str,
    path: &'static str,
    module: &'static str,
    owner: &'static str,
    wired_by: &'static str,
) -> RouteSpec {
    RouteSpec {
        method,
        path,
        module,
        owner,
        wired_by,
    }
}

/// Every route of server-api.md §3, in its order. `data status` and `img/{key}` live in T34's modules but are wired by
/// T11, which runs before T34 (CONTRACTS.md, T02d).
pub static ROUTES: &[RouteSpec] = &[
    row("GET", "/api/meta", "meta", "T11", "T11"),
    row("GET", "/api/catalog", "catalog", "T11", "T11"),
    row("GET", "/api/weapons/{key}", "catalog", "T11", "T11"),
    row("GET", "/api/items/{key}", "catalog", "T11", "T11"),
    row("GET", "/api/enemies/{key}", "catalog", "T11", "T11"),
    row("GET", "/api/schema", "catalog", "T11", "T11"),
    row("POST", "/api/candidates", "catalog", "T11", "T11"),
    row("POST", "/api/calc", "calc", "T34", "T34"),
    row("POST", "/api/timeline", "timeline", "T34", "T34"),
    row("POST", "/api/explain", "explain", "T34", "T34"),
    row("POST", "/api/build/encode", "build", "T34", "T34"),
    row("POST", "/api/build/decode", "build", "T34", "T34"),
    row(
        "POST",
        "/api/build/export-inspiration",
        "build",
        "T34",
        "T34",
    ),
    row("POST", "/api/optimize", "optimize", "T34", "T34"),
    row(
        "GET",
        "/api/optimize/{job}/events",
        "optimize",
        "T34",
        "T34",
    ),
    row("GET", "/api/optimize/{job}", "optimize", "T34", "T34"),
    row("DELETE", "/api/optimize/{job}", "optimize", "T34", "T34"),
    row("GET", "/api/data/status", "data", "T34", "T11"),
    row("POST", "/api/data/check", "data", "T34", "T34"),
    row("POST", "/api/data/refresh", "data", "T34", "T34"),
    row(
        "GET",
        "/api/data/refresh/{job}/events",
        "data",
        "T34",
        "T34",
    ),
    row("DELETE", "/api/data/refresh/{job}", "data", "T34", "T34"),
    row("POST", "/api/data/activate", "data", "T34", "T34"),
    row("GET", "/api/img/{key}", "img", "T34", "T11"),
    row("GET", "/*", "web", "T11", "T11"),
];

/// The `/api` routes, before the fallback and the state are added ([`crate::router`]).
pub fn api() -> Router<Arc<AppState>> {
    Router::new()
        .merge(meta::routes())
        .merge(catalog::routes())
        .merge(calc::routes())
        .merge(timeline::routes())
        .merge(explain::routes())
        .merge(build::routes())
        .merge(optimize::routes())
        .merge(data::routes())
        .merge(img::routes())
}
