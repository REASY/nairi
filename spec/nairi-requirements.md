# NAIRI Requirements Index

This document is the entry point for normative requirements and traceability.

## 1. Requirement Documents

1. [Static Functional Requirements](./requirements/fr-static.md)
2. [Runtime Functional Requirements](./requirements/fr-runtime.md)
3. [Network and Pinning Functional Requirements](./requirements/fr-network.md)
4. [Configuration Functional Requirements](./requirements/fr-configuration.md)
5. [AI Orchestration Functional Requirements](./requirements/fr-ai-orchestration.md)
6. [Reporting Functional Requirements](./requirements/fr-reporting.md)
7. [Non-Functional Requirements](./requirements/nfr.md)

## 2. ID Convention

1. `FR-STA-*`: Static analysis functional requirements.
2. `FR-RUN-*`: Runtime analysis functional requirements.
3. `FR-NET-*`: Network inspection and pinning functional requirements.
4. `FR-CFG-*`: Configuration functional requirements.
5. `FR-AI-*`: AI orchestration functional requirements.
6. `FR-RPT-*`: Reporting functional requirements.
7. `NFR-*`: Non-functional requirements.

## 3. Cross-Domain Traceability Summary

| Domain | Requirement IDs | Primary Components | Primary Verification |
| --- | --- | --- | --- |
| Static analysis | `FR-STA-001..005` | Static Analysis Engine | Static artifact diff + rule-hit fixtures |
| Runtime analysis | `FR-RUN-001..005` | Runtime Engine, Frida, eBPF | Instrumentation integration tests |
| Network/pinning | `FR-NET-001..004` | MITM Layer, Patch/Rebuild Service | Controlled TLS/pinning test apps |
| Configuration | `FR-CFG-001..007` | Frontend, Backend API, Configuration Service, Orchestrator | Configuration UI/API + integration validation tests |
| AI orchestration | `FR-AI-001..007` | AI Agent Controller, Orchestrator | End-to-end autonomous run and decision-log tests |
| Reporting | `FR-RPT-001..005` | Reporting Service | Golden report snapshots |
| Non-functional | `NFR-*` | Orchestrator, Evidence Store, Policy Controls, Frontend, Backend API | Compliance, reproducibility, and stack conformance checks |
