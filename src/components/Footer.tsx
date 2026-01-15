import { useState, useEffect } from "react";
import ScannerWidget from "./ScannerWidget";
import UpdateStatus from "./UpdateStatus";
import { Clock, Calendar, Key, ShieldCheck } from "lucide-react";
import "./Footer.css";

interface FooterProps {
  expiresAt: number | null;
}

export default function Footer({ expiresAt }: FooterProps) {
  const [currentTime, setCurrentTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: true,
    });
  };

  const formatDate = (date: Date) => {
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    const year = date.getFullYear();
    return `${month}/${day}/${year}`;
  };

  const formatExpiration = (timestamp: number | null) => {
    if (!timestamp) return "Loading...";
    
    const date = new Date(timestamp);
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    const year = date.getFullYear();
    return `${month}/${day}/${year}`;
  };

  const [showScanner, setShowScanner] = useState(false);

  const handleScanClick = () => {
    setShowScanner(true);
  };

  const handleCloseScanner = () => {
    setShowScanner(false);
  };

  return (
    <>
      <footer className="dashboard-footer">
        <div className="footer-info-group">
          <div className="footer-item">
            <Clock size={16} />
            <span>{formatTime(currentTime)}</span>
          </div>
          <div className="footer-item">
            <Calendar size={16} />
            <span>{formatDate(currentTime)}</span>
          </div>
          <div className="footer-item">
            <Key size={16} />
            <span>Expires: {formatExpiration(expiresAt)}</span>
          </div>
        </div>
        <div className="footer-actions">
          <UpdateStatus />
          <button className="scan-stub-btn" onClick={handleScanClick}>
            <ShieldCheck size={18} style={{ marginRight: '0.5em', verticalAlign: 'middle' }} />
            Scan Stub
          </button>
        </div>
      </footer>
      {showScanner && (
        <ScannerWidget
          onClose={handleCloseScanner}
        />
      )}
    </>
  );
}
