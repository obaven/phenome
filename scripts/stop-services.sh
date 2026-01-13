#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ANALYTICS_PID_FILE="${ROOT_DIR}/.analytics-service.pid"
ML_PID_FILE="${ROOT_DIR}/.ml-service.pid"

stop_pid() {
  local pid_file="$1"
  if [[ -f "${pid_file}" ]]; then
    local pid
    pid="$(cat "${pid_file}")"
    if [[ -n "${pid}" ]]; then
      kill "${pid}" 2>/dev/null || true
      for _ in {1..10}; do
        if kill -0 "${pid}" 2>/dev/null; then
          sleep 1
        else
          break
        fi
      done
      if kill -0 "${pid}" 2>/dev/null; then
        kill -9 "${pid}" 2>/dev/null || true
      fi
    fi
    rm -f "${pid_file}"
  fi
}

stop_pid "${ANALYTICS_PID_FILE}"
stop_pid "${ML_PID_FILE}"
