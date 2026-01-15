import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Download, CheckCircle, XCircle, AlertCircle } from "lucide-react";
import toast from "react-hot-toast";
import "./UpdateStatus.css";

interface UpdateStatus {
  status: string;
  update_available: boolean;
  version: string | null;
  changelog: string | null;
  error: string | null;
  witness: string | null;
}

export default function UpdateStatus() {
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const [currentVersion, setCurrentVersion] = useState<string>("");
  const [lastInstalledVersion, setLastInstalledVersion] = useState<string | null>(null);
  const [isChecking, setIsChecking] = useState(false);
  const [witness, setWitness] = useState<string | null>(null);
  const [isInstalling, setIsInstalling] = useState(false);
  const [checkInterval, setCheckInterval] = useState<number>(4 * 60 * 60 * 1000); // Default 4 hours

  const checkForUpdates = useCallback(async (isManual = false) => {
    setIsChecking(true);
    try {
      const result = await invoke<UpdateStatus>("check_for_updates");
      setUpdateStatus(result);
      
      if (result.status === "waiting_approval") {
        // Extract witness from response
        setWitness(result.witness || null);
        // Always show notification for new updates, even on automatic check
        toast.success("Update available! Check Telegram for approval.", { duration: 5000 });
      } else if (result.status === "up_to_date") {
        // Only show "up to date" message for manual checks
        if (isManual) {
          toast.success("App is up to date!");
        }
      } else if (result.error) {
        // Only show errors for manual checks (to avoid spam)
        if (isManual) {
          toast.error(result.error);
        }
      }
    } catch (error) {
      // Only show errors for manual checks
      if (isManual) {
        toast.error(`Failed to check for updates: ${error}`);
      }
    } finally {
      setIsChecking(false);
    }
  }, []);

  useEffect(() => {
    // Get current version on mount
    invoke<string>("get_app_version")
      .then(setCurrentVersion)
      .catch(console.error);
    
    // Get update check interval from backend
    invoke<number>("get_update_check_interval")
      .then((seconds) => {
        setCheckInterval(seconds * 1000); // Convert to milliseconds
      })
      .catch(() => {
        // Use default if failed
        console.warn("Failed to get update check interval, using default 4 hours");
      });
    
    // Get last installed version
    invoke<{ version: string; file_hash: string; file_url: string; timestamp: number; witness: string; changelog?: string; size: number } | null>("get_stored_version")
      .then((stored) => {
        if (stored) {
          setLastInstalledVersion(stored.version);
        }
      })
      .catch(console.error);
    
    // Check for updates on app load
    checkForUpdates();
  }, [checkForUpdates]);

  // Automatic update check using interval from config
  useEffect(() => {
    const interval = setInterval(() => {
      checkForUpdates();
    }, checkInterval);

    return () => clearInterval(interval);
  }, [checkForUpdates, checkInterval]);

  const checkApproval = async () => {
    if (!witness) return;
    
    try {
      const approved = await invoke<boolean | null>("check_update_approval", { witness });
      
      if (approved === true) {
        toast.success("Update approved! Installing...");
        installUpdate();
      } else if (approved === false) {
        toast.error("Update was rejected.");
        setUpdateStatus(null);
        setWitness(null);
      }
      // null means still waiting
    } catch (error) {
      console.error("Failed to check approval:", error);
    }
  };

  const installUpdate = async () => {
    if (!witness) return;
    
    setIsInstalling(true);
    try {
      const result = await invoke<string>("install_update", { witness });
      toast.success(result, { duration: 10000 });
      
      // Wait a moment, then restart
      setTimeout(async () => {
        try {
          await invoke("restart_application");
        } catch (error) {
          toast.error(`Update installed but restart failed. Please restart manually.`);
        }
      }, 2000);
      
      setUpdateStatus(null);
      setWitness(null);
    } catch (error) {
      toast.error(`Failed to install update: ${error}`);
      setIsInstalling(false);
    }
  };

  // Poll for approval if waiting
  useEffect(() => {
    if (updateStatus?.status === "waiting_approval" && witness) {
      const interval = setInterval(() => {
        checkApproval();
      }, 5000); // Check every 5 seconds

      return () => clearInterval(interval);
    }
  }, [updateStatus?.status, witness]);

  return (
    <div className="update-status-footer">
      {updateStatus?.status === "waiting_approval" && (
        <div className="update-pending-footer">
          <AlertCircle size={14} />
          <span>Update {updateStatus.version} pending approval</span>
        </div>
      )}

      {updateStatus?.status === "error" && updateStatus.error && (
        <div className="update-error-footer">
          <XCircle size={14} />
          <span>Update error</span>
        </div>
      )}

      {updateStatus?.status === "up_to_date" && (
        <div className="update-uptodate-footer">
          <CheckCircle size={14} />
          <span>v{currentVersion}{lastInstalledVersion && lastInstalledVersion !== currentVersion ? ` (last: v${lastInstalledVersion})` : ''}</span>
        </div>
      )}

      {isInstalling && (
        <div className="update-installing-footer">
          <Download size={14} className="spinning" />
          <span>Installing...</span>
        </div>
      )}

      {!updateStatus && !isInstalling && (
        <button
          className="check-update-button-footer"
          onClick={() => checkForUpdates(true)}
          disabled={isChecking}
          title={`Current version: ${currentVersion}`}
        >
          {isChecking ? (
            <>
              <RefreshCw size={18} className="spinning" style={{ marginRight: '0.5em', verticalAlign: 'middle' }} />
              Checking...
            </>
          ) : (
            <>
              <RefreshCw size={18} style={{ marginRight: '0.5em', verticalAlign: 'middle' }} />
              Check Updates
            </>
          )}
        </button>
      )}
    </div>
  );
}
