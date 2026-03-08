# NAIRI AST and Native Graph Pipeline

## Rust + Smali + Ghidra + Memgraph

Author: NAIRI Architecture Spec  
Target: Rust backend (`nairi`)  
Status: Design Specification

---

# 1. Overview

This document defines the deterministic static-evidence pipeline for NAIRI.

Goal:

Transform APK static artifacts into a structured graph that can be used by:

1. Deterministic rule evaluation.
2. AI orchestration for evidence-driven next-step decisions.
3. Reporting and cross-stage correlation.

The pipeline performs:

1. APK decompilation (`apktool`).
2. Manifest parsing.
3. Smali parsing into AST/IR.
4. Native library discovery and headless Ghidra export.
5. IR normalization and symbol resolution.
6. DB Mapping & Graph construction.
7. Batched ingestion into Memgraph.
8. Rule execution and indicator emission.

The AI layer should consume graph evidence first, then optionally request targeted follow-up analysis.

---

# 2. Positioning in NAIRI Spec Set

This spec augments, not replaces, existing NAIRI contracts:

1. [Static Analysis Design](./nairi-static-analysis.md): this document defines the deterministic extraction internals.
2. [AI-Driven Static Analysis](./nairi-ai-driven-static-analysis.md): AI consumes graph evidence and decides adaptive
   follow-up actions.
3. [AI Agent Orchestration](./nairi-ai-agent-orchestration.md): orchestration remains policy-constrained and auditable.
4. [Data Model and Event Schema](./nairi-data-model.md): findings must map to canonical `Indicator`, `Artifact`,
   `Report`, and evidence IDs.
5. [Configuration Specification](./nairi-configuration.md): static stage still runs in configured Docker image.
6. [Reporting Specification](./nairi-reporting.md): outputs must support evidence-linked report generation.

---

# 3. System Architecture

## Module layout

```
backend/
  crates/
    nairi-ast/ (Pure Parser & IR)
      src/
        manifest/
        smali/
        ghidra/
        ir/
        metrics/
    nairi-graph/ (Database & Rules)
      src/
        mapping/
        ingest/
        query/
        rules/
    nairi-orchestrator/ (Workflow & Docker)
      src/
        static_analysis/
```

### Module responsibilities

**`nairi-ast`**
*Pure parser library with no side-effects or DB knowledge. Given an input directory, it returns IR.*

`manifest/`  
Parses `AndroidManifest.xml` and component/permission metadata.

`smali/`  
Parses Smali files into deterministic language-level structures using **tree-sitter**.

`ghidra/`  
Parses pre-generated headless Ghidra JSON export artifacts into IR.

`ir/`  
Canonical intermediate representation for Java/Smali and native domains.

`metrics/`  
Instrumentation for parser throughput and accuracy.

**`nairi-graph`**
*Connects IR to Memgraph and executes rules.*

`mapping/`  
Maps IR into graph nodes/edges with stable IDs and provenance.

`ingest/`  
Batched Memgraph writer and retry logic.

`query/`  
Predefined graph queries for static findings and reporting support.

`rules/`  
Deterministic rule execution over graph-backed evidence.

**`nairi-orchestrator`**
*Coordinates external tools.*

`static_analysis/`  
End-to-end orchestration: invokes `apktool` Docker, invokes `ghidra` Docker, passes the resulting output folder to
`nairi-ast`, passes the IR to `nairi-graph`.

---

# 4. Deterministic Extraction Stages

Input comes from `apktool d app.apk` output:

```
AndroidManifest.xml
smali/
smali_classes2/
res/
assets/
lib/
```

Stage sequence:

1. Parse manifest and components.
2. Parse all smali classes/methods/fields/instructions.
3. Discover native libraries by ABI.
4. Run headless Ghidra and collect structured JSON exports.
5. Normalize all artifacts into IR.
6. Resolve symbols and cross-language relations.
7. Emit graph operations for ingestion.

---

# 5. Ghidra Integration Contract

Ghidra output is a first-class graph input.

## Execution

1. Must run in the configured static-analysis Docker image (
   see [Configuration Specification](./nairi-configuration.md)).
2. Must run headless per discovered `.so` file.
3. Must emit machine-readable JSON artifacts (no Markdown-only output).

## Minimum exported fields per native binary

1. Library identity: `path`, `abi`, `sha256`.
2. Exported functions and addresses.
3. Imported symbols/APIs.
4. Strings of interest.
5. Optional: call edges and function-level tags from scripts.

## Artifact contract

Each Ghidra JSON artifact is persisted as canonical `Artifact` with hash metadata and evidence reference.

---

# 6. Smali Parsing Requirements

The parser must extract:

### Class-level

