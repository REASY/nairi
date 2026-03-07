pub mod auth;

use std::sync::Arc;

use auth::{AuthError, AuthUser, GoogleCallbackQuery};
use axum::extract::{Path, Query, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Json, Router, routing::get, routing::post};
use nairi_core::analysis::AnalysisRun;
use nairi_core::config::{AppConfig, PromptConfig};
use nairi_orchestrator::Orchestrator;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::path::PathBuf;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<Orchestrator>,
    pub auth: Arc<auth::AuthService>,
}

pub fn router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/api/v1/auth/me", get(get_current_user))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/health", get(health))
        .route("/api/v1/config", get(get_config).post(update_config))
        .route(
            "/api/v1/prompts/{name}",
            get(get_prompt).post(update_prompt),
        )
        .route("/api/v1/analyses", post(create_analysis))
        .route("/api/v1/analyses/{id}", get(get_analysis))
        .route("/api/v1/analyses/{id}/stream", get(stream_analysis))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .route("/api/v1/auth/google/login", get(begin_google_login))
        .route("/api/v1/auth/google/callback", get(complete_google_login))
        .merge(protected_routes)
        .with_state(state)
}

async fn require_auth(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    match state.auth.authenticate_request(request.headers()) {
        Ok(user) => {
            request.extensions_mut().insert(user);
            next.run(request).await
        }
        Err(error) => auth_error_response(error),
    }
}

async fn begin_google_login(State(state): State<AppState>) -> Response {
    match state.auth.begin_google_login() {
        Ok(redirect_url) => Redirect::temporary(&redirect_url).into_response(),
        Err(error) => auth_error_response(error),
    }
}

async fn complete_google_login(
    State(state): State<AppState>,
    Query(query): Query<GoogleCallbackQuery>,
) -> Response {
    if let Some(error_description) = query.error_description.as_deref() {
        error!(error_description = %error_description, "google callback returned error description");
    }

    let user = match state.auth.complete_google_login(query).await {
        Ok(user) => user,
        Err(error) => return auth_error_response(error),
    };

    let session_cookie = match state.auth.issue_session_cookie(&user) {
        Ok(cookie) => cookie,
        Err(error) => return auth_error_response(error),
    };

    let cookie_header = match session_cookie.parse() {
        Ok(value) => value,
        Err(_) => {
            return auth_error_response(AuthError::Internal("invalid_session_cookie_header"));
        }
    };

    let mut response = Redirect::to(state.auth.post_login_redirect_url()).into_response();
    response
        .headers_mut()
        .append(header::SET_COOKIE, cookie_header);
    response
}

#[derive(Debug, Serialize)]
struct CurrentUserResponse {
    user: AuthUser,
}

async fn get_current_user(Extension(user): Extension<AuthUser>) -> impl IntoResponse {
    (StatusCode::OK, Json(CurrentUserResponse { user })).into_response()
}

async fn logout(State(state): State<AppState>) -> Response {
    let cookie = state.auth.clear_session_cookie();
    let mut response = StatusCode::NO_CONTENT.into_response();

    if let Ok(cookie_header) = cookie.parse() {
        response
            .headers_mut()
            .append(header::SET_COOKIE, cookie_header);
    }

    response
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
    State(state): State<AppState>,
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

    // Dummy path for now until upload is implemented
    let apk_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("app/build/outputs/apk/release/app-arm64-v8a-release-unsigned.apk");

    let run = state
        .orchestrator
        .create_run(request.package_name, apk_path)
        .await;
    (StatusCode::CREATED, Json(CreateAnalysisResponse { run })).into_response()
}

async fn get_analysis(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let run_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "invalid_id",
                    "run id must be a valid UUID",
                )),
            )
                .into_response();
        }
    };

    match state.orchestrator.get_run(run_id).await {
        Some(run) => (StatusCode::OK, Json(run)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "analysis run not found")),
        )
            .into_response(),
    }
}

async fn stream_analysis(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _run_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return Sse::new(tokio_stream::empty::<Result<Event, Infallible>>()).into_response();
        }
    };

    let rx = state.orchestrator.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|res| res.ok())
        .map(|event| {
            let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
            Ok::<_, Infallible>(Event::default().json_data(json).unwrap_or(Event::default()))
        });

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(1))
                .text("keep-alive-text"),
        )
        .into_response()
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ConfigResponse {
    config: AppConfig,
}

async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.orchestrator.get_config().await;
    (StatusCode::OK, Json(ConfigResponse { config })).into_response()
}

async fn update_config(
    State(state): State<AppState>,
    Json(config): Json<AppConfig>,
) -> impl IntoResponse {
    state.orchestrator.update_config(config.clone()).await;
    (StatusCode::OK, Json(ConfigResponse { config })).into_response()
}

#[derive(Debug, Deserialize)]
struct UpdatePromptRequest {
    content: String,
}

#[derive(Debug, Serialize)]
struct PromptResponse {
    prompt: PromptConfig,
}

async fn get_prompt(State(state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    match state.orchestrator.get_prompt(&name).await {
        Some(prompt) => (StatusCode::OK, Json(PromptResponse { prompt })).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "prompt not found")),
        )
            .into_response(),
    }
}

async fn update_prompt(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<UpdatePromptRequest>,
) -> impl IntoResponse {
    let prompt = PromptConfig {
        name: name.clone(),
        content: request.content,
    };
    state.orchestrator.update_prompt(prompt.clone()).await;
    (StatusCode::OK, Json(PromptResponse { prompt })).into_response()
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

impl ErrorResponse {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

fn auth_error_response(error: AuthError) -> Response {
    let (status, message) = match error {
        AuthError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "Invalid authentication request"),
        AuthError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "Authentication required"),
        AuthError::Upstream(_) => (StatusCode::BAD_GATEWAY, "Google authentication failed"),
        AuthError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal authentication error",
        ),
    };

    (status, Json(ErrorResponse::new(error.code(), message))).into_response()
}
