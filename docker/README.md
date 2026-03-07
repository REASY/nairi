# Docker Layout

This directory keeps NAIRI container assets in one place.

## Structure

1. `images/static-analysis`: Docker image for static malware analysis tools (APKTool + Ghidra).
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

Common metadata override:

```bash
VERSION=0.1.0 \
BUILD_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
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
  -t nairi/static-analysis:dev \
  .
```

## Compose

```bash
make docker-up
```

Enable static-analysis tool container in dev profile:

```bash
GHIDRA_VERSION=11.4 GHIDRA_RELEASE_DATE=20250218 make docker-up-tools
```
