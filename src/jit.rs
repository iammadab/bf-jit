use std::ptr;

/// JIT Notes
///
/// there are two phases
/// 1. Generate the instruction stream
/// 2. Put the instruction stream in memory and then execute it
///
/// Generate the isntruction stream
/// - multiple ways to do this, but this is essentially compilation
/// - take some representation of something (usually at a higher abstraction level)
///   convert it to another representation (usually at a lower abstraction level)
///
/// Execute the instruction stream
/// - first we need to allocate memory (page-aligned) to hold the instruction stream
///     - initially set to RW permissions (os dependent)
/// - next we copy the instruction stream to the allocated memory
/// - we then change the permissions of allocated range to READ_EXEC (RX)
/// - cast the pointer to a function pointer
/// - perform a function call

/// Stores code bytes in executable memory
/// Return a pointer to this memory segment
fn allocate_code(code: &[u8]) -> *mut libc::c_void {
    // create page aligned memory
    let p = unsafe {
        libc::mmap(
            ptr::null_mut(),
            code.len(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0,
        )
    };

    if p == libc::MAP_FAILED {
        panic!("JIT: mmap allocation failed");
    }

    // copy code into allocated memory
    unsafe {
        ptr::copy_nonoverlapping(code.as_ptr(), p as *mut u8, code.len());
    }

    // change protection to RX
    unsafe {
        if libc::mprotect(p, code.len(), libc::PROT_READ | libc::PROT_EXEC) != 0 {
            // free allocated memory
            libc::munmap(p, code.len());
            panic!("failed to change allocated memory permissions")
        }
    }

    // return pointer to allocated memory
    p
}
