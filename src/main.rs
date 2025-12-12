use crate::{interpreter::interpret, parser::Program};
use std::fs;

mod interpreter;
mod jit;
mod parser;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let source = fs::read_to_string(&args[1]).unwrap();

    let program = Program::from_source(source);
    interpret(&program);
}

#[cfg(test)]
mod tests {
    #[test]
    fn loop_optimization() {
        let program = Program::from_source(String::from("[-]"));
        assert_eq!(program.instructions.len(), 1);
        assert_eq!(program.instructions[0], Opcode::LoopSetToZero);

        let program = Program::from_source(String::from("[>>]"));
        assert_eq!(program.instructions.len(), 1);
        assert_eq!(program.instructions[0], Opcode::LoopMovePtr(2, true));

        let program = Program::from_source(String::from("[->>>+<<<]"));
        assert_eq!(program.instructions.len(), 1);
        assert_eq!(program.instructions[0], Opcode::LoopMoveData(3, true));

        let program = Program::from_source(String::from(">>>[-<<<<<<+>>>>>>]"));
        assert_eq!(program.instructions.len(), 2);
        assert_eq!(program.instructions[0], Opcode::IncPtr(3));
        assert_eq!(program.instructions[1], Opcode::LoopMoveData(6, false));
    }
}
