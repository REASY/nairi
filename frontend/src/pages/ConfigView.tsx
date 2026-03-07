import {useEffect, useState} from "react";
import {Settings, CheckCircle, AlertCircle} from "lucide-react";

interface ConfigPayload {
    config: {
        model_name: string;
        api_key: string;
        base_url: string;
        static_analysis_image: string;
        runtime_analysis_image: string;
    };
}

async function parseJsonResponse<T>(response: Response, errorPrefix: string): Promise<T> {
    if (!response.ok) {
        throw new Error(`${errorPrefix}: HTTP ${response.status}`);
    }

    return response.json() as Promise<T>;
}

export default function ConfigView() {
    const [config, setConfig] = useState({
        model_name: "",
        api_key: "",
        base_url: "",
        static_analysis_image: "",
        runtime_analysis_image: ""
    });
    const [isLoading, setIsLoading] = useState(true);
    const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "success" | "error">("idle");

    const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";

    useEffect(() => {
        fetch(`${API_BASE_URL}/api/v1/config`, {credentials: "include"})
            .then((res) => parseJsonResponse<ConfigPayload>(res, "Failed to load config"))
            .then((configData) => {
                setConfig(configData.config);
                setIsLoading(false);
            })
            .catch((err) => {
                console.error("Failed to load config", err);
                setIsLoading(false);
            });
    }, [API_BASE_URL]);

    const handleSave = async () => {
        setSaveStatus("saving");
        try {
            const configResponse = await fetch(`${API_BASE_URL}/api/v1/config`, {
                method: "POST",
                credentials: "include",
                headers: {"Content-Type": "application/json"},
                body: JSON.stringify(config),
            });

            if (!configResponse.ok) {
                throw new Error(
                    `Save failed: config=${configResponse.status}`,
                );
            }

            setSaveStatus("success");
            setTimeout(() => setSaveStatus("idle"), 3000);
        } catch (err) {
            console.error(err);
            setSaveStatus("error");
            setTimeout(() => setSaveStatus("idle"), 3000);
        }
    };

    if (isLoading)
        return (
            <div className="glass-panel">
                <p>Loading configuration...</p>
            </div>
        );

    return (
        <div className="glass-panel">
            <div style={{display: "flex", alignItems: "center", gap: "12px", marginBottom: "24px"}}>
                <Settings size={28} color="var(--accent-purple)"/>
                <h1 style={{margin: 0}}>System Configuration</h1>
            </div>

            <div className="form-group">
                <label>AI Model Name</label>
                <input
                    type="text"
                    value={config.model_name}
                    onChange={(e) => setConfig({...config, model_name: e.target.value})}
                />
            </div>

            <div className="form-group">
                <label>Gemini API Key</label>
                <input
                    type="password"
                    value={config.api_key}
                    onChange={(e) => setConfig({...config, api_key: e.target.value})}
                />
            </div>

            <div className="form-group">
                <label>Google Gemini Base URL</label>
                <input
                    type="text"
                    value={config.base_url}
                    onChange={(e) => setConfig({...config, base_url: e.target.value})}
                />
            </div>

            <h2 style={{marginTop: "32px", marginBottom: "16px"}}>Docker Infrastructure</h2>

            <div className="form-group">
                <label>Static Analysis Docker Image</label>
                <input
                    type="text"
                    value={config.static_analysis_image}
                    onChange={(e) => setConfig({...config, static_analysis_image: e.target.value})}
                    placeholder="e.g. nairi/static-analysis:dev"
                />
            </div>

            <div className="form-group">
                <label>Runtime Analysis Docker Image</label>
                <input
                    type="text"
                    value={config.runtime_analysis_image}
                    onChange={(e) => setConfig({...config, runtime_analysis_image: e.target.value})}
                    placeholder="e.g. nairi/runtime-analysis:dev"
                />
            </div>

            <div style={{display: "flex", alignItems: "center", gap: "16px", marginTop: "16px"}}>
                <button className="btn-primary" onClick={handleSave} disabled={saveStatus === "saving"}>
                    {saveStatus === "saving" ? "Saving..." : "Save Configuration"}
                </button>
                {saveStatus === "success" && <span style={{
                    color: "var(--accent-green)",
                    display: "flex",
                    alignItems: "center",
                    gap: "6px",
                    fontSize: "14px"
                }}><CheckCircle size={16}/> Saved successfully</span>}
                {saveStatus === "error" && <span style={{
                    color: "var(--accent-pink)",
                    display: "flex",
                    alignItems: "center",
                    gap: "6px",
                    fontSize: "14px"
                }}><AlertCircle size={16}/> Error saving config</span>}
            </div>
        </div>
    );
}
