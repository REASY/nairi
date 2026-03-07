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
GOOGLE_OAUTH_CLIENT_ID=... \
GOOGLE_OAUTH_CLIENT_SECRET=... \
GOOGLE_OAUTH_REDIRECT_URI=http://localhost:8080/api/v1/auth/google/callback \
SESSION_SIGNING_KEY=replace-with-long-random-secret \
AUTH_POST_LOGIN_REDIRECT_URL=http://localhost:5173/ \
cargo run -p nairi-server
```

Server defaults to:

1. `http://localhost:8080`

## Google OAuth Environment Variables

1. `GOOGLE_OAUTH_CLIENT_ID` (required)
2. `GOOGLE_OAUTH_CLIENT_SECRET` (required)
3. `GOOGLE_OAUTH_REDIRECT_URI` (required)
4. `SESSION_SIGNING_KEY` (required)
5. `AUTH_POST_LOGIN_REDIRECT_URL` (optional, default `http://localhost:5173/`): Where to redirect the user after a
   successful login.
6. `ALLOWED_ORIGINS` (optional, comma-separated list for CORS + credentials)
7. `SESSION_COOKIE_SECURE` (optional, default true on HTTPS redirects, false on HTTP)
8. `SESSION_COOKIE_NAME` (optional, default `nairi_session`)
9. `SESSION_TTL_SECONDS` (optional, default `28800`)
10. `SESSION_COOKIE_DOMAIN` (optional)
11. `ALLOWED_GOOGLE_HOSTED_DOMAIN` (optional, restrict to Google Workspace domain)

## Routes

1. Public OAuth routes:
    - `GET /api/v1/auth/google/login`
    - `GET /api/v1/auth/google/callback`
2. Protected routes:
    - `GET /api/v1/auth/me`
    - `POST /api/v1/auth/logout`
    - `GET /api/v1/health`
    - `GET|POST /api/v1/config`
    - `GET|POST /api/v1/prompts/:name`
    - `POST /api/v1/analyses`
    - `GET /api/v1/analyses/:id`
    - `GET /api/v1/analyses/:id/stream`
