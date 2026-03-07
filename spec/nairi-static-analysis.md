# NAIRI Static Analysis Design

## 1. Inputs and Outputs

Inputs:

1. Raw APK file.
2. Optional context metadata (source, campaign, tags).
3. Resolved run configuration profile with static analysis Docker image reference.

Outputs:

1. Decompilation artifact bundle.
2. Manifest/component findings.
3. Native binary findings from Ghidra.
4. Static indicator set and confidence scores.

## 2. Processing Steps

1. Validate APK and compute hashes.
2. Resolve and validate configured static-analysis Docker image.
3. Decompile with `apktool`.
4. Parse `AndroidManifest.xml` and resource metadata.
5. Extract `lib/**` native binaries.
6. Run headless Ghidra analysis scripts.
7. Apply static detection rules and create indicators.

## 3. Detection Categories

1. Permissions and component exposure risk.
2. Native loader and JNI bridge behavior.
3. Crypto and obfuscation signatures.
4. Anti-debug, anti-VM, and anti-hooking hints.
5. Suspicious hardcoded strings:
   1. Endpoints and paths.
   2. Wallet/credential keywords.
   3. Command execution templates.

## 4. Required Static Evidence

1. Manifest summary JSON.
2. Component and intent graph export.
3. Native library inventory by ABI.
4. Symbol/import/string extraction outputs.
5. Rule hits with evidence references.

## 5. Execution Environment Contract

1. Static analysis must run in configured Docker image.
2. Configured image must include:
   1. `apktool` executable.
   2. Ghidra headless analysis tooling.
3. Toolchain validation failure must block analysis start.
