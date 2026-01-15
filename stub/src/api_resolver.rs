// API Resolver
// Resolves Windows APIs dynamically to avoid static imports

use winapi::um::libloaderapi::*;
use winapi::um::winnt::*;
use winapi::shared::minwindef::*;
use std::ptr;
use std::ffi::CString;

/// Hash function for API names (djb2 variant)
pub fn hash_api_name(name: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in name.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
}

/// Get module base address
pub unsafe fn get_module_base(module_name: &str) -> *mut u8 {
    let name_cstr = CString::new(module_name).unwrap();
    let handle = GetModuleHandleA(name_cstr.as_ptr() as *const i8);
    if handle.is_null() {
        // Try loading it
        let handle = LoadLibraryA(name_cstr.as_ptr() as *const i8);
        if handle.is_null() {
            return ptr::null_mut();
        }
    }
    handle as *mut u8
}

/// Parse PE export table to find function
pub unsafe fn get_export_address(
    module_base: *mut u8,
    function_name: &str,
) -> Option<*mut winapi::ctypes::c_void> {
    if module_base.is_null() {
        return None;
    }

    // Parse DOS header
    let dos_header = module_base as *const IMAGE_DOS_HEADER;
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return None;
    }

    // Get PE header offset
    let pe_offset = (*dos_header).e_lfanew as usize;
    let nt_headers = module_base.add(pe_offset) as *const IMAGE_NT_HEADERS64;
    
    #[cfg(target_arch = "x86")]
    let nt_headers = module_base.add(pe_offset) as *const IMAGE_NT_HEADERS32;

    // Check PE signature
    if (*nt_headers).Signature != IMAGE_NT_SIGNATURE {
        return None;
    }

    // Get optional header
    #[cfg(target_arch = "x86_64")]
    let optional_header = &(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER64 as *const winapi::ctypes::c_void;
    
    #[cfg(target_arch = "x86")]
    let optional_header = &(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER32 as *const winapi::ctypes::c_void;

    // Get export table RVA
    #[cfg(target_arch = "x86_64")]
    let export_rva = (*(optional_header as *const IMAGE_OPTIONAL_HEADER64))
        .DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT.0 as usize]
        .VirtualAddress as usize;
    
    #[cfg(target_arch = "x86")]
    let export_rva = (*(optional_header as *const IMAGE_OPTIONAL_HEADER32))
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

    // Search for function by name
    for i in 0..num_names {
        let name_ptr = module_base.add(*(module_base.add(name_rva) as *const u32).add(i) as usize) as *const i8;
        
        // Compare strings
        let mut j = 0;
        let func_name_bytes = function_name.as_bytes();
        loop {
            let name_byte = *name_ptr.add(j);
            if name_byte == 0 && j >= func_name_bytes.len() {
                // Found match
                let ordinal = *(module_base.add(name_ordinals_rva) as *const u16).add(i) as usize;
                let func_rva = *(module_base.add(functions_rva) as *const u32).add(ordinal) as usize;
                return Some(module_base.add(func_rva) as *mut winapi::ctypes::c_void);
            }
            if j >= func_name_bytes.len() || name_byte != func_name_bytes[j] as i8 {
                break;
            }
            j += 1;
        }
    }

    None
}

/// Resolve API by name
pub unsafe fn resolve_api(module_name: &str, function_name: &str) -> Option<*mut winapi::ctypes::c_void> {
    let module_base = get_module_base(module_name);
    if module_base.is_null() {
        return None;
    }
    get_export_address(module_base, function_name)
}

/// Resolve API by hash
pub unsafe fn resolve_api_by_hash(module_name: &str, function_hash: u32) -> Option<*mut winapi::ctypes::c_void> {
    let module_base = get_module_base(module_name);
    if module_base.is_null() {
        return None;
    }

    // Parse export table
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

    // Search by hash
    for i in 0..num_names {
        let name_ptr = module_base.add(*(module_base.add(name_rva) as *const u32).add(i) as usize) as *const i8;
        
        // Build function name string
        let mut name_bytes = Vec::new();
        let mut j = 0;
        loop {
            let byte = *name_ptr.add(j);
            if byte == 0 {
                break;
            }
            name_bytes.push(byte as u8);
            j += 1;
            if j > 255 {
                break;
            }
        }
        
        if let Ok(name_str) = String::from_utf8(name_bytes) {
            let hash = hash_api_name(&name_str);
            if hash == function_hash {
                // Found match
                let ordinal = *(module_base.add(name_ordinals_rva) as *const u16).add(i) as usize;
                let func_rva = *(module_base.add(functions_rva) as *const u32).add(ordinal) as usize;
                return Some(module_base.add(func_rva) as *mut winapi::ctypes::c_void);
            }
        }
    }

    None
}
