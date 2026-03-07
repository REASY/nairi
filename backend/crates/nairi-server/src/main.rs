use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use nairi_orchestrator::Orchestrator;
use nairi_storage::Storage;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    init_tracing();

    let storage = Arc::new(Storage::new());
    let orchestrator = Arc::new(Orchestrator::new(storage.clone()));

    let app = build_app(orchestrator);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind backend listener");

    info!("nairi-server listening on {}", addr);
    axum::serve(listener, app)
        .await
        .expect("backend server exited with error");
}

fn build_app(orchestrator: Arc<Orchestrator>) -> Router {
    nairi_api::router(orchestrator)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

fn init_tracing() {
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}