1. Class descriptor.
2. Superclass.
3. Interfaces.
4. Annotations.
5. Access flags.

### Field-level

1. Field name.
2. Field type.
3. Access flags.
4. Constant initializer.

### Method-level

1. Method name.
2. Prototype.
3. Access flags.
4. Registers/locals.
5. Instructions.
6. Annotations.
7. Try/catch blocks.

### Instruction-level

Must support at minimum:

1. `invoke-*`
2. `const*`
3. `sget`/`iget`/`sput`/`iput`
4. `new-instance`
5. `return*`
6. `goto`/`if*`
7. `const-string`
8. `throw`

---

# 7. Intermediate Representation (IR)

Example structures:

```rust
pub struct EvidenceRef {
    pub run_id: String,
    pub artifact_id: String,
    pub source: String, // manifest|smali|ghidra
    pub tool_version: String,
}

pub struct ApkIr {
    pub apk_id: String,
    pub package_name: Option<String>,
    pub classes: Vec<ClassIr>,
    pub native_libs: Vec<NativeLibIr>,
}

pub struct ClassIr {
    pub descriptor: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub methods: Vec<MethodIr>,
    pub fields: Vec<FieldIr>,
}

pub struct MethodIr {
    pub id: String,
    pub name: String,
    pub proto: String,
    pub access_flags: Vec<String>,
    pub instructions: Vec<InstrIr>,
}

pub enum InstrIr {
    Invoke { target: String },
    LoadLibrary { lib_name: String },
    ConstString { value: String },
    FieldRead { field: String },
    FieldWrite { field: String },
    Other { opcode: String },
}

pub struct NativeLibIr {
    pub id: String,
    pub abi: String,
    pub path: String,
    pub sha256: String,
    pub exports: Vec<NativeFunctionIr>,
    pub imports: Vec<NativeImportIr>,
}

pub struct NativeFunctionIr {
    pub id: String,
    pub name: String,
    pub address: Option<u64>,
}

pub struct NativeImportIr {
    pub id: String,
    pub symbol: String,
}
```

All IR entities must carry provenance and evidence references.

---

# 8. Graph Schema (Memgraph)

Memgraph stores unified static evidence using a property graph.

## Node types

1. `Apk`
2. `Package`
3. `Class`
4. `Method`
5. `Field`
6. `Component`
7. `Permission`
8. `StringLiteral`
9. `Api`
10. `NativeLib`
11. `NativeFunction`
12. `NativeImport`
13. `Finding`

## Relationships

```cypher
(:Apk)-[:DECLARES_CLASS]->(:Class)
(:Class)-[:DECLARES_METHOD]->(:Method)
(:Class)-[:DECLARES_FIELD]->(:Field)
(:Method)-[:CALLS]->(:Method)
(:Method)-[:CALLS_API]->(:Api)
(:Method)-[:USES_STRING]->(:StringLiteral)
(:Method)-[:LOADS_LIB]->(:NativeLib)
(:Method)-[:JNI_BINDS]->(:NativeFunction)
(:NativeFunction)-[:DEFINED_IN]->(:NativeLib)
(:NativeFunction)-[:CALLS_IMPORT]->(:NativeImport)
(:Component)-[:REACHES]->(:Method)
(:Apk)-[:USES_PERMISSION]->(:Permission)
(:Finding)-[:EVIDENCED_BY]->(:Method)
(:Finding)-[:EVIDENCED_BY]->(:NativeFunction)
```

Every node/edge should include provenance fields:

`run_id`, `source`, `artifact_id`, `evidence_ref`, `confidence`.

---

# 9. Memgraph Ingestion

Graph writes must be batched and deterministic.

Ingestion stages:

1. Workers emit graph operations.
2. Aggregator deduplicates by stable IDs.
3. Batch writer submits Cypher in bounded chunks.
4. Failed batch retries use idempotent merge semantics.

Example batch write:

```cypher
UNWIND $rows AS row
MERGE (n:NativeFunction {id: row.id})
SET n += row.props
```

---

# 10. Parallel Parsing Strategy

Recommended model:

```
Smali + Native Artifacts
      |
      v
Parallel Parser Workers (Rayon)
      |
      v
Graph Operation Channel
      |
      v
Aggregator
      |
      v
Batch Writer -> Memgraph
```

Good candidates for parallelism:

1. File discovery.
2. Smali parsing.
3. Ghidra artifact normalization.
4. IR extraction and lightweight tagging.

Avoid:

1. Direct DB writes from worker threads.
2. Unbounded shared mutable state.

---

# 11. Resolution Passes

