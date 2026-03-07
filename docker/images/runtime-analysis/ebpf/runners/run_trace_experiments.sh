#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  ./run_trace_experiments.sh [uid|auto] [options]

Runs two experiments and captures these traces inside eadb Debian chroot on device:
  - trace_fs.bt
  - trace_net.bt
  - trace_properties.bt
  - trace_runtime.bt

Experiments:
  1) fresh_launch   (first launch; run right after reinstall for best signal)
  2) second_launch  (force-stop + launch again)

Positional:
  uid|auto                    App UID (e.g. 10095) or auto (default: auto)

Options:
  --package <name>            Package name
  --device <serial>           adb serial (default: localhost:15555)
  --apk <path.apk>            Uninstall package, then install this APK before experiment 1
  --startup-wait <sec>        Wait before launch after traces start (default: 4)
  --phase-seconds <sec>       Total auto-stop budget across all phases (default: 0 = manual Ctrl+C)
  --out-dir <dir>             Local output dir (default: runs/trace_exp_<ts>_uid<uid>)
  --probes-dir <dir>          Local dir with trace_*.bt probes
                              (default: <repo>/backend/runtime/ebpf/probes)

  --sampling <0|1>            Trace sampling flag (default: 0)
  --stack <0|1>               Enable ustack in trace scripts (default: 1)
  --stack-depth <n>           10/20/40/80/120 (default: 80)
  --properties-key-only <0|1> trace_properties key-only mode (default: 0)

  --manual-launch             Do not launch app via adb; launch manually during each phase

  --transport <run|chroot>    Chroot transport:
                              run    -> use /data/eadb/run (default)
                              chroot -> use direct chroot command
  --bpftrace-bin <name|path>  bpftrace binary in chroot (default: bpftrace)
  --eadb-run <path>           Path to eadb runner (default: /data/eadb/run)
  --chroot-root <path>        Chroot root on device (default: /data/eadb/debian)
  --remote-workdir <path>     Dir in chroot containing trace_*.bt (default: /code)
  --remote-out-base <path>    Base dir in chroot for logs (default: /tmp)
  --no-push-traces            Do not push local trace_*.bt to device chroot workdir
  --no-pull                   Do not adb pull logs back to host

  -h, --help                  Show this help

Examples:
  ./run_trace_experiments.sh auto --device localhost:15555
  ./run_trace_experiments.sh auto --apk /path/to/app.apk --device localhost:15555
  ./run_trace_experiments.sh 10095 --stack 1 --stack-depth 80
  ./run_trace_experiments.sh auto --remote-workdir /code --manual-launch
USAGE
}

sq_escape() {
  printf "%s" "$1" | sed "s/'/'\"'\"'/g"
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

UID_TARGET="auto"
if [[ $# -gt 0 && "$1" != -* ]]; then
  UID_TARGET="$1"
  shift
fi

PACKAGE=""
DEVICE="localhost:15555"
APK_PATH=""
STARTUP_WAIT=4
PHASE_SECONDS=0
PHASE_SECONDS_FRESH=0
PHASE_SECONDS_SECOND=0
OUT_DIR=""
PROBES_DIR="${REPO_ROOT}/backend/runtime/ebpf/probes"
SAMPLING=0
ENABLE_STACK=1
STACK_DEPTH=80
PROPS_KEY_ONLY=0
MANUAL_LAUNCH=0

CHROOT_ROOT="/data/eadb/debian"
CHROOT_TRANSPORT="run"
EADB_RUN="/data/eadb/run"
BPFTRACE_BIN="bpftrace"
REMOTE_WORKDIR="/code"
REMOTE_OUT_BASE="/tmp"
NO_PUSH_TRACES=0
NO_PULL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --package)
      PACKAGE="$2"; shift 2 ;;
    --device)
      DEVICE="$2"; shift 2 ;;
    --apk)
      APK_PATH="$2"; shift 2 ;;
    --startup-wait)
      STARTUP_WAIT="$2"; shift 2 ;;
    --phase-seconds)
      PHASE_SECONDS="$2"; shift 2 ;;
    --out-dir)
      OUT_DIR="$2"; shift 2 ;;
    --probes-dir)
      PROBES_DIR="$2"; shift 2 ;;
    --sampling)
      SAMPLING="$2"; shift 2 ;;
    --stack)
      ENABLE_STACK="$2"; shift 2 ;;
    --stack-depth)
      STACK_DEPTH="$2"; shift 2 ;;
    --properties-key-only)
      PROPS_KEY_ONLY="$2"; shift 2 ;;
    --manual-launch)
      MANUAL_LAUNCH=1; shift ;;
    --transport)
      CHROOT_TRANSPORT="$2"; shift 2 ;;
    --bpftrace-bin)
      BPFTRACE_BIN="$2"; shift 2 ;;
    --eadb-run)
      EADB_RUN="$2"; shift 2 ;;
    --chroot-root)
      CHROOT_ROOT="$2"; shift 2 ;;
    --remote-workdir)
      REMOTE_WORKDIR="$2"; shift 2 ;;
    --remote-out-base)
      REMOTE_OUT_BASE="$2"; shift 2 ;;
    --no-push-traces)
      NO_PUSH_TRACES=1; shift ;;
    --no-pull)
      NO_PULL=1; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1 ;;
  esac
