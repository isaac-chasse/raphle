use axum::{extract::Query, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{Errors, GraphState};

#[derive(Serialize)]
pub struct OutgoingEdgeResponse {
    targets: Vec<u32>,
}

#[derive(Deserialize)]
pub struct OutgoingEdgeQuery {
    source: u32,
}

pub async fn get_outgoing_edges(
    state: Extension<GraphState>,
    Query(query): Query<OutgoingEdgeQuery>,
) -> Result<Json<OutgoingEdgeResponse>, Errors> {
    // Return Error if not loaded
    if !*state.graph.lock().unwrap().is_loaded.read().unwrap() {
        error!("Graph data not yet loaded!");
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
        .map(|n| busy_graph.get_node(n).unwrap().into())
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
    let incoming = busy_graph.get_outgoing_edges(target.unwrap());
    let sources: Vec<_> = incoming
        .iter()
        .map(|n| busy_graph.get_node(n).unwrap().into())
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
