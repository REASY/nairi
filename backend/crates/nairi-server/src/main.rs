use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use nairi_api::auth::{AuthService, AuthSettings};
use nairi_orchestrator::Orchestrator;
use nairi_storage::Storage;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let _logger_provider = nairi_core::telemetry::init_otlp_logging(
        "nairi-server",
        tracing::level_filters::LevelFilter::INFO,
        &[
            "nairi_core",
            "nairi_server",
            "nairi_api",
            "nairi_orchestrator",
        ],
    )
        .expect("Failed to initialize OTLP logging");

    let meter_provider = nairi_core::telemetry::init_meter_provider(
        "nairi-server",
        std::time::Duration::from_secs(5),
    )
        .expect("Failed to initialize OTLP metrics");
    opentelemetry::global::set_meter_provider(meter_provider.clone());
    info!("nairi-server OTLP Telemetry initialized");

    if !std::path::Path::new("nairi.db").exists() {
        std::fs::File::create("nairi.db").expect("Failed to create nairi.db");
    }

    let db_url = "sqlite://nairi.db";
    let storage = Arc::new(
        Storage::new(db_url)
            .await
            .expect("Failed to initialize SQLite storage"),
    );
    let workspace_root = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("workspace");
    let orchestrator = Arc::new(Orchestrator::new(storage.clone(), workspace_root));

    let auth_settings = AuthSettings::from_env().expect(
        "Missing Google OAuth configuration. Set GOOGLE_OAUTH_CLIENT_ID, \
GOOGLE_OAUTH_CLIENT_SECRET, GOOGLE_OAUTH_REDIRECT_URI, and SESSION_SIGNING_KEY.",
    );
    let auth_service = Arc::new(AuthService::new(auth_settings));

    let app = build_app(orchestrator, auth_service, allowed_origins_from_env());
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind backend listener");

    info!("nairi-server listening on {}", addr);
    axum::serve(listener, app)
        .await
        .expect("backend server exited with error");
}

fn build_app(
    orchestrator: Arc<Orchestrator>,
    auth_service: Arc<AuthService>,
    allowed_origins: Vec<HeaderValue>,
) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true);

    nairi_api::router(nairi_api::AppState {
        orchestrator,
        auth: auth_service,
    })
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

fn allowed_origins_from_env() -> Vec<HeaderValue> {
    let raw = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| {
        "http://localhost:3000,http://127.0.0.1:3000,http://localhost:5173,http://127.0.0.1:5173"
            .to_string()
    });

    raw.split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(|origin| {
            HeaderValue::from_str(origin)
                .unwrap_or_else(|_| panic!("Invalid origin in ALLOWED_ORIGINS: {origin}"))
        })
        .collect()
}
