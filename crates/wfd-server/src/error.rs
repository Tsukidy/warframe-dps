//! Error responses (server-api.md §2): every non-2xx response has the body `ErrorResponse { error: Message, issues? }`
//! with an i18n key, never English text (I-8). JSON is written as `application/json; charset=utf-8`.
//!
//! Owner: T11 (skeleton by T02d).

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use wfd_model::MsgArg;
use wfd_model::api::ErrorResponse;
use wfd_model::message::{ApiKey, Message};

/// Content type of every JSON body (server-api.md §2 "Format").
pub const JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";

/// A non-2xx response: status and error body.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    /// HTTP status.
    pub status: StatusCode,
    /// The body.
    pub body: ErrorResponse,
}

impl ApiError {
    /// An error without issues.
    pub fn new(status: StatusCode, error: Message) -> Self {
        ApiError {
            status,
            body: ErrorResponse::new(error),
        }
    }

    /// 501 `api.not_implemented` for a route of server-api.md §3 that is not wired yet (`route` = `"POST /api/calc"`).
    pub fn not_implemented(route: &str) -> Self {
        ApiError::new(
            StatusCode::NOT_IMPLEMENTED,
            ApiKey::NotImplemented.message().arg("route", route),
        )
    }

    /// 404 `api.not_found` (`what` = `"weapon"`, `"item"`, `"enemy"`, `"job"`, `"bundle"`, `"route"`).
    pub fn not_found(what: &str, id: impl Into<MsgArg>) -> Self {
        ApiError::new(
            StatusCode::NOT_FOUND,
            ApiKey::NotFound.message().arg("what", what).arg("id", id),
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        json_response(self.status, &self.body)
    }
}

/// A JSON response with the content type of server-api.md §2. A value that cannot be serialized (a non-finite f64
/// outside an `ext_f64` field) becomes 500 `api.internal`.
pub fn json_response<T: Serialize>(status: StatusCode, value: &T) -> Response {
    match serde_json::to_vec(value) {
        Ok(bytes) => (status, [(header::CONTENT_TYPE, JSON_CONTENT_TYPE)], bytes).into_response(),
        Err(_) => {
            let body = ErrorResponse::new(ApiKey::Internal.message().arg("error_id", "serialize"));
            let bytes = serde_json::to_vec(&body).unwrap_or_default();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, JSON_CONTENT_TYPE)],
                bytes,
            )
                .into_response()
        }
    }
}
