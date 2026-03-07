#!/usr/bin/env python3
import argparse
import json
import random
import re
import signal
import subprocess
import time
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Optional


STOP_REQUESTED = False
BOUNDS_RE = re.compile(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]")
PACKAGE_COMPONENT_RE = re.compile(r"([A-Za-z0-9._]+)/([A-Za-z0-9.$_]+)")


def stop_handler(_signum, _frame):
    global STOP_REQUESTED
    STOP_REQUESTED = True


signal.signal(signal.SIGINT, stop_handler)
signal.signal(signal.SIGTERM, stop_handler)


def run(cmd, check=True, text=True, timeout=30):
    return subprocess.run(
        cmd,
        check=check,
        text=text,
        capture_output=True,
        timeout=timeout,
    )


def adb(device, *args, check=True, text=True, timeout=30):
    return run(["adb", "-s", device, *args], check=check, text=text, timeout=timeout)


def adb_shell(device, shell_cmd, check=True, timeout=30):
    return adb(device, "shell", shell_cmd, check=check, timeout=timeout)


def capture_screenshot(device, output_path):
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("wb") as fh:
        proc = subprocess.run(
            ["adb", "-s", device, "exec-out", "screencap", "-p"],
            stdout=fh,
            stderr=subprocess.PIPE,
            check=False,
            timeout=30,
        )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.decode("utf-8", errors="replace"))


def dump_ui_xml(device, output_path):
    remote_path = "/sdcard/nairi_window_dump.xml"
    adb_shell(
        device,
        f"uiautomator dump --compressed {remote_path} >/dev/null 2>&1 || true",
        check=False,
    )
    adb(device, "pull", remote_path, str(output_path), check=False)
    if not output_path.exists():
        raise RuntimeError("UI dump file was not pulled")


def parse_clickable_nodes(xml_path):
    nodes = []
    max_x = 0
    max_y = 0
    tree = ET.parse(xml_path)
    root = tree.getroot()
    for node in root.iter("node"):
        if node.attrib.get("clickable") != "true":
            continue
        if node.attrib.get("enabled") != "true":
            continue
        bounds = node.attrib.get("bounds", "")
        match = BOUNDS_RE.match(bounds)
        if not match:
            continue
        x1, y1, x2, y2 = [int(v) for v in match.groups()]
        if x2 <= x1 or y2 <= y1:
            continue
        max_x = max(max_x, x2)
        max_y = max(max_y, y2)
        nodes.append(
            {
                "x": (x1 + x2) // 2,
                "y": (y1 + y2) // 2,
                "area": (x2 - x1) * (y2 - y1),
                "resource_id": node.attrib.get("resource-id", ""),
                "text": node.attrib.get("text", ""),
                "class_name": node.attrib.get("class", ""),
                "package": node.attrib.get("package", ""),
                "bounds": bounds,
            }
        )

    if max_x > 0 and max_y > 0:
        filtered = []
        min_x = int(max_x * 0.05)
        max_x_allowed = int(max_x * 0.95)
        min_y = int(max_y * 0.08)
        max_y_allowed = int(max_y * 0.93)
        for node in nodes:
            if node["x"] < min_x or node["x"] > max_x_allowed:
                continue
            if node["y"] < min_y or node["y"] > max_y_allowed:
                continue
            filtered.append(node)
        if filtered:
            nodes = filtered

    nodes.sort(key=lambda value: value["area"], reverse=True)
    return nodes


def filter_nodes_by_package(nodes, target_package: str):
    return [node for node in nodes if node.get("package", "") == target_package]


def write_event(log_path, event):
    with log_path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(event, ensure_ascii=True) + "\n")


def extract_package_from_text(text: str) -> Optional[str]:
    for line in text.splitlines():
        match = PACKAGE_COMPONENT_RE.search(line)
        if match:
            return match.group(1)
    return None


def current_foreground_package(device: str) -> Optional[str]:
    window_out = adb_shell(
        device,
        "dumpsys window windows | grep -E 'mCurrentFocus|mFocusedApp' || true",
        check=False,
    ).stdout
    package_name = extract_package_from_text(window_out)
    if package_name:
        return package_name

    activity_out = adb_shell(
        device,
        "dumpsys activity activities | grep mResumedActivity || true",
        check=False,
    ).stdout
    return extract_package_from_text(activity_out)


def resolve_launcher_component(device: str, package_name: str) -> Optional[str]:
    output = adb_shell(
        device,
        f"cmd package resolve-activity --brief -a android.intent.action.MAIN -c android.intent.category.LAUNCHER '{package_name}' || true",
        check=False,
    ).stdout
    for line in reversed(output.splitlines()):
        candidate = line.strip()
        if "/" in candidate:
            return candidate
    return None


def launch_package(device: str, package_name: str, launcher_component: Optional[str]) -> Optional[str]:
    if launcher_component:
        adb_shell(
            device,
            f"am start -W -n '{launcher_component}' >/dev/null 2>&1 || true",
            check=False,
        )
    else:
        adb_shell(
            device,
            f"am start -W -a android.intent.action.MAIN -c android.intent.category.LAUNCHER -p '{package_name}' >/dev/null 2>&1 || true",
            check=False,
        )
    current = current_foreground_package(device)
    if current == package_name:
        return launcher_component

    adb_shell(
        device,
        f"monkey -p '{package_name}' -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1 || true",
        check=False,
    )
    return launcher_component


