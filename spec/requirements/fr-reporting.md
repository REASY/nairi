# NAIRI Reporting Functional Requirements

## 1. Functional Requirements

| ID | Requirement |
| --- | --- |
| `FR-RPT-001` | Produce loaded native library timeline and method usage summary. |
| `FR-RPT-002` | Produce filesystem, Android property, and syscall activity report. |
| `FR-RPT-003` | Produce network and HTTPS behavior report. |
| `FR-RPT-004` | Produce certificate pinning bypass report with pre/post behavior delta. |
| `FR-RPT-005` | Emit machine-readable artifact index and IoC set. |

## 2. Traceability Matrix

| ID | Architecture Components | Primary Evidence / Output | Verification Approach |
| --- | --- | --- | --- |
| `FR-RPT-001` | Reporting Service, Instrumentation Layer | Native load/method report section | Golden snapshot comparison |
| `FR-RPT-002` | Reporting Service, eBPF pipeline | File/property/syscall report section | Structured output validation |
| `FR-RPT-003` | Reporting Service, Network Inspection Layer | HTTPS/network report section | Network fixture replay validation |
| `FR-RPT-004` | Reporting Service, Patch/Rebuild Service | Pinning outcome and delta section | Pinned sample before/after comparison test |
| `FR-RPT-005` | Reporting Service, Evidence Store | JSON artifact manifest and IoC bundle | Contract test against schema |
