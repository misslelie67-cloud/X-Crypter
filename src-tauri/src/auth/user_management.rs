use crate::auth::types::User;
use crate::crypto::encrypt_data;
use crate::storage::UpstashClient;
use tauri::State;

#[tauri::command]
pub async fn create_user_account(
    chat_id: String,
    days: i64,
    upstash: State<'_, UpstashClient>,
) -> Result<String, String> {
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + (days * 86400);

    // Encrypt the chat_id
    let encrypted_chat_id = encrypt_data(&chat_id)?;

    let user = User {
        chat_id: encrypted_chat_id,
        key_hash: String::new(), // Will be set during first login
        expires_at,
        created_at: now,
    };

    // Store user with encrypted chat_id as key
    let user_key = format!("user:{}", encrypt_data(&chat_id)?);
    upstash.set_json(&user_key, &user, (days * 86400) as u64).await?;

    Ok(format!(
        "✅ Account created for chat_id: {}\n📅 Expires: {} days from now\n🔐 Data encrypted",
        chat_id, days
    ))
}
