# NAIRI Runtime Functional Requirements

## 1. Functional Requirements

| ID | Requirement |
| --- | --- |
| `FR-RUN-001` | Execute each app in an isolated redroid sandbox with disposable state. |
| `FR-RUN-002` | Inject Frida hooks for Java and native layers, including library loading and JNI registration points. |
| `FR-RUN-003` | Collect eBPF telemetry for syscall, file-system, and network events. |
| `FR-RUN-004` | Tag each runtime event with timestamp, PID, TID, UID, package, process, and run ID context. |
| `FR-RUN-005` | Support scripted stimulation to trigger runtime behavior. |

## 2. Traceability Matrix

| ID | Architecture Components | Primary Evidence / Output | Verification Approach |
| --- | --- | --- | --- |
| `FR-RUN-001` | Runtime Analysis Engine, Orchestrator | Run metadata + sandbox lifecycle logs | End-to-end run on disposable emulator |
| `FR-RUN-002` | Instrumentation Layer (Frida) | Hook events and call logs | Hook coverage tests on known sample app |
| `FR-RUN-003` | Instrumentation Layer (eBPF) | Syscall/file/network event stream from `backend/runtime/ebpf/probes/{trace_fs.bt,trace_net.bt,trace_properties.bt,trace_runtime.bt}` | Kernel probe integration tests + log contract tests |
| `FR-RUN-004` | Instrumentation Layer, Data Model | Normalized observation records | Schema validation + correlation tests |
| `FR-RUN-005` | Runtime Analysis Engine | Stimulation script execution log | Deterministic replay scenario tests |
