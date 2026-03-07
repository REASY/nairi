# Docker Layout

This directory keeps NAIRI container assets in one place.

## Structure

1. `images/static-analysis`: Docker image for static malware analysis tools (APKTool + JADX + Ghidra + ghidra-cli).
2. `images/backend-server`: Docker image for the Rust Axum backend service.
3. `images/frontend-web`: Docker image for the React frontend served by Nginx.
4. `images/orchestrator`: Starter image for orchestration-related workloads.
5. `compose`: Environment-level compose files (`dev`, `prod`).
6. `scripts`: Helper scripts for building local images.

## Build Examples

Use Make targets (recommended):

```bash
make docker-build
```

Build all images including static-analysis tools:

```bash
GHIDRA_VERSION=11.4 GHIDRA_RELEASE_DATE=20250218 make docker-build-tools
```

Pin tool versions in static-analysis image:

```bash
GHIDRA_VERSION=11.4 \
GHIDRA_RELEASE_DATE=20250218 \
GHIDRA_CLI_REF=master \
RUST_TOOLCHAIN=nightly \
JADX_VERSION=1.5.5 \
GEMINI_CLI_VERSION=0.32.1 \
make build-static
```

Common metadata override:

```bash
VERSION=0.1.0 \
COMMIT_SHA="$(git rev-parse --short HEAD)" \
make docker-build
```

Direct `docker build` examples:

```bash
docker build \
  -f docker/images/backend-server/Dockerfile \
  -t nairi/backend-server:dev \
  .
```

```bash
docker build \
  -f docker/images/frontend-web/Dockerfile \
  -t nairi/frontend-web:dev \
  .
```

Static analysis image requires explicit Ghidra release values:

```bash
docker build \
  -f docker/images/static-analysis/Dockerfile \
  --build-arg GHIDRA_VERSION=11.4 \
  --build-arg GHIDRA_RELEASE_DATE=20250218 \
  --build-arg GHIDRA_CLI_REF=master \
  --build-arg RUST_TOOLCHAIN=nightly \
  --build-arg JADX_VERSION=1.5.5 \
  --build-arg GEMINI_CLI_VERSION=0.32.1 \
  -t nairi/static-analysis:dev \
  .
```

Gemini runtime environment variables:

1. `GEMINI_API_KEY`
2. `GOOGLE_GEMINI_BASE_URL`

## Compose

```bash
make docker-up
```

Enable static-analysis tool container in dev profile:

```bash
GHIDRA_VERSION=11.4 GHIDRA_RELEASE_DATE=20250218 make docker-up-tools
```
