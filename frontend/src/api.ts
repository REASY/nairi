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

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";

export async function createAnalysis(
  payload: CreateAnalysisRequest,
): Promise<CreateAnalysisResponse> {
  const response = await fetch(`${API_BASE_URL}/api/v1/analyses`, {
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
  const response = await fetch(`${API_BASE_URL}/api/v1/analyses/${runId}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch analysis: HTTP ${response.status}`);
  }

  return response.json() as Promise<AnalysisRun>;
}
