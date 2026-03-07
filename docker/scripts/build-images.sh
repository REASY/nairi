#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

VERSION="${VERSION:-dev}"
COMMIT_SHA="${COMMIT_SHA:-$(git -C "${ROOT_DIR}" rev-parse --short HEAD 2>/dev/null || echo unknown)}"
IMAGE_URL="${IMAGE_URL:-https://github.com/REASY/nairi}"
IMAGE_SOURCE="${IMAGE_SOURCE:-https://github.com/REASY/nairi}"
IMAGE_LICENSES="${IMAGE_LICENSES:-Apache-2.0}"
IMAGE_AUTHORS="${IMAGE_AUTHORS:-Artavazd Balaian <reasyu@gmail.com>}"
GHIDRA_CLI_REF="${GHIDRA_CLI_REF:-master}"
RUST_TOOLCHAIN="${RUST_TOOLCHAIN:-nightly}"
JADX_VERSION="${JADX_VERSION:-1.5.5}"
GEMINI_CLI_VERSION="${GEMINI_CLI_VERSION:-0.32.1}"

COMMON_ARGS=(
  --build-arg "VERSION=${VERSION}"
  --build-arg "COMMIT_SHA=${COMMIT_SHA}"
  --build-arg "IMAGE_URL=${IMAGE_URL}"
  --build-arg "IMAGE_SOURCE=${IMAGE_SOURCE}"
  --build-arg "IMAGE_LICENSES=${IMAGE_LICENSES}"
  --build-arg "IMAGE_AUTHORS=${IMAGE_AUTHORS}"
)

docker build \
  "${COMMON_ARGS[@]}" \
  -f "${ROOT_DIR}/docker/images/backend-server/Dockerfile" \
  -t "nairi/backend-server:${VERSION}" \
  "${ROOT_DIR}"

docker build \
  "${COMMON_ARGS[@]}" \
  -f "${ROOT_DIR}/docker/images/frontend-web/Dockerfile" \
  -t "nairi/frontend-web:${VERSION}" \
  "${ROOT_DIR}"

if [[ -n "${GHIDRA_VERSION:-}" && -n "${GHIDRA_RELEASE_DATE:-}" ]]; then
  docker build \
    "${COMMON_ARGS[@]}" \
    -f "${ROOT_DIR}/docker/images/static-analysis/Dockerfile" \
    --build-arg GHIDRA_CLI_REF="${GHIDRA_CLI_REF}" \
    --build-arg RUST_TOOLCHAIN="${RUST_TOOLCHAIN}" \
    --build-arg JADX_VERSION="${JADX_VERSION}" \
    --build-arg GEMINI_CLI_VERSION="${GEMINI_CLI_VERSION}" \
    --build-arg GHIDRA_VERSION="${GHIDRA_VERSION}" \
    --build-arg GHIDRA_RELEASE_DATE="${GHIDRA_RELEASE_DATE}" \
    -t "nairi/static-analysis:${VERSION}" \
    "${ROOT_DIR}"
else
  echo "Skipping static-analysis image build."
  echo "Set GHIDRA_VERSION and GHIDRA_RELEASE_DATE to build it."
fi

echo "Docker images built."
