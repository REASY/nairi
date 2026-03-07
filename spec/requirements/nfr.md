# NAIRI Non-Functional Requirements

## 1. Non-Functional Requirements

| ID | Category | Requirement |
| --- | --- | --- |
| `NFR-REL-001` | Reproducibility | Every analysis run must be reproducible from immutable stored artifacts. |
| `NFR-DAT-001` | Data Integrity | Evidence artifacts must include integrity metadata (at minimum SHA-256). |
| `NFR-AUD-001` | Auditability | Audit logs must capture each analysis stage transition with run linkage. |
| `NFR-AI-001` | Explainability | AI orchestration decisions must be reviewable through structured decision logs and evidence references. |
| `NFR-AI-002` | Safety | AI agent actions must be policy-constrained to allowlisted capabilities. |
| `NFR-AI-003` | Resilience | AI orchestration failures must preserve evidence and emit partial report output. |
| `NFR-CFG-001` | Configuration Integrity | Configuration updates must be validated and rejected if required fields are invalid. |
| `NFR-CFG-002` | Configuration Auditability | Configuration changes must be versioned and auditable with actor and timestamp. |
| `NFR-CFG-003` | Configuration Reproducibility | Each run must reference immutable configuration snapshot used at run start. |
| `NFR-TECH-001` | Technology Baseline | Frontend must be implemented with TypeScript and React. |
| `NFR-TECH-002` | Technology Baseline | Backend must be implemented with Rust and Axum. |
| `NFR-TECH-003` | Contract Consistency | Frontend and backend must share typed API contract artifacts for compile-time validation. |
| `NFR-SEC-001` | Isolation | Runtime analysis must enforce one-sample-per-sandbox isolation and teardown. |
| `NFR-SEC-002` | Network Security | Runtime environment must enforce restricted egress policy (allowlist or sinkhole). |
| `NFR-SEC-003` | Compliance | Interception and sample processing must be restricted to authorized legal/policy contexts. |

## 2. Traceability Matrix

| ID | Primary Components | Primary Control / Output | Verification Approach |
| --- | --- | --- | --- |
| `NFR-REL-001` | Orchestrator, Evidence Store | Deterministic run reconstruction | Reproducibility drill with archived artifacts |
| `NFR-DAT-001` | Evidence Store | Hash metadata and integrity checks | Artifact integrity validation tests |
| `NFR-AUD-001` | Orchestrator, Audit Log Service | Stage transition audit records | Audit trail completeness checks |
| `NFR-AI-001` | AI Agent Controller, Audit Log Service | Decision log records with evidence refs | Explainability contract validation |
| `NFR-AI-002` | AI Agent Controller, Policy Layer | Action policy enforcement logs | Policy-violation injection tests |
| `NFR-AI-003` | AI Agent Controller, Reporting Service | Partial report + preserved artifacts on failure | Failure-path integration tests |
| `NFR-CFG-001` | Configuration Service, Backend API | Validation result logs and rejected invalid writes | Config validation test suite |
| `NFR-CFG-002` | Configuration Service, Audit Log Service | Config change history | Config audit completeness checks |
| `NFR-CFG-003` | Orchestrator, Evidence Store | Run-linked immutable config snapshot | Reproducibility replay tests |
| `NFR-TECH-001` | Frontend Application | React + TypeScript build and CI checks | Frontend pipeline contract check |
| `NFR-TECH-002` | Backend API | Rust + Axum build and integration checks | Backend compile and API smoke tests |
| `NFR-TECH-003` | Frontend Application, Backend API | Shared typed API contract artifacts | Schema drift detection in CI |
| `NFR-SEC-001` | Runtime Analysis Engine | Sandbox lifecycle logs | Isolation teardown integration tests |
| `NFR-SEC-002` | Network Inspection Layer | Egress control policy logs | Network policy enforcement test suite |
| `NFR-SEC-003` | Policy Layer, Access Controls | Authorization and policy tags | Compliance controls review and spot checks |

## 3. Legacy Mapping

| Previous ID | Current ID |
| --- | --- |
| `FR-OPS-001` | `NFR-REL-001` |
| `FR-OPS-002` | `NFR-DAT-001` |
| `FR-OPS-003` | `NFR-AUD-001` |
