//! Shared server state (server-api.md §11).
//!
//! Owner: T11 (skeleton by T02d). The skeleton holds the configuration only, so the route table can be tested before
//! the data layer exists. T11 adds the fields of server-api.md §11: `data: ArcSwap<GameData>` (the active bundle;
//! jobs and requests keep their snapshot across a refresh), `interactive` and `optimizer` thread pools (AD-15), `jobs`
//! (optimizer and data-refresh jobs with their control, event ring buffer and result) and `http` (images and data
//! refresh only).

use crate::config::Config;

/// State shared by every handler (`axum::extract::State<Arc<AppState>>`).
#[derive(Debug)]
pub struct AppState {
    /// The configuration the server was started with.
    pub config: Config,
}

impl AppState {
    /// State for a configuration.
    pub fn new(config: Config) -> Self {
        AppState { config }
    }
}
