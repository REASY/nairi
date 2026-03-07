ARG NODE_BUILDER_IMAGE=node:22-alpine
ARG BASE_IMAGE=nginx:1.27-alpine

FROM ${NODE_BUILDER_IMAGE} AS build

WORKDIR /app

COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

FROM ${BASE_IMAGE}

ARG VERSION=dev
ARG BUILD_DATE=unknown
ARG COMMIT_SHA=unknown
ARG IMAGE_NAME=nairi-frontend-web
ARG IMAGE_DESCRIPTION="NAIRI frontend web UI served by Nginx"
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

COPY docker/images/frontend-web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
