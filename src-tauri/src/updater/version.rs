// Version Management
// Handles version checking and comparison

use crate::updater::{get_current_version, UpdateInfo};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Version comparison result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VersionComparison {
    Older,      // Current version is older
    Same,       // Versions are the same
    Newer,      // Current version is newer (shouldn't happen)
}

/// Compare two version strings (semantic versioning)
pub fn compare_versions(current: &str, available: &str) -> VersionComparison {
    let current_parts: Vec<u32> = current
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    let available_parts: Vec<u32> = available
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    
    // Compare major, minor, patch
    for i in 0..3 {
        let current = current_parts.get(i).copied().unwrap_or(0);
        let available = available_parts.get(i).copied().unwrap_or(0);
        
        if available > current {
            return VersionComparison::Older;
        } else if available < current {
            return VersionComparison::Newer;
        }
    }
    
    VersionComparison::Same
}

/// Check if update is needed
pub fn is_update_needed(update_info: &UpdateInfo) -> bool {
    let current = get_current_version();
    matches!(compare_versions(&current, &update_info.version), VersionComparison::Older)
}

/// Get stored version info
pub fn get_stored_version_info() -> Option<UpdateInfo> {
    let update_dir = crate::updater::get_update_dir().ok()?;
    let version_file = update_dir.join("last_update.json");
    
    if !version_file.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&version_file).ok()?;
    serde_json::from_str(&content).ok()
}

/// Store version info after successful update
pub fn store_version_info(update_info: &UpdateInfo) -> Result<(), String> {
    let update_dir = crate::updater::get_update_dir()?;
    let version_file = update_dir.join("last_update.json");
    
    let content = serde_json::to_string_pretty(update_info)
        .map_err(|e| format!("Failed to serialize version info: {}", e))?;
    
    fs::write(&version_file, content)
        .map_err(|e| format!("Failed to write version info: {}", e))?;
    
    Ok(())
}

/// Get update file path for a version
pub fn get_update_file_path(version: &str) -> Result<PathBuf, String> {
    let update_dir = crate::updater::get_update_dir()?;
    Ok(update_dir.join(format!("update_{}.zip", version)))
}
