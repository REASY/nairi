use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router, routing::get, routing::post};
use nairi_core::analysis::AnalysisRun;
use nairi_orchestrator::Orchestrator;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router(orchestrator: Arc<Orchestrator>) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/analyses", post(create_analysis))
        .route("/api/v1/analyses/:id", get(get_analysis))
        .with_state(orchestrator)
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

#[derive(Debug, Deserialize)]
struct CreateAnalysisRequest {
    package_name: String,
}

#[derive(Debug, Serialize)]
struct CreateAnalysisResponse {
    run: AnalysisRun,
}

async fn create_analysis(
    State(orchestrator): State<Arc<Orchestrator>>,
    Json(request): Json<CreateAnalysisRequest>,
) -> impl IntoResponse {
    if request.package_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_request",
                "package_name cannot be empty",
            )),
        )
            .into_response();
    }

    let run = orchestrator.create_run(request.package_name).await;
    (StatusCode::CREATED, Json(CreateAnalysisResponse { run })).into_response()
}

async fn get_analysis(
    State(orchestrator): State<Arc<Orchestrator>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let run_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_id", "run id must be a valid UUID")),
            )
                .into_response();
        }
    };

    match orchestrator.get_run(run_id).await {
        Some(run) => (StatusCode::OK, Json(run)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "analysis run not found")),
        )
            .into_response(),
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: &'static str,
}

impl ErrorResponse {
    fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
}
