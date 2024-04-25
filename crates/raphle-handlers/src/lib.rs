use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use raphle_experimental::rwlocked_graph;
use std::sync::{Arc, Mutex};

/// Covers all actions one can do to the graph.
pub mod action;

/// Covers the graph health checks.
pub mod status;

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
