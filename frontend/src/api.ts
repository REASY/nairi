export type AnalysisStatus = "queued" | "running" | "completed" | "failed";

export interface AnalysisRun {
  id: string;
  package_name: string;
  status: AnalysisStatus;
  created_at: string;
  updated_at: string;
}

export interface CreateAnalysisRequest {
  package_name: string;
}

export interface CreateAnalysisResponse {
  run: AnalysisRun;
}

export interface AuthUser {
    sub: string;
    email: string;
    name?: string | null;
    picture?: string | null;
}

interface CurrentUserResponse {
    user: AuthUser;
}

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";

function buildApiUrl(path: string): string {
    return `${API_BASE_URL}${path}`;
}

async function apiFetch(path: string, init: RequestInit = {}): Promise<Response> {
    return fetch(buildApiUrl(path), {
        ...init,
        credentials: "include",
    });
}

export function getGoogleLoginUrl(): string {
    return buildApiUrl("/api/v1/auth/google/login");
}

export async function getCurrentUser(): Promise<AuthUser> {
    const response = await apiFetch("/api/v1/auth/me");
    if (!response.ok) {
        throw new Error(`Failed to get current user: HTTP ${response.status}`);
    }

    const body = (await response.json()) as CurrentUserResponse;
    return body.user;
}

export async function logoutCurrentUser(): Promise<void> {
    const response = await apiFetch("/api/v1/auth/logout", {
        method: "POST",
    });

    if (!response.ok && response.status !== 204) {
        throw new Error(`Failed to logout: HTTP ${response.status}`);
    }
}

export async function createAnalysis(
  payload: CreateAnalysisRequest,
): Promise<CreateAnalysisResponse> {
    const response = await apiFetch("/api/v1/analyses", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    throw new Error(`Failed to create analysis: HTTP ${response.status}`);
  }

  return response.json() as Promise<CreateAnalysisResponse>;
}

export async function getAnalysis(runId: string): Promise<AnalysisRun> {
    const response = await apiFetch(`/api/v1/analyses/${runId}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch analysis: HTTP ${response.status}`);
  }

  return response.json() as Promise<AnalysisRun>;
}
