use axum::{routing::get, Extension, Router};
use axum_prometheus::{
    metrics_exporter_prometheus::{Matcher, PrometheusBuilder},
    PrometheusMetricLayer, AXUM_HTTP_REQUESTS_DURATION_SECONDS,
};
use dotenvy::dotenv;
use metrics_process::Collector;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

// use raphle_graph::graph;

use raphle_experimental::rwlocked_graph;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let project_path = std::env::var("PROJECT_PATH").expect("Expected a project path env var");

    // initialize tracing subscriber
    let subscriber = tracing_subscriber::fmt().compact().finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // set up port
    let port = std::env::var("PORT").unwrap_or("8000".to_string());

    // grab benchmark csv
    // SET UP DATA CONNECTION HERE
    // - Just uses hard-path to benchmark TSV
    // - Abstract to CLI connection? or offer Env path or S3?
    let mut csv_path =
        std::env::var("BENCHMARK_PATH").expect("Expected benchmark dataset in env var");
    csv_path = format!("{}{}", project_path, csv_path);
    let expected_node_count = std::env::var("EXPECTED_NODE_COUNT")
        .unwrap_or("1000000".to_string())
        .parse::<u32>()
        .unwrap();

    info!(
        "Memgraph started on port {} with node capacity of {}",
        port, expected_node_count
    );
    info!("Starting up!");

    let graph = rwlocked_graph::RWLockedGraph::new(expected_node_count);
    let graph = Arc::new(Mutex::new(graph));

    let graph_clone = graph.clone();
    let csv_path_clone = csv_path.clone();

    tokio::spawn(async move {
        match graph_clone.lock().unwrap().load_from_tsv(&csv_path_clone) {
            Ok(_) => info!("Loaded graph from CSV"),
            Err(e) => warn!("Failed to load graph from CSV: {}", e),
        }
    })
    .await
    .expect("Failed to spawn task");

    let state = raphle_handlers::status::GraphState { graph };

    let collector = Collector::default();
    collector.describe();

    let metric_layer = PrometheusMetricLayer::new();
    // this is the default if you use [`PrometheusMetricLayer:pair`]
    let metric_handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
            &[
                0.00001, 0.00002, 0.00005, 0.0001, 0.0002, 0.0005, 0.001, 0.002, 0.005, 0.01, 0.02,
                0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0,
            ],
        )
        .unwrap()
        .install_recorder()
        .unwrap();

    let server = Router::new()
        .route("/health", get(raphle_handlers::status::health))
        .layer(Extension(state))
        .route(
            "/metrics",
            get(|| async move {
                collector.collect();
                metric_handle.render()
            }),
        )
        .layer(metric_layer);

    println!("Listening on port: {}", port);

    let listen_address = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(listen_address).await.unwrap();
    axum::serve(listener, server).await.unwrap();
}
