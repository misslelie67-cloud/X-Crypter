import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import AuthScreen from "./components/AuthScreen";
import Dashboard from "./components/Dashboard";
import { EncryptionProvider } from "./contexts/EncryptionContext";
import "./styles/index.css";

type Screen = "loading" | "auth" | "dashboard";

interface SessionResponse {
  session_id: string;
  chat_id: string;
  expires_at: number;
}

function RootApp() {
  const [currentScreen, setCurrentScreen] = useState<Screen>("loading");
  const [sessionData, setSessionData] = useState<SessionResponse | null>(null);

  useEffect(() => {
    checkSession();
  }, []);

  const checkSession = async () => {
    const storedSessionId = localStorage.getItem("session_id");
    
    if (storedSessionId) {
      try {
        const session = await invoke<SessionResponse>("validate_session", {
          sessionId: storedSessionId,
        });
        
        setSessionData(session);
        setCurrentScreen("dashboard");
      } catch (err) {
        console.error("Session validation failed:", err);
        localStorage.removeItem("session_id");
        localStorage.removeItem("chat_id");
        setCurrentScreen("auth");
      }
    } else {
      setCurrentScreen("auth");
    }
  };

  const handleAuthCheck = () => {
    setCurrentScreen("auth");
  };

  const handleLoginSuccess = (session: SessionResponse) => {
    setSessionData(session);
    setCurrentScreen("dashboard");
  };

  const handleLogout = () => {
    localStorage.removeItem("session_id");
    localStorage.removeItem("chat_id");
    setSessionData(null);
    setCurrentScreen("auth");
  };

  return (
    <EncryptionProvider>
      {currentScreen === "loading" && <App onAuthCheck={handleAuthCheck} />}
      {currentScreen === "auth" && <AuthScreen onLoginSuccess={handleLoginSuccess} />}
      {currentScreen === "dashboard" && sessionData && (
        <Dashboard sessionData={sessionData} onLogout={handleLogout} />
      )}
    </EncryptionProvider>
  );
}

export default RootApp;
