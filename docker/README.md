# Docker Layout

This directory keeps NAIRI container assets in one place.

## Structure

1. `images/static-analysis`: Docker image for static malware analysis tools (APKTool + JADX + Ghidra + ghidra-cli).
2. `images/runtime-analysis`: Docker image for runtime malware analysis tools (ADB + Gemini CLI).
3. `images/backend-server`: Docker image for the Rust Axum backend service.
4. `images/frontend-web`: Docker image for the React frontend served by Nginx.
5. `images/orchestrator`: Starter image for orchestration-related workloads.
6. `compose`: Environment-level compose files (`dev`, `prod`).
7. `scripts`: Helper scripts for building local images.

## Build Examples

Use Make targets (recommended):

```bash
make docker-build
```

Build all images including analysis tools:

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

Build runtime-analysis image:

```bash
GEMINI_CLI_VERSION=0.32.1 \
ANDROID_CMDLINE_TOOLS_URL=https://dl.google.com/android/repository/commandlinetools-linux-14742923_latest.zip \
ANDROID_PLATFORM_API=36 \
make build-runtime
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

Runtime analysis image build example:

```bash
docker build \
  -f docker/images/runtime-analysis/Dockerfile \
  --build-arg GEMINI_CLI_VERSION=0.32.1 \
  --build-arg ANDROID_CMDLINE_TOOLS_URL=https://dl.google.com/android/repository/commandlinetools-linux-14742923_latest.zip \
  --build-arg ANDROID_PLATFORM_API=36 \
  -t nairi/runtime-analysis:dev \
  .
```

The runtime image installs Android SDK with:

1. `sdkmanager "platform-tools" "platforms;android-36"`
2. On `linux/arm64`, image uses distro `adb` (`android-sdk-platform-tools`) for binary compatibility while still
   installing SDK packages via `sdkmanager`.

Gemini runtime environment variables:

1. `GEMINI_API_KEY`
2. `GOOGLE_GEMINI_BASE_URL`
3. `ADB_CONNECTION_STRING`

## Compose

```bash
make docker-up
```

Enable analysis tool containers in dev profile:

```bash
GHIDRA_VERSION=11.4 GHIDRA_RELEASE_DATE=20250218 make docker-up-tools
```

Run runtime-analysis container directly:

```bash
docker run --rm -it \
  -e GEMINI_API_KEY="$GEMINI_API_KEY" \
  -e GOOGLE_GEMINI_BASE_URL="$GOOGLE_GEMINI_BASE_URL" \
  -e ADB_CONNECTION_STRING="host.docker.internal:5555" \
  nairi/runtime-analysis:dev \
  bash
```
