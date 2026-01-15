// Tauri Commands for Update System

use crate::storage::UpstashClient;
use crate::updater::*;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckUpdateResponse {
    pub status: String,
    pub update_available: bool,
    pub version: Option<String>,
    pub changelog: Option<String>,
    pub error: Option<String>,
    pub witness: Option<String>, // Add witness to response
}

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates(
    upstash: State<'_, UpstashClient>,
) -> Result<CheckUpdateResponse, String> {
    // Try to reload .env file in case it wasn't loaded at startup
    // This helps when running in different contexts
    let _ = dotenvy::dotenv();

    let config = UpdateConfig::from_env().map_err(|e| {
        // Provide helpful error message
        format!("{} - Make sure WITNESS_SECRET is set in your .env file", e)
    })?;
    let current_version = get_current_version();

    // Fetch update info from Upstash (preferred) or external server
    let update_info: UpdateInfo = if let Some(update_info) = upstash.get_update_metadata().await? {
        // Found in Upstash
        update_info
    } else if let Some(server_url) = &config.update_server_url {
        // Fallback to external server if configured
        let client = reqwest::Client::new();
        let update_url = format!("{}/api/updates/latest", server_url);

        let response = match client.get(&update_url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(CheckUpdateResponse {
                    status: "error".to_string(),
                    update_available: false,
                    version: None,
                    changelog: None,
                    error: Some(format!("Failed to check for updates: {}", e)),
                    witness: None,
                });
            }
        };

        if !response.status().is_success() {
            return Ok(CheckUpdateResponse {
                status: "error".to_string(),
                update_available: false,
                version: None,
                changelog: None,
                error: Some(format!("Update server error: {}", response.status())),
                witness: None,
            });
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse update info: {}", e))?
    } else {
        // No update source configured
        return Ok(CheckUpdateResponse {
            status: "up_to_date".to_string(),
            update_available: false,
            version: Some(current_version),
            changelog: None,
            error: None,
            witness: None,
        });
    };

    // Verify witness signature
    if !verify_witness(&update_info, &config.witness_secret) {
        return Ok(CheckUpdateResponse {
            status: "error".to_string(),
            update_available: false,
            version: None,
            changelog: None,
            error: Some("Invalid witness signature".to_string()),
            witness: None,
        });
    }

    // Check if witness is expired (older than 1 hour)
    if is_witness_expired(update_info.timestamp) {
        return Ok(CheckUpdateResponse {
            status: "error".to_string(),
            update_available: false,
            version: None,
            changelog: None,
            error: Some("Update witness has expired. Please check for a newer update.".to_string()),
            witness: None,
        });
    }

    // Check if update is needed
    let update_needed = is_update_needed(&update_info);

    if !update_needed {
        return Ok(CheckUpdateResponse {
            status: "up_to_date".to_string(),
            update_available: false,
            version: Some(current_version),
            changelog: None,
            error: None,
            witness: None,
        });
    }

    // Send beacon to Telegram for approval
    let witness_request = WitnessRequest::from_update_info(&update_info);
    if let Err(e) = telegram::send_update_beacon(&witness_request, &config, &upstash).await {
        return Ok(CheckUpdateResponse {
            status: "error".to_string(),
            update_available: true,
            version: Some(update_info.version.clone()),
            changelog: update_info.changelog.clone(),
            error: Some(format!("Failed to send approval request: {}", e)),
            witness: None,
        });
    }

    Ok(CheckUpdateResponse {
        status: "waiting_approval".to_string(),
        update_available: true,
        version: Some(update_info.version.clone()),
        changelog: update_info.changelog.clone(),
        error: None,
        witness: Some(update_info.witness.clone()),
    })
}

/// Check approval status for a witness
#[tauri::command]
pub async fn check_update_approval(
    witness: String,
    upstash: State<'_, UpstashClient>,
) -> Result<Option<bool>, String> {
    telegram::check_approval_status(&witness, &upstash).await
}

/// Download and install update (after approval)
#[tauri::command]
pub async fn install_update(
    witness: String,
    upstash: State<'_, UpstashClient>,
) -> Result<String, String> {
    let config = UpdateConfig::from_env()?;

    // Verify approval
    let approved = telegram::check_approval_status(&witness, &upstash)
        .await?
        .ok_or("Approval not found")?;

    if !approved {
        return Err("Update not approved".to_string());
    }

    // Fetch update info again from Upstash or external server
    let update_info: UpdateInfo = if let Some(update_info) = upstash.get_update_metadata().await? {
        update_info
    } else if let Some(server_url) = &config.update_server_url {
        let client = reqwest::Client::new();
        let update_url = format!("{}/api/updates/latest", server_url);
        client
            .get(&update_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch update info: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse update info: {}", e))?
    } else {
        return Err("No update source available".to_string());
    };

    // Verify witness matches
    if update_info.witness != witness {
        return Err("Witness mismatch".to_string());
    }

    // Download update
    let update_file = download::download_update(&update_info, &config, None).await?;

    // Extract ZIP file to staging
    let staging_dir = install::extract_update_zip(&update_file)?;

    // Try to apply update immediately (replace crypter modules)
    // If files are locked, we'll mark as pending for next startup
    let apply_result = install::apply_crypter_update(&staging_dir);

    match apply_result {
        Ok(_) => {
            // Successfully applied - store version and clean up
            version::store_version_info(&update_info)?;

            // Clean up staging directory
            if staging_dir.exists() {
                std::fs::remove_dir_all(&staging_dir).ok();
            }

            // Clean up downloaded ZIP
            std::fs::remove_file(&update_file).ok();

            Ok(format!(
                "Update {} installed successfully! The app will restart now.",
                update_info.version
            ))
        }
        Err(e) => {
            // Files might be locked - mark as pending for next startup
            install::mark_update_pending(&update_info)?;

            // Keep ZIP file for next startup
            // Clean up staging directory
            if staging_dir.exists() {
                std::fs::remove_dir_all(&staging_dir).ok();
            }

            Ok(format!(
                "Update {} downloaded. Will be applied on next app restart. Error: {}",
                update_info.version, e
            ))
        }
    }
}

/// Get current app version
#[tauri::command]
pub fn get_app_version() -> String {
    get_current_version()
}

/// Restart the application
#[tauri::command]
pub fn restart_application() -> Result<(), String> {
    install::restart_app()
}

/// Get update check interval in seconds
#[tauri::command]
pub fn get_update_check_interval() -> Result<u64, String> {
    let config = UpdateConfig::from_env()?;
    Ok(config.check_interval_seconds)
}

/// Get stored version info (last installed version)
#[tauri::command]
pub fn get_stored_version() -> Result<Option<UpdateInfo>, String> {
    Ok(version::get_stored_version_info())
}
