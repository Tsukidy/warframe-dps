//! `wfd-server`: the `warframe-dps` binary's library (docs/architecture/server-api.md). One process serves the JSON
//! API and the built frontend on localhost, runs optimizer and data-refresh jobs, and opens the browser (CLAUDE.md,
//! AD-14).
//!
//! Skeleton by T02d (work-breakdown.md T02d): [`router`] answers every route of server-api.md §3 with 501
//! `api.not_implemented` until its task wires it ([`routes::ROUTES`]), and [`serve`] binds a loopback port and serves
//! the router until Ctrl-C. Ownership after the handover (work-breakdown.md §8): T11 owns the crate (CLI, startup
//! sequence, `AppState`, pools, jobs, SSE, static embedding, middleware, logging, configuration) except the route
//! modules `calc`, `timeline`, `explain`, `build`, `optimize`, `data` and `img`, which T34 owns.

#![warn(missing_docs)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::Context;

pub mod cli;
pub mod config;
pub mod error;
pub mod routes;
pub mod state;
pub mod web;

pub use config::Config;
pub use state::AppState;

/// The application router: every route of server-api.md §3 (501 until wired) and the fallbacks (`GET /*`; unknown
/// `/api` paths and method mismatches 404 `api.not_found`, [`web`]). `method_not_allowed_fallback` applies to the
/// routes registered before it, so every route is added in [`routes::api`].
pub fn router(state: Arc<AppState>) -> axum::Router {
    routes::api()
        .fallback(web::fallback)
        .method_not_allowed_fallback(web::method_not_allowed)
        .with_state(state)
}

/// Runs the server (server-api.md §1.2): binds `cfg.listen` or `127.0.0.1` on `cfg.port` (0 = OS-assigned), prints
/// the banner line with the URL, and serves [`router`] until Ctrl-C.
///
/// Skeleton: T11 adds the rest of §1.2 (configuration file, logging, data loading, IPv6 loopback, port fallback,
/// single-instance detection, browser opening, job cancellation on shutdown).
pub async fn serve(cfg: Config) -> anyhow::Result<()> {
    let ip = cfg.listen.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let listener = tokio::net::TcpListener::bind(SocketAddr::new(ip, cfg.port))
        .await
        .with_context(|| format!("binding {ip} port {}", cfg.port))?;
    let addr = listener.local_addr().context("reading the bound address")?;
    if let Some(warning) = listen_warning(cfg.listen) {
        eprintln!("{warning}");
    }
    println!("{}", banner(addr, None));
    let state = Arc::new(AppState::new(cfg));
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serving")?;
    Ok(())
}

/// The startup line of server-api.md §1.2 step 5: `warframe-dps <version> — http://127.0.0.1:<port>/ (data
/// <data_version>)`; without data (skeleton) the parenthesis says `no data loaded`.
pub fn banner(addr: SocketAddr, data_version: Option<&str>) -> String {
    let data = match data_version {
        Some(v) => format!("data {v}"),
        None => "no data loaded".to_owned(),
    };
    format!(
        "warframe-dps {} — http://{addr}/ ({data})",
        env!("CARGO_PKG_VERSION")
    )
}

/// The warning of server-api.md §1.2 step 4 for `--listen ADDR` (LAN use is opt-in): printed to stderr when the
/// address is not a loopback address; `None` for loopback or without `--listen`.
pub fn listen_warning(listen: Option<IpAddr>) -> Option<String> {
    let ip = listen.filter(|ip| !ip.is_loopback())?;
    Some(format!(
        "warning: --listen {ip}: the server is reachable from other machines on the network; \
         it has no authentication (server-api.md §12.3)"
    ))
}

/// Resolves on Ctrl-C; never, when the signal cannot be installed.
async fn shutdown_signal() {
    if tokio::signal::ctrl_c().await.is_err() {
        std::future::pending::<()>().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_names_the_url() {
        let v4 = banner("127.0.0.1:7878".parse().unwrap(), Some("2026.11.27"));
        assert_eq!(
            v4,
            format!(
                "warframe-dps {} — http://127.0.0.1:7878/ (data 2026.11.27)",
                env!("CARGO_PKG_VERSION")
            )
        );
        assert!(
            banner("[::1]:7879".parse().unwrap(), None)
                .contains("http://[::1]:7879/ (no data loaded)")
        );
    }

    #[test]
    fn listen_warns_for_non_loopback_addresses() {
        assert_eq!(listen_warning(None), None);
        assert_eq!(listen_warning(Some("127.0.0.1".parse().unwrap())), None);
        assert_eq!(listen_warning(Some("::1".parse().unwrap())), None);
        for lan in ["0.0.0.0", "192.168.1.20", "::"] {
            let w = listen_warning(Some(lan.parse().unwrap())).expect(lan);
            assert!(
                w.starts_with("warning: --listen ") && w.contains(lan),
                "{w}"
            );
        }
    }
}
