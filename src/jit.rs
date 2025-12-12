use std::{mem, ptr};

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

fn execute_jit() {
    // mov eax edi
    // ret
    let code: [u8; 3] = [0x89, 0xF8, 0xC3];

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
        panic!("mmap failed");
    }

    // copy code into allocated memory
    unsafe {
        ptr::copy_nonoverlapping(code.as_ptr(), p as *mut u8, code.len());
    }

    // flip protection to RX
    unsafe {
        if libc::mprotect(p, code.len(), libc::PROT_READ | libc::PROT_EXEC) != 0 {
            // free memory
            libc::munmap(p, code.len());
            panic!("failed to change allocated memory permissions")
        }
    }

    // cast to a function pointer
    let f: extern "C" fn(i32) -> i32 = unsafe { mem::transmute(p) };
    println!("{}", f(23232));
}

#[cfg(test)]
mod tests {
    use crate::jit::execute_jit;

    #[test]
    fn test_jit_execution() {
        execute_jit();
    }
}
