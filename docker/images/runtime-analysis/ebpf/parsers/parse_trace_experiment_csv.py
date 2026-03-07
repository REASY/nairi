#!/usr/bin/env python3
"""
Export trace experiment results into TWO CSV files:

1) summary_metrics.csv
   High-level metrics per phase.

2) grouped_checks.csv
   Grouped rows by (operation, path/key, result/value), per phase.
   This is intended for detailed check analysis.

Input layout:
  <experiment_dir>/
    fresh_launch/{fs.log,net.log,properties.log,runtime.log}
    second_launch/{fs.log,net.log,properties.log,runtime.log}

Usage:
  python3 parse_trace_experiment_csv.py runs/trace_exp_20260303_173432_uid10100
  python3 parse_trace_experiment_csv.py runs/trace_exp_20260303_173432_uid10100 \
      --summary-out summary.csv --grouped-out grouped.csv
"""

from __future__ import annotations

import argparse
import csv
import re
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

# FS (new format with status)
FS_EVENT_NEW_RE = re.compile(
    r"^\[obs-fs\]\[(?P<seq>\d+)\]\s+"
    r"(?P<op>\w+)\s+uid=(?P<uid>\d+)\s+pid=(?P<pid>\d+)\s+tid=(?P<tid>\d+)\s+"
    r"comm=(?P<comm>.+?)\s+path=(?P<path>.*?)\s+(?:rc=(?P<rc>-?\d+)|ret=(?P<ret>0x[0-9a-fA-F]+))\s+ok=(?P<ok>[01])$"
)
# FS (old format attempt-only)
FS_EVENT_OLD_RE = re.compile(
    r"^\[obs-fs\]\[(?P<seq>\d+)\]\s+"
    r"(?P<op>\w+)\s+uid=(?P<uid>\d+)\s+pid=(?P<pid>\d+)\s+tid=(?P<tid>\d+)\s+"
    r"comm=(?P<comm>.+?)\s+path=(?P<path>.*)$"
)
FS_DONE_RE = re.compile(r"^\[obs-fs\]\s+done\s+uid=(?P<uid>\d+)\s+fs_events=(?P<count>\d+)$")

NET_SOCK_RE = re.compile(
    r"^\[obs-net\]\[sock#(?P<seq>\d+)\]\s+uid=(?P<uid>\d+)\s+pid=(?P<pid>\d+)\s+tid=(?P<tid>\d+)\s+"
    r"comm=(?P<comm>.+?)\s+fd=(?P<fd>-?\d+)\s+domain=(?P<domain>-?\d+)\s+type=(?P<type>-?\d+)\s+proto=(?P<proto>-?\d+)$"
)
NET_CONN_RE = re.compile(
    r"^\[obs-net\]\[conn#(?P<seq>\d+)\]\s+uid=(?P<uid>\d+)\s+pid=(?P<pid>\d+)\s+tid=(?P<tid>\d+)\s+"
    r"comm=(?P<comm>.+?)\s+fd=(?P<fd>-?\d+)\s+domain=(?P<domain>-?\d+)\s+type=(?P<type>-?\d+)\s+proto=(?P<proto>-?\d+)\s+addr_ptr=(?P<addr>\S+)$"
)
NET_DONE_RE = re.compile(
    r"^\[obs-net\]\s+done\s+uid=(?P<uid>\d+)\s+sockets=(?P<sockets>\d+)\s+connects=(?P<connects>\d+)$"
)

PROP_EVENT_RE = re.compile(
    r"^\[obs-prop\]\[(?P<seq>\d+)\]\s+uid=(?P<uid>\d+)\s+pid=(?P<pid>\d+)\s+tid=(?P<tid>\d+)\s+"
    r"comm=(?P<comm>.+?)\s+key=(?P<key>\S+)"
    r"(?:\s+len=(?P<len>-?\d+)(?:\s+val=(?P<val>.*))?)?$"
)
PROP_DONE_RE = re.compile(
    r"^\[obs-prop\]\s+done\s+uid=(?P<uid>\d+)\s+property_calls=(?P<calls>\d+)\s+value_calls=(?P<vcalls>\d+)$"
)

