//! Catalogue, detail, schema and candidate routes (server-api.md §5.2-§5.7), backed by `wfd-data` (`GameData::catalog`,
//! `weapon_detail`, `item_detail`, `enemy_detail`, `schema`, `candidates`). Catalogue and detail routes send
//! `ETag: "<data_version>"` and honour `If-None-Match` (§2 "Caching"); an unknown key is 404 `api.not_found`.
//!
//! Owner: T11 (skeleton by T02d).

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::error::ApiError;
use crate::state::AppState;

/// The routes of this module.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/catalog", get(catalog))
        .route("/api/weapons/{key}", get(weapon))
        .route("/api/items/{key}", get(item))
        .route("/api/enemies/{key}", get(enemy))
        .route("/api/schema", get(schema))
        .route("/api/candidates", post(candidates))
}

/// `GET /api/catalog` → `Catalog` (§5.2).
pub async fn catalog() -> ApiError {
    ApiError::not_implemented("GET /api/catalog")
}

/// `GET /api/weapons/{key}` → `WeaponDetail` (§5.3).
pub async fn weapon() -> ApiError {
    ApiError::not_implemented("GET /api/weapons/{key}")
}

/// `GET /api/items/{key}` → `ItemDetail` (§5.4).
pub async fn item() -> ApiError {
    ApiError::not_implemented("GET /api/items/{key}")
}

/// `GET /api/enemies/{key}` → `EnemyDetail` (§5.5).
pub async fn enemy() -> ApiError {
    ApiError::not_implemented("GET /api/enemies/{key}")
}

/// `GET /api/schema` → `ScenarioSchema` (§5.6).
pub async fn schema() -> ApiError {
    ApiError::not_implemented("GET /api/schema")
}

/// `POST /api/candidates`: `CandidatesRequest` → `Candidate[]` (§5.7; legality is `wfd-data::legality`).
pub async fn candidates() -> ApiError {
    ApiError::not_implemented("POST /api/candidates")
}
