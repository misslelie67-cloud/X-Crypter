// Witness Generation and Verification
// HMAC-SHA256 based witness system for secure updates

use crate::updater::UpdateInfo;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};

type HmacSha256 = Hmac<Sha256>;

/// Generate witness for update info
pub fn generate_witness(
    version: &str,
    file_hash: &str,
    timestamp: i64,
    secret: &str,
) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    
    let message = format!("{}:{}:{}", version, file_hash, timestamp);
    mac.update(message.as_bytes());
    
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Verify witness matches update info
pub fn verify_witness(update_info: &UpdateInfo, secret: &str) -> bool {
    let expected_witness = generate_witness(
        &update_info.version,
        &update_info.file_hash,
        update_info.timestamp,
        secret,
    );
    
    expected_witness == update_info.witness
}

/// Generate witness request for Telegram approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessRequest {
    pub witness: String,
    pub version: String,
    pub file_hash: String,
    pub timestamp: i64,
    pub changelog: Option<String>,
}

impl WitnessRequest {
    pub fn from_update_info(update_info: &UpdateInfo) -> Self {
        WitnessRequest {
            witness: update_info.witness.clone(),
            version: update_info.version.clone(),
            file_hash: update_info.file_hash.clone(),
            timestamp: update_info.timestamp,
            changelog: update_info.changelog.clone(),
        }
    }
    
    /// Format message for Telegram
    pub fn to_telegram_message(&self) -> String {
        let mut msg = format!(
            "🔄 *Update Available*\n\n\
            Version: `{}`\n\
            Hash: `{}`\n\
            Witness: `{}`\n",
            self.version,
            &self.file_hash[..16],  // First 16 chars
            &self.witness[..16]
        );
        
        if let Some(changelog) = &self.changelog {
            msg.push_str(&format!("\n*Changelog:*\n{}", changelog));
        }
        
        msg.push_str("\n\nUse `/approve {}` to approve or `/reject {}` to reject");
        msg
    }
}

/// Check if witness is expired (older than 1 hour)
pub fn is_witness_expired(timestamp: i64) -> bool {
    let now = chrono::Utc::now().timestamp();
    (now - timestamp) > 3600  // 1 hour
}
