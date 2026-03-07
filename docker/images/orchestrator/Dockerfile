ARG BASE_IMAGE=rust:1.94-bookworm

FROM ${BASE_IMAGE}

ARG VERSION=dev
ARG BUILD_DATE=unknown
ARG COMMIT_SHA=unknown
ARG IMAGE_NAME=nairi-orchestrator
ARG IMAGE_DESCRIPTION="NAIRI orchestration runtime image"
ARG IMAGE_URL=https://github.com/REASY/nairi
ARG IMAGE_SOURCE=https://github.com/REASY/nairi
ARG IMAGE_LICENSES=Apache-2.0
ARG IMAGE_AUTHORS="Artavazd Balaian <reasyu@gmail.com>"
ARG BASE_IMAGE

LABEL \
    org.opencontainers.image.name="${IMAGE_NAME}" \
    org.opencontainers.image.description="${IMAGE_DESCRIPTION}" \
    org.opencontainers.image.url="${IMAGE_URL}" \
    org.opencontainers.image.source="${IMAGE_SOURCE}" \
    org.opencontainers.image.version="${VERSION}" \
    org.opencontainers.image.licenses="${IMAGE_LICENSES}" \
    org.opencontainers.image.authors="${IMAGE_AUTHORS}" \
    org.opencontainers.image.base.name="${BASE_IMAGE}" \
    org.opencontainers.image.created="${BUILD_DATE}" \
    org.opencontainers.image.revision="${COMMIT_SHA}"

WORKDIR /src/backend

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/crates ./crates

# Starter image for orchestration workloads.
# TODO: switch command to a dedicated worker binary when added.
CMD ["cargo", "run", "-p", "nairi-server"]
