---
name: adb
description: Android Debug Bridge skill for connecting to devices/emulators and collecting runtime artifacts.
---

# ADB Runtime Analysis Skill

Use this skill when runtime analysis needs Android device access.

## Preconditions

1. `adb` is installed and on `PATH`.
2. `ADB_CONNECTION_STRING` is provided (for remote device/emulator), for example `host.docker.internal:5555`.

## Connection Workflow

1. Start server:

```bash
adb start-server
```

2. Connect (if `ADB_CONNECTION_STRING` is set):

```bash
adb connect "${ADB_CONNECTION_STRING}"
```

3. Verify targets:

```bash
adb devices -l
```

## Common Runtime Collection Commands

List packages:

```bash
adb shell pm list packages
```

Find app process:

```bash
adb shell ps -A | grep -i "<package_or_process>"
```

Logcat (full):

```bash
adb logcat -d
```

Filtered logcat:

```bash
adb logcat -d | grep -Ei "<package>|frida|native|ssl|pinning"
```

Pull files from device:

```bash
adb pull /data/local/tmp ./device-artifacts
```

Run a shell command:

```bash
adb shell "<command>"
```

## Reporting Requirements

Always include in report:

1. `adb version`
2. Connection target used (`ADB_CONNECTION_STRING`)
3. `adb devices -l` output summary
4. Exact runtime commands executed
5. Any command failures with stderr

