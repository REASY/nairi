=== Tool Execution Log ===

## 1. Tool checks

```bash
$ apktool --version
3.0.1
$ jadx --version
1.5.5
$ ghidra --version
ghidra 0.1.9
```

## 2. APKTool phase

```bash
$ apktool d -f /workspace/target.apk -o /workspace/decompiled/apktool
I: Using Apktool 3.0.1 on target.apk with 8 threads
I: Baksmaling classes.dex...
I: Baksmaling classes2.dex...
I: Baksmaling classes3.dex...
I: Baksmaling classes4.dex...
I: Baksmaling classes5.dex...
I: Baksmaling classes6.dex...
I: Baksmaling classes7.dex...
I: Loading resource table...
I: Decoding value resources...
I: Loading resource table from file: /root/.local/share/apktool/framework/1.apk
I: Decoding file resources...
I: Generating values XMLs...
I: Decoding AndroidManifest.xml with resources...
I: Copying original files...
I: Copying assets...
I: Copying lib...
I: Copying unknown files...
```

## 3. JADX phase

```bash
$ jadx --deobf /workspace/target.apk -d /workspace/decompiled/jadx
INFO  - loading ...
INFO  - processing ...
INFO  - progress: 0 of 3523 (0%)
INFO  - progress: 955 of 3523 (27%)
INFO  - progress: 1197 of 3523 (33%)
INFO  - progress: 1583 of 3523 (44%)
INFO  - progress: 1983 of 3523 (56%)
INFO  - progress: 2193 of 3523 (62%)
INFO  - progress: 2590 of 3523 (73%)
INFO  - progress: 2811 of 3523 (79%)
INFO  - progress: 2988 of 3523 (84%)
INFO  - progress: 3149 of 3523 (89%)
INFO  - progress: 3272 of 3523 (92%)
                                                             
ERROR - finished with errors, count: 27
```

## 4. Ghidra-CLI phase

```bash
$ ghidra import /workspace/decompiled/apktool/lib/arm64-v8a/libuniffi_obfuscate.so --project analysis_project
Starting Ghidra bridge...
Error: Ghidra process exited with status: exit status: 1: ERROR Abort due to Headless analyzer error: Directory not found: /root/.cache/ghidra-cli/projects (HeadlessAnalyzer) java.io.FileNotFoundException: Directory not found: /root/.cache/ghidra-cli/projects
$ ghidra analyze --project analysis_project --program libuniffi_obfuscate.so
Starting Ghidra bridge...
Error: Ghidra process exited with status: exit status: 1: ERROR Abort due to Headless analyzer error: Could not find project: /root/.cache/ghidra-cli/projects/analysis_project (HeadlessAnalyzer) java.io.IOException: Could not find project: /root/.cache/ghidra-cli/projects/analysis_project
$ ghidra import /workspace/decompiled/apktool/lib/arm64-v8a/libjnidispatch.so --project analysis_project
Starting Ghidra bridge...
Error: Ghidra process exited with status: exit status: 1: ERROR Abort due to Headless analyzer error: Directory not found: /root/.cache/ghidra-cli/projects (HeadlessAnalyzer) java.io.FileNotFoundException: Directory not found: /root/.cache/ghidra-cli/projects
$ ghidra analyze --project analysis_project --program libjnidispatch.so
Starting Ghidra bridge...
Error: Ghidra process exited with status: exit status: 1: ERROR Abort due to Headless analyzer error: Could not find project: /root/.cache/ghidra-cli/projects/analysis_project (HeadlessAnalyzer) java.io.IOException: Could not find project: /root/.cache/ghidra-cli/projects/analysis_project
```
