// String Obfuscator
// Encrypts string literals and API names to hide them from static analysis

use rand::Rng;

/// Encrypt a string using XOR with a random key
pub fn encrypt_string(s: &str) -> (Vec<u8>, Vec<u8>) {
    let mut rng = rand::thread_rng();
    let key_len = s.len().max(16); // Minimum 16 bytes for key
    let key: Vec<u8> = (0..key_len).map(|_| rng.gen()).collect();
    
    let encrypted: Vec<u8> = s.bytes()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key_len])
        .collect();
    
    (encrypted, key)
}

/// Generate decryption code for an encrypted string
/// Note: Infrastructure for Phase 8 string obfuscation (full implementation requires AST parsing)
#[allow(dead_code)]
pub fn generate_decrypt_code(encrypted_var: &str, key_var: &str, output_var: &str) -> String {
    format!(
        r#"    let {}: Vec<u8> = {}.iter()
        .enumerate()
        .map(|(i, b)| b ^ {}[i % {}.len()])
        .collect();
    let {} = String::from_utf8({}).unwrap_or_default();"#,
        output_var, encrypted_var, key_var, key_var, output_var, output_var
    )
}

/// Encrypt API name and return encrypted bytes and key
/// Note: Infrastructure for Phase 8 - APIs already resolved dynamically
#[allow(dead_code)]
pub fn encrypt_api_name(api_name: &str) -> (Vec<u8>, Vec<u8>) {
    encrypt_string(api_name)
}

/// Generate code to decrypt API name at runtime
/// Note: Infrastructure for Phase 8 - APIs already resolved dynamically
#[allow(dead_code)]
pub fn generate_api_decrypt_code(encrypted_var: &str, key_var: &str, output_var: &str) -> String {
    format!(
        r#"    let {}: Vec<u8> = {}.iter()
        .enumerate()
        .map(|(i, b)| b ^ {}[i % {}.len()])
        .collect();
    let {} = String::from_utf8({}).unwrap_or_default();"#,
        output_var, encrypted_var, key_var, key_var, output_var, output_var
    )
}

/// Generate Rust code for encrypted string constant
/// Note: Infrastructure for Phase 8 string obfuscation (full implementation requires AST parsing)
#[allow(dead_code)]
pub fn generate_encrypted_string_code(var_name: &str, encrypted: &[u8], key: &[u8]) -> (String, String) {
    let encrypted_var = format!("{}_ENC", var_name);
    let key_var = format!("{}_KEY", var_name);
    
    let mut encrypted_code = format!("const {}: &[u8] = &[\n", encrypted_var);
    for chunk in encrypted.chunks(16) {
        encrypted_code.push_str("    ");
        for byte in chunk {
            encrypted_code.push_str(&format!("0x{:02x}, ", byte));
        }
        encrypted_code.push_str("\n");
    }
    encrypted_code.push_str("];\n");
    
    let mut key_code = format!("const {}: &[u8] = &[\n", key_var);
    for chunk in key.chunks(16) {
        key_code.push_str("    ");
        for byte in chunk {
            key_code.push_str(&format!("0x{:02x}, ", byte));
        }
        key_code.push_str("\n");
    }
    key_code.push_str("];\n");
    
    (encrypted_code, key_code)
}

/// Find all string literals in Rust code (simple regex-based approach)
/// Note: Infrastructure for Phase 8 - full implementation requires proper AST parsing
#[allow(dead_code)]
pub fn find_string_literals(code: &str) -> Vec<(usize, usize, String)> {
    // Find string literals like "text" or r#"text"#
    let mut strings = Vec::new();
    let mut in_string = false;
    let mut start = 0;
    let mut current = String::new();
    let mut escape_next = false;
    let mut raw_string = false;
    let mut raw_hashes = 0;
    
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        if escape_next {
            escape_next = false;
            if in_string {
                current.push(ch);
            }
            i += 1;
            continue;
        }
        
        if ch == '\\' && in_string && !raw_string {
            escape_next = true;
            current.push(ch);
            i += 1;
            continue;
        }
        
        if !in_string {
            // Check for raw string start: r#" or r##
            if ch == 'r' && i + 1 < chars.len() {
                let next = chars[i + 1];
                if next == '#' {
                    raw_string = true;
                    raw_hashes = 0;
                    i += 2;
                    // Count # characters
                    while i < chars.len() && chars[i] == '#' {
                        raw_hashes += 1;
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '"' {
                        in_string = true;
                        start = i;
                        current.clear();
                        i += 1;
                        continue;
                    }
                }
            } else if ch == '"' {
                in_string = true;
                start = i;
                current.clear();
                i += 1;
                continue;
            }
        } else {
            if raw_string {
                // Check for raw string end: "# or ##
                if ch == '"' {
                    let mut found_end = true;
                    for j in 0..raw_hashes {
                        if i + 1 + j >= chars.len() || chars[i + 1 + j] != '#' {
                            found_end = false;
                            break;
                        }
                    }
                    if found_end {
                        strings.push((start, i, current.clone()));
                        in_string = false;
                        raw_string = false;
                        raw_hashes = 0;
                        current.clear();
                        i += raw_hashes + 1;
                        continue;
                    }
                }
            } else {
                if ch == '"' {
                    strings.push((start, i, current.clone()));
                    in_string = false;
                    current.clear();
                    i += 1;
                    continue;
                }
            }
            current.push(ch);
        }
        
        i += 1;
    }
    
    strings
}

/// Replace string literals in code with encrypted versions
/// Note: This is a simplified implementation - full version would require proper AST parsing
/// Currently returns code as-is since full string obfuscation requires syn crate for AST parsing
#[allow(dead_code)]
pub fn obfuscate_strings_in_code(code: &str) -> String {
    let strings = find_string_literals(code);
    let mut result = code.to_string();
    
    // Replace from end to start to preserve indices
    for (start, end, original) in strings.into_iter().rev() {
        // Skip if it's a format string or already obfuscated
        if original.contains("{}") || original.contains("{:?}") || original.starts_with("0x") {
            continue;
        }
        
        // Encrypt the string
        let (encrypted, key) = encrypt_string(&original);
        let var_name = format!("STR_{}", rand::thread_rng().gen::<u32>());
        let (_enc_const, _key_const) = generate_encrypted_string_code(&var_name, &encrypted, &key);
        
        // Generate decryption code
        let dec_var = format!("{}_DEC", var_name);
        let _decrypt_code = generate_decrypt_code(
            &format!("{}_ENC", var_name),
            &format!("{}_KEY", var_name),
            &dec_var,
        );
        
        // Replace the string literal with the decrypted variable
        // For now, we'll insert the constants before the function and use the decrypted var
        // This is a simplified version - full implementation would track insertion points
        let replacement = format!("{}", dec_var);
        result.replace_range(start..=end, &replacement);
    }
    
    result
}
