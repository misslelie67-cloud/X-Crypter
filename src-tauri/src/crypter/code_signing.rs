// Code Signing
// Signs the compiled executable with a certificate to make it look legitimate

use std::path::PathBuf;
use std::process::Command;

/// Find signtool.exe path on Windows
#[cfg(target_os = "windows")]
fn find_signtool() -> Result<String, String> {
    // Try environment variable first (set by Visual Studio)
    if let Ok(kits_root) = std::env::var("WindowsSdkDir") {
        let signtool = PathBuf::from(kits_root)
            .join("bin")
            .join("x64")
            .join("signtool.exe");
        if signtool.exists() {
            return Ok(signtool.to_string_lossy().to_string());
        }
    }
    
    // Try Program Files paths
    let program_files = std::env::var("ProgramFiles(x86)")
        .or_else(|_| std::env::var("ProgramFiles"))
        .unwrap_or_else(|_| r"C:\Program Files (x86)".to_string());
    
    let signtool_paths = vec![
        format!(r"{}\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe", program_files),
        format!(r"{}\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe", program_files),
        format!(r"{}\Windows Kits\10\bin\10.0.18362.0\x64\signtool.exe", program_files),
        format!(r"{}\Windows Kits\10\bin\x64\signtool.exe", program_files),
        r"C:\Program Files\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe".to_string(),
        r"C:\Program Files\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe".to_string(),
        r"C:\Program Files\Windows Kits\10\bin\x64\signtool.exe".to_string(),
        r"signtool.exe".to_string(), // Try in PATH
    ];
    
    for path in &signtool_paths {
        if path == "signtool.exe" {
            // Check if it's in PATH
            if Command::new("where")
                .arg("signtool.exe")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return Ok("signtool.exe".to_string());
            }
        } else if std::path::Path::new(path).exists() {
            return Ok(path.clone());
        }
    }
    
    Err("signtool.exe not found. Please install Windows SDK or add signtool.exe to PATH.".to_string())
}

/// Sign executable with certificate using signtool.exe
pub fn sign_executable(
    exe_path: &PathBuf,
    certificate_path: &str,
    certificate_password: &str,
) -> Result<(), String> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err("Code signing is only available on Windows. signtool.exe is a Windows SDK tool.".to_string());
    }
    
    #[cfg(target_os = "windows")]
    {
        // Check if executable exists
        if !exe_path.exists() {
            return Err(format!("Executable not found: {:?}", exe_path));
        }
        
        // Check if certificate file exists
        let cert_path = PathBuf::from(certificate_path);
        if !cert_path.exists() {
            return Err(format!("Certificate file not found: {}", certificate_path));
        }
        
        let signtool_path = find_signtool()?;
    
        // Build signtool command
        // signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com /fd sha256 executable.exe
        let output = Command::new(&signtool_path)
            .args(&[
                "sign",
                "/f", certificate_path,
                "/p", certificate_password,
                "/t", "http://timestamp.digicert.com", // Timestamp server
                "/fd", "sha256", // File digest algorithm
                "/tr", "http://timestamp.digicert.com", // RFC 3161 timestamp server
                "/td", "sha256", // Timestamp digest algorithm
            ])
            .arg(exe_path)
            .output()
            .map_err(|e| format!("Failed to execute signtool: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!(
                "Code signing failed:\nSTDOUT: {}\nSTDERR: {}",
                stdout, stderr
            ));
        }
        
        Ok(())
    }
}

/// Verify if executable is signed
pub fn verify_signature(exe_path: &PathBuf) -> Result<bool, String> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err("Signature verification is only available on Windows. signtool.exe is a Windows SDK tool.".to_string());
    }
    
    #[cfg(target_os = "windows")]
    {
        if !exe_path.exists() {
            return Err(format!("Executable not found: {:?}", exe_path));
        }
        
        let signtool_path = find_signtool()?;
        
        // Verify signature
        // signtool verify /pa executable.exe
        let output = Command::new(&signtool_path)
            .args(&["verify", "/pa"])
            .arg(exe_path)
            .output()
            .map_err(|e| format!("Failed to execute signtool verify: {}", e))?;
        
        Ok(output.status.success())
    }
}