RT_PRTCL_RE = re.compile(
    r"^\[obs-rt\]\[prctl#(?P<seq>\d+)\]\s+uid=(?P<uid>\d+)\s+pid=(?P<pid>\d+)\s+tid=(?P<tid>\d+)\s+"
    r"comm=(?P<comm>.+?)\s+option=(?P<option>\S+)\s+arg2=(?P<arg2>\S+)\s+arg3=(?P<arg3>\S+)$"
)
RT_SYSCALL_RE = re.compile(
    r"^\[obs-rt\]\[syscall#(?P<seq>\d+)\]\s+uid=(?P<uid>\d+)\s+pid=(?P<pid>\d+)\s+tid=(?P<tid>\d+)\s+"
    r"comm=(?P<comm>.+?)\s+nr=(?P<nr>\S+)\s+arg1=(?P<arg1>\S+)\s+arg2=(?P<arg2>\S+)$"
)
RT_UNAME_RE = re.compile(
    r"^\[obs-rt\]\[uname#(?P<seq>\d+)\]\s+uid=(?P<uid>\d+)\s+pid=(?P<pid>\d+)\s+tid=(?P<tid>\d+)\s+"
    r"comm=(?P<comm>.+?)\s+buf=(?P<buf>\S+)$"
)
RT_DONE_RE = re.compile(
    r"^\[obs-rt\]\s+done\s+uid=(?P<uid>\d+)\s+prctl=(?P<prctl>\d+)\s+uname=(?P<uname>\d+)\s+syscall=(?P<syscall>\d+)$"
)

EMU_PATH_TOKENS = (
    "qemu",
    "genyd",
    "goldfish",
    "emulator",
    "vbox",
    "nox",
    "ldplayer",
)

ROOT_TAMPER_PATH_TOKENS = (
    "/su",
    "magisk",
    "zygisk",
    "xposed",
    "riru",
    "frida",
    "substrate",
    "busybox",
    "supersu",
    "superuser",
)

EMU_PROP_TOKENS = (
    "qemu",
    "goldfish",
    "genyd",
    "sdk_gphone",
    "emulator",
    "vbox",
)

ROOT_TAMPER_PROP_TOKENS = (
    "magisk",
    "zygisk",
    "xposed",
    "frida",
    "substrate",
    "supersu",
    "superuser",
    "ro.secure",
    "ro.debuggable",
    "ro.boot.verifiedbootstate",
    "ro.boot.vbmeta.device_state",
    "service.adb.root",
)


def _contains_any(value: str, tokens: tuple[str, ...]) -> bool:
    lv = value.lower()
    return any(t in lv for t in tokens)


@dataclass
class Summary:
    fs_total: int = 0
    fs_ok: int = 0
    fs_fail: int = 0
    fs_unknown: int = 0
    fs_reported: Optional[int] = None

    fs_emu_total: int = 0
    fs_emu_ok: int = 0
    fs_emu_fail: int = 0
    fs_emu_unknown: int = 0

    fs_root_total: int = 0
    fs_root_ok: int = 0
    fs_root_fail: int = 0
    fs_root_unknown: int = 0

    prop_total: int = 0
    prop_len_gt0: int = 0
    prop_len_eq0: int = 0
    prop_len_missing: int = 0
    prop_reported: Optional[int] = None
    prop_reported_values: Optional[int] = None

    prop_emu_total: int = 0
    prop_root_total: int = 0

    net_sockets: int = 0
    net_connects: int = 0
    net_sockets_reported: Optional[int] = None
    net_connects_reported: Optional[int] = None

    rt_prctl: int = 0
    rt_uname: int = 0
    rt_syscall: int = 0
    rt_prctl_reported: Optional[int] = None
    rt_uname_reported: Optional[int] = None
    rt_syscall_reported: Optional[int] = None


