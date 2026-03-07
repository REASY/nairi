# NAIRI Data Model and Event Schema

## 1. Core Entities

1. `Sample`
   1. `sample_id`
   2. `sha256`, `sha1`, `md5`
   3. `package_name`, `version`, `signer`
   4. `source`, `ingest_ts`
2. `Run`
   1. `run_id`, `sample_id`
   2. `env_id`, `profile_id`
   3. `start_ts`, `end_ts`, `status`
3. `ConfigurationProfile`
   1. `profile_id`, `name`, `is_active`
   2. `static_analysis_image`
   3. `adb_connection_string`
   4. `validation_status`, `validated_at`
   5. `updated_by`, `updated_at`
4. `RunConfigSnapshot`
   1. `run_id`, `profile_id`
   2. `static_analysis_image`
   3. `adb_connection_string`
   4. `resolved_at`
5. `Observation`
   1. `obs_id`, `run_id`
   2. `timestamp`, `source`, `type`
   3. `pid`, `tid`, `uid`, `process`, `package`
   4. `payload` (source-specific JSON)
6. `Artifact`
   1. `artifact_id`, `run_id`, `kind`, `path`
   2. `sha256`, `created_ts`, `retention_class`
7. `Indicator`
   1. `indicator_id`, `run_id`, `rule_id`
   2. `severity`, `confidence`
   3. `evidence_refs[]`
8. `AgentDecision`
   1. `decision_id`, `run_id`, `timestamp`
   2. `decision_type`, `rationale_summary`
   3. `input_evidence_refs[]`
   4. `action`, `outcome`, `status`
9. `Report`
   1. `report_id`, `run_id`
   2. `risk_score`, `verdict`, `ioc_refs[]`

## 2. Observation Sources

Allowed values:

1. `static`
2. `frida`
3. `ebpf`
4. `mitm`
5. `orchestrator`
6. `ai_agent`

## 3. Minimal Event Payload Contracts

1. File event payload:
   1. `path`, `op`, `flags`, `result`, `errno`
2. Syscall payload:
   1. `name`, `args`, `result`, `errno`
3. Network payload:
   1. `dst_ip`, `dst_port`, `proto`, `direction`
4. Native load payload:
   1. `library`, `loader_api`, `base_addr`
5. Pinning payload:
   1. `detector`, `stage`, `status`, `details`
