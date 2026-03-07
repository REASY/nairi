# NAIRI AI Orchestration Functional Requirements

## 1. Functional Requirements

| ID | Requirement |
| --- | --- |
| `FR-AI-001` | The system must support single-action analysis initiation: user uploads APK and triggers `Analyse`. |
| `FR-AI-002` | The AI agent must generate and execute an end-to-end run plan covering static, runtime, network, and reporting stages. |
| `FR-AI-003` | The AI agent must adapt stage flow based on observed evidence (for example, trigger pinning bypass when required). |
| `FR-AI-004` | The AI agent must enforce bounded retries and deterministic failure handling with partial report fallback. |
| `FR-AI-005` | Every agent decision must be logged with decision type, rationale summary, evidence refs, action, and outcome. |
| `FR-AI-006` | The system must expose stage-level progress to users during autonomous execution. |
| `FR-AI-007` | The agent must produce a final report with confidence/uncertainty markers and evidence lineage. |

## 2. Traceability Matrix

| ID | Architecture Components | Primary Evidence / Output | Verification Approach |
| --- | --- | --- | --- |
| `FR-AI-001` | Sample Intake, Orchestrator UI/API | Run initiation record from single user action | UX/API workflow test |
| `FR-AI-002` | AI Agent Controller, Orchestrator | Run plan and executed stage graph | End-to-end orchestration test |
| `FR-AI-003` | AI Agent Controller, Patch/Rebuild Service | Adaptive branch logs and stage transitions | Pinned app scenario test |
| `FR-AI-004` | AI Agent Controller, Reporting Service | Retry log and partial report fallback | Failure-injection integration test |
| `FR-AI-005` | AI Agent Controller, Audit Log Service | Decision log records | Log contract and completeness checks |
| `FR-AI-006` | Orchestrator, UI/API Layer | Stage status timeline | Live status API test |
| `FR-AI-007` | Reporting Service, Evidence Store | Final report with confidence and evidence links | Golden report schema validation |
