ARG BASE_IMAGE=ubuntu:24.04

FROM ${BASE_IMAGE}

ARG DEBIAN_FRONTEND=noninteractive
ARG APKTOOL_VERSION=2.11.1
ARG GHIDRA_VERSION
ARG GHIDRA_RELEASE_DATE
ARG VERSION=dev
ARG BUILD_DATE=unknown
ARG COMMIT_SHA=unknown
ARG IMAGE_NAME=nairi-static-analysis
ARG IMAGE_DESCRIPTION="NAIRI static analysis tools image with APKTool and Ghidra"
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
    curl \
    file \
    git \
    openjdk-21-jre-headless \
    python3 \
    unzip \
    wget \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*

RUN wget -q -O /usr/local/bin/apktool \
    "https://raw.githubusercontent.com/iBotPeaches/Apktool/master/scripts/linux/apktool" \
    && chmod +x /usr/local/bin/apktool \
    && wget -q -O /usr/local/bin/apktool.jar \
    "https://github.com/iBotPeaches/Apktool/releases/download/v${APKTOOL_VERSION}/apktool_${APKTOOL_VERSION}.jar"

RUN test -n "${GHIDRA_VERSION}" || (echo "Missing GHIDRA_VERSION build-arg" && exit 1) \
    && test -n "${GHIDRA_RELEASE_DATE}" || (echo "Missing GHIDRA_RELEASE_DATE build-arg" && exit 1)

RUN wget -q -O /tmp/ghidra.zip \
    "https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_${GHIDRA_VERSION}_build/ghidra_${GHIDRA_VERSION}_PUBLIC_${GHIDRA_RELEASE_DATE}.zip" \
    && unzip -q /tmp/ghidra.zip -d /opt \
    && rm -f /tmp/ghidra.zip \
    && ln -s /opt/ghidra_${GHIDRA_VERSION}_PUBLIC /opt/ghidra

ENV GHIDRA_HOME=/opt/ghidra
ENV PATH="${GHIDRA_HOME}/support:${PATH}"
WORKDIR /workspace

CMD ["/bin/bash"]
