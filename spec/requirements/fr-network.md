# NAIRI Network and Pinning Functional Requirements

## 1. Functional Requirements

| ID | Requirement |
| --- | --- |
| `FR-NET-001` | Route app traffic through MITM inside the sandboxed analysis environment. |
| `FR-NET-002` | Capture TLS metadata and HTTP transactions with redaction policy enforcement. |
| `FR-NET-003` | Detect certificate pinning failures during runtime analysis. |
| `FR-NET-004` | Execute pinning bypass flow: runtime hook attempt first, static patch/rebuild/resign/redeploy fallback second. |

## 2. Traceability Matrix

| ID | Architecture Components | Primary Evidence / Output | Verification Approach |
| --- | --- | --- | --- |
| `FR-NET-001` | Network Inspection Layer | Proxy route logs and flow metadata | Controlled connectivity test through MITM |
| `FR-NET-002` | Network Inspection Layer, Reporting Service | TLS/HTTP capture artifacts with policy tags | Redaction fixture tests + capture validation |
| `FR-NET-003` | Network Inspection Layer, Instrumentation Layer | Pinning detection events | Pinned test app with expected TLS failures |
| `FR-NET-004` | Patch/Rebuild Service, Runtime Analysis Engine | Bypass attempt logs + patched artifact + rerun result | Integration tests on runtime and static bypass paths |
