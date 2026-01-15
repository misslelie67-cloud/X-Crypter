// Reflective DLL Loading
// Loads DLLs directly from memory without using LoadLibrary

use crate::api_resolver::resolve_api;
use std::ffi::CString;
use std::ptr;
use winapi::um::errhandlingapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::memoryapi::*;
use winapi::um::winnt::*;

/// Load DLL from memory
pub unsafe fn load_dll_from_memory(dll_data: &[u8]) -> Result<*mut winapi::ctypes::c_void, String> {
    // Step 1: Parse DOS header
    if dll_data.len() < 64 {
        return Err("DLL data too small".to_string());
    }

    let dos_header = dll_data.as_ptr() as *const IMAGE_DOS_HEADER;
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return Err("Invalid DOS signature".to_string());
    }

    // Step 2: Get PE header offset
    let pe_offset = (*dos_header).e_lfanew as usize;
    if pe_offset >= dll_data.len() {
        return Err("Invalid PE offset".to_string());
    }

    // Step 3: Parse PE header
    let nt_headers = dll_data.as_ptr().add(pe_offset) as *const IMAGE_NT_HEADERS64;

    #[cfg(target_arch = "x86")]
    let nt_headers = dll_data.as_ptr().add(pe_offset) as *const IMAGE_NT_HEADERS32;

    if (*nt_headers).Signature != IMAGE_NT_SIGNATURE {
        return Err("Invalid PE signature".to_string());
    }

    // Step 4: Get image size and base
    #[cfg(target_arch = "x86_64")]
    let size_of_image = (*nt_headers).OptionalHeader.SizeOfImage as usize;

    #[cfg(target_arch = "x86")]
    let size_of_image =
        (*(&(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER32)).SizeOfImage as usize;

    #[cfg(target_arch = "x86_64")]
    let image_base = (*nt_headers).OptionalHeader.ImageBase as usize;

    #[cfg(target_arch = "x86")]
    let image_base =
        (*(&(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER32)).ImageBase as usize;

    // Step 5: Allocate memory
    let allocated_base = VirtualAlloc(
        image_base as *mut _,
        size_of_image,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );

    let final_base = if allocated_base.is_null() {
        // Try allocating at any address
        VirtualAlloc(
            ptr::null_mut(),
            size_of_image,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    } else {
        allocated_base
    };

    if final_base.is_null() {
        return Err("Failed to allocate memory for DLL".to_string());
    }

    // Step 6: Copy PE headers
    let header_size = pe_offset + (*nt_headers).FileHeader.SizeOfOptionalHeader as usize + 24;
    ptr::copy_nonoverlapping(
        dll_data.as_ptr(),
        final_base as *mut u8,
        header_size.min(dll_data.len()),
    );

    // Step 7: Copy sections
    let num_sections = (*nt_headers).FileHeader.NumberOfSections as usize;
    let section_header_offset =
        pe_offset + 24 + (*nt_headers).FileHeader.SizeOfOptionalHeader as usize;

    for i in 0..num_sections {
        if section_header_offset + (i + 1) * 40 > dll_data.len() {
            break;
        }

        let section_header =
            dll_data.as_ptr().add(section_header_offset + i * 40) as *const IMAGE_SECTION_HEADER;
        let virtual_address = (*section_header).VirtualAddress as usize;
        let raw_data_ptr = (*section_header).PointerToRawData as usize;
        let raw_data_size = (*section_header).SizeOfRawData as usize;

        if raw_data_ptr + raw_data_size <= dll_data.len() {
            ptr::copy_nonoverlapping(
                dll_data.as_ptr().add(raw_data_ptr),
                (final_base as usize + virtual_address) as *mut u8,
                raw_data_size,
            );
        }
    }

    // Step 8: Resolve imports
    resolve_dll_imports(final_base as *mut u8, nt_headers)?;

    // Step 9: Apply relocations
    apply_dll_relocations(final_base as *mut u8, nt_headers, image_base)?;

    // Step 10: Set section permissions
    set_dll_section_permissions(final_base as *mut u8, nt_headers)?;

    // Step 11: Call DllMain
    #[cfg(target_arch = "x86_64")]
    let entry_point_rva = (*nt_headers).OptionalHeader.AddressOfEntryPoint as usize;

    #[cfg(target_arch = "x86")]
    let entry_point_rva = (*(&(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER32))
        .AddressOfEntryPoint as usize;

    let entry_point = (final_base as usize + entry_point_rva) as *const ();

    type DllMain = extern "system" fn(
        hinst_dll: *mut winapi::ctypes::c_void,
        fdw_reason: u32,
        lpv_reserved: *mut winapi::ctypes::c_void,
    ) -> i32;

    let dll_main: DllMain = std::mem::transmute(entry_point);
    let _ = dll_main(final_base, DLL_PROCESS_ATTACH, ptr::null_mut());

    Ok(final_base)
}

/// Resolve DLL imports
unsafe fn resolve_dll_imports(
    image_base: *mut u8,
    nt_headers: *const IMAGE_NT_HEADERS64,
) -> Result<(), String> {
    #[cfg(target_arch = "x86_64")]
    let import_table_rva = (*nt_headers).OptionalHeader.DataDirectory
        [IMAGE_DIRECTORY_ENTRY_IMPORT.0 as usize]
        .VirtualAddress as usize;

    #[cfg(target_arch = "x86")]
    let import_table_rva = (*(&(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER32))
        .DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT.0 as usize]
        .VirtualAddress as usize;

    if import_table_rva == 0 {
        return Ok(()); // No imports
    }

    let mut import_desc = image_base.add(import_table_rva) as *mut IMAGE_IMPORT_DESCRIPTOR;

    while (*import_desc).Name != 0 {
        let dll_name_ptr = image_base.add((*import_desc).Name as usize) as *const i8;
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
                break;
            }
        }

        let dll_name = CString::new(dll_name_vec).map_err(|_| "Invalid DLL name".to_string())?;

        let module = LoadLibraryA(dll_name.as_ptr() as *const i8);
        if module.is_null() {
            import_desc = import_desc.add(1);
            continue;
        }

        // Resolve functions
        let mut thunk_data =
            image_base.add((*import_desc).u.OriginalFirstThunk as usize) as *mut IMAGE_THUNK_DATA64;
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
                let import_by_name = image_base.add((*thunk_data).u1.AddressOfData as usize)
                    as *const IMAGE_IMPORT_BY_NAME;
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

/// Apply DLL relocations
unsafe fn apply_dll_relocations(
    image_base: *mut u8,
    nt_headers: *const IMAGE_NT_HEADERS64,
    preferred_base: usize,
) -> Result<(), String> {
    #[cfg(target_arch = "x86_64")]
    let reloc_table_rva = (*nt_headers).OptionalHeader.DataDirectory
        [IMAGE_DIRECTORY_ENTRY_BASERELOC.0 as usize]
        .VirtualAddress as usize;

    #[cfg(target_arch = "x86")]
    let reloc_table_rva = (*(&(*nt_headers).OptionalHeader as *const IMAGE_OPTIONAL_HEADER32))
        .DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC.0 as usize]
        .VirtualAddress as usize;

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
        let entries = (reloc_block as *const u16).add(4);

        for i in 0..num_entries {
            let entry = *entries.add(i);
            let reloc_type = entry >> 12;
            let offset = entry & 0xFFF;

            if reloc_type == IMAGE_REL_BASED_DIR64 {
                let reloc_addr = image_base
                    .add((*reloc_block).VirtualAddress as usize + offset as usize)
                    as *mut u64;
                *reloc_addr = (*reloc_addr as usize).wrapping_add(delta) as u64;
            } else if reloc_type == IMAGE_REL_BASED_HIGHLOW {
                let reloc_addr = image_base
                    .add((*reloc_block).VirtualAddress as usize + offset as usize)
                    as *mut u32;
                *reloc_addr = (*reloc_addr as usize).wrapping_add(delta) as u32;
            }
        }

        reloc_block =
            image_base.add((*reloc_block).SizeOfBlock as usize) as *const IMAGE_BASE_RELOCATION;
    }

    Ok(())
}

/// Set DLL section permissions
unsafe fn set_dll_section_permissions(
    image_base: *mut u8,
    nt_headers: *const IMAGE_NT_HEADERS64,
) -> Result<(), String> {
    let section_header_offset = 24 + (*nt_headers).FileHeader.SizeOfOptionalHeader as usize;
    let num_sections = (*nt_headers).FileHeader.NumberOfSections as usize;
    let section_headers =
        (nt_headers as *const u8).add(section_header_offset) as *const IMAGE_SECTION_HEADER;

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
        VirtualProtect(
            section_addr as *mut _,
            virtual_size,
            protection,
            &mut old_protect,
        );
    }

    Ok(())
}