def ensure_target_foreground(device: str, package_name: str, launcher_component: Optional[str]) -> tuple[bool, Optional[str], Optional[str]]:
    focused = current_foreground_package(device)
    if focused == package_name:
        return True, focused, launcher_component

    if focused and focused != package_name and not focused.startswith("com.android.systemui"):
        adb_shell(device, f"am force-stop '{focused}' >/dev/null 2>&1 || true", check=False)

    if not launcher_component:
        launcher_component = resolve_launcher_component(device, package_name)

    launcher_component = launch_package(device, package_name, launcher_component)
    time.sleep(1.0)
    focused = current_foreground_package(device)
    return focused == package_name, focused, launcher_component


def main():
    parser = argparse.ArgumentParser(
        description="Simple runtime UI explorer that captures screenshots/UI dumps and performs input events."
    )
    parser.add_argument("--device", required=True, help="adb device serial")
    parser.add_argument("--package", required=True, help="Android package name")
    parser.add_argument("--out-dir", required=True, help="Output directory")
    parser.add_argument("--steps", type=int, default=90, help="Maximum exploration steps")
    parser.add_argument("--interval-sec", type=float, default=2.0, help="Delay between actions")
    parser.add_argument("--startup-delay-sec", type=float, default=3.0, help="Delay before first action")
    parser.add_argument(
        "--monkey-every",
        type=int,
        default=0,
        help="Run a small monkey burst every N steps (0 disables monkey)",
    )
    parser.add_argument(
        "--strict-package",
        dest="strict_package",
        action="store_true",
        default=True,
        help="Only tap clickable nodes that belong to target package (default: enabled).",
    )
    parser.add_argument(
        "--allow-cross-package",
        dest="strict_package",
        action="store_false",
        help="Allow tapping clickable nodes from any package.",
    )
    parser.add_argument("--seed", type=int, default=1337, help="Random seed")
    args = parser.parse_args()

    random.seed(args.seed)

    out_dir = Path(args.out_dir)
    screenshots_dir = out_dir / "screenshots"
    dumps_dir = out_dir / "ui-dumps"
    screenshots_dir.mkdir(parents=True, exist_ok=True)
    dumps_dir.mkdir(parents=True, exist_ok=True)
    events_log = out_dir / "actions.jsonl"

    write_event(
        events_log,
        {
            "event": "ui_explorer_start",
            "device": args.device,
            "package": args.package,
            "steps": args.steps,
            "interval_sec": args.interval_sec,
            "monkey_every": args.monkey_every,
        },
    )

    launcher_component = resolve_launcher_component(args.device, args.package)
    launch_package(args.device, args.package, launcher_component)
    time.sleep(args.startup_delay_sec)

    recent_taps = []
    for step in range(1, args.steps + 1):
        if STOP_REQUESTED:
            break

        now = time.time()
        screenshot_path = screenshots_dir / f"step_{step:04d}.png"
        ui_dump_path = dumps_dir / f"step_{step:04d}.xml"
        step_event = {
            "event": "ui_step",
            "step": step,
            "timestamp": now,
            "screenshot": str(screenshot_path),
            "ui_dump": str(ui_dump_path),
        }

        in_target, focused_pkg, launcher_component = ensure_target_foreground(
            args.device, args.package, launcher_component
        )
        step_event["focused_package"] = focused_pkg
        if not in_target:
            step_event["action"] = {
                "type": "recover_target",
                "status": "failed",
            }
            write_event(events_log, step_event)
            time.sleep(args.interval_sec)
            continue

        try:
            capture_screenshot(args.device, screenshot_path)
        except Exception as exc:  # noqa: BLE001
            step_event["screenshot_error"] = str(exc)

        clickable_nodes = []
        try:
            dump_ui_xml(args.device, ui_dump_path)
            clickable_nodes = parse_clickable_nodes(ui_dump_path)
            if args.strict_package:
                clickable_nodes = filter_nodes_by_package(clickable_nodes, args.package)
        except Exception as exc:  # noqa: BLE001
            step_event["ui_dump_error"] = str(exc)

        action = {"type": "none"}
        if args.monkey_every > 0 and step % args.monkey_every == 0:
            adb_shell(
                args.device,
                f"monkey -p '{args.package}' -c android.intent.category.LAUNCHER --throttle 120 12",
                check=False,
                timeout=60,
            )
            action = {"type": "monkey_burst", "events": 12}
        elif clickable_nodes:
            candidates = clickable_nodes[: min(12, len(clickable_nodes))]
            random.shuffle(candidates)
            selected = None
            for candidate in candidates:
                coord = f"{candidate['x']}:{candidate['y']}"
                if coord in recent_taps:
                    continue
                selected = candidate
                break
            if selected is None:
                selected = candidates[0]
            adb_shell(
                args.device,
                f"input tap {selected['x']} {selected['y']}",
                check=False,
            )
            coord = f"{selected['x']}:{selected['y']}"
            recent_taps.append(coord)
            recent_taps = recent_taps[-12:]
            action = {
                "type": "tap",
                "x": selected["x"],
                "y": selected["y"],
                "resource_id": selected["resource_id"],
                "text": selected["text"],
                "class_name": selected["class_name"],
                "bounds": selected["bounds"],
            }
        else:
            adb_shell(args.device, "input keyevent KEYCODE_BACK", check=False)
            action = {"type": "keyevent", "value": "KEYCODE_BACK"}

        step_event["clickable_nodes"] = len(clickable_nodes)
        step_event["action"] = action
        write_event(events_log, step_event)

        slept = 0.0
        while slept < args.interval_sec and not STOP_REQUESTED:
            time.sleep(0.2)
            slept += 0.2

    write_event(events_log, {"event": "ui_explorer_stop", "timestamp": time.time()})


if __name__ == "__main__":
    main()
