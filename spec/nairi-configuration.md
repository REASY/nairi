# NAIRI Configuration Specification

## 1. Purpose

NAIRI must provide a configuration UI/API so operators can control mandatory analysis infrastructure without code changes.

Minimum required configuration fields:

1. Static analysis Docker image (pre-installed `apktool` and `Ghidra`).
2. Android device connection string usable by `adb`.

## 2. Configuration Model

`ConfigurationProfile` minimum fields:

1. `profile_id`
2. `name`
3. `static_analysis_image`
4. `adb_connection_string`
5. `is_active`
6. `updated_by`
7. `updated_at`

## 3. UI Requirements

The frontend must provide a configuration screen with:

1. Input: static analysis Docker image reference.
2. Input: ADB connection string.
3. Action: `Save`.
4. Action: `Validate`.
5. Read-only state: last validation result and timestamp.

## 4. Backend/API Requirements

1. Create/update/read active configuration profile.
2. Validate static image by running tool checks inside container context:
   1. `apktool --version`
   2. Ghidra headless availability check
3. Validate ADB target by testing connection against configured string.
4. Expose profile snapshot used for each run.

## 5. Runtime Usage Contract

1. At run start, orchestrator resolves active configuration profile.
2. AI agent must consume resolved configuration as immutable run input.
3. Static analysis stage must execute using configured Docker image.
4. Runtime stage must target device/emulator using configured ADB connection string.
5. Any missing/invalid required configuration must hard-fail before analysis starts.

## 6. Validation Rules (Minimum)

1. `static_analysis_image` must be a valid image reference (`repo/name:tag` or digest).
2. `adb_connection_string` must be valid for `adb connect`/device selection workflow.
3. Validation errors must be explicit and user-actionable.

## 7. Audit and Traceability

1. Every config change must be audited with actor and timestamp.
2. Each analysis run must persist the exact configuration snapshot used.
3. Reports must include configuration profile ID and relevant runtime target metadata.
