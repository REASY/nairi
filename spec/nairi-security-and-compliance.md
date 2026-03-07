# NAIRI Security and Compliance Controls

## 1. Isolation Controls

1. One sample per sandboxed emulator run.
2. Mandatory emulator teardown after run completion.
3. No cross-run writable state reuse.

## 2. Network Controls

1. Default-deny egress with explicit allowlist or sinkhole.
2. Traffic interception only in authorized lab environment.
3. Outbound traffic logging with run linkage.

## 3. Data Protection

1. Hash all stored artifacts for integrity checks.
2. Apply retention classes to sensitive captures.
3. Restrict access to raw payloads and decrypted traffic artifacts.
4. Protect configuration profiles and connection targets with access control and audit logging.

## 4. Audit and Traceability

1. Log each stage transition in orchestrator.
2. Track who triggered each run and which profile was used.
3. Preserve evidence chain from sample ingest to final report.

## 5. Legal and Policy Guardrails

1. Analyze only samples covered by legal authorization.
2. Enforce policy for interception and sensitive data handling.
3. Support run-level policy tags for compliance reporting.
