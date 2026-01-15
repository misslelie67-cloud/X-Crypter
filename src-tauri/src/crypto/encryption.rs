use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use sha2::{Sha256, Digest};
use std::env;

pub fn encrypt_data(plaintext: &str) -> Result<String, String> {
    let key_string = env::var("ENCRYPTION_KEY")
        .map_err(|_| "ENCRYPTION_KEY not set in environment")?;
    
    let key_bytes = general_purpose::STANDARD
        .decode(&key_string)
        .map_err(|e| format!("Invalid encryption key format: {}", e))?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    // Generate a random nonce (96 bits for AES-GCM)
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Combine nonce + ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(&result))
}

pub fn decrypt_data(encrypted: &str) -> Result<String, String> {
    let key_string = env::var("ENCRYPTION_KEY")
        .map_err(|_| "ENCRYPTION_KEY not set in environment")?;
    
    let key_bytes = general_purpose::STANDARD
        .decode(&key_string)
        .map_err(|e| format!("Invalid encryption key format: {}", e))?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    let encrypted_bytes = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Invalid base64: {}", e))?;
    
    if encrypted_bytes.len() < 12 {
        return Err("Invalid encrypted data length".to_string());
    }
    
    let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
}

pub fn hash_chat_id(chat_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(chat_id.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD.encode(result)
}

#[allow(dead_code)]
pub fn generate_encryption_key() -> String {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    general_purpose::STANDARD.encode(key.as_slice())
}
