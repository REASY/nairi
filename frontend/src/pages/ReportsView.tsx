import {useEffect, useState} from "react";
import {useParams, useNavigate} from "react-router-dom";
import {FileText, Download, Activity, CheckCircle} from "lucide-react";
import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import "highlight.js/styles/github-dark.css";
import {listAnalyses, getAnalysisReport, AnalysisRun} from "../api";

export default function ReportsView() {
    const {runId} = useParams<{ runId?: string }>();
    const navigate = useNavigate();

    const [runs, setRuns] = useState<AnalysisRun[]>([]);
    const [report, setReport] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);
    const [reportLoading, setReportLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Fetch all completed runs for the sidebar
    useEffect(() => {
        listAnalyses()
            .then(data => {
                const completed = data.filter(r => r.status === "completed");
                setRuns(completed);
                setLoading(false);

                // If a runId isn't provided but we have completed runs, select the first one automatically
                if (!runId && completed.length > 0) {
                    navigate(`/reports/${completed[0].id}`, {replace: true});
                }
            })
            .catch(err => {
                console.error(err);
                setError(err.message);
                setLoading(false);
            });
    }, [navigate, runId]);

    // Fetch the specific report when runId changes
    useEffect(() => {
        if (!runId) return;

        setReportLoading(true);
        getAnalysisReport(runId)
            .then(text => {
                setReport(text);
                setError(null);
            })
            .catch(err => {
                console.error(err);
                setError(err.message);
                setReport(null);
            })
            .finally(() => {
                setReportLoading(false);
            });
    }, [runId]);

    const activeRun = runs.find(r => r.id === runId);

    const handleDownload = () => {
        if (!report || !activeRun) return;
        const blob = new Blob([report], {type: "text/markdown"});
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `${activeRun.package_name}_report.md`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    };

    if (loading) {
        return <div className="glass-panel"><p>Loading reports...</p></div>;
    }

    if (runs.length === 0) {
        return (
            <div className="glass-panel" style={{height: "60vh", display: "grid", placeItems: "center"}}>
                <div style={{textAlign: "center", color: "var(--text-muted)"}}>
                    <Activity size={48} opacity={0.2} style={{margin: "0 auto 16px"}}/>
                    <p>No completed analyses available yet.</p>
                </div>
            </div>
        );
    }

    return (
        <div style={{display: "flex", gap: "24px", height: "calc(100vh - 120px)"}}>
            {/* Sidebar for completed runs */}
            <div className="glass-panel"
                 style={{width: "300px", display: "flex", flexDirection: "column", padding: "16px", overflowY: "auto"}}>
                <h3 style={{
                    marginBottom: "16px",
                    color: "var(--text-color)",
                    display: "flex",
                    alignItems: "center",
                    gap: "8px"
                }}>
                    <CheckCircle size={18} color="var(--success-color)"/>
                    Completed Scans
                </h3>
                <div style={{display: "flex", flexDirection: "column", gap: "8px"}}>
                    {runs.map(r => (
                        <div
                            key={r.id}
                            onClick={() => navigate(`/reports/${r.id}`)}
                            style={{
                                padding: "12px",
                                borderRadius: "8px",
                                cursor: "pointer",
                                background: r.id === runId ? "rgba(0, 240, 255, 0.1)" : "rgba(255, 255, 255, 0.03)",
                                border: r.id === runId ? "1px solid var(--accent-cyan)" : "1px solid transparent",
                                transition: "all 0.2s ease"
                            }}
                        >
                            <div style={{
                                fontWeight: 500,
                                color: r.id === runId ? "var(--accent-cyan)" : "var(--text-color)",
                                marginBottom: "4px",
                                wordBreak: "break-all"
                            }}>
                                {r.package_name}
                            </div>
                            <div style={{fontSize: "0.8em", color: "var(--text-muted)"}}>
                                {new Date(r.created_at).toLocaleDateString()}
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Main Report Area */}
            <div className="glass-panel"
                 style={{flex: 1, display: "flex", flexDirection: "column", overflow: "hidden"}}>
                {reportLoading ? (
                    <div style={{display: "flex", justifyContent: "center", alignItems: "center", height: "100%"}}>
                        <p style={{color: "var(--text-muted)"}}>Loading report content...</p>
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
                ) : !report ? (
                    <div style={{display: "flex", justifyContent: "center", alignItems: "center", height: "100%"}}>
                        <p style={{color: "var(--text-muted)"}}>Select a report to view.</p>
                    </div>
                ) : (
                    <>
                        <div style={{
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "space-between",
                            marginBottom: "20px",
                            paddingBottom: "16px",
                            borderBottom: "1px solid var(--border-glass)"
                        }}>
                            <div style={{display: "flex", alignItems: "center", gap: "12px"}}>
                                <FileText size={24} color="var(--accent-purple)"/>
                                <h1 style={{margin: 0, fontSize: "1.5rem"}}>
                                    {activeRun?.package_name}
                                </h1>
                            </div>
                            <button className="btn-secondary" onClick={handleDownload}
                                    style={{display: "flex", alignItems: "center", gap: "8px"}}>
                                <Download size={16}/>
                                Download MD
                            </button>
                        </div>

                        <div style={{
                            flex: 1,
                            overflowY: "auto",
                            paddingRight: "16px",
                            fontFamily: "system-ui, -apple-system, sans-serif",
                            lineHeight: 1.6
                        }}>
                            <div className="markdown-preview">
                                <Markdown rehypePlugins={[rehypeHighlight]}>{report}</Markdown>
                            </div>
                        </div>
                    </>
                )}
            </div>
        </div>
    );
}
