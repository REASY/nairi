# NAIRI Reporting Specification

## 1. Report Types

1. Executive summary report.
2. Technical behavior report.
3. Artifact manifest and IoC export.

## 2. Mandatory Technical Sections

1. **Native Libraries and Methods**
   1. Library load timeline.
   2. JNI and native method call summary.
2. **Filesystem, Android Properties, Syscalls**
   1. File path access attempts and outcomes.
   2. Property access events and values (policy-filtered).
   3. Syscall statistics and suspicious sequences.
3. **HTTPS and C2 Behavior**
   1. Domain/IP/SNI map.
   2. HTTP request/response summary.
   3. Exfiltration or beaconing indicators.
4. **Pinning Bypass Outcome**
   1. Pinning evidence.
   2. Bypass path selected.
   3. Pre/post behavior delta.

## 3. Report Quality Requirements

1. Every finding references concrete evidence IDs.
2. Severity and confidence are explicit per indicator.
3. Timeline is deterministic and reproducible from stored artifacts.
4. Output supports both human-readable markdown/PDF and machine-readable JSON.
