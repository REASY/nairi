export type AnalysisStatus = "queued" | "running" | "completed" | "failed";

export interface AnalysisRun {
    id: string;
    package_name: string;
    status: AnalysisStatus;
    created_at: string;
    updated_at: string;
}

export interface CreateAnalysisRequest {
    file: File;
    packageName: string;
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
    const formData = new FormData();
    formData.append("file", payload.file);
    formData.append("package_name", payload.packageName);

    const response = await apiFetch("/api/v1/analyses", {
        method: "POST",
        body: formData,
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

export interface ListAnalysesResponse {
    runs: AnalysisRun[];
}

export async function listAnalyses(): Promise<AnalysisRun[]> {
    const response = await apiFetch(`/api/v1/analyses`);
    if (!response.ok) {
        throw new Error(`Failed to fetch analyses: HTTP ${response.status}`);
    }

    const data = await response.json() as ListAnalysesResponse;
    return data.runs;
}

export interface ReportResponse {
    report: string;
}

export async function getAnalysisReport(runId: string): Promise<string> {
    const response = await apiFetch(`/api/v1/analyses/${runId}/report`);
    if (!response.ok) {
        throw new Error(`Failed to fetch report: HTTP ${response.status}`);
    }

    const data = await response.json() as ReportResponse;
    return data.report;
}
