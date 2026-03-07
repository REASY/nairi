# NAIRI AI-Driven Static Analysis Execution

## 1. Overview

While the [Static Analysis Design](nairi-static-analysis.md) defines *what* must happen, this document specifies *how*
the AI Agent systematically drives the static analysis phase. Instead of a rigid script, the static analysis phase is
executed by an autonomous coding sub-agent (e.g., Gemini-driven interpreter) that has access to specific tools.

## 2. Agent Environment & Tools

The Static Analysis Agent is implemented by baking an autonomous CLI agent (e.g., `gemini-cli` or `aider`) directly into
an isolated Docker container alongside the necessary reverse-engineering toolchain.

The NAIRI Orchestrator simply mounts the APK and an output directory, and runs a command similar to:

```bash
docker run --rm -v /path/to/apk:/apk/target.apk -v /path/to/output:/output \
  nairi-static:latest \
  gemini-cli --prompt "Decompile /apk/target.apk, analyze interesting code chunks, run Ghidra on native libs, and write a summary report to /output/report.md"
```

The embedded agent operates within the sandbox and is granted the following tools:

1. **`run_terminal_command`**: Execute arbitrary shell commands (restricted to the sandbox workspace).
2. **`read_file_chunk`**: Read specific line ranges of a file. Essential for analyzing large `.smali` or `.java`
   decompiled files without blowing up context window.
3. **`grep_search`**: Search the workspace for regex/strings (e.g., finding `http://`, `AES`, `Cipher`, or specific API
   calls).
4. **`write_file`**: Write files to the workspace (e.g., producing headless Ghidra scripts).

## 3. The Autonomous Inspection Loop

The Agent is prompted with a high-level goal: *"Analyze the provided APK. Identify malicious indicators, networking,
crypto usage, and native library behavior. Produce a final Markdown report."*

### Step 3.1: Unpacking

The agent autonomously decides to run:

```bash
apktool d target.apk -o /workspace/decompiled
```

It then reads `AndroidManifest.xml` to identify the Entry Points (Main Activity, Services, Broadcast Receivers) and
requested permissions.

### Step 3.2: Chunked Code Analysis

Rather than absorbing the entire codebase, the AI agent performs targeted investigation:

1. **Reconnaissance**: The agent runs `grep -r "http" /workspace/decompiled/smali` to find network calls.
2. **Chunk Reading**: For interesting files identified by grep, the agent uses `read_file_chunk` to read the context
   around the match (e.g., lines 100-150).
3. **Summarization**: The agent internally summarizes the behavior of that chunk (e.g., "This class sends device
   telemetry to a hardcoded IP").
4. **Iterative Tracing**: If the agent finds an obfuscated method call, it recursively searches for the method
   definition and reads that file chunk.

### Step 3.3: Headless Ghidra Interaction

When the agent discovers a `System.loadLibrary("foo")` call in the Android code, it investigates the native library:

1. The agent locates `lib/arm64-v8a/libfoo.so`.
2. The agent writes a custom Python script for Ghidra (`analyze_lib.py`) tailored to what it suspects the library does.
3. The agent executes Ghidra headlessly:

```bash
analyzeHeadless /ghidra_project MyProject -import libfoo.so -postScript analyze_lib.py
```

4. The agent reads the output script logs to conclude behavior (e.g., finding anti-debugging tricks or JNI native method
   registrations).

## 4. Final Aggregation

Once the agent exhausts its investigation leads or reaches a time/token limit, it compiles its findings. It produces a
structured Markdown artifact containing:

- Confirmed malicious or suspicious capabilities.
- Code snippets (evidence) justifying the findings.
- The list of files and native libraries analyzed.

This output is passed back to the main NAIRI Orchestrator to be combined with the Runtime Analysis results.
