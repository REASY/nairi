#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  /runtime/run_runtime_analysis.sh [options]

Coordinates two runtime planes:
1) Trace Plane: runs eBPF two-phase tracing via /ebpf/runners/run_trace_experiments.sh
2) Interaction Plane: explores UI via /runtime/ui_explorer.py

Options:
  --device <serial>              adb serial/target (default: $ADB_CONNECTION_STRING or host.docker.internal:15555)
  --package <name>               Android package name (default: $PACKAGE_NAME)
  --apk <path>                   APK path in container (default: /workspace/target.apk)
  --reports-dir <dir>            Output root directory (default: /workspace/reports)
  --trace-script <path>          Trace runner script (default: /ebpf/runners/run_trace_experiments.sh)
  --probes-dir <dir>             bpftrace probes directory (default: /ebpf/probes)
  --trace-phase-seconds <sec>    Total trace duration budget across both phases (default: 75)
  --ui-steps <n>                 Max UI exploration steps (default: 120)
  --ui-interval-sec <sec>        Delay between UI actions (default: 2.0)
  --ui-monkey-every <n>          Monkey burst frequency (default: 0, recommended)
  --skip-ui                      Disable UI interaction plane
  -h, --help                     Show this help

Environment:
  ADB_CONNECTION_STRING
  PACKAGE_NAME
USAGE
}

DEVICE="${ADB_CONNECTION_STRING:-host.docker.internal:15555}"
PACKAGE="${PACKAGE_NAME:-}"
APK_PATH="/workspace/target.apk"
REPORTS_DIR="/workspace/reports"
TRACE_SCRIPT="/ebpf/runners/run_trace_experiments.sh"
PROBES_DIR="/ebpf/probes"
TRACE_PHASE_SECONDS=75
UI_STEPS=120
UI_INTERVAL_SEC=2.0
UI_MONKEY_EVERY=0
SKIP_UI=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --device)
      DEVICE="$2"; shift 2 ;;
    --package)
      PACKAGE="$2"; shift 2 ;;
    --apk)
      APK_PATH="$2"; shift 2 ;;
    --reports-dir)
      REPORTS_DIR="$2"; shift 2 ;;
    --trace-script)
      TRACE_SCRIPT="$2"; shift 2 ;;
    --probes-dir)
      PROBES_DIR="$2"; shift 2 ;;
    --trace-phase-seconds)
      TRACE_PHASE_SECONDS="$2"; shift 2 ;;
    --ui-steps)
      UI_STEPS="$2"; shift 2 ;;
    --ui-interval-sec)
      UI_INTERVAL_SEC="$2"; shift 2 ;;
    --ui-monkey-every)
      UI_MONKEY_EVERY="$2"; shift 2 ;;
    --skip-ui)
      SKIP_UI=1; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1 ;;
  esac
done

if [[ -z "$PACKAGE" ]]; then
  echo "Package is required. Provide --package or PACKAGE_NAME env var." >&2
  exit 1
fi
if [[ ! -f "$APK_PATH" ]]; then
  echo "APK not found at $APK_PATH" >&2
  exit 1
fi
if [[ ! -x "$TRACE_SCRIPT" ]]; then
  echo "Trace runner not executable: $TRACE_SCRIPT" >&2
  exit 1
