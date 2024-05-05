use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{Errors, GraphState};

#[derive(Deserialize)]
pub struct EdgeBody {
    new_edges: Vec<Edge>,
}

#[derive(Deserialize)]
pub struct Edge {
    source: u32,
    target: u32,
}

/// Sends a [`Vec<Edge>`] to the [`GraphState`]. Will enqueque new edges to the [`GraphState`]
/// if the graph is not fully loaded. Used to post many new edges to the graph.
pub async fn post_edges(
    state: Extension<GraphState>,
    Json(body): Json<EdgeBody>,
) -> impl IntoResponse {
    for new_edge in body.new_edges {
        // If the graph isn't loaded yet, enqueque the follow requests
        if !*state
            .graph
            .lock()
            .unwrap()
            .is_loaded
            .read()
            .unwrap()
        {
            state
                .graph
                .lock()
                .unwrap()
                .enqueue_add_edge(new_edge.source, new_edge.target);
            warn!("graph not fully loaded, added edge to queue");
            continue;
        }

        state
            .graph
            .lock()
            .unwrap()
            .add_edge(new_edge.source, new_edge.target)
    }
    info!("successfully loaded edges");
    StatusCode::OK
}

/// Sends a new [`Edge`] to the [`GraphState`]. Will enqueue the edge to the [`GraphState`] if
/// the graph is not fully loaded. Used to add new, single edges to the graph.
pub async fn post_edge(
    state: Extension<GraphState>,
    body: Json<Edge>,
) -> impl IntoResponse {
    // If the graph isn't loaded yet, enqueque the follow request
    if !*state
        .graph
        .lock()
        .unwrap()
        .is_loaded
        .read()
        .unwrap()
    {
        state
            .graph
            .lock()
            .unwrap()
            .enqueue_add_edge(body.source, body.target);
        warn!("graph not loaded, added edge to queue");
        return StatusCode::OK;
    }

    state
        .graph
        .lock()
        .unwrap()
        .add_edge(body.source, body.target);
    info!("successfully added edge");
    StatusCode::OK
}

#[derive(Serialize)]
pub struct OutgoingEdgeResponse {
    targets: Vec<u32>,
}

#[derive(Deserialize)]
pub struct OutgoingEdgeQuery {
    source: u32,
}

/// Requests the outgoing edges from a given source node-ID (of type `u32`). Returns a [`Vec<u32>`]
/// which is the set of target node-IDs, or returns [`Errors::StillLoading`] when the graph is
/// still loading.
pub async fn get_outgoing_edges(
    state: Extension<GraphState>,
    Query(query): Query<OutgoingEdgeQuery>,
) -> Result<Json<OutgoingEdgeResponse>, Errors> {
    // Return Error if not loaded
    if !*state.graph.lock().unwrap().is_loaded.read().unwrap() {
        error!("graph data not yet loaded!");
        return Err(Errors::StillLoading);
    }

    let source = state.graph.lock().unwrap().get_node(query.source);
    if source.is_none() {
        warn!("source not present");
        return Ok(Json(OutgoingEdgeResponse { targets: vec![] }));
    }

    // This feels very hacky.
    // I think there is a better way to ensure we don't spam lock the graph
    // I also think we may need to free the busy_graph?
    let busy_graph = state.graph.lock().unwrap();
    let outgoing = busy_graph.get_outgoing_edges(source.unwrap());
    let targets: Vec<_> = outgoing
        .iter()
        .map(|n| busy_graph.get_node(n).unwrap())
        .collect();

    Ok(Json(OutgoingEdgeResponse { targets }))
}

#[derive(Serialize)]
pub struct IncomingEdgeResponse {
    sources: Vec<u32>,
}

#[derive(Deserialize)]
pub struct IncomingEdgeQuery {
    target: u32,
}

/// Requests the incoming edges from a given target node-ID (of type `u32`). Returns a [`Vec<u32>`]
/// which is the set of source node-IDs, or returns [`Errors::StillLoading`] when the graph is
/// still loading.
pub async fn get_incoming_edges(
    state: Extension<GraphState>,
    Query(query): Query<IncomingEdgeQuery>,
) -> Result<Json<IncomingEdgeResponse>, Errors> {
    // Return Error if not loaded
    if !*state.graph.lock().unwrap().is_loaded.read().unwrap() {
        error!("Graph data not yet loaded!");
        return Err(Errors::StillLoading);
    }

    let target = state.graph.lock().unwrap().get_node(query.target);
    if target.is_none() {
        warn!("source not present");
        return Ok(Json(IncomingEdgeResponse { sources: vec![] }));
    }

    // This feels very hacky.
    // I think there is a better way to ensure we don't spam lock the graph
    // I also think we may need to free the busy_graph?
    let busy_graph = state.graph.lock().unwrap();
    let incoming = busy_graph.get_incoming_edges(target.unwrap());
    let sources: Vec<_> = incoming
        .iter()
        .map(|n| busy_graph.get_node(n).unwrap())
        .collect();

    Ok(Json(IncomingEdgeResponse { sources }))
}

#[derive(Serialize)]
pub struct HasEdgeResponse {
    has_edge: bool,
}

#[derive(Deserialize)]
pub struct HasEdgeQuery {
    source: u32,
    target: u32,
}

/// Queries the [`GraphState`] for a given edges, which is a source-target pair. Returns a `bool`
/// or [`Errors::StillLoading`] if the graph if not fully loaded.
pub async fn get_has_edge(
    state: Extension<GraphState>,
    Query(query): Query<HasEdgeQuery>,
) -> Result<Json<HasEdgeResponse>, Errors> {
    // Return Error if not loaded
    if !*state.graph.lock().unwrap().is_loaded.read().unwrap() {
        error!("Graph data not yet loaded!");
        return Err(Errors::StillLoading);
    }

    let source = state.graph.lock().unwrap().get_node(query.source);
    let target = state.graph.lock().unwrap().get_node(query.target);
    if source.is_none() || target.is_none() {
        return Ok(Json(HasEdgeResponse { has_edge: false }));
    }

    let has_edge = state
        .graph
        .lock()
        .unwrap()
        .has_edge(source.unwrap(), target.unwrap());
    info!("{}", has_edge);
    Ok(Json(HasEdgeResponse { has_edge }))
}

// #[derive(Deserialize)]
// pub struct RemoveEdge {
//     todo!(),
// }
