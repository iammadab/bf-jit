use std::{default, ptr};

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

struct CodeBuilder {
    bytes: Vec<u8>,
}

impl CodeBuilder {
    fn new() -> Self {
        Self { bytes: vec![] }
    }

    /// Append new bytes to code stream
    fn emit_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.bytes.extend_from_slice(bytes);
        self
    }

    /// Append u32 (as little endian bytes) to the code stream
    fn emit_u32(&mut self, val: u32) -> &mut Self {
        self.bytes.extend_from_slice(val.to_le_bytes().as_slice());
        self
    }
}

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

#[cfg(test)]
mod tests {
    use std::mem::transmute;

    use crate::jit::{CodeBuilder, allocate_code};

    #[test]
    fn test_jit_execution() {
        // I want to JIT compile a function that takes
        // two arguments a and b
        // and returns 2 * ( a + b )
        // fn double_sum(a: i32, b: i32) -> i32 {
        //      2 * (a + b)
        // }

        // based on x86-64 System V ABI
        // a = EDI
        // b = ESI

        // Assembly
        // mov eax, edi
        // add eax, esi
        // shl eax, 1
        // ret

        let mut builder = CodeBuilder::new();

        // mov eax, edi
        // 89 f8
        builder.emit_bytes(&[0x89, 0xf8]);
        // add eax, esi
        // 01 f0
        builder.emit_bytes(&[0x01, 0xf0]);
        // shl eax, 1
        // d1, e0
        builder.emit_bytes(&[0xd1, 0xe0]);
        // ret
        // c3
        builder.emit_bytes(&[0xc3]);

        // cast to fn
        let p = allocate_code(&builder.bytes);
        let double_add: extern "C" fn(i32, i32) -> i32 = unsafe { transmute(p) };

        assert_eq!(double_add(1, 2), 6);
        assert_eq!(double_add(34, 22), 112);
    }
}
