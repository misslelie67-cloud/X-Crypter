import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { XCircle, AlertCircle, Clock, Loader2 } from "lucide-react";
import "./AuthScreen.css";

interface QRResponse {
  token: string;
  qr_code: string;
  expires_in: number;
}

interface AuthStatusResponse {
  status: "created" | "pending" | "approved" | "denied" | "expired";
  expired: boolean;
}

interface SessionResponse {
  session_id: string;
  chat_id: string;
  expires_at: number;
}

interface AuthScreenProps {
  onLoginSuccess: (session: SessionResponse) => void;
}

export default function AuthScreen({ onLoginSuccess }: AuthScreenProps) {
  const [qrCode, setQrCode] = useState<string | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [countdown, setCountdown] = useState<number>(0);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<"idle" | "waiting" | "pending" | "approved" | "denied" | "timeout" | "verifying">("idle");
  
  const pollingIntervalRef = useRef<number | null>(null);
  const qrRefreshIntervalRef = useRef<number | null>(null);
  const countdownIntervalRef = useRef<number | null>(null);
  const verifyingRef = useRef<boolean>(false);

  // Generate QR code on mount
  useEffect(() => {
    generateQR();
    
    return () => {
      if (pollingIntervalRef.current) clearInterval(pollingIntervalRef.current);
      if (qrRefreshIntervalRef.current) clearInterval(qrRefreshIntervalRef.current);
      if (countdownIntervalRef.current) clearInterval(countdownIntervalRef.current);
    };
  }, []);

  const generateQR = async () => {
    try {
      const response = await invoke<QRResponse>("generate_qr_code");
      setQrCode(response.qr_code);
      setToken(response.token);
      setCountdown(response.expires_in);
      setStatus("waiting");
      setError(null);
      startCountdown(response.expires_in);
    } catch (err) {
      setError(`Failed to generate QR code: ${err}`);
    }
  };

  const startCountdown = (seconds: number) => {
    if (countdownIntervalRef.current) clearInterval(countdownIntervalRef.current);
    
    let remaining = seconds;
    countdownIntervalRef.current = setInterval(() => {
      remaining -= 1;
      setCountdown(remaining);
      
      if (remaining <= 0) {
        if (countdownIntervalRef.current) clearInterval(countdownIntervalRef.current);
        if (status === "pending") {
          setStatus("timeout");
        } else if (status === "waiting") {
          // Auto-regenerate QR if it expires while waiting
          generateQR();
        }
      }
    }, 1000);
  };

  // Auto-refresh QR code every 60 seconds
  useEffect(() => {
    if (status === "waiting" && qrCode && !verifyingRef.current) {
      qrRefreshIntervalRef.current = setInterval(() => {
        generateQR();
      }, 60000);
    }

    return () => {
      if (qrRefreshIntervalRef.current) clearInterval(qrRefreshIntervalRef.current);
    };
  }, [status, qrCode]);

  // Verify approved session
  const verifySession = async (sessionToken: string) => {
    if (verifyingRef.current) return;
    
    verifyingRef.current = true;
    setStatus("verifying");
    
    // Stop all polling
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current);
      pollingIntervalRef.current = null;
    }
    if (qrRefreshIntervalRef.current) {
      clearInterval(qrRefreshIntervalRef.current);
      qrRefreshIntervalRef.current = null;
    }
    if (countdownIntervalRef.current) {
      clearInterval(countdownIntervalRef.current);
      countdownIntervalRef.current = null;
    }

    try {
      const session = await invoke<SessionResponse>("exchange_token", { token: sessionToken });
      
      // Store session
      localStorage.setItem("session_id", session.session_id);
      localStorage.setItem("chat_id", session.chat_id);
      
      setStatus("approved");
      
      // Navigate to dashboard via callback
      setTimeout(() => {
        onLoginSuccess(session);
      }, 1500);
    } catch (err) {
      setError(`Verification failed: ${err}`);
      verifyingRef.current = false;
      setStatus("idle");
    }
  };

  // Poll for auth status
  useEffect(() => {
    if (!token || status === "approved" || status === "denied" || status === "timeout" || status === "verifying") {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
        pollingIntervalRef.current = null;
      }
      return;
    }

    pollingIntervalRef.current = setInterval(async () => {
      try {
        const response = await invoke<AuthStatusResponse>("check_auth_status", { token });
        
        if (response.expired) {
          setStatus("timeout");
          if (pollingIntervalRef.current) clearInterval(pollingIntervalRef.current);
          return;
        }

        if (response.status === "approved") {
          // Stop polling immediately
          if (pollingIntervalRef.current) {
            clearInterval(pollingIntervalRef.current);
            pollingIntervalRef.current = null;
          }
          
          await verifySession(token);
        } else if (response.status === "denied") {
          setStatus("denied");
          if (pollingIntervalRef.current) {
            clearInterval(pollingIntervalRef.current);
            pollingIntervalRef.current = null;
          }
        } else if (response.status === "pending") {
          // User has scanned and bot sent buttons - show overlay
          setStatus("pending");
        }
      } catch (err) {
        console.error("Error checking auth status:", err);
      }
    }, 2000);

    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
        pollingIntervalRef.current = null;
      }
    };
  }, [token, status]);

  const handleRetry = () => {
    setStatus("idle");
    setError(null);
    setToken(null);
    setQrCode(null);
    verifyingRef.current = false;
    generateQR();
  };

  return (
    <div className="auth-screen">
      <div className="auth-container">
        <h1 className="auth-title">X-Crypter</h1>
        {status === "denied" && (
          <div className="status-container error">
            <div className="status-icon"><XCircle size={64} /></div>
            <h3 className="status-title">Login Declined</h3>
            <p className="status-message">
              The login request was declined. Please scan a new QR code to try again.
            </p>
            <button onClick={handleRetry} className="retry-button">
              Scan New QR Code
            </button>
          </div>
        )}

        {(status === "approved" || status === "verifying") && (
          <div className="status-container success">
            <div className="status-icon loading"><Loader2 size={64} /></div>
            <h3 className="status-title">
              {status === "verifying" ? "Verifying..." : "Login Approved"}
            </h3>
            <p className="status-message">
              {status === "verifying" ? "Completing authentication..." : "Redirecting to dashboard..."}
            </p>
          </div>
        )}

        {error && status !== "denied" && (
          <div className="status-container error">
            <div className="status-icon"><AlertCircle size={64} /></div>
            <h3 className="status-title">Error</h3>
            <p className="status-message">{error}</p>
            <button onClick={handleRetry} className="retry-button">
              Try Again
            </button>
          </div>
        )}

        {(status === "waiting" || status === "pending" || status === "timeout") && qrCode && (
          <div className="qr-wrapper">
            <div className="qr-display">
              <div 
                className="qr-code"
                dangerouslySetInnerHTML={{ __html: qrCode }}
              />
              
              {status === "pending" && (
                <div className="qr-overlay">
                  <div className="overlay-content">
                    <div className="pulse-icon"><Clock size={64} /></div>
                    <h3 className="overlay-title">Waiting Approval</h3>
                    <p className="overlay-message">
                      Check your Telegram to approve or decline
                    </p>
                  </div>
                </div>
              )}

              {status === "timeout" && (
                <div className="qr-overlay">
                  <div className="overlay-content">
                    <div className="overlay-icon"><Clock size={64} /></div>
                    <h3 className="overlay-title">Approval Timeout</h3>
                    <p className="overlay-message">
                      Request expired. Please scan a new QR code.
                    </p>
                    <button onClick={handleRetry} className="retry-button">
                      Start Over
                    </button>
                  </div>
                </div>
              )}
            </div>
            
            {status === "waiting" && (
              <div className="qr-instructions">
                <div className="countdown">
                  Expires in {countdown} seconds
                </div>
              </div>
            )}
          </div>
        )}

        {status === "idle" && !qrCode && !error && (
          <div className="qr-loading">
            <div className="loading-spinner"><Loader2 size={64} /></div>
            <p className="loading-text">Generating QR Code...</p>
          </div>
        )}
      </div>
    </div>
  );
}
