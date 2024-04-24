use axum::{extract::Query, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::GraphState;

/// [`HealthStatusQuery`] is the simplest way to check on an in-memory graph.
#[derive(Deserialize)]
pub struct HealthStatusQuery {
    stats: String,
}

/// [`HealthStatus`] is a struct with simple in-memory graph stats.
#[derive(Serialize)]
pub struct HealthStatus {
    status: &'static str,
    version: &'static str, // TODO: grab this from cargo workspace
    node_count: Option<u32>,
    edge_count: Option<u32>,
    loaded: bool,
}

/// Calls to the [`GraphState`] for its [`HealthStatus`] via a [`HealthStatusQuery`]
pub async fn health(
    state: Extension<GraphState>,
    Query(query): Query<HealthStatusQuery>,
) -> impl IntoResponse {
    let mut status = HealthStatus {
        status: "ok",
        version: "0.1.0",
        node_count: None,
        edge_count: None,
        loaded: *state.graph.lock().unwrap().is_loaded.read().unwrap(),
    };

    // if stats are requested, query them from the graph
    // TODO: implement node and edge counts
    if query.stats == "true" {
        warn!("stats will return 0 until its functionality is implemented!");
        status.node_count = Some(0);
        status.edge_count = Some(0);
    }

    Json(status)
}
