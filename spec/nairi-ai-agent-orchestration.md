# NAIRI AI Agent Orchestration Specification

## 1. Purpose

NAIRI's primary differentiator is an AI coding/analysis agent that executes the full analysis workflow autonomously.

User interaction goal:

1. Upload APK.
2. Press `Analyse`.
3. Receive complete correlated report with artifacts.

## 2. Agent Role

The AI agent is the control plane for analysis orchestration and evidence-driven decision making.

The agent is responsible for:

1. Planning stage order for static, runtime, network, and pinning workflows.
2. Executing stage tasks through approved tool adapters.
3. Monitoring outcomes and adapting next actions.
4. Producing deterministic, reviewable run logs and final outputs.

## 3. Control Loop

Each analysis run follows this loop:

1. **Plan**
   1. Create run plan from sample metadata and profile.
   2. Select required tools and checks.
2. **Execute**
   1. Trigger static analysis.
   2. Trigger runtime analysis in sandbox.
   3. Trigger network inspection.
3. **Evaluate**
   1. Validate stage outputs.
   2. Detect blockers (for example TLS pinning, crashes, anti-analysis behavior).
4. **Adapt**
   1. Retry with bounded strategy.
   2. Escalate to bypass flow where policy allows.
5. **Finalize**
   1. Correlate observations.
   2. Emit report and evidence manifest.

## 4. Decision Model

Agent decisions are policy-constrained and evidence-based.

Required decision classes:

1. Stage branching (what to run next).
2. Retry/backoff (when and how often).
3. Pinning bypass strategy selection.
4. Confidence scoring and uncertainty declaration in reports.

## 5. Safety and Governance

1. Agent must only invoke allowlisted actions in the analysis environment.
2. Agent must not perform destructive or out-of-scope actions outside run policy.
3. Agent decisions must be logged with:
   1. Input evidence references.
   2. Decision rationale summary.
   3. Action and result.
4. Agent must provide safe fallback states:
   1. Partial report on failure.
   2. Explicit error reasons and unresolved gaps.

## 6. User Experience Contract

From the analyst perspective:

1. Single action start (`Analyse`).
2. Live run status by stage.
3. Final report with confidence and evidence lineage.
4. Optional advanced controls remain profile-based, not mandatory.

## 7. Integration Points

The AI agent integrates with:

1. Orchestrator APIs.
2. Static engine toolchain (`apktool`, `Ghidra` scripts).
3. Runtime pipeline (redroid, Frida, eBPF).
4. Configuration service (static-analysis image + ADB connection profile).
5. MITM and pinning bypass service.
6. Reporting and evidence store.

## 8. Failure Handling

1. Bounded retries for transient failures.
2. Stage skip only when policy permits and must be declared in report.
3. Automatic run termination on security policy violation.
4. Evidence preservation on failure for forensic replay.

## 9. Configuration Inputs

1. Agent run planning must consume the resolved active configuration profile.
2. Static-stage execution environment must use configured Docker image with required toolchain.
3. Runtime-stage target selection must use configured ADB connection string.
4. Missing/invalid required configuration must fail fast before stage execution.
