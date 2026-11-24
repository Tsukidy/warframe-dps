//! HTTP-level error keys (server-api.md §2 "Errors" and "Status codes", §12.3). The body of every non-2xx response is
//! `ErrorResponse { error: Message, issues? }` (`crate::api::error`).

use super::message_keys;

message_keys! {
    /// Keys of HTTP errors (area `api`).
    pub enum ApiKey, area = "api" {
        /// 400: the body is not valid JSON or does not match the request type (position of the error).
        BadJson = "api.bad_json" [line, column],
        /// 404: unknown key, job or bundle id (`what` = `"weapon"`, `"item"`, `"enemy"`, `"job"`, `"bundle"`, `"route"`).
        NotFound = "api.not_found" [what, id],
        /// 422: the document is malformed; the reasons are the response's `issues` (doc and import keys).
        Invalid = "api.invalid" [],
        /// 500: a handler panicked or failed; `error_id` names the log entry (server-api.md §11).
        Internal = "api.internal" [error_id],
        /// 413: the body is larger than the 1 MB limit (server-api.md §12.3).
        PayloadTooLarge = "api.payload_too_large" [limit],
        /// 421: the `Host` header is not a loopback address of this server (DNS-rebinding guard, §12.3).
        BadHost = "api.bad_host" [host],
        /// 403: a non-GET request carries a foreign `Origin` (§12.3).
        BadOrigin = "api.bad_origin" [origin],
    }
}
