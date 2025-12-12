use std::{mem, ptr};

fn execute_jit() {
    // use mmap to create page algined memory (RW)
    // copy code into page
    // use mprotect to change the permissions to (RX)
    // cast to a function and execute

    let code: [u8; 6] = [0xB8, 42, 0x00, 0x00, 0x00, 0xC3];

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
            panic!("failed to ")
        }
    }

    let f: extern "C" fn() -> i32 =
        unsafe { mem::transmute::<*mut libc::c_void, extern "C" fn() -> i32>(p) };

    println!("{}", f());
}

#[cfg(test)]
mod tests {
    use crate::jit::execute_jit;

    #[test]
    fn test_jit_execution() {
        execute_jit();
    }
}
