import {useState, useRef} from "react";
import {UploadCloud, CheckCircle, Play, Shield, Loader, File as FileIcon} from "lucide-react";
import {createAnalysis} from "../api";

export default function UploadView() {
    const [file, setFile] = useState<File | null>(null);
    const [packageName, setPackageName] = useState("");
    const [isDragging, setIsDragging] = useState(false);
    const [analyzing, setAnalyzing] = useState(false);
    const [progress, setProgress] = useState(0);
    const fileInputRef = useRef<HTMLInputElement>(null);

    const handleAnalyze = async () => {
        if (!file || !packageName.trim()) {
            alert("Please select an APK and enter a package name.");
            return;
        }

        setAnalyzing(true);
        setProgress(1);

        try {
            const {run} = await createAnalysis({file, packageName});
            const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
            const eventSource = new EventSource(`${API_BASE_URL}/api/v1/analyses/${run.id}/stream`, {
                withCredentials: true,
            });

            eventSource.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    if (data && data.StatusUpdate) {
                        const status = typeof data.StatusUpdate.status === 'string'
                            ? data.StatusUpdate.status.toLowerCase()
                            : String(data.StatusUpdate.status);

                        if (status === "completed") {
                            setProgress(3);
                            eventSource.close();
                            setAnalyzing(false);
                            alert("Analysis completed successfully! You can view the report in the Live Runs or Reports tab.");
                        } else if (status === "running") {
                            setProgress(1);
                        } else if (status === "failed") {
                            setAnalyzing(false);
                            eventSource.close();
                            alert("Analysis failed!");
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

                <div
                    className={`dropzone ${isDragging ? "dragging" : ""} ${file ? "has-file" : ""}`}
                    onDragOver={(e) => {
                        e.preventDefault();
                        setIsDragging(true);
                    }}
                    onDragLeave={() => setIsDragging(false)}
                    onDrop={(e) => {
                        e.preventDefault();
                        setIsDragging(false);
                        const droppedFile = e.dataTransfer.files[0];
                        if (droppedFile && (droppedFile.name.endsWith(".apk") || droppedFile.name.endsWith(".xapk"))) {
                            setFile(droppedFile);
                            if (!packageName) setPackageName(droppedFile.name.replace(/\.x?apk$/, ""));
                        }
                    }}
                    onClick={() => fileInputRef.current?.click()}
                    style={{
                        borderColor: isDragging ? "var(--accent-cyan)" : file ? "var(--accent-purple)" : "var(--border-glass)",
                        background: isDragging ? "rgba(0, 240, 255, 0.05)" : file ? "rgba(187, 134, 252, 0.05)" : "rgba(0,0,0,0.2)"
                    }}
                >
                    <input
                        type="file"
                        accept=".apk,.xapk"
                        ref={fileInputRef}
                        style={{display: "none"}}
                        onChange={(e) => {
                            const selectedFile = e.target.files?.[0];
                            if (selectedFile) {
                                setFile(selectedFile);
                                if (!packageName) setPackageName(selectedFile.name.replace(/\.x?apk$/, ""));
                            }
                        }}
                    />

                    {file ? (
                        <>
                            <FileIcon size={64} className="icon" color="var(--accent-purple)"/>
                            <h2 style={{color: "var(--text-color)"}}>{file.name}</h2>
                            <p style={{color: "var(--text-muted)"}}>
                                {(file.size / (1024 * 1024)).toFixed(2)} MB • Ready for analysis
                            </p>
                        </>
                    ) : (
                        <>
                            <UploadCloud size={64} className="icon"/>
                            <h2>Upload APK for Analysis</h2>
                            <p>
                                Drag & Drop or <span
                                style={{color: "var(--accent-cyan)", cursor: "pointer"}}>Browse</span>
                            </p>
                            <p style={{fontSize: "12px"}}>Supported: .apk, .xapk up to 1GB</p>
                        </>
                    )}
                </div>

                {file && (
                    <div className="form-group" style={{marginTop: "24px", maxWidth: "400px", margin: "24px auto"}}>
                        <label>Target Package Name</label>
                        <input
                            type="text"
                            value={packageName}
                            onChange={(e) => setPackageName(e.target.value)}
                            placeholder="e.g. com.example.app"
                        />
                    </div>
                )}

                <div style={{display: "flex", justifyContent: "center", marginTop: file ? "0" : "24px"}}>
                    <button
                        className="btn-primary"
                        onClick={handleAnalyze}
                        disabled={analyzing || !file}
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
