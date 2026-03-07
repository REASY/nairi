# NAIRI Backend Workspace

Rust workspace for NAIRI backend services and libraries.

## Workspace Crates

1. `nairi-core`: domain models and shared types.
2. `nairi-storage`: storage abstractions and implementations.
3. `nairi-orchestrator`: analysis-run lifecycle orchestration.
4. `nairi-api`: Axum route and handler layer.
5. `nairi-server`: executable server binary.

## Run

```bash
cargo run -p nairi-server
```

Server defaults to:

1. `http://localhost:8080`

## Routes

1. `GET /api/v1/health`
2. `POST /api/v1/analyses`
3. `GET /api/v1/analyses/:id`
