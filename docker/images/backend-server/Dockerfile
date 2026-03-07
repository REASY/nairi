ARG RUST_BUILDER_IMAGE=rust:1.94-bookworm
ARG BASE_IMAGE=debian:bookworm-slim

FROM ${RUST_BUILDER_IMAGE} AS builder

WORKDIR /src/backend

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/crates ./crates

RUN cargo build --release -p nairi-server

FROM ${BASE_IMAGE}

ARG VERSION=dev
ARG BUILD_DATE=unknown
ARG COMMIT_SHA=unknown
ARG IMAGE_NAME=nairi-backend-server
ARG IMAGE_DESCRIPTION="NAIRI backend API server in Rust with Axum"
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

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/backend/target/release/nairi-server /usr/local/bin/nairi-server

ENV RUST_LOG=info
EXPOSE 8080

CMD ["/usr/local/bin/nairi-server"]
