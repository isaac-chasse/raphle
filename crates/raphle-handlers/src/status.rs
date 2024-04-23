use std::sync::{Arc, Mutex};

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use raphle_experimental::rwlocked_graph;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// [`std::sync::Arc`] of an instatiated in-memory graph.
#[derive(Clone)]
pub struct GraphState {
    pub graph: Arc<Mutex<rwlocked_graph::RWLockedGraph>>,
}

/// Graph-specific errors.
pub enum Errors {
    /// [`Errors::StillLoading`] occurs when requests are made to a graph that is still loading
    /// into memory.
    StillLoading,
}

impl IntoResponse for Errors {
    fn into_response(self) -> Response {
        let body = match self {
            Errors::StillLoading => "graph data is still loading to memory",
        };

        // just call another implementation of [`IntoResponse`]
        (StatusCode::SERVICE_UNAVAILABLE, body).into_response()
    }
}

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
