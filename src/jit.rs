// I want to implement jit compilation for brainfuck
// first I'd do hand implementation, then I'd use cranelift
// and potentially LLVM

// let us look into hand implementations
// I have the following opcodes
// IncPtr(u8)
// DecPtr(u8)
// IncData(u8)
// DecData(u8)
// ReadStin
// WriteStdout
// LoopSetToZero
// LoopMovePtr(u8, bool)
// LoopMoveData(u8, bool)
// JumpIfDataZero(usize)
// JumpIfDataNotZero(usize)
//
// I need to replicate the model in an x86 system
// we need contiguous memory (we have this)
// we start at some address (this represents the data pointer address)
// IncPtr and DecPtr will move this address back and forth
// IncData and DecData will update the content of the register
// WriteStdout and ReadStdin should be possible with system V calls
// LoopSetToZero just sets the current data to zero
// LoopMovePtr depending on the bool we either + or -
//  think there should be ways to do branchless compilation here
// LoopMoveData similar to the one above but with the actualy data not the pointer
// JumpIfDataZero
//  we can use a compare then jump appropriately
// JumpIfDataNotZero
//  similar as above

// How do I handle the Jump instructions
// my parser already has the position of the next jump instruction
// but when jumping in assembly the position changes
// I think I can also use a stack

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
        match insn {}
    }

    builder.take_bytes()
}