done

if ! command -v adb >/dev/null 2>&1; then
  echo "adb not found in PATH" >&2
  exit 1
fi

if ! [[ "$STARTUP_WAIT" =~ ^[0-9]+$ ]]; then
  echo "--startup-wait must be numeric" >&2
  exit 1
fi
if ! [[ "$PHASE_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "--phase-seconds must be numeric" >&2
  exit 1
fi
if [[ -n "$APK_PATH" && ! -f "$APK_PATH" ]]; then
  echo "--apk path does not exist: $APK_PATH" >&2
  exit 1
fi
if ! [[ "$SAMPLING" =~ ^[01]$ ]]; then
  echo "--sampling must be 0 or 1" >&2
  exit 1
fi
if ! [[ "$ENABLE_STACK" =~ ^[01]$ ]]; then
  echo "--stack must be 0 or 1" >&2
  exit 1
fi
if ! [[ "$PROPS_KEY_ONLY" =~ ^[01]$ ]]; then
  echo "--properties-key-only must be 0 or 1" >&2
  exit 1
fi
case "$STACK_DEPTH" in
  10|20|40|80|120) ;;
  *)
    echo "--stack-depth must be one of 10,20,40,80,120" >&2
    exit 1 ;;
esac
case "$CHROOT_TRANSPORT" in
  run|chroot) ;;
  *)
    echo "--transport must be run or chroot" >&2
    exit 1 ;;
esac
if [[ "${REMOTE_WORKDIR:0:1}" != "/" ]]; then
  echo "--remote-workdir must be an absolute path inside chroot (example: /code)" >&2
  exit 1
fi
if [[ "$NO_PUSH_TRACES" -eq 0 ]]; then
  if [[ ! -d "$PROBES_DIR" ]]; then
    echo "--probes-dir does not exist or is not a directory: $PROBES_DIR" >&2
    exit 1
  fi
  for f in trace_fs.bt trace_net.bt trace_properties.bt trace_runtime.bt; do
    if [[ ! -f "$PROBES_DIR/$f" ]]; then
      echo "missing probe in --probes-dir: $PROBES_DIR/$f" >&2
      exit 1
    fi
  done
fi

adb_cmd() {
  adb -s "$DEVICE" "$@"
}

adb_shell() {
  adb -s "$DEVICE" shell "$@"
}

adb_shell_capture() {
  adb -s "$DEVICE" shell "$@" 2>/dev/null | tr -d '\r'
}

adb_in_chroot() {
  local cmd="$1"
  case "$CHROOT_TRANSPORT" in
    run)
      # Run through /data/eadb/run to match interactive workflow used on-device.
      adb_shell "set -e; if [ ! -x '$EADB_RUN' ]; then echo \"eadb runner not executable: $EADB_RUN\" >&2; exit 1; fi; cat <<'__EADB_CMD__' | '$EADB_RUN'
