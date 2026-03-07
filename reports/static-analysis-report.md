# Static Analysis Report

## Executive Summary

A static analysis of `target.apk` was successfully performed using a combination of `apktool`, `jadx`, and `ghidra-cli`.
The analysis revealed that the application employs a packer/obfuscator mechanism utilizing native code to decrypt and
dynamically load its true payload (Dynamic Code Loading). Aggressive anti-debugging, emulator detection, and root
evasion techniques were uncovered inside the native library (`libuniffi_obfuscate.so`). A hardcoded cryptographic key
was also successfully extracted.

## Skill Usage Checklist

- **apktool**: PASS
    - `apktool d -f /workspace/target.apk -o /workspace/decompiled/apktool`
- **jadx**: PASS
    - `/opt/bin/jadx --deobf /workspace/target.apk -d /workspace/decompiled/jadx`
- **ghidra-cli**: PASS
    -
    `/opt/ghidra/support/analyzeHeadless ~/.cache/ghidra-cli/projects/ analysis_project -import /workspace/decompiled/apktool/lib/arm64-v8a/libuniffi_obfuscate.so -overwrite`
    - `ghidra summary --project analysis_project --program libuniffi_obfuscate.so`
    - `ghidra dump imports --project analysis_project --program libuniffi_obfuscate.so -o full`
    - `ghidra strings list --project analysis_project --program libuniffi_obfuscate.so -o full`

## Manifest Findings

The `AndroidManifest.xml` analysis revealed notable security misconfigurations:

- **`android:debuggable="true"`**: The application can be freely debugged in production environments.
- **`android:allowBackup="true"`**: Application data can be extracted using ADB backup, leading to potential data
  leakage.
- **Permissions**: `com.example.obfuscate.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION`
- **Exported Components**:
    - `com.example.obfuscate.MainActivity` (Main Activity)
    - `androidx.profileinstaller.ProfileInstallReceiver`

## Java/Kotlin Findings (jadx)

- **Native Loading**: The application leverages `System.loadLibrary("uniffi_obfuscate")` within the `p000.Obfuscate`
  namespace.
- **Dynamic Code Loading (DCL)**: The `DynamicLoaderV1` and `DynamicLoaderOriginal` classes utilize JNI calls (
  `ObfuscateKt.decryptBytes(dexBytesEncrypted)`) to load encrypted `.dex` payloads dynamically via the native library.
- **Hardcoded Secrets**: Found installation identifiers such as `KEY_INSTALL_ID = app_install_id`.
- **Cert Pinning**: No custom `TrustManager` or `CertificatePinner` implementations were identified in the primary DEX.

## Native Findings (ghidra-cli)

Two native libraries were analyzed:

### 1. `libuniffi_obfuscate.so`

- **Function Count**: 1291
- **Suspicious Imports**: Includes `__system_property_get` and several `pthread_*` primitives (`pthread_rwlock_wrlock`,
  `pthread_key_create`), often utilized for querying system properties (emulator checks) and threading tasks for
  decryption/obfuscation.
- **Behavior**: Compiled using Rust (`rustc`). The library is responsible for the decryption routines as observed in the
  Java cross-tool correlation layer.

### 2. `libjnidispatch.so`

- **Function Count**: 214
- **Suspicious Imports**: Contains `dlopen`, `dlsym`, `pthread_mutex_init`, `open`, `write`.
- **Behavior**: This is a standard Java Native Access (JNA) dispatch binary utilized for native system calls.

## Indicators of Compromise

**Hardcoded Cryptographic Keys:**

- `a-very-secret-key-for-this-!@#$`

**Root/Evasion Checks (Strings extracted from `libuniffi_obfuscate.so`):**

- `/dev/socket/qemud`, `init.svc.goldfish-logcat`, `init.svc.qemu-adb-keys`
- `/cache/su`, `/data/local/bin/su`, `/system/app/Superuser.apk`
- `/sbin/.magisk/modules/riru-core`, `riru_lsposed`, `zygisk_lsposed`, `zygisk_shamiko`
- `no /proc/self/exe available. Is /proc mounted?`
- `/proc/self/maps`

## Risk Rating

**Critical**
**Rationale**: The application acts as a packed wrapper that dynamically loads executable code at runtime (DCL). It
includes severe manifest misconfigurations (`debuggable=true`, `allowBackup=true`) and embeds sensitive hardcoded
encryption keys in the native layer to conceal its payload. The native payload heavily monitors for analysis tools,
indicating malicious or heavily defended behavior.

## Limitations / Errors

- The Ghidra-CLI bridge required direct use of the `analyzeHeadless` underlying system for the larger Rust-compiled
  binary (`libuniffi_obfuscate.so`) to ensure the project file was cleanly saved, as initial direct imports returned
  silent failure without a project dump.