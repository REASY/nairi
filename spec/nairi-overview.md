# NAIRI Overview and Scope

## 1. Project

**NAIRI** means **Native Android Inspection & Risk Intelligence**.

NAIRI is an automated malware analysis system for Android applications, focused on combining:

1. Static reverse engineering.
2. Runtime instrumentation in sandboxed emulators.
3. Correlated risk intelligence and investigation artifacts.

## 2. Primary Goals

1. Analyze APKs statically with `apktool` and native binary analysis.
2. Analyze runtime behavior with `Frida` and `eBPF` inside sandboxed `redroid`.
3. Produce analyst reports for:
   1. Native libraries and method activity.
   2. File-system, Android property, and syscall behavior.
   3. HTTPS behavior through MITM interception.
   4. Certificate pinning bypass and post-bypass behavior.
4. Deliver autonomous, single-action analysis where users upload an APK and press `Analyse`.
5. Provide operator configuration for required analysis infrastructure inputs:
   1. Static-analysis Docker image (`apktool` + `Ghidra`).
   2. ADB connection string for Android target.

## 3. Non-Goals (Initial Version)

1. Full automatic malware family clustering.
2. Universal unpacking for all commercial protectors.
3. Production-grade physical-device farm support.

## 4. Scope Boundary

In scope:

1. APK ingestion to final report pipeline.
2. Static + dynamic evidence correlation.
3. Pinning bypass workflow for analysis visibility.
4. AI agent orchestration and decision logging for full-run automation.

Out of scope:

1. Endpoint remediation or live enterprise enforcement.
2. End-user incident response playbook automation.
