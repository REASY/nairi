ARG BASE_IMAGE=ubuntu:24.04

FROM ${BASE_IMAGE}

ARG DEBIAN_FRONTEND=noninteractive
ARG APKTOOL_VERSION=2.11.1
ARG GHIDRA_VERSION
ARG GHIDRA_RELEASE_DATE
ARG GHIDRA_CLI_REF=master
ARG RUST_TOOLCHAIN=stable
ARG JADX_VERSION=1.5.5
ARG GEMINI_CLI_VERSION=0.32.1
ARG NODE_MAJOR=22
ARG VERSION=dev
ARG COMMIT_SHA=unknown
ARG IMAGE_NAME=nairi-static-analysis
ARG IMAGE_DESCRIPTION="NAIRI static analysis tools image with APKTool, JADX, Ghidra, ghidra-cli, and Gemini CLI"
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
    org.opencontainers.image.revision="${COMMIT_SHA}"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    file \
    gcc \
    git \
    gnupg \
    libc6-dev \
    libssl-dev \
    openjdk-21-jdk-headless \
    pkg-config \
    python3 \
    python3-pip \
    unzip \
    wget \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key \
    | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" \
    > /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

RUN java -version && javac -version

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH="${CARGO_HOME}/bin:${PATH}"

RUN set -eux; \
    arch="$(dpkg --print-architecture)"; \
    case "${arch}" in \
      amd64) rust_arch='x86_64-unknown-linux-gnu' ;; \
      arm64) rust_arch='aarch64-unknown-linux-gnu' ;; \
      armhf) rust_arch='armv7-unknown-linux-gnueabihf' ;; \
      i386) rust_arch='i686-unknown-linux-gnu' ;; \
      *) echo "Unsupported architecture: ${arch}" >&2; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/dist/${rust_arch}/rustup-init"; \
    wget -q "${url}" -O /tmp/rustup-init; \
    chmod +x /tmp/rustup-init; \
    /tmp/rustup-init -y --no-modify-path --default-toolchain "${RUST_TOOLCHAIN}"; \
    rm -f /tmp/rustup-init; \
    chmod -R a+w "${RUSTUP_HOME}" "${CARGO_HOME}"; \
    "${CARGO_HOME}/bin/rustup" --version; \
    "${CARGO_HOME}/bin/cargo" --version; \
    "${CARGO_HOME}/bin/rustc" --version

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

RUN git clone --depth 1 --branch "${GHIDRA_CLI_REF}" \
    https://github.com/akiselev/ghidra-cli /tmp/ghidra-cli \
    && cd /tmp/ghidra-cli \
    && cargo install --path . \
    && rm -rf /tmp/ghidra-cli

RUN wget -q -O /tmp/jadx.zip \
    "https://github.com/skylot/jadx/releases/download/v${JADX_VERSION}/jadx-${JADX_VERSION}.zip" \
    && unzip -q /tmp/jadx.zip -d /opt \
    && rm -f /tmp/jadx.zip \
    && ln -s /opt/jadx-${JADX_VERSION} /opt/jadx \
    && ln -sf /opt/jadx/bin/jadx /usr/local/bin/jadx \
    && ln -sf /opt/jadx/bin/jadx-gui /usr/local/bin/jadx-gui

# Install AI Agents
RUN npm install -g "@google/gemini-cli@${GEMINI_CLI_VERSION}" && \
    pip3 install --break-system-packages aider-chat

COPY docker/images/static-analysis/.agents/skills/ /root/.agents/skills/

ENV GHIDRA_HOME=/opt/ghidra
ENV GHIDRA_INSTALL_DIR=/opt/ghidra
ENV JADX_HOME=/opt/jadx
ENV JAVA_HOME=/usr/lib/jvm/default-java
ENV GEMINI_API_KEY=""
ENV GOOGLE_GEMINI_BASE_URL=""
ENV PATH="${CARGO_HOME}/bin:${JAVA_HOME}/bin:${GHIDRA_HOME}/support:${JADX_HOME}/bin:${PATH}"
WORKDIR /workspace

# Default command if nothing is passed; orchestrator will override this
CMD ["/bin/bash"]
