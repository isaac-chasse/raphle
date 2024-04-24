use axum::{extract::Query, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{Errors, GraphState};

#[derive(Serialize)]
pub struct OutgoingEdgeResponse {
    sources: Vec<u32>,
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
    if !state
        .graph
        .lock()
        .unwrap()
        .is_loaded
        .read()
        .unwrap()
        .clone()
    {
        error!("Graph data not yet loaded!");
        return Err(Errors::StillLoading);
    }

    let source = state.graph.lock().unwrap().get_node(query.source);
    if source.is_none() {
        return Ok(Json(OutgoingEdgeResponse { sources: vec![] }));
    }

    let sources = state
        .graph
        .lock()
        .unwrap()
        .get_outgoing_edges(source.unwrap())
        .iter()
        .map(|n| state.graph.lock().unwrap().get_node(n).unwrap())
        .collect();
    Ok(Json(OutgoingEdgeResponse { sources }))
}

pub struct IncomingEdgeResponse {
    targets: Vec<u32>,
}

pub struct IncomingEdgeQuery {
    target: u32,
}

pub async fn get_incoming_edges() {
    todo!()
}

#[derive(Serialize)]
pub struct HasEdgeResponse {
    has_edge: bool,
}

#[derive(Deserialize)]
pub struct HasEdgeQuery {
    source: String,
    target: String,
}

pub async fn get_has_edge(
    state: Extension<GraphState>,
    Query(query): Query<HasEdgeQuery>,
) -> Result<Json<HasEdgeResponse>, Errors> {
    // Return Error if not loaded
    if !state
        .graph
        .lock()
        .unwrap()
        .is_loaded
        .read()
        .unwrap()
        .clone()
    {
        error!("Graph data not yet loaded!");
        return Err(Errors::StillLoading);
    }

    let source = state
        .graph
        .lock()
        .unwrap()
        .get_node(query.source.parse::<u32>().unwrap());
    let target = state
        .graph
        .lock()
        .unwrap()
        .get_node(query.target.parse::<u32>().unwrap());
    if source.is_none() || target.is_none() {
        return Ok(Json(HasEdgeResponse { has_edge: false }));
    }

    let has_edge = state
        .graph
        .lock()
        .unwrap()
        .has_edge(source.unwrap(), target.unwrap());
    Ok(Json(HasEdgeResponse { has_edge }))
}