set -euo pipefail
$cmd
exit
__EADB_CMD__"
      ;;
    chroot)
      local esc
      esc="$(sq_escape "$cmd")"
      adb_shell "chroot '$CHROOT_ROOT' /bin/bash -lc '$esc'"
      ;;
  esac
}

# Ensure adb root (best-effort)
echo "[init] ensuring adbd root on $DEVICE..."
adb_cmd root >/dev/null 2>&1 || true
sleep 1

if [[ -n "$APK_PATH" ]]; then
  if [[ "$UID_TARGET" != "auto" ]]; then
    echo "[init] explicit UID was provided, but --apk reinstall can change UID; forcing auto UID resolution."
    UID_TARGET="auto"
  fi
  echo "[init] reinstall requested via APK: $APK_PATH"
  echo "[init] uninstalling package '$PACKAGE' (ignore if not installed)..."
  adb_cmd uninstall "$PACKAGE" >/dev/null 2>&1 || true
  echo "[init] installing APK..."
  if ! install_out="$(adb_cmd install -r -d "$APK_PATH" 2>&1)"; then
    echo "[init] adb install failed:" >&2
    echo "$install_out" >&2
    exit 1
  fi
  echo "[init] install result: $(printf "%s" "$install_out" | tr -d '\r' | tail -n1)"
fi

if [[ "$UID_TARGET" == "auto" || -z "$UID_TARGET" ]]; then
  echo "[init] resolving UID for package '$PACKAGE'..."
  uid_line="$(adb_shell "cmd package list packages -U | grep -m1 '^package:${PACKAGE} uid:' || true" | tr -d '\r')"
  if [[ -n "${uid_line:-}" ]]; then
    echo "[init] package uid line: $uid_line"
    UID_TARGET="$(printf "%s\n" "$uid_line" | sed -n "s/^package:${PACKAGE} uid:\\([0-9][0-9]*\\)$/\\1/p" | head -n1)"
  else
    UID_TARGET=""
  fi
else
  echo "[init] using provided UID: $UID_TARGET"
fi

if ! [[ "$UID_TARGET" =~ ^[0-9]+$ ]]; then
  echo "Could not resolve UID for package '$PACKAGE'."
  echo "Try passing UID explicitly: ./run_trace_experiments.sh <uid> ..." >&2
  exit 1
fi
echo "[init] resolved UID: $UID_TARGET"

TS="$(date +%Y%m%d_%H%M%S)"
if [[ -z "$OUT_DIR" ]]; then
  OUT_DIR="runs/trace_exp_${TS}_uid${UID_TARGET}"
fi
mkdir -p "$OUT_DIR"

REMOTE_EXP_DIR="${REMOTE_OUT_BASE%/}/trace_exp_${TS}_uid${UID_TARGET}"
REMOTE_PULL_ROOT="${CHROOT_ROOT%/}${REMOTE_EXP_DIR}"
REMOTE_WORKDIR_HOST_PATH="${CHROOT_ROOT%/}${REMOTE_WORKDIR}"

push_trace_scripts() {
  adb_shell "mkdir -p '$REMOTE_WORKDIR_HOST_PATH'"
  for f in trace_fs.bt trace_net.bt trace_properties.bt trace_runtime.bt; do
    adb_cmd push "$PROBES_DIR/$f" "$REMOTE_WORKDIR_HOST_PATH/$f" >/dev/null
  done
}

# Validate remote environment and trace files.
if [[ "$NO_PUSH_TRACES" -eq 0 ]]; then
  echo "[init] pushing trace scripts to $REMOTE_WORKDIR_HOST_PATH..."
  push_trace_scripts
