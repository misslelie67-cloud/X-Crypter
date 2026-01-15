use aes_gcm::{aead::OsRng, Aes256Gcm, KeyInit};
use base64::{engine::general_purpose, Engine as _};

fn main() {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let encoded = general_purpose::STANDARD.encode(key.as_slice());
    println!("\n🔐 Generated AES-256 Encryption Key:\n{}\n", encoded);
    println!("Add this to your .env file:");
    println!("ENCRYPTION_KEY={}\n", encoded);
}
