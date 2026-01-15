import { useState, useRef } from "react";
import { ShieldCheck, XCircle, Loader2, Upload, AlertCircle, CheckCircle2, FileCheck, X } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import "./ScannerWidget.css";

interface ScanResult {
  engine: string;
  status: "Clean" | "Infected" | "Error";
  threats: string[];
  error?: string;
}

interface ScanResponse {
  clamav?: ScanResult;
  windows_defender?: ScanResult;
  overall_status: "Clean" | "Infected" | "Error";
}

interface ScannerWidgetProps {
  onClose: () => void;
}

export default function ScannerWidget({ onClose }: ScannerWidgetProps) {
  const [file, setFile] = useState<File | null>(null);
  const [filePath, setFilePath] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [scanResults, setScanResults] = useState<ScanResponse | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const processFile = async (selectedFile: File) => {
    setFile(selectedFile);
    setScanResults(null);
    setIsUploading(true);

    try {
      // Read file as ArrayBuffer
      const arrayBuffer = await selectedFile.arrayBuffer();
      const fileData = Array.from(new Uint8Array(arrayBuffer));

      // Save to temp location via Tauri
      const tempPath = await invoke<string>("save_uploaded_file", {
        fileData: fileData,
        fileName: selectedFile.name,
      });

      setFilePath(tempPath);
    } catch (error: any) {
      console.error("Failed to save file:", error);
      setFile(null);
      setFilePath(null);
    } finally {
      setIsUploading(false);
    }
  };

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (selectedFile) {
      await processFile(selectedFile);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const droppedFile = e.dataTransfer.files?.[0];
    if (droppedFile) {
      await processFile(droppedFile);
    }
  };

  const handleScan = async () => {
    if (!filePath) {
      return;
    }

    setScanning(true);
    setScanResults(null);

    try {
      const results = await invoke<ScanResponse>("scan_file", {
        filePath: filePath,
      });

      setScanResults(results);
    } catch (error: any) {
      console.error("Scan failed:", error);
      setScanResults({
        overall_status: "Error",
        clamav: {
          engine: "Scanner",
          status: "Error",
          threats: [],
          error: error?.message || error?.toString() || "Unknown error occurred",
        },
      });
    } finally {
      setScanning(false);
    }
  };

  const handleReset = () => {
    setFile(null);
    setFilePath(null);
    setScanResults(null);
    setScanning(false);
    setIsDragging(false);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  const handleBrowseClick = () => {
    fileInputRef.current?.click();
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
  };

  const getStatusIcon = (status: "Clean" | "Infected" | "Error") => {
    switch (status) {
      case "Clean":
        return <CheckCircle2 size={24} color="#00ff80" />;
      case "Infected":
        return <XCircle size={24} color="#ff0055" />;
      case "Error":
        return <AlertCircle size={24} color="#ffaa00" />;
    }
  };

  const getStatusText = (status: "Clean" | "Infected" | "Error") => {
    switch (status) {
      case "Clean":
        return "Clean";
      case "Infected":
        return "Threats Found";
      case "Error":
        return "Error";
    }
  };

  return (
    <div className="scanner-widget-overlay">
      <div className="scanner-widget-modal">
        <div className="scanner-widget-header">
          <div className="scanner-widget-title">
            <ShieldCheck size={22} style={{ marginRight: '0.6em', verticalAlign: 'middle' }} />
            Stub Scanner
          </div>
          <button className="scanner-widget-close" onClick={onClose} title="Close">
            <XCircle size={22} />
          </button>
        </div>

        <div className="scanner-widget-content">
          {/* Upload Section - Drag and Drop */}
          {!file && !scanning && (
            <div
              className={`scanner-drop-zone ${isDragging ? 'scanner-drop-zone-active' : ''}`}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
              onClick={handleBrowseClick}
            >
              <input
                ref={fileInputRef}
                id="scanner-file-input"
                type="file"
                accept=".exe,.bin,.dll,.dat,.scr,.com,.bat,.cmd,.msi"
                className="scanner-file-input"
                onChange={handleFileChange}
                aria-label="Select stub file to scan"
              />
              
              <div className="scanner-drop-content">
                {isUploading ? (
                  <>
                    <Loader2 size={48} className="scanner-spinner" />
                    <div className="scanner-drop-title">Uploading file...</div>
                  </>
                ) : (
                  <>
                    <div className="scanner-drop-icon-wrapper">
                      <Upload size={48} className="scanner-drop-icon" />
                    </div>
                    <div className="scanner-drop-title">Drop your stub file here</div>
                    <div className="scanner-drop-subtitle">or click to browse</div>
                    <div className="scanner-drop-formats">
                      Supports: .exe, .dll, .bin, .dat, .scr, .com, .bat, .cmd, .msi
                    </div>
                  </>
                )}
              </div>
            </div>
          )}

          {/* File Selected, Ready to Scan */}
          {file && !scanning && !scanResults && (
            <div className="scanner-file-preview">
              <div className="scanner-file-card">
                <div className="scanner-file-icon-wrapper">
                  <FileCheck size={32} />
                </div>
                <div className="scanner-file-details">
                  <div className="scanner-file-name">{file.name}</div>
                  <div className="scanner-file-meta">
                    <span className="scanner-file-size">{formatFileSize(file.size)}</span>
                    <span className="scanner-file-separator">•</span>
                    <span className="scanner-file-type">{file.type || 'Unknown type'}</span>
                  </div>
                </div>
                <button 
                  className="scanner-file-remove" 
                  onClick={handleReset}
                  title="Remove file"
                >
                  <X size={18} />
                </button>
              </div>
              
              <div className="scanner-action-buttons">
                <button className="scanner-scan-button" onClick={handleScan}>
                  <ShieldCheck size={20} />
                  <span>Start Scan</span>
                </button>
                <button className="scanner-change-button" onClick={handleReset}>
                  Change File
                </button>
              </div>
            </div>
          )}

          {/* Scanning */}
          {scanning && (
            <div className="scanner-scanning-state">
              <div className="scanner-scanning-icon">
                <Loader2 size={56} className="scanner-spinner" />
              </div>
              <div className="scanner-scanning-title">Scanning file...</div>
              <div className="scanner-scanning-subtitle">
                Analyzing with ClamAV and Windows Defender
              </div>
              <div className="scanner-scanning-progress">
                <div className="scanner-progress-bar">
                  <div className="scanner-progress-fill"></div>
                </div>
              </div>
            </div>
          )}

          {/* Scan Results */}
          {scanResults && !scanning && (
            <div className="scanner-results">
              <div className="scanner-overall-status">
                <div className="scanner-status-icon-large">
                  {scanResults.overall_status === "Clean" && (
                    <CheckCircle2 size={64} color="#00ff80" />
                  )}
                  {scanResults.overall_status === "Infected" && (
                    <XCircle size={64} color="#ff0055" />
                  )}
                  {scanResults.overall_status === "Error" && (
                    <AlertCircle size={64} color="#ffaa00" />
                  )}
                </div>
                <div className="scanner-overall-status-text">
                  {scanResults.overall_status === "Clean" && <span>No threats detected!</span>}
                  {scanResults.overall_status === "Infected" && <span>Threats found!</span>}
                  {scanResults.overall_status === "Error" && <span>Scan completed with errors</span>}
                </div>
              </div>

              <div className="scanner-engine-results">
                {/* ClamAV Results */}
                {scanResults.clamav && (
                  <div className="scanner-engine-card">
                    <div className="scanner-engine-header">
                      {getStatusIcon(scanResults.clamav.status)}
                      <span className="scanner-engine-name">ClamAV</span>
                      <span className={`scanner-engine-badge scanner-status-${scanResults.clamav.status.toLowerCase()}`}>
                        {getStatusText(scanResults.clamav.status)}
                      </span>
                    </div>
                    {scanResults.clamav.threats.length > 0 && (
                      <div className="scanner-threats-list">
                        <div className="scanner-threats-label">Threats detected:</div>
                        {scanResults.clamav.threats.map((threat, idx) => (
                          <div key={idx} className="scanner-threat-badge">{threat}</div>
                        ))}
                      </div>
                    )}
                    {scanResults.clamav.error && (
                      <div className="scanner-error-message">{scanResults.clamav.error}</div>
                    )}
                  </div>
                )}

                {/* Windows Defender Results */}
                {scanResults.windows_defender && (
                  <div className="scanner-engine-card">
                    <div className="scanner-engine-header">
                      {getStatusIcon(scanResults.windows_defender.status)}
                      <span className="scanner-engine-name">Windows Defender</span>
                      <span className={`scanner-engine-badge scanner-status-${scanResults.windows_defender.status.toLowerCase()}`}>
                        {getStatusText(scanResults.windows_defender.status)}
                      </span>
                    </div>
                    {scanResults.windows_defender.threats.length > 0 && (
                      <div className="scanner-threats-list">
                        <div className="scanner-threats-label">Threats detected:</div>
                        {scanResults.windows_defender.threats.map((threat, idx) => (
                          <div key={idx} className="scanner-threat-badge">{threat}</div>
                        ))}
                      </div>
                    )}
                    {scanResults.windows_defender.error && (
                      <div className="scanner-error-message">{scanResults.windows_defender.error}</div>
                    )}
                  </div>
                )}
              </div>

              <button className="scanner-new-scan-button" onClick={handleReset}>
                <Upload size={18} />
                <span>Scan Another File</span>
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
