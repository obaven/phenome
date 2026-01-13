#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ANALYTICS_PID_FILE="${ROOT_DIR}/.analytics-service.pid"
ML_PID_FILE="${ROOT_DIR}/.ml-service.pid"

cleanup() {
  "${ROOT_DIR}/scripts/stop-services.sh" || true
}

trap cleanup EXIT

(
  cd "${ROOT_DIR}"
  cargo run --bin analytics-service --features analytics >/tmp/rotappo-analytics.log 2>&1 &
  echo $! >"${ANALYTICS_PID_FILE}"
)

(
  cd "${ROOT_DIR}"
  cargo run --bin ml-service --features ml >/tmp/rotappo-ml.log 2>&1 &
  echo $! >"${ML_PID_FILE}"
)

sleep 2

(
  cd "${ROOT_DIR}"
  cargo run --bin tui --features tui,module-bootstrappo
)
