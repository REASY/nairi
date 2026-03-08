# NAIRI AI-Driven Static Analysis Execution

## 1. Overview

While [Static Analysis Design](./nairi-static-analysis.md) defines what must happen, and
[AST and Native Graph Pipeline](./nairi-ast-pipeline.md) defines deterministic extraction internals, this document
specifies how the AI agent drives adaptive static analysis decisions.

The static phase is evidence-first:

1. Deterministic parser/native tooling builds graph-backed evidence.
2. AI consumes that evidence to decide follow-up actions.
3. AI can perform targeted deep dives when evidence is insufficient.

## 2. Agent Environment and Tools

The static analysis agent runs inside the configured static-analysis Docker image with reverse-engineering tooling.

Orchestrator executes the static agent with mounted APK and output paths.

The embedded agent uses restricted tools such as:

1. `run_terminal_command` for controlled tool execution.
2. `read_file_chunk` for targeted file context.
3. `grep_search` for focused code/symbol discovery.
4. `write_file` for generated scripts/config.
5. Graph/evidence readers for deterministic artifact and query inspection.

## 3. Autonomous Evidence-Driven Loop

The agent receives a goal similar to: analyze static behavior and produce evidence-linked indicators.

### Step 3.1: Deterministic extraction

1. Run `apktool` decompilation.
2. Parse manifest and smali.
3. Discover native libraries and execute headless Ghidra scripts.
4. Ingest normalized evidence into graph.

### Step 3.2: Graph-first triage

1. Execute predefined graph queries for core risk patterns.
2. Evaluate rule hits and unresolved confidence gaps.
3. Prioritize components/methods/libs for deeper inspection.

### Step 3.3: Targeted deep dives

When needed, the agent performs bounded deep analysis:

1. Read specific smali/native contexts tied to graph evidence.
2. Generate and run additional Ghidra scripts for one library/function scope.
3. Add new evidence and re-evaluate indicators.

## 4. Final Aggregation

The static phase returns structured outputs for orchestration and reporting:

1. Evidence-linked indicator set with severity and confidence.
2. Artifact list including deterministic parser and Ghidra outputs.
3. AI decision log entries describing follow-up actions and rationale.

These outputs are correlated with runtime/network stages by the main NAIRI orchestrator.
