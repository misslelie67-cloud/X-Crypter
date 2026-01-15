// Telegram Integration for Update Approvals
// Sends update beacons and handles approval responses

use crate::updater::{WitnessRequest, UpdateConfig};
use crate::storage::UpstashClient;
use serde::{Deserialize, Serialize};
use reqwest::Client;

const APPROVAL_TTL_SECS: u64 = 3600; // 1 hour

/// Approval status stored in Upstash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStatus {
    pub witness: String,
    pub approved: bool,
    pub timestamp: i64,
    pub version: String,
}

/// Send update beacon to Telegram
pub async fn send_update_beacon(
    witness_request: &WitnessRequest,
    config: &UpdateConfig,
    upstash: &UpstashClient,
) -> Result<(), String> {
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .map_err(|_| "TELEGRAM_BOT_TOKEN not set")?;
    
    let chat_id = config.telegram_chat_id
        .ok_or("TELEGRAM_CHAT_ID not set")?;
    
    // Store approval request in Upstash
    let approval_key = format!("update_approval:{}", witness_request.witness);
    let approval_status = ApprovalStatus {
        witness: witness_request.witness.clone(),
        approved: false,
        timestamp: chrono::Utc::now().timestamp(),
        version: witness_request.version.clone(),
    };
    
    upstash.set_json(&approval_key, &approval_status, APPROVAL_TTL_SECS).await?;
    
    // Send message to Telegram
    let client = Client::new();
    let message = witness_request.to_telegram_message();
    let api_url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    
    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": message,
        "parse_mode": "Markdown",
        "reply_markup": {
            "inline_keyboard": [[
                {
                    "text": "✅ Approve",
                    "callback_data": format!("approve_update:{}", witness_request.witness)
                },
                {
                    "text": "❌ Reject",
                    "callback_data": format!("reject_update:{}", witness_request.witness)
                }
            ]]
        }
    });
    
    let response = client
        .post(&api_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send Telegram message: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Telegram API error: {}", error_text));
    }
    
    Ok(())
}

/// Check if witness is approved
pub async fn check_approval_status(
    witness: &str,
    upstash: &UpstashClient,
) -> Result<Option<bool>, String> {
    let approval_key = format!("update_approval:{}", witness);
    let status: Option<ApprovalStatus> = upstash.get_json(&approval_key).await?;
    
    Ok(status.map(|s| s.approved))
}

/// Poll for approval (with timeout)
/// Utility function for future blocking approval checks
#[allow(dead_code)]
pub async fn wait_for_approval(
    witness: &str,
    timeout_seconds: u64,
    upstash: &UpstashClient,
) -> Result<bool, String> {
    let start = std::time::Instant::now();
    let poll_interval = std::time::Duration::from_secs(5);
    let timeout = std::time::Duration::from_secs(timeout_seconds);
    
    loop {
        if start.elapsed() > timeout {
            return Err("Approval timeout".to_string());
        }
        
        if let Some(approved) = check_approval_status(witness, upstash).await? {
            return Ok(approved);
        }
        
        tokio::time::sleep(poll_interval).await;
    }
}
