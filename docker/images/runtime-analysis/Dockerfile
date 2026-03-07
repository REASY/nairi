ARG BASE_IMAGE=ubuntu:24.04

FROM ${BASE_IMAGE}

ARG DEBIAN_FRONTEND=noninteractive
ARG ANDROID_CMDLINE_TOOLS_URL=https://dl.google.com/android/repository/commandlinetools-linux-14742923_latest.zip
ARG ANDROID_PLATFORM_API=36
ARG GEMINI_CLI_VERSION=0.32.1
ARG NODE_MAJOR=22
ARG VERSION=dev
ARG COMMIT_SHA=unknown
ARG IMAGE_NAME=nairi-runtime-analysis
ARG IMAGE_DESCRIPTION="NAIRI runtime analysis image with Android SDK platform-tools and Gemini CLI"
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
    android-sdk-platform-tools \
    android-tools-adb \
    ca-certificates \
    curl \
    file \
    git \
    gnupg \
    iproute2 \
    netcat-openbsd \
    openjdk-21-jdk-headless \
    procps \
    python3 \
    python3-pip \
    unzip \
    wget \
    && rm -rf /var/lib/apt/lists/*

RUN adb --version

RUN mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key \
    | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" \
    > /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

ENV ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_HOME=/opt/android-sdk
ENV PATH="${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin:${PATH}"


RUN npm install -g "@google/gemini-cli@${GEMINI_CLI_VERSION}" && \
    pip3 install --break-system-packages frida-tools

COPY docker/images/runtime-analysis/.agents/skills/ /root/.agents/skills/
COPY docker/images/runtime-analysis/ebpf /ebpf
COPY docker/images/runtime-analysis/runtime /runtime

RUN chmod +x /ebpf/runners/run_trace_experiments.sh \
    /runtime/run_runtime_analysis.sh \
    /runtime/ui_explorer.py

ENV GEMINI_API_KEY=""
ENV GOOGLE_GEMINI_BASE_URL=""
ENV ADB_CONNECTION_STRING=""
WORKDIR /workspace

CMD ["/bin/bash"]
