// Update System Module
// Handles remote updates with witness-based approval

pub mod witness;
pub mod download;
pub mod version;
pub mod telegram;
pub mod commands;
pub mod install;

pub use witness::*;
pub use version::*;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Update information from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub file_hash: String,  // SHA256 of update file
    pub file_url: String,   // URL to download update
    pub timestamp: i64,     // Unix timestamp
    pub witness: String,    // HMAC-SHA256 witness
    pub changelog: Option<String>,
    pub size: u64,          // File size in bytes
}

/// Update status
/// Used for internal state tracking and future features
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateStatus {
    Checking,
    Available(UpdateInfo),
    Downloading { progress: f64 },
    WaitingApproval { witness: String },
    Approved { witness: String },
    Rejected { witness: String },
    Installing,
    Installed { version: String },
    Error(String),
    UpToDate,
}

/// Update configuration
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub update_server_url: Option<String>,  // Optional: only needed if not using Upstash
    pub telegram_chat_id: Option<i64>,
    pub check_interval_seconds: u64,  // Update check interval in seconds
    pub witness_secret: String,  // Shared secret for witness generation
}

impl UpdateConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(UpdateConfig {
            update_server_url: std::env::var("UPDATE_SERVER_URL").ok(),
            telegram_chat_id: std::env::var("TELEGRAM_CHAT_ID")
                .ok()
                .and_then(|s| s.parse().ok()),
            check_interval_seconds: std::env::var("UPDATE_CHECK_INTERVAL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(86400), // Default: 24 hours
            witness_secret: std::env::var("WITNESS_SECRET")
                .map_err(|_| "WITNESS_SECRET not set")?,
        })
    }
}

/// Get current app version
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get app data directory for storing updates
pub fn get_update_dir() -> Result<PathBuf, String> {
    // Use temp directory for now (can be changed to app data dir in production)
    let temp_dir = std::env::temp_dir();
    let update_dir = temp_dir.join("x-crypter-updates");
    std::fs::create_dir_all(&update_dir)
        .map_err(|e| format!("Failed to create update directory: {}", e))?;
    Ok(update_dir)
}
