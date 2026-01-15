import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import Lottie from "lottie-react";
import loginAnimation from "./lottie/login.json";
import "./App.css";

interface AppProps {
  onAuthCheck: () => void;
  sessionId?: string | null;
  onLogout?: () => void;
}

function App({ onAuthCheck, sessionId, onLogout }: AppProps) {
  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    // Validate session every 6 hours (21600000 ms)
    async function validate() {
      if (!sessionId) return;
      try {
        await invoke("validate_session", { sessionId });
      } catch (e) {
        // Session expired or invalid
        if (onLogout) onLogout();
      }
    }
    validate(); // Initial check
    intervalRef.current = window.setInterval(validate, 6 * 60 * 60 * 1000); // 6 hours
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [sessionId, onLogout]);

  useEffect(() => {
    // For now, just redirect after 5 seconds
    const timer = setTimeout(() => {
      onAuthCheck();
    }, 5000);
    return () => clearTimeout(timer);
  }, [onAuthCheck]);

  return (
    <div className="loading-screen">
      <Lottie 
        animationData={loginAnimation} 
        loop={true}
        style={{ width: 550, height: 550 }}
      />
    </div>
  );
}

export default App;
