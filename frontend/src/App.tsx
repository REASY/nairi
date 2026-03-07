import { FormEvent, useEffect, useMemo, useState } from "react";
import { AnalysisRun, AnalysisStatus, createAnalysis, getAnalysis } from "./api";

const TERMINAL_STATES: AnalysisStatus[] = ["completed", "failed"];

export default function App() {
  const [packageName, setPackageName] = useState("");
  const [runs, setRuns] = useState<AnalysisRun[]>([]);
  const [activeRunId, setActiveRunId] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const activeRun = useMemo(
    () => runs.find((run) => run.id === activeRunId) ?? null,
    [activeRunId, runs],
  );

  useEffect(() => {
    if (!activeRunId) {
      return;
    }

    const timer = window.setInterval(async () => {
      try {
        const latestRun = await getAnalysis(activeRunId);
        setRuns((previous) =>
          previous.map((run) => (run.id === latestRun.id ? latestRun : run)),
        );

        if (TERMINAL_STATES.includes(latestRun.status)) {
          window.clearInterval(timer);
        }
      } catch (error) {
        const message =
          error instanceof Error ? error.message : "Failed to refresh run status.";
        setErrorMessage(message);
      }
    }, 2000);

    return () => window.clearInterval(timer);
  }, [activeRunId]);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setErrorMessage(null);
    setIsSubmitting(true);

    try {
      const created = await createAnalysis({
        package_name: packageName.trim(),
      });
      setRuns((previous) => [created.run, ...previous]);
      setActiveRunId(created.run.id);
      setPackageName("");
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to start analysis run.";
      setErrorMessage(message);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className="page">
      <section className="panel">
        <h1>NAIRI Console</h1>
        <p>Upload APK in future UI. For now, submit package/sample name and run analysis.</p>

        <form className="row" onSubmit={onSubmit}>
          <input
            placeholder="Package or sample name (e.g. com.example.malware)"
            value={packageName}
            onChange={(event) => setPackageName(event.target.value)}
            required
          />
          <button type="submit" disabled={isSubmitting}>
            {isSubmitting ? "Starting..." : "Analyse"}
          </button>
        </form>

        {errorMessage ? <p className="error">{errorMessage}</p> : null}
      </section>

      <section className="panel">
        <h2>Active Run</h2>
        {activeRun ? (
          <dl className="details">
            <div>
              <dt>Run ID</dt>
              <dd>{activeRun.id}</dd>
            </div>
            <div>
              <dt>Package</dt>
              <dd>{activeRun.package_name}</dd>
            </div>
            <div>
              <dt>Status</dt>
              <dd>{activeRun.status}</dd>
            </div>
            <div>
              <dt>Updated</dt>
              <dd>{new Date(activeRun.updated_at).toLocaleString()}</dd>
            </div>
          </dl>
        ) : (
          <p>No active run.</p>
        )}
      </section>

      <section className="panel">
        <h2>Recent Runs</h2>
        {runs.length === 0 ? (
          <p>No runs started yet.</p>
        ) : (
          <ul className="runs">
            {runs.map((run) => (
              <li key={run.id}>
                <button type="button" onClick={() => setActiveRunId(run.id)}>
                  <span>{run.package_name}</span>
                  <span>{run.status}</span>
                  <span>{new Date(run.created_at).toLocaleTimeString()}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
