// Update Installation
// Handles extracting ZIP files and replacing crypter modules

use crate::updater::{UpdateInfo, get_update_dir};
use std::fs;
use std::path::PathBuf;
use zip::ZipArchive;

/// Extract ZIP file to staging directory
pub fn extract_update_zip(zip_path: &PathBuf) -> Result<PathBuf, String> {
    let update_dir = get_update_dir()?;
    let staging_dir = update_dir.join("staging");
    
    // Clean staging directory if it exists
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)
            .map_err(|e| format!("Failed to clean staging directory: {}", e))?;
    }
    
    fs::create_dir_all(&staging_dir)
        .map_err(|e| format!("Failed to create staging directory: {}", e))?;
    
    // Open ZIP file
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
    
    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read file {} from ZIP: {}", i, e))?;
        
        let file_name = file.name();
        let outpath = staging_dir.join(file_name);
        
        // Skip if it's a directory (ends with /)
        if file_name.ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
            continue;
        }
        
        // Create parent directories if needed
        if let Some(p) = outpath.parent() {
            if !p.exists() {
                fs::create_dir_all(p)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }
        
        // Extract file
        let mut outfile = fs::File::create(&outpath)
            .map_err(|e| format!("Failed to create file {:?}: {}", outpath, e))?;
        
        std::io::copy(&mut file, &mut outfile)
            .map_err(|e| format!("Failed to extract file {:?}: {}", outpath, e))?;
    }
    
    Ok(staging_dir)
}

/// Replace crypter modules with new ones from staging
pub fn apply_crypter_update(staging_dir: &PathBuf) -> Result<(), String> {
    // Try to find project root
    let project_root = find_project_root()?;
    let crypter_dir = project_root.join("src-tauri").join("src").join("crypter");
    
    if !crypter_dir.exists() {
        return Err("Crypter directory not found".to_string());
    }
    
    // Source directory in staging (should be src-tauri/src/crypter/)
    let staging_crypter = staging_dir.join("src-tauri").join("src").join("crypter");
    
    if !staging_crypter.exists() {
        return Err("Staging crypter directory not found in update".to_string());
    }
    
    // List of files to replace
    let files_to_replace = vec![
        "mod.rs",
        "encryptor.rs",
        "pe_reader.rs",
        "stub_gen.rs",
        "mutator.rs",
        "key_manager.rs",
        "code_signing.rs",
        "resource_manipulation.rs",
        "string_obfuscator.rs",
        "obfuscator.rs",
    ];
    
    // Backup existing files first
    let backup_dir = crypter_dir.join("backup");
    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to remove old backup: {}", e))?;
    }
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    
    // Backup and replace files
    for filename in &files_to_replace {
        let source_file = staging_crypter.join(filename);
        let target_file = crypter_dir.join(filename);
        let backup_file = backup_dir.join(filename);
        
        // Backup existing file if it exists
        if target_file.exists() {
            fs::copy(&target_file, &backup_file)
                .map_err(|e| format!("Failed to backup {}: {}", filename, e))?;
        }
        
        // Replace with new file
        if source_file.exists() {
            fs::copy(&source_file, &target_file)
                .map_err(|e| format!("Failed to replace {}: {}", filename, e))?;
        }
    }
    
    Ok(())
}

/// Find project root directory
fn find_project_root() -> Result<PathBuf, String> {
    // Try multiple strategies to find project root
    
    // Strategy 1: Look for Cargo.toml in current directory or parents
    let mut current = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if it's the Tauri project (has src-tauri subdirectory)
            let src_tauri = current.join("src-tauri");
            if src_tauri.exists() {
                return Ok(current);
            }
        }
        
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    
    // Strategy 2: Use executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    // Try to find project root from executable location
    // (executable is typically in target/release/ or similar)
    let mut current = exe_path.parent().unwrap().to_path_buf();
    
    loop {
        let cargo_toml = current.join("Cargo.toml");
        let src_tauri = current.join("src-tauri");
        if cargo_toml.exists() && src_tauri.exists() {
            return Ok(current);
        }
        
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    
    Err("Could not find project root directory".to_string())
}

/// Mark update as pending (to be applied on next startup)
pub fn mark_update_pending(update_info: &UpdateInfo) -> Result<(), String> {
    let update_dir = get_update_dir()?;
    let pending_file = update_dir.join("pending_update.json");
    
    let content = serde_json::to_string_pretty(update_info)
        .map_err(|e| format!("Failed to serialize update info: {}", e))?;
    
    fs::write(&pending_file, content)
        .map_err(|e| format!("Failed to write pending update: {}", e))?;
    
    Ok(())
}

/// Check for pending update on startup
pub fn check_pending_update() -> Result<Option<UpdateInfo>, String> {
    let update_dir = get_update_dir()?;
    let pending_file = update_dir.join("pending_update.json");
    
    if !pending_file.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(&pending_file)
        .map_err(|e| format!("Failed to read pending update: {}", e))?;
    
    let update_info: UpdateInfo = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse pending update: {}", e))?;
    
    Ok(Some(update_info))
}

/// Apply pending update (called on app startup)
pub fn apply_pending_update() -> Result<(), String> {
    let update_info = match check_pending_update()? {
        Some(info) => info,
        None => return Ok(()), // No pending update
    };
    
    // Get update file path
    let update_file = crate::updater::version::get_update_file_path(&update_info.version)?;
    
    if !update_file.exists() {
        // Update file not found, clear pending
        clear_pending_update()?;
        return Err("Update file not found".to_string());
    }
    
    // Extract ZIP
    let staging_dir = extract_update_zip(&update_file)?;
    
    // Apply update
    apply_crypter_update(&staging_dir)?;
    
    // Store version info
    crate::updater::version::store_version_info(&update_info)?;
    
    // Clear pending update
    clear_pending_update()?;
    
    // Clean up staging directory
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir).ok();
    }
    
    Ok(())
}

/// Clear pending update marker
pub fn clear_pending_update() -> Result<(), String> {
    let update_dir = get_update_dir()?;
    let pending_file = update_dir.join("pending_update.json");
    
    if pending_file.exists() {
        fs::remove_file(&pending_file)
            .map_err(|e| format!("Failed to remove pending update: {}", e))?;
    }
    
    Ok(())
}

/// Restart the application
pub fn restart_app() -> Result<(), String> {
    use std::process::Command;
    
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    #[cfg(target_os = "windows")]
    {
        // On Windows, use cmd to start new instance and exit
        let exe_str = exe_path.to_str()
            .ok_or("Failed to convert path to string")?;
        Command::new("cmd")
            .args(&["/C", "start", "", exe_str])
            .spawn()
            .map_err(|e| format!("Failed to start new instance: {}", e))?;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On Unix-like systems, spawn new instance
        Command::new(&exe_path)
            .spawn()
            .map_err(|e| format!("Failed to start new instance: {}", e))?;
    }
    
    // Give it a moment to start, then exit
    std::thread::sleep(std::time::Duration::from_millis(500));
    std::process::exit(0);
}
