// Key Management System
// Manages encryption keys for the crypter

use crate::crypter::encryptor::{EncryptionMethod, EncryptionResult};
use serde::{Deserialize, Serialize};

/// Encryption keys structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKeys {
    pub method: String,
    pub key: Vec<u8>,
    pub iv: Option<Vec<u8>>,
    pub key2: Option<Vec<u8>>, // For multi-layer encryption
}

impl EncryptionKeys {
    /// Create from EncryptionResult
    pub fn from_result(result: &EncryptionResult) -> Self {
        Self {
            method: match result.method {
                EncryptionMethod::AES256 => "aes".to_string(),
                EncryptionMethod::XOR => "xor".to_string(),
                EncryptionMethod::RC4 => "rc4".to_string(),
                EncryptionMethod::Custom => "custom".to_string(),
            },
            key: result.key.clone(),
            iv: result.iv.clone(),
            key2: result.key2.clone(),
        }
    }

    /// Convert to base64 for embedding in stub
    /// Note: Will be used when embedding keys in stub (currently keys are XOR obfuscated)
    #[allow(dead_code)]
    pub fn to_base64(&self) -> String {
        let json = serde_json::to_string(self)
            .expect("Failed to serialize keys");
        // Use base64 crate - check which API version 0.22 uses
        // For 0.22, it should be base64::encode or base64::engine
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.encode(json.as_bytes())
    }

    /// Create from base64
    /// Note: Will be used for key retrieval/deserialization
    #[allow(dead_code)]
    pub fn from_base64(encoded: &str) -> Result<Self, String> {
        use base64::{Engine as _, engine::general_purpose};
        let decoded = general_purpose::STANDARD.decode(encoded)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;
        let json = String::from_utf8(decoded)
            .map_err(|e| format!("Failed to decode UTF-8: {}", e))?;
        serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize keys: {}", e))
    }

    /// Generate random key of specified length
    /// Note: Alternative to Encryptor::generate_key, kept for consistency
    #[allow(dead_code)]
    pub fn generate_key(len: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen()).collect()
    }

    /// Generate random IV (16 bytes for AES)
    /// Note: Alternative to Encryptor::generate_iv, kept for consistency
    #[allow(dead_code)]
    pub fn generate_iv() -> Vec<u8> {
        Self::generate_key(16)
    }
}

/// Key storage (for future use - keys will be embedded in stub)
/// Note: Currently keys are embedded directly, this is for future key management features
#[allow(dead_code)]
pub struct KeyStorage;

impl KeyStorage {
    /// Store keys (currently just returns them - will embed in stub later)
    /// Note: Alternative key storage method, kept for future use
    #[allow(dead_code)]
    pub fn store_keys(keys: &EncryptionKeys) -> String {
        keys.to_base64()
    }

    /// Retrieve keys from storage
    /// Note: Alternative key retrieval method, kept for future use
    #[allow(dead_code)]
    pub fn retrieve_keys(encoded: &str) -> Result<EncryptionKeys, String> {
        EncryptionKeys::from_base64(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypter::encryptor::{Encryptor, EncryptionMethod};

    #[test]
    fn test_key_serialization() {
        let encryptor = Encryptor::new(EncryptionMethod::AES256);
        let data = b"test data";
        let result = encryptor.encrypt_data(data).unwrap();
        
        let keys = EncryptionKeys::from_result(&result);
        let encoded = keys.to_base64();
        let decoded = EncryptionKeys::from_base64(&encoded).unwrap();
        
        assert_eq!(keys.key, decoded.key);
        assert_eq!(keys.iv, decoded.iv);
    }
}