fi
echo "[init] validating bpftrace and trace scripts in chroot workdir '$REMOTE_WORKDIR'..."
if ! adb_in_chroot "set -euo pipefail; cd '$REMOTE_WORKDIR'; ls trace_fs.bt trace_net.bt trace_properties.bt trace_runtime.bt >/dev/null; if command -v '$BPFTRACE_BIN' >/dev/null 2>&1; then :; elif [ -x '$BPFTRACE_BIN' ]; then :; elif [ -x /usr/bin/bpftrace ]; then :; elif [ -x /usr/local/bin/bpftrace ]; then :; elif [ -x /bin/bpftrace ]; then :; else exit 127; fi"; then
  echo "[init] validation failed inside chroot." >&2
  echo "[init] hint: if PATH differs in /data/eadb/run, try --bpftrace-bin /usr/bin/bpftrace" >&2
  echo "[init] hint: you can also try --transport chroot for troubleshooting." >&2
  exit 1
fi
echo "[init] remote validation passed"

launch_app() {
  local phase="$1"
  local pid=""
  local launcher_component=""

  app_pid() {
    adb_shell_capture "pidof '$PACKAGE' || true" | awk '{print $1}'
  }

  wait_for_app_pid() {
    local timeout_s="$1"
    local i
    for ((i = 0; i < timeout_s; i++)); do
      pid="$(app_pid)"
      if [[ -n "${pid:-}" ]]; then
        return 0
      fi
      sleep 1
    done
    return 1
  }

  if [[ "$MANUAL_LAUNCH" -eq 1 ]]; then
    echo "[$phase] manual launch enabled; launch app now."
  else
    echo "[$phase] launching $PACKAGE..."
    adb_shell "am force-stop '$PACKAGE'" >/dev/null 2>&1 || true

    launcher_component="$(adb_shell_capture "cmd package resolve-activity --brief -a android.intent.action.MAIN -c android.intent.category.LAUNCHER '$PACKAGE' || true" | tail -n1)"

    if [[ "$launcher_component" == */* ]]; then
      echo "[$phase] launch attempt: am start -W -n $launcher_component"
      adb_shell "am start -W -n '$launcher_component'" >/dev/null 2>&1 || true
      if wait_for_app_pid 12; then
        echo "[$phase] app launched via component (pid=$pid)"
        return 0
      fi
      echo "[$phase] component launch did not produce a live pid"
    fi

    echo "[$phase] launch attempt: am start -W MAIN/LAUNCHER (package-scoped)"
    adb_shell "am start -W -a android.intent.action.MAIN -c android.intent.category.LAUNCHER -p '$PACKAGE'" >/dev/null 2>&1 || true
    if wait_for_app_pid 12; then
      echo "[$phase] app launched via am start (pid=$pid)"
      return 0
    fi
    echo "[$phase] am start did not produce a live pid"

    echo "[$phase] launch fallback: monkey -p"
    adb_shell "monkey -p '$PACKAGE' -c android.intent.category.LAUNCHER 1" >/dev/null 2>&1 || true
    if wait_for_app_pid 12; then
      echo "[$phase] app launched via monkey (pid=$pid)"
      return 0
    fi

    echo "[$phase] failed to launch $PACKAGE (no pid observed after all launch methods)" >&2
    return 1
  fi
}

capture_phase_remote() {
  local phase="$1"
  local phase_seconds="${2:-0}"
  local remote_phase_dir="${REMOTE_EXP_DIR%/}/${phase}"

  local remote_script
  remote_script=$(cat <<SCRIPT
set -euo pipefail
cd '$REMOTE_WORKDIR'
mkdir -p '$remote_phase_dir'

NOFILE_BEFORE="\$(ulimit -n 2>/dev/null || echo unknown)"
ulimit -n 120000 2>/dev/null || true
NOFILE_AFTER="\$(ulimit -n 2>/dev/null || echo unknown)"

BPFTRACE_BIN='$BPFTRACE_BIN'
if command -v "\$BPFTRACE_BIN" >/dev/null 2>&1; then
  BPFTRACE_BIN="\$(command -v "\$BPFTRACE_BIN")"
elif [ -x "\$BPFTRACE_BIN" ]; then
  :
elif [ -x /usr/bin/bpftrace ]; then
  BPFTRACE_BIN=/usr/bin/bpftrace
elif [ -x /usr/local/bin/bpftrace ]; then
  BPFTRACE_BIN=/usr/local/bin/bpftrace
elif [ -x /bin/bpftrace ]; then
  BPFTRACE_BIN=/bin/bpftrace
else
  echo "bpftrace not found in chroot; checked: $BPFTRACE_BIN, /usr/bin/bpftrace, /usr/local/bin/bpftrace, /bin/bpftrace" >&2
  exit 127
fi

pids=""
cleanup() {
  for p in \$pids; do kill -INT "\$p" 2>/dev/null || true; done
  for p in \$pids; do wait "\$p" 2>/dev/null || true; done
}
trap cleanup EXIT INT TERM

"\$BPFTRACE_BIN" trace_fs.bt '$UID_TARGET' '$SAMPLING' '$ENABLE_STACK' '$STACK_DEPTH' > '$remote_phase_dir/fs.log' 2>&1 &
pids="\$pids \$!"

"\$BPFTRACE_BIN" trace_net.bt '$UID_TARGET' '$SAMPLING' '$ENABLE_STACK' '$STACK_DEPTH' > '$remote_phase_dir/net.log' 2>&1 &
pids="\$pids \$!"

"\$BPFTRACE_BIN" trace_runtime.bt '$UID_TARGET' '$SAMPLING' '$ENABLE_STACK' '$STACK_DEPTH' > '$remote_phase_dir/runtime.log' 2>&1 &
pids="\$pids \$!"

"\$BPFTRACE_BIN" trace_properties.bt '$UID_TARGET' '$PROPS_KEY_ONLY' '$SAMPLING' '$ENABLE_STACK' '$STACK_DEPTH' > '$remote_phase_dir/properties.log' 2>&1 &
pids="\$pids \$!"

echo "uid=$UID_TARGET phase=$phase bpftrace=\$BPFTRACE_BIN pids=\$pids started_at=\$(date -Iseconds)" > '$remote_phase_dir/_remote_meta.txt'
echo "nofile_before=\$NOFILE_BEFORE nofile_after=\$NOFILE_AFTER" >> '$remote_phase_dir/_remote_meta.txt'
echo "phase_seconds=$phase_seconds" >> '$remote_phase_dir/_remote_meta.txt'
if [ "$phase_seconds" -gt 0 ]; then
  sleep "$phase_seconds"
  for p in \$pids; do kill -INT "\$p" 2>/dev/null || true; done
  for p in \$pids; do wait "\$p" 2>/dev/null || true; done
else
  wait
fi
SCRIPT
)

  adb_in_chroot "$remote_script"
}

pull_phase_logs() {
  local phase="$1"
  local local_phase_dir="$OUT_DIR/$phase"
  local remote_phase_pull_dir="${REMOTE_PULL_ROOT%/}/${phase}"

  mkdir -p "$local_phase_dir"
  # Pull contents into phase dir directly to avoid nested phase/phase folders.
  adb_cmd pull "${remote_phase_pull_dir}/." "$local_phase_dir" >/dev/null
}

write_meta() {
  cat > "$OUT_DIR/meta.txt" <<META
uid=$UID_TARGET
package=$PACKAGE
device=$DEVICE
apk_path=$APK_PATH
startup_wait=$STARTUP_WAIT
phase_seconds_total=$PHASE_SECONDS
phase_seconds_fresh=$PHASE_SECONDS_FRESH
phase_seconds_second=$PHASE_SECONDS_SECOND
sampling=$SAMPLING
enable_stack=$ENABLE_STACK
stack_depth=$STACK_DEPTH
properties_key_only=$PROPS_KEY_ONLY
manual_launch=$MANUAL_LAUNCH
probes_dir=$PROBES_DIR
transport=$CHROOT_TRANSPORT
bpftrace_bin=$BPFTRACE_BIN
eadb_run=$EADB_RUN
chroot_root=$CHROOT_ROOT
remote_workdir=$REMOTE_WORKDIR
remote_out_base=$REMOTE_OUT_BASE
remote_workdir_host_path=$REMOTE_WORKDIR_HOST_PATH
remote_exp_dir=$REMOTE_EXP_DIR
created_at=$(date -Iseconds)
META
}

run_phase() {
  local phase="$1"
  local phase_seconds="$2"
  local launcher_pid=""
  local rc=0
  local phase_interrupted=0

  echo "[$phase] starting remote captures in chroot..."
  if [[ "$MANUAL_LAUNCH" -eq 0 ]]; then
    (
      sleep "$STARTUP_WAIT"
      launch_app "$phase"
    ) &
    launcher_pid="$!"
  else
    launch_app "$phase"
  fi

  if [[ "$phase_seconds" -gt 0 ]]; then
    echo "[$phase] capturing... auto-stop after ${phase_seconds}s."
    set +e
    capture_phase_remote "$phase" "$phase_seconds"
    rc=$?
    set -e
  elif [[ "$PHASE_SECONDS" -gt 0 ]]; then
    echo "[$phase] skipping capture because total phase budget was exhausted."
    if [[ -n "$launcher_pid" ]]; then
      kill "$launcher_pid" 2>/dev/null || true
      wait "$launcher_pid" 2>/dev/null || true
    fi
    return 0
  else
    echo "[$phase] capturing... press Ctrl+C to stop this phase."
    trap 'phase_interrupted=1' INT
    set +e
    capture_phase_remote "$phase"
    rc=$?
    set -e
    trap - INT
  fi

  if [[ "$phase_interrupted" -eq 1 ]]; then
    if [[ "$rc" -eq 0 ]]; then
      rc=130
    fi
    echo "[$phase] Ctrl+C received; stopping current phase capture"
  fi

  if [[ -n "$launcher_pid" ]]; then
    if ! wait "$launcher_pid"; then
      echo "[$phase] launcher routine reported failure" >&2
      exit 1
    fi
  fi

  if [[ "$rc" -ne 0 && "$rc" -ne 130 && "$rc" -ne 143 ]]; then
    echo "[$phase] capture failed with exit code $rc" >&2
    exit "$rc"
  fi
  echo "[$phase] remote capture stopped"

  if [[ "$NO_PULL" -eq 0 ]]; then
    echo "[$phase] pulling logs..."
    pull_phase_logs "$phase"
    echo "[$phase] logs pulled"
  fi
}

if [[ "$PHASE_SECONDS" -gt 0 ]]; then
  if [[ "$PHASE_SECONDS" -eq 1 ]]; then
    PHASE_SECONDS_FRESH=1
    PHASE_SECONDS_SECOND=0
  else
    PHASE_SECONDS_FRESH=$((PHASE_SECONDS / 2))
    PHASE_SECONDS_SECOND=$((PHASE_SECONDS - PHASE_SECONDS_FRESH))
    if [[ "$PHASE_SECONDS_FRESH" -eq 0 ]]; then
      PHASE_SECONDS_FRESH=1
      PHASE_SECONDS_SECOND=$((PHASE_SECONDS - 1))
    fi
  fi
fi

write_meta

echo "UID: $UID_TARGET"
echo "Local out dir: $OUT_DIR"
echo "Remote out dir (in chroot): $REMOTE_EXP_DIR"
echo "Transport: $CHROOT_TRANSPORT"
if [[ "$PHASE_SECONDS" -gt 0 ]]; then
  echo "Phase budget total: ${PHASE_SECONDS}s (fresh_launch=${PHASE_SECONDS_FRESH}s, second_launch=${PHASE_SECONDS_SECOND}s)"
fi

echo "Experiment 1 (fresh_launch): run right after reinstall for best signal."
run_phase "fresh_launch" "$PHASE_SECONDS_FRESH"

echo "Experiment 2 (second_launch): force-stop + launch again."
run_phase "second_launch" "$PHASE_SECONDS_SECOND"

echo "All done."
if [[ "$NO_PULL" -eq 0 ]]; then
  echo "Local logs: $OUT_DIR"
else
  echo "Remote logs (chroot): $REMOTE_EXP_DIR"
fi
