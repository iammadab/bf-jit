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
                // if current value is not zero
                // move the pointer by some stride
                // continue until you hit a zero
                //
                // cmp if r13 is 0
                // if zero then jump (custom address, I think this should be the address for the
                // next instruction)
                // - how do I know what the custom address is going to be?
                // - I could create the bytes separately
                // if not zero then update the data pointer

                // cmp byte ptr [r13 + 0], 0
                // jz <next_insn>
                // add r13 imm32
                // jmp <start>
                // <next_insn>

                // jump computes target as follows
                // next_rip + signed(displacement)

                let start = builder.len();

                // cmp byte ptr [r13 + 0], 0
                builder.emit_bytes(&[0x41, 0x80, 0x7D, 0x00, 0x00]);

                // if zero then jump to next instruction
                // jz <next_insn>
                // we need to figure out what next instruction is
                // this is relative to the size of jz
                // so len after push

                builder.emit_bytes(&[0x0F, 0x84]);
                let disp_pos = builder.len();
                // this value will be patched later
                builder.emit_u32(0);
                let first_patch_rip = builder.len();

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

                // seems we'd have to patch this one also
                builder.emit_bytes(&[0xE9]);
                builder.emit_u32(0);

                let end = builder.len();
                // end - first_patch_rip should give the first patch
                // while end - start should give us the beginning byte

                // jump back to start
                todo!()
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
