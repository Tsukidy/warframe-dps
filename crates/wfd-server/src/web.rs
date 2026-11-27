//! Static frontend and the fallbacks of the router (server-api.md §3 `GET /*`, §10).
//!
//! Owner: T11 (skeleton by T02d). T11 embeds the frontend (`build.rs` copies `frontend/dist` or writes a placeholder
//! into `$OUT_DIR/web`, `include_dir!`), serves hashed assets with long caching and `index.html` with `no-cache`, and
//! answers every non-`/api` path that is not a file with `index.html` (SPA routing). Until then non-`/api` GET paths
//! answer 501. A request that matches no route of §3 (an unknown `/api` path, a known path with a method the route
//! table does not list, or a non-GET request outside `/api`) is 404 `api.not_found` with `what = "route"` and
//! `id = "METHOD path"`, as JSON like every other error (§2; 405 is not one of the status codes of §2). For a method
//! mismatch axum still sets the `Allow` header.

use axum::http::{Method, Uri};

use crate::error::ApiError;

/// The route-table row this module serves (server-api.md §3).
pub const STATIC_ROUTE: &str = "GET /*";

/// Fallback of the router for paths no route matches: unknown `/api` paths and non-GET requests are 404; other GET
/// paths are the embedded frontend (501 until T11).
pub async fn fallback(method: Method, uri: Uri) -> ApiError {
    let path = uri.path();
    let api = path == "/api" || path.starts_with("/api/");
    if !api && (method == Method::GET || method == Method::HEAD) {
        ApiError::not_implemented(STATIC_ROUTE)
    } else {
        unknown_route(&method, path)
    }
}

/// Fallback for a path of the route table requested with a method its row does not list (`GET /api/calc`,
/// `PUT /api/meta`): 404 `api.not_found`, `what = "route"`.
pub async fn method_not_allowed(method: Method, uri: Uri) -> ApiError {
    unknown_route(&method, uri.path())
}

/// 404 `api.not_found` with `what = "route"` and `id = "METHOD path"` (the form of `api.not_implemented`'s `route`).
pub fn unknown_route(method: &Method, path: &str) -> ApiError {
    ApiError::not_found("route", format!("{method} {path}"))
}
