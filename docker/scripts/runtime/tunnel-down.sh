#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CONFIG_PATH="${NAIRI_SSH_CONFIG:-${ROOT_DIR}/ops/ssh/ssh.config}"
SOCKET_PATH="${NAIRI_SSH_SOCKET:-${ROOT_DIR}/ops/ssh/tunnel.sock}"
SSH_HOST_ALIAS="${NAIRI_SSH_HOST:-redroid-gcp}"

if [[ ! -f "${CONFIG_PATH}" ]]; then
  echo "Missing SSH config: ${CONFIG_PATH}" >&2
  exit 1
fi

if ssh -S "${SOCKET_PATH}" -F "${CONFIG_PATH}" -O check "${SSH_HOST_ALIAS}" >/dev/null 2>&1; then
  ssh -S "${SOCKET_PATH}" -F "${CONFIG_PATH}" -O exit "${SSH_HOST_ALIAS}" >/dev/null
  echo "Tunnel stopped for ${SSH_HOST_ALIAS}."
else
  echo "Tunnel is not running (${SSH_HOST_ALIAS})."
fi

rm -f "${SOCKET_PATH}"
