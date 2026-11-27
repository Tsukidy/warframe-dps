//! The route table (server-api.md §3) and the skeleton's answers (work-breakdown.md T02d): every route exists and
//! answers 501 `api.not_implemented` with its `"METHOD path"` until wired; unknown `/api` paths are 404 `api.not_found`.
//! T11 and T34 replace the 501 expectations route by route as they wire them ([`RouteSpec::wired_by`]).

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;
use wfd_model::MsgArg;
use wfd_model::api::ErrorResponse;
use wfd_server::error::JSON_CONTENT_TYPE;
use wfd_server::routes::{ROUTES, RouteSpec};
use wfd_server::{AppState, Config, router};

/// The `(method, path)` rows of the route table of server-api.md §3, in document order.
fn documented_routes() -> Vec<(String, String)> {
    let doc = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/architecture/server-api.md");
    let text = std::fs::read_to_string(&doc).expect("server-api.md");
    let section = text
        .split("\n## 3. Route table\n")
        .nth(1)
        .and_then(|s| s.split("\n## ").next())
        .expect("section 3");
    let mut rows = Vec::new();
    for line in section.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // | Method | Path | Request | Response |  → ["", method, path, request, response, ""]
        if cells.len() < 4 || !matches!(cells[1], "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
            continue;
        }
        let path = cells[2].trim_matches('`');
        rows.push((cells[1].to_owned(), path.to_owned()));
    }
    rows
}

#[test]
fn route_table_matches_server_api_section_3() {
    let documented = documented_routes();
    assert!(documented.len() >= 25, "parsed {} rows", documented.len());
    let ours: Vec<(String, String)> = ROUTES
        .iter()
        .map(|r| (r.method.to_owned(), r.path.to_owned()))
        .collect();
    assert_eq!(ours, documented);
}

/// work-breakdown.md §8: T34 owns `routes/{calc,timeline,explain,build,optimize,data,img}.rs`, T11 the rest.
#[test]
fn route_modules_have_their_section_8_owners() {
    const T34: &[&str] = &[
        "calc", "timeline", "explain", "build", "optimize", "data", "img",
    ];
    for r in ROUTES {
        let owner = if T34.contains(&r.module) {
            "T34"
        } else {
            "T11"
        };
        assert_eq!(r.owner, owner, "{}", r.name());
        assert!(matches!(r.wired_by, "T11" | "T34"), "{}", r.name());
    }
    let unique: BTreeSet<_> = ROUTES.iter().map(|r| (r.method, r.path)).collect();
    assert_eq!(unique.len(), ROUTES.len(), "duplicate route rows");
}

fn app() -> axum::Router {
    router(Arc::new(AppState::new(Config::default())))
}

/// A concrete URI for a path template.
fn concrete(path: &str) -> String {
    match path {
        "/*" => "/".to_owned(),
        p => p.replace("{key}", "1204").replace("{job}", "opt-7f3a9c"),
    }
}

async fn call(method: &str, uri: &str) -> (StatusCode, Option<String>, ErrorResponse) {
    let request = Request::builder()
        .method(Method::from_bytes(method.as_bytes()).unwrap())
        .uri(uri)
        .header(header::HOST, "127.0.0.1:7878")
        .body(Body::empty())
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_owned());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: ErrorResponse = serde_json::from_slice(&bytes).unwrap_or_else(|e| {
        panic!("{method} {uri}: body is not an ErrorResponse ({e}): {bytes:?}")
    });
    (status, content_type, body)
}

fn route_arg(body: &ErrorResponse) -> Option<&str> {
    match body.error.args.get("route") {
        Some(MsgArg::Str(s)) => Some(s),
        _ => None,
    }
}

#[tokio::test]
async fn every_route_answers_501_until_wired() {
    for r in ROUTES {
        let (status, content_type, body) = call(r.method, &concrete(r.path)).await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{}", r.name());
        assert_eq!(
            content_type.as_deref(),
            Some(JSON_CONTENT_TYPE),
            "{}",
            r.name()
        );
        assert_eq!(
            body.error.key.as_str(),
            "api.not_implemented",
            "{}",
            r.name()
        );
        assert_eq!(route_arg(&body), Some(r.name().as_str()));
        assert!(body.issues.is_empty());
    }
}

#[tokio::test]
async fn spa_paths_fall_back_to_the_frontend_route() {
    let web = ROUTES.iter().find(|r| r.path == "/*").map(RouteSpec::name);
    for uri in [
        "/",
        "/optimizer",
        "/data",
        "/about",
        "/assets/index-1a2b3c.js",
    ] {
        let (status, _, body) = call("GET", uri).await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{uri}");
        assert_eq!(route_arg(&body), web.as_deref(), "{uri}");
    }
}

#[tokio::test]
async fn unknown_api_paths_are_not_found() {
    for (method, uri) in [
        ("GET", "/api"),
        ("GET", "/api/nope"),
        ("POST", "/api/weapons"),
        ("DELETE", "/api/meta/x"),
        ("POST", "/not-api"),
    ] {
        let (status, _, body) = call(method, uri).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {uri}");
        assert_eq!(body.error.key.as_str(), "api.not_found");
        assert_eq!(
            body.error.args.get("what"),
            Some(&MsgArg::Str("route".into()))
        );
    }
}
