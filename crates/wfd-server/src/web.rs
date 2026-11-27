//! Static frontend and the fallback route (server-api.md §3 `GET /*`, §10).
//!
//! Owner: T11 (skeleton by T02d). T11 embeds the frontend (`build.rs` copies `frontend/dist` or writes a placeholder
//! into `$OUT_DIR/web`, `include_dir!`), serves hashed assets with long caching and `index.html` with `no-cache`, and
//! answers every non-`/api` path that is not a file with `index.html` (SPA routing). Until then non-`/api` GET paths
//! answer 501. Unknown `/api` paths are 404 `api.not_found` with `what = "route"`.

use axum::http::{Method, Uri};

use crate::error::ApiError;

/// The route-table row this module serves (server-api.md §3).
pub const STATIC_ROUTE: &str = "GET /*";

/// Fallback of the router: unknown `/api` paths are 404; other GET paths are the embedded frontend (501 until T11).
pub async fn fallback(method: Method, uri: Uri) -> ApiError {
    let path = uri.path();
    let api = path == "/api" || path.starts_with("/api/");
    if !api && (method == Method::GET || method == Method::HEAD) {
        ApiError::not_implemented(STATIC_ROUTE)
    } else {
        ApiError::not_found("route", path)
    }
}
