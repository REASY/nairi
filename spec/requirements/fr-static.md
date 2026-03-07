# NAIRI Static Functional Requirements

## 1. Functional Requirements

| ID | Requirement |
| --- | --- |
| `FR-STA-001` | Decompile APK resources and smali with `apktool`. |
| `FR-STA-002` | Parse manifest and extract permissions, components, exported flags, and intents. |
| `FR-STA-003` | Extract native `.so` libraries grouped by ABI. |
| `FR-STA-004` | Run headless `Ghidra` analysis on extracted native binaries. |
| `FR-STA-005` | Detect static indicators including dynamic loading, reflection, native loader staging, command execution pathways, and anti-analysis hints. |

## 2. Traceability Matrix

| ID | Architecture Components | Primary Evidence / Output | Verification Approach |
| --- | --- | --- | --- |
| `FR-STA-001` | Sample Intake, Static Analysis Engine | Decompiled artifact bundle | Unit test + fixture APK decompile check |
| `FR-STA-002` | Static Analysis Engine | Manifest/component JSON summary | Parser validation against known manifest fixtures |
| `FR-STA-003` | Static Analysis Engine | Native library inventory by ABI | Fixture APK with multi-ABI libs |
| `FR-STA-004` | Static Analysis Engine | Ghidra analysis outputs | Headless script integration test |
| `FR-STA-005` | Static Analysis Engine, Reporting Service | Static rule hits with confidence | Rule-pack regression suite |
