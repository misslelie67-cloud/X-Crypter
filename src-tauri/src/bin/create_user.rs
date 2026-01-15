use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    chat_id: String,
    key_hash: String,
    expires_at: i64,
    created_at: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let chat_id = "1395416592";
    let days = 30i64;

    println!("🔐 Creating encrypted account for chat_id: {}", chat_id);
    println!("📅 Duration: {} days", days);

    // Encrypt chat_id using aes-gcm
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Key, Nonce,
    };
    use base64::{engine::general_purpose, Engine as _};

    let key_string = std::env::var("ENCRYPTION_KEY")?;
    let key_bytes = general_purpose::STANDARD.decode(&key_string)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, chat_id.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut encrypted_data = nonce_bytes.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);
    let encrypted_chat_id = general_purpose::STANDARD.encode(&encrypted_data);

    // Generate deterministic hash for lookup key
    let mut hasher = Sha256::new();
    hasher.update(chat_id.as_bytes());
    let hash_result = hasher.finalize();
    let chat_id_hash = general_purpose::STANDARD.encode(hash_result);

    println!("🔒 Encrypted chat_id: {}", encrypted_chat_id);
    println!("🔑 Lookup hash: {}", chat_id_hash);

    // Create user object
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + (days * 86400);

    let user = User {
        chat_id: encrypted_chat_id.clone(),
        key_hash: String::new(),
        expires_at,
        created_at: now,
    };

    // Store in Upstash
    let upstash_url = std::env::var("UPSTASH_REDIS_REST_URL")?;
    let upstash_token = std::env::var("UPSTASH_REDIS_REST_TOKEN")?;
    let client = reqwest::Client::new();

    let user_key = format!("user:{}", chat_id_hash);
    let user_json = serde_json::to_string(&user)?;
    let ttl_seconds = days * 86400;

    // URL-encode the key and value for Upstash
    let encoded_key = urlencoding::encode(&user_key);
    let encoded_value = urlencoding::encode(&user_json);

    let url = format!(
        "{}/set/{}/{}/ex/{}",
        upstash_url, encoded_key, encoded_value, ttl_seconds
    );

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", upstash_token))
        .send()
        .await?;

    if response.status().is_success() {
        println!("\n✅ Account created successfully!");
        println!("📋 User Key: {}", user_key);
        println!("📅 Expires: {} days from now", days);
        println!("🔐 All data encrypted");
    } else {
        eprintln!("❌ Failed to create account: {}", response.text().await?);
    }

    Ok(())
}
