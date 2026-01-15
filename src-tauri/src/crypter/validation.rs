// File Validation Module
// Validates files before encryption

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub file_info: Option<FileInfo>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub size: u64,
    pub is_pe: bool,
    pub is_dll: bool,
    pub is_exe: bool,
    pub architecture: Option<String>,
    pub entry_point: Option<u64>,
}

const MIN_FILE_SIZE: u64 = 64; // Minimum size for a valid PE file
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB max

/// Validate file before encryption
pub fn validate_file(file_path: &str) -> ValidationResult {
    let mut result = ValidationResult {
        valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
        file_info: None,
    };

    // Check if file exists
    let path = PathBuf::from(file_path);
    if !path.exists() {
        result.valid = false;
        result
            .errors
            .push(format!("File does not exist: {}", file_path));
        return result;
    }

    // Check if it's a file (not a directory)
    if !path.is_file() {
        result.valid = false;
        result
            .errors
            .push(format!("Path is not a file: {}", file_path));
        return result;
    }

    // Get file metadata
    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => {
            result.valid = false;
            result
                .errors
                .push(format!("Failed to read file metadata: {}", e));
            return result;
        }
    };

    let file_size = metadata.len();

    // Validate file size
    if file_size < MIN_FILE_SIZE {
        result.valid = false;
        result.errors.push(format!(
            "File too small ({} bytes). Minimum size: {} bytes",
            file_size, MIN_FILE_SIZE
        ));
        return result;
    }

    if file_size > MAX_FILE_SIZE {
        result.valid = false;
        result.errors.push(format!(
            "File too large ({} MB). Maximum size: {} MB",
            file_size / (1024 * 1024),
            MAX_FILE_SIZE / (1024 * 1024)
        ));
        return result;
    }

    // Try to read and validate PE structure
    match validate_pe_structure(file_path) {
        Ok(file_info) => {
            result.file_info = Some(file_info.clone());

            // Additional checks based on file info
            if file_info.is_dll {
                result
                    .warnings
                    .push("File is a DLL. Make sure this is intended.".to_string());
            }

            if file_info.size > 50 * 1024 * 1024 {
                result
                    .warnings
                    .push("Large file detected. Encryption may take longer.".to_string());
            }
        }
        Err(e) => {
            result.valid = false;
            result.errors.push(format!("Invalid PE file: {}", e));
        }
    }

    result
}

/// Validate PE file structure
fn validate_pe_structure(file_path: &str) -> Result<FileInfo, String> {
    let data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    if data.len() < 64 {
        return Err("File too small to contain DOS header".to_string());
    }

    // Check DOS signature (MZ)
    if data[0] != 0x4D || data[1] != 0x5A {
        return Err(
            "Invalid DOS signature. File must be a valid PE executable (MZ signature not found)"
                .to_string(),
        );
    }

    // Get PE header offset
    let pe_offset = u32::from_le_bytes([data[60], data[61], data[62], data[63]]) as usize;

    if pe_offset >= data.len() {
        return Err("PE header offset out of bounds".to_string());
    }

    if pe_offset + 24 > data.len() {
        return Err("File too small to contain PE header".to_string());
    }

    // Check PE signature
    if data[pe_offset] != 0x50
        || data[pe_offset + 1] != 0x45
        || data[pe_offset + 2] != 0x00
        || data[pe_offset + 3] != 0x00
    {
        return Err("Invalid PE signature. File must be a valid PE executable".to_string());
    }

    // Parse machine type
    let machine = u16::from_le_bytes([data[pe_offset + 4], data[pe_offset + 5]]);
    let architecture = match machine {
        0x014c => Some("x86 (32-bit)".to_string()),
        0x8664 => Some("x64 (64-bit)".to_string()),
        0x01c4 => Some("ARM".to_string()),
        0xAA64 => Some("ARM64".to_string()),
        _ => Some(format!("Unknown (0x{:04X})", machine)),
    };

    // Check characteristics
    let characteristics = u16::from_le_bytes([data[pe_offset + 22], data[pe_offset + 23]]);
    let is_dll = (characteristics & 0x2000) != 0; // IMAGE_FILE_DLL
    let is_exe = !is_dll;

    // Try to get entry point (requires optional header)
    let size_of_optional_header =
        u16::from_le_bytes([data[pe_offset + 20], data[pe_offset + 21]]) as usize;
    let entry_point = if pe_offset + 24 + size_of_optional_header <= data.len()
        && size_of_optional_header >= 24
    {
        let optional_header_offset = pe_offset + 24;
        // Entry point is at offset 16 in optional header (for both 32 and 64 bit)
        if optional_header_offset + 16 + 4 <= data.len() {
            Some(u32::from_le_bytes([
                data[optional_header_offset + 16],
                data[optional_header_offset + 17],
                data[optional_header_offset + 18],
                data[optional_header_offset + 19],
            ]) as u64)
        } else {
            None
        }
    } else {
        None
    };

    Ok(FileInfo {
        size: data.len() as u64,
        is_pe: true,
        is_dll,
        is_exe,
        architecture,
        entry_point,
    })
}

/// Validate file extension
pub fn validate_file_extension(file_path: &str) -> Result<(), String> {
    let path = PathBuf::from(file_path);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("exe") | Some("dll") | Some("scr") | Some("sys") | Some("com") => Ok(()),
        Some(ext) => Err(format!(
            "Unsupported file extension: .{}. Supported: .exe, .dll, .scr, .sys, .com",
            ext
        )),
        None => Err("File has no extension. Please use a valid PE executable file.".to_string()),
    }
}

/// Quick validation check (for frontend)
#[allow(dead_code)]
pub fn quick_validate(file_path: &str) -> Result<FileInfo, String> {
    let validation = validate_file(file_path);

    if !validation.valid {
        return Err(validation.errors.join("; "));
    }

    validation
        .file_info
        .ok_or_else(|| "File info not available".to_string())
}
