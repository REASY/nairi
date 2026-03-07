import {useEffect, useState} from "react";
import {FileText, CheckCircle, AlertCircle} from "lucide-react";
import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import "highlight.js/styles/github-dark.css";

type PromptName = "static_analysis" | "runtime_analysis";

const PROMPT_OPTIONS: Array<{ name: PromptName; label: string }> = [
    {name: "static_analysis", label: "Static Analysis Prompt"},
    {name: "runtime_analysis", label: "Runtime Analysis Prompt"},
];

export default function PromptsView() {
    const [selectedPrompt, setSelectedPrompt] = useState<PromptName>("static_analysis");
    const [prompt, setPrompt] = useState("");
    const [savedPrompt, setSavedPrompt] = useState("");
    const [isLoading, setIsLoading] = useState(true);
    const [isEditing, setIsEditing] = useState(false);
    const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "success" | "error">("idle");

    const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";

    useEffect(() => {
        setIsLoading(true);
        fetch(`${API_BASE_URL}/api/v1/prompts/${selectedPrompt}`, {credentials: "include"})
            .then(res => {
                if (!res.ok) throw new Error(`Prompt fetch failed: ${res.status}`);
                return res.json();
            })
            .then(data => {
                setPrompt(data.prompt.content);
                setSavedPrompt(data.prompt.content);
                setIsLoading(false);
            })
            .catch(err => {
                console.error("Failed to load prompt", err);
                setIsLoading(false);
            });
    }, [API_BASE_URL, selectedPrompt]);

    const handleSave = async () => {
        setSaveStatus("saving");
        try {
            const res = await fetch(`${API_BASE_URL}/api/v1/prompts/${selectedPrompt}`, {
                method: 'POST',
                credentials: 'include',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({content: prompt})
            });
            if (!res.ok) throw new Error(`Failed to save prompt: ${res.status}`);
            setSavedPrompt(prompt);
            setIsEditing(false);
            setSaveStatus("success");
            setTimeout(() => setSaveStatus("idle"), 3000);
        } catch (err) {
            console.error(err);
            setSaveStatus("error");
            setTimeout(() => setSaveStatus("idle"), 3000);
        }
    };

    if (isLoading) return <div className="glass-panel"><p>Loading prompt...</p></div>;

    return (
        <div className="glass-panel" style={{height: "100%", display: "flex", flexDirection: "column"}}>
            <div style={{display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "24px"}}>
                <div style={{display: "flex", alignItems: "center", gap: "12px"}}>
                    <FileText size={28} color="var(--accent-purple)"/>
                    <h1 style={{margin: 0}}>Prompt Configuration</h1>
                </div>
                <div>
                    {!isEditing ? (
                        <div style={{display: "flex", alignItems: "center", gap: "12px"}}>
                            {saveStatus === "success" && <span style={{
                                color: "var(--accent-green)",
                                display: "flex",
                                alignItems: "center",
                                gap: "6px",
                                fontSize: "14px"
                            }}><CheckCircle size={16}/> Saved</span>}
                            {saveStatus === "error" && <span style={{
                                color: "var(--accent-pink)",
                                display: "flex",
                                alignItems: "center",
                                gap: "6px",
                                fontSize: "14px"
                            }}><AlertCircle size={16}/> Error</span>}
                            <button className="btn-secondary" onClick={() => setIsEditing(true)}>Edit Prompt</button>
                        </div>
                    ) : (
                        <div style={{display: "flex", gap: "8px"}}>
                            <button
                                className="btn-secondary"
                                onClick={() => {
                                    setPrompt(savedPrompt);
                                    setIsEditing(false);
                                }}
                            >
                                Cancel
                            </button>
                            <button className="btn-primary" onClick={handleSave} disabled={saveStatus === "saving"}>
                                {saveStatus === "saving" ? "Saving..." : "Save Changes"}
                            </button>
                        </div>
                    )}
                </div>
            </div>

            <div className="form-group" style={{marginBottom: "16px"}}>
                <label htmlFor="prompt-selector">Prompt Type</label>
                <select
                    id="prompt-selector"
                    value={selectedPrompt}
                    onChange={(event) => {
                        setSelectedPrompt(event.target.value as PromptName);
                        setIsEditing(false);
                        setSaveStatus("idle");
                    }}
                    style={{
                        width: "100%",
                        padding: "12px",
                        borderRadius: "10px",
                        border: "1px solid var(--border-glass)",
                        background: "rgba(0, 0, 0, 0.25)",
                        color: "var(--text-primary)"
                    }}
                    disabled={isLoading || saveStatus === "saving"}
                >
                    {PROMPT_OPTIONS.map((option) => (
                        <option key={option.name} value={option.name}>
                            {option.label}
                        </option>
                    ))}
                </select>
            </div>

            <div style={{
                flex: 1,
                overflowY: "auto",
                padding: "16px",
                background: "rgba(0, 0, 0, 0.2)",
                borderRadius: "8px",
                border: "1px solid var(--border-glass)"
            }}>
                {isEditing ? (
                    <textarea
                        value={prompt}
                        onChange={(e) => setPrompt(e.target.value)}
                        style={{
                            width: "100%",
                            height: "100%",
                            minHeight: "400px",
                            padding: "12px",
                            background: "rgba(0, 0, 0, 0.4)",
                            border: "none",
                            color: "var(--text-primary)",
                            fontFamily: "monospace",
                            resize: "none",
                            outline: "none"
                        }}
                    />
                ) : (
                    <div className="markdown-preview"
                         style={{fontFamily: "system-ui, -apple-system, sans-serif", lineHeight: 1.6}}>
                        <Markdown rehypePlugins={[rehypeHighlight]}>{prompt}</Markdown>
                    </div>
                )}
            </div>
        </div>
    );
}
