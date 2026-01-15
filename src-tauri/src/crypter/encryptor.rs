// X-Crypter Encryption Engine
// Supports multiple encryption algorithms for PE file encryption

use aes::Aes256;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use cbc::Encryptor as CbcEncryptor;
use rand::Rng;
use std::fs;

/// Encryption methods supported by the crypter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMethod {
    AES256,
    XOR,
    RC4,
    Custom,
}

/// Encryption result containing encrypted data and keys
#[derive(Debug, Clone)]
pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub key: Vec<u8>,
    pub iv: Option<Vec<u8>>,
    pub key2: Option<Vec<u8>>, // For multi-layer encryption
    pub method: EncryptionMethod,
}

/// Main encryptor struct
pub struct Encryptor {
    method: EncryptionMethod,
}

impl Encryptor {
    /// Create a new encryptor with specified method
    pub fn new(method: EncryptionMethod) -> Self {
        Self { method }
    }

    /// Generate random key of specified length
    pub fn generate_key(len: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen()).collect()
    }

    /// Generate random IV (16 bytes for AES)
    pub fn generate_iv() -> Vec<u8> {
        Self::generate_key(16)
    }

    /// Encrypt file from disk
    /// Note: Alternative to encrypt_data, kept for convenience
    #[allow(dead_code)]
    pub fn encrypt_file(&self, file_path: &str) -> Result<EncryptionResult, String> {
        let data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        self.encrypt_data(&data)
    }

    /// Encrypt data in memory
    pub fn encrypt_data(&self, data: &[u8]) -> Result<EncryptionResult, String> {
        match self.method {
            EncryptionMethod::AES256 => self.encrypt_aes256(data),
            EncryptionMethod::XOR => self.encrypt_xor(data),
            EncryptionMethod::RC4 => self.encrypt_rc4(data),
            EncryptionMethod::Custom => self.encrypt_custom(data),
        }
    }

    /// AES-256-CBC encryption
    fn encrypt_aes256(&self, data: &[u8]) -> Result<EncryptionResult, String> {
        let key = Self::generate_key(32); // 256-bit key
        let iv = Self::generate_iv(); // 128-bit IV

        // Create encryptor
        let cipher = CbcEncryptor::<Aes256>::new_from_slices(&key, &iv)
            .map_err(|e| format!("Failed to create AES encryptor: {}", e))?;

        // Encrypt (CBC mode requires padding)
        let encrypted = cipher.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(data);

        Ok(EncryptionResult {
            encrypted_data: encrypted,
            key,
            iv: Some(iv),
            key2: None,
            method: EncryptionMethod::AES256,
        })
    }

    /// XOR encryption (symmetric)
    fn encrypt_xor(&self, data: &[u8]) -> Result<EncryptionResult, String> {
        // Generate random key (32 bytes)
        let key = Self::generate_key(32);

        let mut encrypted = Vec::with_capacity(data.len());
        let key_len = key.len();

        for (i, byte) in data.iter().enumerate() {
            encrypted.push(byte ^ key[i % key_len]);
        }

        Ok(EncryptionResult {
            encrypted_data: encrypted,
            key,
            iv: None,
            key2: None,
            method: EncryptionMethod::XOR,
        })
    }

    /// RC4 stream cipher encryption
    fn encrypt_rc4(&self, data: &[u8]) -> Result<EncryptionResult, String> {
        // Generate random key (16 bytes for RC4)
        let key = Self::generate_key(16);

        let encrypted = rc4_encrypt(data, &key);

        Ok(EncryptionResult {
            encrypted_data: encrypted,
            key,
            iv: None,
            key2: None,
            method: EncryptionMethod::RC4,
        })
    }

    /// Custom multi-layer encryption
    /// Layer 1: XOR
    /// Layer 2: AES-256
    /// Layer 3: RC4
    fn encrypt_custom(&self, data: &[u8]) -> Result<EncryptionResult, String> {
        // Layer 1: XOR
        let key1 = Self::generate_key(32);
        let xor_encrypted: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, byte)| byte ^ key1[i % key1.len()])
            .collect();

        // Layer 2: AES-256
        let key2 = Self::generate_key(32);
        let iv = Self::generate_iv();
        let cipher = CbcEncryptor::<Aes256>::new_from_slices(&key2, &iv)
            .map_err(|e| format!("Failed to create AES encryptor: {}", e))?;
        let aes_encrypted =
            cipher.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(&xor_encrypted);

        // Layer 3: RC4
        let key3 = Self::generate_key(16);
        let final_encrypted = rc4_encrypt(&aes_encrypted, &key3);

        Ok(EncryptionResult {
            encrypted_data: final_encrypted,
            key: key1,                         // Primary key (XOR)
            iv: Some(iv),                      // AES IV
            key2: Some([key2, key3].concat()), // Combined AES + RC4 keys
            method: EncryptionMethod::Custom,
        })
    }
}

/// RC4 Key Scheduling Algorithm (KSA)
fn rc4_ksa(key: &[u8]) -> [u8; 256] {
    let mut s = [0u8; 256];
    for i in 0..256 {
        s[i] = i as u8;
    }

    let mut j = 0u8;
    for i in 0..256 {
        j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
        s.swap(i, j as usize);
    }

    s
}

/// RC4 Pseudo-Random Generation Algorithm (PRGA)
fn rc4_prga(s: &mut [u8; 256], data_len: usize) -> Vec<u8> {
    let mut keystream = Vec::with_capacity(data_len);
    let mut i = 0u8;
    let mut j = 0u8;

    for _ in 0..data_len {
        i = i.wrapping_add(1);
        j = j.wrapping_add(s[i as usize]);
        s.swap(i as usize, j as usize);
        let k = s[(s[i as usize].wrapping_add(s[j as usize])) as usize];
        keystream.push(k);
    }

    keystream
}

/// RC4 encryption/decryption (symmetric)
fn rc4_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mut s = rc4_ksa(key);
    let keystream = rc4_prga(&mut s, data.len());

    data.iter()
        .zip(keystream.iter())
        .map(|(d, k)| d ^ k)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_encryption() {
        let encryptor = Encryptor::new(EncryptionMethod::XOR);
        let data = b"Hello, World!";

        let result = encryptor.encrypt_data(data).unwrap();
        assert_ne!(data, result.encrypted_data.as_slice());
        assert_eq!(result.key.len(), 32);
    }

    #[test]
    fn test_rc4_encryption() {
        let encryptor = Encryptor::new(EncryptionMethod::RC4);
        let data = b"Test data for RC4";

        let result = encryptor.encrypt_data(data).unwrap();
        assert_ne!(data, result.encrypted_data.as_slice());
    }

    #[test]
    fn test_aes256_encryption() {
        let encryptor = Encryptor::new(EncryptionMethod::AES256);
        let data = b"Test data for AES-256 encryption";

        let result = encryptor.encrypt_data(data).unwrap();
        assert_ne!(data, result.encrypted_data.as_slice());
        assert!(result.iv.is_some());
    }

    #[test]
    fn test_custom_encryption() {
        let encryptor = Encryptor::new(EncryptionMethod::Custom);
        let data = b"Test data for custom multi-layer encryption";

        let result = encryptor.encrypt_data(data).unwrap();
        assert_ne!(data, result.encrypted_data.as_slice());
        assert!(result.key2.is_some());
    }
}
