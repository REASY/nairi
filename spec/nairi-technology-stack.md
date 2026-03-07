# NAIRI Technology Stack Specification

## 1. Scope

This document defines mandatory implementation technologies for NAIRI application layers.

## 2. Mandatory Technology Choices

1. **Frontend**
   1. Language: `TypeScript`.
   2. Framework: `React`.
2. **Backend**
   1. Language: `Rust`.
   2. Web framework: `Axum`.

These choices are normative for MVP and production baseline unless superseded by an approved architecture decision record.

## 3. Layer Mapping

1. **Web UI Layer (Frontend, TypeScript + React)**
   1. APK upload and `Analyse` initiation.
   2. Live run status and stage timeline.
   3. Final report and artifact navigation.
2. **API and Orchestration Layer (Backend, Rust + Axum)**
   1. Authenticated API endpoints for intake, run control, and report retrieval.
   2. Orchestration integration for AI-driven stage execution.
   3. Event streaming APIs for real-time status updates.

## 4. API Contract and Interop Requirements

1. Backend APIs must expose stable versioned contracts.
2. Frontend API clients must be typed from shared API schema artifacts.
3. Request/response payloads must support run correlation IDs and evidence IDs.

## 5. Reliability and Performance Expectations

1. Backend services use async Rust execution model for concurrent analysis workflow coordination.
2. Frontend must handle long-running analysis sessions with resilient reconnect behavior for live status.

## 6. Security Expectations by Stack

1. Frontend must enforce secure upload constraints and strict input validation at UI boundaries.
2. Backend must enforce authentication/authorization and policy checks before any analysis action is executed.
3. Secrets and signing keys must never be embedded in frontend bundles.

## 7. Repository and Delivery Guidance

Recommended workspace layout:

1. `frontend/` for TypeScript + React application.
2. `backend/` for Rust + Axum services.
3. `backend/runtime/ebpf/` for probes/parsers/runtime tracing assets.
4. `research/trace-experiments/` for manual experiment runners and ad-hoc analysis workflows.
5. `spec/` for contracts and architecture docs.
6. Shared API schema directory for typed client generation and backend contract validation.
