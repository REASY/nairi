#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CONFIG_PATH="${NAIRI_SSH_CONFIG:-${ROOT_DIR}/ops/ssh/ssh.config}"
SOCKET_PATH="${NAIRI_SSH_SOCKET:-${ROOT_DIR}/ops/ssh/tunnel.sock}"
SSH_HOST_ALIAS="${NAIRI_SSH_HOST:-redroid-gcp}"
ADB_LOCAL_PORT="${NAIRI_ADB_LOCAL_PORT:-5555}"

if [[ ! -f "${CONFIG_PATH}" ]]; then
  echo "Missing SSH config: ${CONFIG_PATH}" >&2
  echo "Create it from template: cp ${ROOT_DIR}/ops/ssh/ssh.config.example ${CONFIG_PATH}" >&2
  exit 1
fi

if ssh -S "${SOCKET_PATH}" -F "${CONFIG_PATH}" -O check "${SSH_HOST_ALIAS}" >/dev/null 2>&1; then
  echo "Tunnel is already running (${SSH_HOST_ALIAS})."
else
  ssh -fN -M -S "${SOCKET_PATH}" -F "${CONFIG_PATH}" "${SSH_HOST_ALIAS}"
  echo "Tunnel started for ${SSH_HOST_ALIAS}."
fi

if nc -z 127.0.0.1 "${ADB_LOCAL_PORT}" >/dev/null 2>&1; then
  echo "ADB tunnel is reachable on localhost:${ADB_LOCAL_PORT}."
else
  echo "Warning: localhost:${ADB_LOCAL_PORT} did not respond yet." >&2
fi

echo "Use this for runtime-analysis container: ADB_CONNECTION_STRING=host.docker.internal:${ADB_LOCAL_PORT}"
