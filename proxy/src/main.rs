use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::{response::IntoResponse, extract::{Path, Request}, Router, routing::any};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::{info, Level};
use tracing_subscriber;
use reqwest;

#[derive(Default)]
struct Stats {
    requests: u64,
}

async fn forward_handler(
    Path(path): Path<String>,
    req: Request,
    stats: Arc<Mutex<Stats>>
) -> impl IntoResponse {
    let start = Instant::now();

    {
        let mut s = stats.lock().unwrap();
        s.requests += 1;
    }

    let backend_url = format!("http://127.0.0.1:8080/{}", path);
    let res = reqwest::get(&backend_url).await.unwrap();
    let status = res.status();
    let body = res.text().await.unwrap();

    let duration = start.elapsed();
    info!("Forwarded [{}] {} -> {} (status: {}, time: {:?})",
          req.method(),
          path,
          backend_url,
          status,
          duration
    );

    body
}


#[tokio::main]
async fn main() {
    // Initialiser le logger
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let stats = Arc::new(Mutex::new(Stats::default()));

    let app = Router::new()
        .route("/{*path}", any({
            let stats = stats.clone();
            move |path, req| forward_handler(path, req, stats)
        }))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7070")
        .await
        .unwrap();

    println!("Proxy running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .unwrap();
}