fi
if ! [[ "$TRACE_PHASE_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "--trace-phase-seconds must be numeric" >&2
  exit 1
fi
if ! [[ "$UI_STEPS" =~ ^[0-9]+$ ]]; then
  echo "--ui-steps must be numeric" >&2
  exit 1
fi
if ! [[ "$UI_MONKEY_EVERY" =~ ^[0-9]+$ ]]; then
  echo "--ui-monkey-every must be numeric" >&2
  exit 1
fi

mkdir -p "$REPORTS_DIR"
TRACE_OUT_DIR="${REPORTS_DIR%/}/runtime-traces"
UI_OUT_DIR="${REPORTS_DIR%/}/ui-explorer"
mkdir -p "$TRACE_OUT_DIR" "$UI_OUT_DIR"

COMMAND_LOG="${REPORTS_DIR%/}/runtime-command-log.md"
REPORT_MD="${REPORTS_DIR%/}/runtime-analysis-report.md"
FINDINGS_JSON="${REPORTS_DIR%/}/runtime-findings.json"
RUN_LOG="${REPORTS_DIR%/}/runtime-runner.log"
UI_LOG="${UI_OUT_DIR%/}/ui-explorer.log"

cat > "$COMMAND_LOG" <<LOG
# Runtime Command Log

LOG

log_cmd() {
  local cmd="$1"
  {
    printf "## %s\n\n" "$(date -Iseconds)"
    printf '```bash\n%s\n```\n\n' "$cmd"
  } >> "$COMMAND_LOG"
}

run_cmd() {
  local cmd="$1"
  log_cmd "$cmd"
  bash -lc "$cmd"
}

UI_PID=""
cleanup() {
  if [[ -n "${UI_PID:-}" ]]; then
    kill "$UI_PID" 2>/dev/null || true
    wait "$UI_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

{
  echo "[runtime] started $(date -Iseconds)"
  echo "[runtime] device=${DEVICE}"
  echo "[runtime] package=${PACKAGE}"
  echo "[runtime] apk=${APK_PATH}"
  echo "[runtime] reports=${REPORTS_DIR}"

  run_cmd "adb start-server"
  run_cmd "adb connect '${DEVICE}'"
  run_cmd "adb -s '${DEVICE}' root || true"
  run_cmd "adb -s '${DEVICE}' devices -l"
  run_cmd "adb -s '${DEVICE}' install -r -d '${APK_PATH}'"

  if [[ "$SKIP_UI" -eq 0 ]]; then
    log_cmd "python3 /runtime/ui_explorer.py --device '${DEVICE}' --package '${PACKAGE}' --out-dir '${UI_OUT_DIR}' --steps '${UI_STEPS}' --interval-sec '${UI_INTERVAL_SEC}' --monkey-every '${UI_MONKEY_EVERY}' --strict-package"
    python3 /runtime/ui_explorer.py \
      --device "$DEVICE" \
      --package "$PACKAGE" \
      --out-dir "$UI_OUT_DIR" \
      --steps "$UI_STEPS" \
      --interval-sec "$UI_INTERVAL_SEC" \
      --monkey-every "$UI_MONKEY_EVERY" \
      --strict-package \
      > "$UI_LOG" 2>&1 &
    UI_PID="$!"
    echo "[runtime] ui explorer pid=${UI_PID}"
  else
    echo "[runtime] ui explorer disabled (--skip-ui)"
  fi

  TRACE_CMD="bash '${TRACE_SCRIPT}' auto --package '${PACKAGE}' --device '${DEVICE}' --out-dir '${TRACE_OUT_DIR}' --probes-dir '${PROBES_DIR}' --transport run --phase-seconds '${TRACE_PHASE_SECONDS}'"
  run_cmd "$TRACE_CMD"

  if [[ -n "${UI_PID:-}" ]]; then
    kill "$UI_PID" 2>/dev/null || true
    wait "$UI_PID" 2>/dev/null || true
    UI_PID=""
  fi

  PARSE_CMD="python3 /ebpf/parsers/parse_trace_experiment_csv.py '${TRACE_OUT_DIR}' --summary-out '${TRACE_OUT_DIR}/summary_metrics.csv' --grouped-out '${TRACE_OUT_DIR}/grouped_checks.csv'"
  log_cmd "$PARSE_CMD"
  bash -lc "$PARSE_CMD" || true

  summary_rows=0
  grouped_rows=0
  ui_screenshots=0
  if [[ -f "${TRACE_OUT_DIR}/summary_metrics.csv" ]]; then
    summary_rows="$(($(wc -l < "${TRACE_OUT_DIR}/summary_metrics.csv") - 1))"
    if [[ "$summary_rows" -lt 0 ]]; then summary_rows=0; fi
  fi
  if [[ -f "${TRACE_OUT_DIR}/grouped_checks.csv" ]]; then
    grouped_rows="$(($(wc -l < "${TRACE_OUT_DIR}/grouped_checks.csv") - 1))"
    if [[ "$grouped_rows" -lt 0 ]]; then grouped_rows=0; fi
  fi
  if [[ -d "${UI_OUT_DIR}/screenshots" ]]; then
    ui_screenshots="$(find "${UI_OUT_DIR}/screenshots" -type f -name '*.png' | wc -l | tr -d ' ')"
  fi

  cat > "$REPORT_MD" <<MD
# Runtime Analysis Report

## Overview
- Device: \`${DEVICE}\`
- Package: \`${PACKAGE}\`
- APK: \`${APK_PATH}\`
- Generated: \`$(date -Iseconds)\`

## Runtime Plan Execution
- Trace plane: completed via \`${TRACE_SCRIPT}\`
- Interaction plane: $( [[ "$SKIP_UI" -eq 0 ]] && echo "enabled" || echo "disabled" )
- Trace duration budget (total): ${TRACE_PHASE_SECONDS}s across fresh_launch + second_launch

## Artifacts
- Trace output: \`${TRACE_OUT_DIR}\`
- UI output: \`${UI_OUT_DIR}\`
- Command log: \`${COMMAND_LOG}\`
- Runner log: \`${RUN_LOG}\`

## Quick Metrics
- Parsed summary rows: ${summary_rows}
- Parsed grouped rows: ${grouped_rows}
- Captured screenshots: ${ui_screenshots}

## Notes
- Use \`${TRACE_OUT_DIR}/summary_metrics.csv\` and \`${TRACE_OUT_DIR}/grouped_checks.csv\` for deeper analysis.
- UI actions and screen captures are under \`${UI_OUT_DIR}\`.
MD

  cat > "$FINDINGS_JSON" <<JSON
{
  "generated_at": "$(date -Iseconds)",
  "device": "${DEVICE}",
  "package": "${PACKAGE}",
  "apk_path": "${APK_PATH}",
  "trace_out_dir": "${TRACE_OUT_DIR}",
  "ui_out_dir": "${UI_OUT_DIR}",
  "summary_metrics_csv": "${TRACE_OUT_DIR}/summary_metrics.csv",
  "grouped_checks_csv": "${TRACE_OUT_DIR}/grouped_checks.csv",
  "command_log_md": "${COMMAND_LOG}",
  "runner_log": "${RUN_LOG}"
}
JSON

  echo "[runtime] finished $(date -Iseconds)"
} | tee "$RUN_LOG"