1. Pass 1: Extract manifest, smali, and native raw entities.
2. Pass 2: Resolve Java symbols (method/field references).
3. Pass 3: Resolve native symbols/imports from Ghidra artifacts.
4. Pass 4: Build cross-language relations (`LOADS_LIB`, `JNI_BINDS`).
5. Pass 5: Link Android entrypoints/components to reachable methods.

---

# 12. Rule Engine and Indicator Mapping

Example deterministic rule families:

1. Exported component exposure.
2. WebView bridge (`addJavascriptInterface`).
3. Dynamic loading (`DexClassLoader`).
4. Custom trust manager patterns.
5. Suspicious native import chains.

Rule output must map to canonical data model:

1. `Finding` graph nodes are intermediate analysis artifacts.
2. Persisted output is `Indicator` with `severity`, `confidence`, and `evidence_refs[]`.
3. Report generation must use these evidence-linked indicators.

---

# 13. AI Usage Model

AI should use graph evidence for planning and adaptation:

1. Consume deterministic graph facts.
2. Decide targeted follow-up actions (for example: run deeper Ghidra script on one library, trigger runtime branch
   checks).
3. Log decision rationale with evidence references.

This preserves AI autonomy while reducing non-deterministic code interpretation.

---

# 14. Graph Queries

### Exported attack surface

```cypher
MATCH (c:Component)-[:REACHES]->(m:Method)
WHERE c.exported = true
MATCH (m)-[:CALLS_API]->(a:Api)
RETURN c,m,a
```

### WebView bridge detection

```cypher
MATCH (m:Method)-[:CALLS_API]->(a:Api)
WHERE a.signature CONTAINS "addJavascriptInterface"
RETURN m
```

### Java -> native suspicious chain

```cypher
MATCH (c:Component)-[:REACHES]->(m:Method)-[:LOADS_LIB]->(l:NativeLib)
WHERE c.exported = true
MATCH (f:NativeFunction)-[:DEFINED_IN]->(l)
MATCH (f)-[:CALLS_IMPORT]->(i:NativeImport)
WHERE i.symbol IN ["SSL_set_custom_verify", "connect", "system"]
RETURN c,m,l,f,i
```

---

# 15. Performance and Reliability Targets

1. Parse medium APK in seconds.
2. Handle 50k+ methods with bounded memory.
3. Keep ingestion batch latency stable under load.
4. Ensure deterministic reruns from immutable artifacts.

---

# 16. Implementation Phases

## Phase 1 (MVP-1 aligned)

1. Manifest parser.
2. Smali parser + IR.
3. Native library discovery.
4. Headless Ghidra JSON export.
5. Baseline graph ingestion and core queries.

## Phase 2

1. Cross-language linking (`LOADS_LIB`, `JNI_BINDS`).
2. Deterministic rule engine.
3. Indicator/report data-model mapping.

## Phase 3

1. Runtime trace correlation hooks.
2. Query library hardening for reporting.
3. Advanced native callgraph enrichment.

---

# 17. Dependencies and Tooling

Suggested crates:

Suggested crates for `nairi-ast` and `nairi-graph`:

```
rayon
serde
serde_json
roxmltree
tree-sitter (and related language crates e.g. tree-sitter-smali)
crossbeam-channel
tracing
metrics
```

Parser implementation must use **tree-sitter** for accurate, deterministic, and maintainable AST generation.

---

# 18. Observability and Audit

Track at minimum:

1. Parse throughput.
2. Failed files/artifacts.
3. Classes/methods/native symbols extracted.
4. Graph operations emitted.
5. Memgraph batch latency/retries.
6. Rule hit counts by severity.

Persist metadata required for auditability:

1. Tool versions (`apktool`, Ghidra scripts).
2. Artifact hashes.
3. Run ID and profile snapshot linkage.

---

# 19. Requirement Traceability

| This spec capability                      | Contract linkage                            |
|-------------------------------------------|---------------------------------------------|
| APK decompile + manifest/smali extraction | `FR-STA-001`, `FR-STA-002`                  |
| Native extraction + Ghidra integration    | `FR-STA-003`, `FR-STA-004`                  |
| Static rules and indicators               | `FR-STA-005`, `FR-RPT-005`                  |
| AI evidence-driven decisions              | `FR-AI-002`, `FR-AI-003`, `NFR-AI-001`      |
| Reproducible evidence pipeline            | `NFR-REL-001`, `NFR-DAT-001`, `NFR-AUD-001` |
| Configured static execution environment   | `FR-CFG-003`, `FR-CFG-006`                  |

---

# 20. Summary

This pipeline makes AST and native analysis deterministic, graph-native, and AI-consumable.

Ghidra output is integrated directly into the same evidence graph, enabling Java-native correlation and stronger static
findings while preserving NAIRI orchestration, reporting, and compliance contracts.
