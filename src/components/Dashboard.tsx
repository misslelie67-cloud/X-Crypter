import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Footer from "./Footer";
import Controller from "./Controller";
import Status from "./Status";
import "./Dashboard.css";

interface SessionResponse {
  session_id: string;
  chat_id: string;
  expires_at: number;
}

interface DashboardProps {
  sessionData: SessionResponse;
  onLogout: () => void;
}

export default function Dashboard({ sessionData }: DashboardProps) {
  const [expiresAt, setExpiresAt] = useState<number | null>(null);
  const [appVersion, setAppVersion] = useState<string>("");

  useEffect(() => {
    // Set expiration from session data
    setExpiresAt(sessionData.expires_at * 1000); // Convert to milliseconds
    
    // Get app version
    invoke<string>("get_app_version")
      .then(setAppVersion)
      .catch(console.error);
  }, [sessionData]);

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h1 className="dashboard-title">X-Crypter</h1>
        {appVersion && (
          <span className="dashboard-version">v{appVersion}</span>
        )}
      </div>

      <div className="dashboard-content">
        <div className="dashboard-grid">
          <div className="dashboard-controller-container">
            <Controller />
          </div>
          <div className="dashboard-status-container">
            <Status />
          </div>
        </div>
      </div>

      <Footer expiresAt={expiresAt} />
    </div>
  );
}
