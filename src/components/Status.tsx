import { useEffect, useRef } from "react";
import { Activity, CheckCircle2, XCircle, AlertCircle, Info, Loader2 } from "lucide-react";
import { useEncryption } from "../contexts/EncryptionContext";
import "./Status.css";

export default function Status() {
  const { logs, isEncrypting } = useEncryption();
  const logsEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new logs are added
  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const getIcon = (type: string) => {
    switch (type) {
      case 'success':
        return <CheckCircle2 size={14} className="log-icon log-icon-success" />;
      case 'error':
        return <XCircle size={14} className="log-icon log-icon-error" />;
      case 'warning':
        return <AlertCircle size={14} className="log-icon log-icon-warning" />;
      case 'progress':
        return <Loader2 size={14} className="log-icon log-icon-progress spinning" />;
      default:
        return <Info size={14} className="log-icon log-icon-info" />;
    }
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false
    });
  };

  return (
    <div className="status">
      <div className="status-header">
        <h2 className="status-title">
          <Activity size={18} />
          Encryption Log
          {isEncrypting && <Loader2 size={14} className="status-spinner spinning" />}
        </h2>
      </div>

      <div className="status-content">
        {logs.length === 0 ? (
          <div className="status-placeholder">
            <Info size={24} />
            <p>No encryption activity yet</p>
            <span>Start encrypting a file to see live logs here</span>
          </div>
        ) : (
          <div className="status-history">
            {logs.map((log) => (
              <div key={log.id} className={`status-log status-log-${log.type}`}>
                <div className="log-header">
                  <div className="log-icon-wrapper">
                    {getIcon(log.type)}
                  </div>
                  <span className="log-time">{formatTime(log.timestamp)}</span>
                </div>
                <div className="log-message">{log.message}</div>
                {log.details && (
                  <div className="log-details">{log.details}</div>
                )}
              </div>
            ))}
            <div ref={logsEndRef} />
          </div>
        )}
      </div>
    </div>
  );
}
