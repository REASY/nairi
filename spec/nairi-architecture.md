# NAIRI System Architecture

## 1. Components

1. **Frontend Application (TypeScript + React)**
   1. Provides APK upload and analysis start UI.
   2. Shows live stage progress and final reports.
2. **Backend API (Rust + Axum)**
   1. Exposes APIs for sample intake, run control, and report retrieval.
   2. Hosts orchestration endpoints and event streams.
3. **Sample Intake**
   1. Receives APK and metadata.
   2. Computes hashes and deduplicates samples.
4. **AI Agent Controller**
   1. Builds run plan from sample and policy profile.
   2. Orchestrates stage execution and branch decisions.
   3. Logs decisions, rationale summaries, and outcomes.
5. **Analysis Orchestrator**
   1. Creates and schedules analysis runs.
   2. Coordinates static and runtime stages.
6. **Static Analysis Engine**
   1. Uses `apktool` for decompile and manifest extraction.
   2. Uses headless `Ghidra` for native library analysis.
7. **Runtime Analysis Engine**
   1. Starts isolated `redroid` instance.
   2. Installs and launches app.
   3. Drives app behavior stimulation.
8. **Instrumentation Layer**
   1. Frida Java/native hooks.
   2. eBPF probes for kernel-level telemetry.
9. **Network Inspection Layer**
   1. MITM proxy routing.
   2. TLS/HTTP capture under policy.
10. **Patch/Rebuild Service**
   1. Pinning bypass attempts.
   2. Rebuild, resign, redeploy workflow.
11. **Reporting and Intelligence Service**
   1. Correlation and risk scoring.
   2. Human + machine output.
12. **Evidence Store**
   1. Immutable artifacts and logs.

## 2. Architecture Flow

1. User uploads APK in frontend (TypeScript + React) and starts `Analyse`.
2. Backend API (Rust + Axum) receives intake request and registers `sample_id`.
3. AI Agent Controller creates run plan and stage graph.
4. Orchestrator runs static stage and emits baseline indicators.
5. Runtime stage boots new sandbox and executes dynamic instrumentation.
6. MITM layer captures network behavior.
7. If pinning blocks inspection, AI agent triggers bypass flow and runtime stage repeats.
8. Report service correlates evidence into final intelligence output.
9. Backend streams status and results to frontend UI.

## 3. Deployment Constraints

1. One isolated emulator environment per run.
2. redroid image must include required eBPF support.
3. Runtime instrumentation must not require internet access outside controlled routes.
4. All outputs are persisted with content hashes for reproducibility.
