# Runtime Orchestration Scripts

This directory contains scripts that coordinate runtime analysis collection inside the runtime-analysis container.

## Files

1. `run_runtime_analysis.sh`
    1. Orchestrates trace and UI interaction planes.
    2. Connects via ADB, installs APK, starts UI exploration, runs two-phase eBPF trace collection, and writes report
       artifacts.
2. `ui_explorer.py`
    1. Captures screenshots/UI dumps and performs basic input events (`tap`, `swipe`, `keyevent`, periodic `monkey`).
    2. Writes actions to `actions.jsonl` for correlation.
