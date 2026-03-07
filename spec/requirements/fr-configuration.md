# NAIRI Configuration Functional Requirements

## 1. Functional Requirements

| ID | Requirement |
| --- | --- |
| `FR-CFG-001` | The system must provide UI and API for managing active NAIRI configuration profile. |
| `FR-CFG-002` | The configuration profile must include `static_analysis_image` and `adb_connection_string` as required fields. |
| `FR-CFG-003` | Static analysis must execute inside the configured Docker image that includes `apktool` and `Ghidra`. |
| `FR-CFG-004` | Runtime analysis must use the configured ADB connection string to select/connect to Android target. |
| `FR-CFG-005` | The system must validate both configuration fields before allowing analysis start. |
| `FR-CFG-006` | The AI agent/orchestrator must consume a resolved configuration snapshot at run start and use it for all run stages. |
| `FR-CFG-007` | Each run record must store the configuration snapshot used for reproducibility and audit. |

## 2. Traceability Matrix

| ID | Architecture Components | Primary Evidence / Output | Verification Approach |
| --- | --- | --- | --- |
| `FR-CFG-001` | Frontend Application, Backend API | Config UI/API workflow | UI + API integration tests |
| `FR-CFG-002` | Backend API, Data Model | Profile schema and persistence records | Contract/schema tests |
| `FR-CFG-003` | Configuration Service, Static Analysis Engine | Container execution logs + tool check results | Containerized static-stage integration test |
| `FR-CFG-004` | Configuration Service, Runtime Analysis Engine | ADB target selection/connection logs | Runtime target connectivity tests |
| `FR-CFG-005` | Backend API, Configuration Service | Validation result objects and status codes | Negative/positive validation tests |
| `FR-CFG-006` | AI Agent Controller, Orchestrator | Run context with immutable config snapshot | Orchestration context tests |
| `FR-CFG-007` | Evidence Store, Reporting Service | Run metadata and report config references | Reproducibility and audit tests |
