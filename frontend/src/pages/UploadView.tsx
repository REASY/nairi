import {useState} from "react";
import {UploadCloud, CheckCircle, Play, Shield, Loader} from "lucide-react";
import {createAnalysis} from "../api";

export default function UploadView() {
    const [analyzing, setAnalyzing] = useState(false);
    const [progress, setProgress] = useState(0);

    const handleAnalyze = async () => {
        setAnalyzing(true);
        setProgress(1);

        try {
            const {run} = await createAnalysis({package_name: "test-app-1.0"});
            const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
            const eventSource = new EventSource(`${API_BASE_URL}/api/v1/analyses/${run.id}/stream`, {
                withCredentials: true,
            });

            eventSource.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    if (data && data.StatusUpdate) {
                        const status = data.StatusUpdate.status;
                        if (status === "Completed") {
                            setProgress(3);
                            eventSource.close();
                        } else if (status === "Running") {
                            setProgress(1);
                        }
                    }
                } catch (e) {
                    console.error("SSE parsing error", e);
                }
            };

            eventSource.onerror = () => {
                eventSource.close();
            };
        } catch (err) {
            console.error(err);
            setAnalyzing(false);
        }
    };

    return (
        <>
            <div className="glass-panel">
                <h1 style={{marginBottom: "24px"}}>
                    APK Analysis <span style={{color: "var(--text-muted)", fontWeight: 400}}>| New Scan</span>
                </h1>

                <div className="dropzone">
                    <UploadCloud size={64} className="icon"/>
                    <h2>Upload APK for Analysis</h2>
                    <p>
                        Drag & Drop or <span style={{color: "var(--accent-cyan)", cursor: "pointer"}}>Browse</span>
                    </p>
                    <p style={{fontSize: "12px"}}>Supported: .apk, .xapk up to 1GB</p>
                </div>

                <div style={{display: "flex", justifyContent: "center"}}>
                    <button
                        className="btn-primary"
                        onClick={handleAnalyze}
                        disabled={analyzing}
                        style={{minWidth: "200px"}}
                    >
                        {analyzing ? "Initializing..." : "Analyse"}
                    </button>
                </div>
            </div>

            {analyzing && (
                <div className="glass-panel">
                    <h2>Active Run Status</h2>
                    <div className="timeline">
                        <div className={`timeline-step ${progress >= 1 ? "completed" : ""}`}>
                            <div className="timeline-icon">
                                {progress > 1 ? <CheckCircle/> : <CheckCircle opacity={0.5}/>}
                            </div>
                            <div className="label">Static Analysis</div>
                            <div className="status-text">
                                {progress > 1 ? "Completed" : progress === 1 ? "Running" : "Pending"}
                            </div>
                        </div>

                        <div
                            className={`timeline-step ${progress >= 2 ? (progress > 2 ? "completed" : "running") : ""}`}
                        >
                            <div className="timeline-icon">
                                {progress > 2 ? (
                                    <CheckCircle/>
                                ) : progress === 2 ? (
                                    <Loader className="spin"/>
                                ) : (
                                    <Play opacity={0.5}/>
                                )}
                            </div>
                            <div className="label">Runtime Analysis</div>
                            <div className="status-text">
                                {progress > 2 ? "Completed" : progress === 2 ? "Running (65%)" : "Pending"}
                            </div>
                        </div>

                        <div className={`timeline-step ${progress >= 3 ? "completed" : ""}`}>
                            <div className="timeline-icon">
                                {progress >= 3 ? <CheckCircle/> : <Shield opacity={0.5}/>}
                            </div>
                            <div className="label">Network MITM</div>
                            <div className="status-text">{progress >= 3 ? "Completed" : "Pending"}</div>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
}
