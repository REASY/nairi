# NAIRI Runtime Analysis Design

## 1. Runtime Environment

1. Use one isolated `redroid` instance per analysis run.
2. Emulator image includes required kernel/eBPF capabilities.
3. Environment is sandboxed with controlled network routing.

## 2. Frida Instrumentation Scope

Mandatory hook points:

1. Java loading APIs:
   1. `System.loadLibrary`
   2. `Runtime.load` and `Runtime.loadLibrary`
2. Native loading APIs:
   1. `dlopen`
   2. `android_dlopen_ext`
3. JNI registration:
   1. `RegisterNatives`
4. TLS-relevant APIs in Java/native stack for pinning diagnostics.

## 3. eBPF Telemetry Scope

1. Syscall events:
   1. `openat`, `read`, `write`, `connect`, `sendto`, `recvfrom`, `execve`
2. File access metadata:
   1. Path, flags, return code, process context.
3. Network metadata:
   1. Destination IP, port, protocol, process context.

## 4. Event Context Requirements

Each event must include:

1. Event timestamp.
2. Data source (`frida` or `ebpf`).
3. PID, TID, UID.
4. Package/process identifiers.
5. Run/session ID for correlation.

## 5. Runtime Workflow

1. Start clean emulator instance.
2. Install APK and launch app.
3. Attach Frida hooks.
4. Start eBPF collectors.
5. Execute behavior stimulation plan.
6. Persist events and artifacts.
7. Stop and destroy emulator.

## 6. Current eBPF Script Baseline

Current runtime eBPF collection is implemented with `bpftrace` scripts:

1. `backend/runtime/ebpf/probes/trace_fs.bt`
   1. Probes libc wrappers: `openat`, `access`, `stat`, `lstat`, `readlink`, `readlinkat`, `opendir`.
   2. Captures path, rc/ret, success/failure, and best-effort errno.
2. `backend/runtime/ebpf/probes/trace_net.bt`
   1. Probes `sys_enter/exit_socket`, `sys_enter_connect`, and `sys_enter_close`.
   2. Captures socket domain/type/proto and connect attempts.
3. `backend/runtime/ebpf/probes/trace_properties.bt`
   1. Probes `__system_property_get` (entry and return).
   2. Captures requested property keys and optional values.
4. `backend/runtime/ebpf/probes/trace_runtime.bt`
   1. Probes `prctl`, `uname`, and libc `syscall(...)` wrapper usage.
5. `research/trace-experiments/run_trace_experiments.sh`
   1. Runs two phases (`fresh_launch`, `second_launch`) and collects all trace logs.
6. `backend/runtime/ebpf/parsers/parse_trace_experiment_csv.py`
   1. Normalizes raw logs into summary and grouped CSV outputs.

## 7. Baseline Gaps to Address

To fully satisfy runtime requirements, implementation should address:

1. Network trace enrichment:
   1. Decode `sockaddr` to destination IP and port (current net script captures pointer/domain only).
2. Syscall coverage depth:
   1. Expand beyond current runtime wrapper focus to explicit syscall event capture targets in requirements.
3. ABI portability:
   1. Avoid hard-coding only `/lib64/` probe targets; support 32-bit library paths when needed.
4. Parser/schema alignment:
   1. Keep parser regex contracts aligned with trace output fields to prevent metric drift.
