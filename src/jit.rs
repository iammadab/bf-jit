use crate::{
    jit_utils::{CodeBuilder, allocate_code},
    parser::{Opcode, Program},
};
use std::mem;

fn jit(program: &Program) {
    let mut memory = Box::new([0_u8; 30_000]);
    let code = compile(program, memory.as_mut_ptr());
    let func_ptr = allocate_code(code.as_slice());
    let exec: extern "C" fn() -> () = unsafe { mem::transmute(func_ptr) };
    exec();
}

fn compile(program: &Program, mem_ptr: *mut u8) -> Vec<u8> {
    let mut builder = CodeBuilder::new();
    let mut bracket_stack = vec![];

    // R13 will serve as the data pointer
    // movabs r13, mem_ptr
    builder.emit_bytes(&[0x49, 0xBD]);
    builder.emit_u64(mem_ptr as u64);

    for insn in &program.instructions {
        match insn {
            Opcode::IncPtr(count) => {
                // add r13 imm32
                builder.emit_bytes(&[0x49, 0x81, 0xC5]);
                builder.emit_u32(*count as u32);
            }

            Opcode::DecPtr(count) => {
                // sub r13, imm32
                builder.emit_bytes(&[0x49, 0x81, 0xED]);
                builder.emit_u32(*count as u32);
            }

            Opcode::IncData(count) => {
                // add byte ptr [r13 + 0], imm8
                builder.emit_bytes(&[0x41, 0x80, 0x45, 0x00, *count]);
            }

            Opcode::DecData(count) => {
                // sub byte ptr [r13 + 0], imm8
                builder.emit_bytes(&[0x41, 0x80, 0x6D, 0x00, *count]);
            }

            Opcode::ReadStdin => {
                // mov rax, 0  (SYS_read)
                builder.emit_bytes(&[0x48, 0xC7, 0xC0]);
                builder.emit_u32(0);

                // mov rdi, 0  (stdin)
                builder.emit_bytes(&[0x48, 0xC7, 0xC7]);
                builder.emit_u32(0);

                // mov rsi, r13
                builder.emit_bytes(&[0x4C, 0x89, 0xEE]);

                // mov rdx, 1
                builder.emit_bytes(&[0x48, 0xC7, 0xC2]);
                builder.emit_u32(1);

                // syscall
                builder.emit_bytes(&[0x0F, 0x05]);
            }

            Opcode::WriteStdout => {
                // mov rax, 1  (SYS_write)
                builder.emit_bytes(&[0x48, 0xC7, 0xC0]);
                builder.emit_u32(1);

                // mov rdi, 1  (stdout)
                builder.emit_bytes(&[0x48, 0xC7, 0xC7]);
                builder.emit_u32(1);

                // mov rsi, r13
                builder.emit_bytes(&[0x4C, 0x89, 0xEE]);

                // mov rdx, 1
                builder.emit_bytes(&[0x48, 0xC7, 0xC2]);
                builder.emit_u32(1);

                // syscall
                builder.emit_bytes(&[0x0F, 0x05]);
            }

            Opcode::LoopSetToZero => {
                // zero out the contents of r13
                // mov byte ptr [r13], 0
                builder.emit_bytes(&[0x41, 0xC6, 0x45, 0x00, 0x00]);
            }

            Opcode::LoopMovePtr(stride, positive) => {
                // start:
                //  cmp byte ptr [r13 + 0], 0
                //  jz end
                //  add r13, imm32              # sub if negative stride
                //  jmp start
                // end:
                //  ...

                let start = builder.len();

                // cmp byte ptr [r13 + 0], 0
                builder.emit_bytes(&[0x41, 0x80, 0x7D, 0x00, 0x00]);

                // jz <p1>
                builder.emit_bytes(&[0x0F, 0x84]);
                let jz_p1 = builder.len();
                builder.emit_u32(0); // placeholder

                // move r13 by stride amount
                if *positive {
                    // add r13, imm32
                    builder.emit_bytes(&[0x49, 0x81, 0xC5]);
                    builder.emit_u32(*stride as u32);
                } else {
                    // sub r13, imm32
                    builder.emit_bytes(&[0x49, 0x81, 0xED]);
                    builder.emit_u32(*stride as u32);
                }

                // jump <p2>
                builder.emit_bytes(&[0xE9]);
                let jump_p2 = builder.len();
                builder.emit_u32(0);

                // patch p1
                // p1 should exit the loop (take us to end)
                // we are moving relative to RIP at the end of 'jz end'
                //  this is given by jz_p1 + 4
                // hence p1 = (jump_p2 + 4) - (jz_p1 + 4)

                let end = builder.len();
                let jz_end = jz_p1 + 4;

                let p1 = (end as i64) - (jz_end as i64);
                builder.patch_u32(jz_p1, p1 as i32 as u32);

                // patch p2
                // p2 should go back to the start of the loop
                // this is represented by the start variable
                // all we need to do is calculate the stride from end to start
                let p2 = (start as i64) - (end as i64);
                builder.patch_u32(jump_p2, p2 as i32 as u32);
            }

            Opcode::LoopMoveData(stride, positive) => {
                // start:
                //  cmp byte ptr [r13 + 0], 0
                //  jz end
                //  lea rax, [r13 + imm32]
                //  mov cl, byte ptr [r13 + 0]
                //  add byte ptr [rax] cl
                //  mov byte ptr [r13], 0
                // end:

                // cmp byte ptr [r13 + 0], 0
                builder.emit_bytes(&[0x41, 0x80, 0x7D, 0x00, 0x00]);

                // jz <end>
                builder.emit_bytes(&[0x0F, 0x84]);
                let jz_p1 = builder.len();
                builder.emit_u32(0);

                // compute the signed stride
                let signed_stride = if *positive {
                    *stride as i32
                } else {
                    -(*stride as i32)
                };

                //  lea rax, [r13 + imm32]
                builder.emit_bytes(&[0x49, 0x8D, 0x85]);
                builder.emit_u32(signed_stride as u32);

                // mov cl, byte ptr [r13 + 0]
                builder.emit_bytes(&[0x41, 0x8A, 0x4D, 0x00]);

                // add byte ptr [rax], cl
                builder.emit_bytes(&[0x00, 0x08]);

                // mov byte ptr [r13], 0
                builder.emit_bytes(&[0x41, 0xC6, 0x45, 0x00, 0x00]);

                let end = builder.len();

                // patch jz
                let p1 = (end as i64) - ((jz_p1 + 4) as i64);
                builder.patch_u32(jz_p1, p1 as i32 as u32);
            }

            Opcode::JumpIfDataZero(_) => {
                // cmp byte ptr [r13 + 0], 0
                // jz <matching_bracket>

                // push the branch test RIP to the stack
                bracket_stack.push(builder.len());

                // cmp byte ptr [r13 + 0], 0
                builder.emit_bytes(&[0x41, 0x80, 0x7D, 0x00, 0x00]);

                // jz <p1>
                builder.emit_bytes(&[0x0F, 0x84]);

                // push the patch point to the stack
                bracket_stack.push(builder.len());

                builder.emit_u32(0); // placeholder
            }

            Opcode::JumpIfDataNotZero(_) => {
                // cmp byte ptr [r13 + 0], 0
                // jnz <matching_bracket>

                // cmp byte ptr [r13 + 0], 0
                builder.emit_bytes(&[0x41, 0x80, 0x7D, 0x00, 0x00]);

                // jnz <p2>
                builder.emit_bytes(&[0x0F, 0x85]);
                let jnz_p2 = builder.len();
                builder.emit_u32(0); // patch point

                let end = builder.len();

                let opening_bracket_patch_point = bracket_stack.pop().unwrap();
                let opening_next_rip = opening_bracket_patch_point + 4;

                // calculate the patch value for the opening bracket
                let jump_point = (end as i64) - (opening_next_rip as i64);
                builder.patch_u32(opening_bracket_patch_point, jump_point as i32 as u32);

                // calculate the patch value for closing brakcet
                let opening_start = bracket_stack.pop().unwrap();
                let jump_point = (opening_start as i64) - (end as i64);
                builder.patch_u32(jnz_p2, jump_point as i32 as u32);
            }
        }
    }

    // ret
    builder.emit_bytes(&[0xC3]);

    builder.take_bytes()
}
