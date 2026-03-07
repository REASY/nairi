# NAIRI, Native Android Inspection & Risk Intelligence

**NAIRI** is an AI-driven, orchestrated mobile application vulnerability analysis tool. It streamlines the process of
reverse engineering Android APKs by leveraging large language models (Google Gemini) alongside specialized static and
runtime analysis environments.

## Architecture

NAIRI uses a modernized, full-stack architecture:

- **Frontend (`/frontend`)**: A beautiful, dynamic React Single Page Application (SPA) built with Vite and TypeScript.
  It provides a real-time dashboard for uploading APKs, modifying AI prompts, and monitoring analysis tasks via
  Server-Sent Events (SSE).
- **Backend (`/backend`)**: A robust orchestration server built in Rust using the complete Axum web framework ecosystem.
  It handles state management, SQLite databasing for analysis history, and manages Docker container lifecycles for the
  actual analysis tools.
- **Analysis Environments (`/docker`)**: Isolated toolchains containerized for reproducibility and security. Includes:
    - `images/static-analysis`: Toolchain containing `apktool`, `ghidra-cli`, and AI integrations.
    - `images/runtime-analysis`: Toolchain configured with ADB and active networking utilities for monitoring app
      behavior dynamically.

## Quick Start

### Prerequisites

- Node.js (v18+)
- Rust (1.75+)
- Docker
- Gemini API Key

### Start the Backend

```bash
cd backend
# Make sure to run the nairi-server binary from the workspace
cargo run -p nairi-server
```

The backend server will automatically spin up on `http://localhost:8080`.

### Start the Frontend

In a new terminal:

```bash
cd frontend
npm install
npm run dev
```

The frontend UI will be accessible at `http://localhost:5173`.

### Setup

1. Open the UI, click **System Config** in the sidebar.
2. Enter your `Gemini API Key` and confirm your preferred `Model Name`.
3. Save configuration.

You can now click **Upload**, drag & drop your APK, and let NAIRI spin up the necessary container analysis pipelines!

## Key Features

- **Real-Time Job Orchestration**: Streamlined visibility into long-running tasks.
- **Dynamic Prompt Engineering**: Directly view and edit the engineering prompts fed to the AI.
- **Persistent AI Reporting**: History of all target scans and generated Markdown reports, available for download or
  immediate reading within the app payload viewer. 
