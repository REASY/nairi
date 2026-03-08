# NAIRI System Architecture

## 1. Components

1. **Frontend Application (TypeScript + React)**
   1. Provides APK upload and analysis start UI.
   2. Provides configuration UI for runtime/static infrastructure settings.
   3. Shows live stage progress and final reports.
2. **Backend API (Rust + Axum)**
   1. Exposes APIs for sample intake, run control, and report retrieval.
   2. Hosts orchestration endpoints and event streams.
3. **Configuration Service**
   1. Stores active configuration profiles.
   2. Validates configured static-analysis Docker image and ADB connection target.
   3. Resolves immutable run configuration snapshots.
4. **Sample Intake**
   1. Receives APK and metadata.
   2. Computes hashes and deduplicates samples.
5. **AI Agent Controller**
   1. Builds run plan from sample and policy profile.
   2. Orchestrates stage execution and branch decisions.
   3. Logs decisions, rationale summaries, and outcomes.
6. **Analysis Orchestrator**
   1. Creates and schedules analysis runs.
   2. Coordinates static and runtime stages.
7. **Static Analysis Engine**
   1. Uses `apktool` for decompile and manifest extraction.
   2. Uses headless `Ghidra` for native library analysis.
   3. Builds deterministic static evidence graph (smali + native) for rules and AI reasoning.
8. **Runtime Analysis Engine**
   1. Starts isolated `redroid` instance.
   2. Installs and launches app.
   3. Drives app behavior stimulation.
9. **Instrumentation Layer**
   1. Frida Java/native hooks.
   2. eBPF probes for kernel-level telemetry.
10. **Network Inspection Layer**
   1. MITM proxy routing.
   2. TLS/HTTP capture under policy.
11. **Patch/Rebuild Service**
   1. Pinning bypass attempts.
   2. Rebuild, resign, redeploy workflow.
12. **Reporting and Intelligence Service**
   1. Correlation and risk scoring.
   2. Human + machine output.
13. **Evidence Store**
   1. Immutable artifacts and logs.

## 2. Architecture Flow

1. User configures required settings in frontend:
   1. Static analysis Docker image (`apktool` + `Ghidra` pre-installed).
   2. ADB device connection string.
2. Backend API persists configuration and triggers validation through Configuration Service.
3. User uploads APK in frontend and starts `Analyse`.
4. Backend API receives intake request and registers `sample_id`.
5. AI Agent Controller resolves active run configuration snapshot and creates run plan.
6. Orchestrator runs static stage in configured Docker image, builds static evidence graph, and emits baseline
   indicators.
7. Runtime stage connects via configured ADB target, boots sandbox, and executes dynamic instrumentation.
8. MITM layer captures network behavior.
9. If pinning blocks inspection, AI agent triggers bypass flow and runtime stage repeats.
10. Report service correlates evidence into final intelligence output.
11. Backend streams status and results to frontend UI.

## 3. Deployment Constraints

1. One isolated emulator environment per run.
2. redroid image must include required eBPF support.
3. Runtime instrumentation must not require internet access outside controlled routes.
4. All outputs are persisted with content hashes for reproducibility.
