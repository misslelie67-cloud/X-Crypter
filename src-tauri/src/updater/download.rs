// Secure Update Download
// Handles downloading and verifying update files

use crate::updater::{UpdateInfo, UpdateConfig};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use reqwest::Client;

/// Download update file securely
pub async fn download_update(
    update_info: &UpdateInfo,
    _config: &UpdateConfig,
    progress_callback: Option<Box<dyn Fn(f64) + Send>>,
) -> Result<PathBuf, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))  // 5 minute timeout
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Get update file path
    let update_file = crate::updater::version::get_update_file_path(&update_info.version)?;
    
    // Download file
    let response = client
        .get(&update_info.file_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Update server returned error: {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    let mut file = fs::File::create(&update_file)
        .map_err(|e| format!("Failed to create update file: {}", e))?;
    
    let mut downloaded: u64 = 0;
    let mut hasher = Sha256::new();
    let mut stream = response.bytes_stream();
    
    use futures_util::StreamExt;
    while let Some(item) = stream.next().await {
        let chunk: Result<bytes::Bytes, reqwest::Error> = item;
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        
        // Write to file
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write update file: {}", e))?;
        
        // Update hash
        hasher.update(&chunk);
        
        // Update progress
        downloaded += chunk.len() as u64;
        if let Some(callback) = &progress_callback {
            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                callback(progress);
            }
        }
    }
    
    // Verify file hash
    let computed_hash = hex::encode(hasher.finalize());
    if computed_hash != update_info.file_hash {
        fs::remove_file(&update_file).ok();
        return Err(format!(
            "File hash mismatch! Expected: {}, Got: {}",
            update_info.file_hash, computed_hash
        ));
    }
    
    Ok(update_file)
}

/// Verify downloaded file integrity
/// Utility function for manual verification
#[allow(dead_code)]
pub fn verify_file_hash(file_path: &PathBuf, expected_hash: &str) -> Result<bool, String> {
    let file_data = fs::read(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let computed_hash = hex::encode(hasher.finalize());
    
    Ok(computed_hash == expected_hash)
}