def parse_phase(phase: str, phase_dir: Path):
    grouped = Counter()
    summary = Summary()

    # fs
    with (phase_dir / "fs.log").open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.rstrip("\r\n")

            m = FS_EVENT_NEW_RE.match(line)
            if m:
                op = m.group("op")
                path = m.group("path")
                ok = int(m.group("ok"))
                rc = m.group("rc")
                ret = m.group("ret")

                result = "ok" if ok == 1 else "fail"
                value = f"rc={rc}" if rc is not None else (f"ret={ret}" if ret is not None else "")
                grouped[(phase, "fs", op, path, result, value)] += 1

                summary.fs_total += 1
                if ok == 1:
                    summary.fs_ok += 1
                else:
                    summary.fs_fail += 1

                if _contains_any(path, EMU_PATH_TOKENS):
                    summary.fs_emu_total += 1
                    if ok == 1:
                        summary.fs_emu_ok += 1
                    else:
                        summary.fs_emu_fail += 1
                    grouped[(phase, "signal", op, path, result, value)] += 1

                if _contains_any(path, ROOT_TAMPER_PATH_TOKENS):
                    summary.fs_root_total += 1
                    if ok == 1:
                        summary.fs_root_ok += 1
                    else:
                        summary.fs_root_fail += 1
                    grouped[(phase, "signal", op, path, result, value)] += 1
                continue

            m = FS_EVENT_OLD_RE.match(line)
            if m:
                op = m.group("op")
                path = m.group("path")
                grouped[(phase, "fs", op, path, "unknown", "") ] += 1

                summary.fs_total += 1
                summary.fs_unknown += 1

                if _contains_any(path, EMU_PATH_TOKENS):
                    summary.fs_emu_total += 1
                    summary.fs_emu_unknown += 1
                    grouped[(phase, "signal", op, path, "unknown", "")] += 1
                if _contains_any(path, ROOT_TAMPER_PATH_TOKENS):
                    summary.fs_root_total += 1
                    summary.fs_root_unknown += 1
                    grouped[(phase, "signal", op, path, "unknown", "")] += 1
                continue

            m = FS_DONE_RE.match(line)
            if m:
                summary.fs_reported = int(m.group("count"))

    # properties
    with (phase_dir / "properties.log").open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.rstrip("\r\n")
            m = PROP_EVENT_RE.match(line)
            if m:
                key = m.group("key")
                ln = m.group("len")
                val = m.group("val")

                summary.prop_total += 1
                op = "__system_property_get"

                if ln is None:
                    result = "len_missing"
                    value = ""
                    summary.prop_len_missing += 1
                else:
                    ilen = int(ln)
                    if ilen > 0:
                        result = "len>0"
                        summary.prop_len_gt0 += 1
                    elif ilen == 0:
                        result = "len=0"
                        summary.prop_len_eq0 += 1
                    else:
                        result = "len<0"
                    value = "" if val is None else val

                grouped[(phase, "properties", op, key, result, value)] += 1

                if _contains_any(key, EMU_PROP_TOKENS):
                    summary.prop_emu_total += 1
                    grouped[(phase, "signal", op, key, result, value)] += 1
                if _contains_any(key, ROOT_TAMPER_PROP_TOKENS):
                    summary.prop_root_total += 1
                    grouped[(phase, "signal", op, key, result, value)] += 1
                continue

            m = PROP_DONE_RE.match(line)
            if m:
                summary.prop_reported = int(m.group("calls"))
                summary.prop_reported_values = int(m.group("vcalls"))

    # net
    with (phase_dir / "net.log").open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.rstrip("\r\n")
            m = NET_SOCK_RE.match(line)
            if m:
                summary.net_sockets += 1
                grouped[(phase, "net", "socket", f"domain={m.group('domain')}", "seen", "") ] += 1
                continue
            m = NET_CONN_RE.match(line)
            if m:
                summary.net_connects += 1
                grouped[(phase, "net", "connect", f"domain={m.group('domain')}", "seen", "") ] += 1
                continue
            m = NET_DONE_RE.match(line)
            if m:
                summary.net_sockets_reported = int(m.group("sockets"))
                summary.net_connects_reported = int(m.group("connects"))

    # runtime
    with (phase_dir / "runtime.log").open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.rstrip("\r\n")
            m = RT_PRTCL_RE.match(line)
            if m:
                summary.rt_prctl += 1
                grouped[(phase, "runtime", "prctl", m.group("option"), "seen", "")] += 1
                continue
            m = RT_SYSCALL_RE.match(line)
            if m:
                summary.rt_syscall += 1
                grouped[(phase, "runtime", "syscall", m.group("nr"), "seen", "")] += 1
                continue
            m = RT_UNAME_RE.match(line)
            if m:
                summary.rt_uname += 1
                grouped[(phase, "runtime", "uname", "uname", "seen", "")] += 1
                continue
            m = RT_DONE_RE.match(line)
            if m:
                summary.rt_prctl_reported = int(m.group("prctl"))
                summary.rt_uname_reported = int(m.group("uname"))
                summary.rt_syscall_reported = int(m.group("syscall"))

    return summary, grouped


def write_summary_csv(out_path: Path, rows: list[dict[str, str | int]]) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["phase", "metric", "count"])
        w.writeheader()
        for row in rows:
            w.writerow(row)


