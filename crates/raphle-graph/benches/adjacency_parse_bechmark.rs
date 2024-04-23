use criterion::{criterion_group, criterion_main, Criterion};
use dotenvy::dotenv;
use raphle_experimental::rwlocked_graph;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::{info, warn};

fn adjacency_parse_benchmark(c: &mut Criterion) {
    dotenv().ok();

    let project_path = std::env::var("PROJECT_PATH").expect("Expected a project path env var");
    let mut csv_path =
        std::env::var("BENCHMARK_PATH").expect("Expected benchmark dataset in env var");
    csv_path = format!("{}{}", project_path, csv_path);
    let expected_node_count = std::env::var("EXPECTED_NODE_COUNT")
        .unwrap_or("1000000".to_string())
        .parse::<u32>()
        .unwrap();

    let graph = rwlocked_graph::RWLockedGraph::new(expected_node_count);
    let graph = Arc::new(Mutex::new(graph));

    let graph_clone = graph.clone();
    let csv_path_clone = csv_path.clone();

    c.bench_function("load_graph", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                match graph_clone.lock().unwrap().load_from_tsv(&csv_path_clone) {
                    Ok(_) => info!("Loaded graph from CSV"),
                    Err(e) => warn!("Failed to load graph from CSV: {}", e),
                }
            });
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(20));
    targets = adjacency_parse_benchmark
}
criterion_main!(benches);
