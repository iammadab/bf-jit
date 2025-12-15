use crate::{
    jit_utils::CodeBuilder,
    parser::{Opcode, Program},
};

fn jit(program: &Program) {
    let memory = [0_u8; 30_000];
    compile(program, memory.as_ptr());
}
fn compile(program: &Program, mem_ptr: *const u8) -> Vec<u8> {
    let mut builder = CodeBuilder::new();

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
                todo!()
            }

            Opcode::WriteStdout => {
                todo!()
            }

            Opcode::LoopSetToZero => {
                // xor r13, r13
                builder.emit_bytes(&[0x4D, 0x31, 0xED]);
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
                todo!()
            }

            Opcode::JumpIfDataZero(_) => {
                todo!()
            }

            Opcode::JumpIfDataNotZero(_) => {
                todo!()
            }
        }
    }

    builder.take_bytes()
}