def write_grouped_csv(out_path: Path, grouped: Counter) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(
            f,
            fieldnames=["phase", "source", "operation", "target", "result", "value", "count"],
        )
        w.writeheader()
        for (phase, source, op, target, result, value), cnt in sorted(
            grouped.items(),
            key=lambda x: (x[0][0], x[0][1], x[0][2], x[0][3], x[0][4], x[0][5]),
        ):
            w.writerow(
                {
                    "phase": phase,
                    "source": source,
                    "operation": op,
                    "target": target,
                    "result": result,
                    "value": value,
                    "count": cnt,
                }
            )


def summary_to_rows(phase: str, s: Summary) -> list[dict[str, str | int]]:
    metrics = {
        "fs_total": s.fs_total,
        "fs_ok": s.fs_ok,
        "fs_fail": s.fs_fail,
        "fs_unknown": s.fs_unknown,
        "fs_reported": "" if s.fs_reported is None else s.fs_reported,
        "fs_emulator_total": s.fs_emu_total,
        "fs_emulator_ok": s.fs_emu_ok,
        "fs_emulator_fail": s.fs_emu_fail,
        "fs_emulator_unknown": s.fs_emu_unknown,
        "fs_root_tamper_total": s.fs_root_total,
        "fs_root_tamper_ok": s.fs_root_ok,
        "fs_root_tamper_fail": s.fs_root_fail,
        "fs_root_tamper_unknown": s.fs_root_unknown,
        "properties_total": s.prop_total,
        "properties_len_gt0": s.prop_len_gt0,
        "properties_len_eq0": s.prop_len_eq0,
        "properties_len_missing": s.prop_len_missing,
        "properties_reported": "" if s.prop_reported is None else s.prop_reported,
        "properties_values_reported": "" if s.prop_reported_values is None else s.prop_reported_values,
        "properties_emulator_total": s.prop_emu_total,
        "properties_root_tamper_total": s.prop_root_total,
        "net_sockets_total": s.net_sockets,
        "net_connects_total": s.net_connects,
        "net_sockets_reported": "" if s.net_sockets_reported is None else s.net_sockets_reported,
        "net_connects_reported": "" if s.net_connects_reported is None else s.net_connects_reported,
        "runtime_prctl_total": s.rt_prctl,
        "runtime_uname_total": s.rt_uname,
        "runtime_syscall_total": s.rt_syscall,
        "runtime_prctl_reported": "" if s.rt_prctl_reported is None else s.rt_prctl_reported,
        "runtime_uname_reported": "" if s.rt_uname_reported is None else s.rt_uname_reported,
        "runtime_syscall_reported": "" if s.rt_syscall_reported is None else s.rt_syscall_reported,
    }
    return [{"phase": phase, "metric": k, "count": v} for k, v in metrics.items()]


def main() -> None:
    ap = argparse.ArgumentParser(description="Export summary + grouped CSVs from trace experiment logs")
    ap.add_argument("experiment_dir", type=Path, help="Path to runs/trace_exp_* directory")
    ap.add_argument("--summary-out", type=Path, help="Output CSV for summary metrics")
    ap.add_argument("--grouped-out", type=Path, help="Output CSV for grouped checks")
    args = ap.parse_args()

    exp = args.experiment_dir
    fresh_dir = exp / "fresh_launch"
    second_dir = exp / "second_launch"
    if not fresh_dir.exists() or not second_dir.exists():
        raise SystemExit(f"Missing fresh_launch/second_launch under: {exp}")

    summary_out = args.summary_out or (exp / "summary_metrics.csv")
    grouped_out = args.grouped_out or (exp / "grouped_checks.csv")

    fresh_summary, fresh_grouped = parse_phase("fresh_launch", fresh_dir)
    second_summary, second_grouped = parse_phase("second_launch", second_dir)

    summary_rows = []
    summary_rows.extend(summary_to_rows("fresh_launch", fresh_summary))
    summary_rows.extend(summary_to_rows("second_launch", second_summary))

    grouped = Counter()
    grouped.update(fresh_grouped)
    grouped.update(second_grouped)

    write_summary_csv(summary_out, summary_rows)
    write_grouped_csv(grouped_out, grouped)

    print(f"Wrote summary CSV: {summary_out}")
    print(f"Wrote grouped CSV: {grouped_out}")
    print(f"Summary rows: {len(summary_rows)}")
    print(f"Grouped rows: {len(grouped)}")


if __name__ == "__main__":
    main()
