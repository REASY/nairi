# NAIRI Network Inspection and Pinning Bypass

## 1. MITM Network Inspection

1. Route emulator traffic through a controlled MITM proxy.
2. Install analysis CA in emulator trust configuration.
3. Capture:
   1. DNS/SNI/domain metadata.
   2. TLS handshake and certificate metadata.
   3. HTTP method, path, headers, and payload under policy.

## 2. Redaction and Retention Policy

1. Redact secrets in headers/payloads by rule.
2. Mark raw payload storage as optional by run profile.
3. Tag all captures with retention class.

## 3. Pinning Detection

Pinning is considered active when:

1. TLS requests fail with validation errors despite trusted lab CA.
2. Frida TLS hooks indicate custom trust manager/certificate checks.
3. App behavior diverges between direct and MITM routes.

## 4. Pinning Bypass Strategy

1. **Stage 1: Runtime bypass (preferred)**
   1. Apply Frida hook-based trust override.
   2. Re-test blocked requests.
2. **Stage 2: Static patch fallback**
   1. Decompile app.
   2. Patch pinning logic in Java/smali or native bridge points.
   3. Rebuild and resign APK.
   4. Redeploy and rerun runtime analysis.

## 5. Required Pinning Report

1. Detection evidence.
2. Bypass method used and success/failure state.
3. Patched artifacts and patch metadata.
4. Pre/post network behavior comparison.
