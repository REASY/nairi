# NAIRI Roadmap and Acceptance Criteria

## 1. MVP Phases

1. **MVP-1: Static Baseline**
   1. Sample intake.
   2. `apktool` decompilation and manifest/component parser.
   3. Native extraction and basic Ghidra automation.
2. **MVP-2: Dynamic Core**
   1. redroid orchestration.
   2. Frida hook pipeline.
   3. eBPF syscall/file/network telemetry.
3. **MVP-3: Network Intelligence**
   1. MITM routing and capture.
   2. Report correlation across static/runtime/network evidence.
4. **MVP-4: Pinning Bypass**
   1. Runtime bypass strategy.
   2. Static patch/rebuild/resign fallback.
   3. Pre/post rerun comparison report.
5. **MVP-5: AI Autonomous Analyst**
   1. Single-action run initiation (`upload + Analyse`).
   2. Autonomous stage planning and adaptive branching.
   3. Decision-log explainability and policy-constrained execution.
   4. Partial-report fallback for failure paths.

## 2. Acceptance Criteria

1. NAIRI reports loaded native `.so` libraries and JNI mappings for a known test sample.
2. NAIRI captures filesystem operations, property access, and syscall profile with process attribution.
3. NAIRI captures HTTPS metadata and decrypted traffic when pinning bypass succeeds.
4. NAIRI completes at least one successful pinning bypass path and reruns the sample.
5. Final report is reproducible from immutable artifact set and includes risk verdict plus IoCs.
6. NAIRI completes full analysis from single user action without manual stage orchestration.
7. NAIRI provides machine-readable decision logs for each autonomous branch.

## 3. Exit Conditions per Phase

1. MVP-1 exits when static findings are deterministic across repeated runs.
2. MVP-2 exits when runtime telemetry is stable and correctly correlated by run/session.
3. MVP-3 exits when network reports include complete request lineage to process context.
4. MVP-4 exits when both runtime and static bypass paths are demonstrated on representative test apps.
5. MVP-5 exits when autonomous runs are stable, explainable, and policy-compliant across representative malware fixtures.
