import {useEffect, useState} from "react";
import {Activity, Play, CheckCircle, AlertCircle, Clock} from "lucide-react";
import {listAnalyses, AnalysisRun} from "../api";

export default function LiveRunsView() {
    const [runs, setRuns] = useState<AnalysisRun[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchRuns = async () => {
        try {
            const data = await listAnalyses();
            setRuns(data);
            setError(null);
        } catch (err: any) {
            console.error(err);
            setError(err.message || "Failed to load runs");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchRuns();
        const interval = setInterval(fetchRuns, 5000); // Polling every 5s for live updates
        return () => clearInterval(interval);
    }, []);

    const getStatusIcon = (status: string) => {
        switch (status) {
            case "running":
                return <Play size={16} className="spin" color="var(--accent-cyan)"/>;
            case "completed":
                return <CheckCircle size={16} color="var(--success-color)"/>;
            case "failed":
                return <AlertCircle size={16} color="var(--error-color)"/>;
            default:
                return <Clock size={16} color="var(--text-muted)"/>;
        }
    };

    return (
        <div className="glass-panel" style={{minHeight: "60vh"}}>
            <div style={{display: "flex", alignItems: "center", gap: "12px", marginBottom: "24px"}}>
                <Activity size={28} color="var(--accent-cyan)"/>
                <h1>Live Runs</h1>
            </div>

            {loading && runs.length === 0 ? (
                <div style={{display: "flex", justifyContent: "center", padding: "40px"}}>
                    <p style={{color: "var(--text-muted)"}}>Loading analysis history...</p>
                </div>
            ) : error ? (
                <div style={{
                    padding: "16px",
                    background: "rgba(255, 68, 68, 0.1)",
                    borderRadius: "8px",
                    color: "var(--error-color)"
                }}>
                    {error}
                </div>
            ) : runs.length === 0 ? (
                <div style={{textAlign: "center", padding: "60px 20px", color: "var(--text-muted)"}}>
                    <Activity size={48} opacity={0.2} style={{margin: "0 auto 16px"}}/>
                    <p>No analyses have been run yet.</p>
                </div>
            ) : (
                <div style={{overflowX: "auto"}}>
                    <table style={{width: "100%", borderCollapse: "collapse", textAlign: "left"}}>
                        <thead>
                        <tr style={{borderBottom: "1px solid var(--border-glass)", color: "var(--text-muted)"}}>
                            <th style={{padding: "12px 16px", fontWeight: 500}}>Target Package</th>
                            <th style={{padding: "12px 16px", fontWeight: 500}}>Status</th>
                            <th style={{padding: "12px 16px", fontWeight: 500}}>Started</th>
                            <th style={{padding: "12px 16px", fontWeight: 500}}>Action</th>
                        </tr>
                        </thead>
                        <tbody>
                        {runs.map((run) => (
                            <tr key={run.id} style={{borderBottom: "1px solid rgba(255,255,255,0.05)"}}>
                                <td style={{padding: "16px", fontWeight: 500, color: "var(--text-color)"}}>
                                    {run.package_name}
                                </td>
                                <td style={{padding: "16px"}}>
                                    <div style={{display: "flex", alignItems: "center", gap: "8px"}}>
                                        {getStatusIcon(run.status)}
                                        <span style={{
                                            color: run.status === 'completed' ? 'var(--success-color)' :
                                                run.status === 'failed' ? 'var(--error-color)' :
                                                    run.status === 'running' ? 'var(--accent-cyan)' : 'var(--text-muted)'
                                        }}>
                                                {run.status.charAt(0).toUpperCase() + run.status.slice(1)}
                                            </span>
                                    </div>
                                </td>
                                <td style={{padding: "16px", color: "var(--text-muted)", fontSize: "0.9em"}}>
                                    {new Date(run.created_at).toLocaleString()}
                                </td>
                                <td style={{padding: "16px"}}>
                                    <button
                                        className="btn-secondary"
                                        style={{padding: "6px 12px", fontSize: "0.85em"}}
                                        disabled={run.status !== 'completed'}
                                    >
                                        View Report
                                    </button>
                                </td>
                            </tr>
                        ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    );
}
