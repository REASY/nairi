import {useEffect, useState} from "react";
import {Settings} from "lucide-react";

interface ConfigPayload {
    config: {
        model_name: string;
        api_key: string;
        base_url: string;
    };
}

interface PromptPayload {
    prompt: {
        content: string;
    };
}

async function parseJsonResponse<T>(response: Response, errorPrefix: string): Promise<T> {
    if (!response.ok) {
        throw new Error(`${errorPrefix}: HTTP ${response.status}`);
    }

    return response.json() as Promise<T>;
}

export default function ConfigView() {
    const [config, setConfig] = useState({model_name: "", api_key: "", base_url: ""});
    const [prompt, setPrompt] = useState("");
    const [isLoading, setIsLoading] = useState(true);

    const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";

    useEffect(() => {
        Promise.all([
            fetch(`${API_BASE_URL}/api/v1/config`, {credentials: "include"}).then((res) =>
                parseJsonResponse<ConfigPayload>(res, "Failed to load config"),
            ),
            fetch(`${API_BASE_URL}/api/v1/prompts/static_analysis`, {
                credentials: "include",
            }).then((res) => parseJsonResponse<PromptPayload>(res, "Failed to load prompt")),
        ])
            .then(([configData, promptData]) => {
                setConfig(configData.config);
                setPrompt(promptData.prompt.content);
                setIsLoading(false);
            })
            .catch((err) => {
                console.error("Failed to load config", err);
                setIsLoading(false);
            });
    }, [API_BASE_URL]);

    const handleSave = async () => {
        try {
            const [configResponse, promptResponse] = await Promise.all([
                fetch(`${API_BASE_URL}/api/v1/config`, {
                    method: "POST",
                    credentials: "include",
                    headers: {"Content-Type": "application/json"},
                    body: JSON.stringify(config),
                }),
                fetch(`${API_BASE_URL}/api/v1/prompts/static_analysis`, {
                    method: "POST",
                    credentials: "include",
                    headers: {"Content-Type": "application/json"},
                    body: JSON.stringify({content: prompt}),
                }),
            ]);

            if (!configResponse.ok || !promptResponse.ok) {
                throw new Error(
                    `Save failed: config=${configResponse.status}, prompt=${promptResponse.status}`,
                );
            }

            alert("Config saved!");
        } catch (err) {
            console.error(err);
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

            <div className="form-group" style={{marginTop: "16px"}}>
                <label>Static Analysis Engine Prompt</label>
                <textarea
                    rows={12}
                    value={prompt}
                    onChange={(e) => setPrompt(e.target.value)}
                    style={{
                        width: "100%",
                        padding: "12px",
                        background: "rgba(0, 0, 0, 0.4)",
                        border: "1px solid var(--border-glass)",
                        color: "var(--text-primary)",
                        borderRadius: "8px",
                        fontFamily: "monospace",
                        resize: "vertical",
                        marginTop: "8px",
                    }}
                />
            </div>

            <button className="btn-primary" onClick={handleSave} style={{marginTop: "16px"}}>
                Save Configuration
            </button>
        </div>
    );
}
