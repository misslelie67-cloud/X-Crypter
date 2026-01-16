// Scanner commands for ClamAV and Windows Defender

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanResult {
    pub engine: String,
    pub status: ScanStatus,
    pub threats: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ScanStatus {
    Clean,
    Infected,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResponse {
    pub clamav: Option<ScanResult>,
    pub windows_defender: Option<ScanResult>,
    pub overall_status: ScanStatus,
}

/// Scan file with ClamAV
#[tauri::command]
pub async fn scan_with_clamav(file_path: String) -> Result<ScanResult, String> {
    // Check if ClamAV is installed
    let clamav_check = Command::new("clamscan").arg("--version").output();

    if clamav_check.is_err() {
        // Try alternative command names
        let alt_check = Command::new("clamdscan").arg("--version").output();

        if alt_check.is_err() {
            return Ok(ScanResult {
                engine: "ClamAV".to_string(),
                status: ScanStatus::Error,
                threats: vec![],
                error: Some(
                    "ClamAV is not installed. Please install ClamAV to use this scanner."
                        .to_string(),
                ),
            });
        }
    }

    let file_path_buf = PathBuf::from(&file_path);
    if !file_path_buf.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Run ClamAV scan
    let output = Command::new("clamscan")
        .arg("--no-summary")
        .arg("--infected")
        .arg(&file_path)
        .output()
        .or_else(|_| {
            // Try alternative command
            Command::new("clamdscan")
                .arg("--no-summary")
                .arg("--infected")
                .arg(&file_path)
                .output()
        })
        .map_err(|e| format!("Failed to execute ClamAV: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // ClamAV returns exit code 1 if threats found, 0 if clean
    let exit_code = output.status.code().unwrap_or(1);

    if exit_code == 0 {
        // Clean
        Ok(ScanResult {
            engine: "ClamAV".to_string(),
            status: ScanStatus::Clean,
            threats: vec![],
            error: None,
        })
    } else if exit_code == 1 {
        // Infected - parse threats from output
        let mut threats = vec![];

        // Parse ClamAV output format: "file_path: ThreatName FOUND"
        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("FOUND") || line.contains(": ") {
                if let Some(threat) = line.split(": ").nth(1) {
                    let threat_name = threat.replace(" FOUND", "").trim().to_string();
                    if !threat_name.is_empty() && !threats.contains(&threat_name) {
                        threats.push(threat_name);
                    }
                }
            }
        }

        Ok(ScanResult {
            engine: "ClamAV".to_string(),
            status: ScanStatus::Infected,
            threats,
            error: None,
        })
    } else {
        // Error
        let error_msg = if !stderr.is_empty() {
            stderr.trim().to_string()
        } else if !stdout.is_empty() {
            stdout.trim().to_string()
        } else {
            format!("ClamAV scan failed with exit code {}", exit_code)
        };

        Ok(ScanResult {
            engine: "ClamAV".to_string(),
            status: ScanStatus::Error,
            threats: vec![],
            error: Some(error_msg),
        })
    }
}

/// Scan file with Windows Defender
#[tauri::command]
pub async fn scan_with_windows_defender(file_path: String) -> Result<ScanResult, String> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = file_path;
        Ok(ScanResult {
            engine: "Windows Defender".to_string(),
            status: ScanStatus::Error,
            threats: vec![],
            error: Some("Windows Defender is only available on Windows".to_string()),
        })
    }

    #[cfg(target_os = "windows")]
    {
        let file_path_buf = PathBuf::from(&file_path);
        if !file_path_buf.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        // Use PowerShell to run Windows Defender scan
        // MpCmdRun.exe is the command-line interface for Windows Defender
        let ps_script = format!(
            r#"
            $result = Start-MpScan -ScanType QuickScan -ScanPath "{}" -ErrorAction SilentlyContinue
            if ($result -eq $null) {{
                Write-Output "ERROR: Failed to start scan"
                exit 1
            }}
            Start-Sleep -Seconds 2
            $threats = Get-MpThreatDetection | Where-Object {{ $_.Resources -like "*{}*" }}
            if ($threats) {{
                foreach ($threat in $threats) {{
                    Write-Output "THREAT: $($threat.ThreatName)"
                }}
                exit 1
            }} else {{
                Write-Output "CLEAN"
                exit 0
            }}
            "#,
            file_path.replace("\\", "\\\\"),
            file_path.replace("\\", "\\\\")
        );

        // Try using MpCmdRun.exe directly first (more reliable)
        let mpcmdrun_path = r"C:\Program Files\Windows Defender\MpCmdRun.exe";
        if PathBuf::from(mpcmdrun_path).exists() {
            let output = Command::new(mpcmdrun_path)
                .args(&["-Scan", "-ScanType", "1", "-File", &file_path])
                .output()
                .map_err(|e| format!("Failed to execute Windows Defender: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(1);

            if exit_code == 0 {
                // Check if threats were found in output
                let output_text = format!("{} {}", stdout, stderr);
                if output_text.contains("threat") || output_text.contains("Threat") {
                    // Parse threats
                    let mut threats = vec![];
                    for line in output_text.lines() {
                        if line.contains("threat") || line.contains("Threat") {
                            // Try to extract threat name
                            if let Some(start) = line.find(":") {
                                let threat = line[start + 1..].trim().to_string();
                                if !threat.is_empty() && !threats.contains(&threat) {
                                    threats.push(threat);
                                }
                            }
                        }
                    }

                    return Ok(ScanResult {
                        engine: "Windows Defender".to_string(),
                        status: ScanStatus::Infected,
                        threats,
                        error: None,
                    });
                }

                return Ok(ScanResult {
                    engine: "Windows Defender".to_string(),
                    status: ScanStatus::Clean,
                    threats: vec![],
                    error: None,
                });
            } else {
                return Ok(ScanResult {
                    engine: "Windows Defender".to_string(),
                    status: ScanStatus::Error,
                    threats: vec![],
                    error: Some(format!("Windows Defender scan failed: {}", stderr)),
                });
            }
        }

        // Fallback to PowerShell
        let output = Command::new("powershell")
            .args(&["-Command", &ps_script])
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output.status.code().unwrap_or(1);

        if exit_code == 0 {
            if stdout.contains("CLEAN") {
                Ok(ScanResult {
                    engine: "Windows Defender".to_string(),
                    status: ScanStatus::Clean,
                    threats: vec![],
                    error: None,
                })
            } else {
                Ok(ScanResult {
                    engine: "Windows Defender".to_string(),
                    status: ScanStatus::Error,
                    threats: vec![],
                    error: Some("Unexpected output from Windows Defender".to_string()),
                })
            }
        } else {
            // Check if threats were found
            let mut threats = vec![];
            for line in stdout.lines() {
                if line.starts_with("THREAT:") {
                    let threat = line.replace("THREAT:", "").trim().to_string();
                    if !threat.is_empty() {
                        threats.push(threat);
                    }
                }
            }

            if !threats.is_empty() {
                Ok(ScanResult {
                    engine: "Windows Defender".to_string(),
                    status: ScanStatus::Infected,
                    threats,
                    error: None,
                })
            } else {
                let error_msg = if !stderr.is_empty() {
                    stderr.trim().to_string()
                } else {
                    format!("Windows Defender scan failed with exit code {}", exit_code)
                };

                Ok(ScanResult {
                    engine: "Windows Defender".to_string(),
                    status: ScanStatus::Error,
                    threats: vec![],
                    error: Some(error_msg),
                })
            }
        }
    }
}

/// Scan file with all available scanners
#[tauri::command]
pub async fn scan_file(file_path: String) -> Result<ScanResponse, String> {
    let file_path_buf = PathBuf::from(&file_path);
    if !file_path_buf.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let mut results = ScanResponse {
        clamav: None,
        windows_defender: None,
        overall_status: ScanStatus::Clean,
    };

    // Scan with ClamAV (available on all platforms)
    match scan_with_clamav(file_path.clone()).await {
        Ok(result) => {
            results.clamav = Some(result.clone());
            if matches!(result.status, ScanStatus::Infected) {
                results.overall_status = ScanStatus::Infected;
            } else if matches!(result.status, ScanStatus::Error)
                && matches!(results.overall_status, ScanStatus::Clean)
            {
                results.overall_status = ScanStatus::Error;
            }
        }
        Err(e) => {
            results.clamav = Some(ScanResult {
                engine: "ClamAV".to_string(),
                status: ScanStatus::Error,
                threats: vec![],
                error: Some(e),
            });
        }
    }

    // Scan with Windows Defender (Windows only)
    match scan_with_windows_defender(file_path.clone()).await {
        Ok(result) => {
            results.windows_defender = Some(result.clone());
            if matches!(result.status, ScanStatus::Infected) {
                results.overall_status = ScanStatus::Infected;
            } else if matches!(result.status, ScanStatus::Error)
                && matches!(results.overall_status, ScanStatus::Clean)
            {
                results.overall_status = ScanStatus::Error;
            }
        }
        Err(e) => {
            results.windows_defender = Some(ScanResult {
                engine: "Windows Defender".to_string(),
                status: ScanStatus::Error,
                threats: vec![],
                error: Some(e),
            });
        }
    }

    Ok(results)
}
