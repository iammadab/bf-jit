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

    for insn in program.instructions {
        match insn {
            Opcode::IncPtr(count) => {
                // add r13 imm32
                builder.emit_bytes(&[0x49, 0x81, 0xC5]);
                builder.emit_u32(count as u32);
            }

            Opcode::DecPtr(count) => {
                // sub r13, imm32
                builder.emit_bytes(&[0x49, 0x81, 0xED]);
                builder.emit_u32(count as u32);
            }

            Opcode::IncData(count) => {
                todo!()
            }

            Opcode::DecData(count) => {
                todo!()
            }
        }
    }

    builder.take_bytes()
}
