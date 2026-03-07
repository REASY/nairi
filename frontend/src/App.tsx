import {useEffect, useState} from "react";
import {Routes, Route, NavLink} from "react-router-dom";
import {ShieldAlert, Upload, Settings, Activity, FileText, LogOut} from "lucide-react";
import ConfigView from "./pages/ConfigView";
import UploadView from "./pages/UploadView";
import PromptsView from "./pages/PromptsView";
import {
    AuthUser,
    getCurrentUser,
    getGoogleLoginUrl,
    logoutCurrentUser,
} from "./api";

type AuthState = "loading" | "authenticated" | "unauthenticated";

function LoginScreen() {
    const handleLogin = () => {
        window.location.assign(getGoogleLoginUrl());
    };

    return (
        <div className="main-content" style={{display: "grid", placeItems: "center"}}>
            <div className="glass-panel" style={{maxWidth: "520px", width: "100%"}}>
                <h1>Authentication Required</h1>
                <p style={{marginBottom: "20px"}}>
                    Sign in with your Google account to access NAIRI endpoints.
                </p>
                <button className="btn-primary" onClick={handleLogin}>
                    Continue with Google
                </button>
            </div>
        </div>
    );
}

export default function App() {
    const [authState, setAuthState] = useState<AuthState>("loading");
    const [user, setUser] = useState<AuthUser | null>(null);

  useEffect(() => {
      let active = true;

      getCurrentUser()
          .then((currentUser) => {
              if (!active) return;
              setUser(currentUser);
              setAuthState("authenticated");
          })
          .catch(() => {
              if (!active) return;
              setUser(null);
              setAuthState("unauthenticated");
          });

      return () => {
          active = false;
      };
  }, []);

    const handleLogout = async () => {
        try {
            await logoutCurrentUser();
    } finally {
            setUser(null);
            setAuthState("unauthenticated");
    }
    };

    if (authState === "loading") {
        return (
            <div className="main-content" style={{display: "grid", placeItems: "center"}}>
                <div className="glass-panel">
                    <p>Checking session...</p>
                </div>
            </div>
        );
    }

    if (authState === "unauthenticated") {
        return <LoginScreen/>;
    }

    return (
        <div className="app-layout">
            <aside className="sidebar">
                <div className="brand">
                    <ShieldAlert size={32}/>
                    <span>NAIRI</span>
                </div>

                <nav className="nav-links">
                    <NavLink to="/" className={({isActive}) => `nav-link ${isActive ? "active" : ""}`}>
                        <Upload size={20}/>
                        Upload
                    </NavLink>
                    <NavLink
                        to="/config"
                        className={({isActive}) => `nav-link ${isActive ? "active" : ""}`}
                    >
                        <Settings size={20}/>
                        System Config
                    </NavLink>
                    <NavLink
                        to="/prompts"
                        className={({isActive}) => `nav-link ${isActive ? "active" : ""}`}
                    >
                        <FileText size={20}/>
                        Prompts
                    </NavLink>
                    <NavLink to="/runs" className={({isActive}) => `nav-link ${isActive ? "active" : ""}`}>
                        <Activity size={20}/>
                        Live Runs
                    </NavLink>
                    <NavLink
                        to="/reports"
                        className={({isActive}) => `nav-link ${isActive ? "active" : ""}`}
                    >
                        <FileText size={20}/>
                        Reports
                    </NavLink>
                </nav>
            </aside>

            <main className="main-content">
                <div
                    className="glass-panel"
                    style={{
                        marginBottom: "24px",
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                        padding: "16px 24px"
                    }}
                >
                    <div style={{display: "flex", alignItems: "center", gap: "16px"}}>
                        {user?.picture ? (
                            <img
                                src={user.picture}
                                alt="Profile"
                                style={{
                                    width: "48px",
                                    height: "48px",
                                    borderRadius: "50%",
                                    border: "2px solid var(--accent-purple)",
                                    objectFit: "cover"
                                }}
                            />
                        ) : (
                            <div style={{
                                width: "48px",
                                height: "48px",
                                borderRadius: "50%",
                                backgroundColor: "var(--accent-purple)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                fontWeight: "bold",
                                fontSize: "1.2rem",
                                color: "white"
                            }}>
                                {user?.name?.charAt(0)?.toUpperCase() || user?.email?.charAt(0)?.toUpperCase() || "U"}
                            </div>
                        )}
            <div>
                <div style={{fontWeight: "600", color: "var(--text-color)", fontSize: "1.1rem"}}>
                    {user?.name || "NAIRI Operator"}
                </div>
                <div style={{color: "var(--text-muted)", fontSize: "0.9rem", marginTop: "4px"}}>
                    {user?.email}
                </div>
            </div>
                    </div>
                    <button
                        className="btn-primary"
                        onClick={handleLogout}
                        style={{
                            background: "rgba(255, 255, 255, 0.05)",
                            border: "1px solid rgba(255,255,255,0.1)",
                            color: "var(--text-color)",
                            padding: "10px 20px"
                        }}
                    >
                        <LogOut size={18} style={{marginRight: "8px", verticalAlign: "bottom"}}/>
                        Sign Out
                    </button>
                </div>

                <Routes>
                    <Route path="/" element={<UploadView/>}/>
                    <Route path="/config" element={<ConfigView/>}/>
                    <Route path="/prompts" element={<PromptsView/>}/>
                    <Route
                        path="/runs"
                        element={
                            <div className="glass-panel">
                                <h1>Live Runs</h1>
                                <p>Not implemented yet.</p>
                            </div>
                        }
                    />
                    <Route
                        path="/reports"
                        element={
                            <div className="glass-panel">
                                <h1>Reports</h1>
                                <p>Not implemented yet.</p>
                            </div>
                        }
                    />
                </Routes>
            </main>
        </div>
  );
}
